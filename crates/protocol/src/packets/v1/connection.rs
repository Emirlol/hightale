#![allow(unused_variables)]

use bytes::Buf;
use uuid::Uuid;

use crate::{
	codec::FixedAscii,
	define_enum,
	define_packet,
	packets::{
		v1::InstantData,
		HostAddress,
	},
};

define_enum! {
	pub enum ClientType {
		Game = 0,
		Editor = 1,
	}
}

define_packet!(
	Connect {
		fixed {
			protocol_hash: FixedAscii<64>,
			client_type: ClientType,
			uuid: Uuid,
		}
		variable {
			opt language: String,
			opt identity_token: String,
			required username: String,
			opt referral_data: Vec<u8>,
			opt referral_source: HostAddress
		}
	}
);

define_enum! {
	pub enum DisconnectType {
		Disconnect = 0,
		Crash = 1,
	}
}

define_packet!(
	Disconnect {
		bitmask {
			required disconnect_type: DisconnectType,
			opt reason: String
		}
	}
);

define_packet!(
	Ping {
		bitmask {
			required id: i32,
			opt time: InstantData [pad=12],
			required last_ping_raw: i32,
			required last_ping_direct: i32,
			required last_ping_tick: i32,
		}
	}
);

define_enum! {
	pub enum PongType {
		Raw = 0,
		Direct = 1,
		Tick = 2
	}
}

define_packet!(
	Pong {
		bitmask {
			required id: i32,
			opt time: InstantData [pad=12],
			required pong_type: PongType,
			required packet_queue_size: i16,
		}
	}
);
