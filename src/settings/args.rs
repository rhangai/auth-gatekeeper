use crate::error::Error;
use config::{Source, Value};
use getopts::Options;
use std::collections::HashMap;

const OPTS: &[(&str, &str, &str, &str)] = &[
	(
		"config-env",
		"config.env",
		"Use env variables prefixed with ENV_PREFIX to configure",
		"ENV_PREFIX",
	),
	(
		"jwt-secret",
		"jwt_secret",
		"Use TOKEN to encode the JWT used by x-auth headers",
		"TOKEN",
	),
];

#[derive(Clone, Debug)]
pub struct ArgsConfig {
	config: HashMap<String, Value>,
}

impl ArgsConfig {
	pub fn new() -> Result<Self, Error> {
		Ok(Self {
			config: Self::collect_config()?,
		})
	}

	pub fn show_help() {
		let options = Self::get_options();
		let brief = format!("Usage: auth-gatekeeper [options]");
		print!("{}", options.usage(&brief));
	}

	fn get_options() -> Options {
		let mut opts = Options::new();
		opts.optflag("h", "help", "Show help");
		for (longname, _, desc, arg_hint) in OPTS {
			opts.optopt("", longname, desc, arg_hint);
		}
		opts
	}

	fn collect_config() -> Result<HashMap<String, Value>, Error> {
		let uri = String::from("args");
		let opts = Self::get_options();
		let args: Vec<String> = std::env::args().collect();
		let matches = opts.parse(&args[1..]).unwrap();

		if matches.opt_present("h") {
			return Err(Error::SettingsShowHelpError);
		}

		let mut m: HashMap<String, Value> = HashMap::new();
		for (longname, configname, _, _) in OPTS {
			if let Some(value) = matches.opt_str(longname) {
				m.insert(configname.to_string(), Value::new(Some(&uri), value));
			}
		}
		Ok(m)
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
