use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
	#[error("name length {0} is longer than maximum of 65535 (2¹⁶-1)")]
	NameTooLong(usize),
	#[error("data length {0} is longer than maximum of 281474976710655 (2²⁴-1)")]
	DataTooLong(usize),
	#[error("io error")]
	IoError(#[from] std::io::Error),
}

pub type AssetResult<T> = Result<T, AssetError>;

#[repr(C)]
pub struct Wad64Header {
	pub magic: u64,
	pub len: u64,
}

#[repr(C)]
pub struct Wad64Entry {
	pub name_len: u16,
	pub data_len_high: u16,
	pub data_len_low: u32,
	pub data_ptr: u64,
}
