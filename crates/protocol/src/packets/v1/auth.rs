#![allow(unused_variables, unused_imports)]

use bytes::Bytes;

use crate::{
	codec::HytaleCodec,
	define_packet,
	packets::HostAddress,
};

define_packet!(
	AuthGrant {
		fixed {}
		variable {
			opt auth_grant: String,
			opt server_identity: String,
		}
	}
);

define_packet!(
	AuthToken {
		fixed {}
		variable {
			opt access_token: String,
			opt server_grant: String,
		}
	}
);

define_packet!(
	ClientReferral {
		fixed {}
		variable {
			opt host_to: HostAddress,
			opt data: Bytes
		}
	}
);

define_packet!(
	ConnectAccept {
		bitmask {
			opt password_challenge: Bytes
		}
	}
);

// Empty signal packet
define_packet!(PasswordAccepted {});

define_packet!(
	PasswordRejected {
		bitmask {
			required attempts_remaining: i32,
			opt new_challenge: Bytes,
		}
	}
);

define_packet!(
	PasswordResponse {
		bitmask {
			opt hash: Bytes
		}
	}
);

define_packet!(
	ServerAuthToken {
		fixed {}
		variable {
			opt server_access_token: String,
			opt password_challenge: Bytes,
		}
	}
);

define_packet!(
	Status {
		bitmask {
			required player_count: i32,
			required max_players: i32,
			opt name: String,
			opt motd: String,
		}
	}
);
