use std::collections::HashMap;

use bytes::{
	Buf,
	BufMut,
	BytesMut,
};
use uuid::Uuid;

use crate::{
	codec::{
		BitOptionVec,
		HytaleCodec,
		PacketError,
		PacketResult,
		VarInt,
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
		bitmask {
			required entry_index: i32,
			required sub_index: i32,
			opt forked_id: Box<ForkedChainId> // Indirection so we don't have infinite size errors due to recursive typing
		}
	}
);

define_packet!(
	CancelInteractionChain {
		bitmask {
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
			entity_id: i32,
			chain_id: i32,
			operation_index: i32,
			interaction_id: i32,
			interaction_type: InteractionType,
			cancel: bool,
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

#[derive(Debug, Clone)]
pub struct InteractionSyncData {
	pub state: InteractionState,
	pub progress: f32,
	pub operation_counter: i32,
	pub root_interaction: i32,
	pub total_forks: i32,
	pub entity_id: i32,
	pub entered_root_interaction: i32,
	pub block_position: Option<BlockPosition>, // Bit 0 (offset 27)
	pub block_face: BlockFace,
	pub block_rotation: Option<BlockRotation>, // Bit 1 (offset 40)
	pub placed_block_id: i32,
	pub charge_value: f32,
	pub chaining_index: i32,
	pub flag_index: i32,
	pub attacker_pos: Option<PositionF>,  // Bit 4 (16) (offset 59)
	pub attacker_rot: Option<DirectionF>, // Bit 5 (32) (offset 83)
	pub raycast_hit: Option<PositionF>,   // Bit 6 (64) (offset 95)
	pub raycast_distance: f32,
	pub raycast_normal: Option<Vector3f>, // Bit 7 (128) (offset 123)
	pub movement_direction: MovementDirection,
	pub apply_force_state: ApplyForceState,
	pub next_label: i32,
	pub generated_uuid: Option<Uuid>, // Bit 8 (Byte 1 bit 0) (offset 141)

	// Variable length fields
	pub fork_counts: Option<HashMap<InteractionType, i32>>, // Bit 2 (4)
	pub hit_entities: Option<Vec<SelectedHitEntity>>,       // Bit 3 (8)
}

impl HytaleCodec for InteractionSyncData {
	fn encode(&self, buf: &mut BytesMut) {
		let mut null_bits_0: u8 = 0;
		let mut null_bits_1: u8 = 0;

		// Calculate mask
		if self.block_position.is_some() {
			null_bits_0 |= 1;
		}
		if self.block_rotation.is_some() {
			null_bits_0 |= 2;
		}
		if self.fork_counts.is_some() {
			null_bits_0 |= 4;
		}
		if self.hit_entities.is_some() {
			null_bits_0 |= 8;
		}
		if self.attacker_pos.is_some() {
			null_bits_0 |= 16;
		}
		if self.attacker_rot.is_some() {
			null_bits_0 |= 32;
		}
		if self.raycast_hit.is_some() {
			null_bits_0 |= 64;
		}
		if self.raycast_normal.is_some() {
			null_bits_0 |= 128;
		}
		if self.generated_uuid.is_some() {
			null_bits_1 |= 1;
		}

		buf.put_u8(null_bits_0);
		buf.put_u8(null_bits_1);

		// Offset 2
		self.state.encode(buf);
		// Offset 3
		buf.put_f32_le(self.progress);
		// Offset 7
		buf.put_i32_le(self.operation_counter);
		// Offset 11
		buf.put_i32_le(self.root_interaction);
		// Offset 15
		buf.put_i32_le(self.total_forks);
		// Offset 19
		buf.put_i32_le(self.entity_id);
		// Offset 23
		buf.put_i32_le(self.entered_root_interaction);

		if let Some(bp) = &self.block_position {
			bp.encode(buf);
		} else {
			buf.put_bytes(0, 12);
		}

		// Offset 39
		self.block_face.encode(buf);

		// Offset 40: BlockRotation (Optional, 3 bytes padding)
		if let Some(br) = &self.block_rotation {
			br.encode(buf);
		} else {
			buf.put_bytes(0, 3);
		}

		// Offset 43
		buf.put_i32_le(self.placed_block_id);
		// Offset 47
		buf.put_f32_le(self.charge_value);
		// Offset 51
		buf.put_i32_le(self.chaining_index);
		// Offset 55
		buf.put_i32_le(self.flag_index);

		// Offset 59: AttackerPos (Optional, 24 bytes padding)
		if let Some(ap) = &self.attacker_pos {
			ap.encode(buf);
		} else {
			buf.put_bytes(0, 24);
		}

		// Offset 83: AttackerRot (Optional, 12 bytes padding)
		if let Some(ar) = &self.attacker_rot {
			ar.encode(buf);
		} else {
			buf.put_bytes(0, 12);
		}

		// Offset 95: RaycastHit (Optional, 24 bytes padding)
		if let Some(rh) = &self.raycast_hit {
			rh.encode(buf);
		} else {
			buf.put_bytes(0, 24);
		}

		// Offset 119
		buf.put_f32_le(self.raycast_distance);

		// Offset 123: RaycastNormal (Optional, 12 bytes padding)
		if let Some(rn) = &self.raycast_normal {
			rn.encode(buf);
		} else {
			buf.put_bytes(0, 12);
		}

		// Offset 135
		self.movement_direction.encode(buf);
		// Offset 136
		self.apply_force_state.encode(buf);
		// Offset 137
		buf.put_i32_le(self.next_label);

		// Offset 141: GeneratedUUID (Optional, 16 bytes padding)
		if let Some(uuid) = &self.generated_uuid {
			uuid.encode(buf);
		} else {
			buf.put_bytes(0, 16);
		}

		// Offset 157: ForkCounts offset
		let fork_offset_idx = buf.len();
		buf.put_i32_le(0);

		// Offset 161: HitEntities offset
		let hit_offset_idx = buf.len();
		buf.put_i32_le(0);

		// End of fixed header = 165
		let start_of_header = 0;
		let var_base_pos = buf.len();

		// 4. Write Variable Body
		if let Some(map) = &self.fork_counts {
			let curr = buf.len();
			let rel = (curr - var_base_pos) as i32;
			// Patch offset
			let slice = &mut buf[fork_offset_idx..fork_offset_idx + 4];
			slice.copy_from_slice(&rel.to_le_bytes());

			VarInt(map.len() as i32).encode(buf);
			for (k, v) in map {
				k.encode(buf);
				buf.put_i32_le(*v);
			}
		}

		if let Some(list) = &self.hit_entities {
			let curr = buf.len();
			let rel = (curr - var_base_pos) as i32;
			let slice = &mut buf[hit_offset_idx..hit_offset_idx + 4];
			slice.copy_from_slice(&rel.to_le_bytes());

			VarInt(list.len() as i32).encode(buf);
			for item in list {
				item.encode(buf);
			}
		}
	}

	fn decode(src: &mut impl Buf) -> PacketResult<Self> {
		let mut buf = std::io::Cursor::new(src.copy_to_bytes(src.remaining()));
		let start_pos = buf.position();

		if buf.remaining() < 165 {
			return Err(PacketError::Incomplete);
		}

		let null_bits_0 = buf.get_u8();
		let null_bits_1 = buf.get_u8();

		// Offset 2
		let state = InteractionState::decode(&mut buf)?;
		let progress = buf.get_f32_le();
		let operation_counter = buf.get_i32_le();
		let root_interaction = buf.get_i32_le();
		let total_forks = buf.get_i32_le();
		let entity_id = buf.get_i32_le();
		let entered_root_interaction = buf.get_i32_le();

		// Offset 27
		let block_position = if (null_bits_0 & 1) != 0 {
			Some(BlockPosition::decode(&mut buf)?)
		} else {
			buf.advance(12);
			None
		};

		// Offset 39
		let block_face = BlockFace::decode(&mut buf)?;

		// Offset 40
		let block_rotation = if (null_bits_0 & 2) != 0 {
			Some(BlockRotation::decode(&mut buf)?)
		} else {
			buf.advance(3);
			None
		};

		// Offset 43
		let placed_block_id = buf.get_i32_le();
		let charge_value = buf.get_f32_le();
		let chaining_index = buf.get_i32_le();
		let flag_index = buf.get_i32_le();

		// Offset 59
		let attacker_pos = if (null_bits_0 & 16) != 0 {
			Some(PositionF::decode(&mut buf)?)
		} else {
			buf.advance(24);
			None
		};

		// Offset 83
		let attacker_rot = if (null_bits_0 & 32) != 0 {
			Some(DirectionF::decode(&mut buf)?)
		} else {
			buf.advance(12);
			None
		};

		// Offset 95
		let raycast_hit = if (null_bits_0 & 64) != 0 {
			Some(PositionF::decode(&mut buf)?)
		} else {
			buf.advance(24);
			None
		};

		// Offset 119
		let raycast_distance = buf.get_f32_le();

		// Offset 123
		let raycast_normal = if (null_bits_0 & 128) != 0 {
			Some(Vector3f::decode(&mut buf)?)
		} else {
			buf.advance(12);
			None
		};

		// Offset 135
		let movement_direction = MovementDirection::decode(&mut buf)?;
		let apply_force_state = ApplyForceState::decode(&mut buf)?;
		let next_label = buf.get_i32_le();

		// Offset 141
		let generated_uuid = if (null_bits_1 & 1) != 0 {
			Some(Uuid::decode(&mut buf)?)
		} else {
			buf.advance(16);
			None
		};

		buf.set_position(start_pos + 157);

		let fork_counts_offset = buf.get_i32_le();
		let hit_entities_offset = buf.get_i32_le();

		let var_base = start_pos + 165;

		let fork_counts = if (null_bits_0 & 4) != 0 {
			// Java: offset + 165 + offsetValue
			buf.set_position(var_base + fork_counts_offset as u64);

			let count = VarInt::decode(&mut buf)?.0;
			let mut map = HashMap::with_capacity(count as usize);
			for _ in 0..count {
				let k = InteractionType::decode(&mut buf)?;
				let v = buf.get_i32_le();
				map.insert(k, v);
			}
			Some(map)
		} else {
			None
		};

		let hit_entities = if (null_bits_0 & 8) != 0 {
			buf.set_position(var_base + hit_entities_offset as u64);

			let count = VarInt::decode(&mut buf)?.0;
			let mut list = Vec::with_capacity(count as usize);
			for _ in 0..count {
				list.push(SelectedHitEntity::decode(&mut buf)?);
			}
			Some(list)
		} else {
			None
		};

		Ok(Self {
			state,
			progress,
			operation_counter,
			root_interaction,
			total_forks,
			entity_id,
			entered_root_interaction,
			block_position,
			block_face,
			block_rotation,
			placed_block_id,
			charge_value,
			chaining_index,
			flag_index,
			attacker_pos,
			attacker_rot,
			raycast_hit,
			raycast_distance,
			raycast_normal,
			movement_direction,
			apply_force_state,
			next_label,
			generated_uuid,
			fork_counts,
			hit_entities,
		})
	}
}

define_packet! {
	InteractionChainData {
		bitmask {
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
			active_hotbar_slot: i32,
			active_utility_slot: i32,
			active_tools_slot: i32,
			initial: bool,
			desync: bool,
			override_root_interaction: i32,
			interaction_type: InteractionType,
			equip_slot: i32,
			chain_id: i32,
			state: InteractionState,
			operation_base_index: i32,
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
