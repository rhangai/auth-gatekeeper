import Fastify, { FastifyInstance } from 'fastify';
import { Provider } from './provider/provider';
import { ProviderOpenId } from './provider/openid';
import { Request, Reply } from './http';
import { CookieManager } from './util/cookie';
import { configParse, Config } from './config';

/**
 * Application class
 */
export class App {
	private fastify: FastifyInstance;
	private readonly cookieManager: CookieManager;
	private readonly provider: Provider;

	/// Construct the application
	private constructor(private readonly config: Config) {
		this.fastify = Fastify({ logger: true });
		this.fastify.register(require('fastify-cookie'));
		this.fastify.get('/login', this.login.bind(this));
		this.fastify.get('/login-callback', this.loginCallback.bind(this));
		this.fastify.get('/auth/validate', this.authValidate.bind(this));
		this.fastify.get('/auth/login-callback', this.authLoginCallback.bind(this));
		this.cookieManager = new CookieManager(this.config.cookie);
		this.provider = new ProviderOpenId(this.config.provider);
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
			await this.cookieManager.clear(reply);
			return reply.status(401).send('401 Unauthorized');
		}
		await this.cookieManager.setFromTokenSet(reply, tokenSet);
		return reply.redirect('/');
	}

	/**
	 * Validate the current request.
	 * @param request
	 * @param reply
	 */
	private async authValidate(request: Request, reply: Reply) {
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
		const cookies = await this.cookieManager.serializeFromTokenSet(tokenSet);
		cookies.forEach((c, i) => reply.header('x-auth-set-cookie-' + (i + 1), c));
		if (!tokenSet) {
			return reply.status(401).send('401 Unauthorized');
		}
		if (tokenSet.idToken) {
			reply.header('x-auth-id-token', JSON.stringify(tokenSet.idToken));
		}
		reply.header('x-auth-redirect', '/');
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
