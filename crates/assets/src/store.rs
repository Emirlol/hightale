use std::{
	collections::HashSet,
	fs,
	io::Error,
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
};
use std::io::ErrorKind;
use bytes::Bytes;
use rayon::{
	iter::ParallelIterator,
	prelude::IntoParallelIterator,
};
use walkdir::WalkDir;

pub(crate) use crate::{
	asset::Asset,
	codec::AssetCodec,
	error::*,
	event::{
		AssetEvent,
		EventSink,
		NoopSink,
	},
	index::{
		AssetIndex,
		HashMapIndex,
		UpsertResult,
	},
	types::{
		prepared_err,
		prepared_ok,
		Analyzed,
		LoadState,
		Prepared,
	},
	AssetUpdateQuery,
	InputRef,
	LoadMode,
	LoadOptions,
	LoadOutcome,
	UpsertPolicy,
	ValidationContext,
	ValidationPolicy,
	ValidationResults,
	WithInput,
};

/// Blanket impl to allow using Box<dyn AssetCodec<A>>
impl<A, T> AssetCodec<A> for Box<T>
where
	A: Asset,
	T: AssetCodec<A> + ?Sized,
{
	fn decode(&self, bytes: Bytes) -> StoreResult<A> {
		(**self).decode(bytes)
	}

	fn encode(&self, asset: &A) -> StoreResult<Bytes> {
		(**self).encode(asset)
	}

	fn validate_defaults(&self) -> StoreResult<()> {
		(**self).validate_defaults()
	}
}

pub struct AssetStore<A, I, C>
where
	A: Asset,
	I: AssetIndex<A>,
	C: AssetCodec<A>,
{
	index: I,
	codec: C,
	events: Arc<dyn EventSink<A>>,
}

impl<A> AssetStore<A, HashMapIndex<A>, Box<dyn AssetCodec<A>>>
where
	A: Asset,
{
	pub fn new(codec: impl AssetCodec<A> + 'static) -> Self {
		AssetStoreBuilder::new().codec(Box::new(codec)).build().expect("Builder defaults should be valid")
	}
}

impl<A, I, C> AssetStore<A, I, C>
where
	A: Asset + Send,
	A::Key: Send,
	I: AssetIndex<A>,
	C: AssetCodec<A> + Sync,
{
	pub fn len(&self) -> usize {
		self.index.len()
	}

	pub fn is_empty(&self) -> bool {
		self.index.len() == 0
	}

	pub fn get(&self, key: &A::Key) -> Option<&A> {
		self.index.get(key)
	}

	pub fn insert(&mut self, asset: A) -> StoreResult<()> {
		self.index.insert(asset)
	}

	pub fn remove(&mut self, key: &A::Key) -> Option<A> {
		self.index.remove(key)
	}

	pub fn iter(&self) -> impl Iterator<Item = (&A::Key, &A)> {
		self.index.iter()
	}

	/// Load assets from a directory, recursively.
	pub fn load_dir(&mut self, root: impl AsRef<Path>, opts: LoadOptions<A>) -> StoreResult<LoadOutcome<A::Key>> {
		let root = root.as_ref().to_path_buf();
		if !root.exists() {
			return Err(StoreError::Io(Error::new(ErrorKind::NotFound, format!("Root path {:?} does not exist", root))));
		}

		let mode = opts.mode;
		let mut load_state = LoadState::new(root.clone(), mode);
		let mut paths: Vec<PathBuf> = Vec::new();

		let filter = &opts.filter;

		for entry in WalkDir::new(&root).into_iter() {
			let entry = match entry {
				Ok(e) => e,
				Err(e) => {
					let err = StoreError::Io(e.into_io_error().unwrap_or_else(|| Error::other("unknown walkdir error")));
					load_state.issue_or_err(WithInput::new(InputRef::path(root.clone()), err))?;
					continue;
				}
			};
			if !entry.file_type().is_file() {
				continue;
			}
			let path: PathBuf = entry.path().to_path_buf();
			if filter(&path) {
				paths.push(path);
			}
		}

		self.load_paths_impl(&mut load_state, paths, &opts)?;
		Ok(load_state.finish())
	}

	/// Load from raw paths to assets.
	pub fn load_paths(&mut self, paths: impl IntoParallelIterator<Item = PathBuf>, opts: LoadOptions<A>) -> StoreResult<LoadOutcome<A::Key>> {
		let LoadOptions { filter, .. } = &opts;

		let mut load_state = LoadState::new(PathBuf::new(), opts.mode);
		let filtered = paths.into_par_iter().filter(|p| filter(p));

		self.load_paths_impl(&mut load_state, filtered, &opts)?;
		Ok(load_state.finish())
	}

	/// Load from decoded bytes to assets.
	/// `LoadOptions` filtering is ignored at this stage.
	pub fn load_bytes(&mut self, inputs: impl IntoParallelIterator<Item = WithInput<Bytes>>, opts: LoadOptions<A>) -> StoreResult<LoadOutcome<A::Key>> {
		let mut load_state = LoadState::new(PathBuf::new(), opts.mode);
		self.load_bytes_impl(&mut load_state, inputs, &opts)?;
		Ok(load_state.finish())
	}

	/// Load from assets into the store.
	/// `LoadOptions` filtering is ignored at this stage.
	pub fn load_assets(&mut self, items: impl IntoParallelIterator<Item = WithInput<A>>, opts: LoadOptions<A>) -> StoreResult<LoadOutcome<A::Key>> {
		let mut load_state = LoadState::new(PathBuf::new(), opts.mode);
		self.load_assets_impl(&mut load_state, items, &opts)?;
		Ok(load_state.finish())
	}

	/// Load from raw paths to bytes.
	/// Expects paths to have already been filtered.
	fn load_paths_impl(&mut self, load_state: &mut LoadState<A::Key>, paths: impl IntoParallelIterator<Item = PathBuf>, opts: &LoadOptions<A>) -> StoreResult<()> {
		let prepared: Vec<Prepared<Bytes>> = paths
			.into_par_iter()
			.map(|path| match fs::read(&path) {
				Ok(b) => prepared_ok(InputRef::path(path), Bytes::from(b)),
				Err(e) => prepared_err(InputRef::path(path), StoreError::Io(e)),
			})
			.collect();

		let mut inputs: Vec<WithInput<Bytes>> = Vec::new();
		load_state.summary.files_seen += prepared.len();
		for item in prepared {
			match item {
				Ok(input) => inputs.push(input),
				Err(input) => load_state.issue_or_err(input)?,
			}
		}

		self.load_bytes_impl(load_state, inputs, opts)
	}

	/// Load from decoded bytes to assets.
	/// `LoadOptions` filtering is ignored at this stage.
	fn load_bytes_impl(&mut self, load_state: &mut LoadState<A::Key>, inputs: impl IntoParallelIterator<Item = WithInput<Bytes>>, opts: &LoadOptions<A>) -> StoreResult<()> {
		let codec = &self.codec;
		let prepared: Vec<Prepared<A>> = inputs
			.into_par_iter()
			.map(|input_bytes| match codec.decode(input_bytes.value) {
				Ok(a) => prepared_ok(input_bytes.input, a),
				Err(e) => {
					let err = match e {
						StoreError::Io(_) | StoreError::Decode(_) => e,
						other => StoreError::Decode(other.to_string()),
					};
					prepared_err(input_bytes.input, err)
				}
			})
			.collect();

		let mut assets: Vec<WithInput<A>> = Vec::new();
		for item in prepared {
			match item {
				Ok(input) => assets.push(input),
				Err(input) => load_state.issue_or_err(input)?,
			}
		}

		self.load_assets_impl(load_state, assets, opts)
	}

	/// Load from assets into the store, applying query, validation and upserting into the index.
	/// `LoadOptions` filtering is ignored at this stage.
	fn load_assets_impl(&mut self, load_state: &mut LoadState<A::Key>, items: impl IntoParallelIterator<Item = WithInput<A>>, opts: &LoadOptions<A>) -> StoreResult<()> {
		let query = &opts.query;
		let validation = &opts.validation;

		// Parallel analyze
		let analyzed: Vec<Analyzed<A>> = items
			.into_par_iter()
			.map(|WithInput { input, value: asset }| {
				let key = asset.key().clone();

				let mut issues = Vec::new();
				if !query.allows(&key) {
					return Analyzed::Skip { issues };
				}

				if let Some(vcfg) = validation {
					let ctx = ValidationContext::new(input.clone(), Some(key.clone()));
					let mut results = ValidationResults::<A::Key>::default();
					(vcfg.validator)(&asset, &ctx, &mut results);

					let has_errors = results.has_errors(); // Get the count before consuming the results into issues
					issues = results.into_issues();

					if has_errors {
						match vcfg.policy {
							ValidationPolicy::CollectOnly => {}
							ValidationPolicy::SkipInvalidAsset => {
								return Analyzed::Skip { issues };
							}
							ValidationPolicy::FailWholeLoad => {
								return Analyzed::Fail {
									input,
									error: StoreError::Validation(format!("validation failed for key {:?}", key)),
									issues,
								};
							}
						}
					}
				}

				Analyzed::Accept { input, key, asset, issues }
			})
			.collect();

		// Sequentially process analyzed results since we need to mutate the index and load state and those are not thread-safe
		for item in analyzed {
			load_state.summary.decoded += 1;

			match item {
				Analyzed::Accept { input, key, asset, issues } => {
					load_state.validation_issues.extend(issues);
					load_state.seen.insert(key.clone());
					load_state.summary.loaded.insert(key.clone(), input);

					match query.upsert {
						UpsertPolicy::Upsert => match self.index.upsert(asset) {
							UpsertResult::Added => load_state.summary.added += 1,
							UpsertResult::Updated => load_state.summary.updated += 1,
						},
						UpsertPolicy::AddOnly => {
							if self.index.get(&key).is_some() {
								continue;
							}
						}
						UpsertPolicy::UpdateOnly => {
							if self.index.get(&key).is_none() {
								continue;
							}
						}
					}
				}
				Analyzed::Skip { issues } => {
					load_state.validation_issues.extend(issues);
				}
				Analyzed::Fail { input, error, issues } => {
					load_state.validation_issues.extend(issues);
					match error {
						StoreError::Validation(_) => {
							if let LoadMode::Strict = load_state.mode {
								load_state.issue_or_err(WithInput::new(input, error))?
							} else {
								// Already recorded in validation issues
							}
						}
						other => load_state.issue_or_err(WithInput::new(input, other))?,
					}
				}
			}
		}

		for k in &query.remove_keys {
			if self.index.remove(k).is_some() {
				load_state.summary.removed += 1;
			}
		}

		if query.remove_missing {
			load_state.summary.removed += self.remove_missing_keys(&load_state.seen, query);
		}

		if load_state.summary.removed > 0 {
			self.events.emit(AssetEvent::Removed { removed: load_state.summary.removed });
		}

		self.events.emit(AssetEvent::Loaded {
			added: load_state.summary.added,
			updated: load_state.summary.updated,
		});

		Ok(())
	}

	fn remove_missing_keys(&mut self, seen: &HashSet<A::Key>, query: &AssetUpdateQuery<A::Key>) -> usize {
		let to_remove: Vec<A::Key> = self.index.iter().map(|(k, _)| k.clone()).filter(|k| query.allows(k) && !seen.contains(k)).collect();

		let mut removed = 0;
		for k in to_remove {
			if self.index.remove(&k).is_some() {
				removed += 1;
			}
		}
		removed
	}
}

pub struct AssetStoreBuilder<A, I, C>
where
	A: Asset,
	I: AssetIndex<A>,
	C: AssetCodec<A>,
{
	index: Option<I>,
	codec: Option<C>,
	events: Option<Arc<dyn EventSink<A>>>,
}

impl<A> Default for AssetStoreBuilder<A, HashMapIndex<A>, Box<dyn AssetCodec<A>>>
where
	A: Asset,
{
	fn default() -> Self {
		Self::new()
	}
}

impl<A> AssetStoreBuilder<A, HashMapIndex<A>, Box<dyn AssetCodec<A>>>
where
	A: Asset,
{
	pub fn new() -> Self {
		Self {
			index: Some(HashMapIndex::new()),
			codec: None,
			events: Some(Arc::new(NoopSink)),
		}
	}
}

impl<A, I, C> AssetStoreBuilder<A, I, C>
where
	A: Asset,
	I: AssetIndex<A>,
	C: AssetCodec<A>,
{
	pub fn index(mut self, index: I) -> Self {
		self.index = Some(index);
		self
	}

	pub fn codec(mut self, codec: C) -> Self {
		self.codec = Some(codec);
		self
	}

	pub fn events(mut self, sink: Arc<dyn EventSink<A>>) -> Self {
		self.events = Some(sink);
		self
	}

	pub fn build(self) -> StoreResult<AssetStore<A, I, C>> {
		let index = self.index.ok_or_else(|| StoreError::Codec("missing index".into()))?;
		let codec = self.codec.ok_or_else(|| StoreError::Codec("missing codec".into()))?;
		let events = self.events.unwrap_or_else(|| Arc::new(NoopSink));

		codec.validate_defaults()?;

		Ok(AssetStore { index, codec, events })
	}
}
