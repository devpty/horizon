// port of rect_structs.h

// deps: none
// stat: done

pub trait Rect: Default + Copy {
	fn get_w(&self) -> u32;
	fn get_h(&self) -> u32;
	fn area(&self) -> u32;
	fn perimeter(&self) -> u32;
	fn get_wh(&self) -> RectWH;
	fn path_mul(&self) -> f64 {self.get_wh().path_mul()}
}

pub trait OutputRect: Rect {
	const ALLOW_FLIP: bool;
	fn from_xywhf(x: u32, y: u32, w: u32, h: u32, f: bool) -> Self;
	fn get_x(&self) -> u32;
	fn get_y(&self) -> u32;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RectWH {
	pub w: u32,
	pub h: u32,
}

impl RectWH {
	pub fn new(w: u32, h: u32) -> Self {
		Self {w, h}
	}
	pub fn flip(&self) -> Self {
		Self { w: self.h, h: self.w }
	}
	pub fn max_size(&self) -> u32 {
		if self.h > self.w {self.h} else {self.w}
	}
	pub fn min_size(&self) -> u32 {
		if self.h < self.w {self.h} else {self.w}
	}
	pub fn path_mul(&self) -> f64 {
		self.max_size() as f64 / (self.min_size() * self.area()) as f64
	}
	pub fn expand_with_mut<R: OutputRect>(&mut self, r: R) {
		self.w = self.w.max(r.get_x() + r.get_w());
		self.h = self.h.max(r.get_y() + r.get_h());
	}
}

impl Rect for RectWH {
	fn get_w(&self) -> u32 { self.w }
	fn get_h(&self) -> u32 { self.h }
	fn area(&self) -> u32 { self.w * self.h }
	fn perimeter(&self) -> u32 { 2 * (self.w + self.h) }
	fn get_wh(&self) -> RectWH { self.clone() }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RectXYWH {
	pub x: u32,
	pub y: u32,
	pub w: u32,
	pub h: u32,
}

impl RectXYWH {
	pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
		Self {x, y, w, h}
	}
}

impl Rect for RectXYWH {
	fn get_w(&self) -> u32 { self.w }
	fn get_h(&self) -> u32 { self.h }
	fn area(&self) -> u32 { self.w * self.h }
	fn perimeter(&self) -> u32 { 2 * (self.w + self.h) }
	fn get_wh(&self) -> RectWH {RectWH::new(self.w, self.h)}
}
impl OutputRect for RectXYWH {
	const ALLOW_FLIP: bool = false;
	fn from_xywhf(x: u32, y: u32, w: u32, h: u32, _f: bool) -> Self {
		// commenting that out because it should never happen (because ALLOW_FLIP is false)
		// if f {
		// 	panic!("can't flip a RectXYWH")
		// }
		Self { x, y, w, h }
	}
	fn get_x(&self) -> u32 { self.x }
	fn get_y(&self) -> u32 { self.y }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct RectXYWHF {
	pub x: u32,
	pub y: u32,
	pub w: u32,
	pub h: u32,
	pub flipped: bool,
}

impl RectXYWHF {
	pub fn new(x: u32, y: u32, w: u32, h: u32, flipped: bool) -> Self {
		if flipped {
			Self {x, y, h, w, flipped: true}
		} else {
			Self {x, y, w, h, flipped: false}
		}
	}
}

impl Rect for RectXYWHF {
	fn get_w(&self) -> u32 { self.w }
	fn get_h(&self) -> u32 { self.h }
	fn area(&self) -> u32 { self.w * self.h }
	fn perimeter(&self) -> u32 { 2 * (self.w + self.h) }
	fn get_wh(&self) -> RectWH {RectWH::new(self.w, self.h)}
}

impl OutputRect for RectXYWHF {
	const ALLOW_FLIP: bool = true;
	fn from_xywhf(x: u32, y: u32, w: u32, h: u32, f: bool) -> Self {
		Self::new(x, y, w, h, f)
	}
	fn get_x(&self) -> u32 { self.x }
	fn get_y(&self) -> u32 { self.y }
}
