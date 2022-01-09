use winit::event;
use winit::event_loop;
use winit::window;
use winit::dpi;

pub struct StartInfo {

}

struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: dpi::PhysicalSize<u32>,
	target_size: dpi::PhysicalSize<u32>,
}

impl State {
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
			present_mode: wgpu::PresentMode::Mailbox,
		};
		Self {
			surface, device, queue, config, size,
			target_size: size
		}
	}
	fn actual_resize(&mut self) {
		let new_size = self.target_size;
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
		}
	}
	fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) {
		self.target_size = new_size;
		self.actual_resize();
	}
	fn input(&mut self, event: &event::WindowEvent) -> bool {
		false
	}
	fn update(&mut self) {

	}
	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("RenderEncoder"),
		});
		{
			encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("RenderPass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.1, g: 0.2, b: 0.3, a: 1.0,
						}),
						store: true,
					},
				}],
				depth_stencil_attachment: None
			});
		};
		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();
		Ok(())
	}
}

pub async fn start(info: StartInfo) {
	env_logger::init();
	let event_loop = event_loop::EventLoop::new();
	let window = window::WindowBuilder::new()
		.with_title("horizon")
		.build(&event_loop).expect("Failed to window");
	let mut state = State::new(&window).await;
	let mut resize_id = 1u128;
	event_loop.run(move |event, _, control_flow| match event {
		event::Event::RedrawRequested(window_id) if window_id == window.id() => {
			state.update();
			match state.render() {
				Ok(_) => {},
				Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
				Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = event_loop::ControlFlow::Exit,
				Err(e) => eprintln!("{:?}", e),
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
						..
					},..
				} => *control_flow = event_loop::ControlFlow::Exit,
				event::WindowEvent::Resized(physical_size) => {
					println!("###### RESIZE EVENT {}", resize_id);
					resize_id += 1;
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
