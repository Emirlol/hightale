use bytes::Bytes;

use crate::{
	asset::Asset,
	error::{
		StoreError,
		StoreResult,
	},
};

pub trait AssetCodec<A: Asset>: Send + Sync {
	fn decode(&self, bytes: Bytes) -> StoreResult<A>;

	fn encode(&self, _asset: &A) -> StoreResult<Bytes> {
		Err(StoreError::Codec("encode not implemented".into()))
	}

	fn validate_defaults(&self) -> StoreResult<()> {
		Ok(())
	}
}
