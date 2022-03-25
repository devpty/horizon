//! in-game debugger ui
use std::collections::VecDeque;
use std::time::Instant;

use egui::plot;

use crate::egui_util;

pub struct DebuggerState {
	profiler_data: VecDeque<(f32, Vec<f32>)>,
	checkpoint: Instant,
	checkpoint_data: Vec<f32>,
	checkpoint_labels: Vec<&'static str>,
	p_checkpoint_data: Vec<f32>,
	p_checkpoint_labels: Vec<&'static str>,
	sample_time: f32,
	modifiers_changed: bool,
	dm_screen_offset: [f64; 2],
}

impl DebuggerState {
	pub fn start_check(&mut self) {
		self.checkpoint = Instant::now();
		self.p_checkpoint_data = self.checkpoint_data.clone();
		self.p_checkpoint_labels = self.checkpoint_labels.clone();
		self.checkpoint_data = vec![];
		self.checkpoint_labels = vec![];
	}
	pub fn checkpoint(&mut self, name: &'static str) {
		self.checkpoint_data
			.push(self.checkpoint.elapsed().as_secs_f32());
		self.checkpoint_labels.push(name);
	}
	pub fn get_debug_modifiers(&mut self) -> (bool, [f64; 2]) {
		let mod_changed = self.modifiers_changed;
		self.modifiers_changed = false;
		(mod_changed, self.dm_screen_offset)
	}
}

impl egui_util::EguiComponent for DebuggerState {
	fn new() -> Self {
		Self {
			profiler_data: VecDeque::new(),
			checkpoint: Instant::now(),
			checkpoint_data: vec![],
			checkpoint_labels: vec![],
			p_checkpoint_data: vec![],
			p_checkpoint_labels: vec![],
			sample_time: 1.0,
			modifiers_changed: false,
			dm_screen_offset: [0.5, 0.5],
		}
	}
	fn render(&mut self, context: egui::CtxRef, delta_time: f64) {
		// fn render(&mut self, context: egui::Context, delta_time: f64) {
		self.profiler_data
			.push_front((delta_time as f32, self.p_checkpoint_data.clone()));
		egui::Window::new("debug").show(&context, |ui| {
			egui::CollapsingHeader::new("Frame Times").show(ui, |ui| {
				let mut sum = 0.0;
				let items: Vec<_> = self
					.profiler_data
					.iter()
					.enumerate()
					.map(|(i, v)| {
						sum += v.0;
						(i, (sum, v.1.clone(), v.0))
					})
					.take_while(|(_, (v, _, _))| *v <= self.sample_time)
					.collect();
				let mut lines = vec![];
				let mut real_time = self.sample_time;
				if items.len() > 0 {
					let line_count = items.iter().map(|v| v.1 .1.len()).max().unwrap_or(0);
					for i in 0..line_count {
						let mut res = vec![];
						let mut time_sum = 0.0;
						let mut time_count = 0;
						for j in &items {
							if j.1 .1.len() >= line_count {
								let add = j.1 .1[i];
								res.push((j.1 .0, add));
								time_count += 1;
								time_sum += add;
							}
						}
						lines.push((res, time_sum / time_count as f32));
					}
					if let Some(last) = items.last() {
						real_time = last.1 .0;
						for _ in last.0 + 1..self.profiler_data.len() {
							self.profiler_data.pop_back();
						}
					}
				}
				ui.add(
					egui::Slider::new(&mut self.sample_time, 0.1..=100.0)
						.text("Sample time")
						.suffix(" s")
						.logarithmic(true)
						.clamp_to_range(false),
				);
				let framerate = self.profiler_data.len() as f32 / real_time;
				ui.label(format!(
					"Frame time of {:.3}ms ({:.3}fps)",
					1000.0 / framerate,
					framerate
				));
				let mut prev_time = 0.0;
				for ((_, i), j) in lines.iter().zip(self.p_checkpoint_labels.iter()) {
					ui.label(format!(
						"\t•  {} took {:.3}ms",
						j,
						(*i - prev_time) * 1000.0
					));
					prev_time = *i;
				}
				ui.label(format!(
					"\t•  free time of {:.3}ms",
					(prev_time - sum / self.profiler_data.len() as f32) * 1000.0
				));
				ui.label(format!("{} samples", self.profiler_data.len()));
				egui::CollapsingHeader::new("Plot").show(ui, |ui| {
					let plot_factor = if self.sample_time - real_time < 0.1 {
						self.sample_time / real_time
					} else {
						1.0
					};
					plot::Plot::new("debug_plot")
						.allow_drag(false)
						.allow_zoom(false)
						.include_x(0)
						.include_y(0)
						.height(100.0)
						.show(ui, |plot| {
							for (i, _) in lines {
								plot.line(plot::Line::new(plot::Values::from_values_iter(
									i.iter().map(|(j, v)| {
										plot::Value::new(
											-(plot_factor * *j) as f64,
											1000.0 * *v as f64,
										)
									}),
								)));
							}
							plot.line(plot::Line::new(plot::Values::from_values_iter(
								items.iter().map(|(_, (j, _, v))| {
									plot::Value::new(-(plot_factor * *j) as f64, 1000.0 * *v as f64)
								}),
							)));
						});
				});
			});
			egui::CollapsingHeader::new("Fun Debug Modifiers").show(ui, |ui| {
				self.modifiers_changed |= ui
					.add(
						egui::Slider::new(&mut self.dm_screen_offset[0], 0.0..=1.0)
							.text("state.screen_offset[0]"),
					)
					.changed();
				self.modifiers_changed |= ui
					.add(
						egui::Slider::new(&mut self.dm_screen_offset[1], 0.0..=1.0)
							.text("state.screen_offset[1]"),
					)
					.changed();
			});
			ui.allocate_space(ui.available_size());
		});
	}
}
