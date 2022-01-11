use winit::event;
use winit::event_loop;
use winit::window;
use winit::dpi;
use wgpu::util::DeviceExt;
use log::{error, warn, info, debug, trace};

/// game-specific info goes here
pub struct StartInfo {

}

/// wgpu::VertexBufferLayout that owns it's attributes
struct FakeVertexBufferLayout {
	array_stride: wgpu::BufferAddress,
	attributes: Box<[wgpu::VertexAttribute]>,
	step_mode: wgpu::VertexStepMode,
}

macro_rules! fake_vertex_attr_array {
	( $type:ty, $step:tt, $( $loc:tt => $data:tt ),* ) => {
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 3],
	color: [f32; 3],
}

impl Vertex {
	fn desc() -> FakeVertexBufferLayout {
		fake_vertex_attr_array!(Vertex, Vertex, 0 => Float32x3, 1 => Float32x3)
	}
}

const VERTEXES: &[Vertex] = &[
	Vertex { position: [ 0.0,  0.5, 0.0], color: [1.0, 0.0, 0.0] },
	Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
	Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

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
	vertex_len: u32,
}

impl State {
	// we *might* have to make this as a "init" rather than "new" if some of the
	// settings in here are worth changing and require reconstruction
	async fn new(window: &window::Window) -> Self {
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
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_preferred_format(&adapter).unwrap(),
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
		let render_pipeline_layout = device.create_pipeline_layout(
			&wgpu::PipelineLayoutDescriptor {
				label: Some("RenderPipelineLayout"),
				bind_group_layouts: &[],
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
						blend: Some(wgpu::BlendState::REPLACE),
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
		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("VertexBuffer"),
				contents: bytemuck::cast_slice(VERTEXES),
				usage: wgpu::BufferUsages::VERTEX,
		});
		Self {
			surface, device, queue, config, size, render_pipeline, vertex_buffer,
			target_size: size,
			vertex_len: VERTEXES.len() as u32,
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
				// TODO(1e1001): compute screen scaling here based on a integer /
				//               non-integer scale setting
			}
		}
	}
	/// resizes the surface, may not actually cause a resize
	fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) {
		self.target_size = new_size;
	}
	/// process an event, if it returns false then it'll fall back to horizon
	/// handling
	fn input(&mut self, event: &event::WindowEvent) -> bool {
		false
	}
	/// run on a game-tick
	fn update(&mut self) {

	}
	/// renders a frame on screen
	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		self.hard_resize(false);
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(
			&wgpu::TextureViewDescriptor::default()
		);
		let mut encoder = self.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {label: Some("RenderEncoder"),}
		);
		{ // block because we need to let the render pass drop before submitting
			// the queue.
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("RenderPass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.0, g: 0.0, b: 0.0, a: 1.0,
						}),
						store: true,
					},
				}],
				depth_stencil_attachment: None
			});
			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.draw(0..self.vertex_len, 0..1);
		};
		self.queue.submit(std::iter::once(encoder.finish()));
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
	let mut state = State::new(&window).await;
	state.hard_resize(true);
	event_loop.run(move |event, _, control_flow| match event {
		event::Event::RedrawRequested(window_id) if window_id == window.id() => {
			state.update();
			match state.render() {
				Ok(_) => {},
				Err(wgpu::SurfaceError::Lost) => state.hard_resize(true),
				Err(wgpu::SurfaceError::OutOfMemory) => {
					error!("Out of memory, Exiting!");
					*control_flow = event_loop::ControlFlow::Exit
				},
				Err(e) => error!("{:?}", e),
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
	});
}
