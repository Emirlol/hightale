use macros::define_packet;

use crate::{
	define_enum,
	id_dispatch,
	v2::Vector3f,
};

define_packet! {
	AOECircleSelector {
		fixed {
			required range: f32,
			opt(1) offset: Vector3f
		}
	}
}

define_packet! {
	AOECylinderSelector {
		fixed {
			required range: f32,
			required height: f32,
			opt(1) offset: Vector3f
		}
	}
}

define_packet! {
	RaycastSelector {
		fixed {
			opt(1) offset: Vector3f,
			required distance: i32,
			required block_tag_index: i32,
			required ignore_fluids: bool,
			required ignore_empty_collision_material: bool
		}
	}
}

define_enum! {
	pub enum HorizontalSelectorDirection {
		ToLeft = 0,
		ToRight = 1,
	}
}

define_packet! {
	HorizontalSelector {
		extend_top: f32,
		extend_bottom: f32,
		yaw_length: f32,
		yaw_start_offset: f32,
		pitch_offset: f32,
		roll_offset: f32,
		start_distance: f32,
		end_distance: f32,
		direction: HorizontalSelectorDirection,
		test_line_of_sight: bool
	}
}

define_packet! {
	StabSelector {
		extend_top: f32,
		extend_bottom: f32,
		extend_left: f32,
		extend_right: f32,
		yaw_offset: f32,
		pitch_offset: f32,
		roll_offset: f32,
		start_distance: f32,
		end_distance: f32,
		test_line_of_sight: bool
	}
}

id_dispatch! {
	Selector {
		0 => AOECircleSelector,
		1 => AOECylinderSelector,
		2 => RaycastSelector,
		3 => HorizontalSelector,
		4 => StabSelector,
	}
}
