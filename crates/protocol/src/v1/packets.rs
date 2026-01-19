#![allow(unused_variables, clippy::enum_variant_names)]

use std::collections::HashMap;

use bytes::{
	Buf,
	Bytes,
	BytesMut,
};
use entities::{
	ChangeVelocityType,
	VelocityConfig,
};
use ordered_float::OrderedFloat;
use uuid::Uuid;

use crate::{
	codec::{
		BitOptionVec,
		FixedAscii,
		HytaleCodec,
		PacketError,
		PacketResult,
		VarInt,
	},
	define_enum,
	define_packet,
	v1::{
		buildertools::BuilderToolState,
		interaction::InteractionType,
	},
};

pub mod assets;
pub mod auth;
pub mod buildertools;
pub mod camera;
pub mod connection;
pub mod entities;
pub mod interaction;
pub mod interface;
pub mod inventory;
pub mod machinima;
pub mod player;
pub mod serveraccess;
pub mod setup;
pub mod window;
pub mod world;
pub mod worldmap;

/// Max size for variable length items, like strings, maps, lists, etc.
pub const MAX_SIZE: i32 = 4_096_000;

define_packet! { HostAddress { port: u16, host: String } }

define_packet! {
   Asset {
	   hash: FixedAscii<64>, // 64-char Hex String
	   name: String,         // Filename (e.g. "models/player.json")
   }
}

define_packet! { InstantData { seconds: i64, nanos: i32 } }

define_packet! { Vector2f { x: f32, y: f32 } }

define_packet! { Vector3f { x: f32, y: f32, z: f32 } }

define_packet! { PositionF { x: f64, y: f64, z: f64 } }

define_enum! {
	pub enum PositionType {
		AttachedToPlusOffset = 0,
		Custom = 1
	}
}

define_packet! { DirectionF { yaw: f32, pitch: f32, roll: f32 } }

define_enum! {
	pub enum RotationType {
		AttachedToPlusOffset = 0,
		Custom = 1
	}
}

define_enum! {
	pub enum CanMoveType {
		AttachedToLocalPlayer = 0,
		Always = 1
	}
}

define_enum! {
	pub enum PositionDistanceOffsetType {
		DistanceOffset = 0,
		DistanceOffsetRaycast = 1,
		None = 2
	}
}

define_enum! {
	pub enum ApplyMovementType {
		CharacterController = 0,
		Position = 1
	}
}

define_enum! {
	pub enum ApplyLookType {
		LocalPlayerLookOrientation = 0,
		Rotation = 1
	}
}

define_enum! {
	pub enum MouseInputType {
		LookAtTarget = 0,
		LookAtTargetBlock = 1,
		LookAtTargetEntity = 2,
		LookAtPlane = 3
	}
}

define_enum! {
	pub enum AttachedToType {
		LocalPlayer = 0,
		EntityId = 1,
		None = 2
	}
}
define_enum! {
	pub enum MovementForceRotationType {
		AttachedToHead = 0,
		CameraRotation = 1,
		Custom = 2
	}
}

define_enum! {
	pub enum MouseInputTargetType {
		Any = 0,
		Block = 1,
		Entity = 2,
		None = 3
	}
}

define_enum! {
	pub enum MaybeBool {
		Null = 0,
		False = 1,
		True = 2
	}
}

define_packet! {
	StringParamValue {
		fixed {
			opt(0) value: String
		}
	}
}

#[derive(Debug, Clone)]
pub enum ParamValue {
	String(StringParamValue),
	Bool(bool),
	Double(f64),
	Int(i32),
	Long(i64),
}

impl HytaleCodec for ParamValue {
	fn encode(&self, buf: &mut BytesMut) {
		match self {
			ParamValue::String(v) => {
				VarInt(0).encode(buf);
				v.encode(buf);
			}
			ParamValue::Bool(v) => {
				VarInt(1).encode(buf);
				v.encode(buf);
			}
			ParamValue::Double(v) => {
				VarInt(2).encode(buf);
				v.encode(buf);
			}
			ParamValue::Int(v) => {
				VarInt(3).encode(buf);
				v.encode(buf);
			}
			ParamValue::Long(v) => {
				VarInt(4).encode(buf);
				v.encode(buf);
			}
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let type_id = VarInt::decode(buf)?.0;

		match type_id {
			0 => Ok(ParamValue::String(<StringParamValue as HytaleCodec>::decode(buf)?)),
			1 => Ok(ParamValue::Bool(<bool as HytaleCodec>::decode(buf)?)),
			2 => Ok(ParamValue::Double(<f64 as HytaleCodec>::decode(buf)?)),
			3 => Ok(ParamValue::Int(<i32 as HytaleCodec>::decode(buf)?)),
			4 => Ok(ParamValue::Long(<i64 as HytaleCodec>::decode(buf)?)),
			_ => Err(PacketError::InvalidEnumVariant(type_id as u8)),
		}
	}
}

define_packet! {
	FormattedMessage {
		fixed {
			required bold: MaybeBool,
			required italic: MaybeBool,
			required monospace: MaybeBool,
			required underlined: MaybeBool,
			required markup_enabled: bool,
		}
		variable {
			opt raw_text: String,
			opt message_id: String,
			opt children: Vec<FormattedMessage>,
			opt params: HashMap<String, ParamValue>,
			opt message_params: HashMap<String, FormattedMessage>,
			opt color: String,
			opt link: String
		}
	}
}

define_packet! {
   ItemWithAllMetadata {
	   fixed {
		   required quantity: i32,
		   required durability: f64,
		   required max_durability: f64,
		   required override_dropped_item_animation: bool
	   }
	   variable {
		   required item_id: String,
		   opt metadata: String
	   }
   }
}

define_packet! {
	MaterialQuantity {
		fixed {
			required item_tag: i32,
			required quantity: i32
		}
		variable {
			opt item_id: String,
			opt resource_type_id: String
		}
	}
}

define_enum!(
	pub enum BenchType {
		Crafting = 0,
		Processing = 1,
		DiagramCrafting = 2,
		StructuralCrafting = 3,
	}
);

define_packet! {
   BenchRequirement {
	   fixed {
		   required bench_type: BenchType,
		   required required_tier_level: i32,
	   }
	   variable {
		   opt id: String,
		   opt categories: Vec<String>
	   }
   }
}

define_packet! {
   CraftingRecipe {
	   fixed {
		   required knowledge_required: bool,
		   required time_seconds: f32,
		   required required_memories_level: i32
	   }
	   variable {
		   opt id: String,
		   opt inputs: Vec<MaterialQuantity>,
		   opt outputs: Vec<MaterialQuantity>,
		   opt primary_output: MaterialQuantity,
		   opt bench_requirement: BenchRequirement,
	   }
   }
}

define_enum! {
	pub enum Rotation {
		None = 0,
		Ninety = 1,
		OneEighty = 2,
		TwoSeventy = 3,
	}
}

define_packet! {
	BlockRotation {
		yaw: Rotation,
		pitch: Rotation,
		roll: Rotation,
	}
}

define_enum! {
	pub enum BlockFace {
		None = 0,
		Up = 1,
		Down = 2,
		North = 3,
		South = 4,
		East = 5,
		West = 6,
	}
}

define_enum! {
	pub enum MovementDirection {
		None = 0,
		Forward = 1,
		Back = 2,
		Left = 3,
		Right = 4,
		ForwardLeft = 5,
		ForwardRight = 6,
		BackLeft = 7,
		BackRight = 8,
	}
}

define_packet! {
	SelectedHitEntity {
		fixed {
			required network_id: i32,
			opt hit_location: Vector3f [pad=12],
			opt position: PositionF [pad=24],
			opt body_rotation: DirectionF [pad=12]
		}
	}
}

define_enum! {
	pub enum ComponentUpdateType {
		Nameplate = 0,
		UIComponents = 1,
		CombatText = 2,
		Model = 3,
		PlayerSkin = 4,
		Item = 5,
		Block = 6,
		Equipment = 7,
		EntityStats = 8,
		Transform = 9,
		MovementStates = 10,
		EntityEffects = 11,
		Interactions = 12,
		DynamicLight = 13,
		Interactable = 14,
		Intangible = 15,
		Invulnerable = 16,
		RespondToHit = 17,
		HitboxCollision = 18,
		Repulsion = 19,
		Prediction = 20,
		Audio = 21,
		Mounted = 22,
		NewSpawn = 23,
		ActiveAnimations = 24,
	}
}

define_packet! {
   Nameplate {
	   fixed {
		   opt text: String
	   }
   }
}

define_packet! {
   CombatTextUpdate {
	   fixed {
		   required hit_angle_deg: f32,
		   opt text: String
	   }
   }
}

define_packet! { RangeF { min: f32, max: f32 } }

define_packet! { RangeI { min: i32, max: i32 } }

define_packet! { RangeB { min: u8, max: u8 } }

define_enum! {
	pub enum CameraNode {
		None = 0,
		Head = 1,
		LShoulder = 2,
		RShoulder = 3,
		Belly = 4,
	}
}

define_packet! {
   CameraAxis {
	   fixed {
		   opt angle_range: RangeF [pad=8],
		   opt target_nodes: Vec<CameraNode>,
	   }
   }
}

define_packet! {
   CameraSettings {
	   fixed {
		   opt position_offset: Vector3f [pad=12],
	   }
	   variable {
		   opt yaw: CameraAxis,
		   opt pitch: CameraAxis,
		   // No roll
	   }
   }
}

define_packet! {
   Animation {
	   fixed {
		   required speed: f32,
		   required blending_duration: f32,
		   required looping: bool,
		   required weight: f32,
		   required sound_event_indexx: i32,
		   required passive_loop_count: i32
	   }
	   variable {
		   opt name: String,
		   opt footstep_invervals_count: Vec<i32>
	   }
   }
}

define_packet! {
   AnimationSet {
	   fixed {
		   opt next_animation_delay: RangeF [pad=8],
	   }
	   variable {
		   opt id: String,
		   opt animations: Vec<Animation>,
	   }
   }
}

define_packet! {
   ModelAttachment {
	   variable {
		   opt model: String,
		   opt texture: String,
		   opt gradient_set: String,
		   opt gradient_id: String
	   }
   }
}

define_packet! { Hitbox {
	min_pos: Vector3f,
	max_pos: Vector3f,
} }
define_enum! {
	pub enum EntityPart {
		// This is supposed to be Self = 0 but that's a rust keyword, it can't even be used as r#Self.
		This = 0,
		Entity = 1,
		PrimaryItem = 2,
		SecondaryItem = 3
	}
}

define_packet! { Color { red: u8, green: u8, blue: u8 } }
define_packet! { ColorAlpha { alpha: u8, red: u8, green: u8, blue: u8 } }

define_packet! {
   ModelParticle {
	   fixed {
		   required scale: f32,
		   opt(1) color: Color [pad=3],
		   required target_entity_part: EntityPart,
		   opt(3) position_offset: Vector3f [pad=12],
		   opt(4) rotation_offset: DirectionF [pad=12],
		   required detached_from_model: bool,
	   }
	   variable {
		   opt(0) system_id: String,
		   opt(2) target_node_name: String
	   }
   }
}
define_packet! {
   ModelTrail {
	   fixed {
		   required target_entity_part: EntityPart,
		   opt(2) position_offset: Vector3f [pad=12],
		   opt(3) rotation_offset: DirectionF [pad=12],
		   required fixed_rotation: bool,
	   }
	   variable {
		   opt(0) trail_id: String,
		   opt(1) target_node_name: String
	   }
   }
}
define_packet! { ColorLight {
	radius: u8,
	red: u8,
	green: u8,
	blue: u8,
} }
define_packet! {
   DetailBox {
	   fixed {
		   opt offset: Vector3f [pad=12],
		   opt r#box: Hitbox // Box is a keyword in rust
	   }
   }
}
define_enum! {
	pub enum Phobia {
		None = 0,
		Arachnophobia = 1
	}
}

define_packet! {
   Model {
	   mask_size: 2,
	   fixed {
		   required scale: f32,
		   required eye_height: f32,
		   required crouch_offset: f32,
		   opt(8) hitbox: Hitbox [pad=24],
		   opt(11) light: ColorLight [pad=4],
		   required phobia: Phobia,
	   }
	   variable {
		   opt(0) asset_id: String,
		   opt(1) path: String,
		   opt(2) texture: String,
		   opt(3) gradient_set: String,
		   opt(4) gradient_id: String,
		   opt(5) camera: CameraSettings,
		   opt(6) animation_sets: HashMap<String, AnimationSet>,
		   opt(7) attachments: Vec<ModelAttachment>,

		   opt(9) particles: Vec<ModelParticle>,
		   opt(10) trails: Vec<ModelTrail>,
		   opt(12) detail_boxes: HashMap<String, Vec<DetailBox>>,
		   opt(13) phobia_model: Box<Model>,
	   }
   }
}
define_packet! {
   Equipment {
	   variable {
		   opt armor_ids: Vec<String>,
		   opt right_hand_item_id: String,
		   opt left_hand_item_id: String,
	   }
   }
}
define_enum! {
	pub enum EntityStatOp {
		Init = 0,
		Remove = 1,
		PutModifier = 2,
		RemoveModifier = 3,
		Add = 4,
		Set = 5,
		Minimize = 6,
		Maximize = 7,
		Reset = 8,
	}
}

define_enum! {
	pub enum ModifierTarget {
		Min = 0,
		Max = 1
	}
}

define_enum! {
	pub enum CalculationType {
		Additive = 0,
		Multiplicative = 1,
	}
}

define_packet! { Modifier {
	target: ModifierTarget,
	calculation_type: CalculationType,
	amount: f32,
} }
define_packet! {
   EntityStatUpdate {
	   fixed {
		   required op: EntityStatOp,
		   required predictable: bool,
		   required value: f32,
		   opt(2) modifier: Modifier [pad=6]
	   }
	   variable {
		   opt modifiers: HashMap<String, Modifier>,
		   opt modifier_key: String
	   }
   }
}
define_packet! {
   ModelTransform {
	   fixed {
		   opt position: PositionF [pad=24],
		   opt body_orientation: DirectionF [pad=12],
		   opt look_orientation: DirectionF [pad=12],
	   }
   }
}
define_packet! { MovementStates {
	idle: bool,
	horizontal_idle: bool,
	jumping: bool,
	flying: bool,
	walking: bool,
	running: bool,
	sprinting: bool,
	crouching: bool,
	forced_crouching: bool,
	falling: bool,
	climbing: bool,
	in_fluid: bool,
	swimming: bool,
	swim_jumping: bool,
	on_ground: bool,
	mantling: bool,
	sliding: bool,
	mounting: bool,
	rolling: bool,
	sitting: bool,
	gliding: bool,
	sleeping: bool,
} }
define_enum! {
	pub enum EffectOp {
		Add = 0,
		Remove = 1
	}
}

define_packet! {
   EntityEffectUpdate {
	   fixed {
		   required effect_op: EffectOp,
		   required id: i32,
		   required remaining_time: f32,
		   required infinite: bool,
		   required debuff: bool,
		   opt status_effect_icon: String
	   }
   }
}
define_enum!(
	pub enum MountController {
		Minecart = 0,
		BlockMount = 1,
	}
);

define_enum! {
	pub enum BlockMountType {
		Seat = 0,
		Bed = 1
	}
}

define_packet! {
   BlockMount {
	   fixed {
		   required mount_type: BlockMountType,
		   opt position: Vector3f [pad=12],
		   opt orientation: Vector3f [pad=12],
		   required block_type_id: i32,
	   }
   }
}
define_packet! {
   MountedUpdate {
	   fixed {
		   required mounted_to_entity: i32,
		   opt attachment_offset: Vector3f [pad=12],
		   required mount_controller: MountController,
		   opt block: BlockMount [pad=30],
	   }
   }
}
define_packet! {
   ComponentUpdate {
	   mask_size: 3,
	   fixed {
		   required update_type: ComponentUpdateType,
		   required block_id: i32,
		   required entity_scale: f32,
		   opt(8) transform: ModelTransform [pad=49],
		   opt(9) movement_states: MovementStates [pad=22],
		   opt(12) dynamic_light: ColorLight [pad=4],
		   required hitbox_collision_config_index: i32,
		   required repulsion_config_index: i32,
		   required prediction_id: Uuid,
		   opt(15) mounted: MountedUpdate [pad=48],
	   }
	   variable {
		   opt(0) nameplate: Nameplate,
		   opt(1) entity_ui_components: Vec<i32>,
		   opt(2) combat_text_update: CombatTextUpdate,
		   opt(3) model: Model,
		   opt(4) skin: setup::PlayerSkin,
		   opt(5) item: ItemWithAllMetadata,
		   opt(6) equipment: Equipment,
		   opt(7) entity_stat_updates: HashMap<i32, Vec<EntityStatUpdate>>,
		   opt(10) entity_effect_updates: Vec<EntityEffectUpdate>,
		   opt(11) interactions: HashMap<InteractionType, i32>,
		   opt(13) sound_event_ids: Vec<i32>,
		   opt(14) interaction_hint: String,
		   opt(16) active_animations: BitOptionVec<String>,
	   }
   }
}
define_packet! {
   EntityUpdate {
	   fixed {
		   required network_id: i32
	   }
	   variable {
		   opt removed: Vec<ComponentUpdateType>,
		   opt updates: Vec<ComponentUpdate>
	   }
   }
}
define_packet! {
   ItemQuantity {
	   fixed {
		   required quantity: i32,
		   opt item_id: String,
	   }
   }
}
define_packet! { HalfFloatPosition { x: i16, y: i16, z: i16 } }

define_packet! { TeleportAck { teleport_id: u8 } }

define_packet! { Vector3d { x: f64, y: f64, z: f64 } }

define_packet! {
   DamageCause {
	   variable {
		   opt id: String,
		   opt damage_text_color: String
	   }
   }
}
define_enum! {
	pub enum DebugShape {
		Sphere = 0,
		Cylinder = 1,
		Cone = 2,
		Cube = 3,
		Frustum = 4,
	}
}

define_enum! {
	pub enum MouseButtonType {
		Left = 0,
		Middle = 1,
		Right = 2,
		X1 = 3,
		X2 = 4,
	}
}

define_enum! {
	pub enum MouseButtonState {
		Pressed = 0,
		Released = 1,
	}
}

define_packet! { MouseButtonEvent {
	mouse_button_type: MouseButtonType,
	state: MouseButtonState,
	clicks: u8
} }

define_packet! {
	Vector3i {
		x: i32,
		y: i32,
		z: i32,
	}
}

define_packet! { Vector2i { x: i32, y: i32 } }

define_packet! {
   MouseMotionEvent {
	   fixed {
		   opt relative_motion: Vector2i [pad=8],
		   opt mouse_button_type: Vec<MouseButtonType>,
	   }
   }
}
define_packet! {
   WorldInteraction {
	   fixed {
		   required entity_id: i32,
		   opt block_position: Vector3i [pad=12],
		   opt block_rotation: BlockRotation [pad=3],
	   }
   }
}
define_enum! {
	pub enum GameMode {
		Adventure = 0,
		Creative = 1
	}
}

define_packet! { SavedMovementStates { flying: bool } }

define_enum! {
	pub enum PickupLocation {
		Hotbar = 0,
		Storage = 1
	}
}

define_packet! { MovementSettings {
	mass: f32,
	drag_coefficient: f32,
	inverted_gravity: bool,
	velocity_resistance: f32,
	jump_force: f32,
	swim_jump_force: f32,
	jump_buffer_duration: f32,
	jump_buffer_max_y_velocity: f32,
	acceleration: f32,
	air_drag_range: RangeF,
	air_drag_speed_range: RangeF,
	air_friction_range: RangeF,
	air_friction_speed_range: RangeF,
	air_speed_multiplier: f32,
	air_control_speed_range: RangeF,
	air_control_multiplier_range: RangeF,
	combo_air_speed_multiplier: f32,
	base_speed: f32,
	climb_speed: f32,
	climb_speed_lateral: f32,
	climb_up_sprint_speed: f32,
	climb_down_sprint_speed: f32,
	horizontal_fly_speed: f32,
	vertical_fly_speed: f32,
	speed_multiplier_range: RangeF,
	wish_direction_gravity: Vector2f,
	wish_direction_weight: Vector2f,
	can_fly: bool,
	collision_expulsion_force: f32,
	forward_walk_speed_multiplier: f32,
	backward_walk_speed_multiplier: f32,
	strafe_walk_speed_multiplier: f32,
	forward_run_speed_multiplier: f32,
	backward_run_speed_multiplier: f32,
	strafe_run_speed_multiplier: f32,
	forward_crouch_speed_multiplier: f32,
	backward_crouch_speed_multiplier: f32,
	strafe_crouch_speed_multiplier: f32,
	forward_sprint_speed_multiplier: f32,
	variable_jump_fall_force: f32,
	fall_effect_duration: f32,
	fall_jump_force: f32,
	fall_momentum_loss: f32,
	auto_jump_obstacle_speed_loss: f32,
	auto_jump_obstacle_sprint_speed_loss: f32,
	auto_jump_obstacle_effect_duration: f32,
	auto_jump_obstacle_sprint_effect_duration: f32,
	auto_jump_obstacle_max_angle: f32,
	auto_jump_disable_jumping: bool,
	min_slide_entry_speed: f32,
	slide_exit_speed: f32,
	// These 2 make more sense as individual f32s rather than ranges
	min_fall_speed_to_engage_roll: f32,
	max_fall_speed_to_engage_roll: f32,
	roll_start_speed_modifier: f32,
	roll_exit_speed_modifier: f32,
	roll_time_to_complete: f32,
} }
define_enum! {
	pub enum SortType {
		Name = 0,
		Type = 1,
		Rarity = 2,
	}
}

define_enum! {
	pub enum WindowType {
		Container = 0,
		PocketCrafting = 1,
		BasicCrafting = 2,
		DiagramCrafting = 3,
		StructuralCrafting = 4,
		Processing = 5,
		Memories = 6,
	}
}

define_packet! {
   ExtraResources {
	   fixed {
		   opt resources: Vec<ItemQuantity>
	   }
   }
}
// Similar to ParamValue
#[derive(Debug, Clone)]
pub enum WindowAction {
	CraftRecipeAction(window::CraftRecipeAction),
	TierUpgradeAction(window::TierUpgradeAction),
	SelectSlotAction(window::SelectSlotAction),
	ChangeBlockAction(window::ChangeBlockAction),
	SetActiveAction(window::SetActiveAction),
	CraftItemAction(window::CraftItemAction),
	UpdateCategoryAction(window::UpdateCategoryAction),
	CancelCraftingAction(window::CancelCraftingAction),
	SortItemsAction(window::SortItemsAction),
}

impl HytaleCodec for WindowAction {
	fn encode(&self, buf: &mut BytesMut) {
		match self {
			WindowAction::CraftRecipeAction(v) => {
				VarInt(0).encode(buf);
				v.encode(buf);
			}
			WindowAction::TierUpgradeAction(v) => {
				VarInt(1).encode(buf);
				v.encode(buf);
			}
			WindowAction::SelectSlotAction(v) => {
				VarInt(2).encode(buf);
				v.encode(buf);
			}
			WindowAction::ChangeBlockAction(v) => {
				VarInt(3).encode(buf);
				v.encode(buf);
			}
			WindowAction::SetActiveAction(v) => {
				VarInt(4).encode(buf);
				v.encode(buf);
			}
			WindowAction::CraftItemAction(v) => {
				VarInt(5).encode(buf);
				v.encode(buf);
			}
			WindowAction::UpdateCategoryAction(v) => {
				VarInt(6).encode(buf);
				v.encode(buf);
			}
			WindowAction::CancelCraftingAction(v) => {
				VarInt(7).encode(buf);
				v.encode(buf);
			}
			WindowAction::SortItemsAction(v) => {
				VarInt(8).encode(buf);
				v.encode(buf);
			}
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let type_id = VarInt::decode(buf)?.0;

		match type_id {
			0 => Ok(WindowAction::CraftRecipeAction(window::CraftRecipeAction::decode(buf)?)),
			1 => Ok(WindowAction::TierUpgradeAction(window::TierUpgradeAction::decode(buf)?)),
			2 => Ok(WindowAction::SelectSlotAction(window::SelectSlotAction::decode(buf)?)),
			3 => Ok(WindowAction::ChangeBlockAction(window::ChangeBlockAction::decode(buf)?)),
			4 => Ok(WindowAction::SetActiveAction(window::SetActiveAction::decode(buf)?)),
			5 => Ok(WindowAction::CraftItemAction(window::CraftItemAction::decode(buf)?)),
			6 => Ok(WindowAction::UpdateCategoryAction(window::UpdateCategoryAction::decode(buf)?)),
			7 => Ok(WindowAction::CancelCraftingAction(window::CancelCraftingAction::decode(buf)?)),
			8 => Ok(WindowAction::SortItemsAction(window::SortItemsAction::decode(buf)?)),
			_ => Err(PacketError::InvalidEnumVariant(type_id as u8)),
		}
	}
}

define_enum! {
	pub enum SoundCategory {
		Music = 0,
		Ambient = 1,
		SFX = 2,
		UI = 3,
	}
}

define_enum! {
	pub enum BlockParticleEvent {
		Walk = 0,
		Run = 1,
		Sprint = 2,
		SoftLand = 3,
		HardLand = 4,
		MoveOut = 5,
		Hit = 6,
		Break = 7,
		Build = 8,
		Physics = 9,
	}
}

define_packet! {
   Transform {
	   fixed {
		   opt position: PositionF [pad=24],
		   opt orientation: DirectionF [pad=12],
	   }
   }
}
define_packet! {
   Objective {
	   fixed {
		   required objective_uuid: Uuid,
	   }
	   variable {
		   opt objective_title_key: String,
		   opt objective_description_key: String,
		   opt objective_line_id: String,
		   opt tasks: Vec<ObjectiveTask>
	   }
   }
}
define_packet! {
   ObjectiveTask {
	   fixed {
		   required current_completion: i32,
		   required completion_needed: i32,
		   opt task_description_key: String
	   }
   }
}
define_enum! {
	pub enum UpdateType {
		Init = 0,
		AddOrUpdate = 1,
		Remove = 2,
	}
}

define_packet! {
	AmbienceFXBlockSoundSet {
		fixed {
			required block_sound_set_index: i32,
			opt percent: RangeF,
		}
	}
}

define_packet! {
	AmbienceFXConditions {
		mask_size: 2
		fixed {
			required never: bool,
			required environment_tag_pattern_index: i32,
			required weather_tag_pattern_index: i32,
			opt(4) altitude: RangeI [pad=8],
			opt(5) walls: RangeB [pad=2],
			required roof: bool,
			required roof_material_tag_pattern_index: i32,
			required floor: bool,
			opt(6) sun_light_level: RangeB [pad=2],
			opt(7) torch_light_level: RangeB [pad=2],
			opt(8) global_light_level: RangeB [pad=2],
			opt(9) day_time: RangeF [pad=8],
		}
		variable {
			opt(0) environment_indices: Vec<i32>,
			opt(1) weather_indices: Vec<i32>,
			opt(2) fluid_fx_indices: Vec<i32>,
			opt(3) surrounding_block_sound_sets: Vec<AmbienceFXBlockSoundSet>
		}
	}
}

define_enum! {
	pub enum AmbienceFXSoundPlay3D {
		Random = 0,
		LocationName = 1,
		No = 2
	}
}

define_enum! {
	pub enum AmbienceFXAltitude {
		Normal = 0,
		Lowest = 1,
		Highest = 2,
		Random = 3
	}
}

define_packet! {
	AmbienceFXSound {
		fixed {
			required sound_event_index: i32,
			required play_3d: AmbienceFXSoundPlay3D,
			required block_sound_set_index: i32,
			required altitude: AmbienceFXAltitude,
			opt frequency: RangeF [pad=8],
			opt volume: RangeI [pad=8],
		}
	}
}

define_packet! {
	AmbienceFXMusic {
		fixed {
			required volume: f32,
			opt tracks: Vec<String>,
		}
	}
}

define_enum! {
	pub enum AmbienceTransitionSpeed {
		Default = 0,
		Fast = 1,
		Instant = 2
	}
}

define_packet! {
	AmbienceFXAmbientBed {
		fixed {
			required volume: f32,
			required transition_speed: AmbienceTransitionSpeed,
			opt track: String,
		}
	}
}

define_packet! {
	AmbienceFXSoundEffect {
		reverb_effect_index: i32,
		equalizer_effect_index: i32,
		is_instant: bool,
	}
}

define_packet! {
	AmbienceFX {
		fixed {
			opt(5) sound_effect: AmbienceFXSoundEffect [pad=9],
			required priority: i32,
			required audio_category_index: i32,
		}
		variable {
			opt(0) id: String,
			opt(1) conditions: AmbienceFXConditions,
			opt(2) sounds: Vec<AmbienceFXSound>,
			opt(3) music: AmbienceFXMusic,
			opt(4) ambient_bed: AmbienceFXAmbientBed,
			opt(6) blocked_ambience_fx_indices: Vec<i32>,
		}
	}
}

define_packet! {
	AudioCategory {
		fixed {
			required volume: f32,
			opt id: String,
		}
	}
}

define_packet! {
	BlockBreakingDecal {
		fixed {
			opt stage_textures: Vec<String>,
		}
	}
}

define_packet! {
	BlockGroup {
		fixed {
			opt names: Vec<String>,
		}
	}
}

define_packet! {
	BlockParticleSet {
		fixed {
			opt(1) color: Color [pad=3],
			required scale: f32,
			opt(2) position_offset: Vector3f [pad=12],
			opt(3) rotation_offset: DirectionF [pad=12],
		}
		variable {
			opt(0) id: String,
			opt(4) particle_system_ids: HashMap<BlockParticleEvent, String>,
		}
	}
}

define_packet! {
	BlockSet {
		variable {
			opt name: String,
			opt blocks: Vec<i32>
		}
	}
}

define_packet! {
	// This is the same as RangeF. Don't ask why. I don't have the answers.
	FloatRange {
		inclusive_min: f32,
		inclusive_max: f32
	}
}

define_enum! {
	pub enum BlockSoundEvent {
		Walk = 0,
		Land = 1,
		MoveIn = 2,
		MoveOut = 3,
		Hit = 4,
		Break = 5,
		Build = 6,
		Clone = 7,
		Harvest = 8,
	}
}

define_packet! {
	BlockSoundSet {
		fixed {
			opt(2) move_in_repeat_range: FloatRange [pad=8],
		}
		variable {
			opt(0) id: String,
			opt(1) sound_event_indices: HashMap<BlockSoundEvent, i32>,
		}
	}
}

define_enum! {
	pub enum DrawType {
		Empty = 0,
		GizmoCube = 1,
		Cube = 2,
		Model = 3,
		CubeWithModel = 4,
	}
}

define_enum! {
	pub enum BlockMaterial {
		Empty = 0,
		Solid = 1
	}
}

define_enum! {
	pub enum Opacity {
		Solid = 0,
		Semitransparent = 1,
		Cutout = 2,
		Transparent = 3,
	}
}

define_enum! {
	pub enum ShaderType {
		None = 0,
		Wind = 1,
		WindAttached = 2,
		WindRandom = 3,
		WindFractal = 4,
		Ice = 5,
		Water = 6,
		Lava = 7,
		Slime = 8,
		Ripple = 9,
	}
}

define_packet! {
	ModelTexture {
		fixed {
			required weight: f32,
			opt texture: String
		}
	}
}

define_enum! {
	pub enum BlockSupportsRequiredForType {
		Any = 0,
		All = 1,
	}
}
define_enum! {
	pub enum BlockNeighbor {
		Up = 0,
		Down = 1,
		North = 2,
		East = 3,
		South = 4,
		West = 5,
		UpNorth = 6,
		UpSouth = 7,
		UpEast = 8,
		UpWest = 9,
		DownNorth = 10,
		DownSouth = 11,
		DownEast = 12,
		DownWest = 13,
		NorthEast = 14,
		SouthEast = 15,
		SouthWest = 16,
		NorthWest = 17,
		UpNorthEast = 18,
		UpSouthEast = 19,
		UpSouthWest = 20,
		UpNorthWest = 21,
		DownNorthEast = 22,
		DownSouthEast = 23,
		DownSouthWest = 24,
		DownNorthWest = 25,
	}
}

define_enum! {
	pub enum SupportMatch {
		Ignored = 0,
		Required = 1,
		Disallowed = 2,
	}
}

define_packet! {
	RequiredBlockFaceSupport {
		fixed {
			required block_type_id: i32,
			required tag_index: i32,
			required fluid_id: i32,
			required support: SupportMatch,
			required match_self: SupportMatch,
			required allow_support_propagation: bool,
			required rotate: bool,
		}
		variable {
			opt face_type: String,
			opt self_face_type: String,
			opt block_set_id: String,
			opt filler: Vec<Vector3i>,
		}
	}
}

define_packet! {
	BlockFaceSupport {
		variable {
			opt face_type: String,
			opt filler: Vec<Vector3i>,
		}
	}
}

define_packet! {
	BlockTextures {
		fixed {
			required weight: f32,
		}
		variable {
			opt top: String,
			opt bottom: String,
			opt front: String,
			opt back: String,
			opt left: String,
			opt right: String,
		}
	}
}

define_enum! {
	pub enum ShadingMode {
		Standard = 0,
		Flat = 1,
		Fullbright = 2,
		Reflective = 3,
	}
}

define_enum! {
	pub enum RandomRotation {
		None = 0,
		YawPitchRollStep1 = 1,
		YawStep1 = 2,
		YawStep1XZ = 3,
		YawStep90 = 4,
	}
}

define_enum! {
	pub enum VariantRotation {
		None = 0,
		Wall = 1,
		UpDown = 2,
		Pipe = 3,
		DoublePipe = 4,
		NESW = 5,
		UpDownNESW = 6,
		All = 7,
	}
}

define_packet! {
	Tint {
		top: i32,
		bottom: i32,
		front: i32,
		back: i32,
		left: i32,
		right: i32,
	}
}

define_packet! {
	BlockMovementSettings {
		is_climbable: bool,
		climb_up_speed_multiplier: f32,
		climb_down_speed_multiplier: f32,
		climb_lateral_speed_multiplier: f32,
		is_bouncy: bool,
		bounce_velocity: f32,
		drag: f32,
		friction: f32,
		terminal_velocity_modifier: f32,
		horizontal_speed_multiplier: f32,
		acceleration: f32,
		jump_force_multiplier: f32,
	}
}

define_packet! {
	BlockFlags {
		is_usable: bool,
		is_stackable: bool,
	}
}

define_packet! {
	BlockBreaking {
		fixed {
			required health: f32,
			required quantity: i32,
			required quality: i32,
		}
		variable {
			opt gather_type: String,
			opt item_id: String,
			opt drop_list_id: String,
		}
	}
}

define_packet! {
	Harvesting {
		variable {
			opt item_id: String,
			opt drop_list_id: String,
		}
	}
}

define_packet! {
	SoftBlock {
		fixed {
			required is_weapon_breakable: bool
		}
		variable {
			opt item_id: String,
			opt drop_list_id: String,
		}
	}
}

define_packet! {
	BlockGathering {
		variable {
			opt breaking: BlockBreaking,
			opt harvesting: Harvesting,
			opt soft_block: SoftBlock,
		}
	}
}

define_enum! {
	pub enum BlockPreviewVisibility {
		AlwaysVisible = 0,
		AlwaysHidden = 1,
		Default = 2,
	}
}

define_enum! {
	pub enum BlockPlacementRotationMode {
		FacingPlayer = 0,
		StairFacingPlayer = 1,
		BlockNormal = 2,
		Default = 3,
	}
}

define_packet! {
	BlockPlacementSettings {
		placement_in_empty_blocks: bool,
		preview_visibility: BlockPreviewVisibility,
		rotation_mode: BlockPlacementRotationMode,
		wall_placement_override_block_id: i32,
		floor_placement_override_block_id: i32,
		ceiling_placement_override_block_id: i32,
	}
}

define_packet! {
	ModelDisplay {
		fixed {
			opt(2) translation: Vector3f [pad=12],
			opt(3) rotation: Vector3f [pad=12],
			opt(4) scale: Vector3f [pad=12],
		}
		variable {
			opt(0) node: String,
			opt(1) attach_to: String
		}
	}
}

define_packet! {
	RailPoint {
		fixed {
			opt point: Vector3f [pad=12],
			opt normal: Vector3f [pad=12],
		}
	}
}

define_packet! {
	RailConfig {
		fixed {
			opt points: Vec<RailPoint>
		}
	}
}

define_packet! {
	BenchUpgradingRequirement {
		fixed {
			required time_seconds: f64,
			opt material: Vec<MaterialQuantity>
		}
	}
}

define_packet! {
	BenchTierLevel {
		fixed {
			required crafting_time_reduction_modifier: f64,
			required extra_input_slot: i32,
			required extra_output_slot: i32,
			opt bench_upgrade_requirement: BenchUpgradingRequirement
		}
	}
}

define_packet! {
	Bench {
		fixed {
			opt bench_tier_levels: Vec<BenchTierLevel>,
		}
	}
}

define_enum! {
	pub enum ConnectedBlockRuleSetType {
		Stair = 0,
		Roof = 1
	}
}

define_packet! {
	StairConnectedBlockRuleSet {
		fixed {
			required straight_block_id: i32,
			required corner_left_block_id: i32,
			required corner_right_block_id: i32,
			required inverted_corner_left_block_id: i32,
			required inverted_corner_right_block_id: i32,
			opt material_name: String,
		}
	}
}

define_packet! {
	RoofConnectedBlockRuleSet {
		fixed {
			required topper_block_id: i32,
			required width: i32,
		}
		variable {
			opt regular: StairConnectedBlockRuleSet,
			opt hollow: StairConnectedBlockRuleSet,
			opt material_name: String,
		}
	}
}

define_packet! {
	ConnectedBlockRuleSet {
		fixed {
			required rule_set_type: ConnectedBlockRuleSetType,
		}
		variable {
			opt stair: StairConnectedBlockRuleSet,
			opt roof: RoofConnectedBlockRuleSet,
		}
	}
}

define_packet! {
	BlockType {
		mask_size: 4
		fixed {
			required unknown: bool,
			required draw_type: DrawType,
			required material: BlockMaterial,
			required opacity: Opacity,
			required hitbox: i32,
			required interaction_hitbox: i32,
			required model_scale: f32,
			required looping: bool,
			required max_support_distance: i32,
			required block_supports_required_for: BlockSupportsRequiredForType,
			required requires_alpha_blending: bool,
			required cube_shading_mode: ShadingMode,
			required random_rotation: RandomRotation,
			required variant_rotation: VariantRotation,
			required rotation_yaw_placement_offset: Rotation,
			required block_sound_set_index: i32,
			required ambient_sound_event_index: i32,
			opt(13) particle_color: Color [pad=3],
			opt(14) light: ColorLight [pad=4],
			opt(15) tint: Tint [pad=24],
			opt(16) biome_tint: Tint [pad=24],
			required group: i32,
			opt(19) movement_settings: BlockMovementSettings [pad=42],
			opt(20) flags: BlockFlags [pad=2],
			opt(23) placement_settings: BlockPlacementSettings [pad=16],
			required ignore_support_when_placed: bool,
			required transition_to_tag: i32,
		}
		variable {
			opt(0) item: String,
			opt(1) name: String,
			opt(2) shader_effect: Vec<ShaderType>,
			opt(3) model: String,
			opt(4) model_texture: Vec<ModelTexture>,
			opt(5) model_animation: String,
			opt(6) support: HashMap<BlockNeighbor, Vec<RequiredBlockFaceSupport>>,
			opt(7) supporting: HashMap<BlockNeighbor, Vec<BlockFaceSupport>>,
			opt(8) cube_textures: Vec<BlockTextures>,
			opt(9) cube_side_mask_texture: String,
			opt(10) particles: Vec<ModelParticle>,
			opt(11) block_particle_set_id: String,
			opt(12) block_breaking_decal_id: String,
			opt(17) transition_texture: String,
			opt(18) transition_to_groups: Vec<i32>,
			opt(21) interaction_hint: String,
			opt(22) gathering: BlockGathering,
			opt(24) display: ModelDisplay,
			opt(25) rail: RailConfig,
			opt(26) interactions: HashMap<InteractionType, i32>,
			opt(27) states: HashMap<String, i32>,
			opt(28) tag_indexes: Vec<i32>,
			opt(29) bench: Bench,
			opt(30) connected_block_rule_set: ConnectedBlockRuleSet,
		}
	}
}

define_enum! {
	pub enum EasingType {
		Linear = 0,
		QuadIn = 1,
		QuadOut = 2,
		QuadInOut = 3,
		CubicIn = 4,
		CubicOut = 5,
		CubicInOut = 6,
		QuartIn = 7,
		QuartOut = 8,
		QuartInOut = 9,
		QuintIn = 10,
		QuintOut = 11,
		QuintInOut = 12,
		SineIn = 13,
		SineOut = 14,
		SineInOut = 15,
		ExpoIn = 16,
		ExpoOut = 17,
		ExpoInOut = 18,
		CircIn = 19,
		CircOut = 20,
		CircInOut = 21,
		ElasticIn = 22,
		ElasticOut = 23,
		ElasticInOut = 24,
		BackIn = 25,
		BackOut = 26,
		BackInOut = 27,
		BounceIn = 28,
		BounceOut = 29,
		BounceInOut = 30,
	}
}

define_packet! {
	EasingConfig {
		time: f32,
		easing_type: EasingType,
	}
}

define_enum! {
	pub enum NoiseType {
		Sin = 0,
		Cos = 1,
		PerlinLinear = 2,
		PerlinHermite = 3,
		PerlinQuintic = 4,
		Random = 5,
	}
}

define_packet! {
	ClampConfig {
		range: RangeF,
		normalize: bool
	}
}

define_packet! {
	NoiseConfig {
		fixed {
			required seed: i32,
			required noise_type: NoiseType,
			required frequency: f32,
			required amplitude: f32,
			opt clamp: ClampConfig,
		}
	}
}

define_packet! {
	OffsetNoise {
		variable {
			opt x: Vec<NoiseConfig>,
			opt y: Vec<NoiseConfig>,
			opt z: Vec<NoiseConfig>,
		}
	}
}

define_packet! {
	RotationNoise {
		variable {
			opt pitch: Vec<NoiseConfig>,
			opt yaw: Vec<NoiseConfig>,
			opt roll: Vec<NoiseConfig>,
		}
	}
}

define_packet! {
	CameraShakeConfig {
		fixed {
			required duration: f32,
			required start_time: f32,
			required continuous: bool,
			opt ease_in: EasingConfig [pad=5],
			opt ease_out: EasingConfig [pad=5],
		}
		variable {
			opt offset: OffsetNoise,
			opt rotation: RotationNoise,
		}
	}
}

define_packet! {
	CameraShake {
		variable {
			opt first_person: CameraShakeConfig,
			opt third_person: CameraShakeConfig,
		}
	}
}

define_packet! {
	MovementEffects {
		disable_forward: bool,
		disable_backward: bool,
		disable_left: bool,
		disable_right: bool,
		disable_sprint: bool,
		disable_jump: bool,
		disable_crouch: bool,
	}
}

define_packet! {
	AbilityEffects {
		fixed {
			opt disabled: Vec<InteractionType>
		}
	}
}

define_packet! {
	ApplicationEffects {
		mask_size: 2
		fixed {
			opt(0) entity_bottom_tint: Color [pad=3],
			opt(1) entity_top_tint: Color [pad=3],
			required horizontal_speed_multiplier: f32,
			required sound_event_index_local: i32,
			required sound_event_index_world: i32,
			opt(7) movement_effects: MovementEffects [pad=7],
			required mouse_sensitivity_adjustment_target: f32,
			required mouse_sensitivity_adjustment_duration: f32,
		}
		variable {
			opt(2) entity_animation_id: String,
			opt(3) particles: Vec<ModelParticle>,
			opt(4) first_person_particles: Vec<ModelParticle>,
			opt(5) screen_effect: String,
			opt(6) model_vfx_id: String,
			opt(8) ability_effects: AbilityEffects,
		}
	}
}

define_packet! {
	ModelOverride {
		variable {
			opt model: String,
			opt texture: String,
			opt animation_sets: HashMap<String, AnimationSet>
		}
	}
}

define_enum! {
	pub enum OverlapBehavior {
		Extend = 0,
		Overwrite = 1,
		Ignore = 2,
	}
}

define_enum! {
	pub enum ValueType {
		Percent = 0,
		Absolute = 1
	}
}

define_packet! {
	EntityEffect {
		fixed {
			required world_removal_sound_event_index: i32,
			required local_removal_sound_event_index: i32,
			required duration: f32,
			required infinite: bool,
			required debuff: bool,
			required overlap_behavior: OverlapBehavior,
			required damage_calculator_cooldown: f64,
			required value_type: ValueType,
		}
		variable {
			opt id: String,
			opt name: String,
			opt application_effects: ApplicationEffects,
			opt model_override: ModelOverride,
			opt status_effect_icon: String,
			opt stat_modifiers: HashMap<i32, f32>,
		}
	}
}

define_packet! {
	EntityStatEffects {
		fixed {
			required trigger_at_zero: bool,
			required sound_event_index: i32,
			opt particles: Vec<ModelParticle>
		}
	}
}

define_enum! {
	pub enum EntityStatResetBehavior {
		InitialValue = 0,
		MaxValue = 1,
	}
}

define_packet! {
	EntityStatType {
		fixed {
			required value: f32,
			required min: f32,
			required max: f32,
			required reset_behavior: EntityStatResetBehavior,
		}
		variable {
			opt id: String,
			opt min_value_effects: EntityStatEffects,
			opt max_value_effects: EntityStatEffects,
		}
	}
}

define_enum! {
	pub enum EntityUIType {
		EntityStat = 0,
		CombatText = 1
	}
}

define_packet! {
	RangeVector2f {
		fixed {
			opt x: RangeF [pad=8],
			opt y: RangeF [pad=8],
		}
	}
}

define_enum! {
	pub enum CombatTextEntityUIAnimationEventType {
		Scale = 0,
		Position = 1,
		Opacity = 2
	}
}

define_packet! {
	CombatTextEntityUIComponentAnimationEvent {
		fixed {
			required event_type: CombatTextEntityUIAnimationEventType,
			required start_at: f32,
			required end_at: f32,
			required start_scale: f32,
			required end_scale: f32,
			opt position_offset: Vector2f [pad=8],
			required start_opacity: f32,
			required end_opacity: f32,
		}
	}
}

define_packet! {
	EntityUIComponent {
		fixed {
			required entity_ui_type: EntityUIType,
			opt hitbox_offset: Vector2f [pad=8],
			required unknown: bool,
			required entity_stat_index: i32,
			opt combat_text_random_position_offset_range: RangeVector2f [pad=16],
			required combat_text_viewport_margin: f32,
			required combat_text_duration: f32,
			required combat_text_hit_angle_modifier_strength: f32,
			required combat_text_font_size: f32,
			opt combat_text_color: Color [pad=3],
			opt combat_text_animation_events: Vec<CombatTextEntityUIComponentAnimationEvent> // This has no padding
		}
	}
}

define_packet! {
	FluidParticle {
		fixed {
			opt(1) color: Color [pad=3],
			required scale: f32,
			opt(0) system_id: String
		}
	}
}

define_packet! {
	WorldEnvironment {
		fixed {
			opt(1) water_tint: Color [pad=3],
		}
		variable {
			opt(0) id: String,
			opt(2) fluid_particles: HashMap<i32, FluidParticle>,
			opt(3) tag_indexes: Vec<i32>,
		}
	}
}

define_packet! {
	EqualizerEffect {
		fixed {
			required low_gain: f32,
			required low_cut_off: f32,
			required low_mid_gain: f32,
			required low_mid_center: f32,
			required low_mid_width: f32,
			required high_mid_gain: f32,
			required high_mid_center: f32,
			required high_mid_width: f32,
			required high_gain: f32,
			required high_cut_off: f32,
			opt id: String
		}
	}
}

define_enum! {
	pub enum ItemGridInfoDisplayMode {
		Tooltip = 0,
		Adjacent = 1,
		None = 2
	}
}

define_packet! {
	ItemCategory {
		fixed {
			required order: i32,
			required info_display_mode: ItemGridInfoDisplayMode,
		}
		variable {
			opt id: String,
			opt name: String,
			opt icon: String,
			opt children: Vec<ItemCategory>
		}
	}
}

define_enum! {
	pub enum FluidFog {
		Color = 0,
		ColorLight = 1,
		EnvironmentTint = 2
	}
}

define_packet! {
	NearFar {
		near: f32,
		far: f32
	}
}

define_packet! {
	FluidFXMovementSettings {
		swim_up_speed: f32,
		swim_down_speed: f32,
		sink_speed: f32,
		horizontal_speed_multiplier: f32,
		field_of_view_multiplier: f32,
		entry_velocity_multiplier: f32
	}
}

define_packet! {
	FluidFX {
		fixed {
			required shader: ShaderType,
			required fog_mode: FluidFog,
			opt(1) fog_color: Color [pad=3],
			opt(2) fog_distance: NearFar [pad=8],
			required fog_depth_start: f32,
			required fog_depth_falloff: f32,
			opt(3) color_filter: Color [pad=3],
			required color_saturation: f32,
			required distortion_amplitude: f32,
			required distortion_frequency: f32,
			opt(5) movement_settings: FluidFXMovementSettings [pad=32],
		}
		variable {
			opt(0) id: String,
			opt(4) particle: FluidParticle,
		}
	}
}

define_packet! {
	Fluid {
		fixed {
			required max_fluid_level: i32,
			required requires_alpha_blending: bool,
			required opacity: Opacity,
			opt(3) light: ColorLight [pad=4],
			required fluid_fx_index: i32,
			required block_sound_set_index: i32,
			opt(5) particle_color: Color [pad=3],
		}
		variable {
			opt(0) id: String,
			opt(1) cube_textures: Vec<BlockTextures>,
			opt(2) shader_effect: Vec<ShaderType>,
			opt(4) block_particle_set_id: String,
			opt(6) tag_indexes: Vec<i32>,
		}
	}
}

define_enum! {
	pub enum CollisionType {
		Hard = 0,
		Soft = 1
	}
}

define_packet! {
	HitboxCollisionConfig {
		collision_type: CollisionType,
		soft_collision_offset_ratio: f32
	}
}

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
			opt(5) camera_shake: CameraShake [pad=9],
			opt(6) movement_effects: MovementEffects [pad=7],
			required start_delay: f32,
		}
		variable {
			opt(0) particles: Vec<ModelParticle>,
			opt(1) first_person_particles: Vec<ModelParticle>,
			opt(2) trails: Vec<ModelTrail>,
			opt(3) item_player_animations_id: String,
			opt(4) item_animation_id: String,
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
			opt blocked_by: Vec<InteractionType>,
			opt blocking: Vec<InteractionType>,
			opt interrupted_by: Vec<InteractionType>,
			opt interrupting: Vec<InteractionType>,
		}
	}
}

define_packet! {
	InteractionCamera {
		fixed {
			required time: f32,
			opt position: Vector3f [pad=12],
			opt rotation: DirectionF [pad=12],
		}
	}
}

define_packet! {
	InteractionCameraSettings {
		variable {
			opt first_person: Vec<InteractionCamera>,
			opt third_person: Vec<InteractionCamera>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
		}
	}
}

define_packet! {
	ModifyInventoryInteraction {
		mask_size: 2
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			opt(5) required_game_mode: GameMode,
			required adjust_held_item_quantity: i32,
			required adjust_held_item_durability: i32,
		}
		variable {
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(6) item_to_remove: ItemWithAllMetadata,
			opt(7) item_to_add: ItemWithAllMetadata,
			opt(8) broken_item: String
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
			required next: i32,
			required failed: i32,
			required allow_indefinite_hold: bool,
			required display_progress: bool,
			required cancel_on_other_click: bool,
			required fail_on_damage: bool,
			required mouse_sensitivity_adjustment_target: f32,
			required mouse_sensitivity_adjustment_duration: f32,
			opt(7) charging_delay: ChargingDelay
		}
		variable {
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) charged_next: HashMap<OrderedFloat<f32>, i32>, // f32 can't be a key since NaN might not be equal to NaN due to there being millions of ways to represent it in the IEEE 754 standard, but the java side treats all NaNs as equal so this is a workaround
			opt(6) forks: HashMap<InteractionType, i32>,
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
			opt(1) color: Color [pad=3],
			opt(2) position_offset: Vector3f [pad=12],
			opt(3) rotation_offset: DirectionF [pad=12],
			opt(0) system_id: String
		}
	}
}

define_packet! {
	DamageEffects {
		fixed {
			required sound_event_index: i32
		}
		variable {
			opt model_particles: Vec<ModelParticle>,
			opt world_particles: Vec<WorldParticle>
		}
	}
}

define_packet! {
	WieldingInteraction {
		mask_size: 2
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			required allow_indefinite_hold: bool,
			required display_progress: bool,
			required cancel_on_other_click: bool,
			required fail_on_damage: bool,
			required mouse_sensitivity_adjustment_target: f32,
			required mouse_sensitivity_adjustment_duration: f32,
			opt(7) charging_delay: ChargingDelay,
			required has_modifiers: bool,
			opt(8) angled_wielding: AngledWielding
		}
		variable {
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) charged_next: HashMap<OrderedFloat<f32>, i32>, // f32 can't be a key since NaN might not be equal to NaN due to there being millions of ways to represent it in the IEEE 754 standard, but the java side treats all NaNs as equal so this is a workaround
			opt(6) forks: HashMap<InteractionType, i32>,
			opt(9) blocked_effects: DamageEffects
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) chain_id: String,
			opt(6) chaining_next: Vec<i32>,
			opt(7) flags: HashMap<String, i32>
		}
	}
}

define_packet! {
	ConditionInteraction {
		mask_size: 2
		fixed {
			required wait_for_data_from: WaitForDataFrom,
			required horizontal_speed_multiplier: f32,
			required run_time: f32,
			required cancel_on_item_change: bool,
			required next: i32,
			required failed: i32,
			opt(5) required_game_mode: GameMode,
			opt(6) jumping: bool, // @Nullable Boolean be like
			opt(7) swimming: bool,
			opt(8) crouching: bool,
			opt(9) running: bool,
			opt(10) flying: bool,
		}
		variable {
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) costs: HashMap<i32, f32>
		}
	}
}

define_packet! {
	BlockIdMatcher {
		fixed {
			required tag_index: i32
		}
		variable {
			opt id: String,
			opt state: String
		}
	}
}

define_packet! {
	BlockMatcher {
		fixed {
			required face: BlockFace,
			required static_face: bool,
			opt block: BlockIdMatcher
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) matchers: Vec<BlockMatcher>
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) variable: String
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) block_changes: HashMap<i32, i32>
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) state_changes: HashMap<String, String>
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) refill_fluiids: Vec<i32>
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
		fixed {
			opt matchers: Vec<EntityMatcher>
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) hit_entity_rules: Vec<HitEntity>
		}
	}
}

define_packet! {
	AngledDamage {
		fixed {
			required angle: f64,
			required angle_distance: f64,
			required next: i32,
			opt damage_effects: DamageEffects
		}
	}
}

define_packet! {
	TargetedDamage {
		fixed {
			required index: i32,
			required next: i32,
			opt damage_effects: DamageEffects
		}
	}
}

define_packet! {
	EntityStatOnHit {
		fixed {
			required entity_stat_index: i32,
			required amount: f32,
			required multiplier_per_extra_entities_hit_count: f32,
			opt multipliers_per_entities_hit: Vec<f32>
		}
	}
}

define_packet! {
	DamageEntityInteraction {
		mask_size: 2
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) damage_effects: DamageEffects,
			opt(6) angled_damage: Vec<AngledDamage>,
			opt(7) targeted_damage: HashMap<String, TargetedDamage>,
			opt(8) entity_status_on_hit: Vec<EntityStatOnHit>
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) next: Vec<i32>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) entity_effects: Vec<i32>,
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
			opt direction: Vector3f [pad=12],
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
			opt(5) velocity_config: VelocityConfig [pad=21],
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) forces: Vec<AppliedForce>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) serial_interactions: Vec<i32>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) stat_modifiers: HashMap<i32, f32>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) config_id: String,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt cooldown_id: String,
			opt charge_times: Vec<f32>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) cooldown: InteractionCooldown
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) cooldown: InteractionCooldown
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) cooldown_id: String
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) chain_id: String,
			opt(6) flag: String,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) cooldown_id: String
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) chain_id: String
		}
	}
}

define_packet! {
	DeployableConfig {
		fixed {
			required allow_place_on_walls: bool
		}
		variable {
			opt model: Model,
			opt model_preview: Model
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			// DeployableConfig is too big due to the Models it containts, it causes the this enum variant's size to shoot up to over 1688 without boxing. With boxing it's lower than 200, which is what we want & makes clippy happy.
			opt(5) deployable_config: Box<DeployableConfig>,
			opt(6) costs: HashMap<i32, f32>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
			opt(5) memories_next: HashMap<i32, i32>,
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
			opt(0) effects: InteractionEffects,
			opt(1) settings: HashMap<GameMode, InteractionSettings>,
			opt(2) rules: InteractionRules,
			opt(3) tags: Vec<i32>,
			opt(4) camera: InteractionCameraSettings,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Interaction {
	SimpleBlockInteraction(SimpleBlockInteraction),
	SimpleInteraction(SimpleInteraction),
	PlaceBlockInteraction(PlaceBlockInteraction),
	BreakBlockInteraction(BreakBlockInteraction),
	PickBlockInteraction(PickBlockInteraction),
	UseBlockInteraction(UseBlockInteraction),
	UseEntityInteraction(UseEntityInteraction),
	BuilderToolInteraction(BuilderToolInteraction),
	ModifyInventoryInteraction(ModifyInventoryInteraction),
	ChargingInteraction(ChargingInteraction),
	WieldingInteraction(WieldingInteraction),
	ChainingInteraction(ChainingInteraction),
	ConditionInteraction(ConditionInteraction),
	StatsConditionInteraction(StatsConditionInteraction),
	BlockConditionInteraction(BlockConditionInteraction),
	ReplaceInteraction(ReplaceInteraction),
	ChangeBlockInteraction(ChangeBlockInteraction),
	ChangeStateInteraction(ChangeStateInteraction),
	FirstClickInteraction(FirstClickInteraction),
	RefillContainerInteraction(RefillContainerInteraction),
	SelectInteraction(SelectInteraction),
	DamageEntityInteraction(DamageEntityInteraction),
	RepeatInteraction(RepeatInteraction),
	ParallelInteraction(ParallelInteraction),
	ChangeActiveSlotInteraction(ChangeActiveSlotInteraction),
	EffectConditionInteraction(EffectConditionInteraction),
	ApplyForceInteraction(ApplyForceInteraction),
	ApplyEffectInteraction(ApplyEffectInteraction),
	ClearEntityEffectInteraction(ClearEntityEffectInteraction),
	SerialInteraction(SerialInteraction),
	ChangeStatInteraction(ChangeStatInteraction),
	MovementConditionInteraction(MovementConditionInteraction),
	ProjectileInteraction(ProjectileInteraction),
	RemoveEntityInteraction(RemoveEntityInteraction),
	ResetCooldownInteraction(ResetCooldownInteraction),
	TriggerCooldownInteraction(TriggerCooldownInteraction),
	CooldownConditionInteraction(CooldownConditionInteraction),
	ChainFlagInteraction(ChainFlagInteraction),
	IncrementCooldownInteraction(IncrementCooldownInteraction),
	CancelChainInteraction(CancelChainInteraction),
	RunRootInteraction(RunRootInteraction),
	CameraInteraction(CameraInteraction),
	SpawnDeployableFromRaycastInteraction(SpawnDeployableFromRaycastInteraction),
	MemoriesConditionInteraction(MemoriesConditionInteraction),
	ToggleGliderInteraction(ToggleGliderInteraction),
}

impl HytaleCodec for Interaction {
	fn encode(&self, buf: &mut BytesMut) {
		match self {
			Interaction::SimpleBlockInteraction(interaction) => {
				VarInt(0).encode(buf);
				interaction.encode(buf);
			}
			Interaction::SimpleInteraction(interaction) => {
				VarInt(1).encode(buf);
				interaction.encode(buf);
			}
			Interaction::PlaceBlockInteraction(interaction) => {
				VarInt(2).encode(buf);
				interaction.encode(buf);
			}
			Interaction::BreakBlockInteraction(interaction) => {
				VarInt(3).encode(buf);
				interaction.encode(buf);
			}
			Interaction::PickBlockInteraction(interaction) => {
				VarInt(4).encode(buf);
				interaction.encode(buf);
			}
			Interaction::UseBlockInteraction(interaction) => {
				VarInt(5).encode(buf);
				interaction.encode(buf);
			}
			Interaction::UseEntityInteraction(interaction) => {
				VarInt(6).encode(buf);
				interaction.encode(buf);
			}
			Interaction::BuilderToolInteraction(interaction) => {
				VarInt(7).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ModifyInventoryInteraction(interaction) => {
				VarInt(8).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ChargingInteraction(interaction) => {
				VarInt(9).encode(buf);
				interaction.encode(buf);
			}
			Interaction::WieldingInteraction(interaction) => {
				VarInt(10).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ChainingInteraction(interaction) => {
				VarInt(11).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ConditionInteraction(interaction) => {
				VarInt(12).encode(buf);
				interaction.encode(buf);
			}
			Interaction::StatsConditionInteraction(interaction) => {
				VarInt(13).encode(buf);
				interaction.encode(buf);
			}
			Interaction::BlockConditionInteraction(interaction) => {
				VarInt(14).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ReplaceInteraction(interaction) => {
				VarInt(15).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ChangeBlockInteraction(interaction) => {
				VarInt(16).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ChangeStateInteraction(interaction) => {
				VarInt(17).encode(buf);
				interaction.encode(buf);
			}
			Interaction::FirstClickInteraction(interaction) => {
				VarInt(18).encode(buf);
				interaction.encode(buf);
			}
			Interaction::RefillContainerInteraction(interaction) => {
				VarInt(19).encode(buf);
				interaction.encode(buf);
			}
			Interaction::SelectInteraction(interaction) => {
				VarInt(20).encode(buf);
				interaction.encode(buf);
			}
			Interaction::DamageEntityInteraction(interaction) => {
				VarInt(21).encode(buf);
				interaction.encode(buf);
			}
			Interaction::RepeatInteraction(interaction) => {
				VarInt(22).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ParallelInteraction(interaction) => {
				VarInt(23).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ChangeActiveSlotInteraction(interaction) => {
				VarInt(24).encode(buf);
				interaction.encode(buf);
			}
			Interaction::EffectConditionInteraction(interaction) => {
				VarInt(25).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ApplyForceInteraction(interaction) => {
				VarInt(26).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ApplyEffectInteraction(interaction) => {
				VarInt(27).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ClearEntityEffectInteraction(interaction) => {
				VarInt(28).encode(buf);
				interaction.encode(buf);
			}
			Interaction::SerialInteraction(interaction) => {
				VarInt(29).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ChangeStatInteraction(interaction) => {
				VarInt(30).encode(buf);
				interaction.encode(buf);
			}
			Interaction::MovementConditionInteraction(interaction) => {
				VarInt(31).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ProjectileInteraction(interaction) => {
				VarInt(32).encode(buf);
				interaction.encode(buf);
			}
			Interaction::RemoveEntityInteraction(interaction) => {
				VarInt(33).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ResetCooldownInteraction(interaction) => {
				VarInt(34).encode(buf);
				interaction.encode(buf);
			}
			Interaction::TriggerCooldownInteraction(interaction) => {
				VarInt(35).encode(buf);
				interaction.encode(buf);
			}
			Interaction::CooldownConditionInteraction(interaction) => {
				VarInt(36).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ChainFlagInteraction(interaction) => {
				VarInt(37).encode(buf);
				interaction.encode(buf);
			}
			Interaction::IncrementCooldownInteraction(interaction) => {
				VarInt(38).encode(buf);
				interaction.encode(buf);
			}
			Interaction::CancelChainInteraction(interaction) => {
				VarInt(39).encode(buf);
				interaction.encode(buf);
			}
			Interaction::RunRootInteraction(interaction) => {
				VarInt(40).encode(buf);
				interaction.encode(buf);
			}
			Interaction::CameraInteraction(interaction) => {
				VarInt(41).encode(buf);
				interaction.encode(buf);
			}
			Interaction::SpawnDeployableFromRaycastInteraction(interaction) => {
				VarInt(42).encode(buf);
				interaction.encode(buf);
			}
			Interaction::MemoriesConditionInteraction(interaction) => {
				VarInt(43).encode(buf);
				interaction.encode(buf);
			}
			Interaction::ToggleGliderInteraction(interaction) => {
				VarInt(44).encode(buf);
				interaction.encode(buf);
			}
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let type_id = VarInt::decode(buf)?.0;

		match type_id {
			0 => Ok(Interaction::SimpleBlockInteraction(SimpleBlockInteraction::decode(buf)?)),
			1 => Ok(Interaction::SimpleInteraction(SimpleInteraction::decode(buf)?)),
			2 => Ok(Interaction::PlaceBlockInteraction(PlaceBlockInteraction::decode(buf)?)),
			3 => Ok(Interaction::BreakBlockInteraction(BreakBlockInteraction::decode(buf)?)),
			4 => Ok(Interaction::PickBlockInteraction(PickBlockInteraction::decode(buf)?)),
			5 => Ok(Interaction::UseBlockInteraction(UseBlockInteraction::decode(buf)?)),
			6 => Ok(Interaction::UseEntityInteraction(UseEntityInteraction::decode(buf)?)),
			7 => Ok(Interaction::BuilderToolInteraction(BuilderToolInteraction::decode(buf)?)),
			8 => Ok(Interaction::ModifyInventoryInteraction(ModifyInventoryInteraction::decode(buf)?)),
			9 => Ok(Interaction::ChargingInteraction(ChargingInteraction::decode(buf)?)),
			10 => Ok(Interaction::WieldingInteraction(WieldingInteraction::decode(buf)?)),
			11 => Ok(Interaction::ChainingInteraction(ChainingInteraction::decode(buf)?)),
			12 => Ok(Interaction::ConditionInteraction(ConditionInteraction::decode(buf)?)),
			13 => Ok(Interaction::StatsConditionInteraction(StatsConditionInteraction::decode(buf)?)),
			14 => Ok(Interaction::BlockConditionInteraction(BlockConditionInteraction::decode(buf)?)),
			15 => Ok(Interaction::ReplaceInteraction(ReplaceInteraction::decode(buf)?)),
			16 => Ok(Interaction::ChangeBlockInteraction(ChangeBlockInteraction::decode(buf)?)),
			17 => Ok(Interaction::ChangeStateInteraction(ChangeStateInteraction::decode(buf)?)),
			18 => Ok(Interaction::FirstClickInteraction(FirstClickInteraction::decode(buf)?)),
			19 => Ok(Interaction::RefillContainerInteraction(RefillContainerInteraction::decode(buf)?)),
			20 => Ok(Interaction::SelectInteraction(SelectInteraction::decode(buf)?)),
			21 => Ok(Interaction::DamageEntityInteraction(DamageEntityInteraction::decode(buf)?)),
			22 => Ok(Interaction::RepeatInteraction(RepeatInteraction::decode(buf)?)),
			23 => Ok(Interaction::ParallelInteraction(ParallelInteraction::decode(buf)?)),
			24 => Ok(Interaction::ChangeActiveSlotInteraction(ChangeActiveSlotInteraction::decode(buf)?)),
			25 => Ok(Interaction::EffectConditionInteraction(EffectConditionInteraction::decode(buf)?)),
			26 => Ok(Interaction::ApplyForceInteraction(ApplyForceInteraction::decode(buf)?)),
			27 => Ok(Interaction::ApplyEffectInteraction(ApplyEffectInteraction::decode(buf)?)),
			28 => Ok(Interaction::ClearEntityEffectInteraction(ClearEntityEffectInteraction::decode(buf)?)),
			29 => Ok(Interaction::SerialInteraction(SerialInteraction::decode(buf)?)),
			30 => Ok(Interaction::ChangeStatInteraction(ChangeStatInteraction::decode(buf)?)),
			31 => Ok(Interaction::MovementConditionInteraction(MovementConditionInteraction::decode(buf)?)),
			32 => Ok(Interaction::ProjectileInteraction(ProjectileInteraction::decode(buf)?)),
			33 => Ok(Interaction::RemoveEntityInteraction(RemoveEntityInteraction::decode(buf)?)),
			34 => Ok(Interaction::ResetCooldownInteraction(ResetCooldownInteraction::decode(buf)?)),
			35 => Ok(Interaction::TriggerCooldownInteraction(TriggerCooldownInteraction::decode(buf)?)),
			36 => Ok(Interaction::CooldownConditionInteraction(CooldownConditionInteraction::decode(buf)?)),
			37 => Ok(Interaction::ChainFlagInteraction(ChainFlagInteraction::decode(buf)?)),
			38 => Ok(Interaction::IncrementCooldownInteraction(IncrementCooldownInteraction::decode(buf)?)),
			39 => Ok(Interaction::CancelChainInteraction(CancelChainInteraction::decode(buf)?)),
			40 => Ok(Interaction::RunRootInteraction(RunRootInteraction::decode(buf)?)),
			41 => Ok(Interaction::CameraInteraction(CameraInteraction::decode(buf)?)),
			42 => Ok(Interaction::SpawnDeployableFromRaycastInteraction(SpawnDeployableFromRaycastInteraction::decode(buf)?)),
			43 => Ok(Interaction::MemoriesConditionInteraction(MemoriesConditionInteraction::decode(buf)?)),
			44 => Ok(Interaction::ToggleGliderInteraction(ToggleGliderInteraction::decode(buf)?)),
			_ => Err(PacketError::InvalidEnumVariant(type_id as u8)),
		}
	}
}

define_packet! {
	WiggleWeights {
		x: f32,
		x_deceleration: f32,
		y: f32,
		y_deceleration: f32,
		z: f32,
		z_deceleration: f32,
		roll: f32,
		roll_deceleration: f32,
		pitch: f32,
		pitch_deceleration: f32,
	}
}

define_packet! {
	ItemPullbackConfiguration {
		fixed {
			opt left_offset_override: Vector3f [pad=12],
			opt left_rotation_override: Vector3f [pad=12],
			opt right_offset_override: Vector3f [pad=12],
			opt right_rotation_override: Vector3f [pad=12],
		}
	}
}

define_packet! {
	ItemAnimation {
		fixed {
			required keep_previous_first_person_animation: bool,
			required speed: f32,
			required blending_duration: f32,
			required looping: bool,
			required clips_geometry: bool,
		}
		variable {
			opt third_person: String,
			opt third_person_moving: String,
			opt third_person_face: String,
			opt first_person: String,
			opt first_person_override: String,
		}
	}
}

define_packet! {
	ItemPlayerAnimations {
		fixed {
			opt(2) wigge_weights: WiggleWeights [pad=40],
			opt(4) pullback_config: ItemPullbackConfiguration [pad=49],
			required use_first_person_override: bool
		}
		variable {
			opt id: String,
			opt animations: HashMap<String, ItemAnimation>,
			opt camera: CameraSettings
		}
	}
}

define_packet! {
	ItemQuality {
		fixed {
			opt(6) text_color: Color [pad=3],
			required visible_quality_label: bool,
			required render_special_slot: bool,
			required hide_from_search: bool,
		}
		variable {
			opt item_tooltip_texture: String,
			opt item_tooltip_arrow_texture: String,
			opt slot_texture: String,
			opt block_slot_texture: String,
			opt special_slot_texture: String,
			opt localization_key: String
		}
	}
}

define_packet! {
	ItemReticle {
		fixed {
			required hide_base: bool,
			required duration: f32,
			opt parts: Vec<String>
		}
	}
}

define_enum! {
	pub enum ItemReticleClientEvent {
		OnHit = 0,
		Wielding = 1,
		OnMovementLeft = 2,
		OnMovementRight = 3,
		OnMovementBack = 4,
	}
}

define_packet! {
	ItemReticleConfig {
		variable {
			opt id: String,
			opt base: Vec<String>,
			opt server_events: HashMap<i32, ItemReticle>,
			opt client_events: HashMap<ItemReticleClientEvent, ItemReticle>,
		}
	}
}

define_packet! {
	AssetIconProperties {
		fixed {
			required scale: f32,
			opt translation: Vector2f [pad=8],
			opt rotation: Vector3f [pad=12],
		}
	}
}

define_packet! {
	ItemTranslationProperties {
		variable {
			opt name: String,
			opt description: String,
		}
	}
}

define_packet! {
	ItemResourceType {
		fixed {
			required quantity: i32,
			opt id: String
		}
	}
}

define_packet! {
	ItemToolSpec {
		fixed {
			required power: f32,
			required quality: i32,
			opt gather_type: String
		}
	}
}

define_packet! {
	ItemTool {
		fixed {
			required speed: f32,
			opt specs: Vec<ItemToolSpec>
		}
	}
}

define_packet! {
	ItemWeapon {
		fixed {
			required render_dual_wielded: bool,
		}
		variable {
			opt entity_stats_to_clear: Vec<i32>,
			opt stat_modifiers: HashMap<i32, Vec<Modifier>>
		}
	}
}

define_enum! {
	pub enum ItemArmorSlot {
		Head = 0,
		Chest = 1,
		Hands = 2,
		Legs = 3,
	}
}

define_enum! {
	pub enum Cosmetic {
		Haircut = 0,
		FacialHair = 1,
		Undertop = 2,
		Overtop = 3,
		Pants = 4,
		Overpants = 5,
		Shoes = 6,
		Gloves = 7,
		Cape = 8,
		HeadAccessory = 9,
		FaceAccessory = 10,
		EarAccessory = 11,
		Ear = 12,
	}
}

define_packet! {
	ItemArmor {
		fixed {
			required armor_slot: ItemArmorSlot,
			required base_damage_resistance: i32,
		}
		variable {
			opt cosmetics_to_hide: Vec<Cosmetic>,
			opt stat_modifiers: HashMap<i32, Vec<Modifier>>,
			opt damage_resistance: HashMap<i32, Vec<Modifier>>,
			opt damage_enhancement: HashMap<i32, Vec<Modifier>>,
			opt damage_class_enhancement: HashMap<i32, Vec<Modifier>>,
		}
	}
}

define_packet! {
	ItemGlider {
		terminal_velocity: f32,
		fall_speed_multiplier: f32,
		horizontal_speed_multiplier: f32,
		speed: f32,
	}
}

define_packet! {
	ItemUtility {
		fixed {
			required usable: bool,
			required compatible: bool
		}
		variable {
			opt entity_stats_to_clear: Vec<i32>,
			opt stat_modifiers: HashMap<i32, Vec<Modifier>>
		}
	}
}

define_packet! {
	BlockSelectorToolData {
		durability_loss_on_use: f32
	}
}

define_packet! {
	ItemBuilderToolData {
		variable {
			opt ui: Vec<String>,
			opt tools: Vec<BuilderToolState>
		}
	}
}

define_packet! {
	ItemEntityConfig {
		fixed {
			opt(1) particle_color: Color [pad=3],
			required show_item_particles: bool,
		}
		variable {
			opt(0) particle_system_id: String
		}
	}
}

define_enum! {
	pub enum PrioritySlot {
		Default = 0,
		MainHand = 1,
		OffHand = 2
	}
}

define_packet! {
	InteractionPriority {
		fixed {
			opt values: HashMap<PrioritySlot, i32>
		}
	}
}

define_packet! {
	InteractionConfiguration {
		fixed {
			required display_outlines: bool,
			required debug_outlines: bool,
			required all_entities: bool,
		}
		variable {
			opt use_distance: HashMap<GameMode, f32>,
			opt priorities: HashMap<InteractionType, InteractionPriority>
		}
	}
}

define_packet! {
	ItemAppearanceCondition {
		fixed {
			opt(5) condition: FloatRange [pad=8],
			required condition_value_type: ValueType,
			required local_sound_event_id: i32,
			required world_sound_event_id: i32,
		}
		variable {
			opt(0) particles: Vec<ModelParticle>,
			opt(1) first_person_particles: Vec<ModelParticle>,
			opt(2) model: String,
			opt(3) texture: String,
			opt(4) model_vfx_id: String,
		}
	}
}

define_packet! {
	ItemBase {
		mask_size: 4
		fixed {
			required scale: f32,
			required use_player_animations: bool,
			required max_stack: i32,
			required reticle_index: i32,
			opt(6) icon_properties: AssetIconProperties [pad=25],
			required item_level: i32,
			required quality_index: i32,
			required consumable: bool,
			required variant: bool,
			required block_id: i32,
			opt(12) glider_config: ItemGlider [pad=16],
			opt(14) block_selector_tool: BlockSelectorToolData [pad=4],
			opt(22) light: ColorLight [pad=8],
			required durability: f64,
			required sound_event_index: i32,
			required item_sound_set_index: i32,
			opt(30) pullback_config: ItemPullbackConfiguration [pad=49],
			required clips_geometry: bool,
			required render_deployable_preview: bool,
		}
		variable {
			opt(0) id: String,
			opt(1) model: String,
			opt(2) texture: String,
			opt(3) animation: String,
			opt(4) player_animations_id: String,
			opt(5) icon: String,
			opt(7) appearance_conditions: Vec<ItemAppearanceCondition>,
			opt(8) resource_types: Vec<ItemResourceType>,
			opt(9) tool: ItemTool,
			opt(10) weapon: ItemWeapon,
			opt(11) armor: ItemArmor,
			opt(13) utility: ItemUtility,
			opt(15) builder_tool_data: ItemBuilderToolData,
			opt(16) item_entity: ItemEntityConfig,
			opt(17) set: String,
			opt(18) categories: Vec<String>,
			opt(19) particles: Vec<ModelParticle>,
			opt(20) first_person_particles: Vec<ModelParticle>,
			opt(21) trails: Vec<ModelTrail>,
			opt(23) interactions: HashMap<InteractionType, i32>,
			opt(24) interaction_vars: HashMap<String, i32>,
			opt(25) interaction_config: InteractionConfiguration,
			opt(26) dropped_item_animation: String,
			opt(27) tag_indexes: Vec<i32>,
			opt(28) item_appearance_conditions: HashMap<i32, Vec<ItemAppearanceCondition>>,
			opt(29) display_entity_stats_hud: Vec<i32>,
		}
	}
}

define_enum! {
	pub enum ItemSoundEvent {
		Drag = 0,
		Drop = 1
	}
}

define_packet! {
	ItemSoundSet {
		variable {
			opt id: String,
			opt sound_event_indices: HashMap<ItemSoundEvent, i32>
		}
	}
}

define_enum! {
	pub enum SwitchTo {
		Disappear = 0,
		PostColor = 1,
		Distortion = 2,
		Transparency = 3
	}
}

define_enum! {
	pub enum EffectDirection {
		None = 0,
		BottomUp = 1,
		TopDown = 2,
		ToCenter = 3,
		FromCenter = 4,
	}
}

define_enum! {
	pub enum LoopOption {
		PlayOnce = 0,
		Loop = 1,
		LoopMirror = 2,
	}
}

define_enum! {
	pub enum CurveType {
		Linear = 0,
		QuartIn = 1,
		QuartOut = 2,
		QuartInOut = 3,
	}
}

define_packet! {
	ModelVFX {
		fixed {
			required switch_to: SwitchTo,
			required effect_direction: EffectDirection,
			required animation_duration: f32,
			opt(1) animation_range: Vector2f [pad=8],
			required loop_option: LoopOption,
			required curve_type: CurveType,
			opt(2) highlight_color: Color [pad=3],
			required highlight_thickness: f32,
			required use_bloom_on_highlight: bool,
			required use_progessive_highlight: bool,
			opt(3) noise_scale: Vector2f [pad=8],
			opt(4) noise_scroll_speed: Vector2f [pad=8],
			opt(5) post_color: Color [pad=3],
			required post_color_opacity: f32,
			opt(0) id: String
		}

	}
}

define_enum! {
	pub enum EmitShape {
		Sphere = 0,
		Cube = 1
	}
}

define_packet! {
	RangeVector3f {
		fixed {
			opt x: RangeF [pad=8],
			opt y: RangeF [pad=8],
			opt z: RangeF [pad=8],
		}
	}
}

define_packet! {
	InitialVelocity {
		fixed {
			opt yaw: RangeF [pad=8],
			opt pitch: RangeF [pad=8],
			opt speed: RangeF [pad=8],
		}
	}
}

define_enum! {
	pub enum ParticleRotationInfluence {
		None = 0,
		Billboard = 1,
		BillboardY = 2,
		BillboardVelocity = 3,
		Velocity = 4,
	}
}

define_enum! {
	pub enum ParticleCollisionBlockType {
		None = 0,
		Air = 1,
		Solid = 2,
		All = 3
	}
}

define_enum! {
	pub enum ParticleCollisionAction {
		Expire = 0,
		LastFrame = 1,
		Linger = 2
	}
}

define_packet! {
	ParticleCollision {
		block_type: ParticleCollisionBlockType,
		action: ParticleCollisionAction,
		particle_rotation_influence: ParticleRotationInfluence,
	}
}

define_enum! {
	pub enum FXRenderMode {
		BlendLinear = 0,
		BlendAdd = 1,
		Erosion = 2,
		Distortion = 3,
	}
}

define_enum! {
	pub enum UVMotionCurveType {
		Constant = 0,
		IncreaseLinear = 1,
		IncreaseQuartIn = 2,
		IncreaseQuartInOut = 3,
		IncreaseQuartOut = 4,
		DecreaseLinear = 5,
		DecreaseQuartIn = 6,
		DecreaseQuartInOut = 7,
		DecreaseQuartOut = 8,
	}
}

define_packet! {
	UVMotion {
		fixed {
			required add_random_uv_offset: bool,
			required speed_x: f32,
			required speed_y: f32,
			required scale: f32,
			required strength: f32,
			required strength_curve_type: UVMotionCurveType,
			opt texture: String,
		}
	}
}

define_packet! {
	ParticleAttractor {
		fixed {
			opt position: Vector3f [pad=12],
			opt radial_axis: Vector3f [pad=12],
			required trail_position_multiplier: f32,
			required radius: f32,
			required radial_acceleration: f32,
			required radial_tangent_acceleration: f32,
			opt linear_acceleration: Vector3f [pad=12],
			required radial_impulse: f32,
			required radial_tangent_impulse: f32,
			opt linear_impulse: Vector3f [pad=12],
			opt damping_multiplier: Vector3f [pad=12],
		}
	}
}

define_packet! {
	IntersectionHighlight {
		fixed {
			required highlight_threshold: f32,
			opt highlight_color: Color [pad=3],
		}
	}
}

define_packet! {
	Size {
		width: i32,
		height: i32,
	}
}

define_enum! {
	pub enum ParticleUVOption {
		None = 0,
		RandomFlipU = 1,
		RandomFlipV = 2,
		RandomFlipUV = 3,
		FlipU = 4,
		FlipV = 5,
		FlipUV = 6,
	}
}

define_enum! {
	pub enum ParticleScaleRatioConstraint {
		OneToOne = 0,
		Preserved = 1,
		None = 2
	}
}

define_enum! {
	pub enum SoftParticle {
		Enable = 0,
		Disable = 1,
		Require = 2
	}
}

define_packet! {
	ParticleAnimationFrame {
		fixed {
			opt frame_index: RangeI [pad=8],
			opt scale: RangeVector2f [pad=18],
			opt rotation: RangeVector3f [pad=25],
			opt color: Color [pad=3],
			required opacity: f32,
		}
	}
}

define_packet! {
	Particle {
		fixed {
			opt(1) frame_size: Size [pad=8],
			required uv_option: ParticleUVOption,
			required scale_ratio_constraint: ParticleScaleRatioConstraint,
			required soft_particles: SoftParticle,
			required soft_particles_fade_factor: f32,
			required use_sprite_blending: bool,
			opt(2) initial_animation_frame: ParticleAnimationFrame [pad=58],
			opt(3) collision_animation_frame: ParticleAnimationFrame [pad=58],
		}
		variable {
			opt(0) texture_path: String,
			opt(4) animation_frames: HashMap<i32, ParticleAnimationFrame>,
		}
	}
}

define_packet! {
	ParticleSpawner {
		mask_size: 2
		fixed {
			required shape: EmitShape,
			opt(2) emit_offset: RangeVector3f [pad=25],
			required camera_offset: f32,
			required use_emit_direction: bool,
			required life_span: f32,
			opt(3) spawn_rate: RangeF [pad=8],
			required spawn_burst: bool,
			opt(4) wave_delay: RangeF [pad=8],
			opt(5) total_particles: RangeI [pad=8],
			required max_concurrent_particles: i32,
			opt(6) initial_velocity: InitialVelocity [pad=25],
			required velocity_stretch_multiplier: f32,
			required particle_rotation_influence: ParticleRotationInfluence,
			required particle_rotate_with_spawner: bool,
			required is_low_res: bool,
			required trail_spawner_position_multiplier: f32,
			required trail_spawner_rotation_multiplier: f32,
			opt(7) particle_collision: ParticleCollision [pad=3],
			required render_mode: FXRenderMode,
			required light_influence: f32,
			required linear_filtering: bool,
			opt(8) particle_life_span: RangeF [pad=8],
			opt(11) intersection_highlight: IntersectionHighlight [pad=24],
		}
		variable {
			opt(0) id: String,
			opt(1) particle: Particle,
			opt(9) uv_motion: UVMotion,
			opt(10) attractors: Vec<ParticleAttractor>,
		}

	}
}

define_packet! {
	ParticleSpawnerGroup {
		mask_size: 2
		fixed {
			opt(1) position_offset: Vector3f [pad=12],
			opt(2) rotation_offset: Vector3f [pad=12],
			required fixed_rotation: bool,
			required start_delay: f32,
			opt(3) spawn_rate: RangeF [pad=8],
			opt(4) wave_delay: RangeF [pad=8],
			required total_spawners: i32,
			required max_concurrent: i32,
			opt(5) initial_velocity: InitialVelocity [pad=25],
			opt(6) emit_offset: RangeVector3f [pad=25],
			opt(7) life_span: RangeF [pad=8],
		}
		variable {
			opt(0) spawner_id: String,
			opt(8) attractors: Vec<ParticleAttractor>,
		}
	}
}

define_packet! {
	ParticleSystem {
		fixed {
			required life_span: f32,
			required cull_distance: f32,
			required bounding_radius: f32,
			required is_important: bool,
		}
		variable {
			opt id: String,
			opt spawners: Vec<ParticleSpawnerGroup>,
		}
	}
}

define_enum! {
	pub enum PhysicsType {
		Standard = 0, // Braindead enum
	}
}

define_enum! {
	pub enum RotationMode {
		None = 0,
		Velocity = 1,
		VelocityDamped = 2,
		VelocityRoll = 3
	}
}

define_packet! {
	PhysicsConfig {
		physics_type: PhysicsType,
		density: f64,
		gravity: f64,
		bounciness: f64,
		bounce_count: i32,
		bounce_limit: f64,
		sticks_vertically: bool,
		compute_yaw: bool,
		compute_pitch: bool,
		rotation_mode: RotationMode,
		move_out_of_solid_speed: f64,
		terminal_velocity_air: f64,
		density_air: f64,
		terminal_velocity_water: f64,
		density_water: f64,
		hit_water_impulse_loss: f64,
		rotation_force: f64,
		speed_rotation_factor: f32,
		swimming_damping_factor: f64,
		allow_rolling: bool,
		rolling_friction_factor: f64,
		rolling_speed: f32,
	}
}

define_packet! {
	ProjectileConfig {
		fixed {
			opt(0) physics_config: PhysicsConfig [pad=122],
			required launch_force: f64,
			opt(2) spawn_offset: Vector3f [pad=12],
			opt(3) rotation_offset: DirectionF [pad=12],
			required launch_local_sound_event_index: i32,
			required projectile_sound_event_index: i32,
		}
		variable {
			opt(1) model: Model,
			opt(4) interactions: HashMap<InteractionType, i32>,
		}
	}
}

define_packet! {
	RepulsionConfig {
		radius: f32,
		force: RangeF
	}
}

define_packet! {
	ResourceType {
		variable {
			opt id: String,
			opt icon: String
		}
	}
}

define_packet! {
	ReverbEffect {
		fixed {
			required dry_gain: f32,
			required modal_density: f32,
			required diffusion: f32,
			required gain: f32,
			required high_frequency_gain: f32,
			required decay_time: f32,
			required high_frequency_decay_ratio: f32,
			required reflection_gain: f32,
			required reflection_delay: f32,
			required late_reverb_gain: f32,
			required late_reverb_delay: f32,
			required room_rolloff_factor: f32,
			required air_absorption_high_frequency_gain: f32,
			required limit_decay_high_frequency: bool,
			opt id: String
		}
	}
}

define_packet! {
	RootInteractionSettings {
		fixed {
			required allow_skip_chain_on_click: bool,
			opt cooldown: InteractionCooldown
		}
	}
}

define_packet! {
	RootInteraction {
		fixed {
			required click_queuing_timeout: f32,
			required require_new_click: bool,
		}
		variable {
			opt id: String,
			opt interactions: Vec<i32>,
			opt cooldown: InteractionCooldown,
			opt settings: HashMap<GameMode, RootInteractionSettings>,
			opt rules: InteractionRules,
			opt tags: Vec<i32>,
		}
	}
}

define_packet! {
	SoundEventLayerRandomSettings {
		volume: RangeF,
		pitch: RangeF,
		max_start_offset: f32
	}
}

define_packet! {
	SoundEventLayer {
		fixed {
			required volume: f32,
			required start_delay: f32,
			required looping: bool,
			required probability: i32,
			required probability_reroll_delay: f32,
			required round_robin_history_size: i32,
			opt random_settings: SoundEventLayerRandomSettings [pad=20],
			opt files: Vec<String>
		}
	}
}

define_packet! {
	SoundEvent {
		fixed {
			required volume: f32,
			required pitch: f32,
			required music_ducking_volume: f32,
			required ambient_ducking_volume: f32,
			required max_instance: i32,
			required prevent_sound_interruption: bool,
			required start_attenuation_distance: f32,
			required max_distance: f32,
			required audio_category: i32,
		}
		variable {
			opt id: String,
			opt layers: Vec<SoundEventLayer>,
		}
	}
}

define_packet! {
	SoundSet {
		fixed {
			required category: SoundCategory
		}
		variable {
			opt id: String,
			opt sounds: HashMap<String, i32>,
		}
	}
}

define_enum! {
	pub enum TagPatternType {
		Equals = 0,
		And = 1,
		Or = 2,
		Not = 3
	}
}

define_packet! {
	TagPattern {
		fixed {
			required pattern_type: TagPatternType,
			required tag_index: i32,
		}
		variable {
			opt operands: Vec<TagPattern>,
			opt not: Box<TagPattern>
		}
	}
}

define_packet! {
	Edge {
		fixed {
			opt color: ColorAlpha [pad=4],
			required width: f32
		}
	}
}

define_packet! {
	Trail {
		fixed {
			required life_span: i32,
			required roll: f32,
			opt(2) start: Edge [pad=9],
			opt(3) end: Edge [pad=9],
			required light_influence: f32,
			required render_mode: FXRenderMode,
			opt(4) intersection_highlight: IntersectionHighlight [pad=24],
			required smooth: bool,
			opt(5) frame_size: Size [pad=8],
			opt(6) frame_range: RangeI [pad=8],
			required frame_life_span: i32,
		}
		variable {
			opt(0) id: String,
			opt(1) texture: String
		}
	}
}

define_enum! {
	pub enum MovementType {
		None = 0,
		Idle = 1,
		Crouching = 2,
		Walking = 3,
		Running = 4,
		Sprinting = 5,
		Climbing = 6,
		Swimming = 7,
		Flying = 8,
		Sliding = 9,
		Rolling = 10,
		Mounting = 11,
		SprintMounting = 12,
	}
}

define_packet! {
	ViewBobbing {
		fixed {
			opt first_person: CameraShakeConfig
		}
	}
}

define_packet! {
	FogOptions {
		ignore_fog_limits: bool,
		effective_view_distance_multiplier: f32,
		fog_far_view_distance: f32,
		fog_height_camera_offset: f32,
		fog_height_camera_overriden: bool,
		fog_height_camera_fixed: f32,
	}
}

define_packet! {
	Cloud {
		variable {
			opt texture: String,
			opt speeds: HashMap<OrderedFloat<f32>, f32>,
			opt colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
		}
	}
}

define_packet! {
	WeatherParticle {
		fixed {
			opt(1) color: Color [pad=3],
			required scale: f32,
			required is_overground_only: bool,
			required position_offset_multiplier: f32,
			opt(0) system_id: String,
		}
	}
}

define_packet! {
	Weather {
		mask_size: 4
		fixed {
			opt(24) fog: NearFar [pad=8],
			opt(25) fog_options: FogOptions [pad=18]
		}
		variable {
			opt(0) id: String,
			opt(1) tag_indexes: Vec<i32>,
			opt(2) stars: String,
			opt(3) moons: HashMap<i32, String>,
			opt(4) clouds: Vec<Cloud>,
			opt(5) sunlight_damping_multiplier: HashMap<OrderedFloat<f32>, f32>,
			opt(6) sunlight_colors: HashMap<OrderedFloat<f32>, Color>,
			opt(7) sky_top_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(8) sky_bottom_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(9) sky_sunset_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(10) sun_colors:  HashMap<OrderedFloat<f32>, Color>,
			opt(11) sun_scales: HashMap<OrderedFloat<f32>, f32>,
			opt(12) sun_glow_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(13) moon_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(14) moon_scales: HashMap<OrderedFloat<f32>, f32>,
			opt(15) moon_glow_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(16) fog_colors: HashMap<OrderedFloat<f32>, Color>,
			opt(17) fog_height_falloffs: HashMap<OrderedFloat<f32>, f32>,
			opt(18) fog_densities: HashMap<OrderedFloat<f32>, f32>,
			opt(19) screen_effect: String,
			opt(20) screen_effect_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(21) color_filters: HashMap<OrderedFloat<f32>, Color>,
			opt(22) water_tints: HashMap<OrderedFloat<f32>, Color>,
			opt(23) particle: WeatherParticle,
		}
	}
}

// Helper struct so we can check compression before decoding
pub struct PacketInfo {
	pub compressed: bool,
}

macro_rules! packet_enum {
    (
        // Syntax: ID => Variant(Type) [compressed?]
        $( $id:literal => $variant:ident($module:ident::$st:ident) $( [ $compressed:tt ] )? ),* $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub enum Packet {
            $( $variant($module::$st), )*
            Unknown(i32, Bytes),
        }

        impl Packet {
            pub fn id(&self) -> i32 {
                match self {
                    $( Packet::$variant(_) => $id, )*
                    Packet::Unknown(id, _) => *id,
                }
            }

            pub fn is_compressed(&self) -> bool {
                match self {
                    // If [compressed] is present, return true, else false
                    $( Packet::$variant(_) => {
                        0 $( + is_compressed_helper!($compressed) )? == 1
                    }, )*
                    _ => false
                }
            }

            pub fn encode(&self, buf: &mut BytesMut) {
                match self {
                    $( Packet::$variant(pkt) => pkt.encode(buf), )*
                    Packet::Unknown(_, data) => buf.extend_from_slice(data),
                }
            }

            pub fn decode(id: i32, buf: &mut impl Buf) -> PacketResult<Self> {
                match id {
                    $(
                        $id => {
                            let pkt = $module::$st::decode(buf)?;
                            Ok(Packet::$variant(pkt))
                        }
                    )*
                    _ => {
                        Ok(Packet::Unknown(id, buf.copy_to_bytes(buf.remaining())))
                    }
                }
            }
        }

        // Helper to lookup compression BEFORE decoding (needed for reading frames)
        pub fn is_id_compressed(id: i32) -> bool {
            match id {
                 $(
                    $id => {
                        0 $( + is_compressed_helper!($compressed) )? == 1
                    },
                 )*
                 _ => false
            }
        }

        // Auto-impl From<Struct> for Packet
        $(
            impl From<$module::$st> for Packet {
                fn from(p: $module::$st) -> Self {
                    Packet::$variant(p)
                }
            }
        )*
    };
}

// Helper macro to detect presence of the token
macro_rules! is_compressed_helper {
	(compressed) => {
		1
	};
}

packet_enum! {
	// Connection
	0 => Connect(connection::Connect),
	1 => Disconnect(connection::Disconnect),
	2 => Ping(connection::Ping),
	3 => Pong(connection::Pong),

	// Auth
	10 => Status(auth::Status),
	11 => AuthGrant(auth::AuthGrant),
	12 => AuthToken(auth::AuthToken),
	13 => ServerAuthToken(auth::ServerAuthToken),
	14 => ConnectAccept(auth::ConnectAccept),
	15 => PasswordResponse(auth::PasswordResponse),
	16 => PasswordAccepted(auth::PasswordAccepted),
	17 => PasswordRejected(auth::PasswordRejected),
	18 => ClientReferral(auth::ClientReferral),

	// Setup
	20 => WorldSettings(setup::WorldSettings) [compressed],
	21 => WorldLoadProgress(setup::WorldLoadProgress),
	22 => WorldLoadFinished(setup::WorldLoadFinished),
	23 => RequestAssets(setup::RequestAssets) [compressed],
	24 => AssetInitialize(setup::AssetInitialize),
	25 => AssetPart(setup::AssetPart) [compressed],
	26 => AssetFinalize(setup::AssetFinalize),
	27 => RemoveAssets(setup::RemoveAssets),
	28 => RequestCommonAssetsRebuild(setup::RequestCommonAssetsRebuild),
	29 => SetUpdateRate(setup::SetUpdateRate),
	30 => SetTimeDilation(setup::SetTimeDilation),
	31 => UpdateFeatures(setup::UpdateFeatures),
	32 => ViewRadius(setup::ViewRadius),
	33 => PlayerOptions(setup::PlayerOptions),
	34 => ServerTags(setup::ServerTags),

	// Assets
	40 => UpdateBlockTypes(assets::UpdateBlockTypes) [compressed],
	41 => UpdateBlockHitboxes(assets::UpdateBlockHitboxes) [compressed],
	42 => UpdateBlockSoundSets(assets::UpdateBlockSoundSets) [compressed],
	43 => UpdateItemSoundSets(assets::UpdateItemSoundSets) [compressed],
	44 => UpdateBlockParticleSets(assets::UpdateBlockParticleSets) [compressed],
	45 => UpdateBlockBreakingDecals(assets::UpdateBlockBreakingDecals) [compressed],
	46 => UpdateBlockSets(assets::UpdateBlockSets) [compressed],
	47 => UpdateWeathers(assets::UpdateWeathers) [compressed],
	48 => UpdateTrails(assets::UpdateTrails) [compressed],
	49 => UpdateParticleSystems(assets::UpdateParticleSystems) [compressed],
	50 => UpdateParticleSpawners(assets::UpdateParticleSpawners) [compressed],
	51 => UpdateEntityEffects(assets::UpdateEntityEffects) [compressed],
	52 => UpdateItemPlayerAnimations(assets::UpdateItemPlayerAnimations) [compressed],
	53 => UpdateModelvfxs(assets::UpdateModelVFXs) [compressed],
	54 => UpdateItems(assets::UpdateItems) [compressed],
	55 => UpdateItemQualities(assets::UpdateItemQualities) [compressed],
	56 => UpdateItemCategories(assets::UpdateItemCategories) [compressed],
	57 => UpdateItemReticles(assets::UpdateItemReticles) [compressed],
	58 => UpdateFieldcraftCategories(assets::UpdateFieldcraftCategories) [compressed],
	59 => UpdateResourceTypes(assets::UpdateResourceTypes) [compressed],
	60 => UpdateRecipes(assets::UpdateRecipes) [compressed],
	61 => UpdateEnvironments(assets::UpdateEnvironments) [compressed],
	62 => UpdateAmbienceFX(assets::UpdateAmbienceFX) [compressed],
	63 => UpdateFluidFX(assets::UpdateFluidFX) [compressed],
	64 => UpdateTranslations(assets::UpdateTranslations) [compressed],
	65 => UpdateSoundEvents(assets::UpdateSoundEvents) [compressed],
	66 => UpdateInteractions(assets::UpdateInteractions) [compressed],
	67 => UpdateRootInteractions(assets::UpdateRootInteractions) [compressed],
	68 => UpdateUnarmedInteractions(assets::UpdateUnarmedInteractions) [compressed],
	69 => TrackOrUpdateObjective(assets::TrackOrUpdateObjective),
	70 => UntrackObjective(assets::UntrackObjective),
	71 => UpdateObjectiveTask(assets::UpdateObjectiveTask),
	72 => UpdateEntityStatTypes(assets::UpdateEntityStatTypes) [compressed],
	73 => UpdateEntityUIComponents(assets::UpdateEntityUIComponents) [compressed],
	74 => UpdateHitboxCollisionConfig(assets::UpdateHitboxCollisionConfig) [compressed],
	75 => UpdateRepulsionConfig(assets::UpdateRepulsionConfig) [compressed],
	76 => UpdateViewBobbing(assets::UpdateViewBobbing) [compressed],
	77 => UpdateCameraShake(assets::UpdateCameraShake) [compressed],
	78 => UpdateBlockGroups(assets::UpdateBlockGroups) [compressed],
	79 => UpdateSoundSets(assets::UpdateSoundSets) [compressed],
	80 => UpdateAudioCategories(assets::UpdateAudioCategories) [compressed],
	81 => UpdateReverbEffects(assets::UpdateReverbEffects) [compressed],
	82 => UpdateEqualizerEffects(assets::UpdateEqualizerEffects) [compressed],
	83 => UpdateFluids(assets::UpdateFluids) [compressed],
	84 => UpdateTagPatterns(assets::UpdateTagPatterns) [compressed],
	85 => UpdateProjectileConfigs(assets::UpdateProjectileConfigs) [compressed],

	// Player
	100 => SetClientId(player::SetClientId),
	101 => SetGameMode(player::SetGameMode),
	102 => SetMovementStates(player::SetMovementStates),
	103 => SetBlockPlacementOverride(player::SetBlockPlacementOverride),
	104 => JoinWorld(player::JoinWorld),
	105 => ClientReady(player::ClientReady),
	106 => LoadHotbar(player::LoadHotbar),
	107 => SaveHotbar(player::SaveHotbar),
	108 => ClientMovement(player::ClientMovement),
	109 => ClientTeleport(player::ClientTeleport),
	110 => UpdateMovementSettings(player::UpdateMovementSettings),
	111 => MouseInteraction(player::MouseInteraction),
	112 => DamageInfo(player::DamageInfo),
	113 => ReticleEvent(player::ReticleEvent),
	114 => DisplayDebug(player::DisplayDebug),
	115 => ClearDebugShapes(player::ClearDebugShapes),
	116 => SyncPlayerPreferences(player::SyncPlayerPreferences),
	117 => ClientPlaceBlock(player::ClientPlaceBlock),
	118 => UpdateMemoriesFeatureStatus(player::UpdateMemoriesFeatureStatus),
	119 => RemoveMapMarker(player::RemoveMapMarker),

	// World
	131 => SetChunk(world::SetChunk) [compressed],
	132 => SetChunkHeightmap(world::SetChunkHeightmap) [compressed],
	133 => SetChunkTintmap(world::SetChunkTintmap) [compressed],
	134 => SetChunkEnvironments(world::SetChunkEnvironments) [compressed],
	135 => UnloadChunk(world::UnloadChunk),
	136 => SetFluids(world::SetFluids) [compressed], // There is a gap here even though it's still the same section
	140 => ServerSetBlock(world::ServerSetBlock),
	141 => ServerSetBlocks(world::ServerSetBlocks),
	142 => ServerSetFluid(world::ServerSetFluid),
	143 => ServerSetFluids(world::ServerSetFluids),
	144 => UpdateBlockDamage(world::UpdateBlockDamage),
	145 => UpdateTimeSettings(world::UpdateTimeSettings),
	146 => UpdateTime(world::UpdateTime),
	147 => UpdateEditorTimeOverride(world::UpdateEditorTimeOverride),
	148 => ClearEditorTimeOverride(world::ClearEditorTimeOverride),
	149 => UpdateWeather(world::UpdateWeather),
	150 => UpdateEditorWeatherOverride(world::UpdateEditorWeatherOverride),
	151 => UpdateEnvironmentMusic(world::UpdateEnvironmentMusic),
	152 => SpawnParticleSystem(world::SpawnParticleSystem),
	153 => SpawnBlockParticleSystem(world::SpawnBlockParticleSystem),
	154 => PlaySoundEvent2D(world::PlaySoundEvent2D),
	155 => PlaySoundEvent3D(world::PlaySoundEvent3D),
	156 => PlaySoundEventEntity(world::PlaySoundEventEntity),
	157 => UpdateSleepState(world::UpdateSleepState),
	158 => SetPaused(world::SetPaused),
	159 => ServerSetPaused(world::ServerSetPaused),

	// Entities
	160 => SetEntitySeed(entities::SetEntitySeed),
	161 => EntityUpdates(entities::EntityUpdates) [compressed],
	162 => PlayAnimation(entities::PlayAnimation),
	163 => ChangeVelocity(entities::ChangeVelocity),
	164 => ApplyKnockback(entities::ApplyKnockback),
	165 => SpawnModelParticles(entities::SpawnModelParticles),
	166 => MountMovement(entities::MountMovement),

	// Inventory
	170 => UpdatePlayerInventory(inventory::UpdatePlayerInventory) [compressed],
	171 => SetCreativeItem(inventory::SetCreativeItem),
	172 => DropCreativeItem(inventory::DropCreativeItem),
	173 => SmartGiveCreativeItem(inventory::SmartGiveCreativeItem),
	174 => DropItemStack(inventory::DropItemStack),
	175 => MoveItemStack(inventory::MoveItemStack),
	176 => SmartMoveItemStack(inventory::SmartMoveItemStack),
	177 => SetActiveSlot(inventory::SetActiveSlot),
	178 => SwitchHotbarBlockSet(inventory::SwitchHotbarBlockSet),
	179 => InventoryAction(inventory::InventoryAction),

	// Window
	200 => OpenWindow(window::OpenWindow) [compressed],
	201 => UpdateWindow(window::UpdateWindow) [compressed],
	202 => CloseWindow(window::CloseWindow),
	203 => SendWindowAction(window::SendWindowAction),
	204 => ClientOpenWindow(window::ClientOpenWindow),

	// Interface
	210 => ServerMessage(interface::ServerMessage),
	211 => ChatMessage(interface::ChatMessage),
	212 => Notification(interface::Notification),
	213 => KillFeedMessage(interface::KillFeedMessage),
	214 => ShowEventTitle(interface::ShowEventTitle),
	215 => HideEventTitle(interface::HideEventTitle),
	216 => SetPage(interface::SetPage),
	217 => CustomHud(interface::CustomHud) [compressed],
	218 => CustomPage(interface::CustomPage) [compressed],
	219 => CustomPageEvent(interface::CustomPageEvent),
	222 => EditorBlocksChange(interface::EditorBlocksChange) [compressed],
	223 => ServerInfo(interface::ServerInfo),
	224 => AddToServerPlayerList(interface::AddToServerPlayerList),
	225 => RemoveFromServerPlayerList(interface::RemoveFromServerPlayerList),
	226 => UpdateServerPlayerList(interface::UpdateServerPlayerList),
	227 => UpdateServerPlayerListPing(interface::UpdateServerPlayerListPing),
	228 => UpdateKnownRecipes(interface::UpdateKnownRecipes),
	229 => UpdatePortal(interface::UpdatePortal),
	230 => UpdateVisibleHudComponents(interface::UpdateVisibleHudComponents),
	231 => ResetUserInterfaceState(interface::ResetUserInterfaceState),
	232 => UpdateLanguage(interface::UpdateLanguage),
	233 => WorldSavingStatus(interface::WorldSavingStatus),
	234 => OpenChatWithCommand(interface::OpenChatWithCommand),

	// World Map
	240 => UpdateWorldMapSettings(worldmap::UpdateWorldMapSettings),
	241 => UpdateWorldMap(worldmap::UpdateWorldMap) [compressed],
	242 => ClearWorldMap(worldmap::ClearWorldMap),
	243 => UpdateWorldMapVisible(worldmap::UpdateWorldMapVisible),
	244 => TeleportToWorldMapMarker(worldmap::TeleportToWorldMapMarker),
	245 => TeleportToWorldMapPosition(worldmap::TeleportToWorldMapPosition),

	// Server Access
	250 => RequestServerAccess(serveraccess::RequestServerAccess),
	251 => UpdateServerAccess(serveraccess::UpdateServerAccess),
	252 => SetServerAccess(serveraccess::SetServerAccess),

	// Machinima
	260 => RequestMachinimaActorModel(machinima::RequestMachinimaActorModel),
	261 => SetMachinimaActorModel(machinima::SetMachinimaActorModel),
	262 => UpdateMachinimaScene(machinima::UpdateMachinimaScene) [compressed],

	// Camera
	280 => SetServerCamera(camera::SetServerCamera),
	281 => CameraShakeEffect(camera::CameraShakeEffect),
	282 => RequestFlyCameraMode(camera::RequestFlyCameraMode),
	283 => SetFlyCameraMode(camera::SetFlyCameraMode),

	// Interaction
	290 => SyncInteractionChains(interaction::SyncInteractionChains),
	291 => CancelInteractionChain(interaction::CancelInteractionChain),
	292 => PlayInteractionFor(interaction::PlayInteractionFor),
	293 => MountNPC(interaction::MountNPC),
	294 => DisountNPC(interaction::DismountNPC),

	// Builder Tools
	400 => BuilderToolArgUpdate(buildertools::BuilderToolArgUpdate),
	401 => BuilderToolEntityAction(buildertools::BuilderToolEntityAction),
	402 => BuilderToolSetEntityTransform(buildertools::BuilderToolSetEntityTransform),
	403 => BuilderToolExtrudeAction(buildertools::BuilderToolExtrudeAction),
	404 => BuilderToolStackArea(buildertools::BuilderToolStackArea),
	405 => BuilderToolSelectionTransform(buildertools::BuilderToolSelectionTransform),
	406 => BuilderToolRotateClipboard(buildertools::BuilderToolRotateClipboard),
	407 => BuilderToolPasteClipboard(buildertools::BuilderToolPasteClipboard),
	408 => BuilderToolSetTransformationModeState(buildertools::BuilderToolSetTransformationModeState),
	409 => BuilderToolSelectionUpdate(buildertools::BuilderToolSelectionUpdate),
	410 => BuilderToolSelectionToolAskForClipboard(buildertools::BuilderToolSelectionToolAskForClipboard),
	411 => BuilderToolSelectionToolReplyWithClipboard(buildertools::BuilderToolSelectionToolReplyWithClipboard) [compressed],
	412 => BuilderToolGeneralAction(buildertools::BuilderToolGeneralAction),
	413 => BuilderToolOnUseInteraction(buildertools::BuilderToolOnUseInteraction),
	414 => BuilderToolLineAction(buildertools::BuilderToolLineAction),
	415 => BuilderToolShowAnchor(buildertools::BuilderToolShowAnchor),
	416 => BuilderToolHideAnchors(buildertools::BuilderToolHideAnchors),
	417 => PrefabUnselectPrefab(buildertools::PrefabUnselectPrefab),
	418 => BuilderToolsSetSoundSet(buildertools::BuilderToolsSetSoundSet),
	419 => BuilderToolLaserPointer(buildertools::BuilderToolLaserPointer),
	420 => BuilderToolSetEntityScale(buildertools::BuilderToolSetEntityScale),
	421 => BuilderToolSetEntityPickupEnabled(buildertools::BuilderToolSetEntityPickupEnabled),
	422 => BuilderToolSetEntityLight(buildertools::BuilderToolSetEntityLight),
	423 => BuilderToolSetNPCDebug(buildertools::BuilderToolSetNPCDebug),
}
