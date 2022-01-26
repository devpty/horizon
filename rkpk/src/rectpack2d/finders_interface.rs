use std::{cell, cmp};

use super::rect_structs;
use super::best_bin_finder;
use rect_structs::Rect;

type Comparator = dyn Fn(rect_structs::RectWH, rect_structs::RectWH) -> cmp::Ordering;

#[derive(Debug, Copy, Clone)]
pub enum DiscardStep {
	Tries(u32),
	Step(u32),
}

#[derive(Debug, Copy, Clone)]
pub struct FinderInput {
	pub start_size: u32,
	pub discard_step: DiscardStep,
	pub allow_flipping: bool,
}

pub fn find_best_packing<RectT: rect_structs::OutputRect>(
	subjects: &mut Vec<RectT>,
	input: FinderInput,
	comparators: &[&Comparator],
) -> Option<rect_structs::RectWH> {
	// rect_type = RectT
	// order_type = Vec<RectT>
	let mut orders = Vec::with_capacity(comparators.len());
	let rc_vec = subjects.iter_mut().map(|n| cell::RefCell::new(n)).collect::<Vec<_>>();
	for comparator in comparators {
		let mut copy: Vec<_> = rc_vec.iter().collect();
		copy.sort_by(|a, b| comparator(a.borrow().get_wh(), b.borrow().get_wh()));
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
	&|a, b| a.path_mul().partial_cmp(&b.path_mul()).unwrap_or(cmp::Ordering::Equal)
];
