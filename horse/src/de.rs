use std::{io, mem, collections};
// use serde::{de, Deserialize};
use crate::{Error, Result};
use crate::types::{self, VType, VValue};

pub struct Deserializer<T: io::Read> {
	input: T
}

impl<T: io::Read> Deserializer<T> {
	pub fn from_reader(input: T) -> Self {
		Self { input }
	}
	fn unwrap(self) -> T {
		self.input
	}
	fn next(&mut self) -> Result<u8> {
		let buf = [0u8];
		Error::cast(self.input.read_exact(&mut buf))?;
		Ok(buf[0])
	}
	fn next_n(&mut self, n: usize) -> Result<Vec<u8>> {
		let vec = vec![0; n];
		for i in 0..n {
			vec[i] = self.next()?;
		}
		Ok(vec)
	}
	fn read_float<V: types::FloatLike>(&mut self) -> Result<V> {
		Ok(<V>::from_bytes(self.next_n(mem::size_of::<V>())?))
	}
	fn read_int<V: types::IntLike>(&mut self) -> Result<V> {
		let byte_0 = self.next()?;
		let sig = byte_0 & 0xE0;
		let size = mem::size_of::<V>();
		let mut bytes = vec![0; size];
		Ok(match sig {
			<V>::SIG_LARGE => {
				for i in 0..size {
					bytes[i] = self.next()?;
				}
				<V>::from_bytes(bytes)
			}
			<V>::SIG_SMALL => {
				let sign = byte_0 & 0x10 != 0;
				let trans = size-mem::size_of::<V::DownSizing>();
				bytes[trans - 1] = if sign {
					for i in 0..trans - 1 {
						bytes[i] = 0xFF;
					}
					byte_0 | 0xE0
				} else {
					byte_0
				};
				for i in trans..size {
					bytes[i] = self.next()?;
				}
				<V>::from_bytes(bytes)
			}
			_ => <V>::from_down(self.read_int::<V::Down>()?)
		})
	}
	fn read_bin(&mut self, len: usize) -> Result<Vec<u8>> {
		self.next_n(len)
	}
	fn read_list(&mut self, n: usize) -> Result<Vec<VValue>> {
		let out = vec![VValue::Null; n];
		for i in 0..n {
			out[n] = self.read_value()?;
		}
		Ok(out)
	}
	fn read_dict(&mut self, len: usize) -> Result<collections::HashMap<Vec<u8>, VValue>> {
		let out = collections::HashMap::new();
		for i in 0..len {
			let key = self.read_bin(self.read_int()?)?;
			let value = self.read_value()?;
			out[&key] = value;
		}
		Ok(out)
	}
	fn read_value(&mut self) -> Result<VValue> {
		let ty_int = self.next()?;
		Ok(match match ty_int.try_into() {
			Ok(v) => v, Err(_) => return Err(Error::InvalidType(ty_int))
		} {
			VType::Null    => VValue::Null,
			VType::False   => VValue::Bool(false),
			VType::True    => VValue::Bool(true),
			VType::I8      => VValue::I8(  self.read_int()?),
			VType::I16     => VValue::I16( self.read_int()?),
			VType::I32     => VValue::I32( self.read_int()?),
			VType::I64     => VValue::I64( self.read_int()?),
			VType::I128    => VValue::I128(self.read_int()?),
			VType::U8      => VValue::U8(  self.read_int()?),
			VType::U16     => VValue::U16( self.read_int()?),
			VType::U32     => VValue::U32( self.read_int()?),
			VType::U64     => VValue::U64( self.read_int()?),
			VType::U128    => VValue::U128(self.read_int()?),
			VType::F32     => VValue::F32(self.read_float()?),
			VType::F64     => VValue::F64(self.read_float()?),
			VType::Char    => {
				let int = self.read_int()?;
				VValue::Char(Error::opt(char::from_u32(int), Error::InvalidChar(int))?)
			},
			VType::Bin     => VValue::Bin(self.read_bin(self.read_int()?)?),
			VType::OptSome => VValue::Opt(Some(Box::new(self.read_value()?))),
			VType::OptNone => VValue::Opt(None),
			VType::List    => VValue::List(self.read_list(self.read_int()?)?),
			VType::Dict    => VValue::Dict(self.read_dict(self.read_int()?)?),
			VType::Pair    => VValue::Pair(Box::new((self.read_value()?, self.read_value()?))),
		})
	}
}

// pub fn from_reader<'a, V: Deserialize<'a>, T: io::Read>(s: T) -> Result<(V, T)> {
// 	let mut deserializer = Deserializer::from_reader(s);
// 	let t = T::deserialize(&mut deserializer)?;
// 	Ok((t, deserializer.unwrap()))
// }

// impl<'de, 'a, T: io::Read> de::Deserializer<'de> for &'a mut Deserializer<T> {
// 	type Error = Error;
// 	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
// 	where V: de::Visitor<'de> {
// 		let ty_int = self.next()?;
// 		match match ty_int.try_into() {
// 			Ok(v) => v, Err(_) => return Err(Error::InvalidType(ty_int))
// 		} {
// 			VType::Null    => visitor.visit_unit(),
// 			VType::False   => visitor.visit_bool(false),
// 			VType::True    => visitor.visit_bool(true),
// 			VType::I8      => visitor.visit_i8(  self.read_int()?),
// 			VType::I16     => visitor.visit_i16( self.read_int()?),
// 			VType::I32     => visitor.visit_i32( self.read_int()?),
// 			VType::I64     => visitor.visit_i64( self.read_int()?),
// 			VType::I128    => visitor.visit_i128(self.read_int()?),
// 			VType::U8      => visitor.visit_i8(  self.read_int()?),
// 			VType::U16     => visitor.visit_i16( self.read_int()?),
// 			VType::U32     => visitor.visit_i32( self.read_int()?),
// 			VType::U64     => visitor.visit_i64( self.read_int()?),
// 			VType::U128    => visitor.visit_i128(self.read_int()?),
// 			VType::F32     => visitor.visit_f32(self.read_float::<f32>()?),
// 			VType::F64     => visitor.visit_f64(self.read_float::<f64>()?),
// 			VType::Char    => visitor.visit_char(Error::opt(char::from_u32(self.read_int::<u32>()?), Error::InvalidChar)?),
// 			VType::Bin     => ,
// 			VType::OptSome => ,
// 			VType::OptNone => ,
// 			VType::List    => ,
// 			VType::Dict    => ,
// 			VType::Pair    => ,
// 		}
// 	}
// }
