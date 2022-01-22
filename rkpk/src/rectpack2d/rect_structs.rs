// port of rect_structs.h

// deps: none
// stat: done

pub trait Rect {
	fn get_x(&self) -> u32;
	fn get_y(&self) -> u32;
	fn get_w(&self) -> u32;
	fn get_h(&self) -> u32;
	fn area(&self) -> u32;
	fn perimeter(&self) -> u32;
	fn get_wh(&self) -> RectWH;
	fn path_mul(&self) -> f64 {self.get_wh().path_mul()}
}

pub trait OutputRectType: Rect {}

#[derive(Debug, Default, Copy, Clone)]
pub struct RectWH {
	pub w: u32,
	pub h: u32,
}

impl RectWH {
	pub fn new() -> Self{
		Self {w: 0, h: 0}
	}
	pub fn from(w: u32, h: u32) -> Self {
		Self {w, h}
	}
	pub fn flip(&self) -> Self {
		Self { w: self.h, h: self.w }
	}
	pub fn flip_mut(&mut self) -> &mut Self {
		std::mem::swap(&mut self.w, &mut self.h);
		self
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
	pub fn expand_with_mut<R: Rect>(&mut self, r: R) {
		self.w = self.w.max(r.get_x() + r.get_w());
		self.h = self.h.max(r.get_y() + r.get_h());
	}
}

impl Rect for RectWH {
	fn get_x(&self) -> u32 { 0 }
	fn get_y(&self) -> u32 { 0 }
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
	pub fn new() -> Self{
		Self {x: 0, y: 0, w: 0, h: 0}
	}
	pub fn from(x: u32, y: u32, w: u32, h: u32) -> Self {
		Self {x, y, w, h}
	}
}

impl Rect for RectXYWH {
	fn get_x(&self) -> u32 { self.x }
	fn get_y(&self) -> u32 { self.y }
	fn get_w(&self) -> u32 { self.w }
	fn get_h(&self) -> u32 { self.h }
	fn area(&self) -> u32 { self.w * self.h }
	fn perimeter(&self) -> u32 { 2 * (self.w + self.h) }
	fn get_wh(&self) -> RectWH {RectWH::from(self.w, self.h)}
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
	pub fn new() -> Self{
		Self {x: 0, y: 0, w: 0, h: 0, flipped: false}
	}
	pub fn from(x: u32, y: u32, w: u32, h: u32, flipped: bool) -> Self {
		Self {x, y, w, h, flipped}
	}
	pub fn from_xywh(r: RectXYWH) -> Self {
		Self {x: r.x, y: r.y, w: r.w, h: r.h, flipped: false}
	}
}

impl Rect for RectXYWHF {OutputRectType
	fn get_x(&self) -> u32 { self.x }
	fn get_y(&self) -> u32 { self.y }
	fn get_w(&self) -> u32 { self.w }
	fn get_h(&self) -> u32 { self.h }
	fn area(&self) -> u32 { self.w * self.h }
	fn perimeter(&self) -> u32 { 2 * (self.w + self.h) }
	fn get_wh(&self) -> RectWH {RectWH::from(self.w, self.h)}
}

impl OutputRectType for RectXYWH {}
impl OutputRectType for RectXYWHF {}

pub type SpaceRect = RectXYWH;
