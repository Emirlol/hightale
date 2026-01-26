use std::collections::{
	hash_map::Entry,
	HashMap,
};

use crate::{
	asset::Asset,
	error::*,
};

pub trait AssetIndex<A: Asset> {
	fn get(&self, key: &A::Key) -> Option<&A>;
	fn get_mut(&mut self, key: &A::Key) -> Option<&mut A>;

	fn insert(&mut self, asset: A) -> StoreResult<()>;
	fn upsert(&mut self, asset: A) -> UpsertResult;
	fn remove(&mut self, key: &A::Key) -> Option<A>;

	fn iter(&self) -> Box<dyn Iterator<Item = (&A::Key, &A)> + '_>;
	fn len(&self) -> usize;
	fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpsertResult {
	Added,
	Updated,
}

#[derive(Debug, Default)]
pub struct HashMapIndex<A: Asset> {
	map: HashMap<A::Key, A>,
}

impl<A: Asset> HashMapIndex<A> {
	pub fn new() -> Self {
		Self { map: HashMap::new() }
	}
}

impl<A: Asset> AssetIndex<A> for HashMapIndex<A> {
	fn get(&self, key: &A::Key) -> Option<&A> {
		self.map.get(key)
	}

	fn get_mut(&mut self, key: &A::Key) -> Option<&mut A> {
		self.map.get_mut(key)
	}

	fn insert(&mut self, asset: A) -> StoreResult<()> {
		let key = asset.key().clone();
		if self.map.contains_key(&key) {
			return Err(StoreError::DuplicateKey);
		}
		self.map.insert(key, asset);
		Ok(())
	}

	fn upsert(&mut self, asset: A) -> UpsertResult {
		let key = asset.key().clone();
		match self.map.entry(key) {
			Entry::Occupied(mut e) => {
				e.insert(asset);
				UpsertResult::Updated
			}
			Entry::Vacant(e) => {
				e.insert(asset);
				UpsertResult::Added
			}
		}
	}

	fn remove(&mut self, key: &A::Key) -> Option<A> {
		self.map.remove(key)
	}

	fn iter(&self) -> Box<dyn Iterator<Item = (&A::Key, &A)> + '_> {
		Box::new(self.map.iter())
	}

	fn len(&self) -> usize {
		self.map.len()
	}
}
