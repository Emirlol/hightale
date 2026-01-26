use std::collections::HashMap;

use bytes::Bytes;
use macros::define_packet;
use ordered_float::OrderedFloat;

use crate::{
	codec::BoundedVarLen,
	define_enum,
	v2::{
		BlockType,
		FormattedMessage,
		InstantData,
		Model,
		Vector3f,
	},
};

define_packet! {
	AssetEditorActivateButton {
		variable {
			opt(1) button_id: String
		}
	}
}

define_packet! {
	AssetEditorAsset {
		variable {
			opt(1) hash: String,
			opt(2) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorAssetListSetup {
		fixed {
			req is_read_only: bool,
			req can_be_deleted: bool,
			req tree: AssetEditorFileTree
		}
		variable {
			opt(1) pack: String,
			opt(2) paths: Vec<AssetEditorFileEntry>
		}
	}
}

define_packet! {
	AssetEditorAssetListUpdate {
		variable {
			opt(1) pack: String,
			opt(2) additions: Vec<AssetEditorFileEntry>,
			opt(4) deletions: Vec<AssetEditorFileEntry>
		}
	}
}

define_packet! {
	AssetEditorAssetPackSetup {
		variable {
			opt(1) packs: HashMap<String, AssetPackManifest>
		}
	}
}

define_packet! {
	AssetEditorAssetType {
		fixed {
			req is_colored_icon: bool,
			req editor_type: AssetEditorEditorType
		}
		variable {
			opt(1) id: String,
			opt(2) icon: String,
			opt(4) path: String,
			opt(8) file_extensionp: String
		}
	}
}

define_packet! {
	AssetEditorAssetUpdated {
		variable {
			opt(1) path: AssetPath,
			opt(2) data: Bytes
		}
	}
}
define_packet! {
	AssetEditorAuthorization {
		can_use: bool
	}
}

define_packet! {
	AssetEditorCapabilities {
		can_discard_assets: bool,
		can_edit_assets: bool,
		can_create_asset_packs: bool,
		can_edit_asset_packs: bool,
		can_delete_asset_packs: bool
	}
}

define_packet! {
	AssetEditorCreateAsset {
		fixed {
			req token: i32,
			opt(1) rebuild_caches: AssetEditorRebuildCaches
		}
		variable {
			opt(2) path: AssetPath,
			opt(4) data: Bytes,
			opt(8) button_id: String
		}
	}
}

define_packet! {
	AssetEditorCreateAssetPack {
		fixed {
			req token: i32
		}
		variable {
			opt(1) manifest: AssetPackManifest
		}
	}
}

define_packet! {
	AssetEditorCreateDirectory {
		fixed {
			req token: i32
		}
		variable {
			opt(1) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorDeleteAsset {
		fixed {
			req token: i32
		}
		variable {
			opt(1) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorDeleteAssetPack {
		variable {
			opt(1) id: String
		}
	}
}

define_packet! {
	AssetEditorDeleteDirectory {
		fixed {
			req token: i32
		}
		variable {
			opt(1) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorDiscardChanges {
		variable {
			opt(1) assets: Vec<TimestampedAssetReference>
		}
	}
}

define_enum! {
	pub enum AssetEditorEditorType {
		None = 0,
		Text = 1,
		JsonSource = 2,
		JsonConfig = 3,
		Model = 4,
		Texture = 5,
		Animation = 6,
	}
}

define_packet! {
	AssetEditorEnableAssetPack {
		fixed {
			req enabled: bool
		}
		variable {
			opt(1) id: String
		}
	}
}

define_packet! { AssetEditorExportAssetFinalize }

define_packet! {
	AssetEditorExportAssetInitialize {
		fixed {
			req size: i32,
			req failed: bool
		}
		variable {
			opt(1) asset: AssetEditorAsset,
			opt(2) old_path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorExportAssetPart {
		variable {
			opt(1) part: Bytes
		}
	}
}

define_packet! {
	AssetEditorExportAssets {
		variable {
			opt(1) paths: Vec<AssetPath>
		}
	}
}

define_packet! {
	AssetEditorExportComplete {
		variable {
			opt(1) assets: Vec<TimestampedAssetReference>
		}
	}
}

define_packet! {
	AssetEditorExportDeleteAssets {
		variable {
			opt(1) asset: Vec<AssetEditorAsset>
		}
	}
}

define_packet! {
	AssetEditorFetchAsset {
		fixed {
			req token: i32,
			req is_from_opened_tab: bool
		}
		variable {
			opt(1) path: AssetPath,
		}
	}
}

define_packet! {
	AssetEditorFetchAssetReply {
		fixed {
			req token: i32
		}
		variable {
			opt(1) contents: Bytes
		}
	}
}

define_packet! {
	AssetEditorFetchAutoCompleteData {
		fixed {
			req token: i32
		}
		variable {
			opt(1) dataset: String,
			opt(2) query: String
		}
	}
}

define_packet! {
	AssetEditorFetchAutoCompleteDataReply {
		fixed {
			req token: i32
		}
		variable {
			opt(1) results: Vec<String>
		}
	}
}

define_packet! {
	AssetEditorFetchJsonAssetWithParents {
		fixed {
			req token: i32,
			req is_from_opened_tab: bool
		}
		variable {
			opt(1) path: AssetPath,
		}
	}
}

define_packet! {
	AssetEditorFetchJsonAssetWithParentsReply {
		fixed {
			req token: i32
		}
		variable {
			opt(1) assets: HashMap<AssetPath, String>
		}
	}
}

define_packet! { AssetEditorFetchLastModifiedAssets }

define_packet! {
	AssetEditorFileEntry {
		fixed {
			req is_directory: bool
		}
		variable {
			opt(1) path: String
		}
	}
}

define_enum! {
	pub enum AssetEditorFileTree {
		Server = 0,
		Common = 1
	}
}

define_packet! { AssetEditorInitialize }

define_packet! {
	AssetEditorJsonAssetUpdated {
		variable {
			opt(1) path: String,
			opt(2) commands: Vec<JsonUpdateCommand>
		}
	}
}

define_packet! {
	AssetEditorLastModifiedAssets {
		variable {
			opt(1) assets: Vec<AssetInfo>
		}
	}
}

define_packet! {
	AssetEditorModifiedAssetsCount {
		count: i32
	}
}

define_packet! {
	AssetEditorPopupNotification {
		fixed {
			req popup_notification_type : AssetEditorPopupNotificationType
		}
	}
}

define_enum! {
	pub enum AssetEditorPopupNotificationType {
		Info = 0,
		Success = 1,
		Error = 2,
		Warning = 3
	}
}

define_packet! {
	AssetEditorPreviewCameraSettings {
		fixed {
			req model_scale: OrderedFloat<f32>,
			opt(1) camera_position: Vector3f,
			opt(2) camera_orientation: Vector3f
		}
	}
}

define_packet! {
	AssetEditorRebuildCaches {
		block_textures: bool,
		models: bool,
		model_textures: bool,
		map_geometry: bool,
		item_icons: bool
	}
}

define_packet! {
	AssetEditorRedoChanges {
		fixed {
			req token: i32
		}
		variable {
			opt(1) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorRenameAsset {
		fixed {
			req token: i32
		}
		variable {
			opt(1) path: AssetPath,
			opt(2) new_path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorRenameDirectory {
		fixed {
			req token: i32
		}
		variable {
			opt(1) path: AssetPath,
			opt(2) new_path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorRequestChildrenList {
		variable {
			opt(1) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorRequestChildrenListReply {
		variable {
			opt(1) path: AssetPath,
			opt(2) children_ids: Vec<String>
		}
	}
}

define_packet! {
	AssetEditorRequestDataset {
		variable {
			opt(1) name: String
		}
	}
}

define_packet! {
	AssetEditorRequestDatasetReply {
		variable {
			opt(1) name: String,
			opt(2) ids: Vec<String>
		}
	}
}

define_packet! {
	AssetEditorSelectAsset {
		variable {
			opt(1) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorSetGameTime {
		fixed {
			opt(1) game_time: InstantData,
			required paused: bool,
		}
	}
}

define_packet! {
	AssetEditorSetupAssetTypes {
		variable {
			opt(1) types: Vec<AssetEditorAssetType>
		}
	}
}

define_packet! {
	AssetEditorSetupSchemas {
		variable {
			opt(1) schemas: Vec<SchemaFile>
		}
	}
}

define_packet! {
	AssetEditorSubscribeModifiedAssetsChanges {
		subscribe: bool
	}
}

define_packet! {
	AssetEditorUndoChanges {
		fixed {
			req token: i32
		}
		variable {
			opt(1) path: AssetPath
		}
	}
}

define_packet! {
	AssetEditorUndoRedoReply {
		fixed {
			req token: i32
		}
		variable {
			opt(1) command: JsonUpdateCommand
		}
	}
}

define_packet! {
	AssetEditorUpdateAsset {
		fixed {
			req token: i32,
			req asset_index: i32,
		}
		variable {
			opt(1) asset_type: String,
			opt(2) path: AssetPath,
			opt(4) data: Bytes
		}
	}
}

define_packet! {
	AssetEditorUpdateAssetPack {
		variable {
			opt(1) id: String,
			opt(2) manifest: AssetPackManifest
		}
	}
}

define_packet! {
	AssetEditorUpdateJsonAsset {
		fixed {
			req token: i32,
			req asset_index: i32,
		}
		variable {
			opt(1) asset_type: String,
			opt(2) path: AssetPath,
			opt(4) commands: Vec<JsonUpdateCommand>
		}
	}
}

define_packet! {
	AssetEditorUpdateModelPreview {
		fixed {
			opt(1) camera: AssetEditorPreviewCameraSettings
		}
		variable {
			opt(2) path: AssetPath,
			opt(4) model: Box<Model>,
			opt(8) block: Box<BlockType>,
		}
	}
}

define_packet! {
	AssetEditorUpdateSecondsPerGameDay {
		daytime_duration_seconds: i32,
		nighttime_duration_seconds: i32
	}
}

define_packet! {
	AssetEditorUpdateWeatherPreviewLock {
		locked: bool
	}
}

define_packet! {
	AssetInfo {
		fixed {
			req is_deleted: bool,
			req is_new: bool,
			req last_modification_date: u64
		}
		variable {
			opt(1) path: AssetPath,
			opt(2) old_path: AssetPath,
			opt(4) last_modification_username: String
		}
	}
}

define_packet! {
	AssetPackManifest {
		variable {
			opt(1) name: String,
			opt(2) group: String,
			opt(4) website: String,
			opt(8) description: String,
			opt(16) version: String,
			opt(32) author_info: Vec<AuthorInfo>
		}
	}
}

define_packet! {
	#[derive(Hash, Eq, PartialEq)]
	AssetPath {
		variable {
			opt(1) pack: String,
			opt(2) path: String
		}
	}
}

define_packet! {
	AuthorInfo {
		variable {
			opt(1) name: String,
			opt(2) email: String,
			opt(3) url: String
		}
	}
}

define_packet! {
	FailureReply {
		fixed {
			req token: i32
		}
		variable {
			opt(1) message: FormattedMessage
		}
	}
}

define_packet! {
	JsonUpdateCommand {
		fixed {
			req command_type: JsonUpdateType,
			opt(1) rebuild_caches: AssetEditorRebuildCaches
		}
		variable {
			opt(2) path: Vec<String>,
			opt(4) value: String,
			opt(8) previous_value: String,
			opt(16) first_created_property: Vec<String>
		}
	}
}

define_enum! {
	pub enum JsonUpdateType {
		SetProperty = 0,
		InsertProperty = 1,
		RemoveProperty = 2,
	}
}

define_packet! {
	SchemaFile {
		variable {
			opt(1) content: BoundedVarLen<String, 16777215>
		}
	}
}

define_packet! {
	SuccessReply {
		fixed {
			req token: i32
		}
		variable {
			opt(1) message: FormattedMessage
		}
	}
}

define_packet! {
	TimestampedAssetReference {
		variable {
			opt(1) path: AssetPath,
			opt(2) timestamp: String
		}
	}
}

define_enum! {
	pub enum WriteUpdateType {
		Add = 0,
		Update = 1,
		Remove = 2
	}
}
