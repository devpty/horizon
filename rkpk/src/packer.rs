use std::collections;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Packer {
	pub allow_flipping: bool,
	images: collections::HashMap<crate::PackerKey, Vec<crate::PackerImage>>,
}

impl Packer {
	pub fn new() -> Self {
		Self {
			allow_flipping: false,
			images: collections::HashMap::new()
		}
	}
	pub fn allow_flipping(mut self, v: bool) -> Self {
		self.allow_flipping = v;
		self
	}
	pub fn add_image(mut self, id: &'static str, layer: Option<&'static str>, cache: &mut crate::ImageCache, image: crate::Image, ty: crate::ImageType) -> Self {
		self.images.insert(
			crate::PackerKey {id, layer},
			ty.to_rects(&image, cache).iter().map(|v| crate::PackerImage {image, source_loc: (v.0, v.1), packed_pos: crate::Rect(0, 0, v.2, v.3, v.4)}).collect()
		);
		self
	}
	pub fn dedup(mut self, cache: &mut crate::ImageCache) -> Self {
		let mut set = collections::HashSet::new();
		for i in self.images.iter() {
			println!("checking {:?}", i.0);
			for j in i.1.iter().enumerate() {
				print!("- image {:>3} ({:?}): ", j.0, j.1);
				let bytes = j.1.image.to_entry(cache).data.clone().into_raw();
				if set.contains(&bytes) {
					println!("duplicate");
				} else {
					println!("unique");
					set.insert(bytes);
				}
			}
		}
		self
	}
	pub fn pack(mut self, cache: &mut crate::ImageCache) -> Self {
		self
	}
}
