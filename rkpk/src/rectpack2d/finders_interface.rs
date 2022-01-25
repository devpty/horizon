use std::cmp;

use super::rect_structs;
use super::best_bin_finder;

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
