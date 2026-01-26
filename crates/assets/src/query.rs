use std::{
	collections::HashSet,
	hash::Hash,
};

#[derive(Debug, Clone, Default)]
pub enum QueryScope<K> {
	#[default]
	All,
	Only(HashSet<K>),
	Except(HashSet<K>),
}

#[derive(Debug, Clone, Copy, Default)]
pub enum UpsertPolicy {
	#[default]
	Upsert,
	AddOnly,
	UpdateOnly,
}

#[derive(Debug, Clone)]
pub struct AssetUpdateQuery<K>
where
	K: Eq + Hash,
{
	pub scope: QueryScope<K>,
	pub remove_missing: bool,
	pub upsert: UpsertPolicy,
	pub remove_keys: HashSet<K>,
}

impl<K> Default for AssetUpdateQuery<K>
where
	K: Eq + Hash,
{
	fn default() -> Self {
		Self {
			scope: QueryScope::default(),
			remove_missing: false,
			upsert: UpsertPolicy::default(),
			remove_keys: HashSet::new(),
		}
	}
}

impl<K> AssetUpdateQuery<K>
where
	K: Eq + Hash,
{
	pub fn new() -> Self {
		Self::default()
	}

	pub fn only(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self.scope = QueryScope::Only(HashSet::from_iter(keys));
		self
	}

	pub fn except(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self.scope = QueryScope::Except(HashSet::from_iter(keys));
		self
	}

	pub fn remove_missing(mut self, value: bool) -> Self {
		self.remove_missing = value;
		self
	}

	pub fn upsert_policy(mut self, policy: UpsertPolicy) -> Self {
		self.upsert = policy;
		self
	}

	pub fn remove_keys(mut self, keys: HashSet<K>) -> Self {
		self.remove_keys = keys;
		self
	}

	pub fn allows(&self, key: &K) -> bool {
		match &self.scope {
			QueryScope::All => true,
			QueryScope::Only(set) => set.contains(key),
			QueryScope::Except(set) => !set.contains(key),
		}
	}
}
