use std::io::Cursor;

use bytes::{
	Buf,
	BytesMut,
};
use tracing::debug;
use uuid::Uuid;

use crate::{
	codec::{
		HytaleCodec,
		VarInt,
	},
	define_packet,
	packets::HostAddress,
};

#[derive(Debug, Clone)]
pub struct Connect {
	pub protocol_hash: String, // Fixed 64 bytes
	pub client_type: u8,
	pub uuid: Uuid,
	pub language: Option<String>,
	pub identity_token: Option<String>,
	pub username: String,
	pub referral_data: Option<Vec<u8>>,
	pub referral_source: Option<HostAddress>,
}

impl HytaleCodec for Connect {
	fn encode(&self, _buf: &mut BytesMut) {
		unimplemented!("Server does not send Connect");
	}

	fn decode(buf: &mut impl Buf) -> Result<Self, crate::codec::PacketError> {
		#[cfg(debug_assertions)]
		debug!("Decoding Connect Packet, remaining: {}", buf.remaining());

		if buf.remaining() < 102 {
			return Err(crate::codec::PacketError::Incomplete);
		}

		let mut buf = Cursor::new(buf.copy_to_bytes(buf.remaining()));
		let start_pos = buf.position() as usize;

		let null_bits = buf.get_u8();

		// Protocol Hash (Fixed 64 bytes ASCII)
		let mut hash_bytes = vec![0u8; 64];
		buf.copy_to_slice(&mut hash_bytes);
		// Trim nulls if any
		let protocol_hash = String::from_utf8_lossy(&hash_bytes).trim_matches(char::from(0)).to_string();

		let client_type = buf.get_u8();
		let uuid = Uuid::from_u128(buf.get_u128());

		// Read Offsets (i32 LE)
		// 5 offsets at: 82, 86, 90, 94, 98 (relative to start+1?)
		// Java: buf.getIntLE(offset + 82)
		// We are currently at position 1 + 64 + 1 + 16 = 82. Perfect.

		let lang_off = buf.get_i32_le();
		let token_off = buf.get_i32_le();
		let user_off = buf.get_i32_le();
		let ref_data_off = buf.get_i32_le();
		let ref_src_off = buf.get_i32_le();

		#[cfg(debug_assertions)]
		debug!("Offsets: Lang={}, Token={}, User={}, RefData={}, RefSrc={}", lang_off, token_off, user_off, ref_data_off, ref_src_off);

		// Variable Block Start = 102 (relative to packet start)
		// buf position is now 102.
		let var_start = start_pos + 102;

		let language = if (null_bits & 1) != 0 {
			buf.set_position((var_start as i32 + lang_off) as u64);
			Some(<String as HytaleCodec>::decode(&mut buf)?)
		} else {
			None
		};

		let identity_token = if (null_bits & 2) != 0 {
			buf.set_position((var_start as i32 + token_off) as u64);
			Some(<String as HytaleCodec>::decode(&mut buf)?)
		} else {
			None
		};

		#[cfg(debug_assertions)]
		debug!("Jumping to Username at {}", var_start as i32 + user_off);

		// Username (Always present)
		buf.set_position((var_start as i32 + user_off) as u64);
		let username = <String as HytaleCodec>::decode(&mut buf)?;

		#[cfg(debug_assertions)]
		debug!("Username decoded: {}", username);

		let referral_data = if (null_bits & 4) != 0 {
			buf.set_position((var_start as i32 + ref_data_off) as u64);
			Some(<Vec<u8> as HytaleCodec>::decode(&mut buf)?)
		} else {
			None
		};

		let referral_source = if (null_bits & 8) != 0 {
			buf.set_position((var_start as i32 + ref_src_off) as u64);
			Some(HostAddress::decode(&mut buf)?)
		} else {
			None
		};

		Ok(Connect {
			protocol_hash,
			client_type,
			uuid,
			language,
			identity_token,
			username,
			referral_data,
			referral_source,
		})
	}
}

define_packet!(Disconnect { reason: String, type_id: VarInt });
