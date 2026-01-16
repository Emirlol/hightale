use std::any::Any;

use anyhow::{
	anyhow,
	Result,
};

pub(crate) type BoxedArg = Box<dyn Any + Send + Sync>;

pub trait ArgParser: Send + Sync {
	/// Validates the input and potentially transforms it.
	fn parse(&self, input: &str) -> Result<BoxedArg>;
	/// Returns suggestions for tab completion.
	fn suggestions(&self) -> Vec<String>;
}

/// Helper trait for the macro
pub trait Parsable {
	type Parser: ArgParser + Default + 'static;
}

// --- Standard Implementations ---

#[derive(Default)]
pub struct StringParser; // 'word'
impl ArgParser for StringParser {
	fn parse(&self, input: &str) -> Result<BoxedArg> {
		Ok(Box::new(input.to_string()))
	}
	fn suggestions(&self) -> Vec<String> {
		vec![]
	}
}
impl Parsable for String {
	type Parser = StringParser;
}

#[derive(Default)]
pub struct IntegerParser;
impl ArgParser for IntegerParser {
	fn parse(&self, input: &str) -> Result<BoxedArg> {
		let val = input.parse::<i32>().map_err(|e| anyhow!(e))?;
		Ok(Box::new(val))
	}
	fn suggestions(&self) -> Vec<String> {
		(0..10).map(|i| i.to_string()).collect()
	}
}
impl Parsable for i32 {
	type Parser = IntegerParser;
}

#[derive(Default)]
pub struct LongParser;
impl ArgParser for LongParser {
	fn parse(&self, input: &str) -> Result<BoxedArg> {
		let val = input.parse::<i64>().map_err(|e| anyhow!(e))?;
		Ok(Box::new(val))
	}
	fn suggestions(&self) -> Vec<String> {
		(0..10).map(|i| i.to_string()).collect()
	}
}
impl Parsable for i64 {
	type Parser = LongParser;
}

#[derive(Default)]
pub struct FloatParser;
impl ArgParser for FloatParser {
	fn parse(&self, input: &str) -> Result<BoxedArg> {
		let val = input.parse::<f32>().map_err(|e| anyhow!(e))?;
		Ok(Box::new(val))
	}
	fn suggestions(&self) -> Vec<String> {
		(0..10).map(|i| i.to_string()).collect()
	}
}
impl Parsable for f32 {
	type Parser = FloatParser;
}

#[derive(Default)]
pub struct DoubleParser;
impl ArgParser for DoubleParser {
	fn parse(&self, input: &str) -> Result<BoxedArg> {
		let val = input.parse::<f64>().map_err(|e| anyhow!(e))?;
		Ok(Box::new(val))
	}
	fn suggestions(&self) -> Vec<String> {
		(0..10).map(|i| i.to_string()).collect()
	}
}
impl Parsable for f64 {
	type Parser = DoubleParser;
}

#[derive(Default)]
pub struct BoolParser;
impl ArgParser for BoolParser {
	fn parse(&self, input: &str) -> Result<BoxedArg> {
		match input {
			"true" => Ok(Box::new(true)),
			"false" => Ok(Box::new(false)),
			_ => Err(anyhow!("Expected true/false")),
		}
	}
	fn suggestions(&self) -> Vec<String> {
		vec!["true".to_string(), "false".to_string()]
	}
}
impl Parsable for bool {
	type Parser = BoolParser;
}
