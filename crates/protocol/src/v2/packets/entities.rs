use macros::define_packet;
use crate::{
	define_enum,
	v2::{
		DirectionF,
		EntityUpdate,
		ModelParticle,
		MovementStates,
		PositionF,
		Vector3f,
	},
};

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

define_packet! { VelocityConfig {
	ground_resistance: f32,
	ground_resistance_max: f32,
	air_resistance: f32,
	air_resistance_max: f32,
	threshold: f32,
	style: VelocityThresholdStyle
} }

define_packet! {
	ApplyKnockback {
		fixed {
			opt(1) hit_position: PositionF,
			required pos: Vector3f,
			required change_type: ChangeVelocityType
		}
	}
}

define_packet! {
	ChangeVelocity {
		fixed {
			required velocity: Vector3f,
			required change_type: ChangeVelocityType,
			opt(1) config: VelocityConfig
		}
	}
}

define_packet! {
	EntityUpdates {
		variable {
			opt(1) removed: Vec<i32>,
			opt(2) updates: Vec<EntityUpdate>
		}
	}
}

define_packet! {
	MountMovement {
		fixed {
			opt(1) absolute_position: PositionF,
			opt(2) body_orientation: DirectionF,
			opt(4) movement_states: MovementStates,
		}
	}
}

define_enum! {
	pub enum AnimationSlot {
		Movement = 0,
		Status = 1,
		Action = 2,
		Face = 3,
		Emote = 4,
	}
}

define_packet! {
	PlayAnimation {
		fixed {
			required entity_id: i32,
			required slot: AnimationSlot,
		}
		variable {
			opt(1) item_animations_id: String,
			opt(2) animation_id: String
		}
	}
}

define_packet! { SetEntitySeed { entity_seed: i32 } }

define_packet! {
	SpawnModelParticles {
		fixed {
			required entity_id: i32,
		}
		variable {
			opt(1) model_particles: Vec<ModelParticle>
		}
	}
}
