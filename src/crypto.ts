import crypto from 'crypto';

type AuthCryptoOptions = {
	cipherAlgorithm: string;
	ivLength: number;
	saltLength: number;
	tagLength: number;
	keyLength: number;
	keyIterations: number;
	keyAlgorithm: string;
};

export class Crypto {
	private static readonly CRYPTO_DATA_ENCODING = 'utf8';
	private static readonly CRYPTO_ENCODING = 'base64';
	private static readonly CRYPTO_OPTIONS: Record<number, AuthCryptoOptions> = {
		1: {
			cipherAlgorithm: 'aes-256-gcm',
			ivLength: 16,
			saltLength: 64,
			tagLength: 16,
			keyLength: 32,
			keyIterations: 1024,
			keyAlgorithm: 'sha512',
		},
	};

	/// Version of the current options being used when encrypting
	private readonly currentVersion: number;

	// Construct the crypto module
	constructor(private readonly key: string) {
		const versions = Object.keys(Crypto.CRYPTO_OPTIONS).map((v) => parseInt(v, 10));
		this.currentVersion = Math.max(...versions);
	}

	/**
	 * Encrypt some data using the version from this crypto library
	 * @param data
	 * @param version
	 */
	async encrypt(data: string, version?: number) {
		const optionsVersion = version ?? this.currentVersion;
		const options = Crypto.CRYPTO_OPTIONS[optionsVersion];
		if (!options) throw new Error(`Versão invalida da criptografia`);

		const iv = await Crypto.getRandomBytes(options.ivLength);
		const salt = await Crypto.getRandomBytes(options.saltLength);
		const key = await this.getDerivedKey(salt, options.keyIterations, options.keyLength, options.keyAlgorithm);

		const cipher = crypto.createCipheriv(options.cipherAlgorithm, key, iv);
		const encrypted = Buffer.concat([cipher.update(data, Crypto.CRYPTO_DATA_ENCODING), cipher.final()]);
		let tag: Buffer | null = null;
		if (options.tagLength) {
			tag = (cipher as any).getAuthTag();
		}

		const bufferVersion = Buffer.from([optionsVersion]);
		const output: Buffer[] = [bufferVersion, iv, salt, tag, encrypted].filter(Boolean) as Buffer[];
		return Buffer.concat(output).toString(Crypto.CRYPTO_ENCODING);
	}

	/**
	 * Decrypt some data.
	 * @param token
	 */
	async decrypt(encrypted: string): Promise<string> {
		const encryptedBuffer = Buffer.from(encrypted, Crypto.CRYPTO_ENCODING);

		const optionsVersion = encryptedBuffer[0];
		const options = Crypto.CRYPTO_OPTIONS[optionsVersion];
		if (!options) throw new Error(`Versão invalida da criptografia`);

		const ivOffset = 1;
		const iv = encryptedBuffer.slice(ivOffset, ivOffset + options.ivLength);

		const saltOffset = ivOffset + options.ivLength;
		const salt = encryptedBuffer.slice(saltOffset, saltOffset + options.saltLength);
		const key = await this.getDerivedKey(salt, options.keyIterations, options.keyLength, options.keyAlgorithm);

		const tagOffset = saltOffset + options.saltLength;
		const tag = encryptedBuffer.slice(tagOffset, tagOffset + options.tagLength);

		const encryptedDataOffset = tagOffset + options.tagLength;
		const encryptedData = encryptedBuffer.slice(encryptedDataOffset);

		const decipher = crypto.createDecipheriv(options.cipherAlgorithm, key, iv);
		if (options.tagLength) {
			(decipher as any).setAuthTag(tag);
		}

		let output = decipher.update(encryptedData, undefined, Crypto.CRYPTO_DATA_ENCODING);
		output += decipher.final(Crypto.CRYPTO_DATA_ENCODING);
		return output;
	}

	/**
	 * Get the derived key to use as encryption key on the data.
	 */
	private getDerivedKey(salt: Buffer, iterations: number, length: number, algorithm: string): Promise<Buffer> {
		return new Promise<Buffer>((resolve, reject) => {
			crypto.pbkdf2(this.key, salt, iterations, length, algorithm, (err, derivedKey) => {
				err ? reject(err) : resolve(derivedKey);
			});
		});
	}

	/**
	 * Pega a chave crypto
	 * @param salt O sal da chave
	 */
	private static getRandomBytes(size: number): Promise<Buffer> {
		return new Promise<Buffer>((resolve, reject) => {
			crypto.randomBytes(size, (err, bytes) => {
				err ? reject(err) : resolve(bytes);
			});
		});
	}
}
