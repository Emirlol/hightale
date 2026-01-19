#![allow(unused_variables, unused_imports)]

use bytes::Bytes;

use super::HostAddress;
use crate::{
	codec::HytaleCodec,
	define_packet,
};

define_packet! {
   AuthGrant {
	   variable {
		   opt auth_grant: String,
		   opt server_identity: String,
	   }
   }
}

define_packet! {
   AuthToken {
	   variable {
		   opt access_token: String,
		   opt server_grant: String,
	   }
   }
}

define_packet! {
   ClientReferral {
	   variable {
		   opt host_to: HostAddress,
		   opt data: Bytes
	   }
   }
}

define_packet! { 
	ConnectAccept {
		fixed {
			opt password_challenge: Bytes
		}
	}
 } 
// Empty signal packet
define_packet! { PasswordAccepted {} }

define_packet! { 
	PasswordRejected {
		fixed {
			required attempts_remaining: i32,
			opt new_challenge: Bytes,
		}
	}
 } 
define_packet! { 
	PasswordResponse {
		fixed {
			opt hash: Bytes
		}
	}
 } 
define_packet! { 
	ServerAuthToken {
		variable {
			opt server_access_token: String,
			opt password_challenge: Bytes,
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
			opt name: String,
			opt motd: String,
		}
	}
 } 