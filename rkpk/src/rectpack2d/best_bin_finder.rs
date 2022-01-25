use super::rect_structs;

enum BinDimension {
	Both, Width, Height
}

enum BestPackingForOrderingResult {
	TotalArea(u32),
	Rect(rect_structs::RectWH),
}


pub fn find_best_packing_impl<RectT: rect_structs::OutputRect>(
	orders: Vec<Vec<&mut RectT>>,
	input: finders_interface::FinderInput,
) -> Option<rect_structs::RectWH> {
	let max_bin = rect_structs::RectWH::from(input.start_size, input.start_size);
	// Option<&Vec<&mut RectT>>
	let mut best_order = None;
	let mut best_bin = max_bin;
	let root = empty_spaces::EmptySpaces::new();
	root.allow_flipping = input.allow_flipping;
	for current_order in orders {
		// continue here
	}
}