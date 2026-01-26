use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use bytes::Bytes;
use rayon::prelude::IntoParallelIterator;
use assets::{
	AssetCodec,
	AssetStore,
	HashMapIndex,
	InputRef,
	LoadOptions,
	StoreError,
	StoreResult,
	WithInput,
};
use protocol::v2;

use crate::{
	common_asset::CommonAsset,
	loader,
};

struct NoopCodec;

impl AssetCodec<CommonAsset> for NoopCodec {
	fn decode(&self, _bytes: Bytes) -> StoreResult<CommonAsset> {
		Err(StoreError::Decode("CommonAsset decode not supported".into()))
	}
}

pub struct CommonAssetStore {
	store: AssetStore<CommonAsset, HashMapIndex<CommonAsset>, Box<dyn AssetCodec<CommonAsset>>>,
	by_name: HashMap<String, CommonAsset>,
	name_order: Vec<String>,
}

impl Default for CommonAssetStore {
	fn default() -> Self {
		Self::new()
	}
}

impl CommonAssetStore {
	pub fn new() -> Self {
		let store = AssetStore::new(NoopCodec);
		Self {
			store,
			by_name: HashMap::new(),
			name_order: Vec::new(),
		}
	}

	pub fn load_from_pack(&mut self, pack_root: &std::path::Path) -> StoreResult<()> {
		let assets = loader::load_pack(pack_root)?;
		self.track_names(&assets);
		let items = assets.into_par_iter().map(|a| WithInput::new(InputRef::path(pack_root.to_path_buf()), a));
		self.store.load_assets(items, LoadOptions::report())?;
		Ok(())
	}

	pub fn load_from_dir(&mut self, root: &std::path::Path) -> StoreResult<()> {
		let assets = loader::load_dir(root)?;
		self.track_names(&assets);
		let items = assets.into_par_iter().map(|a| WithInput::new(InputRef::path(root.to_path_buf()), a));
		self.store.load_assets(items, LoadOptions::report())?;
		Ok(())
	}

	pub fn required_assets(&self) -> Vec<v2::Asset> {
		self.name_order
			.iter()
			.filter_map(|name| self.by_name.get(name))
			.map(|a| a.to_protocol())
			.collect()
	}

	pub fn get_by_hash(&self, hash: &str) -> Option<&CommonAsset> {
		self.store.get(&hash.to_lowercase())
	}

	pub fn get_by_name(&self, name: &str) -> Option<&CommonAsset> {
		self.by_name.get(name)
	}

	pub fn iter_all(&self) -> impl Iterator<Item = &CommonAsset> {
		self.store.iter().map(|(_, a)| a)
	}

	fn track_names(&mut self, assets: &[CommonAsset]) {
		for asset in assets {
			if !self.by_name.contains_key(&asset.name) {
				self.name_order.push(asset.name.clone());
			}
			self.by_name.insert(asset.name.clone(), asset.clone());
		}
	}
}
