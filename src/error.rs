#[derive(Debug)]
pub enum Error {
	CryptoError,
	CryptoCipherError,
	CryptoNonceError,
	CryptoDeriveKeyWrongSizeError,
	CryptoRandomBytesError,

	SettingsError(&'static str),
	SettingsConfigError(config::ConfigError),
	SettingsUrlParseError(actix_web::http::uri::InvalidUri),
	SettingsShowHelpError,

	JwtError(jsonwebtoken::errors::Error),

	JsonError(serde_json::Error),

	RequestError(actix_web::client::SendRequestError),
	RequestJsonError(actix_web::client::JsonPayloadError),

	ApiError,
}

impl std::error::Error for Error {}

impl actix_web::error::ResponseError for Error {
	fn error_response(&self) -> actix_web::HttpResponse {
		log::error!("Uncaught error: {}", self);
		actix_web::HttpResponse::InternalServerError().finish()
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		#[allow(unreachable_patterns)]
		let message: String = match self {
			Error::CryptoError => String::from("Crypto error"),
			Error::CryptoCipherError => String::from("Error using the cipher"),
			Error::CryptoNonceError => String::from("Error creating the nonce"),
			Error::CryptoDeriveKeyWrongSizeError => String::from("Error creating the nonce"),
			Error::CryptoRandomBytesError => String::from("Error creating the nonce"),

			Error::SettingsConfigError(ref error) => format!("Config Error: {}", error.to_string()),
			_ => String::from("Error"),
		};
		write!(f, "{}", message)
	}
}

impl From<config::ConfigError> for Error {
	fn from(error: config::ConfigError) -> Error {
		Error::SettingsConfigError(error)
	}
}

/// JWT Encode decode error
impl From<jsonwebtoken::errors::Error> for Error {
	fn from(error: jsonwebtoken::errors::Error) -> Error {
		Error::JwtError(error)
	}
}

/// JSON serialize/dererialize error
impl From<serde_json::Error> for Error {
	fn from(error: serde_json::Error) -> Error {
		Error::JsonError(error)
	}
}

/// Request error
impl From<actix_web::client::SendRequestError> for Error {
	fn from(error: actix_web::client::SendRequestError) -> Error {
		Error::RequestError(error)
	}
}
impl From<actix_web::client::JsonPayloadError> for Error {
	fn from(error: actix_web::client::JsonPayloadError) -> Error {
		Error::RequestJsonError(error)
	}
}

/// Converts from an URL parser error
impl From<actix_web::http::uri::InvalidUri> for Error {
	fn from(error: actix_web::http::uri::InvalidUri) -> Error {
		Error::SettingsUrlParseError(error)
	}
}
