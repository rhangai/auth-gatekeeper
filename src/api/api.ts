import { ApiRestConfig, ApiRest } from './api-rest';

export type ApiConfig =
	| ApiRestConfig
	| {
			[otherApiConfig: string]: unknown;
	  };

export interface Api {
	onIdToken?(idToken: Record<string, any>): Promise<void>;
}

export function apiCreate(config: ApiConfig) {
	if (config.api === 'rest') {
		return new ApiRest(config as ApiRestConfig);
	} else if (!config.api) {
		return null;
	}
	throw new Error(`Invalid api "${config.api}".`);
}
