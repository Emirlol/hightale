use std::collections::HashMap;

use bytes::Buf;
use protocol_macros::define_packet;
use uuid::Uuid;

use crate::{
	codec::BitOptionVec,
	define_enum,
	v2::{
		BlockFace,
		BlockRotation,
		DirectionF,
		MovementDirection,
		PositionF,
		SelectedHitEntity,
		Vector3f,
		Vector3i,
	},
};

define_packet! {
	ForkedChainId {
		fixed {
			required entry_index: i32,
			required sub_index: i32,
		}
		variable {
			opt(1) forked_id: Box<ForkedChainId> // Indirection so we don't have infinite size errors due to recursive typing
		}
	}
}

define_packet! {
	CancelInteractionChain {
		fixed {
			required chain_id: i32,
		}
		variable {
			opt(1) forked_id: ForkedChainId
		}
	}
}

// Empty signal packet
define_packet! { DismountNPC }

define_packet! { MountNPC {
	anchor_pos: Vector3f,
	entity_id: i32
} }

define_enum! {
	pub enum InteractionType {
		Primary = 0,
		Secondary = 1,
		Ability1 = 2,
		Ability2 = 3,
		Ability3 = 4,
		Use = 5,
		Pick = 6,
		Pickup = 7,
		CollisionEnter = 8,
		CollisionLeave = 9,
		Collision = 10,
		EntityStatEffect = 11,
		SwapTo = 12,
		SwapFrom = 13,
		Death = 14,
		Wielding = 15,
		ProjectileSpawn = 16,
		ProjectileHit = 17,
		ProjectileMiss = 18,
		ProjectileBounce = 19,
		Held = 20,
		HeldOffhand = 21,
		Equipped = 22,
		Dodge = 23,
		GameModeSwap = 24,
	}
}

define_packet! {
	PlayInteractionFor {
		fixed {
			required entity_id: i32,
			required chain_id: i32,
			required operation_index: i32,
			required interaction_id: i32,
			required interaction_type: InteractionType,
			required cancel: bool,
		}
		variable {
			opt(1) forked_id: ForkedChainId,
			opt(2) interacted_item_id: String
		}
	}
}

define_enum! {
	pub enum InteractionState {
		Finished = 0,
		Skip = 1,
		ItemChanged = 2,
		Failed = 3,
		NotFinished = 4,
	}
}

define_enum! {
	pub enum ApplyForceState {
		Waiting = 0,
		Ground = 1,
		Collision = 2,
		Timer = 3,
	}
}

define_packet! {
	InteractionSyncData {
		fixed {
			required state: InteractionState,
			required progress: f32,
			required operation_counter: i32,
			required root_interaction: i32,
			required total_forks: i32,
			required entity_id: i32,
			required entered_root_interaction: i32,
			opt(0, 1) block_position: Vector3i,
			required block_face: BlockFace,
			opt(0, 2) block_rotation: BlockRotation,
			required placed_block_id: i32,
			required charge_value: f32,
			required chaining_index: i32,
			required flag_index: i32,
			opt(0, 4) attacker_pos: PositionF,
			opt(0, 8) attacker_rot: DirectionF,
			opt(0, 16) raycast_hit: PositionF,
			required raycast_distance: f32,
			opt(0, 32) raycast_normal: Vector3f,
			required movement_direction: MovementDirection,
			required apply_force_state: ApplyForceState,
			required next_label: i32,
			opt(0, 64) generated_uuid: Uuid,
		}
		variable {
			opt(0, 128) fork_counts: HashMap<InteractionType, i32>,
			opt(1, 1) hit_entities: Vec<SelectedHitEntity>,
		}
	}
}

define_packet! {
	InteractionChainData {
		fixed {
			required entity_id: i32,
			required proxy_id: Uuid,
			opt(1) hit_location: Vector3f,
			opt(2) block_position: Vector3i,
			required target_slot: i32,
			opt(4) hit_normal: Vector3f,
		}
		variable {
			opt(8) hit_detail: String
		}
	}
}

define_packet! {
	SyncInteractionChain {
		fixed {
			required active_hotbar_slot: i32,
			required active_utility_slot: i32,
			required active_tools_slot: i32,
			required initial: bool,
			required desync: bool,
			required override_root_interaction: i32,
			required interaction_type: InteractionType,
			required equip_slot: i32,
			required chain_id: i32,
			required state: InteractionState,
			required operation_base_index: i32,
		}
		variable {
			opt(1) item_in_hand_id: String,
			opt(2) utility_item_id: String,
			opt(4) tools_item_id: String,
			opt(8) forked_id: ForkedChainId,
			opt(16) data: Box<InteractionChainData>, // Boxed to reduce enum variant size by ~100 bytes
			opt(32) new_forks: Vec<SyncInteractionChain>,
			opt(64) interaction_data: BitOptionVec<InteractionSyncData>,
		}
	}
}

define_packet! {
	SyncInteractionChains {
		variable {
			required updates: Vec<SyncInteractionChain>,
		}
	}
}
