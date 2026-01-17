// Reexport the latest version of the protocol. I don't know if we'll ever need multiple versions at the same time since the client requires updating before launching, but just in case.
mod v1;
pub use v1::{
	auth,
	connection,
	interface,
	is_id_compressed,
	setup,
	Asset,
	HostAddress,
	Packet,
	PacketInfo,
};
