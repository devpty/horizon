use super::rect_structs;

pub enum CreatedSplits {
	Failed,
	Zero,
	One(rect_structs::RectXYWH),
	Two(rect_structs::RectXYWH, rect_structs::RectXYWH)
}

impl CreatedSplits {
	pub fn count(&self) -> i8 {
		match Self {
			Self::Failed  => -1,
			Self::Zero    => 0,
			Self::One(..) => 1,
			Self::Two(..) => 2,
		}
	}
	pub fn better_than(&self, other: CreatedSplits) -> bool {
		// this seems to consider zero splits better than one or two,
		// i guess we just prefer areas that get filled up completely
		self.count() < other.count()
	}
	pub fn valid(&self) -> bool {
		if let Failed = self {
			false
		} else {
			true
		}
	}
	pub fn new(
		im: rect_structs::RectWH,
		sp: rect_structs::RectXYWH,
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