use std::fmt;



#[derive(Debug)]
pub enum Error {
	Io(std::io::Error),
	Image(image::ImageError),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("{:?}", self))
	}
}
impl std::error::Error for Error {}

/// error utilities
pub mod etil {
	pub fn cast_result<A, B, C, D: FnOnce(B) -> C>(v: Result<A, B>, f: D) -> Result<A, C> {
		match v {
			Ok(r) => Ok(r),
			Err(r) => Err(f(r)),
		}
	}
	pub fn cast_option<A, B>(v: Option<A>, f: B) -> Result<A, B> {
		match v {
			Some(r) => Ok(r),
			None => Err(f),
		}
	}
}
