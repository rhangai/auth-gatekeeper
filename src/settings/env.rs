use config::{Source, Value};
use std::collections::HashMap;

pub struct EnvironmentConfigOptions<'a> {
	prefix: &'a str,
	submatchers: &'a [&'static str],
}

#[derive(Clone, Debug)]
pub struct EnvironmentConfig {
	config: HashMap<String, Value>,
}

impl EnvironmentConfig {
	pub fn with_prefix(prefix: &str, submatchers: &[&'static str]) -> Self {
		let options = EnvironmentConfigOptions {
			prefix: prefix,
			submatchers: submatchers,
		};
		Self {
			config: Self::collect_config(&options),
		}
	}

	fn collect_config(options: &EnvironmentConfigOptions) -> HashMap<String, Value> {
		let mut m = HashMap::new();
		let uri: String = "the environment".into();
		for (key, value) in std::env::vars() {
			if value == "" {
				continue;
			}
			if let Some(new_key) = Self::transform_key(options, &key) {
				m.insert(new_key, Value::new(Some(&uri), value));
			}
		}
		m
	}

	fn transform_key(options: &EnvironmentConfigOptions, key: &str) -> Option<String> {
		let key = key.to_string().to_lowercase();

		let preffix_pattern = format!("{}_", options.prefix).to_lowercase();
		if !key.starts_with(&preffix_pattern) {
			return None;
		}
		let rest = key[preffix_pattern.len()..].to_string();
		for submatcher in options.submatchers {
			if rest == *submatcher {
				return Some(format!("{0}.{0}", submatcher));
			}
			let submatcher_prefix = format!("{}_", submatcher);
			if rest.starts_with(&submatcher_prefix) {
				let data = format!("{}.{}", submatcher, &rest[submatcher_prefix.len()..]);
				return Some(data);
			}
		}
		Some(rest)
	}
}

impl Source for EnvironmentConfig {
	fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
		Box::new((*self).clone())
	}

	fn collect(&self) -> Result<HashMap<String, Value>, config::ConfigError> {
		Ok(self.config.clone())
	}
}
