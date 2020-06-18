import Fastify, { FastifyInstance } from 'fastify';
// @ts-ignore
import Yargs from 'yargs/yargs';
// @ts-ignore
import cookie from 'cookie';
import fs from 'fs';
import path from 'path';
import toml from 'toml';
import { Crypto } from './util/crypto';
import { Provider, ProviderConfig, ProviderTokenSet } from './provider/provider';
import { ProviderOpenId } from './provider/openid';
import { FastifyCookieOptions } from 'fastify-cookie';

type Request = Fastify.FastifyRequest;
type Reply = Fastify.FastifyReply<import('http').ServerResponse>;

export type Config = {
	host: string;
	port: number;
	cookieSecret: string;
	cookieAccessTokenName: string;
	cookieRefreshTokenName: string;
} & ProviderConfig;

const CONFIG_DEFAULTS: Partial<Config> = {
	port: 8080,
	cookieAccessTokenName: 'sat',
	cookieRefreshTokenName: 'srt',
};

/**
 * Application class
 */
export class App {
	private fastify: FastifyInstance;
	private readonly config: Config;
	private readonly provider: Provider;
	private cryptoCookie: Crypto;

	/// Construct the application
	private constructor(config: Config) {
		this.config = { ...CONFIG_DEFAULTS, ...config };
		this.fastify = Fastify({ logger: true });
		this.fastify.register(require('fastify-cookie'));
		this.fastify.get('/login', this.login.bind(this));
		this.fastify.get('/login-callback', this.loginCallback.bind(this));
		this.fastify.get('/auth/validate', this.authValidate.bind(this));
		this.fastify.get('/auth/login-callback', this.authLoginCallback.bind(this));
		this.cryptoCookie = new Crypto(this.config.cookieSecret);
		this.provider = new ProviderOpenId(this.config);
	}

	/**
	 * Start the login flow.
	 * @param request
	 * @param reply
	 */
	private async login(request: Request, reply: Reply) {
		const url = await this.provider.getAuthorizationUrl();
		return reply.redirect(url);
	}

	/**
	 * Callback when returning from the provider.
	 * @param request
	 * @param reply
	 */
	private async loginCallback(request: Request, reply: Reply) {
		const tokenSet = await this.provider.grantAuthorizationCode({
			code: request.query.code,
		});
		if (!tokenSet) {
			await this.cookieClear(reply);
			return reply.status(401).send('401 Unauthorized');
		}
		await this.cookieSetFromTokenSet(reply, tokenSet);
		return reply.redirect(this.config.providerRedirectUrl);
	}

	/**
	 * Validate the current request.
	 * @param request
	 * @param reply
	 */
	private async authValidate(request: Request, reply: Reply) {
		const result = await this.userinfoRefresh(request);
		if (!result) {
			const cookies = await this.cookieSerializeClear();
			cookies.forEach((c, i) => reply.header('x-auth-set-cookie-' + (i + 1), c));
			return reply.status(401).send('401 Unauthorized');
		}
		if (result.tokenSet) {
			const cookies = await this.cookieSerializeFromTokenSet(result.tokenSet);
			cookies.forEach((c, i) => reply.header('x-auth-set-cookie-' + (i + 1), c));
			if (result.tokenSet.idToken) {
				reply.header('x-auth-id-token', JSON.stringify(result.tokenSet.idToken));
			}
		}
		reply.header('x-auth-userinfo', JSON.stringify(result.userinfo));
		return '200 OK';
	}
	/**
	 * Login callback, but instead of setting cookies, it sets the x-auth headers to be handled by the upstream.
	 * @param request
	 * @param reply
	 */
	private async authLoginCallback(request: Request, reply: Reply) {
		const tokenSet = await this.provider.grantAuthorizationCode({
			code: request.query.code,
		});
		const cookies = await this.cookieSerializeFromTokenSet(tokenSet);
		cookies.forEach((c, i) => reply.header('x-auth-set-cookie-' + (i + 1), c));
		if (!tokenSet) {
			return reply.status(401).send('401 Unauthorized');
		}
		if (tokenSet.idToken) {
			reply.header('x-auth-id-token', JSON.stringify(tokenSet.idToken));
		}
		reply.header('x-auth-redirect', this.config.providerRedirectUrl);
		return '200 OK';
	}
	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	private async userinfoRefresh(request: Request) {
		const accessToken = await this.cookieGet(request, this.config.cookieAccessTokenName);
		if (accessToken) {
			const userinfo = await this.provider.userinfo(accessToken);
			if (userinfo) return { userinfo };
		}

		const refreshToken = await this.cookieGet(request, this.config.cookieRefreshTokenName);
		if (!refreshToken) {
			return null;
		}

		// Try to refresh the token
		const tokenSet = await this.provider.grantRefreshToken({
			refresh_token: refreshToken,
		});
		if (!tokenSet) return null;

		// Try to get the userinfo
		const userinfo = await this.provider.userinfo(tokenSet.accessToken);
		if (!userinfo) return null;
		return { userinfo, tokenSet };
	}

	/**
	 * Get a cookie from a request.
	 */
	private async cookieSerialize(
		cookieName: string,
		value: string | null | undefined,
		cookieOptions?: FastifyCookieOptions
	): Promise<string> {
		if (value == null) {
			return cookie.serialize(cookieName, '', { expires: new Date(1), path: '/' });
		}
		const cookieValue = await this.cryptoCookie.encrypt(value);
		return cookie.serialize(cookieName, cookieValue, cookieOptions);
	}
	/**
	 * Clear the cookies
	 */
	private async cookieSerializeClear() {
		return Promise.all([
			this.cookieSerialize(this.config.cookieAccessTokenName, null),
			this.cookieSerialize(this.config.cookieRefreshTokenName, null),
		]);
	}
	/**
	 * Serialize the cookies from the token set
	 */
	private cookieSerializeFromTokenSet(tokenSet: ProviderTokenSet | null | undefined) {
		if (!tokenSet) {
			return this.cookieSerializeClear();
		}
		const cookieAccess = this.cookieSerialize(this.config.cookieAccessTokenName, tokenSet.accessToken, {
			expires: tokenSet.expiresAt,
		});
		const cookieRefresh = this.cookieSerialize(this.config.cookieRefreshTokenName, tokenSet.refreshToken);
		return Promise.all([cookieAccess, cookieRefresh]);
	}

	/**
	 * Get a cookie from the request and pass back
	 */
	private async cookieGet(request: Request, cookieName: string): Promise<string | null> {
		const cookieValue = request.cookies[cookieName];
		if (!cookieValue) {
			return null;
		}
		const value = await this.cryptoCookie.decrypt(cookieValue).catch(() => null);
		if (!value) {
			return null;
		}
		return value;
	}

	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	private async cookieClear(reply: Reply) {
		const setCookies = await this.cookieSerializeClear();
		reply.removeHeader('set-cookie');
		reply.header('set-cookie', setCookies);
	}

	/**
	 * Set the cookies from the tokenSet
	 */
	private async cookieSetFromTokenSet(reply: Reply, tokenSet: ProviderTokenSet) {
		const setCookies = await this.cookieSerializeFromTokenSet(tokenSet);
		reply.removeHeader('set-cookie');
		reply.header('set-cookie', setCookies);
	}

	/**
	 * Run the server
	 */
	async run() {
		try {
			await this.fastify.listen(this.config.port, this.config.host);
			this.fastify.log.info(`server listening `, this.fastify.server.address());
		} catch (err) {
			this.fastify.log.error(err);
			throw err;
		}
	}

	/**
	 * Main entrypoint for the application
	 * @param argv
	 */
	static async main(argv: string[]) {
		// prettier-ignore
		const yargs = Yargs(argv)
			.env('AUTH_PROXY')
			.config("config", function (filepath: string) {
				const ext = path.extname(filepath).toLowerCase();
				if (ext === '.json') {
					return JSON.parse(fs.readFileSync(filepath, 'utf8'));
				} else if (ext === '.toml') {
					return toml.parse(fs.readFileSync(filepath, 'utf8'));
				}
				throw new Error(`Invalid config file. Expecting JSON or TOML.`);
			});

		const app = new App(yargs.argv);
		try {
			await app.run();
		} catch (err) {
			process.exit(1);
		}
	}
}
