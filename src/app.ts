import Fastify, { FastifyInstance } from 'fastify';
// @ts-ignore
import Yargs from 'yargs/yargs';
import fs from 'fs';
import path from 'path';
import toml from 'toml';
import { Crypto } from './crypto';
import { Provider, ProviderConfig, ProviderTokenSet } from './provider/provider';
import { ProviderOpenId } from './provider/openid';
import { FastifyCookieOptions } from 'fastify-cookie';
// @ts-ignore
import cookie from 'cookie';

type Request = Fastify.FastifyRequest;
type Reply = Fastify.FastifyReply<import('http').ServerResponse>;
export type Config = {
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
		this.fastify.get('/callback', this.loginCallback.bind(this));
		this.fastify.get('/auth/validate', this.authValidate.bind(this));
		this.fastify.get('/auth/callback', this.authCallback.bind(this));
		this.cryptoCookie = new Crypto('oi');
		this.provider = new ProviderOpenId(this.config);
	}

	/**
	 * Perform the login. Redirecting the user to the oauth provider class.
	 * @param request
	 * @param reply
	 */
	private async login(request: Request, reply: Reply) {
		const url = await this.provider.getAuthorizationUrl();
		return reply.redirect(url);
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	private async loginCallback(request: Request, reply: Reply) {
		const tokenSet = await this.provider.grantAuthorizationCode({
			code: request.query.code,
		});
		if (!tokenSet) {
			this.cookieClear(reply);
			return reply.status(401).send('401 Unauthorized');
		}
		await this.cookieSetFromTokenSet(reply, tokenSet);
		return reply.redirect(this.config.providerRedirectUrl);
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	private async authValidate(request: Request, reply: Reply) {
		const result = await this.userinfoRefresh(request);
		if (!result) {
			this.cookieClear(reply);
			return reply.status(401).send('401 Unauthorized');
		}
		if (result.tokenSet) {
			await this.cookieSetFromTokenSet(reply, result.tokenSet);
			if (result.tokenSet.idToken) {
				reply.header('x-auth-id-token', JSON.stringify(result.tokenSet.idToken));
			}
		}
		reply.header('x-auth-userinfo', JSON.stringify(result.userinfo));
		return '200 OK';
	}
	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	private async authCallback(request: Request, reply: Reply) {
		const tokenSet = await this.provider.grantAuthorizationCode({
			code: request.query.code,
		});
		if (!tokenSet) {
			return reply.status(401).send('401 Unauthorized');
		}
		const cookieAccessToken = '';
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
	 * Get a cookie from a request.
	 */
	private async cookieSet(
		reply: Reply,
		cookieName: string,
		value: string | null | undefined,
		cookieOptions?: FastifyCookieOptions
	): Promise<void> {
		if (value == null) {
			reply.clearCookie(cookieName);
			return;
		}
		const cookieValue = await this.cryptoCookie.encrypt(value);
		reply.setCookie(cookieName, cookieValue, cookieOptions);
	}
	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	private cookieClear(reply: Reply) {
		reply.clearCookie(this.config.cookieAccessTokenName);
		reply.clearCookie(this.config.cookieRefreshTokenName);
	}

	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	private async cookieSetFromTokenSet(reply: Reply, tokenSet: ProviderTokenSet) {
		await this.cookieSet(reply, this.config.cookieAccessTokenName, tokenSet.accessToken, {
			expires: tokenSet.expiresAt,
		});
		await this.cookieSet(reply, this.config.cookieRefreshTokenName, tokenSet.refreshToken);
	}

	/**
	 * Run the server
	 */
	async run() {
		try {
			await this.fastify.listen(this.config.port);
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
