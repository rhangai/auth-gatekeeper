use config::{Source, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ArgsConfig {}

impl ArgsConfig {
	pub fn new() -> Self {
		Self {}
	}
}

impl Source for ArgsConfig {
	fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
		Box::new((*self).clone())
	}

	fn collect(&self) -> Result<HashMap<String, Value>, config::ConfigError> {
		let mut m = HashMap::new();

		Ok(m)
	}
}
