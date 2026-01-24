use std::collections::HashMap;

use bytes::Bytes;
use macros::define_packet;

use crate::{
	define_enum,
	v2::Asset,
};

// Simple signal packet with no data
define_packet! { AssetFinalize }

define_packet! {
	AssetInitialize {
		fixed {
			required size: i32,
		}
		variable {
			required asset: Asset
		}
	}
}

define_packet! {
	AssetPart {
		variable {
			opt(1) part: Bytes
		}
	}
}

define_enum! {
	pub enum ClientFeature {
		SplitVelocity = 0,
		Mantling = 1,
		SprintForce = 2,
		CrouchSlide = 3,
		SafetyRoll= 4,
		DisplayHealthBars = 5,
		DisplayCombatText = 6,
		CanHideHelmet = 7,
		CanHideCuirass = 8,
		CanHideGauntlets = 9,
		CanHidePants = 10,
	}
}

define_packet! {
	PlayerSkin {
		variable {
			opt(0, 1) body_characteristic: String,
			opt(0, 2) underwear: String,
			opt(0, 4) face: String,
			opt(0, 8) eyes: String,
			opt(0, 16) ears: String,
			opt(0, 32) mouth: String,
			opt(0, 64) facial_hair: String,
			opt(0, 128) haircut: String,
			opt(1, 1) eyebrows: String,
			opt(1, 2) pants: String,
			opt(1, 4) overpants: String,
			opt(1, 8) undertop: String,
			opt(1, 16) overtop: String,
			opt(1, 32) shoes: String,
			opt(1, 64) head_accessory: String,
			opt(1, 128) face_accessory: String,
			opt(2, 1) ear_accessory: String,
			opt(2, 2) skin_feature: String,
			opt(2, 4) gloves: String,
			opt(2, 8) cape: String,
		}
	}
}

define_packet! {
	PlayerOptions {
		variable {
			opt(1) player_skin: Box<PlayerSkin>, // Box it since it's like 480 bytes and it bloats the largest packet size in the packet enum
		}
	}
}
define_packet! {
	RemoveAssets {
		variable {
			opt(1) assets: Vec<Asset>,
		}
	}
}
define_packet! {
	RequestAssets {
		variable {
			opt(1) assets: Vec<Asset>,
		}
	}
}
// Simple signal packet with no data
define_packet! { RequestCommonAssetsRebuild }

define_packet! {
	ServerTags {
		variable {
			opt(1) tags: HashMap<String, i32>,
		}
	}
}
define_packet! { SetTimeDilation { time_dilation: f32 } }

define_packet! { SetUpdateRate { updates_per_second: i32 } }

define_packet! {
	UpdateFeatures {
		variable {
			opt(1) features: HashMap<ClientFeature, bool>,
		}
	}
}
define_packet! { ViewRadius { value: i32 } }

// Simple signal packet with no data
define_packet! { WorldLoadFinished {} }

define_packet! {
	WorldLoadProgress {
		fixed {
			required percent_complete: i32,
			required percent_complete_subitem: i32,
		}
		variable {
			opt(1) message: String,
		}
	}
}
define_packet! {
	WorldSettings {
		fixed {
			required world_height: i32,
		}
		variable {
			opt(1) required_assets: Vec<Asset>
		}
	}
}
