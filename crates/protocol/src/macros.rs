/// Macro to define an enum and automatically implement Enumeration for it.
/// These are enums that are serialized/deserialized as their ordinals.
/// define_enum! {
///     pub enum MyState {
///         Handshake = 0,
///         Play = 1
///     }
/// }
#[macro_export]
macro_rules! define_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident = $val:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, protocol_macros::FixedSize)]
        #[repr(u8)]
        $vis enum $name {
            $($variant = $val),+
        }

        impl $name {
            #[allow(clippy::wrong_self_convention)]
            fn from_u8(v: u8) -> Option<Self> {
                match v {
                    $($val => Some(Self::$variant),)*
                    _ => None,
                }
            }
            #[allow(clippy::wrong_self_convention)]
            fn to_u8(&self) -> u8 {
                *self as u8
            }
        }

        impl $crate::codec::HytaleCodec for $name {
            fn encode(&self, buf: &mut bytes::BytesMut) -> $crate::codec::PacketResult<()> {
                <u8 as $crate::codec::HytaleCodec>::encode(&self.to_u8(), buf)?;
                Ok(())
            }

            fn decode(buf: &mut impl bytes::Buf) -> $crate::codec::PacketResult<Self> {
                use $crate::codec::PacketError;

                // No context here, just propagate upwards since the packets using this enum will already have the field name as context
                let val = <u8 as $crate::codec::HytaleCodec>::decode(buf)?;
                Self::from_u8(val).ok_or_else(|| PacketError::InvalidEnumVariant(val))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
}

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! id_dispatch {
    (
	    $(#[$outer:meta])*
        $packet:ident from $pkg:ident {
            $($id:literal => $name:ident),* $(,)?
        }
    ) => {
        $(#[$outer])*
        #[derive(Clone, Debug)]
        pub enum $packet {
            $(
                $name($pkg::$name),
            )*
        }

        impl crate::codec::HytaleCodec for $packet {
            fn decode(buf: &mut impl bytes::Buf) -> crate::codec::PacketResult<Self> {
                use crate::codec::PacketContext;
                let type_id = crate::codec::VarInt::decode(buf).context("id")?.0;

                match type_id {
                    $(
                        $id => Ok($packet::$name(
                            $pkg::$name::decode(buf).context(concat!("enum variant ", stringify!($name), " (", stringify!($id), ")"))?
                        )),
                    )*
                    _ => Err(crate::codec::PacketError::InvalidEnumVariant(type_id as u8)),
                }
            }

            fn encode(&self, buf: &mut bytes::BytesMut) -> crate::codec::PacketResult<()> {
                use crate::codec::PacketContext;
                match self {
                    $(
                        $packet::$name(inner) => {
                            crate::codec::VarInt($id).encode(buf).context("id")?;
                            inner.encode(buf).context("inner")?;
                        }
                    )*
                }
                Ok(())
            }
        }
    };

    // Without package source, defaults to current module
    (
	    $(#[$outer:meta])*
        $packet:ident {
            $($id:literal => $name:ident),* $(,)?
        }
    ) => {
        $(#[$outer])*
        #[derive(Clone, Debug)]
        pub enum $packet {
            $(
                $name($name),
            )*
        }

        impl crate::codec::HytaleCodec for $packet {
            fn decode(buf: &mut impl bytes::Buf) -> crate::codec::PacketResult<Self> {
                use crate::codec::PacketContext;
                let type_id = crate::codec::VarInt::decode(buf).context("id")?.0;

                match type_id {
                    $(
                        $id => Ok($packet::$name(
                            $name::decode(buf).context(concat!("enum variant ", stringify!($name), " (", stringify!($id), ")"))?
                        )),
                    )*
                    _ => Err(crate::codec::PacketError::InvalidEnumVariant(type_id as u8)),
                }
            }

            fn encode(&self, buf: &mut bytes::BytesMut) -> crate::codec::PacketResult<()> {
                use crate::codec::PacketContext;
                match self {
                    $(
                        $packet::$name(inner) => {
                            crate::codec::VarInt($id).encode(buf).context("id")?;
                            inner.encode(buf).context("inner")?;
                        }
                    )*
                }
                Ok(())
            }
        }
    };
}
