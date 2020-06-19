import { Crypto } from './util/crypto';

export type State = {
	url?: string;
};

export class StateManager {
	private crypto: Crypto;

	constructor(key: string) {
		this.crypto = new Crypto(key);
	}

	serialize(state: State) {
		const json = JSON.stringify(state);
		return this.crypto.encrypt(json);
	}

	async parse(token: string): Promise<State | null> {
		const data = await this.crypto.decrypt(token).catch(() => null);
		if (!data) return null;
		return JSON.parse(data);
	}
}
