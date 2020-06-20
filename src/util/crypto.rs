#[path = "../error.rs"]
mod error;
use error::Error;

pub struct Crypto {
	secret: String,
}

impl Crypto {
	pub fn new(secret: &str) -> Crypto {
		Crypto {
			secret: secret.to_string(),
		}
	}

	/**
	 *
	 */
	pub async fn encrypt(&self, data: &str) -> Result<String, Error> {
		let rand = ring::rand::SystemRandom::new();

		let mut encrypted: Vec<u8> = Crypto::allocate_bytes(97);

		let iv = Crypto::fill_random_bytes(&rand, &mut encrypted[1..17])?;
		let salt = Crypto::fill_random_bytes(&rand, &mut encrypted[17..(17 + 64)])?;

		let mut key: Vec<u8> = Crypto::allocate_bytes(32);
		self.get_derived_key(&mut key, salt, 1024, ring::pbkdf2::PBKDF2_HMAC_SHA512)?;

		println!("Array {:?}", encrypted);

		Ok(String::from(data))
	}

	/**
	 * Get some random bytes
	 */
	fn get_derived_key(
		&self,
		key: &mut [u8],
		salt: &[u8],
		iterations: u32,
		algoritm: ring::pbkdf2::Algorithm,
	) -> Result<(), Error> {
		let iteration_non_zero = std::num::NonZeroU32::new(iterations);
		if iteration_non_zero.is_none() {
			return Err(Error::CryptoError);
		}
		ring::pbkdf2::derive(
			algoritm,
			iteration_non_zero.unwrap(),
			salt,
			self.secret.as_bytes(),
			key,
		);
		Ok(())
	}
	/**
	 * Get some random bytes
	 */
	fn fill_random_bytes<'a>(
		rand: &dyn ring::rand::SecureRandom,
		v: &'a mut [u8],
	) -> Result<&'a [u8], Error> {
		let result = rand.fill(v);
		match result {
			Err(_e) => return Err(Error::CryptoError),
			Ok(_v) => Ok(v),
		}
	}
	/**
	 * Allocate bytes
	 */
	fn allocate_bytes(size: usize) -> Vec<u8> {
		let mut v: Vec<u8> = Vec::with_capacity(size);
		unsafe {
			v.set_len(size);
		};
		v
	}
}
