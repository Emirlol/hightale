use assets::{
	Asset as StoreAsset,
	StoreError,
	StoreResult,
};
use std::path::PathBuf;
use bytes::Bytes;
use zip::ZipArchive;
use std::fs::File;
use std::io::Read;
use protocol::{
	codec::FixedAscii,
	v2,
};

#[derive(Clone, Debug)]
pub struct CommonAsset {
	pub name: String, // relative path with forward slashes
	pub hash: String, // sha256 hex, lowercase
	pub source: CommonAssetSource,
}

#[derive(Clone, Debug)]
pub enum CommonAssetSource {
	Dir(PathBuf),
	Zip { zip_path: PathBuf, entry: String },
}

impl CommonAsset {
	pub fn to_protocol(&self) -> v2::Asset {
		v2::Asset {
			hash: FixedAscii::from(self.hash.as_str()),
			name: self.name.clone().into(),
		}
	}

	pub fn load_bytes(&self) -> StoreResult<Bytes> {
		match &self.source {
			CommonAssetSource::Dir(path) => {
				let bytes = std::fs::read(path).map_err(StoreError::Io)?;
				Ok(Bytes::from(bytes))
			}
			CommonAssetSource::Zip { zip_path, entry } => {
				let file = File::open(zip_path).map_err(StoreError::Io)?;
				let mut archive = ZipArchive::new(file).map_err(|e| StoreError::Decode(e.to_string()))?;
				let mut zip_file = archive.by_name(entry).map_err(|e| StoreError::Decode(e.to_string()))?;
				let mut buf = Vec::with_capacity(zip_file.size() as usize);
				zip_file.read_to_end(&mut buf).map_err(StoreError::Io)?;
				Ok(Bytes::from(buf))
			}
		}
	}
}

impl StoreAsset for CommonAsset {
	type Key = String;

	fn key(&self) -> &Self::Key {
		&self.hash
	}
}
