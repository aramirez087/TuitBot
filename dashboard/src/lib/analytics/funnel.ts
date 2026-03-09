export interface FunnelEvent {
	event: string;
	properties?: Record<string, unknown>;
	timestamp: string;
}

export function trackFunnel(event: string, properties?: Record<string, unknown>): void {
	const entry: FunnelEvent = {
		event,
		properties,
		timestamp: new Date().toISOString(),
	};
	console.info('[tuitbot:funnel]', JSON.stringify(entry));
}
