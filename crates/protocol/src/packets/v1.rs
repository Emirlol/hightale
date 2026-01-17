use bytes::{
	Buf,
	Bytes,
	BytesMut,
};

use crate::codec::{
	FixedAscii,
	HytaleCodec,
	PacketResult,
};

pub mod auth;

pub mod connection;

pub mod setup;

pub mod interface;

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

	// Auth
	10 => Status(auth::Status),
	11 => AuthGrant(auth::AuthGrant),
	12 => AuthToken(auth::AuthToken),
	13 => ServerAuthToken(auth::ServerAuthToken),
	14 => ConnectAccept(auth::ConnectAccept),

	// Setup
	20 => WorldSettings(setup::WorldSettings) [compressed],
	21 => WorldLoadProgress(setup::WorldLoadProgress),
	22 => WorldLoadFinished(setup::WorldLoadFinished),
	23 => RequestAssets(setup::RequestAssets) [compressed],

	// Interface
	223 => ServerInfo(interface::ServerInfo)
}
