fn main() {
	let mut cache = rkpk::ImageCache::new();
	let mut packer = rkpk::Packer::new();
	packer.allow_flipping(false);
	packer.add_image("font", None, &mut cache,
		rkpk::Image::External("rkpk/src/assets/font.png"),
		rkpk::ImageType::Tiled((0, 0), (8, 16), (0, 0), (16, 8)));
	packer.add_image("logo", None, &mut cache,
		rkpk::Image::External("rkpk/src/assets/logo.png"),
		rkpk::ImageType::Whole);
	packer.dedup(&mut cache);
	packer.pack(&mut cache);
	// println!("{:#?}", packer);
	println!("{:#?}", cache);
}
