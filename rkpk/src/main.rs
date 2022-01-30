fn main() {
	let mut cache = rkpk::ImageCache::new();
	let mut packer = rkpk::Packer::new(&mut cache);
	packer.allow_flipping(true);
	packer.add_image("font", None,
		rkpk::Image::External("rkpk/src/assets/font.png"),
		rkpk::ImageType::Tiled((0, 0), (8, 16), (0, 0), (16, 8)));
	packer.add_image("logo", None,
		rkpk::Image::External("rkpk/src/assets/logo.png"),
		rkpk::ImageType::Whole);
	packer.dedup();
	packer.pack();
	println!("{:#?}", packer);
	println!("{:#?}", cache);
}
