pub(crate) mod objects;
pub(crate) mod packets;

pub use objects::*;
pub use packets::*;

pub const PROTOCOL_CRC: i32 = 1789265863;
pub const PROTOCOL_BUILD_NUMBER: i32 = 2;