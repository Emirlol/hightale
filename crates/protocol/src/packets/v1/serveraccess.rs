use crate::{define_enum, define_packet};
use crate::packets::HostAddress;

define_enum! {
	pub enum Access {
		Private = 0,
		LAN = 1,
		Friend = 2,
		Open = 3
	}
}

define_packet!(
	RequestServerAccess {
		access: Access,
		port: u16
	}
);

define_packet!(
	SetServerAccess {
		fixed {
			required access: Access,
			opt password: String
		}
	}
);

define_packet!(
	UpdateServerAccess {
		fixed {
			required access: Access,
			opt hosts: Vec<HostAddress>
		}
	}
);