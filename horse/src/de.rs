use std::{io, mem};
use serde::de::{self, IntoDeserializer};
use crate::{Error, Result};
use crate::types::{self, VType, VValue, Version, Versioned};

pub struct Deserializer<'de, T: io::Read> {
	input: &'de mut T,
	output: Vec<VValue>,
}

impl<'de, T: io::Read> Deserializer<'de, T> {
	pub fn from_reader(input: &'de mut T) -> Self {
		Self { input, output: Vec::new() }
	}
	fn next(&mut self) -> Result<u8> {
		let mut buf = [0u8];
		Error::cast(self.input.read_exact(&mut buf))?;
		Ok(buf[0])
	}
	fn next_n(&mut self, n: usize) -> Result<Vec<u8>> {
		let mut vec = vec![0; n];
		for i in 0..n {
			vec[i] = self.next()?;
		}
		Ok(vec)
	}
	fn read_float<V: types::FloatLike>(&mut self) -> Result<V> {
		Ok(<V>::from_bytes(self.next_n(mem::size_of::<V>())?))
	}
	fn read_int<V: types::IntLike>(&mut self) -> Result<V> {
		self.read_int_internal(None)
	}
	// welcome to type hell, deserializer edition
	fn read_int_internal<V: types::IntLike>(&mut self, start: Option<u8>) -> Result<V> {
		let byte_0 = match start {
			Some(v) => v,
			None => self.next()?
		};
		let sig = byte_0 & 0xE0;
		let size = mem::size_of::<V>();
		let mut bytes = vec![0; size];
		if sig == <V>::SIG_LARGE {
			for i in 0..size {
				bytes[i] = self.next()?;
			}
			Ok(<V>::from_bytes(bytes))
		} else if sig == <V>::SIG_SMALL {
			let sign = <V>::IS_SIGNED && byte_0 & 0x10 != 0;
			let trans = size-mem::size_of::<V::DownSizing>();
			bytes[trans - 1] = if sign {
				for i in 0..trans - 1 {
					bytes[i] = 0xFF;
				}
				byte_0 | 0xE0
			} else {
				byte_0 & 0x1F
			};
			for i in trans..size {
				bytes[i] = self.next()?;
			}
			Ok(<V>::from_bytes(bytes))
		} else if <V>::CAN_DOWN {
			Ok(<V>::from_down(self.read_int_internal::<V::Down>(Some(byte_0))?))
		} else {
			Err(Error::IntCastFail)
		}
	}
	fn read_bin(&mut self) -> Result<Vec<u8>> {
		// have to declare a variable, otherwise it counts as multiple mut borrows,
		// why??
		let n = self.read_int()?;
		self.next_n(n)
	}
	fn read_list(&mut self, n: usize) -> Result<()> {
		for _ in 0..n {
			self.read_value()?;
		}
		Ok(())
	}
	fn read_dict(&mut self, n: usize) -> Result<()> {
		for _ in 0..n {
			self.read_value()?;
			self.read_value()?;
		}
		Ok(())
	}
	fn read_value(&mut self) -> Result<()> {
		let ty_int = self.next()?;
		let ty = match ty_int.try_into() {
			Ok(v) => v,
			Err(_) => return Err(Error::InvalidType(ty_int))
		};
		let res = match ty {
			VType::Null    => Some(VValue::Null),
			VType::False   => Some(VValue::Bool(false)),
			VType::True    => Some(VValue::Bool(true)),
			VType::I8      => Some(VValue::I8(  self.read_int()?)),
			VType::I16     => Some(VValue::I16( self.read_int()?)),
			VType::I32     => Some(VValue::I32( self.read_int()?)),
			VType::I64     => Some(VValue::I64( self.read_int()?)),
			VType::I128    => Some(VValue::I128(self.read_int()?)),
			VType::U8      => Some(VValue::U8(  self.read_int()?)),
			VType::U16     => Some(VValue::U16( self.read_int()?)),
			VType::U32     => Some(VValue::U32( self.read_int()?)),
			VType::U64     => Some(VValue::U64( self.read_int()?)),
			VType::U128    => Some(VValue::U128(self.read_int()?)),
			VType::F32     => Some(VValue::F32(self.read_float()?)),
			VType::F64     => Some(VValue::F64(self.read_float()?)),
			VType::Char    => {
				let int = self.read_int()?;
				Some(VValue::Char(Error::opt(char::from_u32(int), Error::InvalidChar(int))?))
			},
			VType::Bin     => Some(VValue::Bin(self.read_bin()?)),
			VType::OptSome => {
				self.output.push(VValue::Opt(true));
				self.read_value()?;
				None
			},
			VType::OptNone => Some(VValue::Opt(false)),
			VType::List    => {
				let n = self.read_int()?;
				self.output.push(VValue::List(n));
				self.read_list(n)?;
				None
			},
			VType::Dict    => {
				let n = self.read_int()?;
				self.output.push(VValue::Dict(n));
				self.read_dict(n)?;
				None
			},
			VType::Pair    => {
				self.output.push(VValue::Pair);
				self.read_value()?;
				self.read_value()?;
				None
			},
		};
		match res {
			Some(v) => self.output.push(v),
			None => {},
		}
		Ok(())
	}
}

// the unit type is needed since this is technically a deserializer
#[derive(Debug, Clone)]
pub struct HorseTree<'de>(Vec<VValue>, usize, &'de ());

impl<'de> HorseTree<'de> {
	fn peek(&self) -> Result<&VValue> {
		Ok(Error::opt(self.0.get(self.1), Error::UnexpectedEOF)?)
	}
	fn next(&mut self) -> Result<&VValue> {
		// yes that is directly from self.peek(), but using that causes "fun"
		//                                        borrowing issues.
		let out = Error::opt(self.0.get(self.1), Error::UnexpectedEOF)?;
		self.1 += 1;
		Ok(out)
	}
	fn peek_slow(&self) -> Result<VValue> {
		Ok(Error::opt(self.0.get(self.1), Error::UnexpectedEOF)?.clone())
	}
	fn next_slow(&mut self) -> Result<VValue> {
		let out = self.peek_slow()?;
		self.1 += 1;
		Ok(out)
	}
	pub fn deserialize<T>(&mut self) -> Result<T>
	where T: de::Deserialize<'de> {
		T::deserialize(self)
	}
	/// versioned deserializing
	pub fn deserialize_ver<T>(&mut self, ver: Version) -> Result<T>
	where T: Versioned<'de> {
		// convert from the oldest known working version
		if <T>::VERSION < ver {
			Err(Error::VersionInTheFuture(ver, <T>::VERSION))
		} else if <T::Old>::VERSION >= ver && <T::Old>::VERSION != <T>::VERSION {
			let res = self.deserialize_ver::<T::Old>(ver)?;
			Ok(res.into())
		} else {
			self.deserialize()
		}
	}
}

macro_rules! make_deserialize {
	(cst ($name:tt $self:tt $visitor:tt $impl:tt)) => {
		fn $name<V>($self, $visitor: V) -> Result<V::Value>
		where V: de::Visitor<'de> $impl
	};
	(visit ($name_a:tt $name_b:tt $type:tt $err:tt)) => {
		make_deserialize!{cst ($name_a self visitor {
			match self.next()? {
				VValue::$type(v) => visitor.$name_b(*v),
				value => Err(Error::UnexpectedType(value.as_type(), $err))
			}
		})}
	};
	(visit_tl ($name_a:tt $name_b:tt $type:tt $err:tt)) => {
		make_deserialize!{cst ($name_a self visitor {
			match self.next()? {
				VValue::$type => visitor.$name_b(),
				value => Err(Error::UnexpectedType(value.as_type(), $err))
			}
		})}
	};
	(visit_sl ($name_a:tt $name_b:tt)) => {
		make_deserialize!{cst ($name_a self visitor {
			visitor.$name_b(self)
		})}
	};
	(self ($name_a:tt $name_b:tt)) => {
		make_deserialize!{cst ($name_a self visitor {
			self.$name_b(visitor)
		})}
	};
	(self_arg ($name_a:tt $name_b:tt ($($arg:tt),*))) => {
		fn $name_a<V>(self, $($arg:tt),*, visitor: V) -> Result<V::Value>
		where V: de::Visitor<'de> {
			self.$name_b(visitor);
		}
	};
	($op:tt $arg:tt $($eo:tt $ea:tt)+ ) => {
		make_deserialize!{$op $arg}
		make_deserialize!{$($eo $ea)+}
	};
}

impl<'de, 'ez> de::Deserializer<'de> for &'ez mut HorseTree<'de> {
	type Error = Error;
	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		match self.peek()?.as_type() {
			VType::Null    => self.deserialize_unit(visitor),
			VType::False   => self.deserialize_bool(visitor),
			VType::True    => self.deserialize_bool(visitor),
			VType::I8      => self.deserialize_i8(visitor),
			VType::I16     => self.deserialize_i16(visitor),
			VType::I32     => self.deserialize_i32(visitor),
			VType::I64     => self.deserialize_i64(visitor),
			VType::I128    => self.deserialize_i128(visitor),
			VType::U8      => self.deserialize_u8(visitor),
			VType::U16     => self.deserialize_u16(visitor),
			VType::U32     => self.deserialize_u32(visitor),
			VType::U64     => self.deserialize_u64(visitor),
			VType::U128    => self.deserialize_u128(visitor),
			VType::F32     => self.deserialize_f32(visitor),
			VType::F64     => self.deserialize_f64(visitor),
			VType::Char    => self.deserialize_char(visitor),
			VType::Bin     => self.deserialize_bytes(visitor),
			VType::OptSome => self.deserialize_option(visitor),
			VType::OptNone => self.deserialize_option(visitor),
			VType::List    => self.deserialize_seq(visitor),
			VType::Dict    => self.deserialize_map(visitor),
			VType::Pair    => Err(Error::NoDeserializeRawPair),
		}
	}
	make_deserialize!{
		visit (deserialize_bool visit_bool Bool "bool")
		visit (deserialize_i8 visit_i8 I8 "i8")
		visit (deserialize_i16 visit_i16 I16 "i16")
		visit (deserialize_i32 visit_i32 I32 "i32")
		visit (deserialize_i64 visit_i64 I64 "i64")
		visit (deserialize_i128 visit_i128 I128 "i128")
		visit (deserialize_u8 visit_u8 U8 "u8")
		visit (deserialize_u16 visit_u16 U16 "u16")
		visit (deserialize_u32 visit_u32 U32 "u32")
		visit (deserialize_u64 visit_u64 U64 "u64")
		visit (deserialize_u128 visit_u128 U128 "u128")
		visit (deserialize_f32 visit_f32 F32 "f32")
		visit (deserialize_f64 visit_f64 F64 "f64")
		visit (deserialize_char visit_char Char "char")
		self  (deserialize_string deserialize_str)
		visit_tl (deserialize_unit visit_unit Null "null")
	}
	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		match self.next()? {
			VValue::Bin(v) => {
				// i don't know how to use visit_borrowed_str so i'm gonna not
				visitor.visit_str(&String::from_utf8_lossy(v))
			},
			value => Err(Error::UnexpectedType(value.as_type(), "bin"))
		}
	}
	fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		match self.next()? {
			VValue::Bin(v) => visitor.visit_bytes(v),
			value => Err(Error::UnexpectedType(value.as_type(), "bin"))
		}
	}
	fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		match self.next_slow()? {
			VValue::Bin(v) => visitor.visit_byte_buf(v),
			value => Err(Error::UnexpectedType(value.as_type(), "bin"))
		}
	}
	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		match self.next()? {
			VValue::Opt(false) => visitor.visit_none(),
			VValue::Opt(true) => visitor.visit_some(self),
			value => Err(Error::UnexpectedType(value.as_type(), "opt"))
		}
	}
	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		match self.next_slow()? {
			VValue::List(len) => {
				visitor.visit_seq(HorseTreeSeq {de: self, left: len})
			},
			value => Err(Error::UnexpectedType(value.as_type(), "list"))
		}
	}
	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		match self.next_slow()? {
			VValue::Dict(len) => {
				visitor.visit_map(HorseTreeSeq {de: self, left: len})
			},
			value => Err(Error::UnexpectedType(value.as_type(), "dict"))
		}
	}
	fn deserialize_unit_struct<V>(
		self, _name: &'static str, visitor: V
	) -> Result<V::Value> where V: de::Visitor<'de> {
		self.deserialize_unit(visitor)
	}
	fn deserialize_newtype_struct<V>(
		self, _name: &'static str, visitor: V
	) -> Result<V::Value> where V: de::Visitor<'de> {
		visitor.visit_newtype_struct(self)
	}
	fn deserialize_tuple<V>(
		self, _len: usize, visitor: V
	) -> Result<V::Value> where V: de::Visitor<'de> {
		self.deserialize_seq(visitor)
	}
	fn deserialize_tuple_struct<V>(
		self, _name: &'static str, _len: usize, visitor: V
	) -> Result<V::Value> where V: de::Visitor<'de> {
		self.deserialize_seq(visitor)
	}
	fn deserialize_struct<V>(
		self, _name: &'static str, _fields: &'static [&'static str], visitor: V
	) -> Result<V::Value> where V: de::Visitor<'de> {
		match self.next_slow()? {
			VValue::Dict(len) => {
				visitor.visit_map(HorseTreeSeq {de: self, left: len})
			},
			VValue::List(len) => {
				visitor.visit_seq(HorseTreeSeq {de: self, left: len})
			},
			value => Err(Error::UnexpectedType(value.as_type(), "dict or list"))
		}
	}
	fn deserialize_enum<V>(
		self, _name: &'static str, _variants: &'static [&'static str], visitor: V
	) -> Result<V::Value> where V: de::Visitor<'de> {
		match self.next()? {
			// unit variant
			VValue::U32(v) => visitor.visit_enum(v.into_deserializer()),
			VValue::Bin(v) => visitor.visit_enum(String::from_utf8_lossy(v).into_deserializer()),
			// {newtype, tuple, struct} variant
			VValue::Pair => {
				visitor.visit_enum(HorseTreeEnum {de: self})
			},
			value => Err(Error::UnexpectedType(value.as_type(), "u32 or pair"))
		}
	}
	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		// accept u32 or bin
		match self.next()? {
			VValue::Bin(v) => visitor.visit_str(&String::from_utf8_lossy(v)),
			VValue::U32(v) => visitor.visit_u32(*v),
			value => Err(Error::UnexpectedType(value.as_type(), "bin or u32"))
		}
	}
	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'de> {
		// i probably won't implement a dedicated version of this since that'd just
		// be unneeded code duplication
		self.deserialize_any(visitor)
	}
}

pub fn from_read<'de, 'ez, T: io::Read>(s: &'ez mut T) -> Result<HorseTree<'de>> {
	let mut ser = Deserializer::from_reader(s);
	ser.read_value()?;
	Ok(HorseTree(ser.output, 0, &()))
}

pub fn from_bytes<'de>(s: &[u8]) -> Result<HorseTree<'de>> {
	Ok(from_read(&mut io::Cursor::new(s))?)
}

struct HorseTreeSeq<'ts, 'nu: 'ts> {
	de: &'ts mut HorseTree<'nu>,
	left: usize,
}

impl<'nu, 'ts> de::SeqAccess<'nu> for HorseTreeSeq<'ts, 'nu> {
	type Error = Error;
	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where T: de::DeserializeSeed<'nu> {
		if self.left == 0 {
			Ok(None)
		} else {
			self.left -= 1;
			seed.deserialize(&mut *self.de).map(Some)
		}
	}
}

impl<'nu, 'ts> de::MapAccess<'nu> for HorseTreeSeq<'ts, 'nu> {
	type Error = Error;
	fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where T: de::DeserializeSeed<'nu> {
		if self.left == 0 {
			Ok(None)
		} else {
			seed.deserialize(&mut *self.de).map(Some)
		}
	}
	fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
	where T: de::DeserializeSeed<'nu> {
		self.left -= 1;
		seed.deserialize(&mut *self.de)
	}
}

struct HorseTreeEnum<'ts, 'nu: 'ts> {
	de: &'ts mut HorseTree<'nu>
}

impl<'nu, 'ts> de::EnumAccess<'nu> for HorseTreeEnum<'ts, 'nu> {
	type Error = Error;
	type Variant = Self;
	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
	where V: de::DeserializeSeed<'nu> {
		Ok((seed.deserialize(&mut *self.de)?, self))
	}
}

impl<'nu, 'ts> de::VariantAccess<'nu> for HorseTreeEnum<'ts, 'nu> {
	type Error = Error;
	// the key's already been parsed at this point
	fn unit_variant(self) -> Result<()> {
		Err(Error::UnitVariantPair)
	}
	fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
	where T: de::DeserializeSeed<'nu> {
		seed.deserialize(self.de)
	}
	fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
	where V: de::Visitor<'nu> {
		de::Deserializer::deserialize_seq(self.de, visitor)
	}
	fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
	where V: de::Visitor<'nu> {
		de::Deserializer::deserialize_struct(self.de, "", &[], visitor)
	}
}
