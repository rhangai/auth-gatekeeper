use crate::error::Error;
use ring::aead::{Algorithm, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

pub type RandomPtr = std::sync::Arc<dyn ring::rand::SecureRandom + Sync + Send>;

const ITERATIONS: u32 = 4;
static ALGORITHM: &Algorithm = &AES_256_GCM;

/// Crypto functions
pub struct Crypto {
	/// Secret to encrypt the data
	secret: String,
	/// Random generator
	random: RandomPtr,
}

impl Crypto {
	///
	/// Create a new crypto using the secret and the random
	///
	pub fn new(secret: &str, random: RandomPtr) -> Crypto {
		Crypto {
			secret: secret.to_string(),
			random: random,
		}
	}
	///
	/// Create a new crypto
	///
	pub fn create_random() -> RandomPtr {
		std::sync::Arc::new(ring::rand::SystemRandom::new())
	}
	///
	/// Encrypt some data
	///
	pub fn encrypt(&self, data: &str) -> Result<String, Error> {
		let data_range_start = 77;
		let data_range_end = data_range_start + data.len();

		let mut encrypted: Vec<u8> = Crypto::allocate_bytes(93 + data.len());
		self.fill_random_bytes(&mut encrypted[1..77])?;
		encrypted[data_range_start..data_range_end].copy_from_slice(&data.as_bytes());

		// Set encrypted version
		encrypted[0] = 1;
		let nonce_bytes = &encrypted[1..13];
		let salt_bytes = &encrypted[13..77];

		let key = self.get_derived_key(
			ALGORITHM.key_len(),
			salt_bytes,
			ITERATIONS,
			ring::pbkdf2::PBKDF2_HMAC_SHA512,
		)?;
		let nonce = Self::get_nonce(&nonce_bytes)?;
		let cipher = Self::get_cipher(&key)?;

		let tag_result = cipher.seal_in_place_separate_tag(
			nonce,
			ring::aead::Aad::empty(),
			&mut encrypted[data_range_start..data_range_end],
		);

		let tag = match tag_result {
			Ok(t) => t,
			Err(_e) => return Err(Error::CryptoError),
		};
		encrypted[data_range_end..].copy_from_slice(&tag.as_ref());
		Ok(base64::encode(encrypted))
	}
	///
	/// Decrypt the data
	///
	pub fn decrypt(&self, data: &str) -> Result<String, Error> {
		let mut encrypted = match base64::decode(data) {
			Ok(v) => v,
			Err(_err) => return Err(Error::CryptoError),
		};
		if encrypted.len() < 93 {
			return Err(Error::CryptoError);
		}
		let data_range_start = 77;
		let nonce_bytes = &encrypted[1..13];
		let salt_bytes = &encrypted[13..77];

		let key = self.get_derived_key(
			ALGORITHM.key_len(),
			salt_bytes,
			ITERATIONS,
			ring::pbkdf2::PBKDF2_HMAC_SHA512,
		)?;

		let nonce = Self::get_nonce(&nonce_bytes)?;
		let cipher = Self::get_cipher(&key)?;

		let decrypted_result = cipher.open_within(
			nonce,
			ring::aead::Aad::empty(),
			&mut encrypted,
			data_range_start..,
		);

		let decrypted_bytes = match decrypted_result {
			Ok(v) => v,
			Err(_err) => return Err(Error::CryptoError),
		};
		let decrypted_text = unsafe { String::from_utf8_unchecked(decrypted_bytes.to_vec()) };
		Ok(decrypted_text)
	}

	///
	/// Get a new cipher to use
	///
	fn get_cipher(key: &[u8]) -> Result<LessSafeKey, Error> {
		let unbound_key = match UnboundKey::new(ALGORITHM, &key) {
			Ok(k) => k,
			Err(_e) => return Err(Error::CryptoCipherError),
		};
		Ok(LessSafeKey::new(unbound_key))
	}

	///
	/// Get a new nonce using the given bytes
	///
	fn get_nonce(bytes: &[u8]) -> Result<Nonce, Error> {
		match Nonce::try_assume_unique_for_key(&bytes) {
			Ok(k) => Ok(k),
			Err(_e) => Err(Error::CryptoNonceError),
		}
	}
	///
	/// Get a derived key into the
	///
	fn get_derived_key(
		&self,
		size: usize,
		salt: &[u8],
		iterations: u32,
		algoritm: ring::pbkdf2::Algorithm,
	) -> Result<Vec<u8>, Error> {
		let iteration_non_zero = std::num::NonZeroU32::new(iterations);
		if iteration_non_zero.is_none() {
			return Err(Error::CryptoDeriveKeyWrongSizeError);
		}
		let mut key: Vec<u8> = Self::allocate_bytes(size);
		ring::pbkdf2::derive(
			algoritm,
			iteration_non_zero.unwrap(),
			salt,
			self.secret.as_bytes(),
			&mut key,
		);
		Ok(key)
	}
	///
	/// Fill the buffer with random data
	///
	fn fill_random_bytes<'a>(&self, v: &'a mut [u8]) -> Result<&'a [u8], Error> {
		let result = self.random.fill(v);
		match result {
			Err(_) => return Err(Error::CryptoRandomBytesError),
			Ok(_) => Ok(v),
		}
	}
	///
	/// Allocate a few bytes into a vector to use
	///
	fn allocate_bytes(size: usize) -> Vec<u8> {
		let mut v: Vec<u8> = Vec::with_capacity(size);
		unsafe {
			v.set_len(size);
		};
		v
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_encryption() {
		let random = Crypto::create_random();
		let c = Crypto::new("Some key to test", random);

		let data = "Some random data";
		let encrypted = c.encrypt(data).unwrap();
		let decrypted = c.decrypt(&encrypted).unwrap();
		assert_eq!(data, decrypted);
	}
}
