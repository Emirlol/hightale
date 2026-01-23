use bytes::Bytes;
use macros::define_packet;

use crate::{
	define_enum,
	v1::Model,
};

define_packet! { RequestMachinimaActorModel {
	variable {
		opt(1) model_id: String,
		opt(2) scene_name: String,
		opt(4) actor_name: String
	}
} }
define_enum! {
	pub enum SceneUpdateType {
		Update = 0,
		Play = 1,
		Stop = 2,
		Frame = 3,
		Save = 4,
	}
}
define_packet! { SetMachinimaActorModel {
	variable {
		opt(1) model: Box<Model>,
		opt(2) scene_name: String,
		opt(4) actor_name: String
	}
} }
define_packet! { UpdateMachinimaScene {
	fixed {
		required frame: f32,
		required update_type: SceneUpdateType
	}
	variable {
		opt(1) scene_name: String,
		opt(2) scene: Bytes
	}
} }
