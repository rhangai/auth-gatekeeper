import { Upstream } from './upstream';

export type UpstreamRestConfig = {
	upstreamType: 'rest';
	upstreamLoginUrl: string;
};

/**
 *
 */
export class UpstreamRest implements Upstream {
	/**
	 *
	 * @param token
	 */
	async onIdToken(idToken: Record<string, any>) {}
}
