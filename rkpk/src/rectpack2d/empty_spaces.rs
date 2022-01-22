// port of empty_spaces.h

// deps: insert_and_split
// stat: done?

use super::insert_and_split;
use super::rect_structs;

pub enum FlippingOption {
	Disabled,
	Enabled,
}

pub struct EmptySpaces<OutputRectType, EmptySpacesProvider> {
	current_aabb: rect_structs::RectWH,
	spaces: EmptySpacesProvider,
	pub flipping_mode: FlippingOption,
}

impl<OutputRectType, EmptySpacesProvider> EmptySpaces<OutputRectType, EmptySpacesProvider> {
	pub fn new(r: rect_structs::RectWH) -> Self {
		let mut res = Self {
			current_aabb: Default::default(),
			spaces: Default::default(),
			flipping_mode: FlippingOption::Enabled
		};
		res.reset(r);
		res
	}
	pub fn reset(&mut self, r: rect_structs::RectWH) {
		self.current_aabb = r;
		self.spaces.reset();
		self.spaces.add(rect_structs::RectXYWH::from(0, 0, r.w, r.h));
	}
	pub fn insert<F>(
		&self,
		image_rectangle: rect_structs::RectWH,
		report_candidate_empty_space: F
	) -> Option<OutputRectType> {
		for i in (0..self.spaces.get_count()).rev() {
			let candidate_space = self.spaces.get(i);
			report_candidate_empty_space(candidate_space);
			macro_rules! accept_result {
				($splits:expr, $flipping:expr) => {{
					self.spaces.remove(i);
					for i in 0..$splits.count() {
						if !self.spaces.add($splits.spaces[i]) {
							return None;
						}
					}
					let result = OutputRectType::make(
						candidate_space.x,
						candidate_space.y,
						image_rectangle.w,
						image_rectangle.h,
						$flipping,
					);
					self.current_aabb.expand_with_mut(result);
					result
				}};
			}
			macro_rules! try_to_insert {
				($img:expr) => {
					insert_and_split::CreatedSplits::insert_and_split($img, candidate_space)
				};
			}
			if !<OutputRectType>::ALLOW_FLIP || self.flipping_mode != FlippingOption::Enabled {
				let normal = try_to_insert!(image_rectangle);
				if normal.valid() {
					return accept_result!(normal, false);
				} else {
					None
				}
			} else {
				let normal = try_to_insert!(image_rectangle);
				let flipped = try_to_insert!(image_rectangle.get_wh().flip());
				if normal.valid() && flipped.valid() {
					if flipped.better_than(normal) {
						accept_result!(flipped, true)
					} else {
						accept_result!(normal, false)
					}
				} else if normal.valid() {
					 accept_result!(normal, false)
				} else if flipped.valid() {
					accept_result!(flipped, true)
				} else {
					None
				}
			}
		}
	}
	pub fn insert_silent(&self, image_rectangle: rect_structs::RectWH) -> Option<OutputRectType> {
		self.insert(image_rectangle, |_|return)
	}
}
