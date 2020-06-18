import got from 'got';
import { URL } from 'url';
import { Provider } from './provider';

export type ProviderOpenIdConfig = {
	provider: 'openid-connect';
	providerClientId: string;
	providerClientSecret: string;
	providerAuthUrl: string;
	providerTokenUrl: string;
	providerValidateUrl: string;
	providerUserinfoUrl: string;
	providerRedirectUrl: string;
};

/**
 * Application class
 */
export class ProviderOpenId implements Provider {
	/// Construct the application
	constructor(private readonly config: ProviderOpenIdConfig) {}
	grant(form: Record<string, string>): unknown {
		throw new Error('Method not implemented.');
	}

	/**
	 * Perform the login. Redirecting the user to the oauth provider class.
	 * @param request
	 * @param reply
	 */
	async getAuthorizationUrl() {
		const url = new URL(this.config.providerAuthUrl);
		url.searchParams.set('response_type', 'code');
		url.searchParams.set('client_id', this.config.providerClientId);
		url.searchParams.set('state', '123456');
		url.searchParams.set('scope', 'openid email profile');
		url.searchParams.set('redirect_uri', this.config.providerRedirectUrl);
		return url.href;
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	async grantAuthorizationCode(form: Record<string, string>) {
		const { body } = await got.post({
			responseType: 'json',
			url: this.config.providerTokenUrl,
			form: {
				grant_type: 'authorization_code',
				client_id: this.config.providerClientId,
				client_secret: this.config.providerClientSecret,
				redirect_uri: this.config.providerRedirectUrl,
				...form,
			},
		});
		return body;
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	async grantRefreshToken(form: Record<string, string>) {
		const { body } = await got.post({
			responseType: 'json',
			url: this.config.providerTokenUrl,
			form: {
				grant_type: 'refresh_token',
				client_id: this.config.providerClientId,
				client_secret: this.config.providerClientSecret,
				...form,
			},
		});
		return body;
	}

	/**
	 * Get the userinfo from the access token
	 */
	async userinfo(accessToken: string | null) {
		if (!accessToken) {
			return null;
		}
		try {
			const { body } = await got.get({
				responseType: 'json',
				url: this.config.providerUserinfoUrl,
				headers: {
					authorization: `bearer ${accessToken}`,
				},
			});
			return body;
		} catch (err) {
			if (err.response.statusCode === 401) {
				return null;
			} else if (err.response.statusCode === 400) {
				return null;
			}
			throw err;
		}
	}
}
