// port of rect_structs.h

// deps: none
// stat: done

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RectWH {
	pub w: u32,
	pub h: u32,
}

impl RectWH {
	pub fn new(w: u32, h: u32) -> Self {
		Self { w, h }
	}
	pub fn max_size(&self) -> u32 {
		if self.h > self.w {
			self.h
		} else {
			self.w
		}
	}
	pub fn min_size(&self) -> u32 {
		if self.h < self.w {
			self.h
		} else {
			self.w
		}
	}
	pub fn expand_with_mut(&mut self, r: RectXYWH) {
		self.w = self.w.max(r.x + r.w);
		self.h = self.h.max(r.y + r.h);
	}
	pub fn area(&self) -> u32 {
		self.w * self.h
	}
	pub fn perimeter(&self) -> u32 {
		2 * (self.w + self.h)
	}
	pub fn path_mul(&self) -> f64 {
		self.max_size() as f64 / (self.min_size() * self.area()) as f64
	}
	pub fn to_xywh(&self) -> RectXYWH {
		RectXYWH::new(0, 0, self.w, self.h)
	}
}

impl From<(u32, u32)> for RectWH {
	fn from((w, h): (u32, u32)) -> Self {
		Self { w, h }
	}
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RectXYWH {
	pub x: u32,
	pub y: u32,
	pub w: u32,
	pub h: u32,
}

impl RectXYWH {
	pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
		Self { x, y, w, h }
	}
	pub fn from_wh(r: RectWH) -> Self {
		Self {
			x: 0,
			y: 0,
			w: r.w,
			h: r.h,
		}
	}
	pub fn area(&self) -> u32 {
		self.w * self.h
	}
	pub fn perimeter(&self) -> u32 {
		2 * (self.w + self.h)
	}
	pub fn to_wh(&self) -> RectWH {
		RectWH::new(self.w, self.h)
	}
	pub fn reset_xy(&self) -> RectXYWH {
		RectXYWH::new(0, 0, self.w, self.h)
	}
}

impl From<(u32, u32, u32, u32)> for RectXYWH {
	fn from((x, y, w, h): (u32, u32, u32, u32)) -> Self {
		Self { x, y, w, h }
	}
}
