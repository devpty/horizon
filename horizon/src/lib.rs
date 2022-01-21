use std::time;
use winit::event;
use winit::event_loop;
use winit::window;
use winit::dpi;
#[allow(unused_imports)]
use log::{error, warn, info, debug, trace};

mod state;
mod debugger;
mod egui_util;
mod utils;

/// game-specific info goes here
pub struct StartInfo {
	pub start_state: bool,
}

/// start horizon
pub async fn start(info: StartInfo) {
	let event_loop = event_loop::EventLoop::new();
	let window = window::WindowBuilder::new()
		.with_title("horizon")
		.with_inner_size(dpi::PhysicalSize {width: 800, height: 480})
		.with_min_inner_size(dpi::PhysicalSize {width: 640, height: 480})
		// .with_resizable(false)
		.build(&event_loop).expect("Failed to window");
	let mut state = state::State::new(&window, info).await;
	state.hard_resize(true);
	let start_time = time::Instant::now();
	let mut prev_time = 0.0;
	event_loop.run(move |event, _, control_flow| {
		if !state.handle_event(&event) { match event {
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
					Err(wgpu::SurfaceError::Outdated) => {},
					Err(e) => error!("Render error: {:?}", e),
				}
			}
			event::Event::MainEventsCleared => {
				window.request_redraw();
			}
			event::Event::WindowEvent {
				ref event, window_id
			} if window_id == window.id() => if !state.input(event) { match event {
				event::WindowEvent::CloseRequested => {
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
			}}
			_ => {}
		}
	}});
}
