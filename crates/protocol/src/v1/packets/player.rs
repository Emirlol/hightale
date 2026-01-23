use macros::define_packet;
use uuid::Uuid;

use crate::v1::{
	BlockRotation,
	DamageCause,
	DebugShape,
	DirectionF,
	GameMode,
	HalfFloatPosition,
	ModelTransform,
	MouseButtonEvent,
	MouseMotionEvent,
	MovementSettings,
	MovementStates,
	PickupLocation,
	PositionF,
	SavedMovementStates,
	TeleportAck,
	Vector2f,
	Vector3d,
	Vector3f,
	Vector3i,
	WorldInteraction,
};

define_packet! { ClearDebugShapes }

define_packet! {
	ClientMovement {
		fixed {
			opt(0, 1) movement_states: MovementStates,
			opt(0, 2) relative_position: HalfFloatPosition,
			opt(0, 4) absolute_position: PositionF,
			opt(0, 8) body_orientation: DirectionF,
			opt(0, 16) look_orientation: DirectionF,
			opt(0, 32) teleport_ack: TeleportAck,
			opt(0, 64) wish_movement: PositionF,
			opt(0, 128) velocity: Vector3d,
			required mounted_to: i32,
			opt(1, 1) rider_movement_states: MovementStates,
		}
	}
}

define_packet! {
	ClientPlaceBlock {
		fixed {
			opt(1) position: Vector3i,
			opt(2) rotation: BlockRotation,
			required placed_block_id: u8,
		}
	}
}

define_packet! { ClientReady {
	ready_for_chunks: bool,
	ready_for_gameplay: bool,
} }

define_packet! {
	ClientTeleport {
		fixed {
			required teleport_id: u8,
			opt(1) model_transform: ModelTransform,
			required reset_velocity: bool,
		}
	}
}

define_packet! {
	DamageInfo {
		fixed {
			opt(1) damage_source_position: Vector3d,
			required damage_amount: f32,
		}
		variable {
			opt(2) damage_cause: DamageCause
		}
	}
}

define_packet! {
	DisplayDebug {
		fixed {
			required shape: DebugShape,
			opt(2) color: Vector3f,
			required time: f32,
			required fade: bool
		}
		variable {
			opt(1) matrix: Vec<f32>,
			opt(4) frustum_projection: Vec<f32>,
		}
	}
}

define_packet! { JoinWorld {
	clear_world: bool,
	fade_in_out: bool,
	world_uuid: Uuid
} }

define_packet! { LoadHotbar { inventory_row: u8 } }

define_packet! {
	MouseInteraction {
		fixed {
			required client_timestamp: i64,
			required active_slot: i32,
			opt(2) screen_point: Vector2f,
			opt(4) mouse_button: MouseButtonEvent,
			opt(16) world_interaction: WorldInteraction,
		}
		variable {
			opt(1) item_in_hand_id: String,
			opt(8) mouse_motion: MouseMotionEvent,
		}
	}
}

define_packet! {
	RemoveMapMarker {
		variable {
			opt(1) marker_id: String
		}
	}
}

define_packet! { ReticleEvent { event_index: i32 } }

define_packet! { SaveHotbar { inventory_row: u8 } }

define_packet! { SetBlockPlacementOverride { enabled: bool } }

define_packet! { SetClientId { client_id: i32 } }

define_packet! { SetGameMode { game_mode: GameMode } }

define_packet! {
	SetMovementStates {
		fixed {
			opt(1) movement_states: SavedMovementStates,
		}
	}
}

define_packet! { SyncPlayerPreferences {
	show_entity_markers: bool,
	armor_items_preferred_pickup_location: PickupLocation,
	weapon_and_tool_items_preferred_pickup_location: PickupLocation,
	usable_items_items_preferred_pickup_location: PickupLocation,
	solid_block_items_preferred_pickup_location: PickupLocation,
	misc_items_preferred_pickup_location: PickupLocation,
	allow_npc_detection: bool,
	respond_to_hit: bool,
} }

define_packet! { UpdateMemoriesFeatureStatus { is_feature_unlock: bool } }

define_packet! {
	UpdateMovementSettings {
		movement_settings: Box<MovementSettings> // Boxed to reduce packet enum variant size, this is like 250 bytes otherwise
	}
}
