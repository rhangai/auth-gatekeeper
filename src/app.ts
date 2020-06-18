import Fastify, { FastifyInstance } from 'fastify';
// @ts-ignore
import Yargs from 'yargs/yargs';
import fs from 'fs';
import path from 'path';
import toml from 'toml';
import { Crypto } from './crypto';
import { Provider, ProviderConfig } from './provider/provider';
import { ProviderOpenId } from './provider/openid';
import { FastifyCookieOptions } from 'fastify-cookie';

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
		this.fastify.get('/auth', this.auth.bind(this));
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
		const data: any = await this.provider.grantAuthorizationCode({
			code: request.query.code,
		});
		await this.setCookie(reply, this.config.cookieAccessTokenName, data.access_token, {
			expires: data.expires_in ? new Date(Date.now() + data.expires_in * 1000) : undefined,
		});
		await this.setCookie(reply, this.config.cookieRefreshTokenName, data.refresh_token ?? null);
		return reply.redirect(this.config.providerRedirectUrl);
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	private async auth(request: Request, reply: Reply) {
		const userinfo = await this.userinfoRefresh(request, reply);
		if (!userinfo) {
			return reply.status(401).send('401 Unauthorized');
		}
		reply.header('x-auth-userinfo', JSON.stringify(userinfo));
		return '200 OK';
	}

	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	private async userinfoRefresh(request: Request, reply: Reply) {
		const accessToken = await this.getCookie(request, this.config.cookieAccessTokenName);
		if (accessToken) {
			const userinfo = await this.provider.userinfo(accessToken);
			if (userinfo) return userinfo;
		}

		const refreshToken = await this.getCookie(request, this.config.cookieRefreshTokenName);
		if (refreshToken) {
			const data: any = await this.provider.grantRefreshToken({
				refresh_token: refreshToken,
			});

			await this.setCookie(reply, this.config.cookieAccessTokenName, data.access_token, {
				expires: data.expires_in ? new Date(Date.now() + data.expires_in * 1000) : undefined,
			});
			if (data.refresh_token) {
				await this.setCookie(reply, this.config.cookieRefreshTokenName, data.refresh_token);
			}
			const userinfo = await this.provider.userinfo(data.access_token);
			return userinfo;
		}
	}

	/**
	 * Get a cookie from a request.
	 */
	private async getCookie(request: Request, cookieName: string): Promise<string | null> {
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
	private async setCookie(
		reply: Reply,
		cookieName: string,
		value: string | null,
		cookieOptions?: FastifyCookieOptions
	): Promise<void> {
		console.log(`Setting cookie ${cookieName} to ${value}`);
		if (value == null) {
			reply.clearCookie(cookieName);
			return;
		}
		const cookieValue = await this.cryptoCookie.encrypt(value);
		reply.setCookie(cookieName, cookieValue, cookieOptions);
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
