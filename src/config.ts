import Yargs from 'yargs/yargs';
import toml from 'toml';
import fs from 'fs';
import path from 'path';
import { CookieConfig } from './util/cookie';
import { ProviderConfig } from './provider/provider';
import { Crypto } from './util/crypto';

export type Config = {
	host: string;
	port: number;
	cookie: CookieConfig;
	provider: ProviderConfig;
};

function configSplitCase(config: string): string[] {
	return config
		.split(/(?=[A-Z])/)
		.filter(Boolean)
		.map((c) => c.toLowerCase());
}

export function configFlattenObject(config: Record<string, any>, parentPath: string[] = []) {
	const flattened: Record<string, any> = {};
	for (const key in config) {
		const value = config[key];
		let currentPath: string[];
		if (parentPath.length > 0 && key === parentPath[parentPath.length - 1]) {
			currentPath = parentPath;
		} else {
			currentPath = [...parentPath, ...configSplitCase(key)];
		}
		if (value !== null && typeof value === 'object') {
			Object.assign(flattened, configFlattenObject(value, currentPath));
		} else {
			const newKey = currentPath.join('-');
			flattened[newKey] = value;
		}
	}
	return flattened;
}

export async function configParse(argv: string[]): Promise<Config> {
	// prettier-ignore
	const yargs = Yargs(argv)
		.env('AUTH_PROXY')
		.config('config', function (filepath: string) {
			const ext = path.extname(filepath).toLowerCase();
			let config: Record<string, any> | null = null;
			if (ext === '.json') {
				config = JSON.parse(fs.readFileSync(filepath, 'utf8'));
			} else if (ext === '.toml') {
				config = toml.parse(fs.readFileSync(filepath, 'utf8'));
			}
			if (!config) throw new Error(`Invalid config file. Expecting JSON or TOML.`);
			return configFlattenObject(config);
		})
		.strict()
		.options({
			config: {
				group: 'main',
				describe: 'Path to the config file',
			},
			host: {
				group: 'main',
				describe: 'Host',
				default: '',
			},
			port: {
				group: 'main',
				describe: 'Port',
				default: 8080,
				type: 'number',
			},
			'cookie-secret': {
				group: 'cookie',
				describe: 'Secret to encrypt the cookies',
				default: null,
			},
			'cookie-access-token-name': {
				group: 'cookie',
				describe: 'Name of the access token cookie',
				default: 'sat',
			},
			'cookie-refresh-token-name': {
				group: 'cookie',
				describe: 'Name of the refresh token cookie',
				default: 'srt',
			},
			provider: {
				group: 'provider',
				describe: 'Provider',
				default: 'oidc',
			},
			'provider-client-id': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
				demandOption: true,
			},
			'provider-client-secret': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
				demandOption: true,
			},
			'provider-auth-url': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
				demandOption: true,
			},
			'provider-token-url': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
				demandOption: true,
			},
			'provider-validate-url': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
				demandOption: true,
			},
			'provider-userinfo-url': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
				demandOption: true,
			},
			'provider-redirect-url': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
				demandOption: true,
			},
			'provider-jwks-url': {
				group: 'provider',
				describe: 'Provider',
				type: 'string',
			},
		});

	const args = yargs.argv;
	return {
		host: args.host,
		port: args.port,
		cookie: {
			cookieSecret: args['cookie-secret'] || (await Crypto.getRandomBytes(32)).toString('base64'),
			cookieAccessTokenName: args['cookie-access-token-name'],
			cookieRefreshTokenName: args['cookie-refresh-token-name'],
		},
		provider: {
			provider: args['provider'] as any,
			providerClientId: args['provider-client-id'],
			providerClientSecret: args['provider-client-secret'],
			providerAuthUrl: args['provider-auth-url'],
			providerTokenUrl: args['provider-token-url'],
			providerValidateUrl: args['provider-validate-url'],
			providerUserinfoUrl: args['provider-userinfo-url'],
			providerRedirectUrl: args['provider-redirect-url'],
			providerJwksUrl: args['provider-jwks-url'],
		},
	};
}
