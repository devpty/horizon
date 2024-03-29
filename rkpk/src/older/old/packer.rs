use std::{collections, fmt};
// use std::hash::{Hash, Hasher};
use super::rectpack2d;

#[derive(Copy, Clone)]
pub enum PackerImage {
	Unique {
		image: crate::Image,
		packed_loc: (u32, u32),
		source_pos: crate::Rect,
	},
	// index in vector
	Duplicate(PackerKey, usize),
}

impl fmt::Debug for PackerImage {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unique {
				image, packed_loc, source_pos
			} => {
				f.write_fmt(format_args!("({:?}, {:?}, {:?})", image, source_pos, packed_loc))
			},
			Self::Duplicate(key, idx) => {
				f.write_fmt(format_args!("(ref {:?}:{})", key, idx))
			}
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PackerKey {
	pub id: &'static str,
	pub layer: Option<&'static str>,
}

impl fmt::Debug for PackerKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{:?}:{:?}", self.layer, self.id))
	}
}

#[derive(Debug)]
pub struct Packer {
	pub allow_flipping: bool,
	images: collections::HashMap<PackerKey, Vec<PackerImage>>,
	packed_size: (u32, u32),
}

impl Packer {
	pub fn new() -> Self {
		Self {
			allow_flipping: false,
			images: collections::HashMap::new(),
			packed_size: (0, 0),
		}
	}
	pub fn allow_flipping(&mut self, v: bool) {
		self.allow_flipping = v;
	}
	pub fn add_image(&mut self, id: &'static str, layer: Option<&'static str>, cache: &mut crate::ImageCache, image: crate::Image, ty: crate::ImageType) {
		self.images.insert(
			PackerKey {id, layer},
			ty.to_rects(&image, cache).iter().map(|v| PackerImage::Unique {image, source_pos: *v, packed_loc: (0, 0)}).collect()
		);
	}
	pub fn dedup(&mut self, cache: &mut crate::ImageCache) {
		let mut set = collections::HashMap::<Vec<u8>, (PackerKey, usize)>::new();
		for i in self.images.iter_mut() {
			println!("checking {:?}", i.0);
			for j in i.1.iter_mut().enumerate() {
				print!("- image {:>3} ({:?}): ", j.0, j.1);
				match j.1 {
					PackerImage::Unique {
						image, source_pos, ..
					} => {
						let bytes = image.to_entry(cache).crop(*source_pos).into_raw();
						if set.contains_key(&bytes) {
							// unwrap: guaranteed to not fail
							let dup = set.get(&bytes).unwrap();
							println!("duplicate {:?}:{}", dup.0, dup.1);
							// mark it as a duplicate
							*j.1 = PackerImage::Duplicate(dup.0, dup.1);
						} else {
							println!("unique {}", j.0);
							set.insert(bytes, (*i.0, j.0));
						}
					},
					PackerImage::Duplicate(key, idx) => {
						println!("pre-duplicate {:?}:{}", key, idx);
					},
				}
			}
		}
	}
	pub fn pack(&mut self) {
		let mut rects_by_layer = collections::HashMap::<Option<&str>, collections::HashMap<&str, &mut Vec<PackerImage>>>::new();
		for (i, iv) in self.images.iter_mut() {
			// id's can be duplicated between layers
			let entry = rects_by_layer.entry(i.layer).or_default();
			// todo: all rects of the same id & index should have the same resolution
			// but they're unique within layers
			if entry.insert(i.id, iv).is_some() {
				// panic: user error, not my problem
				panic!("duplicate identifier {}", i.id);
			}
		}
		// iterate through atlases and generate
		for (_, mut iv) in rects_by_layer {
			let mut rects_to_place = Vec::new();
			let mut rects_info = Vec::new();
			for (j, jv) in iv.iter() {
				for (k, kv) in jv.iter().enumerate() {
					// todo: image size equality checks
					match kv {
						PackerImage::Unique {
							source_pos, ..
						} => {
							// ignore packed_loc's position as it gets re-packed anyways
							let rect = rectpack2d::rect_structs::RectXYWHF::new(0, 0, source_pos.2, source_pos.3, false);
							rects_to_place.push(rect);
							rects_info.push((*j, k))
						},
						// don't handle duplicates as their originals are already handled
						PackerImage::Duplicate(..) => {},
					}
				}
			}
			let rect_size = rectpack2d::finders_interface::find_best_packing(
				&mut rects_to_place,
				rectpack2d::finders_interface::FinderInput {
					start_size: 16384,
					discard_step: rectpack2d::finders_interface::DiscardStep::Tries(4),
					allow_flipping: true,
				},
				rectpack2d::finders_interface::DEFAULT_COMPARATORS,
			).unwrap(); // panic-fail: can fail if packing fails
			self.packed_size = (rect_size.w, rect_size.h);
			for (rect, (j, k)) in rects_to_place.iter().zip(rects_info.iter()) {
				// panic: will never fail (×2)
				match iv.get_mut(*j).unwrap().get_mut(*k).unwrap() {
					PackerImage::Unique { packed_loc, .. } => *packed_loc = (rect.x, rect.y),
					_ => {}
				}
			}
		}
	}
}
