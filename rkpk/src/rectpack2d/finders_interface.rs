// port of finders_interface.h

// deps: insert_and_split, empty_spaces, [empty_space_allocators], best_bin_finder
// stat: almost complete, needs checking after deps are complete

use super::rect_structs;
use super::empty_spaces;
use rect_structs::Rect;

pub struct FinderInput<F, G>
where F: Fn() -> bool, G: Fn() -> bool {
	pub max_bin_size: u32,
	pub discard_step: u32,
	pub handle_successful_insertion: F,
	pub handle_unsuccessful_insertion: G,
	pub flipping_mode: empty_spaces::FlippingOption,
}

type Comparator = dyn Fn(rect_structs::RectWH, rect_structs::RectWH) -> bool;

fn find_best_packing_dont_sort<OutputRect: rect_structs::OutputRectType, EmptySpacesT, F: Fn() -> bool, G: Fn() -> bool>(
	subjects: &Vec<OutputRect>,
	input: &FinderInput<F, G>,
	comparators: Vec<&Comparator>,
) -> rect_structs::RectWH {
	let count_orders = comparators.len();
	let orders = vec![Vec::new(); count_orders];
	{ // todo: why do we need this block
		let initial_pointers = orders[0];
		for s in subjects {
			if s.area() > 0 {
				initial_pointers.push(s);
			}
		}
		for i in 0..count_orders {
			orders[i] = initial_pointers;
		}
	}
	for i in 0..count_orders {
		orders[i].sort_by(|a, b|
			if comparators[i](a.get_wh(), b.get_wh()) {
				std::cmp::Ordering::Greater
			} else {
				std::cmp::Ordering::Less
			}
		);
	}
	find_best_packing_impl::<OutputRect, EmptySpacesT>(
		|callback| for o in orders {callback(o)},
		input
	)
}

const DEFAULT_COMPARATORS: &[&Comparator] = &[
	&|a, b| a.area() > b.area(),
	&|a, b| a.perimeter() > b.perimeter(),
	&|a, b| a.max_size() > b.max_size(),
	&|a, b| a.w > b.w,
	&|a, b| a.h > b.h,
	&|a, b| a.path_mul() > b.path_mul(),
];
