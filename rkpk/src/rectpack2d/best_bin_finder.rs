use super::empty_space_allocators;
use super::empty_spaces;
use super::rect_structs;

enum BinDimension {
	Both, Width, Height
}

enum BestPackingForOrderingResult {
	TotalArea(u32),
	Rect(rect_structs::RectWH),
}

fn best_packing_for_ordering_impl<RectT: rect_structs::OutputRect>(
	root: &mut empty_spaces::EmptySpaces<RectT>,
	ordering: &Vec<&RectT>,
	starting_bin: rect::RectWH,
	discard_step: finders_interface::DiscardStep,
	tried_dimension: BinDimension,
) -> BestPackingForOrderingResult {
	let mut candidate_bin = starting_bin;
	let (discard_step, mut tries_before_discarding) = match discard_step {
		finders_interface::DiscardStep::Step(step) => (step, 0),
		finders_interface::DiscardStep::Tries(tries) => (1, tries),
	}
	let starting_step = match tried_dimension {
		BinDimension::Both => {
			candidate_bin.w /= 2;
			candidate_bin.h /= 2;
			// why the width here?
			candidate_bin.w / 2
		},
		BinDimension::Width => {
			candidate_bin.w /= 2;
			candidate_bin.w / 2
		},
		BinDimension::Height => {
			candidate_bin.h /= 2;
			candidate_bin.h / 2
		}
	};
	let mut step = starting_step;
	loop {
		root.reset(candidate_bin);
		let mut total_inserted_area = 0;
		// in c++ this is a lambda, that's stupid
		let all_inserted = 'check: {
			for rect in ordering {
				match root.insert(rect.get_wh()) {
					Some(_) => total_inserted_area += rect.area(),
					None => break 'check false;
				}
			}
			true
		};
		if all_inserted {
			if step <= discard_step {
				if tries_before_discarding > 0 {
					tries_before_discarding -= 1;
				} else {
					return candidate_bin;
				}
			}
			match tried_dimension {
				BinDimension::Both => {
					candidate_bin.w -= step;
					candidate_bin.h -= step;
				},
				BinDimension::Width => candidate_bin.w -= step,
				BinDimension::Height => candidate_bin.h -= step,
			}
		} else {
			if match tried_dimension {
				BinDimension::Both => {
					candidate_bin.w += step;
					candidate_bin.h += step;
					candidate_bin.area() > starting_bin.area()
				},
				BinDimension::Width => {
					candidate_bin.w += step;
					candidate_bin.w > starting_bin.w
				},
				BinDimension::Height => {
					candidate_bin.h += step;
					candidate_bin.h > starting_bin.h
				}
			} {
				return total_inserted_area;
			}
		}
		step = 1.max(step / 2);
	}
}

fn best_packing_for_ordering<RectT: rect_structs::OutputRect>(
	root: &mut empty_spaces::EmptySpaces<RectT>,
	ordering: &Vec<&RectT>,
	starting_bin: rect_structs::RectWH,
	discard_step: finders_interface::DiscardStep
) -> BestPackingForOrderingResult {
	macro_rules! try_pack {
		($tried:expr, $starting:expr) => {
			best_packing_for_ordering_impl(root, ordering, $starting, discard_step, $tried)
		}
	}
	let mut best_bin = match try_pack!(BinDimension::Both, starting_bin) {
		BestPackingForOrderingResult::Rect(rect) => rect,
		// if it fails then we immediately return
		// there's probably a mathematical reason for this but i don't know it
		best_result => return best_result,
	};
	macro_rules! trial {
		($tried:expr) => {
			match try_pack!($tried, best_bin) {
				BestPackingForOrderingResult::Rect(rect) => best_bin = rect,
				_ => {},
			}
		}
	}
	trial!(BinDimension::Width);
	trial!(BinDimension::Height);
	return best_bin;
}


pub fn find_best_packing_impl<RectT: rect_structs::OutputRect>(
	orders: Vec<Vec<&mut RectT>>,
	input: finders_interface::FinderInput,
) -> Option<rect_structs::RectWH> {
	let max_bin = rect_structs::RectWH::from(input.start_size, input.start_size);
	// Option<&Vec<&mut RectT>>
	let mut best_order = None;
	let mut best_total_inserted = 0;
	let mut best_bin = max_bin;
	let mut root = empty_spaces::EmptySpaces::new(input.allow_flipping);
	for current_order in orders {
		match best_packing_for_ordering(&mut root, current_order, max_bin, input.discard_step) {
			TotalArea(total_inserted) => {
				if best_order.is_none() && total_inserted > best_total_inserted {
					best_order = Some(current_order);
					best_total_inserted = total_inserted;
				}
			},
			Rect(result_bin) => {
				// this will be like 0.0001% faster if i change the <= with a <
				// that messes up the case where the smallest area is equal to the bin area
				if result_bin.area() <= best_bin.area() {
					best_order = current_order;
					best_bin = result_bin;
				}
			}
		}
	}
	// unwrap: used to replace an assert in the source, could probably become a `return None`
	let best_order = best_order.unwrap();
	root.reset(best_bin);
	for rect of best_order {
		match root.insert(rect) {
			Some(res) => rect = res,
			None => return None,
		}
	}
	// original implementation always returned, even on packing failure
	// that's a bad idea since our implementation doesn't have packing callbacks
	Some(root.get_rects_aabb())
}