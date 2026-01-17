use crate::define_packet;
use crate::packets::Asset;

define_packet!(
	WorldSettings {
		fixed {
			world_height: i32,
		}
		variable {
			// If None, client assumes defaults.
			// If Some, client compares hashes.
			required_assets: Option<Vec<Asset>>,
		}
	}
);

define_packet!(
	WorldLoadProgress {
		fixed {
			percent_complete: i32,
			percent_complete_subitem: i32,
		}
		variable {
			status: Option<String>,
		}
	}
);

define_packet!(
	WorldLoadFinished {
		fixed {}
		variable {}
	}
);

define_packet!(
	RequestAssets {
		fixed {}
		variable {
			// The list of assets the client needs us to upload.
			// If empty or None, client is happy.
			assets: Option<Vec<Asset>>,
		}
	}
);
