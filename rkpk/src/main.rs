fn main() {
	let mut cache = rkpk::ImageCache::new();
	let mut packer = rkpk::Packer::new(&mut cache);
	packer.add_new("font", None,
		"rkpk/src/assets/font.png", rkpk::Image::File,
		rkpk::ImageLoad::Tiled {
			init: ((0, 0), (8, 16)),
			gap: (0, 0),
			count: (16, 8),
		}
	).unwrap();
	packer.add_new("logo", None,
		"rkpk/src/assets/logo.png", rkpk::Image::File,
		rkpk::ImageLoad::Whole
	).unwrap();
	packer.deduplicate().unwrap();
	packer.pack();
	println!("{:#?}", packer);
	// println!("{:#?}", cache);
}
