use std::collections::HashMap;

use crate::{
	define_enum,
	define_packet,
	v1::{
		ItemQuantity,
		ItemWithAllMetadata,
		SortType,
	},
};

define_packet! { DropCreativeItem { item: ItemQuantity } }

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

define_packet! { SetCreativeItem {
	inventory_section_id: i32,
	slot_id: i32,
	r#override: bool, // "override" is a reserved keyword
	item: ItemQuantity
} }

define_enum! {
	pub enum SmartMoveType {
		EquipOrMergeStack = 0,
		PutInHotbarOrWindow = 1,
		PutInHotbarOrBackpack = 2,
	}
}

define_packet! { SmartGiveCreativeItem {
	move_type: SmartMoveType,
	item: ItemQuantity
} }

define_packet! { SmartMoveItemStack {
	from_section_id: i32,
	from_slot_id: i32,
	quantity: i32,
	move_type: SmartMoveType,
} }

define_packet! {
	SwitchHotbarBlockSet {
		fixed {
			opt item_id: String
		}
	}
}

define_packet! {
	InventorySection {
		fixed {
			required capacity: i16,
			opt items: HashMap<i32, ItemWithAllMetadata>
		}
	}
}

define_packet! {
	UpdatePlayerInventory {
		fixed {
			required sort_type: SortType,
		}
		variable {
			opt storage: InventorySection,
			opt armor: InventorySection,
			opt hotbar: InventorySection,
			opt utility: InventorySection,
			opt builder_material: InventorySection,
			opt tools: InventorySection,
			opt backpack: InventorySection,
		}
	}
}
