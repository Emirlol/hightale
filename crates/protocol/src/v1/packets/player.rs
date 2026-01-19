use uuid::Uuid;

use super::{
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
use crate::define_packet;

// Empty signal packet
define_packet! { ClearDebugShapes {} }

define_packet! {
   ClientMovement {
	   mask_size: 2
	   fixed {
		   opt movement_states: MovementStates [pad=22],
		   opt relative_position: HalfFloatPosition [pad=6],
		   opt absolute_position: PositionF [pad=24],
		   opt body_orientation: DirectionF [pad=12],
		   opt look_orientation: DirectionF [pad=12],
		   opt teleport_ack: TeleportAck [pad=1],
		   opt wish_movement: PositionF [pad=24],
		   opt velocity: Vector3d [pad=24],
		   required mounted_to: i32,
		   opt rider_movement_states: MovementStates [pad=22],
	   }
   }
}

define_packet! {
   ClientPlaceBlock {
	   fixed {
		   opt position: Vector3i [pad=12],
		   opt rotation: BlockRotation [pad=3],
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
		   opt model_transform: ModelTransform [pad=49],
		   required reset_velocity: bool,
	   }
   }
}

define_packet! {
   DamageInfo {
	   fixed {
		   opt damage_source_position: Vector3d [pad=24],
		   required damage_amount: f32,
		   opt damage_cause: DamageCause
	   }
   }
}

define_packet! {
   DisplayDebug {
	   fixed {
		   required shape: DebugShape,
		   opt(1) color: Vector3f [pad=12],
		   required time: f32,
		   required fade: bool
	   }
	   variable {
		   opt(0) matrix: Vec<f32>,
		   opt(2) frustum_projection: Vec<f32>,
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
		   opt(1) screen_point: Vector2f [pad=8],
		   opt(2) mouse_button: MouseButtonEvent [pad=3],
		   opt(4) world_interaction: WorldInteraction [pad=20], // 1 null + 19 data
	   }
	   variable {
		   opt(0) item_in_hand_id: String,
		   opt(3) mouse_motion: MouseMotionEvent,
	   }
   }
}

define_packet! {
   RemoveMapMarker {
	   fixed {
		   opt marker_id: String
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
		   opt movement_states: SavedMovementStates,
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
