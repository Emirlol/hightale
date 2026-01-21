// Reexport the latest version of the protocol. I don't know if we'll ever need multiple versions at the same time since the client requires updating before launching, but just in case.
pub(crate) mod objects;
pub(crate) mod packets;

pub use objects::*;
pub use packets::*;
