use config::{Source, Value};
use getopts::Options;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ArgsConfig {
	config: HashMap<String, Value>,
}

impl ArgsConfig {
	pub fn new() -> Self {
		Self {
			config: Self::collect_config(),
		}
	}

	fn collect_config() -> HashMap<String, Value> {
		let uri = String::from("args");

		let config = [
			("config-env", "env", "Token for jwt", "ENV_PREFIX"),
			("jwt-secret", "jwt_secret", "Secret for jwt", "TOKEN"),
		];
		let mut opts = Options::new();
		for (longname, _, desc, arg_hint) in &config {
			opts.optopt("", longname, desc, arg_hint);
		}
		let args: Vec<String> = std::env::args().collect();
		let matches = opts.parse(&args[1..]).unwrap();

		let mut m: HashMap<String, Value> = HashMap::new();
		for (longname, configname, _, _) in &config {
			if let Some(value) = matches.opt_str(longname) {
				m.insert(configname.to_string(), Value::new(Some(&uri), value));
			}
		}
		m
	}
}

impl Source for ArgsConfig {
	fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
		Box::new((*self).clone())
	}

	fn collect(&self) -> Result<HashMap<String, Value>, config::ConfigError> {
		Ok(self.config.clone())
	}
}
