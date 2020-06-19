// @ts-ignore
import cookie from 'cookie';
import { Crypto } from '../util/crypto';
import { ProviderTokenSet } from '../provider/provider';
import { FastifyCookieOptions } from 'fastify-cookie';
import { Request, Reply } from '../http';

export type CookieConfig = {
	cookieSecret: string;
	cookieAccessTokenName: string;
	cookieRefreshTokenName: string;
};

/**
 * Application class
 */
export class CookieManager {
	private crypto: Crypto;

	/// Construct the application
	constructor(private readonly config: CookieConfig) {
		this.crypto = new Crypto(this.config.cookieSecret);
	}

	/**
	 * Serialize a cookie into a set-cookie string
	 */
	async serialize(
		cookieName: string,
		value: string | null | undefined,
		cookieOptions?: FastifyCookieOptions
	): Promise<string> {
		if (value == null) {
			return cookie.serialize(cookieName, '', { expires: new Date(1), path: '/' });
		}
		const cookieValue = await this.crypto.encrypt(value);
		return cookie.serialize(cookieName, cookieValue, cookieOptions);
	}
	/**
	 * Serialize the clear cookies
	 */
	async serializeClear() {
		return Promise.all([
			this.serialize(this.config.cookieAccessTokenName, null),
			this.serialize(this.config.cookieRefreshTokenName, null),
		]);
	}

	/**
	 * Serialize the cookies from the token set
	 */
	serializeFromTokenSet(tokenSet: ProviderTokenSet | null | undefined) {
		if (!tokenSet) {
			return this.serializeClear();
		}
		const cookieAccess = this.serialize(this.config.cookieAccessTokenName, tokenSet.accessToken, {
			expires: tokenSet.expiresAt,
		});
		const cookieRefresh = this.serialize(this.config.cookieRefreshTokenName, tokenSet.refreshToken);
		return Promise.all([cookieAccess, cookieRefresh]);
	}

	/**
	 * Get the access token value from the cookie
	 */
	async getAccessToken(request: Request): Promise<string | null> {
		return this.get(request, this.config.cookieAccessTokenName);
	}

	/**
	 * Get the refresh token cookie
	 */
	async getRefreshToken(request: Request): Promise<string | null> {
		return this.get(request, this.config.cookieRefreshTokenName);
	}

	/**
	 * Get a cookie from the request and pass back
	 */
	private async get(request: Request, cookieName: string): Promise<string | null> {
		const cookieValue = request.cookies[cookieName];
		if (!cookieValue) {
			return null;
		}
		const value = await this.crypto.decrypt(cookieValue).catch(() => null);
		if (!value) {
			return null;
		}
		return value;
	}

	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	async clear(reply: Reply) {
		const setCookies = await this.serializeClear();
		reply.removeHeader('set-cookie');
		reply.header('set-cookie', setCookies);
	}

	/**
	 * Set the cookies from the tokenSet
	 */
	async setFromTokenSet(reply: Reply, tokenSet: ProviderTokenSet) {
		const setCookies = await this.serializeFromTokenSet(tokenSet);
		reply.removeHeader('set-cookie');
		reply.header('set-cookie', setCookies);
	}
}
