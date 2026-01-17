#![allow(unused_variables, clippy::enum_variant_names)]

use std::collections::HashMap;

use bytes::{
	Buf,
	BufMut,
	Bytes,
	BytesMut,
};

use crate::{
	codec::{
		FixedAscii,
		HytaleCodec,
		PacketError,
		PacketResult,
		VarInt,
	},
	define_enum,
	define_packet,
};

pub mod auth;
pub mod camera;
pub mod connection;
pub mod interface;
pub mod serveraccess;
pub mod setup;

/// Max size for variable length items, like strings, maps, lists, etc.
pub const MAX_SIZE: i32 = 4_096_000;

#[derive(Debug, Clone)]
pub struct HostAddress {
	pub port: u16,
	pub host: String,
}

impl HytaleCodec for HostAddress {
	fn encode(&self, buf: &mut BytesMut) {
		self.port.encode(buf);
		self.host.encode(buf);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		Ok(HostAddress {
			port: <u16 as HytaleCodec>::decode(buf)?,
			host: <String as HytaleCodec>::decode(buf)?,
		})
	}
}

#[derive(Debug, Clone)]
pub struct Asset {
	pub hash: FixedAscii<64>, // 64-char Hex String
	pub name: String,         // Filename (e.g. "models/player.json")
}

impl HytaleCodec for Asset {
	fn encode(&self, buf: &mut BytesMut) {
		<FixedAscii<64> as HytaleCodec>::encode(&self.hash, buf);
		<String as HytaleCodec>::encode(&self.name, buf);
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		Ok(Asset {
			hash: <FixedAscii<64> as HytaleCodec>::decode(buf)?,
			name: <String as HytaleCodec>::decode(buf)?,
		})
	}
}

#[derive(Debug, Clone)]
pub struct InstantData {
	pub seconds: i64,
	pub nanos: i32,
}

impl HytaleCodec for InstantData {
	fn encode(&self, buf: &mut BytesMut) {
		<i64 as HytaleCodec>::encode(&self.seconds, buf);
		<i32 as HytaleCodec>::encode(&self.nanos, buf);
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		Ok(InstantData {
			seconds: <i64 as HytaleCodec>::decode(buf)?,
			nanos: <i32 as HytaleCodec>::decode(buf)?,
		})
	}
}

#[derive(Debug, Clone, Default)]
pub struct Vector2f {
	pub x: f32,
	pub y: f32,
}

impl HytaleCodec for Vector2f {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_f32_le(self.x);
		buf.put_f32_le(self.y);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 8 {
			return Err(PacketError::Incomplete);
		}
		Ok(Self {
			x: buf.get_f32_le(),
			y: buf.get_f32_le(),
		})
	}
}

#[derive(Debug, Clone, Default)]
pub struct Vector3f {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl HytaleCodec for Vector3f {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_f32_le(self.x);
		buf.put_f32_le(self.y);
		buf.put_f32_le(self.z);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 12 {
			return Err(PacketError::Incomplete);
		}
		Ok(Self {
			x: buf.get_f32_le(),
			y: buf.get_f32_le(),
			z: buf.get_f32_le(),
		})
	}
}

#[derive(Debug, Clone, Default)]
pub struct Position {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl HytaleCodec for Position {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_f64_le(self.x);
		buf.put_f64_le(self.y);
		buf.put_f64_le(self.z);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 12 {
			return Err(PacketError::Incomplete);
		}
		Ok(Self {
			x: buf.get_f64_le(),
			y: buf.get_f64_le(),
			z: buf.get_f64_le(),
		})
	}
}

define_enum! {
	pub enum PositionType {
		AttachedToPlusOffset = 0,
		Custom = 1
	}
}

#[derive(Debug, Clone, Default)]
pub struct Direction {
	pub yaw: f32,
	pub pitch: f32,
	pub roll: f32,
}

impl HytaleCodec for Direction {
	fn encode(&self, buf: &mut BytesMut) {
		buf.put_f32_le(self.yaw);
		buf.put_f32_le(self.pitch);
		buf.put_f32_le(self.roll);
	}
	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		if buf.remaining() < 12 {
			return Err(PacketError::Incomplete);
		}
		Ok(Self {
			yaw: buf.get_f32_le(),
			pitch: buf.get_f32_le(),
			roll: buf.get_f32_le(),
		})
	}
}

define_enum! {
	pub enum RotationType {
		AttachedToPlusOffset = 0,
		Custom = 1
	}
}

define_enum! {
	pub enum CanMoveType {
		AttachedToLocalPlayer = 0,
		Always = 1
	}
}

define_enum! {
	pub enum PositionDistanceOffsetType {
		DistanceOffset = 0,
		DistanceOffsetRaycast = 1,
		None = 2
	}
}

define_enum! {
	pub enum ApplyMovementType {
		CharacterController = 0,
		Position = 1
	}
}

define_enum! {
	pub enum ApplyLookType {
		LocalPlayerLookOrientation = 0,
		Rotation = 1
	}
}

define_enum! {
	pub enum MouseInputType {
		LookAtTarget = 0,
		LookAtTargetBlock = 1,
		LookAtTargetEntity = 2,
		LookAtPlane = 3
	}
}

define_enum! {
	pub enum AttachedToType {
		LocalPlayer = 0,
		EntityId = 1,
		None = 2
	}
}
define_enum! {
	pub enum MovementForceRotationType {
		AttachedToHead = 0,
		CameraRotation = 1,
		Custom = 2
	}
}

define_enum! {
	pub enum MouseInputTargetType {
		Any = 0,
		Block = 1,
		Entity = 2,
		None = 3
	}
}

define_enum! {
	pub enum MaybeBool {
		Null = 0,
		False = 1,
		True = 2
	}
}

define_packet! {
	StringParamValue {
		bitmask {
			opt(0) value: String
		}
	}
}

#[derive(Debug, Clone)]
pub enum ParamValue {
	String(StringParamValue),
	Bool(bool),
	Double(f64),
	Int(i32),
	Long(i64),
}

impl HytaleCodec for ParamValue {
	fn encode(&self, buf: &mut BytesMut) {
		match self {
			ParamValue::String(v) => {
				VarInt(0).encode(buf);
				v.encode(buf);
			}
			ParamValue::Bool(v) => {
				VarInt(1).encode(buf);
				v.encode(buf);
			}
			ParamValue::Double(v) => {

				VarInt(2).encode(buf);
				v.encode(buf);
			}
			ParamValue::Int(v) => {
				VarInt(3).encode(buf);
				v.encode(buf);
			}
			ParamValue::Long(v) => {
				VarInt(4).encode(buf);
				v.encode(buf);
			}
		}
	}

	fn decode(buf: &mut impl Buf) -> PacketResult<Self> {
		let type_id = VarInt::decode(buf)?.0;

		match type_id {
			0 => Ok(ParamValue::String(<StringParamValue as HytaleCodec>::decode(buf)?)),
			1 => Ok(ParamValue::Bool(<bool as HytaleCodec>::decode(buf)?)),
			2 => Ok(ParamValue::Double(<f64 as HytaleCodec>::decode(buf)?)),
			3 => Ok(ParamValue::Int(<i32 as HytaleCodec>::decode(buf)?)),
			4 => Ok(ParamValue::Long(<i64 as HytaleCodec>::decode(buf)?)),
			_ => Err(PacketError::InvalidEnumVariant(type_id as u8)),
		}
	}
}

define_packet! {
	FormattedMessage {
		fixed {
			bold: MaybeBool,
			italic: MaybeBool,
			monospace: MaybeBool,
			underlined: MaybeBool,
			markup_enabled: bool,
		}
		variable {
			opt raw_text: String,
			opt message_id: String,
			opt children: Vec<FormattedMessage>,
			opt params: HashMap<String, ParamValue>,
			opt message_params: HashMap<String, FormattedMessage>,
			opt color: String,
			opt link: String
		}
	}
}

define_packet!(
	ItemWithAllMetadata {
		fixed {
			quantity: i32,
			durability: f64,
			max_durability: f64,
			override_dropped_item_animation: bool
		}
		variable {
			required item_id: String,
			opt metadata: String
		}
	}
);

define_packet! {
	MaterialQuantity {
		fixed {
			item_tag: i32,
			quantity: i32
		}
		variable {
			opt item_id: String,
			opt resource_type_id: String
		}
	}
}

define_enum!(
	pub enum BenchType {
		Crafting = 0,
		Processing = 1,
		DiagramCrafting = 2,
		StructuralCrafting = 3,
	}
);

define_packet!(
	BenchRequirement {
		fixed {
			bench_type: BenchType,
			required_tier_level: i32,
		}
		variable {
			opt id: String,
			opt categories: Vec<String>
		}
	}
);

define_packet!(
	CraftingRecipe {
		fixed {
			knowledge_required: bool,
			time_seconds: f32,
			required_memories_level: i32
		}
		variable {
			opt id: String,
			opt inputs: Vec<MaterialQuantity>,
			opt outputs: Vec<MaterialQuantity>,
			opt primary_output: MaterialQuantity,
			opt bench_requirement: BenchRequirement,
		}
	}
);


// Helper struct so we can check compression before decoding
pub struct PacketInfo {
	pub compressed: bool,
}

macro_rules! packet_enum {
    (
        // Syntax: ID => Variant(Type) [compressed?]
        $( $id:literal => $variant:ident($module:ident::$st:ident) $( [ $compressed:tt ] )? ),* $(,)?
    ) => {
        #[derive(Debug, Clone)]
        pub enum Packet {
            $( $variant($module::$st), )*
            Unknown(i32, Bytes),
        }

        impl Packet {
            pub fn id(&self) -> i32 {
                match self {
                    $( Packet::$variant(_) => $id, )*
                    Packet::Unknown(id, _) => *id,
                }
            }

            pub fn is_compressed(&self) -> bool {
                match self {
                    // If [compressed] is present, return true, else false
                    $( Packet::$variant(_) => {
                        0 $( + is_compressed_helper!($compressed) )? == 1
                    }, )*
                    _ => false
                }
            }

            pub fn encode(&self, buf: &mut BytesMut) {
                match self {
                    $( Packet::$variant(pkt) => pkt.encode(buf), )*
                    Packet::Unknown(_, data) => buf.extend_from_slice(data),
                }
            }

            pub fn decode(id: i32, buf: &mut impl Buf) -> PacketResult<Self> {
                match id {
                    $(
                        $id => {
                            let pkt = $module::$st::decode(buf)?;
                            Ok(Packet::$variant(pkt))
                        }
                    )*
                    _ => {
                        Ok(Packet::Unknown(id, buf.copy_to_bytes(buf.remaining())))
                    }
                }
            }
        }

        // Helper to lookup compression BEFORE decoding (needed for reading frames)
        pub fn is_id_compressed(id: i32) -> bool {
            match id {
                 $(
                    $id => {
                        0 $( + is_compressed_helper!($compressed) )? == 1
                    },
                 )*
                 _ => false
            }
        }

        // Auto-impl From<Struct> for Packet
        $(
            impl From<$module::$st> for Packet {
                fn from(p: $module::$st) -> Self {
                    Packet::$variant(p)
                }
            }
        )*
    };
}

// Helper macro to detect presence of the token
macro_rules! is_compressed_helper {
	(compressed) => {
		1
	};
}

packet_enum! {
	// Connection
	0 => Connect(connection::Connect),
	1 => Disconnect(connection::Disconnect),
	2 => Ping(connection::Ping),
	3 => Pong(connection::Pong),

	// Auth
	10 => Status(auth::Status),
	11 => AuthGrant(auth::AuthGrant),
	12 => AuthToken(auth::AuthToken),
	13 => ServerAuthToken(auth::ServerAuthToken),
	14 => ConnectAccept(auth::ConnectAccept),
	15 => PasswordResponse(auth::PasswordResponse),
	16 => PasswordAccepted(auth::PasswordAccepted),
	17 => PasswordRejected(auth::PasswordRejected),
	18 => ClientReferral(auth::ClientReferral),

	// Setup
	20 => WorldSettings(setup::WorldSettings) [compressed],
	21 => WorldLoadProgress(setup::WorldLoadProgress),
	22 => WorldLoadFinished(setup::WorldLoadFinished),
	23 => RequestAssets(setup::RequestAssets) [compressed],
	24 => AssetInitialize(setup::AssetInitialize),
	25 => AssetPart(setup::AssetPart),
	26 => AssetFinalize(setup::AssetFinalize),
	27 => RemoveAssets(setup::RemoveAssets),
	28 => RequestCommonAssetsRebuild(setup::RequestCommonAssetsRebuild),
	29 => SetUpdateRate(setup::SetUpdateRate),
	30 => SetTimeDilation(setup::SetTimeDilation),
	31 => UpdateFeatures(setup::UpdateFeatures),
	32 => ViewRadius(setup::ViewRadius),
	33 => PlayerOptions(setup::PlayerOptions),
	34 => ServerTags(setup::ServerTags),

	// Interface
	210 => ServerMessage(interface::ServerMessage),
	211 => ChatMessage(interface::ChatMessage),
	212 => Notification(interface::Notification),
	213 => KillFeedMessage(interface::KillFeedMessage),
	214 => ShowEventTitle(interface::ShowEventTitle),
	215 => HideEventTitle(interface::HideEventTitle),
	216 => SetPage(interface::SetPage),
	217 => CustomHud(interface::CustomHud) [compressed],
	218 => CustomPage(interface::CustomPage) [compressed],
	219 => CustomPageEvent(interface::CustomPageEvent),
	222 => EditorBlocksChange(interface::EditorBlocksChange) [compressed],
	223 => ServerInfo(interface::ServerInfo),
	224 => AddToServerPlayerList(interface::AddToServerPlayerList),
	225 => RemoveFromServerPlayerList(interface::RemoveFromServerPlayerList),
	226 => UpdateServerPlayerList(interface::UpdateServerPlayerList),
	227 => UpdateServerPlayerListPing(interface::UpdateServerPlayerListPing),
	228 => UpdateKnownRecipes(interface::UpdateKnownRecipes),
	229 => UpdatePortal(interface::UpdatePortal),
	230 => UpdateVisibleHudComponents(interface::UpdateVisibleHudComponents),
	231 => ResetUserInterfaceState(interface::ResetUserInterfaceState),
	232 => UpdateLanguage(interface::UpdateLanguage),
	233 => WorldSavingStatus(interface::WorldSavingStatus),
	234 => OpenChatWithCommand(interface::OpenChatWithCommand),

	// Server Access
	250 => RequestServerAccess(serveraccess::RequestServerAccess),
	251 => UpdateServerAccess(serveraccess::UpdateServerAccess),
	252 => SetServerAccess(serveraccess::SetServerAccess),

	// Camera
	280 => SetServerCamera(camera::SetServerCamera),
	281 => CameraShakeEffect(camera::CameraShakeEffect),
	282 => RequestFlyCameraMode(camera::RequestFlyCameraMode),
	283 => SetFlyCameraMode(camera::SetFlyCameraMode),
}