use crate::{
	InputRef,
	WithInput,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
	Warning,
	Error,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue<K> {
	pub severity: Severity,
	pub code: &'static str,
	pub message: String,
	pub input: InputRef,
	pub key: Option<K>,
}

#[derive(Debug, Clone)]
pub struct ValidationResults<K> {
	issues: Vec<ValidationIssue<K>>,
}

impl<K> Default for ValidationResults<K> {
	fn default() -> Self {
		Self { issues: Vec::new() }
	}
}

impl<K: Clone> ValidationResults<K> {
	pub fn push(&mut self, issue: ValidationIssue<K>) {
		self.issues.push(issue);
	}

	pub fn warning(&mut self, code: &'static str, message: impl Into<String>, input: InputRef, key: Option<K>) {
		self.push(ValidationIssue {
			severity: Severity::Warning,
			code,
			message: message.into(),
			input,
			key,
		});
	}

	pub fn error(&mut self, code: &'static str, message: impl Into<String>, input: InputRef, key: Option<K>) {
		self.push(ValidationIssue {
			severity: Severity::Error,
			code,
			message: message.into(),
			input,
			key,
		});
	}

	pub fn issues(&self) -> &[ValidationIssue<K>] {
		&self.issues
	}

	pub fn into_issues(self) -> Vec<ValidationIssue<K>> {
		self.issues
	}

	pub fn error_count(&self) -> usize {
		self.issues.iter().filter(|i| i.severity == Severity::Error).count()
	}

	pub fn warning_count(&self) -> usize {
		self.issues.iter().filter(|i| i.severity == Severity::Warning).count()
	}

	pub fn is_clean(&self) -> bool {
		self.issues.is_empty()
	}

	pub fn has_errors(&self) -> bool {
		self.error_count() > 0
	}
}

/// Contextual validation information associated with an optional key.
pub type ValidationContext<K> = WithInput<Option<K>>;
