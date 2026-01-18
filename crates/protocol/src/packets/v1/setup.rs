#![allow(unused_variables, unused_imports)]

use std::collections::HashMap;
use std::io::Cursor;
use crate::{define_enum, define_packet, packets::Asset};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::codec::{HytaleCodec, PacketContext, PacketError, PacketResult};

// Simple signal packet with no data
define_packet!(AssetFinalize {});

define_packet!(
	AssetInitialize {
		size: i32,
		asset: Asset
	}
);

define_packet!(
	AssetPart {
		fixed {
			opt part: Bytes
		}
	}
);

define_enum! {
	pub enum ClientFeature {
		SplitVelocity = 0,
		Mantling = 1,
		SprintForce = 2,
		CrouchSlide = 3,
		SafetyRoll= 4,
		DisplayHealthBars = 5,
		DisplayCombatText = 6,
	}
}

define_packet!(
	PlayerSkin {
		mask_size: 3,
		variable {
			opt(0) body_characteristic: String,
			opt(1) underwear: String,
			opt(2) face: String,
			opt(3) eyes: String,
			opt(4) ears: String,
			opt(5) mouth: String,
			opt(6) facial_hair: String,
			opt(7) haircut: String,
			opt(8) eyebrows: String,
			opt(9) pants: String,
			opt(10) overpants: String,
			opt(11) undertop: String,
			opt(12) overtop: String,
			opt(13) shoes: String,
			opt(14) head_accessory: String,
			opt(15) face_accessory: String,
			opt(16) ear_accessory: String,
			opt(17) skin_feature: String,
			opt(18) gloves: String,
			opt(19) cape: String,
		}
	}
);


define_packet!(
	PlayerOptions {
		fixed {
			opt player_skin: Box<PlayerSkin>, // Box it since it's like 480 bytes and it bloats the largest packet size in the packet enum
		}
	}
);

define_packet!(
	RemoveAssets {
		fixed {
			opt assets: Vec<Asset>,
		}
	}
);

define_packet!(
	RequestAssets {
		fixed {
			opt assets: Vec<Asset>,
		}
	}
);

// Simple signal packet with no data
define_packet!(
	RequestCommonAssetsRebuild {}
);

define_packet!(
	ServerTags {
		fixed {
			opt tags: HashMap<String, i32>,
		}
	}
);

define_packet!(
	SetTimeDilation {
		time_dilation: f32
	}
);

define_packet!(
	SetUpdateRate {
		updates_per_second: i32
	}
);

define_packet!(
	UpdateFeatures {
		fixed {
			opt features: HashMap<ClientFeature, bool>,
		}
	}
);

define_packet!(
	ViewRadius {
		value: i32
	}
);

// Simple signal packet with no data
define_packet!(WorldLoadFinished {});

define_packet!(
	WorldLoadProgress {
		fixed {
			required percent_complete: i32,
			required percent_complete_subitem: i32,
			opt message: String,
		}
	}
);

define_packet!(
	WorldSettings {
		fixed {
			required world_height: i32,
			opt required_assets: Vec<Asset>
		}
	}
);
