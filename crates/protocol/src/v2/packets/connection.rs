use bytes::Bytes;
use macros::define_packet;
use uuid::Uuid;

use crate::{
	codec::{
		AsciiString,
		BoundedVarLen,
		FixedAscii,
	},
	define_enum,
	v2::{
		HostAddress,
		InstantData,
	},
};

define_enum! {
	pub enum ClientType {
		Game = 0,
		Editor = 1,
	}
}

define_packet! {
	Connect {
		fixed {
			required protocol_crc: i32,
			required protocol_build_number: i32,
			required client_version: FixedAscii<20>,
			required client_type: ClientType,
			required uuid: Uuid,
		}
		variable {
			required username: BoundedVarLen<AsciiString, 16>,
			opt(1) identity_token: BoundedVarLen<String, 8192>,
			required language: BoundedVarLen<AsciiString, 16>,
			opt(2) referral_data: BoundedVarLen<Bytes, 4096>,
			opt(4) referral_source: HostAddress
		}
	}
}

define_enum! {
	pub enum DisconnectType {
		Disconnect = 0,
		Crash = 1,
	}
}

define_packet! {
	Disconnect {
		fixed {
			required disconnect_type: DisconnectType,
		}
		variable {
			opt(1) reason: String
		}
	}
}

define_packet! {
	Ping {
		fixed {
			required id: i32,
			opt(1) time: InstantData,
			required last_ping_raw: i32,
			required last_ping_direct: i32,
			required last_ping_tick: i32,
		}
	}
}

define_enum! {
	pub enum PongType {
		Raw = 0,
		Direct = 1,
		Tick = 2
	}
}

define_packet! {
	Pong {
		fixed {
			required id: i32,
			opt(1) time: InstantData,
			required pong_type: PongType,
			required packet_queue_size: i16,
		}
	}
}
