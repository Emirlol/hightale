use crate::{define_enum, define_packet};
use crate::packets::v1::{DirectionF, EntityUpdate, ModelParticle, MovementStates, PositionF};

define_enum! {
	pub enum ChangeVelocityType {
		Add = 0,
		Set = 1
	}
}

define_enum! {
	pub enum VelocityThresholdStyle {
		Linear = 0,
		Exp = 1
	}
}

define_packet!(
	VelocityConfig {
		ground_resistance: f32,
		ground_resistance_max: f32,
		air_resistance: f32,
		air_resistance_max: f32,
		threshold: f32,
		style: VelocityThresholdStyle
	}
);

define_packet!(
	ApplyKnockback {
		fixed {
			opt hit_position: PositionF [pad=24],
			required x: f32,
			required y: f32,
			required z: f32,
			required change_type: ChangeVelocityType
		}
	}
);

define_packet!(
	ChangeVelocity {
		fixed {
			required x: f32,
			required y: f32,
			required z: f32,
			required change_type: ChangeVelocityType,
			opt config: VelocityConfig
		}
	}
);

define_packet!(
	EntityUpdates {
		variable {
			opt removed: Vec<i32>,
			opt updates: Vec<EntityUpdate>
		}
	}
);

define_packet!(
	MountMovement {
		fixed {
			opt absolute_position: PositionF [pad=24],
			opt body_orientation: DirectionF [pad=12],
			opt movement_states: MovementStates [pad=22],
		}
	}
);

define_enum! {
	pub enum AnimationSlot {
		Movement = 0,
		Status = 1,
		Action = 2,
		Face = 3,
		Emote = 4,
	}
}

define_packet!(
	PlayAnimation {
		fixed {
			required entity_id: i32,
			required slot: AnimationSlot,
		}
		variable {
			opt item_animations_id: String,
			opt animation_id: String
		}
	}
);

define_packet!(
	SetEntitySeed {
		entity_seed: i32
	}
);

define_packet!(
	SpawnModelParticles {
		fixed {
			required entity_id: i32,
		}
		variable {
			opt model_particles: Vec<ModelParticle>
		}
	}
);