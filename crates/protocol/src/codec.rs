use std::{
	fmt,
	string::FromUtf8Error,
};

use bytes::{
	Buf,
	BufMut,
	Bytes,
	BytesMut,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PacketError {
	#[error("Incomplete packet")]
	Incomplete,
	#[error("String too long")]
	StringTooLong,
	#[error("Invalid VarInt")]
	InvalidVarInt,
	#[error("UTF8 Error: {0}")]
	Utf8(#[from] FromUtf8Error),
}

pub type PacketResult<T> = Result<T, PacketError>;

/// A trait for types that can be written to the buffer
pub trait HytaleCodec: Sized {
	fn encode(&self, buf: &mut BytesMut);
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

impl HytaleCodec for i64 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_i64_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 8 {
			return Err(PacketError::Incomplete);
		}
		Ok(buf.get_i64_le())
	}
}

impl HytaleCodec for i32 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_i32_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 4 {
			return Err(PacketError::Incomplete);
		}
		Ok(buf.get_i32_le())
	}
}

impl HytaleCodec for u16 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u16_le(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 2 {
			return Err(PacketError::Incomplete);
		}
		Ok(buf.get_u16_le())
	}
}

impl HytaleCodec for u8 {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u8(*self);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if !buf.has_remaining() {
			return Err(PacketError::Incomplete);
		}
		Ok(buf.get_u8())
	}
}

// Wrapper for VarInts (so we can distinguish i32 vs VarInt in macros)
#[derive(Debug, Clone, Copy, Default)]
pub struct VarInt(pub i32);

impl HytaleCodec for VarInt {
	fn encode(&self, buf: &mut BytesMut) {
		let mut x = self.0 as u32;
		loop {
			let mut temp = (x & 0x7F) as u8;
			x >>= 7;
			if x != 0 {
				temp |= 0x80;
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
			let value = (read & 0x7F) as i32;
			result |= value << (7 * num_read);
			num_read += 1;
			if num_read > 5 {
				return Err(PacketError::InvalidVarInt);
			}
			if (read & 0x80) == 0 {
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
		let len = VarInt::decode(buf)?.0 as usize;
		if buf.remaining() < len {
			return Err(PacketError::Incomplete);
		}
		let bytes = buf.copy_to_bytes(len);
		let str = String::from_utf8(bytes.to_vec())?;
		Ok(str)
	}
}

/// Wrapper for fixed-length strings (char array in C-like protocols).
/// Encodes as exactly N bytes, padding with nulls if shorter, truncating if longer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl<const N: usize> fmt::Display for FixedAscii<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Find the first null byte to determine "real" length
		let len = self.0.iter().position(|&b| b == 0).unwrap_or(N);
		let s = String::from_utf8_lossy(&self.0[..len]);
		write!(f, "{}", s)
	}
}

impl<const N: usize> From<FixedAscii<N>> for String {
	fn from(val: FixedAscii<N>) -> Self {
		val.to_string()
	}
}

impl<const N: usize> std::ops::Deref for FixedAscii<N> {
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
		if buf.remaining() < N {
			return Err(PacketError::Incomplete);
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
		if buf.remaining() < len {
			return Err(PacketError::Incomplete);
		}
		Ok(buf.copy_to_bytes(len))
	}
}

impl HytaleCodec for Uuid {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_u128(self.as_u128());
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 16 {
			return Err(PacketError::Incomplete);
		}
		Ok(Uuid::from_u128(buf.get_u128()))
	}
}

impl<T: HytaleCodec> HytaleCodec for Option<T> {
	fn encode(&self, buf: &mut BytesMut) {
		if let Some(inner) = self {
			inner.encode(buf);
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if !buf.has_remaining() {
			return Err(PacketError::Incomplete);
		}
		let present = buf.get_u8() != 0;
		if present { Ok(Some(T::decode(buf)?)) } else { Ok(None) }
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
		let mut out = Vec::with_capacity(len);
		for _ in 0..len {
			out.push(T::decode(buf)?);
		}
		Ok(out)
	}
}
