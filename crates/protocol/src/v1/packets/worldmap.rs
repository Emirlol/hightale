use std::collections::HashMap;

use bytes::Buf;
use macros::define_packet;

use crate::v1::Transform;

define_packet! { BiomeData {
	fixed {
		required zone_id: i32,
		required biome_color: i32
	}
	variable {
		opt(1) zone_name: String,
		opt(2) biome_name: String
	}
} }
define_packet! { ClearWorldMap { } }
define_packet! { ContextMenuItem {
	variable {
		required name: String,
		required command: String
	}
} }
define_packet! { MapChunk {
	fixed {
		required chunk_x: i32,
		required chunk_z: i32,
	}
	variable {
		opt(1) image: MapImage
	}
} }
define_packet! { MapImage {
	fixed {
		required width: i32,
		required height: i32,
	}
	variable {
		opt(1) data: Vec<i32>
	}
} }
define_packet! { MapMarker {
	fixed {
		opt(8) transform: Transform
	}
	variable {
		opt(1) id: String,
		opt(2) name: String,
		opt(4) marker_image: String,
		opt(16) context_menu_items: Vec<ContextMenuItem>
	}
} }
define_packet! { TeleportToWorldMapMarker {
	variable {
		opt(1) marker_id: String
	}
} }
define_packet! { TeleportToWorldMapPosition { position_x: f32, position_y: f32 } }
define_packet! { UpdateWorldMap {
	variable {
		opt(1) chunks: Vec<MapChunk>,
		opt(2) added_markers: Vec<MapMarker>,
		opt(4) removed_markers: Vec<String>
	}
} }
define_packet! { UpdateWorldMapSettings {
	fixed {
		required enabled: bool,
		required allow_teleport_to_coordinates: bool,
		required allow_teleport_to_markers: bool,
		required default_scale: f32,
		required min_scale: f32,
		required max_scale: f32,
	}
	variable {
		opt(1) biome_data_map: HashMap<i16, BiomeData>
	}
} }
define_packet! { UpdateWorldMapVisible { visible: bool } }
