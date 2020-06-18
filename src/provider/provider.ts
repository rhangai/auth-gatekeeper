import { ProviderOpenIdConfig } from './openid';

export type ProviderConfig = ProviderOpenIdConfig;

export type ProviderTokenSet = {
	accessToken: string;
	refreshToken?: string;
	expiresAt?: Date;
	idToken?: Record<string, any>;
};

export interface Provider {
	/// Get the authorization url
	getAuthorizationUrl(): string | Promise<string>;
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
