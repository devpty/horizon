// port of insert_and_split.h

// deps: rect_structs
// stat: done

use super::rect_structs;

#[derive(Debug, Copy, Clone)]
// in c++ this is implemented as a fixed size array
pub enum CreatedSplits {
	Failed,
	Zero,
	One(rect_structs::SpaceRect),
	Two(rect_structs::SpaceRect, rect_structs::SpaceRect)
}

impl CreatedSplits {
	pub fn count(&self) -> i8 {
		match self {
			Self::Failed => -1,
			Self::Zero => 0,
			Self::One(..) => 1,
			Self::Two(..) => 2,
		}
	}
	pub fn better_than(&self, other: CreatedSplits) {
		self.count() < other.count();
	}
	pub fn valid(&self) -> bool {
		match self {
			Self::Failed => false,
			_ => true,
		}
	}
	pub fn insert_and_split(
		im: rect_structs::RectWH,
		sp: rect_structs::SpaceRect,
	) -> Self {
		let free_w = sp.w - im.w;
		let free_h = sp.h - im.h;
		if free_w < 0 || free_h < 0 {
			Self::Failed
		} else if free_w == 0 && free_h == 0 {
			Self::Zero
		} else if free_w > 0 && free_h == 0 {
			let mut r = sp;
			r.x += im.w;
			r.w -= im.w;
			Self::One(r)
		} else if free_w == 0 && free_h > 0 {
			let mut r = sp;
			r.y += im.h;
			r.h -= im.h;
			Self::One(r)
		} else if free_w > free_h {
			Self::Two(
				rect_structs::SpaceRect::from(sp.x + im.w, sp.y, free_w, sp.h),
				rect_structs::SpaceRect::from(sp.x, sp.y + im.h, im.w, free_h),
			)
		} else {
			Self::Two(
				rect_structs::SpaceRect::from(sp.x, sp.y + im.h, im.w, free_h),
				rect_structs::SpaceRect::from(sp.x + im.w, sp.y, free_w, sp.h),
			)
		}
	}

}
