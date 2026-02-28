<script lang="ts">
	import { onMount } from 'svelte';
	import { onboardingData } from '$lib/stores/onboarding';
	import { capabilities, loadCapabilities } from '$lib/stores/runtime';
	import { FolderOpen, Cloud } from 'lucide-svelte';

	let sourceType = $state($onboardingData.source_type);
	let vaultPath = $state($onboardingData.vault_path);
	let vaultWatch = $state($onboardingData.vault_watch);
	let vaultLoopBack = $state($onboardingData.vault_loop_back);
	let folderId = $state($onboardingData.folder_id);
	let serviceAccountKey = $state($onboardingData.service_account_key);
	let pollInterval = $state($onboardingData.poll_interval_seconds);
	let browseError = $state('');

	const canLocalFs = $derived($capabilities?.local_folder ?? true);
	const canNativePicker = $derived($capabilities?.file_picker_native ?? false);
	const canGoogleDrive = $derived($capabilities?.google_drive ?? true);

	onMount(() => {
		loadCapabilities();
	});

	// Auto-switch to google_drive when cloud mode disallows local_fs
	$effect(() => {
		if ($capabilities && !$capabilities.local_folder && sourceType === 'local_fs') {
			sourceType = 'google_drive';
		}
	});

	$effect(() => {
		onboardingData.updateField('source_type', sourceType);
	});

	$effect(() => {
		onboardingData.updateField('vault_path', vaultPath);
	});

	$effect(() => {
		onboardingData.updateField('vault_watch', vaultWatch);
	});

	$effect(() => {
		onboardingData.updateField('vault_loop_back', vaultLoopBack);
	});

	$effect(() => {
		onboardingData.updateField('folder_id', folderId);
	});

	$effect(() => {
		onboardingData.updateField('service_account_key', serviceAccountKey);
	});

	$effect(() => {
		onboardingData.updateField('poll_interval_seconds', pollInterval);
	});

	async function browseFolder() {
		browseError = '';
		try {
			const { open } = await import('@tauri-apps/plugin-dialog');
			const selected = await open({
				directory: true,
				title: 'Select content source folder'
			});
			if (selected) {
				vaultPath = selected as string;
			}
		} catch {
			browseError = 'Could not open folder picker';
		}
	}
</script>

<div class="step">
	<h2>Content Source (Optional)</h2>
	<p class="step-description">
		Connect a content source so the Watchtower can index your content
		and use it for smarter replies and tweets. You can configure this later in Settings.
	</p>

	<div class="field-group">
		<label class="field-label" for="source_type_select">
			Source Type
		</label>
		<select
			id="source_type_select"
			class="text-input"
			bind:value={sourceType}
		>
			{#if canLocalFs}
				<option value="local_fs">Local Folder</option>
			{/if}
			{#if canGoogleDrive}
				<option value="google_drive">Google Drive</option>
			{/if}
		</select>
		{#if !canLocalFs}
			<p class="capability-hint">
				Local folder sources are not available in cloud deployments. Connect a Google Drive folder to provide your content.
			</p>
		{/if}
	</div>

	{#if sourceType === 'local_fs'}
		<div class="field-group">
			<label class="field-label" for="vault_path">
				<FolderOpen size={14} />
				Vault / Notes Folder
			</label>
			<div class="path-row">
				<input
					id="vault_path"
					type="text"
					class="text-input path-input"
					bind:value={vaultPath}
					placeholder="~/Documents/my-vault"
				/>
				{#if canNativePicker}
					<button type="button" class="browse-btn" onclick={browseFolder}>
						<FolderOpen size={14} />
						Browse
					</button>
				{/if}
			</div>
			{#if !canNativePicker}
				<span class="field-hint">
					Enter the full server-side path to your content folder.
				</span>
			{/if}
			{#if browseError}
				<span class="field-error">{browseError}</span>
			{/if}
		</div>
	{:else}
		<div class="field-group">
			<label class="field-label" for="folder_id_input">
				<Cloud size={14} />
				Google Drive Folder ID
			</label>
			<input
				id="folder_id_input"
				type="text"
				class="text-input"
				bind:value={folderId}
				placeholder="1aBcD_eFgHiJkLmNoPqRsTuVwXyZ"
			/>
		</div>

		<div class="field-group">
			<label class="field-label" for="sa_key_input">
				Service Account Key Path
			</label>
			<input
				id="sa_key_input"
				type="text"
				class="text-input"
				bind:value={serviceAccountKey}
				placeholder="~/keys/my-project-sa.json"
			/>
		</div>

		<div class="field-group">
			<label class="field-label" for="poll_interval_input">
				Poll Interval (seconds)
			</label>
			<input
				id="poll_interval_input"
				type="number"
				class="text-input poll-input"
				bind:value={pollInterval}
				min="60"
				max="86400"
			/>
		</div>
	{/if}

	<div class="toggle-group">
		<div class="toggle-row">
			<div class="toggle-info">
				<span class="toggle-label">
					{sourceType === 'google_drive' ? 'Poll for changes' : 'Watch for changes'}
				</span>
				<span class="toggle-hint">
					{sourceType === 'google_drive'
						? 'Periodically check for new or modified files'
						: 'Re-index automatically when files change'}
				</span>
			</div>
			<button
				type="button"
				class="toggle"
				class:active={vaultWatch}
				onclick={() => (vaultWatch = !vaultWatch)}
				role="switch"
				aria-checked={vaultWatch}
				aria-label="Toggle file watching"
			>
				<span class="toggle-track">
					<span class="toggle-thumb"></span>
				</span>
			</button>
		</div>

		{#if sourceType === 'local_fs'}
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="toggle-label">Loop back</span>
					<span class="toggle-hint">Write performance metadata into file frontmatter</span>
				</div>
				<button
					type="button"
					class="toggle"
					class:active={vaultLoopBack}
					onclick={() => (vaultLoopBack = !vaultLoopBack)}
					role="switch"
					aria-checked={vaultLoopBack}
					aria-label="Toggle loop back"
				>
					<span class="toggle-track">
						<span class="toggle-thumb"></span>
					</span>
				</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.step {
		display: flex;
		flex-direction: column;
		gap: 24px;
	}

	h2 {
		font-size: 20px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.step-description {
		font-size: 14px;
		color: var(--color-text-muted);
		margin: -16px 0 0;
		line-height: 1.5;
	}

	.field-group {
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
	}

	.field-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	.capability-hint {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: -4px 0 0;
		padding: 10px 14px;
		background: color-mix(in srgb, var(--color-accent) 6%, transparent);
		border-radius: 8px;
		line-height: 1.4;
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

	.toggle-group {
		display: flex;
		flex-direction: column;
		gap: 4px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 4px 16px;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 0;
		cursor: default;
	}

	.toggle-row + .toggle-row {
		border-top: 1px solid var(--color-border-subtle);
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.toggle-label {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text);
	}

	.toggle-hint {
		font-size: 12px;
		color: var(--color-text-muted);
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
</style>
