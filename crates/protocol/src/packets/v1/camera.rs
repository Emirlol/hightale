use crate::{
	define_enum,
	define_packet,
	packets::v1::{
		DirectionF,
		PositionF,
		Vector2f,
		Vector3f,
	},
};
use crate::packets::v1::{ApplyLookType, ApplyMovementType, AttachedToType, CanMoveType, MouseInputTargetType, MouseInputType, MovementForceRotationType, PositionDistanceOffsetType, PositionType, RotationType};

define_enum! {
	pub enum ClientCameraView {
		FirstPerson = 0,
		ThirdPerson = 1,
		Custom = 2,
	}
}

define_enum! {
	pub enum AccumulationMode {
		Set = 0,
		Sum = 1,
		Average = 2,
	}
}

define_packet!(CameraShakeEffect {
	camera_shake_id: i32,
	intensity: f32,
	mode: AccumulationMode
});

define_packet!(RequestFlyCameraMode { entering: bool });

define_packet!(SetFlyCameraMode { entering: bool });

define_packet!(
	ServerCameraSettings {
		bitmask {
			required position_lerp_speed: f32,
			required rotation_lerp_speed: f32,
			required distance: f32,
			required speed_modifier: f32,
			required allow_pitch_controls: bool,
			required display_cursor: bool,
			required display_reticle: bool,
			required mouse_input_target_type: MouseInputTargetType,
			required send_mouse_motion: bool,
			required skip_character_physics: bool,
			required is_first_person: bool,
			required movement_force_rotation_type: MovementForceRotationType,
			opt movement_force_rotation: DirectionF [pad=12], // Bit 1
			required attached_to_type: AttachedToType,
			required attached_to_entity_id: i32,
			required eye_offset: bool,
			required position_distance_offset_type: PositionDistanceOffsetType,
			opt position_offset: PositionF [pad=24], // Bit 2
			opt rotation_offset: DirectionF [pad=12], // Bit 4
			required position_type: PositionType,
			opt position: PositionF [pad=24], // Bit 8
			required rotation_type: RotationType,
			opt rotation: DirectionF [pad=12], // Bit 16
			required can_move_type: CanMoveType,
			required apply_movement_type: ApplyMovementType,
			opt movement_multiplier: Vector3f [pad=12], // Bit 32
			required apply_look_type: ApplyLookType,
			opt look_multiplier: Vector2f [pad=8], // Bit 64
			required mouse_input_type: MouseInputType,
			opt plane_normal: Vector3f [pad=12], // Bit 128
		}
	}
);

define_packet!(
	SetServerCamera {
		bitmask {
			required client_camera_view: ClientCameraView,
			required is_locked: bool,
			opt camera_settings: Box<ServerCameraSettings>, // Boxed to avoid large packet enum size
		}
	}
);
