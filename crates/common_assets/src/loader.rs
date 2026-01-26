use std::{
	fs::File,
	io::Read,
	path::Path,
};

use assets::{
	StoreError,
	StoreResult,
};
use sha2::{
	Digest,
	Sha256,
};
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::common_asset::{
	CommonAsset,
	CommonAssetSource,
};

pub(crate) fn load_pack(pack_root: &Path) -> StoreResult<Vec<CommonAsset>> {
	if !pack_root.exists() {
		return Err(StoreError::NotFound(pack_root.display().to_string()));
	}

	if pack_root.is_dir() {
		return load_dir_pack(pack_root);
	}

	if is_zip_file(pack_root) {
		return load_zip_pack(pack_root);
	}

	Err(StoreError::NotFound(pack_root.display().to_string()))
}

pub(crate) fn load_dir(root: &Path) -> StoreResult<Vec<CommonAsset>> {
	load_dir_full(root)
}

pub(crate) fn load_dir_full(root: &Path) -> StoreResult<Vec<CommonAsset>> {
	if !root.exists() {
		return Ok(Vec::new());
	}

	let mut out = Vec::new();
	for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
		if !entry.file_type().is_file() {
			continue;
		}
		let path = entry.path();
		let rel = match path.strip_prefix(root) {
			Ok(p) => p,
			Err(_) => continue,
		};

		let name = normalize_name(&rel.to_string_lossy());
		if name.is_empty() || name.len() > 512 {
			continue;
		}

		let hash = sha256_file(path)?;

		out.push(CommonAsset {
			name,
			hash,
			source: CommonAssetSource::Dir(path.to_path_buf()),
		});
	}

	Ok(out)
}

fn load_dir_pack(pack_root: &Path) -> StoreResult<Vec<CommonAsset>> {
	let index_path = pack_root.join("CommonAssetsIndex.hashes");
	if index_path.is_file()
		&& let Ok(assets) = load_dir_index(&index_path)
	{
		return Ok(assets);
	}

	let common_root = pack_root.join("Common");
	load_dir_full(&common_root)
}

fn load_dir_index(index_path: &Path) -> StoreResult<Vec<CommonAsset>> {
	let contents = std::fs::read_to_string(index_path).map_err(StoreError::Io)?;
	let pack_root = index_path.parent().unwrap_or_else(|| Path::new("."));
	let common_root = pack_root.join("Common");

	let mut out = Vec::new();
	for line in contents.lines() {
		if let Some(stripped) = line.strip_prefix("VERSION=") {
			let version = stripped.trim().parse::<u32>().unwrap_or(u32::MAX);
			if version > 0 {
				return Err(StoreError::Decode(format!("Unsupported CommonAssetsIndex.hashes version {}", version)));
			}
			continue;
		}

		let mut parts = line.splitn(2, ' ');
		let hash = match parts.next() {
			Some(h) if is_hex_64(h) => h.to_lowercase(),
			_ => continue,
		};
		let name = match parts.next() {
			Some(n) => normalize_name(n),
			None => continue,
		};
		if name.is_empty() || name.len() > 512 {
			continue;
		}

		out.push(CommonAsset {
			source: CommonAssetSource::Dir(common_root.join(&name)),
			name,
			hash,
		});
	}

	Ok(out)
}

fn load_zip_pack(zip_path: &Path) -> StoreResult<Vec<CommonAsset>> {
	let mut file = File::open(zip_path).map_err(StoreError::Io)?;
	let mut archive = ZipArchive::new(&mut file).map_err(|e| StoreError::Decode(e.to_string()))?;

	if let Ok(assets) = load_zip_index(zip_path, &mut archive) {
		return Ok(assets);
	}

	let mut out = Vec::new();
	for i in 0..archive.len() {
		let mut entry = archive.by_index(i).map_err(|e| StoreError::Decode(e.to_string()))?;
		if entry.is_dir() {
			continue;
		}
		let name = normalize_name(entry.name());
		if !name.starts_with("Common/") {
			continue;
		}
		let rel_name = name.strip_prefix("Common/").unwrap_or(&name).to_string();
		if rel_name.is_empty() || rel_name.len() > 512 {
			continue;
		}

		let hash = sha256_reader(&mut entry)?;
		out.push(CommonAsset {
			name: rel_name,
			hash,
			source: CommonAssetSource::Zip {
				zip_path: zip_path.to_path_buf(),
				entry: name,
			},
		});
	}

	Ok(out)
}

fn load_zip_index(zip_path: &Path, archive: &mut ZipArchive<&mut File>) -> StoreResult<Vec<CommonAsset>> {
	let mut index_file = archive.by_name("CommonAssetsIndex.hashes").map_err(|e| StoreError::Decode(e.to_string()))?;
	let mut contents = String::new();
	index_file.read_to_string(&mut contents).map_err(StoreError::Io)?;

	let mut out = Vec::new();
	for line in contents.lines() {
		if let Some(stripped) = line.strip_prefix("VERSION=") {
			let version = stripped.trim().parse::<u32>().unwrap_or(u32::MAX);
			if version > 0 {
				return Err(StoreError::Decode(format!("unsupported CommonAssetsIndex.hashes version {}", version)));
			}
			continue;
		}

		let mut parts = line.splitn(2, ' ');
		let hash = match parts.next() {
			Some(h) if is_hex_64(h) => h.to_lowercase(),
			_ => continue,
		};
		let name = match parts.next() {
			Some(n) => normalize_name(n),
			None => continue,
		};
		if name.is_empty() || name.len() > 512 {
			continue;
		}

		let entry = format!("Common/{}", name);
		out.push(CommonAsset {
			source: CommonAssetSource::Zip {
				zip_path: zip_path.to_path_buf(),
				entry,
			},
			name,
			hash,
		});
	}

	Ok(out)
}

fn normalize_name(name: &str) -> String {
	if std::path::MAIN_SEPARATOR == '/' {
		return name.to_string();
	}
	name.replace(std::path::MAIN_SEPARATOR, "/")
}

fn is_hex_64(s: &str) -> bool {
	s.len() == 64 && s.as_bytes().iter().all(|b| b.is_ascii_hexdigit())
}

fn sha256_reader(reader: &mut impl Read) -> StoreResult<String> {
	let mut h = Sha256::new();
	let mut buf = [0u8; 64 * 1024];
	loop {
		let read = reader.read(&mut buf).map_err(StoreError::Io)?;
		if read == 0 {
			break;
		}
		h.update(&buf[..read]);
	}
	Ok(hex::encode(h.finalize()))
}

fn sha256_file(path: &Path) -> StoreResult<String> {
	let mut file = File::open(path).map_err(StoreError::Io)?;
	sha256_reader(&mut file)
}

fn is_zip_file(path: &Path) -> bool {
	path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("zip")).unwrap_or(false)
}
