use std::borrow::Cow;
use winit::{
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::Window,
};
use winit::dpi;

async fn run(event_loop: EventLoop<()>, window: Window) {
	let size = window.inner_size();
	let instance = wgpu::Instance::new(wgpu::Backends::all());
	let surface = unsafe { instance.create_surface(&window) };
	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			force_fallback_adapter: false,
			compatible_surface: Some(&surface),
		})
		.await
		.expect("Failed to find an appropriate adapter");

	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor {
				label: None,
				features: wgpu::Features::empty(),
				// Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
				// i can already tell you copied this off the internet somewhere
				limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
			},
			None,
		)
		.await
		.expect("Failed to create device");

	// Load the shaders from disk
	let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
		label: None,
		source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/shader.wgsl"))),
	});

	let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: None,
		bind_group_layouts: &[],
		push_constant_ranges: &[],
	});

	let swapchain_format = surface.get_preferred_format(&adapter).unwrap();

	let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: None,
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vert",
			buffers: &[],
		},
		fragment: Some(wgpu::FragmentState {
			module: &shader,
			entry_point: "frag",
			targets: &[swapchain_format.into()],
		}),
		primitive: wgpu::PrimitiveState::default(),
		depth_stencil: None,
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
	});

	let mut config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: swapchain_format,
		width: size.width,
		height: size.height,
		present_mode: wgpu::PresentMode::Mailbox,
	};

	surface.configure(&device, &config);

	event_loop.run(move |event, _, control_flow| {
		// Have the closure take ownership of the resources.
		// `event_loop.run` never returns, therefore we must do this to ensure
		// the resources are properly cleaned up.
		let _ = (&instance, &adapter, &shader, &pipeline_layout);

		*control_flow = ControlFlow::Wait;
		match event {
			Event::WindowEvent {
				event: WindowEvent::Resized(size),
				..
			} => {
				// Reconfigure the surface with the new size
				config.width = size.width;
				config.height = size.height;
				surface.configure(&device, &config);
			}
			Event::RedrawRequested(_) => {
				let frame = surface
					.get_current_texture()
					.expect("Failed to acquire next swap chain texture");
				let view = frame
					.texture
					.create_view(&wgpu::TextureViewDescriptor::default());
				let mut encoder =
					device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
				{
					let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
						label: None,
						color_attachments: &[wgpu::RenderPassColorAttachment {
							view: &view,
							resolve_target: None,
							ops: wgpu::Operations {
								load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
								store: true,
							},
						}],
						depth_stencil_attachment: None,
					});
					rpass.set_pipeline(&render_pipeline);
					rpass.draw(0..3, 0..1);
				}

				queue.submit(Some(encoder.finish()));
				frame.present();
			}
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => *control_flow = ControlFlow::Exit,
			_ => {}
		}
	});
}

#[tokio::main]
async fn main() {
	let event_loop = EventLoop::new();
	let window = winit::window::WindowBuilder::new()
		.with_title("Horizon")
		.with_min_inner_size(dpi::Size::new(dpi::PhysicalSize {width: 640, height: 480}))
		.build(&event_loop).unwrap();
	env_logger::init();
	run(event_loop, window).await;
}
