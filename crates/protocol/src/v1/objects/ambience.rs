use super::{
	RangeB,
	RangeF,
	RangeI,
};
use crate::{
	define_enum,
	define_packet,
};

define_packet! {
	AmbienceFXBlockSoundSet {
		fixed {
			required block_sound_set_index: i32,
			opt percent: RangeF,
		}
	}
}

define_packet! {
	AmbienceFXConditions {
		mask_size: 2
		fixed {
			required never: bool,
			required environment_tag_pattern_index: i32,
			required weather_tag_pattern_index: i32,
			opt(4) altitude: RangeI [pad=8],
			opt(5) walls: RangeB [pad=2],
			required roof: bool,
			required roof_material_tag_pattern_index: i32,
			required floor: bool,
			opt(6) sun_light_level: RangeB [pad=2],
			opt(7) torch_light_level: RangeB [pad=2],
			opt(8) global_light_level: RangeB [pad=2],
			opt(9) day_time: RangeF [pad=8],
		}
		variable {
			opt(0) environment_indices: Vec<i32>,
			opt(1) weather_indices: Vec<i32>,
			opt(2) fluid_fx_indices: Vec<i32>,
			opt(3) surrounding_block_sound_sets: Vec<AmbienceFXBlockSoundSet>
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
			opt frequency: RangeF [pad=8],
			opt volume: RangeI [pad=8],
		}
	}
}

define_packet! {
	AmbienceFXMusic {
		fixed {
			required volume: f32,
			opt tracks: Vec<String>,
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
			opt track: String,
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
			opt(5) sound_effect: AmbienceFXSoundEffect [pad=9],
			required priority: i32,
			required audio_category_index: i32,
		}
		variable {
			opt(0) id: String,
			opt(1) conditions: AmbienceFXConditions,
			opt(2) sounds: Vec<AmbienceFXSound>,
			opt(3) music: AmbienceFXMusic,
			opt(4) ambient_bed: AmbienceFXAmbientBed,
			opt(6) blocked_ambience_fx_indices: Vec<i32>,
		}
	}
}
