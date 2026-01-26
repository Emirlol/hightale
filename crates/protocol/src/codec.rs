use std::{
	collections::{
		hash_map::Entry,
		HashMap,
	},
	fmt,
	fmt::{
		Debug,
		Display,
	},
	hash::Hash,
	ops::Deref,
	string::FromUtf8Error,
};

use bytes::{
	Buf,
	BufMut,
	Bytes,
	BytesMut,
};
use fmt::Formatter;
use ordered_float::OrderedFloat;
use thiserror::Error;
use uuid::Uuid;

use crate::v2::MAX_SIZE;

// A helper trait to allow casting from any primitive to your specific types
pub trait PacketNum {
	fn to_usize(self) -> usize;
	fn to_i32(self) -> i32;
}

// Macro to implement this trait for all primitive types easily
macro_rules! impl_packet_num {
    ($($t:ty),*) => {
        $(
            impl PacketNum for $t {
                #[inline] fn to_usize(self) -> usize { self as usize }
                #[inline] fn to_i32(self) -> i32 { self as i32 }
            }
        )*
    };
}

impl_packet_num!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

#[derive(Debug, Error)]
pub enum PacketError {
	#[error("In field '{0}': {1}")]
	Context(String, #[source] Box<PacketError>),
	#[error("Incomplete packet")]
	Incomplete,
	#[error("Incomplete packet: expected at least {expected} bytes, found {found} bytes")]
	IncompleteBytes { found: usize, expected: usize },
	#[error("Incomplete packet: expected exactly {expected} bytes, found {found} bytes")]
	IncompleteBytesExact { found: usize, expected: usize },
	#[error("Decoding took more bytes than given padding bytes: actual: {actual} > expected max: {pad}")]
	DecodedMoreThanPadding { actual: usize, pad: usize },
	#[error("String too long: actual: {actual} > expected max: {max_expected}")]
	StringTooLong { actual: usize, max_expected: usize },
	#[error("Negative length: {0}")]
	NegativeLength(i32),
	#[error("Collection too large: actual: {actual} > expected max: {max_expected}")]
	CollectionTooLarge { actual: usize, max_expected: usize },
	#[error("Invalid enum variant '{0}'")]
	InvalidEnumVariant(u8),
	#[error("Duplicate key '{0}' in map")]
	DuplicateKey(String),
	#[error("Invalid VarInt")]
	InvalidVarInt,
	#[error("UTF8 Error: {0}")]
	Utf8(#[from] FromUtf8Error),
	#[error("Non-ASCII character in ASCII string")]
	NonAscii,
}

impl PacketError {
	#[inline]
	pub fn incomplete_bytes(found: impl PacketNum, expected: impl PacketNum) -> Self {
		Self::IncompleteBytes {
			found: found.to_usize(),
			expected: expected.to_usize(),
		}
	}

	#[inline]
	pub fn incomplete_bytes_exact(found: impl PacketNum, expected: impl PacketNum) -> Self {
		Self::IncompleteBytesExact {
			found: found.to_usize(),
			expected: expected.to_usize(),
		}
	}

	#[inline]
	pub fn decoded_more_than_padding(actual: impl PacketNum, pad: impl PacketNum) -> Self {
		Self::DecodedMoreThanPadding {
			actual: actual.to_usize(),
			pad: pad.to_usize(),
		}
	}

	#[inline]
	pub fn string_too_long(actual: impl PacketNum, max_expected: impl PacketNum) -> Self {
		Self::StringTooLong {
			actual: actual.to_usize(),
			max_expected: max_expected.to_usize(),
		}
	}

	#[inline]
	pub fn collection_too_large(actual: impl PacketNum, max_expected: impl PacketNum) -> Self {
		Self::CollectionTooLarge {
			actual: actual.to_usize(),
			max_expected: max_expected.to_usize(),
		}
	}

	#[inline]
	pub fn negative_length(len: impl PacketNum) -> Self {
		Self::NegativeLength(len.to_i32())
	}
}

pub trait PacketContext<T> {
	fn context(self, field: &str) -> PacketResult<T>;
}

impl<T> PacketContext<T> for PacketResult<T> {
	fn context(self, field: &str) -> PacketResult<T> {
		self.map_err(|e| PacketError::Context(field.to_string(), Box::new(e)))
	}
}

pub type PacketResult<T> = Result<T, PacketError>;

/// A trait for types that can be written to the buffer
pub trait HytaleCodec: Sized {
	/// Encodes the instance into the buffer.
	fn encode(&self, buf: &mut BytesMut);
	/// Decodes an instance of the type from the buffer.
	/// This must not consume more bytes than necessary, as the same buffer may be used to decode multiple fields.
	fn decode(buf: &mut impl Buf) -> PacketResult<Self>;
}

// --- Primitives ---

impl HytaleCodec for bool {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u8(if *self { 1 } else { 0 });
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if !buf.has_remaining() {
			return Err(PacketError::Incomplete);
		}
		Ok(buf.get_u8() != 0)
	}
}

impl HytaleCodec for f64 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_f64_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 8 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 4));
		}
		Ok(buf.get_f64_le())
	}
}

impl HytaleCodec for f32 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_f32_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 4 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 4));
		}
		Ok(buf.get_f32_le())
	}
}

impl HytaleCodec for i8 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_i8(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if !buf.has_remaining() {
			return Err(PacketError::incomplete_bytes_exact(0, 1));
		}
		Ok(buf.get_i8())
	}
}

impl HytaleCodec for i16 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_i16_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 2 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 2));
		}
		Ok(buf.get_i16_le())
	}
}

impl HytaleCodec for i32 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_i32_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 4 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 4));
		}
		Ok(buf.get_i32_le())
	}
}

impl HytaleCodec for i64 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_i64_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 8 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 8));
		}
		Ok(buf.get_i64_le())
	}
}

impl HytaleCodec for i128 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_i128_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 16 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 16));
		}
		Ok(buf.get_i128_le())
	}
}

impl HytaleCodec for u8 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u8(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if !buf.has_remaining() {
			return Err(PacketError::incomplete_bytes_exact(0, 1));
		}
		Ok(buf.get_u8())
	}
}

impl HytaleCodec for u16 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u16_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 2 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 2));
		}
		Ok(buf.get_u16_le())
	}
}

impl HytaleCodec for u32 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u32_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 4 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 4));
		}
		Ok(buf.get_u32_le())
	}
}

impl HytaleCodec for u64 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u64_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 8 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 8));
		}
		Ok(buf.get_u64_le())
	}
}

impl HytaleCodec for u128 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u128_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 16 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 16));
		}
		Ok(buf.get_u128_le())
	}
}

// isize and usize explicitly aren't implemented because their size is platform-dependent and aren't reliable for network protocols

/// A variable-length integer encoding (VarInt) as used in Hytale protocol.
/// Encodes integers in 7-bit chunks, with the top bit of each byte indicating if more bytes follow.
#[derive(Debug, Clone, Copy, Default)]
pub struct VarInt(pub i32);

impl HytaleCodec for VarInt {
	fn encode(&self, buf: &mut BytesMut) {
		let mut x = self.0 as u32;
		loop {
			let mut temp = (x & 0b01111111) as u8;
			x >>= 7;
			if x != 0 {
				temp |= 0b10000000; // Top bit indicates more bytes follow
			}
			buf.put_u8(temp);
			if x == 0 {
				break;
			}
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let mut num_read = 0;
		let mut result = 0;
		loop {
			if !buf.has_remaining() {
				return Err(PacketError::Incomplete);
			}
			let read = buf.get_u8();
			let value = (read & 0b01111111) as i32;
			result |= value << (7 * num_read);
			num_read += 1;
			if num_read > 5 {
				return Err(PacketError::InvalidVarInt);
			}
			if (read & 0b10000000) == 0 {
				// Top bit not set, end of VarInt
				break;
			}
		}
		Ok(VarInt(result))
	}
}

impl HytaleCodec for String {
	fn encode(&self, buf: &mut BytesMut) {
		VarInt(self.len() as i32).encode(buf);
		buf.put_slice(self.as_bytes());
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let len_raw = VarInt::decode(buf)?.0;
		if len_raw < 0 {
			return Err(PacketError::negative_length(len_raw));
		}
		if len_raw > MAX_SIZE {
			return Err(PacketError::string_too_long(len_raw, MAX_SIZE));
		}
		let len = len_raw as usize;
		let remaining = buf.remaining();
		if remaining < len {
			return Err(PacketError::incomplete_bytes_exact(remaining, len));
		}
		let bytes = buf.copy_to_bytes(len);
		let str = String::from_utf8(bytes.to_vec())?;
		Ok(str)
	}
}

/// A wrapper for ASCII strings stored as Bytes.
#[derive(Clone)]
pub struct AsciiString(Bytes);

impl TryFrom<Bytes> for AsciiString {
	type Error = PacketError;

	fn try_from(value: Bytes) -> Result<Self, Self::Error> {
		if value.iter().all(|&b| b.is_ascii()) {
			Ok(AsciiString(value))
		} else {
			Err(PacketError::NonAscii)
		}
	}
}

impl Display for AsciiString {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl Debug for AsciiString {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl AsciiString {
	pub fn as_str(&self) -> &str {
		// Validated on creation
		unsafe { str::from_utf8_unchecked(&self.0) }
	}
}

impl AsRef<str> for AsciiString {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl Deref for AsciiString {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl HytaleCodec for AsciiString {
	fn encode(&self, buf: &mut BytesMut) {
		VarInt(self.0.len() as i32).encode(buf);
		buf.put_slice(&self.0);
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let len_raw = VarInt::decode(buf)?.0;
		if len_raw < 0 {
			return Err(PacketError::negative_length(len_raw));
		}
		if len_raw > MAX_SIZE {
			return Err(PacketError::string_too_long(len_raw, MAX_SIZE));
		}
		let len = len_raw as usize;
		let remaining = buf.remaining();
		if remaining < len {
			return Err(PacketError::incomplete_bytes_exact(remaining, len));
		}
		let bytes = buf.copy_to_bytes(len);
		if bytes.iter().all(|&b| b.is_ascii()) {
			Ok(AsciiString(bytes))
		} else {
			Err(PacketError::NonAscii)
		}
	}
}

/// A wrapper for Vec<Option<T>> that encodes validity as a bitmask before the items.
#[derive(Debug, Clone)]
pub struct BitOptionVec<T>(pub Vec<Option<T>>);

impl<T> Deref for BitOptionVec<T> {
	type Target = Vec<Option<T>>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> From<Vec<Option<T>>> for BitOptionVec<T> {
	fn from(v: Vec<Option<T>>) -> Self {
		BitOptionVec(v)
	}
}

impl<T: HytaleCodec> HytaleCodec for BitOptionVec<T> {
	fn encode(&self, buf: &mut BytesMut) {
		let v = &self.0;
		VarInt(v.len() as i32).encode(buf);

		let bitfield_len = v.len().div_ceil(8);
		let mut bits = vec![0u8; bitfield_len];
		for (i, item) in v.iter().enumerate() {
			if item.is_some() {
				bits[i / 8] |= 1 << (i % 8);
			}
		}
		buf.put_slice(&bits);

		for item in v.iter().flatten() {
			item.encode(buf);
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let count_raw = VarInt::decode(buf)?.0;
		if count_raw < 0 {
			return Err(PacketError::negative_length(count_raw));
		} else if count_raw == 0 {
			return Ok(BitOptionVec(vec![]));
		} else if count_raw > MAX_SIZE {
			return Err(PacketError::collection_too_large(count_raw, MAX_SIZE));
		}

		let count = count_raw as usize;
		let bitfield_len = count.div_ceil(8);

		let remaining = buf.remaining();
		if remaining < bitfield_len {
			return Err(PacketError::incomplete_bytes(remaining, bitfield_len));
		}

		let mut bits = vec![0u8; bitfield_len];
		buf.copy_to_slice(&mut bits);

		let mut list = Vec::with_capacity(count);
		for i in 0..count {
			if (bits[i / 8] & (1 << (i % 8))) != 0 {
				list.push(Some(T::decode(buf)?));
			} else {
				list.push(None);
			}
		}
		Ok(BitOptionVec(list))
	}
}

/// Wrapper for fixed-length strings (char array in C-like protocols).
/// Encodes as exactly N bytes, padding with nulls if shorter, truncating if longer.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedAscii<const N: usize>(pub [u8; N]);

impl<const N: usize> From<&str> for FixedAscii<N> {
	fn from(s: &str) -> Self {
		let mut bytes = [0u8; N];
		let input = s.as_bytes();
		// Truncate if input is longer than N
		let len = input.len().min(N);
		bytes[..len].copy_from_slice(&input[..len]);
		FixedAscii(bytes)
	}
}

impl<const N: usize> Display for FixedAscii<N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		// Find the first null byte to determine "real" length
		let len = self.0.iter().position(|&b| b == 0).unwrap_or(N);
		let s = String::from_utf8_lossy(&self.0[..len]);
		write!(f, "{}", s)
	}
}

impl<const N: usize> Debug for FixedAscii<N> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		<Self as Display>::fmt(self, f)
	}
}

impl<const N: usize> From<FixedAscii<N>> for String {
	fn from(val: FixedAscii<N>) -> Self {
		val.to_string()
	}
}

impl<const N: usize> Deref for FixedAscii<N> {
	type Target = [u8; N];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<const N: usize> HytaleCodec for FixedAscii<N> {
	fn encode(&self, buf: &mut BytesMut) {
		let bytes = self.0;
		let write_len = bytes.len().min(N);

		buf.put_slice(&bytes[..write_len]);

		if write_len < N {
			buf.put_bytes(0, N - write_len);
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < N {
			return Err(PacketError::incomplete_bytes_exact(remaining, N));
		}

		let mut bytes = [0u8; N];
		buf.copy_to_slice(&mut bytes);

		Ok(FixedAscii(bytes))
	}
}

impl HytaleCodec for Bytes {
	fn encode(&self, buf: &mut BytesMut) {
		VarInt(self.len() as i32).encode(buf);
		buf.put_slice(self);
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let len = VarInt::decode(buf)?.0 as usize;
		let remaining = buf.remaining();
		if remaining < len {
			return Err(PacketError::incomplete_bytes_exact(remaining, len));
		}
		Ok(buf.copy_to_bytes(len))
	}
}

impl HytaleCodec for Uuid {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u128(self.as_u128());
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let remaining = buf.remaining();
		if remaining < 16 {
			return Err(PacketError::incomplete_bytes_exact(remaining, 16));
		}
		Ok(Uuid::from_u128(buf.get_u128()))
	}
}

impl<T: HytaleCodec> HytaleCodec for Vec<T> {
	fn encode(&self, buf: &mut BytesMut) {
		VarInt(self.len() as i32).encode(buf);
		for item in self {
			item.encode(buf);
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let len = VarInt::decode(buf)?.0 as usize;
		let remaining = buf.remaining();
		if remaining < len {
			return Err(PacketError::incomplete_bytes_exact(remaining, len));
		}
		let mut out = Vec::with_capacity(len);
		for _ in 0..len {
			out.push(T::decode(buf)?);
		}
		Ok(out)
	}
}

impl<K, V: HytaleCodec> HytaleCodec for HashMap<K, V>
where
	K: HytaleCodec + Eq + Hash + Debug,
	V: HytaleCodec,
{
	fn encode(&self, buf: &mut BytesMut) {
		VarInt(self.len() as i32).encode(buf);
		for (key, value) in self {
			key.encode(buf);
			value.encode(buf);
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let len_raw = VarInt::decode(buf)?.0;
		if len_raw < 0 {
			return Err(PacketError::negative_length(len_raw));
		}
		if len_raw > MAX_SIZE {
			return Err(PacketError::collection_too_large(len_raw, MAX_SIZE));
		}
		let len = len_raw as usize;
		let mut map = HashMap::with_capacity(len);
		for _ in 0..len {
			let key = K::decode(buf)?;
			let value = V::decode(buf)?;
			match map.entry(key) {
				Entry::Occupied(entry) => return Err(PacketError::DuplicateKey(format!("{:?}", entry.key()))),
				Entry::Vacant(entry) => {
					entry.insert(value);
				}
			}
		}
		Ok(map)
	}
}

impl<T: HytaleCodec> HytaleCodec for Box<T> {
	fn encode(&self, buf: &mut BytesMut) {
		(**self).encode(buf);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		Ok(Box::new(T::decode(buf)?))
	}
}

impl<T: HytaleCodec + Copy> HytaleCodec for OrderedFloat<T> {
	fn encode(&self, buf: &mut BytesMut) {
		self.0.encode(buf);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		Ok(OrderedFloat(T::decode(buf)?))
	}
}

// Tuples
macro_rules! tuple_codec_impl {
    ($($name:ident),+) => {
	    // Trailing commas are necessary for this to work with single-element tuples. All tuple definitions must end with a trailing comma.
	    // This one is fine       v   since it's not a tuple, it's just the generic parameter list.
		impl<$($name: HytaleCodec),+> HytaleCodec for ($($name,)+) {
			fn encode(&self, buf: &mut BytesMut) {
				#[allow(non_snake_case)] // It's a lot of work to make the names snake_case while keeping the types UpperCamelCase
				let ($($name,)+) = self;
				$(
					$name.encode(buf);
				)+
			}
			fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
				Ok((
					$(
						$name::decode(buf).context(stringify!($name))?,
					)+
				))
			}
		}
	};
}

macro_rules! impl_codec_all_tuples {
    () => {};

    ($first:ident, $($rest:ident),+) => {
        tuple_codec_impl!($first, $($rest),+);
        impl_codec_all_tuples!($($rest),+);
    };
    ($first:ident) => {
        tuple_codec_impl!($first);
    };
}
impl_codec_all_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

pub trait FixedSize {
	const SIZE: usize;
}

macro_rules! impl_fixed_size_primitives {
    ($($name:ty),+ $(,)?) => {
	    $(impl FixedSize for $name {
		    const SIZE: usize = std::mem::size_of::<$name>();
	    })+
    };
}

impl_fixed_size_primitives!((), bool, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, char);

macro_rules! tuple_fixed_size_impl {
    ($($name:ident),+) => {
		impl<$($name: FixedSize),+> FixedSize for ($($name,)+) {
			const SIZE: usize = 0 $(+ $name::SIZE)+;
		}
	};
}

macro_rules! impl_fixed_size_all_tuples {
	() => {};

	($first:ident, $($rest:ident),+) => {
		tuple_fixed_size_impl!($first, $($rest),+);
		impl_fixed_size_all_tuples!($($rest),+);
	};
	($first:ident) => {
		tuple_fixed_size_impl!($first);
	};
}

impl_fixed_size_all_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

impl FixedSize for Uuid {
	const SIZE: usize = 16;
}

impl<const N: usize> FixedSize for FixedAscii<N> {
	const SIZE: usize = N;
}

impl<T: FixedSize> FixedSize for Box<T> {
	const SIZE: usize = T::SIZE;
}

impl<T: FixedSize> FixedSize for OrderedFloat<T> {
	const SIZE: usize = T::SIZE;
}
