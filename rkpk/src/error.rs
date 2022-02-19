use std::fmt;

/// error
#[derive(Debug)]
pub enum Error {
	Io(std::io::Error),
	Image(image::ImageError),
	PackingFailed,
	Shit,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{:?}", self))
	}
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
