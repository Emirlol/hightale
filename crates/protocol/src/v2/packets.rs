#![allow(clippy::enum_variant_names)]

use std::collections::HashMap;

use bytes::{
	Buf,
	Bytes,
	BytesMut,
};
use macros::define_packet;
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
	id_dispatch,
	v2::{
		buildertools::BuilderToolState,
		interaction::InteractionType,
		objects::{
			DirectionF,
			FloatRange,
			PositionF,
			Vector2f,
			Vector2i,
			Vector3f,
			Vector3i,
		},
		Color,
		ColorAlpha,
		ColorLight,
		InteractionCooldown,
		InteractionRules,
		RangeF,
		RangeI,
		RangeVector2f,
		RangeVector3f,
	},
};

pub mod asseteditor;
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

define_packet! {
	HostAddress {
		fixed {
			required port: u16
		}
		variable {
			required host: String
		}
	}
}

define_packet! {
	Asset {
		fixed {
			required hash: FixedAscii<64>, // 64-char Hex String
		}
		variable {
			required name: String,         // Filename (e.g. "models/player.json")
		}
	}
}

define_enum! {
	pub enum PositionType {
		AttachedToPlusOffset = 0,
		Custom = 1
	}
}

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
		variable {
			opt(1) value: String
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
			opt(1) raw_text: String,
			opt(2) message_id: String,
			opt(4) children: Vec<FormattedMessage>,
			opt(8) params: HashMap<String, ParamValue>,
			opt(16) message_params: HashMap<String, FormattedMessage>,
			opt(32) color: String,
			opt(64) link: String
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
			opt(1) metadata: String
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
			opt(1) item_id: String,
			opt(2) resource_type_id: String
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
			opt(1) id: String,
			opt(2) categories: Vec<String>
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
			opt(1) id: String,
			opt(2) inputs: Vec<MaterialQuantity>,
			opt(4) outputs: Vec<MaterialQuantity>,
			opt(8) primary_output: MaterialQuantity,
			opt(16) bench_requirement: Vec<BenchRequirement>,
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
			opt(1) hit_location: Vector3f,
			opt(2) position: PositionF,
			opt(4) body_rotation: DirectionF
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
		variable {
			opt(1) text: String
		}
	}
}

define_packet! {
	CombatTextUpdate {
		fixed {
			required hit_angle_deg: f32,
		}
		variable {
			opt(1) text: String
		}
	}
}

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
			opt(1) angle_range: RangeF,
		}
		variable {
			opt(2) target_nodes: Vec<CameraNode>,
		}
	}
}

define_packet! {
	CameraSettings {
		fixed {
			opt(1) position_offset: Vector3f,
		}
		variable {
			opt(2) yaw: CameraAxis,
			opt(4) pitch: CameraAxis,
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
			opt(1) name: String,
			opt(2) footstep_invervals_count: Vec<i32>
		}
	}
}

define_packet! {
	AnimationSet {
		fixed {
			opt(1) next_animation_delay: RangeF,
		}
		variable {
			opt(2) id: String,
			opt(4) animations: Vec<Animation>,
		}
	}
}

define_packet! {
	ModelAttachment {
		variable {
			opt(1) model: String,
			opt(2) texture: String,
			opt(4) gradient_set: String,
			opt(8) gradient_id: String
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

define_packet! {
	ModelParticle {
		fixed {
			required scale: f32,
			opt(1) color: Color,
			required target_entity_part: EntityPart,
			opt(2) position_offset: Vector3f,
			opt(4) rotation_offset: DirectionF,
			required detached_from_model: bool,
		}
		variable {
			opt(8) system_id: String,
			opt(16) target_node_name: String
		}
	}
}
define_packet! {
	ModelTrail {
		fixed {
			required target_entity_part: EntityPart,
			opt(1) position_offset: Vector3f,
			opt(2) rotation_offset: DirectionF,
			required fixed_rotation: bool,
		}
		variable {
			opt(4) trail_id: String,
			opt(8) target_node_name: String
		}
	}
}

define_packet! {
	DetailBox {
		fixed {
			opt(1) offset: Vector3f,
			opt(2) r#box: Hitbox // Box is a keyword in rust
		}
	}
}
define_enum! {
	pub enum Phobia {
		None = 0,
		Arachnophobia = 1,
		Ophidiophobia = 2,
	}
}

define_packet! {
	Model {
		fixed {
			required scale: f32,
			required eye_height: f32,
			required crouch_offset: f32,
			opt(0, 1) hitbox: Hitbox,
			opt(0, 2) light: ColorLight,
			required phobia: Phobia,
		}
		variable {
			opt(0, 4) asset_id: String,
			opt(0, 8) path: String,
			opt(0, 16) texture: String,
			opt(0, 32) gradient_set: String,
			opt(0, 64) gradient_id: String,
			opt(0, 128) camera: CameraSettings,
			opt(1, 1) animation_sets: HashMap<String, AnimationSet>,
			opt(1, 2) attachments: Vec<ModelAttachment>,

			opt(1, 4) particles: Vec<ModelParticle>,
			opt(1, 8) trails: Vec<ModelTrail>,
			opt(1, 16) detail_boxes: HashMap<String, Vec<DetailBox>>,
			opt(1, 32) phobia_model: Box<Model>,
		}
	}
}
define_packet! {
	Equipment {
		variable {
			opt(1) armor_ids: Vec<String>,
			opt(2) right_hand_item_id: String,
			opt(4) left_hand_item_id: String,
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
			opt(1) modifier: Modifier
		}
		variable {
			opt(2) modifiers: HashMap<String, Modifier>,
			opt(4) modifier_key: String
		}
	}
}
define_packet! {
	ModelTransform {
		fixed {
			opt(1) position: PositionF,
			opt(2) body_orientation: DirectionF,
			opt(4) look_orientation: DirectionF,
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
		}
		variable {
			opt(1) status_effect_icon: String
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
			opt(1) position: Vector3f,
			opt(2) orientation: Vector3f,
			required block_type_id: i32,
		}
	}
}
define_packet! {
	MountedUpdate {
		fixed {
			required mounted_to_entity: i32,
			opt(1) attachment_offset: Vector3f,
			required mount_controller: MountController,
			opt(2) block: BlockMount,
		}
	}
}
define_packet! {
	ComponentUpdate {
		fixed {
			required update_type: ComponentUpdateType,
			required block_id: i32,
			required entity_scale: f32,
			opt(0, 1) transform: ModelTransform,
			opt(0, 2) movement_states: MovementStates,
			opt(0, 4) dynamic_light: ColorLight,
			required hitbox_collision_config_index: i32,
			required repulsion_config_index: i32,
			required prediction_id: Uuid,
			opt(0, 8) mounted: MountedUpdate,
		}
		variable {
			opt(0, 16) nameplate: Nameplate,
			opt(0, 32) entity_ui_components: Vec<i32>,
			opt(0, 64) combat_text_update: CombatTextUpdate,
			opt(0, 128) model: Model,
			opt(1, 1) skin: setup::PlayerSkin,
			opt(1, 2) item: ItemWithAllMetadata,
			opt(1, 4) equipment: Equipment,
			opt(1, 8) entity_stat_updates: HashMap<i32, Vec<EntityStatUpdate>>,
			opt(1, 16) entity_effect_updates: Vec<EntityEffectUpdate>,
			opt(1, 32) interactions: HashMap<InteractionType, i32>,
			opt(1, 64) sound_event_ids: Vec<i32>,
			opt(1, 128) interaction_hint: String,
			opt(2, 1) active_animations: BitOptionVec<String>,
		}
	}
}
define_packet! {
	EntityUpdate {
		fixed {
			required network_id: i32
		}
		variable {
			opt(1) removed: Vec<ComponentUpdateType>,
			opt(2) updates: Vec<ComponentUpdate>
		}
	}
}
define_packet! {
	ItemQuantity {
		fixed {
			required quantity: i32,
		}
		variable {
			opt(1) item_id: String,
		}
	}
}
define_packet! { HalfFloatPosition { x: i16, y: i16, z: i16 } }

define_packet! { TeleportAck { teleport_id: u8 } }

define_packet! { Vector3d { x: f64, y: f64, z: f64 } }

define_packet! {
	DamageCause {
		variable {
			opt(1) id: String,
			opt(2) damage_text_color: String
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
	MouseMotionEvent {
		fixed {
			opt(1) relative_motion: Vector2i,
		}
		variable {
			opt(2) mouse_button_type: Vec<MouseButtonType>,
		}
	}
}
define_packet! {
	WorldInteraction {
		fixed {
			required entity_id: i32,
			opt(1) block_position: Vector3i,
			opt(2) block_rotation: BlockRotation,
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
		variable {
			opt(1) resources: Vec<ItemQuantity>
		}
	}
}

id_dispatch! {
	WindowAction from window {
		0 => CraftRecipeAction,
		1 => TierUpgradeAction,
		2 => SelectSlotAction,
		3 => ChangeBlockAction,
		4 => SetActiveAction,
		5 => CraftItemAction,
		6 => UpdateCategoryAction,
		7 => CancelCraftingAction,
		8 => SortItemsAction,
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
			opt(1) position: PositionF,
			opt(2) orientation: DirectionF,
		}
	}
}
define_packet! {
	Objective {
		fixed {
			required objective_uuid: Uuid,
		}
		variable {
			opt(1) objective_title_key: String,
			opt(2) objective_description_key: String,
			opt(4) objective_line_id: String,
			opt(8) tasks: Vec<ObjectiveTask>
		}
	}
}
define_packet! {
	ObjectiveTask {
		fixed {
			required current_completion: i32,
			required completion_needed: i32,
		}
		variable {
			opt(1) task_description_key: String
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
	AudioCategory {
		fixed {
			required volume: f32,
		}
		variable {
			opt(1) id: String,
		}
	}
}

define_packet! {
	BlockBreakingDecal {
		variable {
			opt(1) stage_textures: Vec<String>,
		}
	}
}

define_packet! {
	BlockGroup {
		variable {
			opt(1) names: Vec<String>,
		}
	}
}

define_packet! {
	BlockParticleSet {
		fixed {
			opt(1) color: Color,
			required scale: f32,
			opt(2) position_offset: Vector3f,
			opt(4) rotation_offset: DirectionF,
		}
		variable {
			opt(8) id: String,
			opt(16) particle_system_ids: HashMap<BlockParticleEvent, String>,
		}
	}
}

define_packet! {
	BlockSet {
		variable {
			opt(1) name: String,
			opt(2) blocks: Vec<i32>
		}
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
			opt(1) move_in_repeat_range: FloatRange,
		}
		variable {
			opt(2) id: String,
			opt(4) sound_event_indices: HashMap<BlockSoundEvent, i32>,
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
		}
		variable {
			opt(1) texture: String
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
			opt(1) face_type: String,
			opt(2) self_face_type: String,
			opt(4) block_set_id: String,
			opt(8) filler: Vec<Vector3i>,
		}
	}
}

define_packet! {
	BlockFaceSupport {
		variable {
			opt(1) face_type: String,
			opt(2) filler: Vec<Vector3i>,
		}
	}
}

define_packet! {
	BlockTextures {
		fixed {
			required weight: f32,
		}
		variable {
			opt(1) top: String,
			opt(2) bottom: String,
			opt(4) front: String,
			opt(8) back: String,
			opt(16) left: String,
			opt(32) right: String,
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
			opt(1) gather_type: String,
			opt(2) item_id: String,
			opt(4) drop_list_id: String,
		}
	}
}

define_packet! {
	Harvesting {
		variable {
			opt(1) item_id: String,
			opt(2) drop_list_id: String,
		}
	}
}

define_packet! {
	SoftBlock {
		fixed {
			required is_weapon_breakable: bool
		}
		variable {
			opt(1) item_id: String,
			opt(2) drop_list_id: String,
		}
	}
}

define_packet! {
	BlockGathering {
		variable {
			opt(1) breaking: BlockBreaking,
			opt(2) harvesting: Harvesting,
			opt(4) soft_block: SoftBlock,
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
			opt(1) translation: Vector3f,
			opt(2) rotation: Vector3f,
			opt(4) scale: Vector3f,
		}
		variable {
			opt(8) node: String,
			opt(16) attach_to: String
		}
	}
}

define_packet! {
	RailPoint {
		fixed {
			opt(1) point: Vector3f,
			opt(2) normal: Vector3f,
		}
	}
}

define_packet! {
	RailConfig {
		variable {
			opt(1) points: Vec<RailPoint>
		}
	}
}

define_packet! {
	BenchUpgradingRequirement {
		fixed {
			required time_seconds: f64,
		}
		variable {
			opt(1) material: Vec<MaterialQuantity>
		}
	}
}

define_packet! {
	BenchTierLevel {
		fixed {
			required crafting_time_reduction_modifier: f64,
			required extra_input_slot: i32,
			required extra_output_slot: i32,
		}
		variable {
			opt(1) bench_upgrade_requirement: BenchUpgradingRequirement
		}
	}
}

define_packet! {
	Bench {
		variable {
			opt(1) bench_tier_levels: Vec<BenchTierLevel>,
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
		}
		variable {
			opt(1) material_name: String,
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
			opt(1) regular: StairConnectedBlockRuleSet,
			opt(2) hollow: StairConnectedBlockRuleSet,
			opt(4) material_name: String,
		}
	}
}

define_packet! {
	ConnectedBlockRuleSet {
		fixed {
			required rule_set_type: ConnectedBlockRuleSetType,
		}
		variable {
			opt(1) stair: StairConnectedBlockRuleSet,
			opt(2) roof: RoofConnectedBlockRuleSet,
		}
	}
}

define_packet! {
	BlockType {
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
			opt(0, 1) particle_color: Color,
			opt(0, 2) light: ColorLight,
			opt(0, 4) tint: Tint,
			opt(0, 8) biome_tint: Tint,
			required group: i32,
			opt(0, 16) movement_settings: BlockMovementSettings,
			opt(0, 32) flags: BlockFlags,
			opt(0, 64) placement_settings: BlockPlacementSettings,
			required ignore_support_when_placed: bool,
			required transition_to_tag: i32,
		}
		variable {
			opt(0, 128) item: String,
			opt(1, 1) name: String,
			opt(1, 2) shader_effect: Vec<ShaderType>,
			opt(1, 4) model: String,
			opt(1, 8) model_texture: Vec<ModelTexture>,
			opt(1, 16) model_animation: String,
			opt(1, 32) support: HashMap<BlockNeighbor, Vec<RequiredBlockFaceSupport>>,
			opt(1, 64) supporting: HashMap<BlockNeighbor, Vec<BlockFaceSupport>>,
			opt(1, 128) cube_textures: Vec<BlockTextures>,
			opt(2, 1) cube_side_mask_texture: String,
			opt(2, 2) particles: Vec<ModelParticle>,
			opt(2, 4) block_particle_set_id: String,
			opt(2, 8) block_breaking_decal_id: String,
			opt(2, 16) transition_texture: String,
			opt(2, 32) transition_to_groups: Vec<i32>,
			opt(2, 64) interaction_hint: String,
			opt(2, 128) gathering: BlockGathering,
			opt(3, 1) display: ModelDisplay,
			opt(3, 2) rail: RailConfig,
			opt(3, 4) interactions: HashMap<InteractionType, i32>,
			opt(3, 8) states: HashMap<String, i32>,
			opt(3, 16) tag_indexes: Vec<i32>,
			opt(3, 32) bench: Bench,
			opt(3, 64) connected_block_rule_set: ConnectedBlockRuleSet,
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
			opt(1) clamp: ClampConfig,
		}
	}
}

define_packet! {
	OffsetNoise {
		variable {
			opt(1) x: Vec<NoiseConfig>,
			opt(2) y: Vec<NoiseConfig>,
			opt(4) z: Vec<NoiseConfig>,
		}
	}
}

define_packet! {
	RotationNoise {
		variable {
			opt(1) pitch: Vec<NoiseConfig>,
			opt(2) yaw: Vec<NoiseConfig>,
			opt(4) roll: Vec<NoiseConfig>,
		}
	}
}

define_packet! {
	CameraShakeConfig {
		fixed {
			required duration: f32,
			required start_time: f32,
			required continuous: bool,
			opt(1) ease_in: EasingConfig,
			opt(2) ease_out: EasingConfig,
		}
		variable {
			opt(4) offset: OffsetNoise,
			opt(8) rotation: RotationNoise,
		}
	}
}

define_packet! {
	CameraShake {
		variable {
			opt(1) first_person: CameraShakeConfig,
			opt(2) third_person: CameraShakeConfig,
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
		variable {
			opt(1) disabled: Vec<InteractionType>
		}
	}
}

define_packet! {
	ApplicationEffects {
		fixed {
			opt(0, 1) entity_bottom_tint: Color,
			opt(0, 2) entity_top_tint: Color,
			required horizontal_speed_multiplier: f32,
			required sound_event_index_local: i32,
			required sound_event_index_world: i32,
			opt(0, 4) movement_effects: MovementEffects,
			required mouse_sensitivity_adjustment_target: f32,
			required mouse_sensitivity_adjustment_duration: f32,
		}
		variable {
			opt(0, 8) entity_animation_id: String,
			opt(0, 16) particles: Vec<ModelParticle>,
			opt(0, 32) first_person_particles: Vec<ModelParticle>,
			opt(0, 64) screen_effect: String,
			opt(0, 128) model_vfx_id: String,
			opt(1, 1) ability_effects: AbilityEffects,
		}
	}
}

define_packet! {
	ModelOverride {
		variable {
			opt(1) model: String,
			opt(2) texture: String,
			opt(4) animation_sets: HashMap<String, AnimationSet>
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
			opt(1) id: String,
			opt(2) name: String,
			opt(4) application_effects: ApplicationEffects,
			opt(8) model_override: ModelOverride,
			opt(16) status_effect_icon: String,
			opt(32) stat_modifiers: HashMap<i32, f32>,
		}
	}
}

define_packet! {
	EntityStatEffects {
		fixed {
			required trigger_at_zero: bool,
			required sound_event_index: i32,
		}
		variable {
			opt(1) particles: Vec<ModelParticle>
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
			opt(1) id: String,
			opt(2) min_value_effects: EntityStatEffects,
			opt(4) max_value_effects: EntityStatEffects,
		}
	}
}

define_enum! {
	pub enum EntityUIType {
		EntityStat = 0,
		CombatText = 1
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
			opt(1) position_offset: Vector2f,
			required start_opacity: f32,
			required end_opacity: f32,
		}
	}
}

define_packet! {
	EntityUIComponent {
		fixed {
			required entity_ui_type: EntityUIType,
			opt(1) hitbox_offset: Vector2f,
			required unknown: bool,
			required entity_stat_index: i32,
			opt(2) combat_text_random_position_offset_range: RangeVector2f,
			required combat_text_viewport_margin: f32,
			required combat_text_duration: f32,
			required combat_text_hit_angle_modifier_strength: f32,
			required combat_text_font_size: f32,
			opt(4) combat_text_color: Color,
		}
		variable {
			opt(8) combat_text_animation_events: Vec<CombatTextEntityUIComponentAnimationEvent>
		}
	}
}

define_packet! {
	FluidParticle {
		fixed {
			opt(1) color: Color,
			required scale: f32,
		}
		variable {
			opt(2) system_id: String
		}
	}
}

define_packet! {
	WorldEnvironment {
		fixed {
			opt(1) water_tint: Color,
		}
		variable {
			opt(2) id: String,
			opt(4) fluid_particles: HashMap<i32, FluidParticle>,
			opt(8) tag_indexes: Vec<i32>,
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
		}
		variable {
			opt(1) id: String
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
			opt(1) id: String,
			opt(2) name: String,
			opt(4) icon: String,
			opt(8) children: Vec<ItemCategory>
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
			opt(1) fog_color: Color,
			opt(2) fog_distance: NearFar,
			required fog_depth_start: f32,
			required fog_depth_falloff: f32,
			opt(4) color_filter: Color,
			required color_saturation: f32,
			required distortion_amplitude: f32,
			required distortion_frequency: f32,
			opt(8) movement_settings: FluidFXMovementSettings,
		}
		variable {
			opt(16) id: String,
			opt(32) particle: FluidParticle,
		}
	}
}

define_packet! {
	Fluid {
		fixed {
			required max_fluid_level: i32,
			required requires_alpha_blending: bool,
			required opacity: Opacity,
			opt(1) light: ColorLight,
			required fluid_fx_index: i32,
			required block_sound_set_index: i32,
			opt(2) particle_color: Color,
		}
		variable {
			opt(4) id: String,
			opt(8) cube_textures: Vec<BlockTextures>,
			opt(16) shader_effect: Vec<ShaderType>,
			opt(32) block_particle_set_id: String,
			opt(64) tag_indexes: Vec<i32>,
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
			opt(1) left_offset_override: Vector3f,
			opt(2) left_rotation_override: Vector3f,
			opt(4) right_offset_override: Vector3f,
			opt(8) right_rotation_override: Vector3f,
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
			opt(1) third_person: String,
			opt(2) third_person_moving: String,
			opt(4) third_person_face: String,
			opt(8) first_person: String,
			opt(16) first_person_override: String,
		}
	}
}

define_packet! {
	ItemPlayerAnimations {
		fixed {
			opt(1) wiggle_weights: WiggleWeights,
			opt(2) pullback_config: ItemPullbackConfiguration,
			required use_first_person_override: bool
		}
		variable {
			opt(4) id: String,
			opt(8) animations: HashMap<String, ItemAnimation>,
			opt(16) camera: CameraSettings
		}
	}
}

define_packet! {
	ItemQuality {
		fixed {
			opt(1) text_color: Color,
			required visible_quality_label: bool,
			required render_special_slot: bool,
			required hide_from_search: bool,
		}
		variable {
			opt(2) id: String,
			opt(4) item_tooltip_texture: String,
			opt(8) item_tooltip_arrow_texture: String,
			opt(16) slot_texture: String,
			opt(32) block_slot_texture: String,
			opt(64) special_slot_texture: String,
			opt(128) localization_key: String
		}
	}
}

define_packet! {
	ItemReticle {
		fixed {
			required hide_base: bool,
			required duration: f32,
		}
		variable {
			opt(1) parts: Vec<String>
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
			opt(1) id: String,
			opt(2) base: Vec<String>,
			opt(4) server_events: HashMap<i32, ItemReticle>,
			opt(8) client_events: HashMap<ItemReticleClientEvent, ItemReticle>,
		}
	}
}

define_packet! {
	AssetIconProperties {
		fixed {
			required scale: f32,
			opt(1) translation: Vector2f,
			opt(2) rotation: Vector3f,
		}
	}
}

define_packet! {
	ItemTranslationProperties {
		variable {
			opt(1) name: String,
			opt(2) description: String,
		}
	}
}

define_packet! {
	ItemResourceType {
		fixed {
			required quantity: i32,
		}
		variable {
			opt(1) id: String
		}
	}
}

define_packet! {
	ItemToolSpec {
		fixed {
			required power: f32,
			required quality: i32,
		}
		variable {
			opt(1) gather_type: String
		}
	}
}

define_packet! {
	ItemTool {
		fixed {
			required speed: f32,
		}
		variable {
			opt(1) specs: Vec<ItemToolSpec>
		}
	}
}

define_packet! {
	ItemWeapon {
		fixed {
			required render_dual_wielded: bool,
		}
		variable {
			opt(1) entity_stats_to_clear: Vec<i32>,
			opt(2) stat_modifiers: HashMap<i32, Vec<Modifier>>
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
			opt(1) cosmetics_to_hide: Vec<Cosmetic>,
			opt(2) stat_modifiers: HashMap<i32, Vec<Modifier>>,
			opt(4) damage_resistance: HashMap<i32, Vec<Modifier>>,
			opt(8) damage_enhancement: HashMap<i32, Vec<Modifier>>,
			opt(16) damage_class_enhancement: HashMap<i32, Vec<Modifier>>,
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
			opt(1) entity_stats_to_clear: Vec<i32>,
			opt(2) stat_modifiers: HashMap<i32, Vec<Modifier>>
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
			opt(1) ui: Vec<String>,
			opt(2) tools: Vec<BuilderToolState>
		}
	}
}

define_packet! {
	ItemEntityConfig {
		fixed {
			opt(1) particle_color: Color,
			required show_item_particles: bool,
		}
		variable {
			opt(2) particle_system_id: String
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
		variable {
			opt(1) values: HashMap<PrioritySlot, i32>
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
			opt(1) use_distance: HashMap<GameMode, f32>,
			opt(2) priorities: HashMap<InteractionType, InteractionPriority>
		}
	}
}

define_packet! {
	ItemAppearanceCondition {
		fixed {
			opt(1) condition: FloatRange,
			required condition_value_type: ValueType,
			required local_sound_event_id: i32,
			required world_sound_event_id: i32,
		}
		variable {
			opt(2) particles: Vec<ModelParticle>,
			opt(4) first_person_particles: Vec<ModelParticle>,
			opt(8) model: String,
			opt(16) texture: String,
			opt(32) model_vfx_id: String,
		}
	}
}

define_packet! {
	ItemBase {
		fixed {
			required scale: f32,
			required use_player_animations: bool,
			required max_stack: i32,
			required reticle_index: i32,
			opt(0, 1) icon_properties: AssetIconProperties,
			required item_level: i32,
			required quality_index: i32,
			required consumable: bool,
			required variant: bool,
			required block_id: i32,
			opt(0, 2) glider_config: ItemGlider,
			opt(0, 4) block_selector_tool: BlockSelectorToolData,
			opt(0, 8) light: ColorLight,
			required durability: f64,
			required sound_event_index: i32,
			required item_sound_set_index: i32,
			opt(0, 16) pullback_config: ItemPullbackConfiguration,
			required clips_geometry: bool,
			required render_deployable_preview: bool,
		}
		variable {
			opt(0, 32) id: String,
			opt(0, 64) model: String,
			opt(0, 128) texture: String,
			opt(1, 1) animation: String,
			opt(1, 2) player_animations_id: String,
			opt(1, 4) icon: String,
			opt(1, 8) appearance_conditions: Vec<ItemAppearanceCondition>,
			opt(1, 16) resource_types: Vec<ItemResourceType>,
			opt(1, 32) tool: ItemTool,
			opt(1, 64) weapon: ItemWeapon,
			opt(1, 128) armor: ItemArmor,
			opt(2, 1) utility: ItemUtility,
			opt(2, 2) builder_tool_data: ItemBuilderToolData,
			opt(2, 4) item_entity: ItemEntityConfig,
			opt(2, 8) set: String,
			opt(2, 16) categories: Vec<String>,
			opt(2, 32) particles: Vec<ModelParticle>,
			opt(2, 64) first_person_particles: Vec<ModelParticle>,
			opt(2, 128) trails: Vec<ModelTrail>,
			opt(3, 1) interactions: HashMap<InteractionType, i32>,
			opt(3, 2) interaction_vars: HashMap<String, i32>,
			opt(3, 4) interaction_config: InteractionConfiguration,
			opt(3, 8) dropped_item_animation: String,
			opt(3, 16) tag_indexes: Vec<i32>,
			opt(3, 32) item_appearance_conditions: HashMap<i32, Vec<ItemAppearanceCondition>>,
			opt(3, 64) display_entity_stats_hud: Vec<i32>,
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
			opt(1) id: String,
			opt(2) sound_event_indices: HashMap<ItemSoundEvent, i32>
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
			opt(1) animation_range: Vector2f,
			required loop_option: LoopOption,
			required curve_type: CurveType,
			opt(2) highlight_color: Color,
			required highlight_thickness: f32,
			required use_bloom_on_highlight: bool,
			required use_progessive_highlight: bool,
			opt(4) noise_scale: Vector2f,
			opt(8) noise_scroll_speed: Vector2f,
			opt(16) post_color: Color,
			required post_color_opacity: f32,
		}
		variable {
			opt(32) id: String
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
	InitialVelocity {
		fixed {
			opt(1) yaw: RangeF,
			opt(2) pitch: RangeF,
			opt(4) speed: RangeF,
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
		}
		variable {
			opt(1) texture: String,
		}
	}
}

define_packet! {
	ParticleAttractor {
		fixed {
			opt(1) position: Vector3f,
			opt(2) radial_axis: Vector3f,
			required trail_position_multiplier: f32,
			required radius: f32,
			required radial_acceleration: f32,
			required radial_tangent_acceleration: f32,
			opt(4) linear_acceleration: Vector3f,
			required radial_impulse: f32,
			required radial_tangent_impulse: f32,
			opt(8) linear_impulse: Vector3f,
			opt(16) damping_multiplier: Vector3f,
		}
	}
}

define_packet! {
	IntersectionHighlight {
		fixed {
			required highlight_threshold: f32,
			opt(1) highlight_color: Color,
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
			opt(1) frame_index: RangeI,
			opt(2) scale: RangeVector2f,
			opt(4) rotation: RangeVector3f,
			opt(8) color: Color,
			required opacity: f32,
		}
	}
}

define_packet! {
	Particle {
		fixed {
			opt(1) frame_size: Size,
			required uv_option: ParticleUVOption,
			required scale_ratio_constraint: ParticleScaleRatioConstraint,
			required soft_particles: SoftParticle,
			required soft_particles_fade_factor: f32,
			required use_sprite_blending: bool,
			opt(2) initial_animation_frame: ParticleAnimationFrame,
			opt(4) collision_animation_frame: ParticleAnimationFrame,
		}
		variable {
			opt(8) texture_path: String,
			opt(16) animation_frames: HashMap<i32, ParticleAnimationFrame>,
		}
	}
}

define_packet! {
	ParticleSpawner {
		fixed {
			required shape: EmitShape,
			opt(0, 1) emit_offset: RangeVector3f,
			required camera_offset: f32,
			required use_emit_direction: bool,
			required life_span: f32,
			opt(0, 2) spawn_rate: RangeF,
			required spawn_burst: bool,
			opt(0, 4) wave_delay: RangeF,
			opt(0, 8) total_particles: RangeI,
			required max_concurrent_particles: i32,
			opt(0, 16) initial_velocity: InitialVelocity,
			required velocity_stretch_multiplier: f32,
			required particle_rotation_influence: ParticleRotationInfluence,
			required particle_rotate_with_spawner: bool,
			required is_low_res: bool,
			required trail_spawner_position_multiplier: f32,
			required trail_spawner_rotation_multiplier: f32,
			opt(0, 32) particle_collision: ParticleCollision,
			required render_mode: FXRenderMode,
			required light_influence: f32,
			required linear_filtering: bool,
			opt(0, 64) particle_life_span: RangeF,
			opt(0, 128) intersection_highlight: IntersectionHighlight,
		}
		variable {
			opt(1, 1) id: String,
			opt(1, 2) particle: Particle,
			opt(1, 4) uv_motion: UVMotion,
			opt(1, 8) attractors: Vec<ParticleAttractor>,
		}

	}
}

define_packet! {
	ParticleSpawnerGroup {
		fixed {
			opt(0, 1) position_offset: Vector3f,
			opt(0, 2) rotation_offset: Vector3f,
			required fixed_rotation: bool,
			required start_delay: f32,
			opt(0, 4) spawn_rate: RangeF,
			opt(0, 8) wave_delay: RangeF,
			required total_spawners: i32,
			required max_concurrent: i32,
			opt(0, 16) initial_velocity: InitialVelocity,
			opt(0, 32) emit_offset: RangeVector3f,
			opt(0, 64) life_span: RangeF,
		}
		variable {
			opt(0, 128) spawner_id: String,
			opt(1, 1) attractors: Vec<ParticleAttractor>,
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
			opt(1) id: String,
			opt(2) spawners: Vec<ParticleSpawnerGroup>,
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
			opt(1) physics_config: PhysicsConfig,
			required launch_force: f64,
			opt(2) spawn_offset: Vector3f,
			opt(4) rotation_offset: DirectionF,
			required launch_local_sound_event_index: i32,
			required projectile_sound_event_index: i32,
		}
		variable {
			opt(8) model: Model,
			opt(16) interactions: HashMap<InteractionType, i32>,
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
			opt(1) id: String,
			opt(2) icon: String
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
		}
		variable {
			opt(1) id: String
		}
	}
}

define_packet! {
	RootInteractionSettings {
		fixed {
			required allow_skip_chain_on_click: bool,
		}
		variable {
			opt(1) cooldown: InteractionCooldown
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
			opt(1) id: String,
			opt(2) interactions: Vec<i32>,
			opt(4) cooldown: InteractionCooldown,
			opt(8) settings: HashMap<GameMode, RootInteractionSettings>,
			opt(16) rules: InteractionRules,
			opt(32) tags: Vec<i32>,
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
			opt(1) random_settings: SoundEventLayerRandomSettings,
		}
		variable {
			opt(2) files: Vec<String>
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
			opt(1) id: String,
			opt(2) layers: Vec<SoundEventLayer>,
		}
	}
}

define_packet! {
	SoundSet {
		fixed {
			required category: SoundCategory
		}
		variable {
			opt(1) id: String,
			opt(2) sounds: HashMap<String, i32>,
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
			opt(1) operands: Vec<TagPattern>,
			opt(2) not: Box<TagPattern>
		}
	}
}

define_packet! {
	Edge {
		fixed {
			opt(1) color: ColorAlpha,
			required width: f32
		}
	}
}

define_packet! {
	Trail {
		fixed {
			required life_span: i32,
			required roll: f32,
			opt(1) start: Edge,
			opt(2) end: Edge,
			required light_influence: f32,
			required render_mode: FXRenderMode,
			opt(4) intersection_highlight: IntersectionHighlight,
			required smooth: bool,
			opt(8) frame_size: Size,
			opt(16) frame_range: RangeI,
			required frame_life_span: i32,
		}
		variable {
			opt(32) id: String,
			opt(64) texture: String
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
		variable {
			opt(1) first_person: CameraShakeConfig
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
			opt(1) texture: String,
			opt(2) speeds: HashMap<OrderedFloat<f32>, f32>,
			opt(4) colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
		}
	}
}

define_packet! {
	WeatherParticle {
		fixed {
			opt(1) color: Color,
			required scale: f32,
			required is_overground_only: bool,
			required position_offset_multiplier: f32,
		}
		variable {
			opt(2) system_id: String,
		}
	}
}

define_packet! {
	Weather {
		fixed {
			opt(0, 1) fog: NearFar,
			opt(0, 2) fog_options: FogOptions
		}
		variable {
			opt(0, 4) id: String,
			opt(0, 8) tag_indexes: Vec<i32>,
			opt(0, 16) stars: String,
			opt(0, 32) moons: HashMap<i32, String>,
			opt(0, 64) clouds: Vec<Cloud>,
			opt(0, 128) sunlight_damping_multiplier: HashMap<OrderedFloat<f32>, f32>,
			opt(1, 1) sunlight_colors: HashMap<OrderedFloat<f32>, Color>,
			opt(1, 2) sky_top_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(1, 4) sky_bottom_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(1, 8) sky_sunset_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(1, 16) sun_colors:  HashMap<OrderedFloat<f32>, Color>,
			opt(1, 32) sun_scales: HashMap<OrderedFloat<f32>, f32>,
			opt(1, 64) sun_glow_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(1, 128) moon_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(2, 1) moon_scales: HashMap<OrderedFloat<f32>, f32>,
			opt(2, 2) moon_glow_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(2, 4) fog_colors: HashMap<OrderedFloat<f32>, Color>,
			opt(2, 8) fog_height_falloffs: HashMap<OrderedFloat<f32>, f32>,
			opt(2, 16) fog_densities: HashMap<OrderedFloat<f32>, f32>,
			opt(2, 32) screen_effect: String,
			opt(2, 64) screen_effect_colors: HashMap<OrderedFloat<f32>, ColorAlpha>,
			opt(2, 128) color_filters: HashMap<OrderedFloat<f32>, Color>,
			opt(3, 1) water_tints: HashMap<OrderedFloat<f32>, Color>,
			opt(3, 2) particle: WeatherParticle,
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

        $(
            impl From<$module::$st> for Packet {
                fn from(p: $module::$st) -> Self {
                    Packet::$variant(p)
                }
            }
        )*

	    #[cfg(test)]
        mod __packet_size_report {
            use super::*;

            #[test]
            fn print_packet_variant_sizes() {
                let mut sizes: Vec<(&'static str, usize)> = vec![
                    $(
                        (
                            stringify!($variant),
                            core::mem::size_of::<$module::$st>(),
                        ),
                    )*
                    ("Unknown(i32, Bytes)", core::mem::size_of::<(i32, Bytes)>()),
                ];

                // Biggest first
                sizes.sort_by_key(|&(_, s)| core::cmp::Reverse(s));

                eprintln!("Packet size: {} bytes", core::mem::size_of::<Packet>());
                eprintln!("Packet alignment: {} bytes", core::mem::align_of::<Packet>());
                eprintln!("--- variant payload sizes ---");
                for (name, sz) in sizes {
                    eprintln!("{:>32}: {:>6} bytes", name, sz);
                }
            }
        }
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

	// Asset Editor
	300 => FailureReply(asseteditor::FailureReply),
	301 => SuccessReply(asseteditor::SuccessReply),
	302 => AssetEditorInitialize(asseteditor::AssetEditorInitialize),
	303 => AssetEditorAuthorization(asseteditor::AssetEditorAuthorization),
	304 => AssetEditorCapabilities(asseteditor::AssetEditorCapabilities),
	305 => AssetEditorSetupSchemas(asseteditor::AssetEditorSetupSchemas) [compressed],
	306 => AssetEditorSetupAssetTypes(asseteditor::AssetEditorSetupAssetTypes),
	307 => AssetEditorCreateDirectory(asseteditor::AssetEditorCreateDirectory),
	308 => AssetEditorDeleteDirectory(asseteditor::AssetEditorDeleteDirectory),
	309 => AssetEditorRenameDirectory(asseteditor::AssetEditorRenameDirectory),
	310 => AssetEditorFetchAsset(asseteditor::AssetEditorFetchAsset),
	311 => AssetEditorFetchJsonAssetWithParents(asseteditor::AssetEditorFetchJsonAssetWithParents),
	312 => AssetEditorFetchAssetReply(asseteditor::AssetEditorFetchAssetReply),
	313 => AssetEditorFetchJsonAssetWithParentsReply(asseteditor::AssetEditorFetchJsonAssetWithParentsReply) [compressed],
	314 => AssetEditorAssetPackSetup(asseteditor::AssetEditorAssetPackSetup),
	315 => AssetEditorUpdateAssetPack(asseteditor::AssetEditorUpdateAssetPack),
	316 => AssetEditorCreateAssetPack(asseteditor::AssetEditorCreateAssetPack),
	317 => AssetEditorDeleteAssetPack(asseteditor::AssetEditorDeleteAssetPack),
	318 => AssetEditorEnableAssetPack(asseteditor::AssetEditorEnableAssetPack),
	319 => AssetEditorAssetListSetup(asseteditor::AssetEditorAssetListSetup) [compressed],
	320 => AssetEditorAssetListUpdate(asseteditor::AssetEditorAssetListUpdate) [compressed],
	321 => AssetEditorRequestChildrenList(asseteditor::AssetEditorRequestChildrenList),
	322 => AssetEditorRequestChildrenListReply(asseteditor::AssetEditorRequestChildrenListReply),
	323 => AssetEditorUpdateJsonAsset(asseteditor::AssetEditorUpdateJsonAsset) [compressed],
	324 => AssetEditorUpdateAsset(asseteditor::AssetEditorUpdateAsset),
	325 => AssetEditorJsonAssetUpdated(asseteditor::AssetEditorJsonAssetUpdated),
	326 => AssetEditorAssetUpdated(asseteditor::AssetEditorAssetUpdated),
	327 => AssetEditorCreateAsset(asseteditor::AssetEditorCreateAsset),
	328 => AssetEditorRenameAsset(asseteditor::AssetEditorRenameAsset),
	329 => AssetEditorDeleteAsset(asseteditor::AssetEditorDeleteAsset),
	330 => AssetEditorDiscardChanges(asseteditor::AssetEditorDiscardChanges),
	331 => AssetEditorFetchAutoCompleteData(asseteditor::AssetEditorFetchAutoCompleteData),
	332 => AssetEditorFetchAutoCompleteDataReply(asseteditor::AssetEditorFetchAutoCompleteDataReply),
	333 => AssetEditorRequestDataset(asseteditor::AssetEditorRequestDataset),
	334 => AssetEditorRequestDatasetReply(asseteditor::AssetEditorRequestDatasetReply),
	335 => AssetEditorActivateButton(asseteditor::AssetEditorActivateButton),
	336 => AssetEditorSelectAsset(asseteditor::AssetEditorSelectAsset),
	337 => AssetEditorPopupNotification(asseteditor::AssetEditorPopupNotification),
	338 => AssetEditorFetchLastModifiedAssets(asseteditor::AssetEditorFetchLastModifiedAssets),
	339 => AssetEditorLastModifiedAssets(asseteditor::AssetEditorLastModifiedAssets),
	340 => AssetEditorModifiedAssetsCount(asseteditor::AssetEditorModifiedAssetsCount),
	341 => AssetEditorSubscribeModifiedAssetsChanges(asseteditor::AssetEditorSubscribeModifiedAssetsChanges),
	342 => AssetEditorExportAssets(asseteditor::AssetEditorExportAssets),
	343 => AssetEditorExportAssetInitialize(asseteditor::AssetEditorExportAssetInitialize),
	344 => AssetEditorExportAssetPart(asseteditor::AssetEditorExportAssetPart) [compressed],
	345 => AssetEditorExportAssetFinalize(asseteditor::AssetEditorExportAssetFinalize),
	346 => AssetEditorExportDeleteAssets(asseteditor::AssetEditorExportDeleteAssets),
	347 => AssetEditorExportComplete(asseteditor::AssetEditorExportComplete),
	348 => AssetEditorRebuildCaches(asseteditor::AssetEditorRebuildCaches),
	349 => AssetEditorUndoChanges(asseteditor::AssetEditorUndoChanges),
	350 => AssetEditorRedoChanges(asseteditor::AssetEditorRedoChanges),
	351 => AssetEditorUndoRedoReply(asseteditor::AssetEditorUndoRedoReply),
	352 => AssetEditorSetGameTime(asseteditor::AssetEditorSetGameTime),
	353 => AssetEditorUpdateSecondsPerGameDay(asseteditor::AssetEditorUpdateSecondsPerGameDay),
	354 => AssetEditorUpdateWeatherPreviewLock(asseteditor::AssetEditorUpdateWeatherPreviewLock),
	355 => AssetEditorUpdateModelPreview(asseteditor::AssetEditorUpdateModelPreview),

	// World, part 2 for some reason
	360 => UpdateSunSettings(world::UpdateSunSettings),
	361 => UpdatePostFxSettings(world::UpdatePostFxSettings),

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
