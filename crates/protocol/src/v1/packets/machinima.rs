use bytes::Bytes;

use super::Model;
use crate::{
	define_enum,
	define_packet,
};

define_packet! { RequestMachinimaActorModel {
	variable {
		opt model_id: String,
		opt scene_name: String,
		opt actor_name: String
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
	fixed {
		opt model: Model
	}
	variable {
		opt scene_name: String,
		opt actor_name: String
	}
} }
define_packet! { UpdateMachinimaScene {
	fixed {
		required frame: f32,
		required update_type: SceneUpdateType
	}
	variable {
		opt scene_name: String,
		opt scene: Bytes
	}
} }
