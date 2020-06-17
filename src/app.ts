import Fastify, { FastifyInstance } from 'fastify';
// @ts-ignore
import Yargs from 'yargs/yargs';
import got from 'got';
import { URL } from 'url';
import fs from 'fs';
import path from 'path';
import toml from 'toml';

type Request = Fastify.FastifyRequest;
type Reply = Fastify.FastifyReply<import('http').ServerResponse>;

type Config = {
	port: number;
	providerClientId: string;
	providerClientSecret: string;
	providerAuthUrl: string;
	providerTokenUrl: string;
	providerValidateUrl: string;
	providerUserinfoUrl: string;
	providerRedirectUrl: string;
};

/**
 * Application class
 */
export class App {
	private fastify: FastifyInstance;

	private constructor(private readonly config: Config) {
		this.fastify = Fastify({ logger: true });
		this.fastify.get('/login', this.login.bind(this));
		this.fastify.get('/callback', this.loginCallback.bind(this));
	}

	/**
	 * Get the userinfo from the request
	 * @param request
	 * @param reply
	 */
	private async login(request: Request, reply: Reply) {
		const url = new URL(this.config.providerAuthUrl);
		url.searchParams.set('response_type', 'code');
		url.searchParams.set('client_id', this.config.providerClientId);
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
		return reply.redirect(this.config.providerRedirectUrl);
	}

	/**
	 * Run the server
	 */
	async run() {
		try {
			await this.fastify.listen(this.config.port);
			this.fastify.log.info(
				`server listening on ${this.fastify.server.address()}`
			);
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
