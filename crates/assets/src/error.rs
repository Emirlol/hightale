use std::io::Error;

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
	#[error("Codec error: {0}")]
	Codec(String),

	#[error("Validation error: {0}")]
	Validation(String),

	#[error("I/O error: {0}")]
	Io(#[source] Error),

	#[error("Decode failed: {0}")]
	Decode(String),

	#[error("Duplicate key")]
	DuplicateKey,

	#[error("Not found")]
	NotFound,
}

pub type StoreResult<T> = Result<T, StoreError>;
