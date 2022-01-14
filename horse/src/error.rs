use std::{fmt, error};
use serde::{ser, de};

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
	Message(String),
	UnexpectedEOF,
	InvalidType(u8),
	InvalidChar(u32),
	Other(Box<dyn error::Error>),
}

impl Error {
	pub fn cast<R, E>(v: std::result::Result<R, E>) -> Result<R>
	where E: error::Error + 'static {
		match v {
			Ok(i) => Ok(i),
			Err(e) => Err(Error::Other(Box::new(e)))
		}
	}
	pub fn opt<V>(v: Option<V>, e: Error) -> Result<V> {
		match v {
			Some(v) => Ok(v),
			None => Err(e)
		}
	}
}

macro_rules! impl_error {
	(for $for:path) => {
		impl $for for Error {
			fn custom<T: fmt::Display>(msg: T) -> Self {
				Error::Message(msg.to_string())
			}
		}
	};
}

impl_error!{for ser::Error}
impl_error!{for de::Error}

macro_rules! impl_display {
	(for $for:path) => {
		impl $for for Error {
			fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
				fmt.write_str(&impl_display!{self,
					Message(msg) => ("Message: {}", msg),
					Other(err) => ("Error: {}", err),
					UnexpectedEOF => ("Unexpected EOF"),
					InvalidType(id) => ("Invalid Type {:x}", id),
					InvalidChar(id) => ("Invalid Char {:x}", id),
				})
			}
		}
	};
	($self:ident, $($a:tt $(($b:tt))? => ($($c:tt)*)),* $(,)?) => {
		match $self {
			$(Error::$a $(($b))? => format!($($c)*)),*
		}
	}
}

impl_display!{for fmt::Display}
impl_display!{for fmt::Debug}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		if let Error::Other(other) = self {
			Some(other.as_ref())
		} else {
			None
		}
	}
}
