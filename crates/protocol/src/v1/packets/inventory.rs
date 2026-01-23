use std::collections::HashMap;

use macros::define_packet;

use crate::{
	define_enum,
	v1::{
		ItemQuantity,
		ItemWithAllMetadata,
		SortType,
	},
};

define_packet! { DropCreativeItem {
	variable {
		required item: ItemQuantity
	}
} }

define_packet! { DropItemStack {
	inventory_section_id: i32,
	slot_id: i32,
	quantity: i32
} }

define_enum! {
	pub enum InventoryActionType {
		TakeAll = 0,
		PutAll = 1,
		QuickStack = 2,
		Sort = 3,
	}
}

define_packet! { InventoryAction {
	inventory_section_id: i32,
	inventory_action_type: InventoryActionType,
	action_data: u8
} }

define_packet! { MoveItemStack {
	from_section_id: i32,
	from_slot_id: i32,
	quantity: i32,
	to_section_id: i32,
	to_slot_id: i32,
} }

define_packet! { SetActiveSlot {
	inventory_section_id: i32,
	active_slot: i32
} }

define_packet! {
	SetCreativeItem {
		fixed {
			required inventory_section_id: i32,
			required slot_id: i32,
			required r#override: bool, // "override" is a reserved keyword
		}
		variable {
			required item: ItemQuantity
		}
	}
}

define_enum! {
	pub enum SmartMoveType {
		EquipOrMergeStack = 0,
		PutInHotbarOrWindow = 1,
		PutInHotbarOrBackpack = 2,
	}
}

define_packet! {
	SmartGiveCreativeItem {
		fixed {
			required move_type: SmartMoveType,
		}
		variable {
			required item: ItemQuantity
		}
	}
}

define_packet! { SmartMoveItemStack {
	from_section_id: i32,
	from_slot_id: i32,
	quantity: i32,
	move_type: SmartMoveType,
} }

define_packet! {
	SwitchHotbarBlockSet {
		variable {
			opt(1) item_id: String
		}
	}
}

define_packet! {
	InventorySection {
		fixed {
			required capacity: i16,
		}
		variable {
			opt(1) items: HashMap<i32, ItemWithAllMetadata>
		}
	}
}

define_packet! {
	UpdatePlayerInventory {
		fixed {
			required sort_type: SortType,
		}
		variable {
			opt(1) storage: InventorySection,
			opt(2) armor: InventorySection,
			opt(4) hotbar: InventorySection,
			opt(8) utility: InventorySection,
			opt(16) builder_material: InventorySection,
			opt(32) tools: InventorySection,
			opt(64) backpack: InventorySection,
		}
	}
}
