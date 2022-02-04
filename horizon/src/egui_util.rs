//! utilities for working with egui
pub struct PanelState {
	open: bool,
	// true = right, false = left
	side: bool,
}

impl PanelState {
	pub fn new(side: bool) -> Self {
		Self {open: false, side: side}
	}
	pub fn is_open(&self) -> bool {
		self.open
	}
	fn arrow(&self) -> &'static str {
		if self.side {"<<"} else {">>"}
	}
	fn side(&self) -> egui::panel::Side {
		if self.side {
			egui::panel::Side::Right
		} else {
			egui::panel::Side::Left
		}
	}
	pub fn toggle_open_mut(&mut self) {
		self.open = !self.open
	}
	pub fn create(&self, title: impl std::hash::Hash) -> egui::SidePanel {
		egui::SidePanel::new(self.side(), title)
	}
	pub fn button(&mut self, ui: &mut egui::Ui) {
		if ui.button(self.arrow()).clicked() {
			self.side = !self.side;
		}
	}
}

pub trait Component {
	fn new() -> Self;
	fn render(&mut self, context: egui::CtxRef, delta_time: f64);
}
