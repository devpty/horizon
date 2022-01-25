use super::insert_and_split;
use super::rect_structs;

pub struct EmptySpaces<RectT: rect_structs::OutputRect> {
	current_aabb: rect_structs::RectWH,
	spaces: Vec<rect_structs::RectXYWH>,
	allow_flipping: bool,
}

enum InsertResult {
	Failed,
	Normal(insert_and_split::CreatedSplits),
	Flipped(insert_and_split::CreatedSplits),
}

impl<RectT: rect_structs::OutputRect> EmptySpaces<RectT> {
	pub fn new(allow_flipping: bool) -> Self {
		Self {
			current_aabb: rect_structs::RectWH::default(),
			spaces: Vec::new(),
			allow_flipping,
		}
	}
	pub fn reset(&mut self, r: rect_structs::RectWH) {
		self.current_aabb = rect_structs::RectWH::default();
		spaces.clear();
		spaces.push(rect_structs::RectXYWH::new(0, 0, r.w, r.h));
	}
	pub fn insert(&mut self, image_rectangle: rect_structs::RectWH) -> Option<RectT> {
		for candidate_space in self.spaces.iter().rev() {
			let res = if <RectT>::ALLOW_FLIP && self.allow_flipping {
				let normal = insert_and_split::CreatedSplits::new(image_rectangle.get_wh(), candidate_space);
				let flipped = insert_and_split::CreatedSplits::new(image_rectangle.get_wh().flip(), candidate_space);
				if normal.valid() && flipped.valid() {
					if flipped.better_than(normal) {
						InsertResult::Flipped(flipped)
					} else {
						InsertResult::Normal(normal)
					}
				} else if normal.valid() {
					InsertResult::Normal(normal)
				} else if flipped.valid() {
					InsertResult::Flipped(normal)
				} else {
					InsertResult::Failed
				}
			} else {
				let normal = insert_and_split::CreatedSplits::new(image_rectangle.get_wh(), candidate_space);
				if normal.valid() {
					InsertResult::Normal(normal)
				} else {
					InsertResult::Failed
				}
			};
			let (flipping_necessary, splits) = match res {
				InsertResult::Failed => return None,
				InsertResult::Normal(rect) => (false, rect),
				InsertResult::Flipped(rect) => (true, rect),
			};
			self.spaces.remove(candidate_space);
			for split in splits.iter() {
				if !self.spaces.add(split) {
					return None;
				}
			}
			// no allow_flip shit here since !allow_flipping will never call the flipping codepath
			let result = RectT::from_xywhf(
				candidate_space.x,
				candidate_space.y,
				image_rectangle.w,
				image_rectangle.h,
				flipping_necessary,
			);
			self.current_aabb.expand_with_mut(result);
			result
		}
	}
}