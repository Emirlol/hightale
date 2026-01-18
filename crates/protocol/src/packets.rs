// Reexport the latest version of the protocol. I don't know if we'll ever need multiple versions at the same time since the client requires updating before launching, but just in case.
mod v1;
pub use v1::{
	auth,
	camera,
	connection,
	entities,
	interaction,
	interface,
	inventory,
	is_id_compressed,
	player,
	serveraccess,
	setup,
	window,
	world,
	Asset,
	HostAddress,
	Packet,
	PacketInfo,
	MAX_SIZE,
};
