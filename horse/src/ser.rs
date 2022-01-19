use std::io;
use serde::{ser, Serialize};
use crate::{Error, Result};
use crate::types::{self, VType, FormatStyle};

pub struct Serializer<T: io::Write> {
	output: T,
	style: FormatStyle,
}

impl<T: io::Write> Serializer<T> {
	fn new(output: T, style: FormatStyle) -> Self {
		Self { output, style }
	}
	fn wr(&mut self, data: &[u8]) -> Result<()> {
		match self.output.write(data) {
			Ok(_) => Ok(()),
			Err(e) => Err(Error::Other(Box::new(e)))
		}
	}
	fn wr_type(&mut self, data: VType) -> Result<()> {
		self.wr(&[data as u8])
	}
	// type hell, because i thought it'd be a good idea to actually use different
	//            int sizes
	fn wr_int<V: types::IntLike>(&mut self, data: V) -> Result<()> {
		match data.try_down() {
			Some(down) => self.wr_int(down),
			None => {
				let mut bytes = data.to_bytes();
				if data >= <V>::SMALL_MIN && data < <V>::SMALL_MAX {
					// small encoding (smencoding)
					// remove sign info
					bytes[0] &= 0x1F;
					bytes[0] |= <V>::SIG_SMALL;
					self.wr(&bytes)
				} else {
					self.wr(&[<V>::SIG_LARGE])?;
					self.wr(&bytes)
				}
			}
		}
	}
	fn wr_float<V: types::FloatLike>(&mut self, data: V) -> Result<()> {
		self.wr(&data.to_bytes())
	}
}

/*
null: ()
false: ()
true: ()
int: (size-class, data)
size-class =
	0 = 0,5 (small u8)
	1 = 1,5 (small u16,  u8)
	2 = 2,5 (small u32,  u16)
	3 = 4,5 (small u64,  u32)
	4 = 8,5 (small u128, u64)
	5 = 16,0 (u128)
f32: (f32)
f64: (f64)
char: u32
bin: (usize, data)
opt: Some(val), None(())
list: (usize, val[])
dict: (usize, pair[])
pair: (val, val)
*/

pub fn to_write<V: Serialize, T: io::Write>(value: &V, writer: T, style: FormatStyle) -> Result<T> {
	let mut serializer = Serializer::new(writer, style);
	value.serialize(&mut serializer)?;
	Ok(serializer.output)
}
pub fn to_bytes<V: Serialize>(value: &V, style: FormatStyle) -> Result<Vec<u8>> {
	Ok(to_write(value, io::Cursor::new(vec![]), style)?.into_inner())
}


impl<'a, T: io::Write> ser::Serializer for &'a mut Serializer<T> {
	type Ok = ();
	type Error = Error;
	type SerializeSeq = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeMap = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;

	fn serialize_bool(self, v: bool) -> Result<()> {
		self.wr_type(if v {VType::True} else {VType::False})
	}
	fn serialize_i8(self, v: i8) -> Result<()> {
		self.wr_type(VType::I8)?;
		self.wr_int(v)
	}
	fn serialize_i16(self, v: i16) -> Result<()> {
		self.wr_type(VType::I16)?;
		self.wr_int(v)
	}
	fn serialize_i32(self, v: i32) -> Result<()> {
		self.wr_type(VType::I32)?;
		self.wr_int(v)
	}
	fn serialize_i64(self, v: i64) -> Result<()> {
		self.wr_type(VType::I64)?;
		self.wr_int(v)
	}
	fn serialize_i128(self, v: i128) -> Result<()> {
		self.wr_type(VType::I128)?;
		self.wr_int(v)
	}
	fn serialize_u8(self, v: u8) -> Result<()> {
		self.wr_type(VType::U8)?;
		self.wr_int(v)
	}
	fn serialize_u16(self, v: u16) -> Result<()> {
		self.wr_type(VType::U16)?;
		self.wr_int(v)
	}
	fn serialize_u32(self, v: u32) -> Result<()> {
		self.wr_type(VType::U32)?;
		self.wr_int(v)
	}
	fn serialize_u64(self, v: u64) -> Result<()> {
		self.wr_type(VType::U64)?;
		self.wr_int(v)
	}
	fn serialize_u128(self, v: u128) -> Result<()> {
		self.wr_type(VType::U128)?;
		self.wr_int(v)
	}
	fn serialize_f32(self, v: f32) -> Result<()> {
		self.wr_type(VType::F32)?;
		self.wr_float(v)
	}
	fn serialize_f64(self, v: f64) -> Result<()> {
		self.wr_type(VType::F64)?;
		self.wr_float(v)
	}
	fn serialize_char(self, v: char) -> Result<()> {
		self.wr_type(VType::Char)?;
		self.wr_int(v as u32)
	}
	fn serialize_str(self, v: &str) -> Result<()> {
		self.serialize_bytes(v.as_bytes())
	}
	fn serialize_bytes(self, v: &[u8]) -> Result<()> {
		self.wr_type(VType::Bin)?;
		self.wr_int(v.len())?;
		self.wr(v)
	}
	fn serialize_none(self) -> Result<()> {
		self.wr_type(VType::OptNone)
	}
	fn serialize_some<V>(self, value: &V) -> Result<()>
	where V: ?Sized + Serialize {
		self.wr_type(VType::OptSome)?;
		value.serialize(self)
	}
	fn serialize_unit(self) -> Result<()> {
		self.wr_type(VType::Null)
	}
	fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
		self.serialize_unit()
	}
	fn serialize_unit_variant(
		self,
		_name: &'static str,
		index: u32,
		variant: &'static str,
	) -> Result<()> {
		match self.style {
			FormatStyle::Compact => self.serialize_u32(index),
			FormatStyle::Expressive => self.serialize_str(variant),
		}
	}
	fn serialize_newtype_struct<V>(
		self,
		_name: &'static str,
		value: &V,
	) -> Result<()>
	where V: ?Sized + Serialize {
		value.serialize(self)
	}
	fn serialize_newtype_variant<V>(
		self,
		_name: &'static str,
		index: u32,
		variant: &'static str,
		value: &V,
	) -> Result<()>
	where V: ?Sized + Serialize {
		self.wr_type(VType::Pair)?;
		match self.style {
			FormatStyle::Compact => {
				self.wr_type(VType::U32)?;
				self.wr_int(index)?;
			},
			FormatStyle::Expressive => self.serialize_str(variant)?,
		}
		value.serialize(self)
	}
	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		self.wr_type(VType::List)?;
		self.wr_int(len.unwrap())?;
		Ok(self)
	}
	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		self.serialize_seq(Some(len))
	}
	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		self.serialize_seq(Some(len))
	}
	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		self.wr_type(VType::Pair)?;
		match self.style {
			FormatStyle::Compact => {
				self.wr_type(VType::U32)?;
				self.wr_int(index)?;
			},
			FormatStyle::Expressive => self.serialize_str(variant)?,
		}
		self.wr_type(VType::List)?;
		self.wr_int(len)?;
		Ok(self)
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		self.wr_type(VType::Dict)?;
		self.wr_int(len.unwrap())?;
		Ok(self)
	}
	fn serialize_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct> {
		match self.style {
			FormatStyle::Compact => self.serialize_seq(Some(len)),
			FormatStyle::Expressive => self.serialize_map(Some(len))
		}
	}
	fn serialize_struct_variant(
		self,
		_name: &'static str,
		index: u32,
		variant: &'static str,
		len: usize,
	) -> Result<Self::SerializeStructVariant> {
		self.wr_type(VType::Pair)?;
		match self.style {
			FormatStyle::Compact => {
				self.wr_type(VType::U32)?;
				self.wr_int(index)?;
			},
			FormatStyle::Expressive => self.serialize_str(variant)?,
		}
		match self.style {
			FormatStyle::Compact => self.serialize_seq(Some(len)),
			FormatStyle::Expressive => self.serialize_map(Some(len))
		}
	}
}

macro_rules! impl_special_serialize {
	($impl:path, $($inner:tt)*) => {
		impl<'a, T: io::Write> $impl for &'a mut Serializer<T> {
			type Ok = ();
			type Error = Error;
			$(impl_special_serialize!{$inner})*
			fn end(self) -> Result<()> {Ok(())}
		}
	};
	(element) => {
		fn serialize_element<V>(&mut self, value: &V) -> Result<()>
		where V: ?Sized + Serialize {
			value.serialize(&mut **self)
		}
	};
	(field) => {
		fn serialize_field<V>(&mut self, value: &V) -> Result<()>
		where V: ?Sized + Serialize {
			value.serialize(&mut **self)
		}
	};
	(key) => {
		fn serialize_key<V>(&mut self, key: &V) -> Result<()>
		where V: ?Sized + Serialize {
			key.serialize(&mut **self)
		}
	};
	(value) => {
		fn serialize_value<V>(&mut self, value: &V) -> Result<()>
		where V: ?Sized + Serialize {
			value.serialize(&mut **self)
		}
	};
	(double_field) => {
		fn serialize_field<V>(&mut self, key: &'static str, value: &V) -> Result<()>
		where V: ?Sized + Serialize {
			if let FormatStyle::Expressive = self.style {
				key.serialize(&mut **self)?;
			}
			value.serialize(&mut **self)
		}
	};
}

impl_special_serialize!{ser::SerializeSeq,           element}
impl_special_serialize!{ser::SerializeTuple,         element}
impl_special_serialize!{ser::SerializeTupleStruct,   field}
impl_special_serialize!{ser::SerializeTupleVariant,  field}
impl_special_serialize!{ser::SerializeMap,           key value}
impl_special_serialize!{ser::SerializeStruct,        double_field}
impl_special_serialize!{ser::SerializeStructVariant, double_field}
