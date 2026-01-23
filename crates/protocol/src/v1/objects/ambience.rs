use macros::define_packet;
use super::{
	RangeB,
	RangeF,
	RangeI,
};
use crate::{
	define_enum,
};

define_packet! {
	AmbienceFXBlockSoundSet {
		fixed {
			required block_sound_set_index: i32,
			opt(1) percent: RangeF,
		}
	}
}

define_packet! {
	AmbienceFXConditions {
		fixed {
			required never: bool,
			required environment_tag_pattern_index: i32,
			required weather_tag_pattern_index: i32,
			opt(0, 16) altitude: RangeI,
			opt(0, 32) walls: RangeB,
			required roof: bool,
			required roof_material_tag_pattern_index: i32,
			required floor: bool,
			opt(0, 64) sun_light_level: RangeB,
			opt(0, 128) torch_light_level: RangeB,
			opt(1, 1) global_light_level: RangeB,
			opt(1, 2) day_time: RangeF,
		}
		variable {
			opt(0, 1) environment_indices: Vec<i32>,
			opt(0, 2) weather_indices: Vec<i32>,
			opt(0, 4) fluid_fx_indices: Vec<i32>,
			opt(0, 8) surrounding_block_sound_sets: Vec<AmbienceFXBlockSoundSet>
		}
	}
}

define_enum! {
	pub enum AmbienceFXSoundPlay3D {
		Random = 0,
		LocationName = 1,
		No = 2
	}
}

define_enum! {
	pub enum AmbienceFXAltitude {
		Normal = 0,
		Lowest = 1,
		Highest = 2,
		Random = 3
	}
}

define_packet! {
	AmbienceFXSound {
		fixed {
			required sound_event_index: i32,
			required play_3d: AmbienceFXSoundPlay3D,
			required block_sound_set_index: i32,
			required altitude: AmbienceFXAltitude,
			opt(1) frequency: RangeF,
			opt(2) volume: RangeI,
		}
	}
}

define_packet! {
	AmbienceFXMusic {
		fixed {
			required volume: f32,
		}
		variable {
			opt(1) tracks: Vec<String>,
		}
	}
}

define_enum! {
	pub enum AmbienceTransitionSpeed {
		Default = 0,
		Fast = 1,
		Instant = 2
	}
}

define_packet! {
	AmbienceFXAmbientBed {
		fixed {
			required volume: f32,
			required transition_speed: AmbienceTransitionSpeed,
		}
		variable {
			opt(1) track: String,
		}
	}
}

define_packet! {
	AmbienceFXSoundEffect {
		reverb_effect_index: i32,
		equalizer_effect_index: i32,
		is_instant: bool,
	}
}

define_packet! {
	AmbienceFX {
		fixed {
			opt(32) sound_effect: AmbienceFXSoundEffect,
			required priority: i32,
			required audio_category_index: i32,
		}
		variable {
			opt(1) id: String,
			opt(2) conditions: AmbienceFXConditions,
			opt(4) sounds: Vec<AmbienceFXSound>,
			opt(8) music: AmbienceFXMusic,
			opt(16) ambient_bed: AmbienceFXAmbientBed,
			opt(64) blocked_ambience_fx_indices: Vec<i32>,
		}
	}
}
