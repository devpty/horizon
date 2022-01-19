fn main() {
	let mut cache = rkpk::ImageCache::new();
	let packer = rkpk::Packer::new()
		.allow_flipping(false)
		.add_image("font", None, &mut cache,
			rkpk::Image::External("src/assets/font.png"),
			rkpk::ImageType::Tiled((0, 0), (8, 16), (0, 0), (16, 8)))
		.add_image("logo", None, &mut cache,
			rkpk::Image::External("src/assets/logo.png"),
			rkpk::ImageType::Whole)
		.dedup(&mut cache)
		.pack(&mut cache);
	//println!("{:#?}", packer);
	println!("{:#?}", cache);
}