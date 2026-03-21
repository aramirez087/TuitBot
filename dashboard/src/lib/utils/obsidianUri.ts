/**
 * Obsidian URI deep-link utilities for desktop builds.
 *
 * These helpers construct `obsidian://open` URIs and delegate to the
 * Tauri `open_external_url` command.  They are safe to import in any
 * build — the Tauri invoke is behind a dynamic import that no-ops
 * silently outside the desktop shell.
 */

/**
 * Build an `obsidian://open` URI for a specific note.
 *
 * The vault name is derived from the last path component of `vaultPath`
 * (e.g. `/Users/alice/notes/marketing` → `marketing`).  Returns `null`
 * when the vault name cannot be determined.
 */
export function buildObsidianUri(vaultPath: string, relativePath: string, heading?: string): string | null {
	const vaultName = vaultPath.split('/').filter(Boolean).pop();
	if (!vaultName) return null;
	const encodedVault = encodeURIComponent(vaultName);
	// Obsidian expects the file path without the .md extension
	const encodedFile = encodeURIComponent(relativePath.replace(/\.md$/, ''));
	let uri = `obsidian://open?vault=${encodedVault}&file=${encodedFile}`;
	if (heading) {
		// Extract the deepest heading segment for the anchor
		const leaf = heading.split(' > ').pop()?.trim();
		if (leaf) uri += `#${encodeURIComponent(leaf)}`;
	}
	return uri;
}

/**
 * Build an `obsidian://open` URI that opens the vault root (no file).
 */
export function buildObsidianVaultUri(vaultPath: string): string | null {
	const vaultName = vaultPath.split('/').filter(Boolean).pop();
	if (!vaultName) return null;
	return `obsidian://open?vault=${encodeURIComponent(vaultName)}`;
}

/**
 * Open a URI via the Tauri `open_external_url` command.
 *
 * Returns `true` on success, `false` when not running inside Tauri
 * (web / self-host) or if the invoke fails.
 */
export async function openExternalUrl(url: string): Promise<boolean> {
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		await invoke('open_external_url', { url });
		return true;
	} catch {
		return false;
	}
}
