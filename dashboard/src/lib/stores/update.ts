import { writable } from 'svelte/store';

export const updateAvailable = writable(false);

/** Check for updates silently on app start. */
export async function checkForUpdate() {
	try {
		const { check } = await import('@tauri-apps/plugin-updater');
		const update = await check();
		if (update) {
			updateAvailable.set(true);
		}
	} catch {
		// Not in Tauri or updater not configured — ignore.
	}
}

/** Download and install the available update. */
export async function installUpdate() {
	try {
		const { check } = await import('@tauri-apps/plugin-updater');
		const update = await check();
		if (update) {
			await update.downloadAndInstall();
			// Relaunch after install
			try {
				const process = await import('@tauri-apps/api/core');
				await process.invoke('plugin:process|restart');
			} catch {
				// If process restart fails, the user can restart manually.
			}
		}
	} catch (e) {
		console.error('Failed to install update:', e);
	}
}
