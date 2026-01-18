#![allow(unused_variables, clippy::enum_variant_names)]

use std::collections::HashMap;

use bytes::{
	Buf,
	Bytes,
	BytesMut,
};

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
};

pub mod auth;
pub mod camera;
pub mod connection;
pub mod entities;
pub mod interaction;
pub mod interface;
pub mod inventory;
pub mod player;
pub mod serveraccess;
pub mod setup;
pub mod window;
pub mod world;
pub mod worldmap;

/// Max size for variable length items, like strings, maps, lists, etc.
pub const MAX_SIZE: i32 = 4_096_000;

define_packet!(HostAddress { port: u16, host: String });

define_packet!(
	Asset {
		hash: FixedAscii<64>, // 64-char Hex String
		name: String,         // Filename (e.g. "models/player.json")
	}
);

define_packet!(InstantData { seconds: i64, nanos: i32 });

define_packet!(Vector2f { x: f32, y: f32 });

define_packet!(Vector3f { x: f32, y: f32, z: f32 });

define_packet!(PositionF { x: f64, y: f64, z: f64 });

define_enum! {
	pub enum PositionType {
		AttachedToPlusOffset = 0,
		Custom = 1
	}
}

define_packet!(DirectionF { yaw: f32, pitch: f32, roll: f32 });

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

define_packet!(
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
);

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

define_packet!(
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
);

define_packet!(
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
);

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

define_packet! {
	BlockPosition {
		x: i32,
		y: i32,
		z: i32,
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

define_packet!(
	Nameplate {
		fixed {
			opt text: String
		}
	}
);

define_packet!(
	CombatTextUpdate {
		fixed {
			required hit_angle_deg: f32,
			opt text: String
		}
	}
);

define_packet!(RangeF { min: f32, max: f32 });

define_enum! {
	pub enum CameraNode {
		None = 0,
		Head = 1,
		LShoulder = 2,
		RShoulder = 3,
		Belly = 4,
	}
}

define_packet!(
	CameraAxis {
		fixed {
			opt angle_range: RangeF [pad=8],
			opt target_nodes: Vec<CameraNode>,
		}
	}
);

define_packet!(
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
);

define_packet!(
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
);

define_packet!(
	AnimationSet {
		fixed {
			opt next_animation_delay: RangeF [pad=8],
		}
		variable {
			opt id: String,
			opt animations: Vec<Animation>,
		}
	}
);

define_packet!(
	ModelAttachment {
		variable {
			opt model: String,
			opt texture: String,
			opt gradient_set: String,
			opt gradient_id: String
		}
	}
);

define_packet!(Hitbox {
	min_x: f32,
	min_y: f32,
	min_z: f32,
	max_x: f32,
	max_y: f32,
	max_z: f32,
});

define_enum! {
	pub enum EntityPart {
		// This is supposed to be Self = 0 but that's a rust keyword, it can't even be used as r#Self.
		This = 0,
		Entity = 1,
		PrimaryItem = 2,
		SecondaryItem = 3
	}
}

define_packet!(Color { red: u8, green: u8, blue: u8 });

define_packet!(
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
);

define_packet!(
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
);

define_packet!(ColorLight {
	radius: u8,
	red: u8,
	green: u8,
	blue: u8,
});

define_packet!(
	DetailBox {
		fixed {
			opt offset: Vector3f [pad=12],
			opt r#box: Hitbox // Box is a keyword in rust
		}
	}
);

define_enum! {
	pub enum Phobia {
		None = 0,
		Arachnophobia = 1
	}
}

define_packet!(
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
);

define_packet!(
	Equipment {
		variable {
			opt armor_ids: Vec<String>,
			opt right_hand_item_id: String,
			opt left_hand_item_id: String,
		}
	}
);

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

define_packet!(Modifier {
	target: ModifierTarget,
	calculation_type: CalculationType,
	amount: f32,
});

define_packet!(
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
);

define_packet!(
	ModelTransform {
		fixed {
			opt position: PositionF [pad=24],
			opt body_orientation: DirectionF [pad=12],
			opt look_orientation: DirectionF [pad=12],
		}
	}
);

define_packet!(MovementStates {
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
});

define_enum! {
	pub enum EffectOp {
		Add = 0,
		Remove = 1
	}
}

define_packet!(
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
);

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

define_packet!(
	BlockMount {
		fixed {
			required mount_type: BlockMountType,
			opt position: Vector3f [pad=12],
			opt orientation: Vector3f [pad=12],
			required block_type_id: i32,
		}
	}
);

define_packet!(
	MountedUpdate {
		fixed {
			required mounted_to_entity: i32,
			opt attachment_offset: Vector3f [pad=12],
			required mount_controller: MountController,
			opt block: BlockMount [pad=30],
		}
	}
);

define_packet!(
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
			required prediction_id: uuid::Uuid,
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
			opt(11) interactions: HashMap<interaction::InteractionType, i32>,
			opt(13) sound_event_ids: Vec<i32>,
			opt(14) interaction_hint: String,
			opt(16) active_animations: BitOptionVec<String>,
		}
	}
);

define_packet!(
	EntityUpdate {
		fixed {
			required network_id: i32
		}
		variable {
			opt removed: Vec<ComponentUpdateType>,
			opt updates: Vec<ComponentUpdate>
		}
	}
);

define_packet!(
	ItemQuantity {
		fixed {
			required quantity: i32,
			opt item_id: String,
		}
	}
);

define_packet!(HalfFloatPosition { x: i16, y: i16, z: i16 });

define_packet!(TeleportAck { teleport_id: u8 });

define_packet!(Vector3d { x: f64, y: f64, z: f64 });

define_packet!(
	DamageCause {
		variable {
			opt id: String,
			opt damage_text_color: String
		}
	}
);

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

define_packet!(MouseButtonEvent {
	mouse_button_type: MouseButtonType,
	state: MouseButtonState,
	clicks: u8
});

define_packet!(Vector2i { x: i32, y: i32 });

define_packet!(
	MouseMotionEvent {
		fixed {
			opt relative_motion: Vector2i [pad=8],
			opt mouse_button_type: Vec<MouseButtonType>,
		}
	}
);

define_packet!(
	WorldInteraction {
		fixed {
			required entity_id: i32,
			opt block_position: BlockPosition [pad=12],
			opt block_rotation: BlockRotation [pad=3],
		}
	}
);

define_enum! {
	pub enum GameMode {
		Adventure = 0,
		Creative = 1
	}
}

define_packet!(SavedMovementStates { flying: bool });

define_enum! {
	pub enum PickupLocation {
		Hotbar = 0,
		Storage = 1
	}
}

define_packet!(MovementSettings {
	mass: f32,
	drag_coefficient: f32,
	inverted_gravity: bool,
	velocity_resistance: f32,
	jump_force: f32,
	swim_jump_force: f32,
	jump_buffer_duration: f32,
	jump_buffer_max_y_velocity: f32,
	acceleration: f32,
	air_drag_min: f32,
	air_drag_max: f32,
	air_drag_min_speed: f32,
	air_drag_max_speed: f32,
	air_friction_min: f32,
	air_friction_max: f32,
	air_friction_min_speed: f32,
	air_friction_max_speed: f32,
	air_speed_multiplier: f32,
	air_control_min_speed: f32,
	air_control_max_speed: f32,
	air_control_min_multiplier: f32,
	air_control_max_multiplier: f32,
	combo_air_speed_multiplier: f32,
	base_speed: f32,
	climb_speed: f32,
	climb_speed_lateral: f32,
	climb_up_sprint_speed: f32,
	climb_down_sprint_speed: f32,
	horizontal_fly_speed: f32,
	vertical_fly_speed: f32,
	max_speed_multiplier: f32,
	min_speed_multiplier: f32,
	wish_direction_gravity_x: f32,
	wish_direction_gravity_y: f32,
	wish_direction_weight_x: f32,
	wish_direction_weight_y: f32,
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
	min_fall_speed_to_engage_roll: f32,
	max_fall_speed_to_engage_roll: f32,
	roll_start_speed_modifier: f32,
	roll_exit_speed_modifier: f32,
	roll_time_to_complete: f32,
});

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

define_packet!(
	ExtraResources {
		fixed {
			opt resources: Vec<ItemQuantity>
		}
	}
);

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

define_packet!(
	Transform {
		fixed {
			opt position: PositionF [pad=24],
			opt orientation: DirectionF [pad=12],
		}
	}
);

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
}
