use bytes::BytesMut;
use crate::codec::HytaleCodec;
use crate::define_packet;

define_packet!(
	Status {
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
	AuthGrant {
		fixed {}
		variable {
			auth_grant: Option<String>,
			server_identity: Option<String>,
		}
	}
);

define_packet!(
	AuthToken {
		fixed {}
		variable {
			access_token: Option<String>,
			server_grant: Option<String>,
		}
	}
);

define_packet!(
	ServerAuthToken {
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
pub struct ConnectAccept {
	pub password_challenge: Option<Vec<u8>>,
}

impl HytaleCodec for ConnectAccept {
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

	fn decode(buf: &mut impl bytes::Buf) -> Result<Self, crate::codec::PacketError> {
		use bytes::Buf;
		if !buf.has_remaining() {
			return Err(crate::codec::PacketError::Incomplete);
		}
		let null_bits = buf.get_u8();
		let password_challenge = if (null_bits & 1) != 0 { Some(Vec::<u8>::decode(buf)?) } else { None };
		Ok(Self { password_challenge })
	}
}