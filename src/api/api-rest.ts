import { Api } from './api';
import got from 'got';

export type ApiRestConfig = {
	api: 'rest';
	apiAuthorization?: string;
	apiIdTokenEndpoint?: string;
};

type ApiRestBoundCallback = (idToken: Record<string, any>) => Promise<void>;

/**
 * Api rest
 */
export class ApiRest implements Api {
	onIdToken?: ApiRestBoundCallback;

	/// Construct the api rest endpoint
	constructor(private readonly config: ApiRestConfig) {
		this.onIdToken = this.bindRequest(config.apiIdTokenEndpoint);
	}

	/**
	 * Get the request headers
	 */
	getHeaders() {
		if (!this.config.apiAuthorization) return {};
		return {
			authorization: `bearer ${this.config.apiAuthorization}`,
		};
	}

	/**
	 *
	 */
	private bindRequest(url: string | undefined): ApiRestBoundCallback | undefined {
		if (!url) return undefined;
		const headers = this.getHeaders();
		return async (data: Record<string, any>) => {
			await got({
				method: 'post',
				url: url,
				headers,
				json: data,
			});
		};
	}
}
