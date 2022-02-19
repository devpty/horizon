//! core rendering and state logic
use std::{collections, iter};
use winit::{dpi, event, window};
use wgpu::util::DeviceExt;
use crate::debugger;
use crate::ecs::{Entity, Component, UpdateInfo};
use crate::egui_util::EguiComponent;
use crate::render::{RenderContext, ResizeBuffer};

/// wgpu::VertexBufferLayout that owns it's attributes
struct FakeVertexBufferLayout {
	array_stride: wgpu::BufferAddress,
	attributes: Box<[wgpu::VertexAttribute]>,
	step_mode: wgpu::VertexStepMode,
}

macro_rules! fake_vertex_attr_array {
	( $type:ty, $step:tt, $( $loc:tt => $data:tt ),* $(,)?) => {
		FakeVertexBufferLayout::new::<$type>(
			Box::new(wgpu::vertex_attr_array![$($loc => $data),*]),
			wgpu::VertexStepMode::$step
		)
	}
}

impl FakeVertexBufferLayout {
	fn new<T>(
		attributes: Box<[wgpu::VertexAttribute]>,
		step_mode: wgpu::VertexStepMode
	) -> Self {
		FakeVertexBufferLayout {
			attributes, step_mode,
			array_stride: std::mem::size_of::<T>() as wgpu::BufferAddress
		}
	}
	fn real(&self) -> wgpu::VertexBufferLayout {
		wgpu::VertexBufferLayout {
			array_stride: self.array_stride,
			step_mode: self.step_mode,
			attributes: self.attributes.as_ref(),
		}
	}
}

trait CanBuffer {
	fn desc() -> FakeVertexBufferLayout;
}

/// all the uniform data that's used, 16-bit values don't work here :(
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct WorldUniform {
	offset: [f32; 2],
	size: [f32; 2],
	atlas_size: [f32; 2],
}

impl WorldUniform {
	fn update_screen_size(&mut self, width: u32, height: u32, integer: bool, offset: [f64; 2]) {
		let target_width = 640u32;
		let target_height = 480u32;
		// determine scaling factor
		let scaling_factor = (width as f64 / target_width as f64)
			.min(height as f64 / target_height as f64);
		// optionally floor it
		let int_scaling_factor = if integer && scaling_factor >= 1.0 {
			scaling_factor.floor()
		} else { scaling_factor };
		// center it
		let scaled_width = int_scaling_factor * target_width as f64;
		let scaled_height = int_scaling_factor * target_height as f64;
		self.size = [
			(int_scaling_factor / width as f64) as f32,
			(int_scaling_factor / height as f64) as f32
		];
		self.offset = [
			((width as f64 - scaled_width) * offset[0] / int_scaling_factor) as f32,
			((height as f64 - scaled_height) * offset[1] / int_scaling_factor) as f32,
		];
		log::trace!("WorldUniform = {:?}", self);
	}
	fn update_atlas_size(&mut self, width: u32, height: u32) {
		self.atlas_size = [width as f32, height as f32];
	}
}

/// vertex data
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	pub pos: [f32; 2],
	pub uv:  [f32; 2],
	pub col: [u8; 4],
}

impl CanBuffer for Vertex {
	fn desc() -> FakeVertexBufferLayout {
		fake_vertex_attr_array!(Self, Vertex,
			0 => Float32x2, // pos
			1 => Float32x2, // uv
			2 => Unorm8x4, // col
		)
	}
}

/// global state info
pub struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: dpi::PhysicalSize<u32>,
	target_size: dpi::PhysicalSize<u32>,
	render_pipeline: wgpu::RenderPipeline,
	egui_platform: egui_winit_platform::Platform,
	egui_rpass: egui_wgpu_backend::RenderPass,
	egui_state: debugger::DebuggerState,
	world_uniform: WorldUniform,
	world_uniform_buffer: wgpu::Buffer,
	world_uniform_bind_group: wgpu::BindGroup,
	start_info: crate::StartInfo,
	atlas_bind_group: wgpu::BindGroup,
	pressed_keys: collections::HashSet<u32>,
	vertex_buffer: ResizeBuffer<Vertex>,
	index_buffer: ResizeBuffer<[u16; 3]>,
	root_entity: Entity,
	dm_screen_offset: [f64; 2],
}

impl State {
	// we *might* have to make this as a "init" rather than "new" if some of the
	// settings in here are worth changing and require reconstruction
	pub async fn new(
		window: &window::Window, start_info: crate::StartInfo
	) -> Self {
		let size = window.inner_size();
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		// SAFETY: it is
		let surface = unsafe { instance.create_surface(window) };
		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.expect("failed to adpater");
		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::default(),
				label: None,
			},
			None
		).await.expect("failed to device");
		let surface_format = surface.get_preferred_format(&adapter).unwrap();
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface_format,
			width: size.width, height: size.height,
			// the three modes are:
			// - Immediate - no v-sync, just immediately send commands to the buffer
			// - Fifo      - full v-sync
			// - Mailbox   - partial v-sync, so it'll vsync but if it has free time
			//               it'll do it immediately? (also not supported on my
			//               computer)
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &config);
		let atlas_bytes = include_bytes!("assets/icon.png");
		let atlas_image = image::load_from_memory(atlas_bytes)
			.expect("failed to load image");
		let atlas_rgba = atlas_image.to_rgba8();
		let atlas_size = atlas_rgba.dimensions();
		let atlas_extent = wgpu::Extent3d {
			width: atlas_size.0,
			height: atlas_size.1,
			depth_or_array_layers: 1,
		};
		let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("AtlasTexture"),
			size: atlas_extent,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});
		queue.write_texture(
			wgpu::ImageCopyTexture {
				texture: &atlas_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			&atlas_rgba,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: std::num::NonZeroU32::new(4 * atlas_size.0),
				rows_per_image: std::num::NonZeroU32::new(atlas_size.1),
			},
			atlas_extent
		);
		let atlas_view = atlas_texture.create_view(
			&wgpu::TextureViewDescriptor::default()
		);
		let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("AtlasSampler"),
			// i want ClampToBorder but no support :(
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Linear,
			border_color: Some(wgpu::SamplerBorderColor::TransparentBlack),
			..Default::default()
		});
		let atlas_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("AtlasBindGroupLayout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float {filterable: true}
						},
						count: None,
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
		});
		let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("AtlasBindGroup"),
			layout: &atlas_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&atlas_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&atlas_sampler),
				},
			]
		});
		let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(
				include_str!("shaders/shader.wgsl").into()
			),
		});

		let egui_platform = egui_winit_platform::Platform::new(
			egui_winit_platform::PlatformDescriptor {
				physical_width: size.width as u32,
				physical_height: size.height as u32,
				scale_factor: window.scale_factor(),
				font_definitions: egui::FontDefinitions::default(),
				style: Default::default(),
		});
		let egui_rpass = egui_wgpu_backend::RenderPass::new(
			&device, surface_format, 1
		);
		let egui_context = egui_platform.context();
		let mut egui_visuals = egui::style::Visuals::dark();
		egui_visuals.window_shadow = egui::epaint::Shadow {
			extrusion: 0.0,
			color: egui::Color32::TRANSPARENT,
		};
		egui_visuals.window_corner_radius = 0.0;
		egui_context.set_visuals(egui_visuals);

		let mut world_uniform = WorldUniform::default();
		world_uniform.update_atlas_size(atlas_size.0, atlas_size.1);
		let vertex_buffer = ResizeBuffer::new(vec![], wgpu::BufferUsages::VERTEX, &device);
		let index_buffer = ResizeBuffer::new(vec![], wgpu::BufferUsages::INDEX, &device);
		let world_uniform_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("UniformBuffer"),
				contents: bytemuck::cast_slice(&[world_uniform]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		});

		let world_uniform_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("UniformBindGroupLayout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					}
				]
		});
		let world_uniform_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("UniformBindGroup"),
				layout: &world_uniform_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: world_uniform_buffer.as_entire_binding(),
					}
				]
		});
		let render_pipeline_layout = device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("RenderPipelineLayout"),
				bind_group_layouts: &[
					&atlas_bind_group_layout,
					&world_uniform_bind_group_layout,
				],
				push_constant_ranges: &[],
		});
		let render_pipeline = device.create_render_pipeline(
			&wgpu::RenderPipelineDescriptor {
				label: Some("RenderPipeline"),
				layout: Some(&render_pipeline_layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: "vert",
					buffers: &[
						Vertex::desc().real(),
					],
				},
				fragment: Some(wgpu::FragmentState {
					module: &shader,
					entry_point: "frag",
					targets: &[wgpu::ColorTargetState {
						format: config.format,
						blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::ALL,
					}]
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Cw,
					cull_mode: Some(wgpu::Face::Back),
					// Fill - filled polygons, probably what you want
					// Line - requires Features::NON_FILL_POLYGON_MODE, but allows debug
					//        lines
					polygon_mode: wgpu::PolygonMode::Fill,
					unclipped_depth: false,
					conservative: false,
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState {
					count: 1,
					mask: !0,
					alpha_to_coverage_enabled: false
				},
				multiview: None,
		});
		Self {
			surface, device, queue, config, size, render_pipeline, world_uniform,
			atlas_bind_group, world_uniform_buffer, world_uniform_bind_group,
			start_info, egui_platform, egui_rpass, vertex_buffer, index_buffer,
			target_size: size,
			egui_state: debugger::DebuggerState::new(),
			pressed_keys: collections::HashSet::new(),
			dm_screen_offset: [0.5; 2],
			root_entity: Entity::new(),
		}
	}
	fn update_render(&mut self) {
		// self.instances = (0..10).map(|i| RectInstRaw {
		// 	pos: [i as f32 * 50.0, 50.0],
		// 	origin: [0, 0],
		// 	uv: [0, 0, 512, 512],
		// 	col: [(i * 25) as u8, 255, (i * 25) as u8, 255],
		// 	size: [64, 64],
		// 	rot: i as u16 * 4096,
		// 	flags: 0,
		// }).collect::<Vec<_>>();
		// self.instances.insert(0, RectInstRaw {
		// 	pos: [0.0, 0.0],
		// 	origin: [0, 0],
		// 	uv: [0, 16, 640, 480],
		// 	col: [255, 255, 255, 255],
		// 	size: [640, 480],
		// 	rot: 0,
		// 	flags: 0,
		// });
		let mut render_context = RenderContext::new(&mut self.vertex_buffer, &mut self.index_buffer);
		render_context.rect(
			[0.0, 0.0],
			[0.0, 0.0],
			[512.0, 512.0], 0.0,
			[[0.0, 0.0], [512.0, 0.0], [512.0, 512.0], [0.0, 512.0]],
			[255, 255, 255, 255],
		);
		self.root_entity.render(&mut render_context);
	}
	fn send_uniform_buffer(&mut self) {
		self.queue.write_buffer(
			&self.world_uniform_buffer, 0,
			bytemuck::cast_slice(&[self.world_uniform])
		);
	}
	// fn resize_instance_buffer(&mut self, new_len: usize) {
	// 	let start_len = self.instances.len();
	// 	if start_len < new_len {
	// 		// SAFETY: RectInstRaw implements bytemuck::Zeroable
	// 		self.instances.resize(new_len, unsafe { std::mem::zeroed() });
	// 	}
	// }
	// fn send_instance_buffer(&mut self) {
	// 	// update instances
	// 	self.update_instances();
	// 	if self.instances.len() > self.instance_len {
	// 		// multiply by two until it fits, makes allocation more O(n) apparently
	// 		let old_len = self.instance_len;
	// 		debug!("instance buffer start resize! things might crash here");
	// 		if self.instance_len == 0 { self.instance_len = 1; }
	// 		while self.instances.len() > self.instance_len { self.instance_len *= 2; }
	// 		debug!("instance buffer resize: {} -> {}", old_len, self.instance_len);
	// 		self.instance_buffer.destroy();
	// 		self.resize_instance_buffer(self.instance_len);
	// 		self.instance_buffer = self.device.create_buffer_init(
	// 			&wgpu::util::BufferInitDescriptor {
	// 				label: Some("InstanceBuffer"),
	// 				contents: bytemuck::cast_slice(&self.instances),
	// 				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
	// 		});
	// 	} else {
	// 		self.resize_instance_buffer(self.instance_len);
	// 		self.queue.write_buffer(
	// 			&self.instance_buffer, 0,
	// 			bytemuck::cast_slice(&self.instances)
	// 		);
	// 	}
	// }
	/// runs a resize event if the window has resized
	pub fn hard_resize(&mut self, force: bool) {
		let new_size = self.target_size;
		if new_size.width > 0 && new_size.height > 0 {
			if force || new_size.width != self.size.width || new_size.height != self.size.height {
				log::trace!("resize");
				log::trace!("sizing: {:?} -> {:?}", self.size, new_size);
				self.size = new_size;
				self.config.width = new_size.width;
				self.config.height = new_size.height;
				self.surface.configure(&self.device, &self.config);
				self.world_uniform.update_screen_size(
					new_size.width, new_size.height, self.start_info.integer_mode, self.dm_screen_offset);
				self.send_uniform_buffer();
			}
		}
	}
	/// global window event handling
	pub fn handle_event<T>(&mut self, event: &event::Event<T>) -> bool
	// where T: std::fmt::Debug {
	// 	debug!("event {:?}", event);
	{
		self.egui_platform.handle_event(&event);
		false
	}
	/// resizes the surface, may not actually cause a resize
	pub fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) {
		self.target_size = new_size;
	}
	/// process an event, if it returns false then it'll fall back to horizon
	/// handling
	pub fn input(&mut self, event: &event::WindowEvent) -> bool {
		// debug!("input {:?}", event);
		match event {
			event::WindowEvent::KeyboardInput {input, ..} => {
				let state = (
					input.state == winit::event::ElementState::Pressed,
					self.pressed_keys.contains(&input.scancode)
				);
				if state.0 {
					self.pressed_keys.insert(input.scancode);
				} else {
					self.pressed_keys.remove(&input.scancode);
				}
				log::debug!("keyboard {} {}", input.scancode, match state {
					(false, false) => "not held",
					(false, true) => "released",
					(true, false) => "pressed",
					(true, true) => "held",
				});
				true
			},
			_ => false
		}
	}
	/// run on a game-tick
	pub fn update(&mut self, delta_time: f64) {
		let info = UpdateInfo {
    	delta_time,
		};
		self.root_entity.update(&info);
	}
	/// renders a frame on screen
	pub fn render(
		&mut self,
		run_time: f64,
		delta_time: f64,
		window: &winit::window::Window,
	) -> Result<(), wgpu::SurfaceError> {
		self.egui_state.start_check();
		let res = self.egui_state.get_debug_modifiers();
		// RA says this is 7 errors
		// it's 0
		(_, self.dm_screen_offset) = res;
		self.update(delta_time);
		self.egui_platform.update_time(run_time);
		self.hard_resize(res.0);
		self.egui_state.checkpoint("update");
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(
			&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {label: Some("RenderEncoder")});
		self.egui_state.checkpoint("init");
		self.update_render();
		self.egui_state.checkpoint("generate");
		self.vertex_buffer.write_data(&self.queue, &self.device);
		self.index_buffer.write_data(&self.queue, &self.device);
		self.egui_state.checkpoint("buffer_write");
		let egui_output_view = output.texture.create_view(
			&wgpu::TextureViewDescriptor::default());
		self.egui_platform.begin_frame();
		let egui_context = self.egui_platform.context();
		self.egui_state.render(egui_context, delta_time);
		let (_, egui_paint_commands) = self.egui_platform.end_frame(Some(window));
		let egui_paint_jobs = self.egui_platform.context()
			.tessellate(egui_paint_commands);
		// let frame_time = (time::Instant::now() - egui_start).as_secs_f32();
		let egui_screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
			physical_width: self.size.width,
			physical_height: self.size.height,
			scale_factor: window.scale_factor() as f32,
		};
		self.egui_rpass.update_texture(
			&self.device, &self.queue, &self.egui_platform.context().font_image());
		self.egui_rpass.update_user_textures(&self.device, &self.queue);
		self.egui_rpass.update_buffers(
			&self.device, &self.queue, &egui_paint_jobs, &egui_screen_descriptor);
		self.egui_state.checkpoint("egui");
		let mut render_pass = encoder.begin_render_pass(
			&wgpu::RenderPassDescriptor {
				label: Some("RenderPass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.02, g: 0.04, b: 0.06, a: 1.0,
						}),
						store: true,
					},
				}],
				depth_stencil_attachment: None
		});
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
		// render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
		render_pass.set_index_buffer(
			self.index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);
		render_pass.set_bind_group(1, &self.world_uniform_bind_group, &[]);
		render_pass.draw_indexed(0..(self.index_buffer.len() * 3) as u32, 0, 0..1);
		self.egui_state.checkpoint("render_pass");
		drop(render_pass);
		self.egui_rpass.execute(
			&mut encoder,
			&egui_output_view,
			&egui_paint_jobs,
			&egui_screen_descriptor,
			None,
		).unwrap();
		self.queue.submit(iter::once(encoder.finish()));
		output.present();
		self.egui_state.checkpoint("present");
		Ok(())
	}
}
