#[derive(Debug)]
pub enum Error {
	CryptoError,
	CryptoCipherError,
	CryptoNonceError,
	CryptoDeriveKeyWrongSizeError,
	CryptoRandomBytesError,

	ConfigError,

	RequestError,
}

impl std::error::Error for Error {}

impl actix_web::error::ResponseError for Error {
	fn error_response(&self) -> actix_web::HttpResponse {
		actix_web::HttpResponse::InternalServerError().finish()
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		#[allow(unreachable_patterns)]
		let message: &'static str = match self {
			Error::CryptoError => "Oh no",
			Error::CryptoCipherError => "Error using the cipher",
			Error::CryptoNonceError => "Error creating the nonce",
			Error::CryptoDeriveKeyWrongSizeError => "Error creating the nonce",
			Error::CryptoRandomBytesError => "Error creating the nonce",
			_ => "Error",
		};
		write!(f, "{}", message)
	}
}
