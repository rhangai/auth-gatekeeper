#[derive(Debug)]
pub enum Error {
	CryptoError,
	CryptoCipherError,
	CryptoNonceError,
	CryptoDeriveKeyWrongSizeError,
	CryptoRandomBytesError,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Oh no, something bad went down")
	}
}
