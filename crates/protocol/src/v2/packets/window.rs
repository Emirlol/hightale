use protocol_macros::define_packet;

use crate::v2::{
	inventory::InventorySection,
	ExtraResources,
	SortType,
	WindowAction,
	WindowType,
};

define_packet! { CancelCraftingAction }
define_packet! { ChangeBlockAction { down: bool } }
define_packet! { ClientOpenWindow { window_type: WindowType } }
define_packet! { CloseWindow { id: i32 } }
define_packet! { CraftItemAction { } }
define_packet! {
	CraftRecipeAction {
		fixed {
			required quantity: i32,
		}
		variable {
			opt(1) recipe_id: String
		}
	}
}
define_packet! { OpenWindow {
	fixed {
		required id: i32,
		required window_type: WindowType,
	}
	variable {
		opt(1) window_data: String,
		opt(2) inventory: InventorySection,
		opt(4) extra_resources: ExtraResources
	}
} }
define_packet! { SelectSlotAction { slot: i32 } }
define_packet! {
	SendWindowAction {
		fixed {
			required id: i32,
		}
		variable {
			required action: WindowAction
		}
	}
}
define_packet! { SetActiveAction { state: bool } }
define_packet! { SortItemsAction { sort_type: SortType } }
define_packet! { TierUpgradeAction }
define_packet! {
	UpdateCategoryAction {
		variable {
			required category: String,
			required item_category: String,
		}
	}
}
define_packet! {
	UpdateWindow {
		fixed {
			required id: i32,
		}
		variable {
			opt(1) window_data: String,
			opt(2) inventory: InventorySection,
			opt(4) extra_resources: ExtraResources
		}
	}
}
