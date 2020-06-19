import { ProviderOpenIdConfig, ProviderOpenId } from './openid';

export type ProviderConfig = ProviderOpenIdConfig | Record<string, unknown>;

export type ProviderTokenSet = {
	accessToken: string;
	refreshToken?: string;
	expiresAt?: Date;
	idToken?: Record<string, any>;
};

export interface Provider {
	/// Get the authorization url
	getAuthorizationUrl(state?: string): string | Promise<string>;
	/**
	 * Grant an authorization code
	 * @param form
	 */
	grantAuthorizationCode(form: Record<string, string>): Promise<ProviderTokenSet | null>;
	/**
	 * Grant the refresh token.
	 * @param form
	 */
	grantRefreshToken(form: Record<string, string>): Promise<ProviderTokenSet | null>;
	/**
	 * Get the userinfo according to the access token
	 * @param accessToken
	 */
	userinfo(accessToken: string | null): unknown | Promise<unknown>;
}

export function providerCreate(config: ProviderConfig): Provider {
	if (config.provider === 'oidc') {
		return new ProviderOpenId(config as ProviderOpenIdConfig);
	}
	throw new Error(`Invalid provider`);
}
