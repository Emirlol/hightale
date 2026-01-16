#![allow(clippy::vec_init_then_push)]
#![allow(unused)]

use bytes::BytesMut;
use tracing::debug;
use uuid::Uuid;

use crate::{
	codec::{
		FixedAscii,
		HytaleCodec,
		VarInt,
	},
	define_packet,
};


#[derive(Debug, Clone)]
pub struct HostAddress {
	pub port: u16,
	pub host: String,
}

impl HytaleCodec for HostAddress {
	fn encode(&self, buf: &mut BytesMut) {
		self.port.encode(buf);
		self.host.encode(buf);
	}
	fn decode(buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::codec::PacketError> {
		Ok(HostAddress {
			port: <u16 as HytaleCodec>::decode(buf)?,
			host: <String as HytaleCodec>::decode(buf)?,
		})
	}
}

#[derive(Debug)]
pub struct ConnectPacket {
	pub protocol_hash: String, // Fixed 64 bytes
	pub client_type: u8,
	pub uuid: Uuid,
	pub language: Option<String>,
	pub identity_token: Option<String>,
	pub username: String,
	pub referral_data: Option<Vec<u8>>,
	pub referral_source: Option<HostAddress>,
}

impl HytaleCodec for ConnectPacket {
	fn encode(&self, _buf: &mut BytesMut) {
		unimplemented!("Server does not send Connect");
	}

	fn decode(buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::codec::PacketError> {
		use bytes::Buf;

		#[cfg(debug_assertions)]
		debug!("Decoding Connect. Buffer Len: {}, Pos: {}", buf.get_ref().len(), buf.position());

		if buf.remaining() < 102 {
			return Err(crate::codec::PacketError::Incomplete);
		}
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
			Some(<String as HytaleCodec>::decode(buf)?)
		} else {
			None
		};

		let identity_token = if (null_bits & 2) != 0 {
			buf.set_position((var_start as i32 + token_off) as u64);
			Some(<String as HytaleCodec>::decode(buf)?)
		} else {
			None
		};

		#[cfg(debug_assertions)]
		debug!("Jumping to Username at {}", var_start as i32 + user_off);

		// Username (Always present)
		buf.set_position((var_start as i32 + user_off) as u64);
		let username = <String as HytaleCodec>::decode(buf)?;

		#[cfg(debug_assertions)]
		debug!("Username decoded: {}", username);

		let referral_data = if (null_bits & 4) != 0 {
			buf.set_position((var_start as i32 + ref_data_off) as u64);
			Some(<Vec<u8> as HytaleCodec>::decode(buf)?)
		} else {
			None
		};

		let referral_source = if (null_bits & 8) != 0 {
			buf.set_position((var_start as i32 + ref_src_off) as u64);
			Some(HostAddress::decode(buf)?)
		} else {
			None
		};

		Ok(ConnectPacket {
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

define_packet!(
	DisconnectPacket (id = 1) {
		reason: String,
		type_id: VarInt,
	}
);

define_packet!(
	StatusPacket (id = 10) {
		fixed {
			player_count: i32,
			max_players: i32,
		}
		variable {
			name: Option<String>,
			motd: Option<String>,
		}
	}
);

define_packet!(
	AuthGrantPacket (id = 11) {
		fixed {}
		variable {
			auth_grant: Option<String>,
			server_identity: Option<String>,
		}
	}
);

define_packet!(
	AuthTokenPacket (id = 12) {
		fixed {}
		variable {
			access_token: Option<String>,
			server_grant: Option<String>,
		}
	}
);

define_packet!(
	ServerAuthTokenPacket (id = 13) {
		fixed {}
		variable {
			server_access_token: Option<String>,
			password_challenge: Option<Vec<u8>>,
		}
	}
);

// In Java, this is actually a bitmask packet, but effectively sequential because it doesn't use the offset table logic (it checks bitmask then reads immediately).
// We treat it as a special case and just implement manually since it's tiny.
// Manual implementation is safest here as it deviates from the standard "Offset" pattern.
#[derive(Debug, Clone)]
pub struct ConnectAcceptPacket {
	pub password_challenge: Option<Vec<u8>>,
}

impl HytaleCodec for ConnectAcceptPacket {
	fn encode(&self, buf: &mut BytesMut) {
		use bytes::BufMut;
		let mut null_bits = 0;
		if self.password_challenge.is_some() {
			null_bits |= 1;
		}
		buf.put_u8(null_bits);
		if let Some(bytes) = &self.password_challenge {
			bytes.encode(buf);
		}
	}

	fn decode(buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::codec::PacketError> {
		use bytes::Buf;
		if !buf.has_remaining() {
			return Err(crate::codec::PacketError::Incomplete);
		}
		let null_bits = buf.get_u8();
		let password_challenge = if (null_bits & 1) != 0 { Some(Vec::<u8>::decode(buf)?) } else { None };
		Ok(Self { password_challenge })
	}
}

#[derive(Debug, Clone)]
pub struct Asset {
	pub hash: FixedAscii<64>, // 64-char Hex String
	pub name: String,         // Filename (e.g. "models/player.json")
}

impl HytaleCodec for Asset {
	fn encode(&self, buf: &mut BytesMut) {
		<FixedAscii<64> as HytaleCodec>::encode(&self.hash, buf);
		<String as HytaleCodec>::encode(&self.name, buf);
	}

	fn decode(buf: &mut std::io::Cursor<&[u8]>) -> Result<Self, crate::codec::PacketError> {
		Ok(Asset {
			hash: <FixedAscii<64> as HytaleCodec>::decode(buf)?,
			name: <String as HytaleCodec>::decode(buf)?,
		})
	}
}

define_packet!(
	WorldSettingsPacket (id = 20) {
		fixed {
			world_height: i32,
		}
		variable {
			// If None, client assumes defaults.
			// If Some, client compares hashes.
			required_assets: Option<Vec<Asset>>,
		}
	}
);

define_packet!(
	WorldLoadProgressPacket (id = 21) {
		fixed {
			percent_complete: i32,
			percent_complete_subitem: i32,
		}
		variable {
			status: Option<String>,
		}
	}
);

define_packet!(
	WorldLoadFinishedPacket (id = 22) {
		fixed {}
		variable {}
	}
);

define_packet!(
	RequestAssetsPacket (id = 23) {
		fixed {}
		variable {
			// The list of assets the client needs us to upload.
			// If empty or None, client is happy.
			assets: Option<Vec<Asset>>,
		}
	}
);
