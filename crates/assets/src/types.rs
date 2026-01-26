use std::{
	collections::{
		HashMap,
		HashSet,
	},
	path::{
		Path,
		PathBuf,
	},
	sync::Arc,
};

use crate::{
	validate::{
		Severity,
		ValidationIssue,
	},
	Asset,
	AssetUpdateQuery,
	StoreError,
	StoreResult,
	ValidationContext,
	ValidationResults,
};

pub struct LoadOptions<A: Asset> {
	pub filter: Box<dyn Fn(&Path) -> bool + Send + Sync>,
	pub query: AssetUpdateQuery<A::Key>,
	pub validation: Option<ValidationConfig<A>>,
	pub mode: LoadMode,
}

impl<A: Asset> LoadOptions<A> {
	pub fn strict() -> Self {
		Self {
			mode: LoadMode::Strict,
			filter: Box::new(|p: &Path| is_json_file(p)),
			query: AssetUpdateQuery::default(),
			validation: None,
		}
	}

	pub fn report() -> Self {
		Self {
			mode: LoadMode::Report,
			filter: Box::new(|p: &Path| is_json_file(p)),
			query: AssetUpdateQuery::default(),
			validation: None,
		}
	}

	pub fn filter(mut self, f: impl Fn(&Path) -> bool + Send + Sync + 'static) -> Self {
		self.filter = Box::new(f);
		self
	}

	pub fn query(mut self, q: AssetUpdateQuery<A::Key>) -> Self {
		self.query = q;
		self
	}

	pub fn validate_with(mut self, policy: ValidationPolicy, v: impl Fn(&A, &ValidationContext<A::Key>, &mut ValidationResults<A::Key>) + Send + Sync + 'static) -> Self {
		self.validation = Some(ValidationConfig { validator: Box::new(v), policy });
		self
	}
}

fn is_json_file(path: &Path) -> bool {
	path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("json")).unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub enum LoadMode {
	Strict,
	Report,
}

pub struct ValidationConfig<A: Asset> {
	#[allow(clippy::type_complexity)] // We can't really fix this since bounds on generic parameters aren't enforced on type aliases
	pub validator: Box<dyn Fn(&A, &ValidationContext<A::Key>, &mut ValidationResults<A::Key>) + Send + Sync>,
	pub policy: ValidationPolicy,
}

pub enum ValidationPolicy {
	CollectOnly,
	SkipInvalidAsset, // Don’t upsert if errors
	FailWholeLoad,    // Stop immediately
}

#[derive(Debug)]
pub enum LoadOutcome<K> {
	Strict(LoadSummary<K>),
	Report(LoadReport<K>),
}

impl<K> LoadOutcome<K> {
	pub fn summary(&self) -> &LoadSummary<K> {
		match self {
			LoadOutcome::Strict(s) => s,
			LoadOutcome::Report(r) => &r.summary,
		}
	}
}

#[derive(Debug, Clone)]
pub struct LoadSummary<K> {
	pub root: PathBuf,
	pub loaded: HashMap<K, InputRef>,
	pub files_seen: usize,
	pub decoded: usize,
	pub added: usize,
	pub updated: usize,
	pub removed: usize,
}

impl<K> LoadSummary<K> {
	pub fn new(root: PathBuf) -> Self {
		Self {
			root,
			loaded: HashMap::new(),
			files_seen: 0,
			decoded: 0,
			added: 0,
			updated: 0,
			removed: 0,
		}
	}
}

#[derive(Debug)]
pub struct LoadIssue {
	pub input: InputRef,
	pub error: StoreError,
}

#[derive(Debug)]
pub struct LoadReport<K> {
	pub summary: LoadSummary<K>,
	pub io_decode_issues: Vec<LoadIssue>,
	pub validation_issues: Vec<ValidationIssue<K>>,
}

impl<K> LoadReport<K> {
	pub fn is_clean(&self) -> bool {
		self.io_decode_issues.is_empty() && self.validation_issues.is_empty()
	}

	pub fn error_count(&self) -> usize {
		self.io_decode_issues.len() + self.validation_issues.iter().filter(|i| i.severity == Severity::Error).count()
	}
}

pub(crate) type Prepared<T> = Result<WithInput<T>, WithInput<StoreError>>;

pub(crate) fn prepared_ok<T>(input_ref: InputRef, value: T) -> Prepared<T> {
	Ok(WithInput::new(input_ref, value))
}

pub(crate) fn prepared_err<T>(input_ref: InputRef, error: StoreError) -> Prepared<T> {
	Err(WithInput::new(input_ref, error))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum InputRef {
	Path(Arc<PathBuf>),
	Label(String),
	#[default]
	Unknown,
}

impl InputRef {
	pub fn path(p: PathBuf) -> Self {
		Self::Path(Arc::new(p))
	}

	pub fn label(s: impl Into<String>) -> Self {
		Self::Label(s.into())
	}
}

#[derive(Debug, Clone)]
pub struct WithInput<T> {
	pub input: InputRef,
	pub value: T,
}

impl<T> WithInput<T> {
	pub fn new(input: InputRef, value: T) -> Self {
		Self { input, value }
	}
}

pub(crate) struct LoadState<K> {
	pub summary: LoadSummary<K>,
	pub seen: HashSet<K>,
	io_decode_issues: Vec<LoadIssue>,
	pub(crate) validation_issues: Vec<ValidationIssue<K>>,
	pub mode: LoadMode,
}

impl<K> LoadState<K> {
	pub fn new(root: PathBuf, mode: LoadMode) -> Self {
		Self {
			summary: LoadSummary::new(root),
			seen: HashSet::new(),
			io_decode_issues: Vec::new(),
			validation_issues: Vec::new(),
			mode,
		}
	}

	pub fn issue_or_err(&mut self, with_input: WithInput<StoreError>) -> StoreResult<()> {
		match self.mode {
			LoadMode::Strict => Err(with_input.value),
			LoadMode::Report => {
				self.io_decode_issues.push(LoadIssue {
					input: with_input.input,
					error: with_input.value,
				});
				Ok(())
			}
		}
	}

	pub fn finish(self) -> LoadOutcome<K> {
		match self.mode {
			LoadMode::Strict => LoadOutcome::Strict(self.summary),
			LoadMode::Report => LoadOutcome::Report(LoadReport {
				summary: self.summary,
				io_decode_issues: self.io_decode_issues,
				validation_issues: self.validation_issues,
			}),
		}
	}
}

pub(crate) enum Analyzed<A: Asset> {
	Accept {
		input: InputRef,
		key: A::Key,
		asset: A,
		issues: Vec<ValidationIssue<A::Key>>,
	},
	Skip {
		issues: Vec<ValidationIssue<A::Key>>,
	}, // query disallowed or SkipInvalidAsset
	Fail {
		input: InputRef,
		error: StoreError,
		issues: Vec<ValidationIssue<A::Key>>,
	}, // FailWholeLoad hit
}
