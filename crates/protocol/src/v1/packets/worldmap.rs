use std::collections::HashMap;

use super::Transform;
use crate::define_packet;

define_packet! { BiomeData {
	fixed {
		required zone_id: i32,
		required biome_color: i32
	}
	variable {
		opt zone_name: String,
		opt biome_name: String
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
		opt image: MapImage
	}
} }
define_packet! { MapImage {
	fixed {
		required width: i32,
		required height: i32,
		opt data: Vec<i32>
	}
} }
define_packet! { MapMarker {
	fixed {
		opt(3) transform: Transform [pad=37]
	}
	variable {
		opt(0) id: String,
		opt(1) name: String,
		opt(2) marker_image: String,
		opt(4) context_menu_items: Vec<ContextMenuItem>
	}
} }
define_packet! { TeleportToWorldMapMarker {
	fixed {
		opt marker_id: String
	}
} }
define_packet! { TeleportToWorldMapPosition { position_x: f32, position_y: f32 } }
define_packet! { UpdateWorldMap {
	variable {
		opt chunks: Vec<MapChunk>,
		opt added_markers: Vec<MapMarker>,
		opt removed_markers: Vec<String>
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
		opt biome_data_map: HashMap<i16, BiomeData>
	}
} }
define_packet! { UpdateWorldMapVisible { visible: bool } }
