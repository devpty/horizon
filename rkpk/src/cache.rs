use image::{RgbaImage, io::Reader};

use crate::{etil, Error};

/// an image lol
pub enum Image {
	File(String),
	Bin(String, &'static [u8]),
}

impl Image {
	// load an image into memory
	fn to_data(&self) -> Result<RgbaImage, Error> {
		Ok(match self {
			Self::File(path) => etil::cast_result(etil::cast_result(Reader::open(path), |e| Error::Io(e))?.decode(), |e| Error::Image(e))?.to_rgba8(),
			Self::Bin(_, data) => etil::cast_result(image::load_from_memory(data), |e| Error::Image(e))?.to_rgba8()
		})
	}
}

pub struct ImageCache {

}
