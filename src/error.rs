#[derive(Debug)]
pub enum Error {
	CryptoError,
	CryptoCipherError,
	CryptoNonceError,
	CryptoDeriveKeyWrongSizeError,
	CryptoRandomBytesError,

	SettingsError(String),

	JwtDecodeError,

	RequestError,
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

			Error::SettingsError(ref message) => format!("SettingsError: {}", &message),
			_ => String::from("Error"),
		};
		write!(f, "{}", message)
	}
}
