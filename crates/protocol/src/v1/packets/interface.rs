#![allow(unused_variables)]

use std::collections::HashMap;

use uuid::Uuid;

use super::{
	CraftingRecipe,
	FormattedMessage,
	ItemWithAllMetadata,
	Vector3i,
};
use crate::{
	define_enum,
	define_packet,
};

define_packet! {
   AddToServerPlayerList {
	   fixed {
		   opt players: Vec<ServerPlayerListPlayer>
	   }
   }
}

define_packet! {
   BlockChange {
	   x: i32,
	   y: i32,
	   z: i32,
	   block_id: i32,
	   rotation: u8
   }
}

define_packet! {
   ChatMessage {
	   fixed {
		   opt message: String
	   }
   }
}

// Idk why these 2 are enums
define_enum! {
	pub enum ChatTagType {
		Item = 0
	}
}

define_enum! {
	pub enum ChatType {
		Chat = 0
	}
}

define_packet! {
   CustomHud {
	   fixed {
		   required clear: bool,
		   opt commands: Vec<CustomUICommand>
	   }
   }
}

define_packet! {
   CustomPage {
	   fixed {
		   required is_initial: bool,
		   required clear: bool,
		   required lifetime: CustomPageLifetime,
	   }
	   variable {
		   opt key: String,
		   opt commands: Vec<CustomUICommand>,
		   opt event_bindings: Vec<CustomUIEventBinding>,
	   }
   }
}

define_packet! {
   CustomPageEvent {
	   fixed {
		   required custom_page_event_type: CustomPageEventType,
		   opt data: String
	   }
   }
}

define_enum! {
	pub enum CustomPageEventType {
		Acknowledge = 0,
		Data = 1,
		Dismiss = 2,
	}
}

define_enum! {
	pub enum CustomPageLifetime {
		CantClose = 0,
		CanDismiss = 1,
		CanDismissOrCloseThroughInteraction = 2,
	}
}

define_packet! {
   CustomUICommand {
	   fixed {
		   required custom_ui_command_type: CustomUICommandType,
	   }
	   variable {
		   opt selector: String,
		   opt data: String,
		   opt text: String
	   }
   }
}

define_enum! {
	pub enum CustomUICommandType {
		Append = 0,
		AppendInline = 1,
		InsertBefore = 2,
		InsertBeforeInline = 3,
		Remove = 4,
		Set = 5,
		Clear = 6
	}
}

define_packet! {
   CustomUIEventBinding {
	   fixed {
		   required custom_ui_event_binding_type: CustomUIEventBindingType,
		   required lock_interface: bool,
	   }
	   variable {
		   opt selector: String,
		   opt data: String,
	   }
   }
}

define_enum! {
	pub enum CustomUIEventBindingType {
		Activating = 0,
		RightClicking = 1,
		DoubleClicking = 2,
		MouseEntered = 3,
		MouseExited = 4,
		ValueChanged = 5,
		ElementReordered = 6,
		Validating = 7,
		Dismissing = 8,
		FocusGained = 9,
		FocusLost = 10,
		KeyDown = 11,
		MouseButtonReleased = 12,
		SlotClicking = 13,
		SlotDoubleClicking = 14,
		SlotMouseEntered = 15,
		SlotMouseExited = 16,
		DragCancelled = 17,
		Dropped = 18,
		SlotMouseDragCompleted = 19,
		SlotMouseDragExited = 20,
		SlotClickReleaseWhileDragging = 21,
		SlotClickPressWhileDragging = 22,
		SelectedTabChanged = 23,
	}
}

define_packet! {
   EditorSelection {
	   min_pos: Vector3i,
	   max_pos: Vector3i
   }
}

define_packet! {
   EditorBlocksChange {
	   fixed {
		   opt(0) selection: EditorSelection [pad=24],
		   required blocks_count: i32,
		   required advanced_preview: bool,
	   }
	   variable {
		   opt(1) blocks_change: Vec<BlockChange>,
		   opt(2) fluids_change: Vec<FluidChange>,
	   }
   }
}

define_packet! {
   FluidChange {
	   pos: Vector3i,
	   fluid_id: i32,
	   fluid_level: u8
   }
}

define_packet! {
   HideEventTitle {
	   fade_out_duration: f32
   }
}

define_enum! {
	pub enum HudComponent {
		Hotbar = 0,
		StatusIcons = 1,
		Reticle = 2,
		Chat = 3,
		Requests = 4,
		Notifications = 5,
		KillFeed = 6,
		InputBindings = 7,
		PlayerList = 8,
		EventTitle = 9,
		Compass = 10,
		ObjectivePanel = 11,
		PortalPanel = 12,
		BuilderToolsLegend = 13,
		Speedometer = 14,
		UtilitySlotSelector = 15,
		BlockVariantSelector = 16,
		BuilderToolsMaterialSlotSelector = 17,
		Stamina = 18,
		AmmoIndicator = 19,
		Health = 20,
		Mana = 21,
		Oxygen = 22,
		Sleep = 23,
	}
}

define_packet! {
   KillFeedMessage {
	   fixed {}
	   variable {
		   opt killer: FormattedMessage,
		   opt decedent: FormattedMessage,
		   opt icon: String
	   }
   }
}

define_packet! {
   Notification {
	   fixed {
		   required style: NotificationStyle,
	   }
	   variable {
		   opt message: FormattedMessage,
		   opt secondary_message: FormattedMessage,
		   opt icon: String,
		   opt item: ItemWithAllMetadata,
	   }
   }
}
define_enum! {
	pub enum NotificationStyle {
		Default = 0,
		Danger = 1,
		Warning = 2,
		Success = 3
	}
}

define_packet! {
   OpenChatWithCommand {
	   fixed {
		   opt command: String
	   }
   }
}
define_enum! {
	pub enum Page {
		None = 0,
		Bench = 1,
		Inventory = 2,
		ToolsSettings = 3,
		Map = 4,
		MachinimaEditor = 5,
		ContentCreation = 6,
		Custom = 7,
	}
}

define_packet! {
   PortalDef {
	   fixed {
		   required exploration_seconds: i32,
		   required breach_seconds: i32,
		   opt name_key: String
	   }
   }
}
define_packet! { PortalState {
	remaining_seconds: i32,
	breaching: bool
} }
define_packet! {
   RemoveFromServerPlayerList {
	   fixed {
		   opt players: Vec<Uuid>,
	   }
   }
}
define_packet! { ResetUserInterfaceState {} }
define_packet! {
   ServerInfo {
	   fixed {
		   required max_players: i32
	   }
	   variable {
		   opt server_name: String,
		   opt motd: String
	   }
   }
}
define_packet! {
   ServerMessage {
	   fixed {
		   required chat_type: ChatType,
		   opt message: FormattedMessage
	   }
   }
}
define_packet! {
   ServerPlayerListPlayer {
	   fixed {
		   required uuid: Uuid,
		   opt(1) world_uuid: Uuid [pad=16],
		   required pin: i32,
		   opt(0) username: String,
	   }
   }
}
define_packet! { ServerPlayerListUpdate { uuid: Uuid, world_uuid: Uuid } }
define_packet! { SetPage {
	page: Page,
	can_close_through_interaction: bool
} }
define_packet! {
   ShowEventTitle {
	   fixed {
		   required fade_in_duration: f32,
		   required fade_out_duration: f32,
		   required duration: f32,
		   required is_major: bool
	   }
	   variable {
		   opt icon: String,
		   opt primary_title: FormattedMessage,
		   opt secondary_title: FormattedMessage,
	   }
   }
}
define_packet! {
   UpdateKnownRecipes {
	   fixed {
		   opt known: HashMap<String, CraftingRecipe>
	   }
   }
}
define_packet! {
   UpdateLanguage {
	   fixed {
		   opt language: String
	   }
   }
}
define_packet! {
   UpdatePortal{
	   variable {
		   opt state: PortalState,
		   opt def: PortalDef,
	   }
   }
}
define_packet! {
   UpdateServerPlayerList {
	   fixed {
		   opt players: Vec<ServerPlayerListUpdate>
	   }
   }
}
define_packet! {
   UpdateServerPlayerListPing {
	   fixed {
		   opt players: HashMap<Uuid, i32>
	   }
   }
}
define_packet! {
   UpdateVisibleHudComponents {
	   fixed {
		   opt components: Vec<HudComponent>
	   }
   }
}
define_packet! { WorldSavingStatus { is_world_saving: bool } }
