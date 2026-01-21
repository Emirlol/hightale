use crate::{
	define_enum,
	define_packet,
	v1::{
		ApplyLookType,
		ApplyMovementType,
		AttachedToType,
		CanMoveType,
		DirectionF,
		MouseInputTargetType,
		MouseInputType,
		MovementForceRotationType,
		PositionDistanceOffsetType,
		PositionF,
		PositionType,
		RotationType,
		Vector2f,
		Vector3f,
	},
};

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

define_packet! { CameraShakeEffect {
	camera_shake_id: i32,
	intensity: f32,
	mode: AccumulationMode
} }

define_packet! { RequestFlyCameraMode { entering: bool } }

define_packet! { SetFlyCameraMode { entering: bool } }

define_packet! {
	ServerCameraSettings {
		fixed {
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
			opt(0) movement_force_rotation: DirectionF [pad=12],
			required attached_to_type: AttachedToType,
			required attached_to_entity_id: i32,
			required eye_offset: bool,
			required position_distance_offset_type: PositionDistanceOffsetType,
			opt(1) position_offset: PositionF [pad=24],
			opt(2) rotation_offset: DirectionF [pad=12],
			required position_type: PositionType,
			opt(3) position: PositionF [pad=24],
			required rotation_type: RotationType,
			opt(4) rotation: DirectionF [pad=12],
			required can_move_type: CanMoveType,
			required apply_movement_type: ApplyMovementType,
			opt(5) movement_multiplier: Vector3f [pad=12],
			required apply_look_type: ApplyLookType,
			opt(6) look_multiplier: Vector2f [pad=8],
			required mouse_input_type: MouseInputType,
			opt(7) plane_normal: Vector3f [pad=12],
		}
	}
}

define_packet! {
	SetServerCamera {
		fixed {
			required client_camera_view: ClientCameraView,
			required is_locked: bool,
			opt camera_settings: Box<ServerCameraSettings>, // Boxed to avoid large packet enum size
		}
	}
}
