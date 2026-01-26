use bytes::Bytes;
use macros::define_packet;

use crate::{
	codec::BoundedVarLen,
	v2::HostAddress,
};

define_packet! {
	AuthGrant {
		variable {
			opt(1) auth_grant: BoundedVarLen<String, 4096>,
			opt(2) server_identity: BoundedVarLen<String, 8192>,
		}
	}
}

define_packet! {
	AuthToken {
		variable {
			opt(1) access_token: BoundedVarLen<String, 8192>,
			opt(2) server_grant: BoundedVarLen<String, 4096>,
		}
	}
}

define_packet! {
	ClientReferral {
		variable {
			opt(1) host_to: HostAddress,
			opt(2) data: Bytes
		}
	}
}

define_packet! {
	ConnectAccept {
		variable {
			opt(1) password_challenge: Bytes
		}
	}
}

define_packet! { PasswordAccepted }

define_packet! {
	PasswordRejected {
		fixed {
			required attempts_remaining: i32,
		}
		variable {
			opt(1) new_challenge: Bytes,
		}
	}
}

define_packet! {
	PasswordResponse {
		variable {
			opt(1) hash: Bytes
		}
	}
}
define_packet! {
	ServerAuthToken {
		variable {
			opt(1) server_access_token: BoundedVarLen<String, 8192>,
			opt(2) password_challenge: BoundedVarLen<Bytes, 64>,
		}
	}
}
define_packet! {
	Status {
		fixed {
			required player_count: i32,
			required max_players: i32,
		}
		variable {
			opt(1) name: BoundedVarLen<String, 128>,
			opt(2) motd: BoundedVarLen<String, 512>,
		}
	}
}
