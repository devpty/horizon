// use std::io;

/// rectangle with flipping
///
/// x, y, width, height, flip
#[derive(Debug, Copy, Clone)]
pub struct Rect(pub u32, pub u32, pub u32, pub u32, pub bool);

impl Rect {
	pub fn with_pos(&self, pos: (u32, u32)) -> Self {
		Self(pos.0, pos.1, self.2, self.3, self.4)
	}
}
