use crate::common::ImageRect;
use crate::common::ImageSize;

use super::empty_spaces;
use super::finders_interface;

#[derive(Debug, Copy, Clone)]
pub enum BinDimension {
	Both,
	Width,
	Height,
}

#[derive(Debug, Copy, Clone)]
pub enum BestPackingForOrderingResult {
	TotalArea(u32),
	Rect(ImageSize),
}

fn best_packing_for_ordering_impl(
	root: &mut empty_spaces::EmptySpaces,
	ordering: &Vec<&mut ImageRect>,
	starting_bin: ImageSize,
	discard_step: finders_interface::DiscardStep,
	tried_dimension: BinDimension,
) -> BestPackingForOrderingResult {
	let mut candidate_bin = starting_bin;
	let (discard_step, mut tries_before_discarding) = match discard_step {
		finders_interface::DiscardStep::Step(step) => (step, 0),
		finders_interface::DiscardStep::Tries(tries) => (1, tries),
	};
	let starting_step = match tried_dimension {
		BinDimension::Both => {
			candidate_bin.0 /= 2;
			candidate_bin.1 /= 2;
			// why the width here?
			candidate_bin.0 / 2
		}
		BinDimension::Width => {
			candidate_bin.0 /= 2;
			candidate_bin.0 / 2
		}
		BinDimension::Height => {
			candidate_bin.1 /= 2;
			candidate_bin.1 / 2
		}
	};
	let mut step = starting_step;
	loop {
		root.reset(candidate_bin);
		let mut total_inserted_area = 0;
		// in c++ this is a lambda, that's stupid
		let all_inserted = 'ch: {
			for rect in ordering {
				match root.insert(**rect) {
					Some(_) => total_inserted_area += rect.2 as u32 * rect.3 as u32,
					None => break 'ch false,
				}
			}
			true
		};
		if all_inserted {
			if step <= discard_step {
				if tries_before_discarding > 0 {
					tries_before_discarding -= 1;
				} else {
					return BestPackingForOrderingResult::Rect(candidate_bin);
				}
			}
			match tried_dimension {
				BinDimension::Both => {
					candidate_bin.0 -= step;
					candidate_bin.1 -= step;
				}
				BinDimension::Width => candidate_bin.0 -= step,
				BinDimension::Height => candidate_bin.1 -= step,
			}
		} else if match tried_dimension {
			BinDimension::Both => {
				candidate_bin.0 += step;
				candidate_bin.1 += step;
				candidate_bin.0 as u32 * candidate_bin.1 as u32 > starting_bin.0 as u32 * starting_bin.1 as u32
			}
			BinDimension::Width => {
				candidate_bin.0 += step;
				candidate_bin.0 > starting_bin.0
			}
			BinDimension::Height => {
				candidate_bin.1 += step;
				candidate_bin.1 > starting_bin.1
			}
		} {
			return BestPackingForOrderingResult::TotalArea(total_inserted_area);
		}
		step = 1.max(step / 2);
	}
}

pub fn best_packing_for_ordering(
	root: &mut empty_spaces::EmptySpaces,
	ordering: &Vec<&mut ImageRect>,
	starting_bin: ImageSize,
	discard_step: finders_interface::DiscardStep,
) -> BestPackingForOrderingResult {
	macro_rules! try_pack {
		($tried:expr, $starting:expr) => {
			best_packing_for_ordering_impl(root, ordering, $starting, discard_step, $tried)
		};
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
				_ => {}
			}
		};
	}
	trial!(BinDimension::Width);
	trial!(BinDimension::Height);
	BestPackingForOrderingResult::Rect(best_bin)
}
