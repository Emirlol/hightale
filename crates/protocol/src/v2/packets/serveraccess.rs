use macros::define_packet;

use crate::{
	define_enum,
	v2::HostAddress,
};

define_enum! {
	pub enum Access {
		Private = 0,
		LAN = 1,
		Friend = 2,
		Open = 3
	}
}

define_packet! { RequestServerAccess { access: Access, port: u16 } }

define_packet! {
	SetServerAccess {
		fixed {
			required access: Access,
		}
		variable {
			opt(1) password: String
		}
	}
}

define_packet! {
	UpdateServerAccess {
		fixed {
			required access: Access,
		}
		variable {
			opt(1) hosts: Vec<HostAddress>
		}
	}
}
