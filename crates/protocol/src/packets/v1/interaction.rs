use std::collections::HashMap;

use bytes::Buf;
use uuid::Uuid;

use crate::{
	codec::{
		BitOptionVec,
		PacketError,
	},
	define_enum,
	define_packet,
	packets::v1::{
		BlockFace,
		BlockPosition,
		BlockRotation,
		DirectionF,
		MovementDirection,
		PositionF,
		SelectedHitEntity,
		Vector3f,
	},
};

define_packet!(
	ForkedChainId {
		fixed {
			required entry_index: i32,
			required sub_index: i32,
			opt forked_id: Box<ForkedChainId> // Indirection so we don't have infinite size errors due to recursive typing
		}
	}
);

define_packet!(
	CancelInteractionChain {
		fixed {
			required chain_id: i32,
			opt forked_id: ForkedChainId
		}
	}
);

// Empty signal packet
define_packet!(DismountNPC {});

define_packet!(MountNPC {
	anchor_x: f32,
	anchor_y: f32,
	anchor_z: f32,
	entity_id: i32
});

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

define_packet!(
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
			opt forked_id: ForkedChainId,
			opt interacted_item_id: String
		}
	}
);

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

define_packet!(
	InteractionSyncData {
		mask_size: 2,
		fixed {
			required state: InteractionState,
			required progress: f32,
			required operation_counter: i32,
			required root_interaction: i32,
			required total_forks: i32,
			required entity_id: i32,
			required entered_root_interaction: i32,
			opt(0) block_position: BlockPosition [pad=12],
			required block_face: BlockFace,
			opt(1) block_rotation: BlockRotation [pad=3],
			required placed_block_id: i32,
			required charge_value: f32,
			required chaining_index: i32,
			required flag_index: i32,
			opt(4) attacker_pos: PositionF [pad=24],
			opt(5) attacker_rot: DirectionF [pad=12],
			opt(6) raycast_hit: PositionF [pad=24],
			required raycast_distance: f32,
			opt(7) raycast_normal: Vector3f [pad=12],
			required movement_direction: MovementDirection,
			required apply_force_state: ApplyForceState,
			required next_label: i32,
			opt(8) generated_uuid: Uuid [pad=16],
		}
		variable {
			opt(2) fork_counts: HashMap<InteractionType, i32>,
			opt(3) hit_entities: Vec<SelectedHitEntity>,
		}
	}
);

define_packet! {
	InteractionChainData {
		fixed {
			required entity_id: i32,
			required proxy_id: Uuid,
			opt(0) hit_location: Vector3f [pad=12],
			opt(2) block_position: BlockPosition [pad=12],
			required target_slot: i32,
			opt(3) hit_normal: Vector3f [pad=12],
			opt(1) hit_detail: String
		}
	}
}

define_packet!(
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
			opt item_in_hand_id: String,
			opt utility_item_id: String,
			opt tools_item_id: String,
			opt forked_id: ForkedChainId,
			opt data: Box<InteractionChainData>, // Boxed to reduce enum variant size by ~100 bytes
			opt new_forks: Vec<SyncInteractionChain>,
			opt interaction_data: BitOptionVec<InteractionSyncData>,
		}
	}
);

define_packet!(
	SyncInteractionChains {
		updates: Vec<SyncInteractionChain>,
	}
);
