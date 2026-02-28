<script lang="ts">
	import { onMount } from 'svelte';
	import { FolderOpen, Cloud } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import { draft, updateDraft } from '$lib/stores/settings';
	import { capabilities, loadCapabilities } from '$lib/stores/runtime';

	let browseError = $state('');

	const currentSource = $derived($draft?.content_sources?.sources?.[0]);
	const sourceType = $derived(currentSource?.source_type ?? 'local_fs');
	const sourcePath = $derived(currentSource?.path ?? '');
	const folderId = $derived(currentSource?.folder_id ?? '');
	const serviceAccountKey = $derived(currentSource?.service_account_key ?? '');
	const sourceWatch = $derived(currentSource?.watch ?? true);
	const sourceLoopBack = $derived(currentSource?.loop_back_enabled ?? true);
	const filePatterns = $derived(currentSource?.file_patterns ?? ['*.md', '*.txt']);
	const pollInterval = $derived(currentSource?.poll_interval_seconds ?? 300);

	const canLocalFs = $derived($capabilities?.local_folder ?? true);
	const canManualPath = $derived($capabilities?.manual_local_path ?? true);
	const canNativePicker = $derived($capabilities?.file_picker_native ?? false);
	const canGoogleDrive = $derived($capabilities?.google_drive ?? true);

	const SectionIcon = $derived(sourceType === 'google_drive' ? Cloud : FolderOpen);

	onMount(() => {
		loadCapabilities();
	});

	// Auto-switch away from local_fs when capabilities disallow it
	$effect(() => {
		if ($capabilities && !$capabilities.local_folder && sourceType === 'local_fs') {
			updateSource({
				source_type: 'google_drive',
				path: null,
				loop_back_enabled: false,
				poll_interval_seconds: 300
			});
		}
	});

	function updateSource(updates: Record<string, unknown>) {
		const current = $draft?.content_sources?.sources?.[0];
		updateDraft('content_sources', {
			sources: [{
				source_type: current?.source_type ?? 'local_fs',
				path: current?.path ?? null,
				folder_id: current?.folder_id ?? null,
				service_account_key: current?.service_account_key ?? null,
				watch: current?.watch ?? true,
				file_patterns: current?.file_patterns ?? ['*.md', '*.txt'],
				loop_back_enabled: current?.loop_back_enabled ?? true,
				poll_interval_seconds: current?.poll_interval_seconds ?? null,
				...updates
			}]
		});
		browseError = '';
	}

	function handleSourceTypeChange(e: Event) {
		const value = (e.target as HTMLSelectElement).value;
		if (value === 'google_drive') {
			updateSource({
				source_type: 'google_drive',
				path: null,
				loop_back_enabled: false,
				poll_interval_seconds: 300
			});
		} else {
			updateSource({
				source_type: 'local_fs',
				folder_id: null,
				service_account_key: null,
				poll_interval_seconds: null
			});
		}
	}

	function handlePathInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		updateSource({ path: value || null });
	}

	function handleFolderIdInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		updateSource({ folder_id: value || null });
	}

	function handleServiceAccountKeyInput(e: Event) {
		const value = (e.target as HTMLInputElement).value;
		updateSource({ service_account_key: value || null });
	}

	function handlePollIntervalInput(e: Event) {
		const value = parseInt((e.target as HTMLInputElement).value, 10);
		updateSource({ poll_interval_seconds: isNaN(value) ? 300 : value });
	}

	async function browseFolder() {
		browseError = '';
		try {
			const { open } = await import('@tauri-apps/plugin-dialog');
			const selected = await open({
				directory: true,
				title: 'Select content source folder'
			});
			if (selected) {
				updateSource({ path: selected as string });
			}
		} catch {
			browseError = 'Could not open folder picker';
		}
	}

	function toggleWatch() {
		updateSource({ watch: !sourceWatch });
	}

	function toggleLoopBack() {
		updateSource({ loop_back_enabled: !sourceLoopBack });
	}
</script>

{#if $draft}
<SettingsSection
	id="sources"
	title="Content Sources"
	description="Connect a content source for the Watchtower to index"
	icon={SectionIcon}
>
	<div class="field-grid">
		<div class="field full-width">
			<label class="field-label" for="source_type">Source Type</label>
			<select
				id="source_type"
				class="text-input"
				value={sourceType}
				onchange={handleSourceTypeChange}
			>
				{#if canLocalFs}
					<option value="local_fs">Local Folder</option>
				{/if}
				{#if canGoogleDrive}
					<option value="google_drive">Google Drive</option>
				{/if}
			</select>
			<span class="field-hint">
				Choose where your content lives
			</span>
			{#if !canLocalFs}
				<div class="capability-notice">
					Local folder sources are not available in cloud deployments. Use Google Drive or another cloud connector to provide content.
				</div>
			{/if}
		</div>

		{#if sourceType === 'local_fs' && (canLocalFs || canManualPath)}
			<div class="field full-width">
				<label class="field-label" for="source_path">Vault / Notes Folder</label>
				<div class="path-row">
					<input
						id="source_path"
						type="text"
						class="text-input path-input"
						value={sourcePath}
						oninput={handlePathInput}
						placeholder="~/Documents/my-vault"
					/>
					{#if canNativePicker}
						<button type="button" class="browse-btn" onclick={browseFolder}>
							<FolderOpen size={14} />
							Browse
						</button>
					{/if}
				</div>
				<span class="field-hint">
					{#if canNativePicker}
						Click Browse to select your Obsidian vault or notes folder.
					{:else}
						Enter the full server-side path to your content folder.
					{/if}
				</span>
				{#if browseError}
					<span class="field-error">{browseError}</span>
				{/if}
			</div>
		{:else if sourceType === 'google_drive'}
			<div class="field full-width">
				<label class="field-label" for="folder_id">Google Drive Folder ID</label>
				<input
					id="folder_id"
					type="text"
					class="text-input"
					value={folderId}
					oninput={handleFolderIdInput}
					placeholder="1aBcD_eFgHiJkLmNoPqRsTuVwXyZ"
				/>
				<span class="field-hint">
					The folder ID from your Google Drive URL
				</span>
			</div>

			<div class="field full-width">
				<label class="field-label" for="service_account_key">Service Account Key Path</label>
				<input
					id="service_account_key"
					type="text"
					class="text-input"
					value={serviceAccountKey}
					oninput={handleServiceAccountKeyInput}
					placeholder="~/keys/my-project-sa.json"
				/>
				<span class="field-hint">
					Path to a Google service account JSON key file with Drive read access
				</span>
			</div>

			<div class="field full-width">
				<label class="field-label" for="poll_interval">Poll Interval (seconds)</label>
				<input
					id="poll_interval"
					type="number"
					class="text-input poll-input"
					value={pollInterval}
					oninput={handlePollIntervalInput}
					min="60"
					max="86400"
				/>
				<span class="field-hint">
					How often to check Google Drive for changes (minimum 60s)
				</span>
			</div>
		{/if}

		<div class="field full-width">
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="field-label">
						{sourceType === 'google_drive' ? 'Poll for Changes' : 'Watch for Changes'}
					</span>
					<span class="field-hint">
						{sourceType === 'google_drive'
							? 'Periodically check for new or modified files'
							: 'Automatically re-index when files are added or modified'}
					</span>
				</div>
				<button
					type="button"
					class="toggle"
					class:active={sourceWatch}
					onclick={toggleWatch}
					role="switch"
					aria-checked={sourceWatch}
					aria-label="Toggle file watching"
				>
					<span class="toggle-track">
						<span class="toggle-thumb"></span>
					</span>
				</button>
			</div>
		</div>

		{#if sourceType === 'local_fs'}
			<div class="field full-width">
				<div class="toggle-row">
					<div class="toggle-info">
						<span class="field-label">Loop Back</span>
						<span class="field-hint">
							Write performance metadata back into source file frontmatter
						</span>
					</div>
					<button
						type="button"
						class="toggle"
						class:active={sourceLoopBack}
						onclick={toggleLoopBack}
						role="switch"
						aria-checked={sourceLoopBack}
						aria-label="Toggle loop back"
					>
						<span class="toggle-track">
							<span class="toggle-thumb"></span>
						</span>
					</button>
				</div>
			</div>
		{/if}

		<div class="field full-width">
			<span class="field-label">File Patterns</span>
			<div class="patterns">
				{#each filePatterns as pattern}
					<span class="pattern-tag">{pattern}</span>
				{/each}
			</div>
			<span class="field-hint">
				File patterns are configured in config.toml. Default: *.md, *.txt
			</span>
		</div>
	</div>
</SettingsSection>
{/if}

<style>
	.field-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.field-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	.capability-notice {
		padding: 10px 14px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-accent) 15%, transparent);
		border-radius: 6px;
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.5;
	}

	.path-row {
		display: flex;
		gap: 8px;
	}

	.text-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
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

	.poll-input {
		max-width: 160px;
	}

	select.text-input {
		cursor: pointer;
	}

	.browse-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 14px;
		background: var(--color-surface-hover);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		cursor: pointer;
		white-space: nowrap;
		transition: all 0.15s;
	}

	.browse-btn:hover {
		background: var(--color-accent);
		border-color: var(--color-accent);
		color: white;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.toggle {
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.toggle-track {
		display: flex;
		align-items: center;
		width: 42px;
		height: 24px;
		padding: 2px;
		background: var(--color-border);
		border-radius: 12px;
		transition: background 0.2s;
	}

	.toggle.active .toggle-track {
		background: var(--color-accent);
	}

	.toggle-thumb {
		width: 20px;
		height: 20px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
	}

	.toggle.active .toggle-thumb {
		transform: translateX(18px);
	}

	.patterns {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.pattern-tag {
		padding: 4px 10px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 4px;
		font-size: 12px;
		font-family: monospace;
		color: var(--color-accent);
	}
</style>
