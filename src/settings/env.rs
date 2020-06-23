use config::{Source, Value};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EnvironmentConfig {
	prefix: String,
	submatchers: &'static [&'static str],
}

impl EnvironmentConfig {
	pub fn with_prefix(prefix: &str, submatchers: &'static [&'static str]) -> Self {
		Self {
			prefix: prefix.to_string(),
			submatchers: submatchers,
		}
	}

	fn transform_key(&self, key: &str) -> Option<String> {
		let key = key.to_string().to_lowercase();

		let preffix_pattern = format!("{}_", self.prefix).to_lowercase();
		if !key.starts_with(&preffix_pattern) {
			return None;
		}
		let rest = key[preffix_pattern.len()..].to_string();
		for submatcher in self.submatchers {
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
		let mut m = HashMap::new();
		let uri: String = "the environment".into();

		for (key, value) in std::env::vars() {
			if value == "" {
				continue;
			}
			if let Some(new_key) = self.transform_key(&key) {
				m.insert(new_key, Value::new(Some(&uri), value));
			}
		}

		Ok(m)
	}
}
