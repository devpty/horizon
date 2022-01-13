use std::iter;
use std::time;
use winit::event;
use winit::event_loop;
use winit::window;
use winit::dpi;
use wgpu::util::DeviceExt;
#[allow(unused_imports)]
use log::{error, warn, info, debug, trace};

/// game-specific info goes here
pub struct StartInfo {
	pub integer: bool,
}

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
	fn new<T>(attributes: Box<[wgpu::VertexAttribute]>, step_mode: wgpu::VertexStepMode) -> Self {
		FakeVertexBufferLayout {
			attributes: attributes, step_mode,
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
/// # screen space mapping
/// to map from ([0, width), [0, height)) -> ([-1, 1], [-1, 1]) do 2 * f32(pos + offset) / f32(size) - vec2<f32>(1.0, 1.0)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct WorldUniform {
	offset: [u32; 2],
	size: [u32; 2],
}

impl WorldUniform {
	fn update_screen_size(&mut self, width: u32, height: u32, integer: bool) {
		let target_width = 640u32;
		let target_height = 480u32;
		// determine scaling factor
		let scaling_factor = (width as f64 / target_width as f64)
			.min(height as f64 / target_height as f64);
		// optionally floor it
		let int_scaling_factor = if integer { scaling_factor.floor() }
			/*                        */ else { scaling_factor };
		// center it
		let scaled_width = (int_scaling_factor * target_width as f64).round() as u32;
		let scaled_height = (int_scaling_factor * target_height as f64).round() as u32;
		self.size = [scaled_width, scaled_height];
		self.offset = [
			(width - scaled_width) / 2,
			(height - scaled_height) / 2,
		];
		debug!("WorldUniform = {:?}", self);
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [u16; 2],
}

impl CanBuffer for Vertex {
	fn desc() -> FakeVertexBufferLayout {
		fake_vertex_attr_array!(Self, Vertex,
			0 => Uint16x2,
		)
	}
}

/// gpu-sent rectangle data
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstRaw {
	/// position in pixels
	pos: [f32; 2],
	/// origin in [0, 65535] -> [0, 1]
	origin: [u16; 2],
	/// uv start position in pixels
	uv: [u16; 4],
	/// color, rgba
	col: [u8; 4],
	/// size in pixels
	size: [u16; 2],
	/// rotation in [0, 65535) -> [0°, 360°)
	rot: u16,
	/// extra flags
	/// 2⁰ - shading
	/// 2¹ - 2¹⁵ - unused
	flags: u16,
}

impl CanBuffer for RectInstRaw {
	fn desc() -> FakeVertexBufferLayout {
		fake_vertex_attr_array!(Self, Instance,
			1 => Float32x2, // pos
			2 => Unorm16x2, // origin
			3 => Uint16x4, // uv
			4 => Unorm8x4, // col
			5 => Uint16x4, // size, rotation, flags
		)
	}
}


const VERTEXES: &[Vertex] = &[
	Vertex { position: [0, 0] },
	Vertex { position: [1, 0] },
	Vertex { position: [0, 1] },
	Vertex { position: [1, 1] },
];
const INDEXES: &[u16] = &[
	0, 2, 1, 1, 2, 3
];

#[derive(Clone, Debug)]
struct RingBuffer<T: Clone>(Vec<T>, usize);
// impl<T: Clone + Default> RingBuffer<T> {
// 	fn new(size: usize) -> Self {
// 		Self(vec![T::default(); size], 0)
// 	}
// }
impl<T: Clone> RingBuffer<T> {
	fn new_fill(size: usize, value: T) -> Self {
		Self(vec![value; size], 0)
	}
	fn push(&mut self, item: T) {
		self.0[self.1] = item;
		self.1 += 1;
		if self.1 == self.0.len() { self.1 = 0; }
	}
	fn iter(&self) -> std::iter::Chain<std::slice::Iter<'_, T>, std::slice::Iter<'_, T>> {
		let (l, r) = self.0.split_at(self.1);
		r.iter().chain(l.iter())
	}
	fn len(&self) -> usize {
		self.0.len()
	}
}

/// global state info
struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: dpi::PhysicalSize<u32>,
	target_size: dpi::PhysicalSize<u32>,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	index_len: u32,
	egui_platform: egui_winit_platform::Platform,
	egui_rpass: egui_wgpu_backend::RenderPass,
	egui_profiler_data: RingBuffer<f32>,
	instances: Vec<RectInstRaw>,
	instance_buffer: wgpu::Buffer,
	instance_len: usize,
	world_uniform: WorldUniform,
	world_uniform_buffer: wgpu::Buffer,
	world_uniform_bind_group: wgpu::BindGroup,
	start_info: StartInfo,
}

impl State {
	// we *might* have to make this as a "init" rather than "new" if some of the
	// settings in here are worth changing and require reconstruction
	async fn new(window: &window::Window, start_info: StartInfo) -> Self {
		let size = window.inner_size();
		let instance = wgpu::Instance::new(wgpu::Backends::all());
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
			present_mode: wgpu::PresentMode::Mailbox,
		};
		surface.configure(&device, &config);
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
		let egui_rpass = egui_wgpu_backend::RenderPass::new(&device, surface_format, 1);
		let egui_context = egui_platform.context();
		let mut egui_visuals = egui::style::Visuals::dark();
		egui_visuals.window_shadow = egui::epaint::Shadow {
			extrusion: 0.0,
			color: egui::Color32::TRANSPARENT,
		};
		egui_visuals.window_corner_radius = 0.0;
		egui_context.set_visuals(egui_visuals);

		let world_uniform = WorldUniform::default();
		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("VertexBuffer"),
				contents: bytemuck::cast_slice(VERTEXES),
				usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("IndexBuffer"),
				contents: bytemuck::cast_slice(INDEXES),
				usage: wgpu::BufferUsages::INDEX,
		});
		let instance_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("InstanceBuffer"),
				contents: &[],
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
		});
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
						RectInstRaw::desc().real(),
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
					front_face: wgpu::FrontFace::Ccw,
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
			surface, device, queue, config, size, render_pipeline, vertex_buffer,
			index_buffer, instance_buffer, world_uniform,
			world_uniform_buffer, world_uniform_bind_group, start_info,
			egui_platform, egui_rpass,
			instances: Vec::new(),
			instance_len: 0,
			target_size: size,
			index_len: INDEXES.len() as u32,
			egui_profiler_data: RingBuffer::new_fill(64, 1.0),
		}
	}
	fn update_instances(&mut self) {
		self.instances = (0..10).map(|i| RectInstRaw {
			pos: [i as f32 * 100.0, 10.0],
			origin: [65535, 0],
			uv: [0, 0, 10, 10],
			col: [(i * 25) as u8, 0, 255, 255],
			size: [50, 70],
			rot: i as u16 * 4096,
			flags: 0,
		}).collect::<Vec<_>>();
	}
	fn send_uniform_buffer(&mut self) {
		self.queue.write_buffer(&self.world_uniform_buffer, 0, bytemuck::cast_slice(&[self.world_uniform]));
	}
	fn resize_instance_buffer(&mut self, new_len: usize) {
		let start_len = self.instances.len();
		if start_len < new_len {
			self.instances.resize(new_len, unsafe { std::mem::zeroed() });
		}
	}
	fn send_instance_buffer(&mut self) {
		// update instances
		self.update_instances();
		if self.instances.len() > self.instance_len {
			// multiply by two until it fits, makes allocation more O(n) apparently
			let old_len = self.instance_len;
			if self.instance_len == 0 { self.instance_len = 1; }
			while self.instances.len() > self.instance_len { self.instance_len *= 2; }
			debug!("instance buffer resize: {} -> {}", old_len, self.instance_len);
			self.instance_buffer.destroy();
			self.resize_instance_buffer(self.instance_len);
			self.instance_buffer = self.device.create_buffer_init(
				&wgpu::util::BufferInitDescriptor {
					label: Some("InstanceBuffer"),
					contents: bytemuck::cast_slice(&self.instances),
					usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			});
		} else {
			self.resize_instance_buffer(self.instance_len);
			self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&self.instances));
		}
	}
	/// runs a resize event if the window has resized
	fn hard_resize(&mut self, force: bool) {
		let new_size = self.target_size;
		if new_size.width > 0 && new_size.height > 0 {
			if force || new_size.width != self.size.width
			|| new_size.height != self.size.height {
				trace!("resize");
				debug!("sizing: {:?} -> {:?}", self.size, new_size);
				self.size = new_size;
				self.config.width = new_size.width;
				self.config.height = new_size.height;
				self.surface.configure(&self.device, &self.config);
				self.world_uniform.update_screen_size(
					new_size.width, new_size.height, self.start_info.integer
				);
				self.send_uniform_buffer();
			}
		}
	}
	/// resizes the surface, may not actually cause a resize
	fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) {
		self.target_size = new_size;
	}
	/// process an event, if it returns false then it'll fall back to horizon
	/// handling
	fn input(&mut self, _event: &event::WindowEvent) -> bool {
		false
	}
	/// run on a game-tick
	fn update(&mut self) {

	}
	/// renders a frame on screen
	fn render(
		&mut self,
		run_time: f64,
		delta_time: f64,
		window: &winit::window::Window,
	) -> Result<(), wgpu::SurfaceError> {
		self.egui_platform.update_time(run_time);
		self.hard_resize(false);
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(
			&wgpu::TextureViewDescriptor::default()
		);
		let mut encoder = self.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {label: Some("RenderEncoder"),}
		);
		self.update_instances();
		self.send_instance_buffer();
		let egui_output_view = output.texture.create_view(
			&wgpu::TextureViewDescriptor::default()
		);
		self.egui_platform.begin_frame();
		let egui_context = self.egui_platform.context();
		self.egui_profiler_data.push(delta_time as f32);
		egui::SidePanel::right("debug_panel")
			.resizable(false)
			.show(&egui_context, |ui|
		{
			ui.label(egui::RichText::from("Horizon Debug").text_style(egui::TextStyle::Heading));
			ui.label(format!("running at {:.0}fps", self.egui_profiler_data.len() as f32 / self.egui_profiler_data.iter().fold(0.0, |l, r| l + r)));
			// ui.label(format!("{:?}", self.egui_profiler_data));
		});
		let (_, egui_paint_commands) = self.egui_platform.end_frame(Some(window));
		let egui_paint_jobs = self.egui_platform.context().tessellate(egui_paint_commands);
		// let frame_time = (time::Instant::now() - egui_start).as_secs_f32();
		let egui_screen_descriptor = egui_wgpu_backend::ScreenDescriptor {
			physical_width: self.size.width,
			physical_height: self.size.height,
			scale_factor: window.scale_factor() as f32,
		};
		self.egui_rpass.update_texture(&self.device, &self.queue, &self.egui_platform.context().font_image());
		self.egui_rpass.update_user_textures(&self.device, &self.queue);
		self.egui_rpass.update_buffers(&self.device, &self.queue, &egui_paint_jobs, &egui_screen_descriptor);
		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
		render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
		render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
		render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
		render_pass.set_bind_group(0, &self.world_uniform_bind_group, &[]);
		render_pass.draw_indexed(0..self.index_len, 0, 0..self.instances.len() as _);
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
		Ok(())
	}
}

/// start horizon
pub async fn start(info: StartInfo) {
	let event_loop = event_loop::EventLoop::new();
	let window = window::WindowBuilder::new()
		.with_title("horizon")
		.build(&event_loop).expect("Failed to window");
	let mut state = State::new(&window, info).await;
	state.hard_resize(true);
	let start_time = time::Instant::now();
	let mut prev_time = 0.0;
	event_loop.run(move |event, _, control_flow| {
		state.egui_platform.handle_event(&event);
		match event {
			event::Event::RedrawRequested(window_id) if window_id == window.id() => {
				state.update();
				let elapsed = start_time.elapsed().as_secs_f64();
				let delta_time = elapsed - prev_time;
				prev_time = elapsed;
				match state.render(elapsed, delta_time, &window) {
					Ok(_) => {},
					Err(wgpu::SurfaceError::Lost) => state.hard_resize(true),
					Err(wgpu::SurfaceError::OutOfMemory) => {
						error!("Out of memory, Exiting!");
						*control_flow = event_loop::ControlFlow::Exit
					},
					Err(e) => error!("Render error: {:?}", e),
				}
			}
			event::Event::MainEventsCleared => {
				window.request_redraw();
			}
			event::Event::WindowEvent {
				ref event, window_id
			} if window_id == window.id() => if !state.input(event) {
				match event {
					event::WindowEvent::CloseRequested | event::WindowEvent::KeyboardInput {
						input: event::KeyboardInput {
							state: event::ElementState::Pressed,
							virtual_keycode: Some(event::VirtualKeyCode::Escape),
							.. // square
						},.. // wow
					} => {
						info!("quit requested");
						*control_flow = event_loop::ControlFlow::Exit
					},
					event::WindowEvent::Resized(physical_size) => {
						state.resize(*physical_size);
					}
					event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
						state.resize(**new_inner_size);
					}
					_ => {},
				}
			}
			_ => {}
		}
	});
}
