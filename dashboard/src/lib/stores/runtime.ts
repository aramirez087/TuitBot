import { writable, derived } from 'svelte/store';
import { api, type DeploymentCapabilities, type DeploymentModeValue } from '$lib/api';

interface RuntimeCapabilities {
	deployment_mode: DeploymentModeValue;
	capabilities: DeploymentCapabilities;
}

const DESKTOP_DEFAULTS: RuntimeCapabilities = {
	deployment_mode: 'desktop',
	capabilities: {
		local_folder: true,
		manual_local_path: true,
		google_drive: true,
		inline_ingest: true,
		file_picker_native: false
	}
};

const capabilitiesData = writable<RuntimeCapabilities | null>(null);

export const capabilities = derived(capabilitiesData, ($d) => $d?.capabilities ?? null);
export const deploymentMode = derived(capabilitiesData, ($d) => $d?.deployment_mode ?? 'desktop');
export const capabilitiesLoaded = derived(capabilitiesData, ($d) => $d !== null);

let fetching = false;
let fetched = false;

export async function loadCapabilities(): Promise<void> {
	if (fetched || fetching) return;
	fetching = true;
	try {
		// Try authenticated runtime status first
		const status = await api.runtime.status();
		capabilitiesData.set({
			deployment_mode: status.deployment_mode,
			capabilities: status.capabilities
		});
		fetched = true;
	} catch {
		// Fall back to unauthenticated config status (works during onboarding)
		try {
			const configStatus = await api.settings.configStatus();
			capabilitiesData.set({
				deployment_mode: configStatus.deployment_mode,
				capabilities: configStatus.capabilities
			});
			fetched = true;
		} catch {
			// Server unreachable — assume desktop defaults
			capabilitiesData.set(DESKTOP_DEFAULTS);
			fetched = true;
		}
	} finally {
		fetching = false;
	}
}
