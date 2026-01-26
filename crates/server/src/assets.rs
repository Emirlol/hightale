//! Asset pack loaders used by the server.t

use std::path::Path;

use anyhow::{Context, Result};
use common_assets::CommonAssetStore;
use tracing::info;

pub fn load_common_assets(pack_root: &Path) -> Result<CommonAssetStore> {
	let mut store = CommonAssetStore::new();
	store.load_from_pack(pack_root).with_context(|| "Failed to load common asset pack")?;

	let total = store.required_assets().len();
	info!("Loaded {} common assets from {}", total, pack_root.display());

	Ok(store)
}
