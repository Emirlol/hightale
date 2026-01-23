#![allow(unused_variables)]

use bytes::Buf;
use uuid::Uuid;
use macros::define_packet;
use crate::{
	codec::FixedAscii,
	define_enum,
	v1::{
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
			required protocol_hash: FixedAscii<64>,
			required client_type: ClientType,
			required uuid: Uuid,
		}
		variable {
			opt(1) language: String,
			opt(2) identity_token: String,
			required username: String,
			opt(4) referral_data: Vec<u8>,
			opt(8) referral_source: HostAddress
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
