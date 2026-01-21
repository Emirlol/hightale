use std::collections::HashMap;

use crate::{
	define_enum,
	define_packet,
	v1::{
		interaction::InteractionType,
		interface::{
			BlockChange,
			FluidChange,
		},
		ColorLight,
		ModelTransform,
		RangeF,
		RangeI,
		Rotation,
		Vector3f,
		Vector3i,
	},
};

define_enum! {
	pub enum Axis {
		X = 0,
		Y = 1,
		Z = 2
	}
}

define_enum! {
	pub enum BrushAxis {
		None = 0,
		Auto = 1,
		X = 2,
		Y = 3,
		Z = 4
	}
}

define_enum! {
	pub enum BrushOrigin {
		Center = 0,
		Bottom = 1,
		Top = 2
	}
}

define_enum! {
	pub enum BrushShape {
		Cube = 0,
		Sphere = 1,
		Cylinder = 2,
		Cone = 3,
		InvertedCone = 4,
		Pyramid = 5,
		InvertedPyramid = 6,
		Dome = 7,
		InvertedDome = 8,
		Diamond = 9,
		Torus = 10,
	}
}

define_enum! {
	pub enum BuilderToolAction {
		SelectionPosition1 = 0,
		SelectionPosition2 = 1,
		SelectionCopy = 2,
		HistoryUndo = 3,
		HistoryRedo = 4,
		ActivateToolMode = 5,
		DeactivateToolMode = 6,
	}
}

define_packet! {
	BuilderToolArg {

	}
}

define_enum! {
	pub enum BuilderToolArgGroup {
		Tool = 0,
		Brush = 1
	}
}

define_enum! {
	pub enum BuilderToolArgType {
		Bool = 0,
		Float = 1,
		Int = 2,
		String = 3,
		Block = 4,
		Mask = 5,
		BrushShape = 6,
		BrushOrigin = 7,
		BrushAxis = 8,
		Rotation = 9,
		Option = 10,
	}
}

define_packet! {
	BuilderToolArgUpdate {
		fixed {
			required token: i32,
			required section: i32,
			required slot: i32,
			required group: BuilderToolArgGroup,
		}
		variable {
			opt id: String,
			opt value: String,
		}
	}
}

define_packet! {
	BuilderToolBlockArg {
		fixed {
			required allow_pattern: bool,
			opt devault_value: String
		}
	}
}

define_packet! {
	BuilderToolBoolArg{
		default_value: bool
	}
}

define_packet! {
	BuilderToolBrushAxisArg {
		default_value: BrushAxis
	}
}

define_packet! {
	BuilderToolBrushData {
		mask_size: 3
		fixed {
			opt(0) width: BuilderToolIntArg [pad=12],
			opt(1) height: BuilderToolIntArg [pad=12],
			opt(2) thickness: BuilderToolIntArg [pad=12],
			opt(3) capped: BuilderToolBoolArg [pad=1],
			opt(4) shape: BuilderToolBrushShapeArg [pad=1],
			opt(5) origin: BuilderToolBrushOriginArg [pad=1],
			opt(6) origin_rotation: BuilderToolBoolArg [pad=1],
			opt(7) rotation_axis: BuilderToolBrushAxisArg [pad=1],
			opt(8) rotation_angle: BuilderToolRotationArg [pad=4],
			opt(9) mirror_axis: BuilderToolBrushAxisArg [pad=1],
			opt(19) use_mask_commands: BuilderToolBoolArg [pad=1],
			opt(20) invert_mask: BuilderToolBoolArg [pad=1],
		}
		variable {
			opt(10) material: BuilderToolBlockArg,
			opt(11) favorite_materials: Vec<BuilderToolBlockArg>,
			opt(12) mask: BuilderToolMaskArg,
			opt(13) mask_above: BuilderToolMaskArg,
			opt(14) mask_not: BuilderToolMaskArg,
			opt(15) mask_below: BuilderToolMaskArg,
			opt(16) mask_adjacent: BuilderToolMaskArg,
			opt(17) mask_neighbor: BuilderToolMaskArg,
			opt(18) mask_commands: Vec<BuilderToolStringArg>,
		}
	}
}

define_packet! {
	BuilderToolBrushOriginArg {
		default_value: BrushOrigin
	}
}

define_packet! {
	BuilderToolBrushShapeArg {
		default_value: BrushShape
	}
}

define_packet! {
	BuilderToolEntityAction {
		action: EntityToolAction
	}
}

define_packet! {
	BuilderToolExtrudeAction {
		pos: Vector3i,
		normal: Vector3i
	}
}

define_packet! {
	BuilderToolFloatArg {
		default_value: f32,
		range: RangeF
	}
}

define_packet! {
	BuilderToolGeneralAction {
		action: BuilderToolAction
	}
}

define_packet! { BuilderToolHideAnchors {} }

define_packet! {
	BuilderToolIntArg {
		default_value: i32,
		range: RangeI
	}
}

define_packet! {
	BuilderToolLaserPointer {
		player_network_id: i32,
		start: Vector3i,
		end: Vector3i,
		color: i32,
		duration_ms: i32
	}
}

define_packet! {
	BuilderToolLineAction {
		start: Vector3i,
		end: Vector3i
	}
}

define_packet! {
	BuilderToolMaskArg {
		fixed {
			opt default_value: String
		}
	}
}

define_packet! {
	BuilderToolOnUseInteraction {
		interaction_type: InteractionType,
		pos: Vector3i,
		offset_for_pain_mode: Vector3i,
		is_alt_play_sculpt_brush_mod_down: bool,
		is_hold_down_interaction: bool,
		is_do_server_raytrace_for_position: bool,
		is_show_edit_notifications: bool,
		max_length_tool_ignore_history: i32,
		raycast_origin: Vector3f,
		raycast_direction: Vector3f
	}
}

define_packet! {
	BuilderToolOptionArg {
		variable {
			opt default_value: String,
			opt options: Vec<String>
		}
	}
}

define_packet! {
	BuilderToolPasteClipboard {
		pos: Vector3i
	}
}

define_packet! {
	BuilderToolRotateClipboard {
		angle: i32,
		axis: Axis
	}
}

define_packet! {
	BuilderToolRotationArg {
		default_value: Rotation
	}
}

define_packet! { BuilderToolSelectionToolAskForClipboard {} }

define_packet! {
	BuilderToolSelectionToolReplyWithClipboard {
		variable {
			opt blocks_change: Vec<BlockChange>,
			opt fluids_change: Vec<FluidChange>
		}
	}
}

define_packet! {
	BuilderToolSelectionTransform {
		fixed {
			opt(1) initial_selection_min: Vector3i [pad=12],
			opt(2) initial_selection_max: Vector3i [pad=12],
			opt(3) initial_rotation_origin: Vector3f [pad=12],
			required cut_original: bool,
			required apply_transformation_to_selection_min_max: bool,
			required is_exiting_transform_mode: bool,
			opt(4) initial_paste_point_for_clipboard_paste: Vector3i [pad=12],
			opt(0) transformation_matrix: Vec<f32>
		}
	}
}

define_packet! {
	BuilderToolSelectionUpdate {
		min: Vector3i,
		max: Vector3i
	}
}

define_packet! {
	BuilderToolSetEntityLight {
		fixed {
			required entity_id: i32,
			opt light: ColorLight [pad=4]
		}
	}
}

define_packet! {
	BuilderToolSetEntityPickupEnabled {
		entity_id: i32,
		enabled: bool
	}
}

define_packet! {
	BuilderToolSetEntityScale {
		entity_id: i32,
		scale: f32
	}
}

define_packet! {
	BuilderToolSetEntityTransform {
		fixed {
			required entity_id: i32,
			opt model_transform: ModelTransform [pad=49]
		}
	}
}

define_packet! {
	BuilderToolSetNPCDebug {
		entity_id: i32,
		enabled: bool
	}
}

define_packet! {
	BuilderToolSetTransformationModeState {
		enabled: bool
	}
}

define_packet! {
	BuilderToolShowAnchor {
		pos: Vector3i,
	}
}

define_packet! {
	BuilderToolsSetSoundSet {
		sound_set_index: i32
	}
}

define_packet! {
	BuilderToolStackArea {
		fixed {
			opt selection_min: Vector3i [pad=12],
			opt selection_max: Vector3i [pad=12],
			required normal: Vector3i,
			required num_stacks: i32,
		}
	}
}

define_packet! {
	BuilderToolState {
		fixed {
			required is_brush: bool
		}
		variable {
			opt id: String,
			opt brush_data: BuilderToolBrushData,
			opt args: HashMap<String, BuilderToolArg>
		}
	}
}

define_packet! {
	BuilderToolStringArg {
		fixed {
			opt default_value: String
		}
	}
}

define_enum! {
	pub enum EntityToolAction {
		Remove = 0,
		Clone = 1,
		Freeze = 2
	}
}

define_packet! { PrefabUnselectPrefab { } }
