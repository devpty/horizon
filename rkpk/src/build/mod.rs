use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{CompositeImage, ImageRect, ImageSize, RkPkResult};

pub mod rectpack2d;

#[derive(Debug)]
pub enum ImageSource {
	Path(PathBuf),
	Raw(CompositeImage),
}

impl ImageSource {
	fn load(&self) -> RkPkResult<CompositeImage> {
		Ok(match self {
			ImageSource::Path(v) => image::io::Reader::open(v)?.decode()?.to_rgba8().into(),
			ImageSource::Raw(v) => CompositeImage { size: v.size, data: v.data.clone() },
		})
	}
}

/// image rectangle templates
#[derive(Debug)]
pub enum ImageLoad {
	Whole,
	Tiled {
		init: ImageRect,
		gap: ImageSize,
		count: ImageSize,
	},
	Atlas(Vec<ImageRect>),
}

impl ImageLoad {
	fn into_rects(self, source: &ImageSource) -> RkPkResult<Vec<ImageRect>> {
		match self {
			ImageLoad::Whole => {
				let (w, h) = source.load()?.size;
				Ok(vec![(0, 0, w, h)])
			},
			ImageLoad::Tiled { init, gap, count } => Ok((0..count.0)
				.flat_map(|x| {
					(0..count.1).map(|y| {
						(
							init.0 + x * (init.2 + gap.0),
							init.1 + y * (init.3 + gap.1),
							init.2,
							init.3,
						)
					})
				})
				.collect()),
			ImageLoad::Atlas(v) => Ok(v),
		}
	}
}

/// packing things
#[derive(Debug)]
pub struct Packer {
	images: HashMap<(String, String), HashMap<String, (ImageSource, ImageLoad)>>,
}

const VALID_EXTENSIONS: &[&str] = &["png"];

fn valid_ext(v: &str) -> bool {
	let ext: String = v.chars().rev().take_while(|v| *v != '.').collect();
	let ext: String = ext.chars().rev().collect();
	for valid in VALID_EXTENSIONS {
		if ext == *valid {
			return true;
		}
	}
	false
}

impl Packer {
	pub fn new() -> Self {
		Self {
			images: HashMap::new(),
		}
	}
	fn set_images_ent(
		&mut self,
		group: String,
		layer: String,
		image: String,
		path: PathBuf,
		load: ImageLoad,
	) {
		println!(
			"cargo:warning=set_images_ent({:?}, {:?}, {:?}, {:?}, {:?})",
			group, layer, image, path, load
		);
		self.images
			.entry((group, layer))
			.or_insert_with(HashMap::new)
			.insert(image, (ImageSource::Path(path), load));
	}
	pub fn add_dir(&mut self, path: impl AsRef<Path>) -> RkPkResult<()> {
		// structure:
		// root/
		//  [group]/
		//   [layer].[ext]
		//   [layer].[ext].atlas
		//   [layer]/
		//    [image].[ext]
		//    [image].[ext].tiled
		//    [image].[ext].atlas
		println!("cargo:warning=graph:{:?}", path.as_ref());
		for group_ent in fs::read_dir(path.as_ref())? {
			let group_ent = group_ent?;
			println!("cargo:warning=group:{:?}", group_ent.path());
			if group_ent.file_type()?.is_dir() {
				let group_name = group_ent.file_name().to_string_lossy().into_owned();
				for layer_ent in fs::read_dir(group_ent.path())? {
					let layer_ent = layer_ent?;
					println!("cargo:warning=layer:{:?}", layer_ent.path());
					let layer_type = layer_ent.file_type()?;
					let layer_name = layer_ent.file_name().to_string_lossy().into_owned();
					if layer_type.is_dir() {
						let layer_path = layer_ent.path();
						for image_ent in fs::read_dir(layer_path.clone())? {
							let image_ent = image_ent?;
							println!("cargo:warning=image:{:?}", image_ent.path());
							if image_ent.file_type()?.is_file() {
								let image_name_os = image_ent.file_name();
								let image_name = image_name_os.to_string_lossy().into_owned();
								if valid_ext(&image_name) {
									// check for .tiled or .atlas
									let mut alt_path = layer_path.clone();
									{
										let mut image_name_tiled = image_name_os.clone();
										image_name_tiled.push(".tiled");
										alt_path.push(image_name_tiled);
									}
									if alt_path.exists() {
										// image is tiled
										todo!("image tiled support");
									} else {
										alt_path.pop();
										{
											let mut image_name_atlas = image_name_os.clone();
											image_name_atlas.push(".atlas");
											alt_path.push(image_name_atlas);
										}
										if alt_path.exists() {
											// image is atlas
											todo!("image atlas support");
										} else {
											// image is whole
											let image_name = image_ent
												.path()
												.file_stem()
												.unwrap()
												.to_string_lossy()
												.into_owned();
											self.set_images_ent(
												group_name.clone(),
												layer_name.clone(),
												image_name,
												image_ent.path(),
												ImageLoad::Whole,
											);
										}
									}
								}
							}
						}
					} else if layer_type.is_file() {
						todo!("layer atlas support");
					}
				}
			}
		}
		Ok(())
	}
	pub fn save_build_info(
		&mut self,
		meta_path: impl Into<String>,
		data_path: impl AsRef<Path>,
		asset_manager: &mut asset::build::Builder,
	) -> RkPkResult<()> {
		println!("cargo:warning={:?}", self);

		// // create a global vec<rect> + vec<assoc>
		// // for each group
		// //  create map<layer, vec<rect>>
		// //  add every rect to it as well as global vec
		// //  duplication checks here?
		// //  convert to a vec<rect> + vec<assoc>
		// //  pack it
		// // pack global rects
		// // save layer images
		// // generate and save metadata code

		// pack each layer (with deduplication)
		for ((group, layer), images) in self.images.iter_mut() {
			let mut rects = vec![];
			let mut rects_associated = vec![];
			for (image_name, (image_src, image_load)) in images.iter_mut() {
				for image_rect in image_load.into_rects(image_src).unwrap() {
					rects.push(image_rect);
				}
				// rects.push(rectpack2d::RectXYWH::new());
			}
		}
		// pack the output image
		// some stupid shit here
		todo!("save_build_info");
	}
}

impl Default for Packer {
	fn default() -> Self {
		Self::new()
	}
}

fn recurse_dir(dir: &Path) -> RkPkResult<Vec<PathBuf>> {
	let mut res = vec![];
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			res.append(&mut recurse_dir(&entry?.path())?);
		}
	} else {
		res.push(dir.to_path_buf());
	}
	Ok(res)
}
