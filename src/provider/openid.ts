import got, { OptionsOfJSONResponseBody } from 'got';
import { URL } from 'url';
import { Provider, ProviderTokenSet } from './provider';
import JWK from 'jwks-rsa';
// @ts-ignore
import jwt from 'jsonwebtoken';

export type ProviderOpenIdConfig = {
	provider: 'oidc';
	providerClientId: string;
	providerClientSecret: string;
	providerAuthUrl: string;
	providerTokenUrl: string;
	providerUserinfoUrl: string;
	providerCallbackUrl: string;
	providerJwksUrl?: string;
};

/**
 * OpenID provider
 */
export class ProviderOpenId implements Provider {
	private readonly jkwsClient?: JWK.JwksClient;

	/// Construct the provider
	constructor(private readonly config: ProviderOpenIdConfig) {
		if (config.providerJwksUrl) {
			this.jkwsClient = JWK({
				jwksUri: config.providerJwksUrl,
			});
		}
	}

	/**
	 * Perform the login. Redirecting the user to the oauth provider class.
	 * @param request
	 * @param reply
	 */
	async getAuthorizationUrl(state?: string) {
		const url = new URL(this.config.providerAuthUrl);
		url.searchParams.set('response_type', 'code');
		url.searchParams.set('client_id', this.config.providerClientId);
		if (state) {
			url.searchParams.set('state', state);
		}
		url.searchParams.set('scope', 'openid email profile');
		url.searchParams.set('redirect_uri', this.config.providerCallbackUrl);
		return url.href;
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	grantAuthorizationCode(form: Record<string, string>): Promise<ProviderTokenSet | null> {
		return this.grant({
			grant_type: 'authorization_code',
			client_id: this.config.providerClientId,
			client_secret: this.config.providerClientSecret,
			redirect_uri: this.config.providerCallbackUrl,
			...form,
		});
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	grantRefreshToken(form: Record<string, string>): Promise<ProviderTokenSet | null> {
		return this.grant({
			grant_type: 'refresh_token',
			client_id: this.config.providerClientId,
			client_secret: this.config.providerClientSecret,
			...form,
		});
	}

	/**
	 * Perform a generic grant
	 * @param form The form to perform the grant
	 */
	private async grant(form: Record<string, string>): Promise<ProviderTokenSet | null> {
		const body = await this.request({
			method: 'post',
			url: this.config.providerTokenUrl,
			form: {
				...form,
			},
		});
		if (!body) return null;
		const expiresAt: Date | undefined = body.expires_in ? new Date(Date.now() + body.expires_in * 1000) : undefined;
		return {
			accessToken: body.access_token,
			refreshToken: body.refresh_token,
			expiresAt,
			idToken: (await this.idTokenDecode(body.id_token)) ?? undefined,
		};
	}

	/**
	 * Get the userinfo from the access token
	 */
	async userinfo(accessToken: string | null) {
		if (!accessToken) {
			return null;
		}
		return this.request({
			method: 'GET',
			url: this.config.providerUserinfoUrl,
			headers: {
				authorization: `bearer ${accessToken}`,
			},
		});
	}

	/**
	 * Perform a json request on the openid provider
	 */
	private async request(
		options: Omit<OptionsOfJSONResponseBody, 'responseType'>
	): Promise<Record<string, any> | null> {
		try {
			const { body } = await got({
				responseType: 'json',
				...options,
			});
			return body as any;
		} catch (err) {
			if (err.response) {
				if (err.response.statusCode === 401) {
					return null;
				} else if (err.response.statusCode === 400) {
					return null;
				}
			}
			throw err;
		}
	}
	/**
	 * Perform a json request on the openid provider
	 */
	private idTokenDecode(idToken: string): Promise<Record<string, any> | null> | null {
		if (!idToken) {
			return null;
		}
		const promise = new Promise<Record<string, any> | null>((resolve, reject) => {
			if (!this.jkwsClient) {
				resolve(jwt.decode(idToken));
				return;
			}
			const getKey = (header: any, callback: any) => {
				this.jkwsClient!.getSigningKey(header.kid, (err, key) => {
					if (err) {
						callback(err);
						return;
					}
					callback(null, key.getPublicKey());
				});
			};
			jwt.verify(idToken, getKey, {}, (err: any, decoded: any) => {
				err ? reject(err) : resolve(decoded);
			});
		});
		return promise.catch(() => null);
	}
}
