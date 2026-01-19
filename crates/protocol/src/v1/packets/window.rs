use super::{
	inventory::InventorySection,
	ExtraResources,
	SortType,
	WindowAction,
	WindowType,
};
use crate::define_packet;

define_packet! { CancelCraftingAction {} }
define_packet! { ChangeBlockAction { down: bool } }
define_packet! { ClientOpenWindow { window_type: WindowType } }
define_packet! { CloseWindow { id: i32 } }
define_packet! { CraftItemAction { } }
define_packet! { CraftRecipeAction {
	fixed {
		required quantity: i32,
		opt recipe_id: String
	}
} }
define_packet! { OpenWindow {
	fixed {
		required id: i32,
		required window_type: WindowType,
	}
	variable {
		opt window_data: String,
		opt inventory: InventorySection,
		opt extra_resources: ExtraResources
	}
} }
define_packet! { SelectSlotAction { slot: i32 } }
define_packet! { SendWindowAction { id: i32, action: WindowAction } }
define_packet! { SetActiveAction { state: bool } }
define_packet! { SortItemsAction { sort_type: SortType } }
define_packet! { TierUpgradeAction { } }
define_packet! { UpdateCategoryAction {
	fixed {
		required category: String,
		required item_category: String,
	}
} }
define_packet! { UpdateWindow {
	fixed {
		required id: i32,
	}
	variable {
		opt window_data: String,
		opt inventory: InventorySection,
		opt extra_resources: ExtraResources
	}
} }
