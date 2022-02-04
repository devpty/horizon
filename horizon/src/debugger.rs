//! in-game debugger ui
use crate::egui_util;
use crate::utils;

pub struct DebuggerState {
	profiler_data: utils::RingBuffer<f32>,
	panel_state: egui_util::PanelState,
}

impl egui_util::Component for DebuggerState {
	fn new() -> Self {
		Self {
			profiler_data: utils::RingBuffer::new_fill(512, 1.0),
			panel_state: egui_util::PanelState::new(false),
		}
	}
	fn render(&mut self, context: egui::CtxRef, delta_time: f64) {
		self.profiler_data.push(delta_time as f32);
		if self.panel_state.is_open() {
			self.panel_state.create("debug_panel")
				.resizable(false)
				.show(&context, |ui| {
					ui.label(egui::RichText::from("Horizon Debug").text_style(egui::TextStyle::Heading));
					self.panel_state.button(ui);
					let framerate_sum = self.profiler_data.iter().fold(0.0, |l, r| l + r);
					ui.label(format!("running at {:.0}fps", self.profiler_data.len() as f32 / framerate_sum));
					use egui::plot;
					plot::Plot::new("debug_plot")
						.view_aspect(2.0)
						.allow_drag(false)
						.allow_zoom(false)
						.include_x(0)
						.include_x(1)
						.include_y(0)
						.show(ui, |plot| {
							let mut framerate_part = 0f32;
							plot.line(plot::Line::new(plot::Values::from_values_iter(
								self.profiler_data.iter().enumerate().map(|n| {
									framerate_part += *n.1;
									plot::Value::new(1.0 + framerate_part - framerate_sum, 1000.0 * *n.1 as f64)
									// plot::Value::new(n.0 as f64 / self.profiler_data.len() as f64, *n.1 as f64)
								}).filter(|n| n.x >= 0.0)
							)));
					});
			});
		}
		egui::Window::new("panel_opener")
			// .auto_sized()
			// .frame(egui::Frame::none())
			// .title_bar(false)
			// .fixed_pos(egui::pos2(0.0, 0.0))
			.show(&context, |ui| {
				ui.label("horizon");
				if ui.button("debug").clicked() {
					self.panel_state.toggle_open_mut();
				}
				ui.allocate_space(ui.available_size());
		});
	}
}
