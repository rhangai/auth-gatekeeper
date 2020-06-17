import Fastify, { FastifyInstance } from 'fastify';
// @ts-ignore
import Yargs from 'yargs/yargs';
import got from 'got';
import { URL } from 'url';
import fs from 'fs';
import path from 'path';
import toml from 'toml';
import { Crypto } from './crypto';
import zlib from 'zlib';

type Request = Fastify.FastifyRequest;
type Reply = Fastify.FastifyReply<import('http').ServerResponse>;

type Config = {
	port: number;
	cookieSecret: string;
	cookieAccessTokenName: string;
	cookieRefreshTokenName: string;
	providerClientId: string;
	providerClientSecret: string;
	providerAuthUrl: string;
	providerTokenUrl: string;
	providerValidateUrl: string;
	providerUserinfoUrl: string;
	providerRedirectUrl: string;
};

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
	private cookieCrypto: Crypto;

	/// Construct the application
	private constructor(config: Config) {
		this.config = { ...CONFIG_DEFAULTS, ...config };
		this.fastify = Fastify({ logger: true });
		this.fastify.register(require('fastify-cookie'));
		this.fastify.get('/login', this.login.bind(this));
		this.fastify.get('/callback', this.loginCallback.bind(this));
		this.fastify.get('/auth', this.auth.bind(this));
		this.cookieCrypto = new Crypto('oi');
	}

	/**
	 * Perform the login. Redirecting the user to the oauth provider class.
	 * @param request
	 * @param reply
	 */
	private async login(request: Request, reply: Reply) {
		const url = new URL(this.config.providerAuthUrl);
		url.searchParams.set('response_type', 'code');
		url.searchParams.set('client_id', this.config.providerClientId);
		url.searchParams.set('state', '123456');
		url.searchParams.set('scope', 'openid email profile');
		url.searchParams.set('redirect_uri', this.config.providerRedirectUrl);
		return reply.redirect(url.href);
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	private async loginCallback(request: Request, reply: Reply) {
		const { body } = await got.post({
			responseType: 'json',
			url: this.config.providerTokenUrl,
			form: {
				grant_type: 'authorization_code',
				code: request.query.code,
				client_id: this.config.providerClientId,
				client_secret: this.config.providerClientSecret,
				redirect_uri: this.config.providerRedirectUrl,
			},
		});
		const data = body as any;
		const accessToken: string = await this.cookieCrypto.encrypt(data.access_token);
		reply.setCookie(this.config.cookieAccessTokenName, accessToken);
		return reply.redirect(this.config.providerRedirectUrl);
	}

	/**
	 * Perform the callback on the login form.
	 * @param request
	 * @param reply
	 */
	private async auth(request: Request, reply: Reply) {
		const userinfo = await this.userinfoFromRequestReply(request, reply);
		if (!userinfo) {
			return reply.status(401).send('401 Unauthorized');
		}
		reply.header('x-auth-userinfo', JSON.stringify(userinfo));
		return '200 OK';
	}

	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	private async userinfoFromRequestReply(request: Request, reply: Reply) {
		const cookieAccessToken = request.cookies[this.config.cookieAccessTokenName];
		if (!cookieAccessToken) {
			return null;
		}
		const accessToken = await this.cookieCrypto.decrypt(cookieAccessToken).catch(() => null);
		if (!accessToken) {
			return null;
		}
		return await this.userinfoFromAccessToken(accessToken);
	}

	/**
	 * Get the userinfo from the access token
	 */
	private async userinfoFromAccessToken(accessToken: string | null) {
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
			}
			throw err;
		}
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
			})
			.option('port', {
				default: 8080,
			});

		const app = new App(yargs.argv);
		try {
			await app.run();
		} catch (err) {
			process.exit(1);
		}
	}
}
