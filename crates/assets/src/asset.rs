use std::{
	fmt::Debug,
	hash::Hash,
};

use crate::validate::{
	ValidationContext,
	ValidationResults,
};

pub trait Asset: Sized {
	type Key: Clone + Eq + Hash + Debug + Send + Sync + 'static;

	fn key(&self) -> &Self::Key;
}

pub trait Validate: Asset {
	fn validate(&self, _ctx: &ValidationContext<Self::Key>, _out: &mut ValidationResults<Self::Key>) {
		// default no-op
	}
}
