use super::insert_and_split;
use super::rect_structs::{RectWH, RectXYWH};

pub struct EmptySpaces {
	pub current_aabb: RectWH,
	pub spaces: Vec<RectXYWH>,
	_marker: std::marker::PhantomData<RectXYWH>,
}

enum InsertResult {
	Failed,
	Normal(insert_and_split::CreatedSplits),
	Flipped(insert_and_split::CreatedSplits),
}

impl EmptySpaces {
	pub fn new() -> Self {
		Self {
			current_aabb: RectWH::default(),
			spaces: Vec::new(),
			_marker: std::marker::PhantomData,
		}
	}
	pub fn reset(&mut self, r: RectWH) {
		self.current_aabb = RectWH::default();
		self.spaces.clear();
		self.spaces.push(RectXYWH::new(0, 0, r.w, r.h));
	}
	pub fn insert(&mut self, image_rectangle: RectXYWH) -> Option<RectXYWH> {
		for (i, candidate_space) in self.spaces.clone().iter().enumerate().rev() {
			let normal = insert_and_split::CreatedSplits::new(image_rectangle.to_wh(), *candidate_space);
			let res = if normal.valid() {
				InsertResult::Normal(normal)
			} else {
				InsertResult::Failed
			};
			let (flipping_necessary, splits) = match res {
				InsertResult::Failed => continue,
				InsertResult::Normal(rect) => (false, rect),
				InsertResult::Flipped(rect) => (true, rect),
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
			self.current_aabb.expand_with_mut(result);
			return Some(result)
		}
		None
	}
}
