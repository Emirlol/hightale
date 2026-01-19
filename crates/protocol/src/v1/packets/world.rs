use bytes::Bytes;
use uuid::Uuid;

use super::{
	BlockParticleEvent,
	Color,
	DirectionF,
	InstantData,
	PositionF,
	SoundCategory,
	Vector3i,
};
use crate::{
	define_enum,
	define_packet,
};

define_packet! { ClearEditorTimeOverride {} }
define_enum! {
	pub enum PaletteType {
		Empty = 0,
		HalfByte = 1,
		Byte = 2,
		Short = 3,
	}
}
define_packet! { PlaySoundEvent2D {
	sound_event_index: i32,
	category: SoundCategory,
	volume_modifier: f32,
	pitch_modifier: f32
} }
define_packet! { PlaySoundEvent3D {
	fixed {
		required sound_event_index: i32,
		required category: SoundCategory,
		opt position: PositionF [pad=24],
		required volume_modifier: f32,
		required pitch_modifier: f32
	}
} }
define_packet! { PlaySoundEventEntity {
	sound_event_index: i32,
	network_id: i32,
	volume_modifier: f32,
	pitch_modifier: f32
} }
define_enum! {
	pub enum RotationAxis {
		X = 0,
		Y = 1,
		Z = 2,
	}
}
define_enum! {
	pub enum RotationDirection {
		Positive = 0,
		Negative = 1,
	}
}
define_packet! { ServerSetBlock {
	pos: Vector3i,
	block_id: i32,
	filler: i16,
	rotation: u8
} }
define_packet! { ServerSetBlocks {
	pos: Vector3i,
	cmds: Vec<SetBlockCmd>
} }
define_packet! { ServerSetFluid {
	pos: Vector3i,
	fluid_id: i32,
	fluid_level: u8,
} }
define_packet! { ServerSetFluids {
	pos: Vector3i,
	cmds: Vec<SetFluidCmd>
} }
define_packet! { ServerSetPaused { paused: bool } }
define_packet! { SetBlockCmd {
	index: i16,
	block_id: i32,
	filler: i16,
	rotation: u8
} }
define_packet! { SetChunk {
	fixed {
		required pos: Vector3i,
	}
	variable {
		opt local_light: Bytes,
		opt global_light: Bytes,
		opt data: Bytes,
	}
} }
define_packet! { SetChunkEnvironments {
	fixed {
		required x: i32,
		required z: i32,
		opt environments: Bytes
	}
} }
define_packet! { SetChunkHeightmap {
	fixed {
		required x: i32,
		required z: i32,
		opt heightmap: Bytes
	}
} }
define_packet! { SetChunkTintmap {
	fixed {
		required x: i32,
		required z: i32,
		opt tintmap: Bytes
	}
} }
define_packet! { SetFluidCmd {
	index: i16,
	fluid_id: i32,
	fluid_level: u8
} }
define_packet! { SetFluids {
	fixed {
		required pos: Vector3i,
		opt data: Bytes
	}
} }
define_packet! { SetPaused { paused: bool } }
define_packet! { SleepClock {
	fixed {
		opt start_game_time: InstantData [pad=12],
		opt target_game_time: InstantData [pad=12],
		required progress: f32,
		required duration_seconds: f32
	}
} }
define_packet! { SleepMultiplayer {
	fixed {
		required sleepers_count: i32,
		required awake_count: i32,
		opt awake_sample: Vec<Uuid>
	}
} }
define_packet! {
   SpawnBlockParticleSystem {
	   fixed {
		   required block_id: i32,
		   required particle_type: BlockParticleEvent,
		   opt position: PositionF [pad=24],
	   }
   }
}
define_packet! { SpawnParticleSystem {
	fixed {
		opt position: PositionF [pad=24],
		opt rotation: DirectionF [pad=12],
		required scale: f32,
		opt color: Color [pad=3],
		opt particle_system_id: String
	}
} }
define_packet! { UnloadChunk { x: i32, z: i32 } }
define_packet! { UpdateBlockDamage{
	fixed {
		opt block_position: Vector3i [pad=12],
		required damage: f32,
		required delta: f32
	}
} }
define_packet! { UpdateEditorTimeOverride {
	fixed {
		opt game_time: InstantData [pad=12],
		required paused: bool
	}
} }
define_packet! { UpdateEditorWeatherOverride { weather_index: i32 } }
define_packet! { UpdateEnvironmentMusic { environment_index: i32 } }
define_packet! { UpdatePostFxSettings {
	global_intensity: f32,
	power: f32,
	sunshaft_scale: f32,
	sun_intensity: f32,
	sunshaft_intensity: f32,
} }
define_packet! { UpdateSleepState {
	fixed {
		required gray_fade: bool,
		required sleep_ui: bool,
		opt clock: SleepClock [pad=33],
		opt multiplayer: SleepMultiplayer,
	}
} }
define_packet! { UpdateSunSettings {
	height_percentage: f32,
	angel_radians: f32
} }
define_packet! { UpdateTime {
	fixed {
		opt game_time: InstantData [pad=12],
	}
} }
define_packet! { UpdateTimeSettings {
	daytime_duration_seconds: i32,
	nighttime_duration_seconds: i32,
	total_moon_phases: u8,
	time_paused: bool
} }
define_packet! { UpdateWeather {
	weather_index: i32,
	transition_seconds: f32
} }
