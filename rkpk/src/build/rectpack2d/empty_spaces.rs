use crate::common::{ImageSize, ImageRect};

use super::insert_and_split;

pub struct EmptySpaces {
	pub current_aabb: ImageSize,
	pub spaces: Vec<ImageRect>,
}

impl EmptySpaces {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			current_aabb: (0, 0),
			spaces: Vec::new(),
		}
	}
	pub fn reset(&mut self, r: ImageSize) {
		self.current_aabb = (0, 0);
		self.spaces.clear();
		self.spaces.push((0, 0, r.0, r.1));
	}
	pub fn insert(&mut self, image_rectangle: ImageRect) -> Option<ImageRect> {
		for (i, candidate_space) in self.spaces.clone().iter().enumerate().rev() {
			let normal =
				insert_and_split::CreatedSplits::new((image_rectangle.0, image_rectangle.1), *candidate_space);
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
			let result = (
				candidate_space.0,
				candidate_space.1,
				image_rectangle.2,
				image_rectangle.3,
			);
			self.current_aabb.0 = self.current_aabb.0.max(result.0 + result.2);
			self.current_aabb.1 = self.current_aabb.1.max(result.1 + result.3);
			return Some(result);
		}
		None
	}
}
