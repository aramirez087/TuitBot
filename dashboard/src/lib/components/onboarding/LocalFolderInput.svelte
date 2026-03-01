<script lang="ts">
	import { FolderOpen } from 'lucide-svelte';

	interface Props {
		path: string;
		isDesktop: boolean;
		canNativePicker: boolean;
		onpathchange: (path: string) => void;
	}

	let { path, isDesktop, canNativePicker, onpathchange }: Props = $props();
	let browseError = $state('');

	async function browseFolder() {
		browseError = '';
		try {
			const { open } = await import('@tauri-apps/plugin-dialog');
			const selected = await open({
				directory: true,
				title: 'Select Obsidian vault or notes folder'
			});
			if (selected) {
				onpathchange(selected as string);
			}
		} catch {
			browseError = 'Could not open folder picker';
		}
	}

	function handleInput(e: Event) {
		onpathchange((e.target as HTMLInputElement).value);
	}
</script>

<div class="folder-input">
	<label class="field-label" for="vault_path">
		<FolderOpen size={14} />
		{isDesktop ? 'Obsidian Vault / Notes Folder' : 'Server Folder Path'}
	</label>
	<div class="path-row">
		<input
			id="vault_path"
			type="text"
			class="text-input path-input"
			value={path}
			oninput={handleInput}
			placeholder={isDesktop ? '~/Documents/my-vault' : '/data/content'}
		/>
		{#if canNativePicker}
			<button type="button" class="browse-btn" onclick={browseFolder}>
				<FolderOpen size={14} />
				Browse
			</button>
		{/if}
	</div>
	{#if isDesktop}
		<span class="field-hint">
			Select your Obsidian vault or Markdown notes folder.
			Tuitbot indexes your content for smarter replies and posts.
		</span>
	{:else}
		<span class="field-hint">
			Enter the full server-side path to your content folder.
		</span>
	{/if}
	{#if browseError}
		<span class="field-error">{browseError}</span>
	{/if}
</div>

<style>
	.folder-input {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.field-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-muted);
		line-height: 1.4;
	}

	.field-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	.path-row {
		display: flex;
		gap: 8px;
	}

	.text-input {
		padding: 10px 14px;
		background: var(--color-surface);
		border: 2px solid var(--color-border);
		border-radius: 8px;
		color: var(--color-text);
		font-size: 14px;
		font-family: var(--font-sans);
		outline: none;
		transition: border-color 0.15s;
	}

	.text-input:focus {
		border-color: var(--color-accent);
	}

	.path-input {
		flex: 1;
	}

	.browse-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		background: var(--color-surface);
		border: 2px solid var(--color-border);
		border-radius: 8px;
		color: var(--color-text);
		font-size: 14px;
		cursor: pointer;
		white-space: nowrap;
		transition: all 0.15s;
	}

	.browse-btn:hover {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
	}
</style>
