use std::collections::HashMap;

use macros::define_packet;
use ordered_float::OrderedFloat;

use crate::{
	define_enum,
	id_dispatch,
	v2::{
		camera::CameraShakeEffect,
		entities::{
			ChangeVelocityType,
			VelocityConfig,
		},
		interaction::InteractionType,
		BlockFace,
		Color,
		DirectionF,
		GameMode,
		ItemWithAllMetadata,
		Model,
		ModelParticle,
		ModelTrail,
		MovementEffects,
		Selector,
		ValueType,
		Vector3f,
	},
};

define_enum! {
	pub enum WaitForDataFrom {
		Client = 0,
		Server = 1,
		None = 2,
	}
}

define_packet! {
	InteractionEffects {
		fixed {
			required world_sound_event_index: i32,
			required local_sound_event_index: i32,
			required wait_for_animation_to_finish: bool,
			required clear_animation_on_finish: bool,
			required clear_sound_event_on_finish: bool,
			opt(1) camera_shake: CameraShakeEffect,
			opt(2) movement_effects: MovementEffects,
			required start_delay: f32,
		}
		variable {
			opt(4) particles: Vec<ModelParticle>,
			opt(8) first_person_particles: Vec<ModelParticle>,
			opt(16) trails: Vec<ModelTrail>,
			opt(32) item_player_animations_id: String,
			opt(64) item_animation_id: String,
		}
	}
}

define_packet! {
	InteractionSettings {
		allow_skip_on_click: bool
	}
}

define_packet! {
	InteractionRules {
		fixed {
			required blocked_by_bypass_index: i32,
			required blocking_bypass_index: i32,
			required interrupted_by_bypass_index: i32,
			required interrupting_bypass_index: i32,
		}
		variable {
			opt(1) blocked_by: Vec<InteractionType>,
			opt(2) blocking: Vec<InteractionType>,
			opt(4) interrupted_by: Vec<InteractionType>,
			opt(8) interrupting: Vec<InteractionType>,
		}
	}
}

define_packet! {
	InteractionCamera {
		fixed {
			required time: f32,
			opt(1) position: Vector3f,
			opt(2) rotation: DirectionF,
		}
	}
}

define_packet! {
	InteractionCameraSettings {
		variable {
			opt(1) first_person: Vec<InteractionCamera>,
			opt(2) third_person: Vec<InteractionCamera>,
		}
	}
}

define_packet! {
	SimpleBlockInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	SimpleInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	PlaceBlockInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required block_id: i32,
			required remove_item_in_hand: bool,
			required allow_drag_placement: bool
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	BreakBlockInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
			required harvest: bool
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	PickBlockInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	UseBlockInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	UseEntityInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	BuilderToolInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	ModifyInventoryInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			opt(0, 1) required_game_mode: GameMode,
			required adjust_held_item_quantity: i32,
			required adjust_held_item_durability: f64,
		}
		variable {
			opt(0, 2) effects: InteractionEffects,
			opt(0, 4) settings: HashMap<GameMode, InteractionSettings>,
			opt(0, 8) rules: InteractionRules,
			opt(0, 16) tags: Vec<i32>,
			opt(0, 32) camera: InteractionCameraSettings,
			opt(0, 64) item_to_remove: ItemWithAllMetadata,
			opt(0, 128) item_to_add: ItemWithAllMetadata,
			opt(1, 1) broken_item: String
		}
	}
}

define_packet! {
	ChargingDelay {
		min_delay: f32,
		max_delay: f32,
		max_total_delay: f32,
		min_health: f32,
		max_health: f32
	}
}

define_packet! {
	ChargingInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required failed: i32,
			required allow_indefinite_hold: bool,
			required display_progress: bool,
			required cancel_on_other_click: bool,
			required fail_on_damage: bool,
			required mouse_sensitivity_adjustment_target: f32,
			required mouse_sensitivity_adjustment_duration: f32,
			opt(1) charging_delay: ChargingDelay
		}
		variable {
			opt(2) effects: InteractionEffects,
			opt(4) settings: HashMap<GameMode, InteractionSettings>,
			opt(8) rules: InteractionRules,
			opt(16) tags: Vec<i32>,
			opt(32) camera: InteractionCameraSettings,
			opt(64) charged_next: HashMap<OrderedFloat<f32>, i32>, // f32 can't be a key since NaN might not be equal to NaN due to there being millions of ways to represent it in the IEEE 754 standard, but the java side treats all NaNs as equal so this is a workaround
			opt(128) forks: HashMap<InteractionType, i32>,
		}
	}
}

define_packet! {
	AngledWielding {
		angle_rad: f32,
		angle_distance_rad: f32,
		has_modifiers: bool
	}
}

define_packet! {
	WorldParticle {
		fixed {
			required scale: f32,
			opt(1) color: Color,
			opt(2) position_offset: Vector3f,
			opt(4) rotation_offset: DirectionF,
		}
		variable {
			opt(8) system_id: String
		}
	}
}

define_packet! {
	DamageEffects {
		fixed {
			required sound_event_index: i32
		}
		variable {
			opt(1) model_particles: Vec<ModelParticle>,
			opt(2) world_particles: Vec<WorldParticle>
		}
	}
}

define_packet! {
	WieldingInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required failed: i32,
			required allow_indefinite_hold: bool,
			required display_progress: bool,
			required cancel_on_other_click: bool,
			required fail_on_damage: bool,
			required mouse_sensitivity_adjustment_target: f32,
			required mouse_sensitivity_adjustment_duration: f32,
			opt(0, 1) charging_delay: ChargingDelay,
			required has_modifiers: bool,
			opt(0, 2) angled_wielding: AngledWielding
		}
		variable {
			opt(0, 4) effects: InteractionEffects,
			opt(0, 8) settings: HashMap<GameMode, InteractionSettings>,
			opt(0, 16) rules: InteractionRules,
			opt(0, 32) tags: Vec<i32>,
			opt(0, 64) camera: InteractionCameraSettings,
			opt(0, 128) charged_next: HashMap<OrderedFloat<f32>, i32>, // f32 can't be a key since NaN might not be equal to NaN due to there being millions of ways to represent it in the IEEE 754 standard, but the java side treats all NaNs as equal so this is a workaround
			opt(1, 1) forks: HashMap<InteractionType, i32>,
			opt(1, 2) blocked_effects: DamageEffects
		}
	}
}

define_packet! {
	ChainingInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required chaining_allowance: f32
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) chain_id: String,
			opt(64) chaining_next: Vec<i32>,
			opt(128) flags: HashMap<String, i32>
		}
	}
}

define_packet! {
	ConditionInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			opt(0, 1) required_game_mode: GameMode,
			opt(0, 2) jumping: bool, // @Nullable Boolean be like
			opt(0, 4) swimming: bool,
			opt(0, 8) crouching: bool,
			opt(0, 16) running: bool,
			opt(0, 32) flying: bool,
		}
		variable {
			opt(0, 64) effects: InteractionEffects,
			opt(0, 128) settings: HashMap<GameMode, InteractionSettings>,
			opt(1, 1) rules: InteractionRules,
			opt(1, 2) tags: Vec<i32>,
			opt(1, 4) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	StatsConditionInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required less_than: bool,
			required lenient: bool,
			required value_type: ValueType
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) costs: HashMap<i32, f32>
		}
	}
}

define_packet! {
	BlockIdMatcher {
		fixed {
			required tag_index: i32
		}
		variable {
			opt(1) id: String,
			opt(2) state: String
		}
	}
}

define_packet! {
	BlockMatcher {
		fixed {
			required face: BlockFace,
			required static_face: bool,
		}
		variable {
			opt(1) block: BlockIdMatcher
		}
	}
}

define_packet! {
	BlockConditionInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) matchers: Vec<BlockMatcher>
		}
	}
}

define_packet! {
	ReplaceInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required default_value: i32
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) variable: String
		}
	}
}

define_packet! {
	ChangeBlockInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
			required world_sound_event_index: i32,
			required require_not_broken: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) block_changes: HashMap<i32, i32>
		}
	}
}

define_packet! {
	ChangeStateInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) state_changes: HashMap<String, String>
		}
	}
}

define_packet! {
	FirstClickInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required click: i32,
			required held: i32
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	RefillContainerInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required use_latest_target: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) refill_fluiids: Vec<i32>
		}
	}
}

define_enum! {
	pub enum FailOnType {
		Neither = 0,
		Entity = 1,
		Block = 2,
		Either = 3,
	}
}

define_enum! {
	pub enum EntityMatcherType {
		Server = 0,
		VulnerableMatcher = 1,
		Player = 2,
	}
}

define_packet! {
	EntityMatcher {
		entity_matcher_type: EntityMatcherType,
		invert: bool
	}
}

define_packet! {
	HitEntity {
		variable {
			opt(1) matchers: Vec<EntityMatcher>
		}
	}
}

define_packet! {
	SelectInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required ignore_owner: bool,
			required hit_entity: i32,
			required fail_on: FailOnType,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) selector: Selector,
			opt(64) hit_entity_rules: Vec<HitEntity>
		}
	}
}

define_packet! {
	AngledDamage {
		fixed {
			required angle: f64,
			required angle_distance: f64,
			required next: i32,
		}
		variable {
			opt(1) damage_effects: DamageEffects
		}
	}
}

define_packet! {
	TargetedDamage {
		fixed {
			required index: i32,
			required next: i32,
		}
		variable {
			opt(1) damage_effects: DamageEffects
		}
	}
}

define_packet! {
	EntityStatOnHit {
		fixed {
			required entity_stat_index: i32,
			required amount: f32,
			required multiplier_per_extra_entities_hit_count: f32,
		}
		variable {
			opt(1) multipliers_per_entities_hit: Vec<f32>
		}
	}
}

define_packet! {
	DamageEntityInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required blocked: i32,
		}
		variable {
			opt(0, 1) effects: InteractionEffects,
			opt(0, 2) settings: HashMap<GameMode, InteractionSettings>,
			opt(0, 4) rules: InteractionRules,
			opt(0, 8) tags: Vec<i32>,
			opt(0, 16) camera: InteractionCameraSettings,
			opt(0, 32) damage_effects: DamageEffects,
			opt(0, 64) angled_damage: Vec<AngledDamage>,
			opt(0, 128) targeted_damage: HashMap<String, TargetedDamage>,
			opt(1, 1) entity_status_on_hit: Vec<EntityStatOnHit>
		}
	}
}

define_packet! {
	RepeatInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required fork_interactions: i32,
			required repeat: i32
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}
define_packet! {
	ParallelInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) next: Vec<i32>,
		}
	}
}

define_packet! {
	ChangeActiveSlotInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required target_slot: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_enum! {
	pub enum Match {
		All = 0,
		None = 1
	}
}

define_enum! {
	pub enum InteractionTarget {
		User = 0,
		Owner = 1,
		Target = 2
	}
}

define_packet! {
	EffectConditionInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required r#match: Match,
			required entity_target: InteractionTarget
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) entity_effects: Vec<i32>,
		}
	}
}

define_enum! {
	pub enum RaycastMode {
		FollowMotion = 0,
		FollowLook = 1
	}
}

define_packet! {
	AppliedForce {
		fixed {
			opt(1) direction: Vector3f,
			required adjust_vertical: bool,
			required force: f32,
		}
	}
}

define_packet! {
	ApplyForceInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			opt(1) velocity_config: VelocityConfig,
			required change_velocity_type: ChangeVelocityType,
			required duration: f32,
			required wait_for_ground: bool,
			required wait_for_collision: bool,
			required ground_check_delay: f32,
			required collision_check_delay: f32,
			required ground_next: i32,
			required collision_next: i32,
			required raycast_distance: f32,
			required raycast_height_offset: f32,
			required raycast_mode: RaycastMode,
		}
		variable {
			opt(2) effects: InteractionEffects,
			opt(4) settings: HashMap<GameMode, InteractionSettings>,
			opt(8) rules: InteractionRules,
			opt(16) tags: Vec<i32>,
			opt(32) camera: InteractionCameraSettings,
			opt(64) forces: Vec<AppliedForce>,
		}
	}
}

define_packet! {
	ApplyEffectInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required effect_id: i32,
			required entity_target: InteractionTarget
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	ClearEntityEffectInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required effect_id: i32,
			required entity_target: InteractionTarget
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	SerialInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) serial_interactions: Vec<i32>,
		}
	}
}

define_enum! {
	pub enum ChangeStatBehavior {
		Add = 0,
		Set = 1
	}
}

define_packet! {
	ChangeStatInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required entity_target: InteractionTarget,
			required value_type: ValueType,
			required change_stat_behavior: ChangeStatBehavior
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) stat_modifiers: HashMap<i32, f32>,
		}
	}
}

define_packet! {
	MovementConditionInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required forward: i32,
			required backward: i32,
			required left: i32,
			required right: i32,
			required forward_left: i32,
			required forward_right: i32,
			required back_left: i32,
			required back_right: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	ProjectileInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) config_id: String,
		}
	}
}

define_packet! {
	RemoveEntityInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required entity_target: InteractionTarget,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	InteractionCooldown {
		fixed {
			required cooldown: f32,
			required click_bypass: bool,
			required skip_cooldown_reset: bool,
			required interrupt_recharge: bool,
		}
		variable {
			opt(1) cooldown_id: String,
			opt(2) charge_times: Vec<f32>,
		}
	}
}

define_packet! {
	ResetCooldownInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) cooldown: InteractionCooldown
		}
	}
}

define_packet! {
	TriggerCooldownInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) cooldown: InteractionCooldown
		}
	}
}

define_packet! {
	CooldownConditionInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) cooldown_id: String
		}
	}
}

define_packet! {
	ChainFlagInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) chain_id: String,
			opt(64) flag: String,
		}
	}
}

define_packet! {
	IncrementCooldownInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required cooldown_increment_time: f32,
			required cooldown_increment_charge: i32,
			required cooldown_increment_charge_time: f32,
			required cooldown_increment_interrupt: bool,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) cooldown_id: String
		}
	}
}

define_packet! {
	CancelChainInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) chain_id: String
		}
	}
}

define_packet! {
	DeployableConfig {
		fixed {
			required allow_place_on_walls: bool
		}
		variable {
			opt(1) model: Model,
			opt(2) model_preview: Model
		}
	}
}

define_packet! {
	RunRootInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required root_interaction: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_enum! {
	pub enum CameraPerspectiveType {
		First = 0,
		Third = 1
	}
}

define_enum! {
	pub enum CameraActionType {
		ForcePerspective = 0,
		Orbit = 1,
		Transition = 2,
	}
}

define_packet! {
	CameraInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required camera_action: CameraActionType,
			required camera_perspective: CameraPerspectiveType,
			required camera_persist: bool,
			required camera_interaction_time: f32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	SpawnDeployableFromRaycastInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required max_distance: f32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			// DeployableConfig is too big due to the Models it containts, it causes the this enum variant's size to shoot up to over 1688 without boxing. With boxing it's lower than 200, which is what we want & makes clippy happy.
			opt(32) deployable_config: Box<DeployableConfig>,
			opt(64) costs: HashMap<i32, f32>,
		}
	}
}

define_packet! {
	MemoriesConditionInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			// No next on this one
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
			opt(32) memories_next: HashMap<i32, i32>,
		}
	}
}

define_packet! {
	ToggleGliderInteraction {
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
		}
		variable {
			opt(1) effects: InteractionEffects,
			opt(2) settings: HashMap<GameMode, InteractionSettings>,
			opt(4) rules: InteractionRules,
			opt(8) tags: Vec<i32>,
			opt(16) camera: InteractionCameraSettings,
		}
	}
}

id_dispatch! {
	Interaction {
		0  => SimpleBlockInteraction,
		1  => SimpleInteraction,
		2  => PlaceBlockInteraction,
		3  => BreakBlockInteraction,
		4  => PickBlockInteraction,
		5  => UseBlockInteraction,
		6  => UseEntityInteraction,
		7  => BuilderToolInteraction,
		8  => ModifyInventoryInteraction,
		9  => ChargingInteraction,
		10 => WieldingInteraction,
		11 => ChainingInteraction,
		12 => ConditionInteraction,
		13 => StatsConditionInteraction,
		14 => BlockConditionInteraction,
		15 => ReplaceInteraction,
		16 => ChangeBlockInteraction,
		17 => ChangeStateInteraction,
		18 => FirstClickInteraction,
		// 19 => RefillContainerInteraction, // This was removed for some reason. There's no case 19 anymore, but rather a default case which throws an exception.
		20 => SelectInteraction,
		21 => DamageEntityInteraction,
		22 => RepeatInteraction,
		23 => ParallelInteraction,
		24 => ChangeActiveSlotInteraction,
		25 => EffectConditionInteraction,
		26 => ApplyForceInteraction,
		27 => ApplyEffectInteraction,
		28 => ClearEntityEffectInteraction,
		29 => SerialInteraction,
		30 => ChangeStatInteraction,
		31 => MovementConditionInteraction,
		32 => ProjectileInteraction,
		33 => RemoveEntityInteraction,
		34 => ResetCooldownInteraction,
		35 => TriggerCooldownInteraction,
		36 => CooldownConditionInteraction,
		37 => ChainFlagInteraction,
		38 => IncrementCooldownInteraction,
		39 => CancelChainInteraction,
		40 => RunRootInteraction,
		41 => CameraInteraction,
		42 => SpawnDeployableFromRaycastInteraction,
		43 => MemoriesConditionInteraction,
		44 => ToggleGliderInteraction,
	}
}
