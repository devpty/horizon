use std::{collections};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VType {
	Null = 0, False, True,
	I8, I16, I32, I64, I128,
	U8, U16, U32, U64, U128,
	F32, F64, Char, Bin,
	OptSome, OptNone,
	List, Dict, Pair,
}

macro_rules! try_from_matches {
	($v:tt $($name:tt),*$(,)?) => {
		match $v {
			$(x if x == Self::$name as u8 => Ok(Self::$name)),*,
			_ => Err(())
		}
	};
}

impl std::convert::TryFrom<u8> for VType {
	type Error = ();
	fn try_from(v: u8) -> std::result::Result<Self, Self::Error> {
		try_from_matches!{v
			Null, False, True,
			I8, I16, I32, I64, I128,
			U8, U16, U32, U64, U128,
			F32, F64, Char, Bin,
			OptSome, OptNone,
			List, Dict, Pair,
		}
	}
}

#[derive(Debug, Clone)]
pub enum VValue {
	Null,
	Bool(bool),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	I128(i128),
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	U128(u128),
	F32(f32),
	F64(f64),
	Char(char),
	Bin(Vec<u8>),
	Opt(Option<Box<VValue>>),
	List(Vec<VValue>),
	Dict(collections::HashMap<Vec<u8>, VValue>),
	Pair(Box<(VValue, VValue)>),
}

// impl PartialEq for VValue {
// 	fn eq(&self, other: &Self) -> bool {
// 		self.as_type() == other.as_type() && match self {
// 			Self::Null => true,
// 			Self::Dict(a) => match other {Self::Dict(b) => {

// 			}}
// 			a => match other {b => a == b, _ => false},
// 		}
// 	}
// }

// impl Eq for VValue {}

impl VValue {
	fn as_type(&self) -> VType {
		match self {
			Self::Null => VType::Null,
			Self::Bool(false) => VType::False,
			Self::Bool(true)  => VType::True,
			Self::I8(_)   => VType::I8,
			Self::I16(_)  => VType::I16,
			Self::I32(_)  => VType::I32,
			Self::I64(_)  => VType::I64,
			Self::I128(_) => VType::I128,
			Self::U8(_)   => VType::U8,
			Self::U16(_)  => VType::U16,
			Self::U32(_)  => VType::U32,
			Self::U64(_)  => VType::U64,
			Self::U128(_) => VType::U128,
			Self::F32(_)  => VType::F32,
			Self::F64(_)  => VType::F64,
			Self::Char(_) => VType::Char,
			Self::Bin(_)  => VType::Bin,
			Self::Opt(Some(_)) => VType::OptSome,
			Self::Opt(None)    => VType::OptNone,
			Self::List(_) => VType::List,
			Self::Dict(_) => VType::Dict,
			Self::Pair(_) => VType::Pair,
		}
	}
}

pub trait CastDown {
	type Down: IntLike;
	type DownSizing;
	const SMALL_MIN: Self;
	const SMALL_MAX: Self;
	const SIG_SMALL: u8;
	const SIG_LARGE: u8;
	fn try_down(self) -> Option<Self::Down>;
	fn from_down(v: Self::Down) -> Self;
}

pub trait CanByte {
	fn to_bytes(&self) -> Vec<u8>;
	fn from_bytes(v: Vec<u8>) -> Self;
}

pub trait IntLike: CanByte + CastDown + Default + TryFrom<u8> + num_traits::int::PrimInt {}
pub trait FloatLike: CanByte + num_traits::float::Float {}

macro_rules! impl_cast_down {
	(small $type:ty, u $max:expr) => {
		const SMALL_MIN: $type = 0;
		const SMALL_MAX: $type = $max;
	};
	(small $type:ty, i $max:expr) => {
		const SMALL_MIN: $type = -$max;
		const SMALL_MAX: $type = $max;
	};
	(extra $type:ty, ($sig_small:tt $sig_large:tt $min:tt $max:expr)) => {
		impl_cast_down!{small $type, $min $max}
		const SIG_SMALL: u8 = $sig_small << 5;
		const SIG_LARGE: u8 = $sig_large << 5;
	};
	(impl $type:ty, $down:ty, $extra:tt) => {
		impl CastDown for $type {
			type Down = $down;
			type DownSizing = Self::Down;
			impl_cast_down!{extra Self, $extra}
			fn try_down(self) -> Option<Self::Down> {
				match <$down>::try_from(self) {
					Ok(v) => Some(v),
					Err(_) => None,
				}
			}
			fn from_down(v: $down) -> Self {v as Self}
		}
	};
	(default $target:ty, $extra:tt) => {
		impl CastDown for $target {
			type Down = Self;
			type DownSizing = ();
			impl_cast_down!{extra Self, $extra}
			fn try_down(self) -> Option<Self::Down> { None }
			fn from_down(v: Self) -> Self {v}
		}
	};
	(arch $target:tt $width:tt $as:tt) => {
		#[cfg(target_pointer_width = $width)]
		impl_cast_down!{$target $as}
	};
	(ar $target:tt $v8:tt $v16:tt $v32:tt $v64:tt $v128:tt) => {
		impl_cast_down!{arch $target "8"   $v8}
		impl_cast_down!{arch $target "16"  $v16}
		impl_cast_down!{arch $target "32"  $v32}
		impl_cast_down!{arch $target "64"  $v64}
		impl_cast_down!{arch $target "128" $v128}
	};
	($target:tt) => {impl_cast_down!{$target $target}};
	($target:tt i8)   => {impl_cast_down!{default $target,   (0 1 i 0x40)}};
	($target:tt i16)  => {impl_cast_down!{impl $target, i8,  (1 2 i 0x1000)}};
	($target:tt i32)  => {impl_cast_down!{impl $target, i16, (2 3 i 0x10000000)}};
	($target:tt i64)  => {impl_cast_down!{impl $target, i32, (3 4 i 0x1000000000000000)}};
	($target:tt i128) => {impl_cast_down!{impl $target, i64, (4 5 i 0x10000000000000000000000000000000)}};
	($target:tt isize) => {impl_cast_down!{ar $target i8 i16 i32 i64 i128}};
	($target:tt u8)   => {impl_cast_down!{default $target,   (0 1 u 0x80)}};
	($target:tt u16)  => {impl_cast_down!{impl $target, u8,  (1 2 u 0x2000)}};
	($target:tt u32)  => {impl_cast_down!{impl $target, u16, (2 3 u 0x20000000)}};
	($target:tt u64)  => {impl_cast_down!{impl $target, u32, (3 4 u 0x2000000000000000)}};
	($target:tt u128) => {impl_cast_down!{impl $target, u64, (4 5 u 0x20000000000000000000000000000000)}};
	($target:tt usize) => {impl_cast_down!{ar $target u8 u16 u32 u64 u128}};
}

macro_rules! impl_util {
	($cmd:tt $val:tt $($rest:tt)+) => {
		impl_util!{$cmd $val}
		impl_util!{$($rest)+}
	};
	(cast_down $target:tt) => {
		impl_cast_down!{$target}
	};
	(can_byte $target:tt) => {
		impl CanByte for $target {
			fn to_bytes(&self) -> Vec<u8> {
				Vec::from(self.to_be_bytes())
			}
			fn from_bytes(v: Vec<u8>) -> Self {
				Self::from_be_bytes(v.try_into().unwrap())
			}
		}
	};
	(int_like $target:tt) => {
		impl_util!{
			can_byte $target
			cast_down $target
		}
		impl IntLike for $target {}
	};
	(float_like $target:tt) => {
		impl_util!{
			can_byte $target
		}
		impl FloatLike for $target {}
	}
}

impl_util!{
	int_like i8
	int_like i16
	int_like i32
	int_like i64
	int_like i128
	int_like isize
	int_like u8
	int_like u16
	int_like u32
	int_like u64
	int_like u128
	int_like usize
	float_like f32
	float_like f64
}
