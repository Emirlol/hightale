use std::marker::PhantomData;

use crate::asset::Asset;

#[derive(Debug, Clone)]
pub enum AssetEvent<A: Asset> {
	Loaded {
		added: usize,
		updated: usize,
	},
	Removed {
		removed: usize,
	},
	Generated {
		generated: usize,
	},
	#[doc(hidden)]
	__Marker(PhantomData<A>),
}

// Currently mostly unused
pub trait EventSink<A: Asset>: Send + Sync {
	fn emit(&self, event: AssetEvent<A>);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoopSink;

impl<A: Asset> EventSink<A> for NoopSink {
	fn emit(&self, _event: AssetEvent<A>) {}
}
