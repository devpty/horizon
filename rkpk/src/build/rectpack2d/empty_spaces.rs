use crate::common::{RectWH, RectXYWH};

use super::insert_and_split::CreatedSplits;

pub struct EmptySpaces {
	pub current_aabb: RectWH,
	pub spaces: Vec<RectXYWH>,
}

impl EmptySpaces {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			current_aabb: RectWH::new(0, 0),
			spaces: Vec::new(),
		}
	}
	pub fn reset(&mut self, r: RectWH) {
		self.current_aabb = RectWH::new(0, 0);
		self.spaces.clear();
		self.spaces.push(RectXYWH::new(0, 0, r.w, r.h));
	}
	pub fn insert(&mut self, image_rectangle: RectXYWH) -> Option<RectXYWH> {
		for (i, candidate_space) in self.spaces.clone().iter().enumerate().rev() {
			let normal = CreatedSplits::new(
				RectWH::new(image_rectangle.x, image_rectangle.y),
				*candidate_space,
			);
			let res = if normal.valid() {
				Option::Some(normal)
			} else {
				Option::None
			};
			let splits = match res {
				Option::None => continue,
				Option::Some(rect) => rect,
			};
			self.spaces.remove(i);
			for split in splits.vec() {
				self.spaces.push(split);
			}
			// no allow_flip shit here since !allow_flipping will never call the flipping codepath
			let result = RectXYWH::new(
				candidate_space.x,
				candidate_space.y,
				image_rectangle.w,
				image_rectangle.h,
			);
			self.current_aabb.w = self.current_aabb.w.max(result.x + result.w);
			self.current_aabb.h = self.current_aabb.h.max(result.y + result.h);
			return Some(result);
		}
		None
	}
}
