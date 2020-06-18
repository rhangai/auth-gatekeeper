import { ProviderOpenIdConfig } from './openid';

export type ProviderConfig = ProviderOpenIdConfig;

export interface Provider {
	getAuthorizationUrl(): string | Promise<string>;

	grantAuthorizationCode(form: Record<string, string>): unknown | Promise<unknown>;
	grantRefreshToken(form: Record<string, string>): unknown | Promise<unknown>;

	userinfo(accessToken: string | null): unknown | Promise<unknown>;
}
