use bytes::Bytes;
use macros::define_packet;

use crate::v2::HostAddress;

define_packet! {
	AuthGrant {
		variable {
			opt(1) auth_grant: String,
			opt(2) server_identity: String,
		}
	}
}

define_packet! {
	AuthToken {
		variable {
			opt(1) access_token: String,
			opt(2) server_grant: String,
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
			opt(1) server_access_token: String,
			opt(2) password_challenge: Bytes,
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
			opt(1) name: String,
			opt(2) motd: String,
		}
	}
}
