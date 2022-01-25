use std::cmp;

use super::rect_structs;
use super::best_bin_finder;

use rect_structs::{Rect};

type Comparator = dyn Fn(rect_structs::RectWH, rect_structs::RectWH) -> cmp::Ordering;

pub enum DiscardStep {
	Tries(u32),
	Step(u32),
}

pub struct FinderInput {
	pub(crate) start_size: u32,
	discard_step: DiscardStep,
	allow_flipping: bool,
}

pub fn find_best_packing<RectT: rect_structs::OutputRect>(
	subjects: &mut Vec<RectT>,
	input: FinderInput,
	comparators: &Vec<&Comparator>,
) -> Option<rect_structs::RectWH> {
	// rect_type = RectT
	// order_type = Vec<RectT>
	let mut orders = Vec::with_capacity(comparators.len());
	let template_vec = subjects.iter_mut().collect::<Vec<_>>();
	for comparator in comparators {
		// RectT has clone but rls thinks not!
		let mut copy: Vec<&mut RectT> = template_vec.clone();
		copy.sort_by(|a, b| comparator(a.get_wh(), b.get_wh()));
		orders.push(copy);
	}
	best_bin_finder::find_best_packing_impl(orders, input)
}

pub const DEFAULT_COMPARATORS: &[&Comparator] = &[
	&|a, b| a.area().cmp(&b.area()),
	&|a, b| a.perimeter().cmp(&b.perimeter()),
	&|a, b| a.max_size().cmp(&b.max_size()),
	&|a, b| a.w.cmp(&b.w),
	&|a, b| a.h.cmp(&b.h),
	&|a, b| a.path_mul().partial_cmp(&b.path_mul()).unwrap_or(std::cmp::Ordering::Equal)
];