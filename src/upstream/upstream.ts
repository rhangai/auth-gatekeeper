export interface Upstream {
	onIdToken?(idToken: Record<string, any>): Promise<void>;
}
