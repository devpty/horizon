use std::cmp;

use crate::common::ImageSize;

use super::best_bin_finder::{best_packing_for_ordering, BestPackingForOrderingResult};
use super::empty_spaces::EmptySpaces;

type Comparator = dyn Fn(ImageSize, ImageSize) -> cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub enum DiscardStep {
	Tries(u16),
	Step(u16),
}

pub fn find_best_packing(
	subjects: &mut Vec<RectXYWH>, // todo here, replace shit
	start_size: u32,
	discard_step: DiscardStep,
	comparators: &[&Comparator],
) -> Option<RectWH> {
	let mut mut_vec = subjects.iter_mut().collect::<Vec<_>>();
	let max_bin = RectWH::new(start_size, start_size);
	let mut best_order = None;
	let mut best_total_inserted = 0;
	let mut best_bin = max_bin;
	let mut root = EmptySpaces::new();
	for comparator in comparators {
		mut_vec.sort_by(|a, b| comparator(a.to_wh(), b.to_wh()));
		match best_packing_for_ordering(&mut root, &mut_vec, max_bin, discard_step) {
			BestPackingForOrderingResult::TotalArea(total_inserted) => {
				if best_order.is_none() && total_inserted > best_total_inserted {
					best_order = Some(comparator);
					best_total_inserted = total_inserted;
				}
			}
			BestPackingForOrderingResult::Rect(result_bin) => {
				// this will be like 0.0001% faster if i change the <= with a <
				// that messes up the case where the smallest area is equal to the bin area
				if result_bin.area() <= best_bin.area() {
					best_order = Some(comparator);
					best_bin = result_bin;
				}
			}
		}
	}
	let best_order = match best_order {
		Some(v) => v,
		None => return None,
	};
	root.reset(best_bin);
	mut_vec.sort_by(|a, b| best_order(a.to_wh(), b.to_wh()));
	for rect in mut_vec {
		match root.insert(*rect) {
			Some(res) => *rect = res,
			None => return None,
		}
	}
	Some(root.current_aabb)
}

pub const DEFAULT_COMPARATORS: &[&Comparator; 6] = &[
	&|a, b| a.area().cmp(&b.area()),
	&|a, b| a.perimeter().cmp(&b.perimeter()),
	&|a, b| a.max_size().cmp(&b.max_size()),
	&|a, b| a.w.cmp(&b.w),
	&|a, b| a.h.cmp(&b.h),
	&|a, b| {
		a.path_mul()
			.partial_cmp(&b.path_mul())
			.unwrap_or(cmp::Ordering::Equal)
	},
];
