import Fastify, { FastifyInstance } from 'fastify';
import { Provider, providerCreate } from './provider/provider';
import { Request, Reply } from './http';
import { CookieManager } from './util/cookie';
import { configParse, Config } from './config';
import { Api, apiCreate } from './api/api';

/**
 * Application class
 */
export class App {
	private fastify: FastifyInstance;
	private readonly cookieManager: CookieManager;
	private readonly provider: Provider;
	private readonly api: Api | null;

	/// Construct the application
	private constructor(private readonly config: Config) {
		this.fastify = Fastify({ logger: true });
		this.fastify.register(require('fastify-cookie'));
		this.fastify.get('/login', this.routeLogin.bind(this));
		this.fastify.get('/callback', this.routeCallback.bind(this));
		this.fastify.get('/validate', this.routeValidate.bind(this));
		this.cookieManager = new CookieManager(this.config.cookie);
		this.provider = providerCreate(this.config.provider);
		this.api = apiCreate(this.config.api);
	}

	/**
	 * Start the login flow.
	 * @param request
	 * @param reply
	 */
	private async routeLogin(request: Request, reply: Reply) {
		const url = await this.provider.getAuthorizationUrl();
		return reply.redirect(url);
	}

	/**
	 * Callback when returning from the provider.
	 * @param request
	 * @param reply
	 */
	private async routeCallback(request: Request, reply: Reply) {
		const tokenSet = await this.provider.grantAuthorizationCode({
			code: request.query.code,
		});
		if (!tokenSet) {
			await this.cookieManager.clear(reply);
			return reply.status(401).send('401 Unauthorized');
		}
		if (tokenSet.idToken) {
			await this.api?.onIdToken?.(tokenSet.idToken);
		}
		await this.cookieManager.setFromTokenSet(reply, tokenSet);
		return reply.redirect('/');
	}

	/**
	 * Validate the current request.
	 * @param request
	 * @param reply
	 */
	private async routeValidate(request: Request, reply: Reply) {
		const result = await this.userinfoRefresh(request);
		if (!result) {
			const cookies = await this.cookieManager.serializeClear();
			cookies.forEach((c, i) => reply.header('x-auth-set-cookie-' + (i + 1), c));
			return reply.status(401).send('401 Unauthorized');
		}
		if (result.tokenSet) {
			const cookies = await this.cookieManager.serializeFromTokenSet(result.tokenSet);
			cookies.forEach((c, i) => reply.header('x-auth-set-cookie-' + (i + 1), c));
			if (result.tokenSet.idToken) {
				await this.api?.onIdToken?.(result.tokenSet.idToken);
				reply.header('x-auth-id-token', JSON.stringify(result.tokenSet.idToken));
			}
		}
		reply.header('x-auth-userinfo', JSON.stringify(result.userinfo));
		return '200 OK';
	}
	/**
	 * Get the userinfo from the request/reply object and refresh if needed
	 */
	private async userinfoRefresh(request: Request) {
		const accessToken = await this.cookieManager.getAccessToken(request);
		if (accessToken) {
			const userinfo = await this.provider.userinfo(accessToken);
			if (userinfo) return { userinfo };
		}

		const refreshToken = await this.cookieManager.getRefreshToken(request);
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
		const config = await configParse(argv);
		const app = new App(config);
		try {
			await app.run();
		} catch (err) {
			process.exit(1);
		}
	}
}
