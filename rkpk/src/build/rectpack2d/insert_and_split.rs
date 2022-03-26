use crate::common::{RectWH, RectXYWH};

#[derive(Debug, Copy, Clone)]
pub enum CreatedSplits {
	Failed,
	Zero,
	One(RectXYWH),
	Two(RectXYWH, RectXYWH),
}

impl CreatedSplits {
	pub fn vec(&self) -> Vec<RectXYWH> {
		match self {
			Self::Failed => vec![],
			Self::Zero => vec![],
			Self::One(a) => vec![*a],
			Self::Two(a, b) => vec![*a, *b],
		}
	}
	pub fn valid(&self) -> bool {
		!matches!(self, Self::Failed)
	}
	pub fn new(im: RectWH, sp: RectXYWH) -> Self {
		// unsigned integer moment
		if im.w > sp.w || im.h > sp.h {
			return Self::Failed;
		}
		let free_w = sp.w - im.w;
		let free_h = sp.h - im.h;
		if free_w == 0 && free_h == 0 {
			Self::Zero
		} else if free_w > 0 && free_h == 0 {
			Self::One(RectXYWH::new(
				sp.x + im.w,
				sp.y,
				sp.w - im.w,
				sp.h,
			))
		} else if free_w == 0 && free_h > 0 {
			Self::One(RectXYWH::new(
				sp.x,
				sp.y + im.h,
				sp.w,
				sp.h - im.h,
			))
		} else if free_w > free_h {
			Self::Two(
				RectXYWH::new(sp.x + im.w, sp.y, free_w, sp.h),
				RectXYWH::new(sp.x, sp.y + im.h, im.w, free_h),
			)
		} else {
			Self::Two(
				RectXYWH::new(sp.x, sp.y + im.h, im.w, free_h),
				RectXYWH::new(sp.x + im.w, sp.y, free_w, sp.h),
			)
		}
	}
}
