use crate::ImageCache;

pub struct Packer<'a> {
	cache: &'a mut ImageCache,
}

impl<'a> Packer<'a> {
	pub fn new(cache: &'a mut ImageCache) -> Self {
		Self { cache }
	}
}
