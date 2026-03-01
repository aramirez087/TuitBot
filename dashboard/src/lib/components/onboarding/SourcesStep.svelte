<script lang="ts">
	import { onMount } from 'svelte';
	import { onboardingData } from '$lib/stores/onboarding';
	import { capabilities, deploymentMode, loadCapabilities } from '$lib/stores/runtime';
	import { activeGoogleDrive } from '$lib/stores/connectors';
	import { Cloud, ChevronDown, ChevronRight } from 'lucide-svelte';
	import DriveConnectCard from './DriveConnectCard.svelte';
	import LocalFolderInput from './LocalFolderInput.svelte';

	let sourceType = $state($onboardingData.source_type);
	let vaultPath = $state($onboardingData.vault_path);
	let vaultWatch = $state($onboardingData.vault_watch);
	let vaultLoopBack = $state($onboardingData.vault_loop_back);
	let folderId = $state($onboardingData.folder_id);
	let pollInterval = $state($onboardingData.poll_interval_seconds);
	let advancedOpen = $state(false);
	let sourceInitialized = $state(false);

	const canLocalFs = $derived($capabilities?.local_folder ?? true);
	const canNativePicker = $derived($capabilities?.file_picker_native ?? false);
	const canGoogleDrive = $derived($capabilities?.google_drive ?? true);
	const mode = $derived($deploymentMode);
	const isCloud = $derived(mode === 'cloud');
	const isSelfHost = $derived(mode === 'self_host');
	const isDesktop = $derived(mode === 'desktop');

	onMount(() => { loadCapabilities(); });

	// Initialize source type from preferred_source_default (fires once)
	$effect(() => {
		if ($capabilities && !sourceInitialized) {
			const pref = $capabilities.preferred_source_default;
			if (pref && pref !== sourceType) { sourceType = pref; }
			sourceInitialized = true;
		}
	});

	// Guard: force google_drive when local_fs is not available
	$effect(() => {
		if ($capabilities && !$capabilities.local_folder && sourceType === 'local_fs') {
			sourceType = 'google_drive';
		}
	});

	$effect(() => { onboardingData.updateField('source_type', sourceType); });
	$effect(() => { onboardingData.updateField('vault_path', vaultPath); });
	$effect(() => { onboardingData.updateField('vault_watch', vaultWatch); });
	$effect(() => { onboardingData.updateField('vault_loop_back', vaultLoopBack); });
	$effect(() => { onboardingData.updateField('folder_id', folderId); });
	$effect(() => { onboardingData.updateField('poll_interval_seconds', pollInterval); });

	function handleConnected(connectionId: number, _email: string) {
		onboardingData.updateField('connection_id', connectionId);
	}

	function handleDisconnected() {
		onboardingData.updateField('connection_id', null);
	}
</script>

<div class="step">
	<h2>Content Source (Optional)</h2>
	<p class="step-description">
		Connect a content source so the Watchtower can index your content
		and use it for smarter replies and tweets. You can configure this later in Settings.
	</p>

	{#if !isCloud}
		<div class="field-group">
			<label class="field-label" for="source_type_select">Source Type</label>
			{#if isDesktop}
				<select id="source_type_select" class="text-input" bind:value={sourceType}>
					{#if canLocalFs}<option value="local_fs">Obsidian Vault / Notes Folder</option>{/if}
					{#if canGoogleDrive}<option value="google_drive">Google Drive</option>{/if}
				</select>
			{:else}
				<select id="source_type_select" class="text-input" bind:value={sourceType}>
					{#if canGoogleDrive}<option value="google_drive">Google Drive</option>{/if}
					{#if canLocalFs}<option value="local_fs">Local Server Folder</option>{/if}
				</select>
			{/if}
		</div>
	{:else}
		<p class="capability-hint">
			Local folder sources are not available in cloud deployments.
			Connect a Google Drive folder to provide your content.
		</p>
	{/if}

	{#if sourceType === 'local_fs' && canLocalFs}
		{#if isSelfHost && !advancedOpen}
			<button type="button" class="advanced-toggle" onclick={() => (advancedOpen = true)}>
				<ChevronRight size={14} /> Advanced: Local Server Folder
			</button>
		{:else}
			{#if isSelfHost}
				<button type="button" class="advanced-toggle" onclick={() => (advancedOpen = false)}>
					<ChevronDown size={14} /> Advanced: Local Server Folder
				</button>
			{/if}
			<LocalFolderInput
				path={vaultPath}
				{isDesktop}
				{canNativePicker}
				onpathchange={(p) => (vaultPath = p)}
			/>
		{/if}
	{/if}

	{#if sourceType === 'google_drive' || isCloud}
		<DriveConnectCard onconnected={handleConnected} ondisconnected={handleDisconnected} />

		{#if $activeGoogleDrive || $onboardingData.connection_id}
			<div class="field-group">
				<label class="field-label" for="folder_id_input">
					<Cloud size={14} /> Google Drive Folder ID (optional)
				</label>
				<input id="folder_id_input" type="text" class="text-input" bind:value={folderId}
					placeholder="1aBcD_eFgHiJkLmNoPqRsTuVwXyZ" />
				<span class="field-hint">
					Without a folder ID, Tuitbot will index your entire Drive.
					Find the folder ID in the Google Drive URL after /folders/.
				</span>
			</div>
			<div class="field-group">
				<label class="field-label" for="poll_interval_input">Poll Interval (seconds)</label>
				<input id="poll_interval_input" type="number" class="text-input poll-input"
					bind:value={pollInterval} min="60" max="86400" />
			</div>
		{:else}
			<p class="connect-hint">
				Connect your Google Drive account to enable automatic syncing.
				You can skip this step and configure it later in Settings.
			</p>
		{/if}
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
			<button type="button" class="toggle" class:active={vaultWatch}
				onclick={() => (vaultWatch = !vaultWatch)} role="switch"
				aria-checked={vaultWatch} aria-label="Toggle file watching">
				<span class="toggle-track"><span class="toggle-thumb"></span></span>
			</button>
		</div>
		{#if sourceType === 'local_fs'}
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="toggle-label">Loop back</span>
					<span class="toggle-hint">Write performance metadata into file frontmatter</span>
				</div>
				<button type="button" class="toggle" class:active={vaultLoopBack}
					onclick={() => (vaultLoopBack = !vaultLoopBack)} role="switch"
					aria-checked={vaultLoopBack} aria-label="Toggle loop back">
					<span class="toggle-track"><span class="toggle-thumb"></span></span>
				</button>
			</div>
		{/if}
	</div>
</div>

<style>
	.step { display: flex; flex-direction: column; gap: 24px; }
	h2 { font-size: 20px; font-weight: 700; color: var(--color-text); margin: 0; }
	.step-description { font-size: 14px; color: var(--color-text-muted); margin: -16px 0 0; line-height: 1.5; }
	.field-group { display: flex; flex-direction: column; gap: 10px; }
	.field-label { display: flex; align-items: center; gap: 6px; font-size: 13px; font-weight: 600; color: var(--color-text); }
	.field-hint { font-size: 12px; color: var(--color-text-muted); line-height: 1.4; }
	.capability-hint { font-size: 13px; color: var(--color-text-muted); margin: -4px 0 0; padding: 10px 14px; background: color-mix(in srgb, var(--color-accent) 6%, transparent); border-radius: 8px; line-height: 1.4; }
	.connect-hint { font-size: 13px; color: var(--color-text-muted); padding: 10px 14px; background: color-mix(in srgb, var(--color-accent) 6%, transparent); border-radius: 8px; line-height: 1.4; margin: 0; }
	.text-input { padding: 10px 14px; background: var(--color-surface); border: 2px solid var(--color-border); border-radius: 8px; color: var(--color-text); font-size: 14px; font-family: var(--font-sans); outline: none; transition: border-color 0.15s; }
	.text-input:focus { border-color: var(--color-accent); }
	.poll-input { max-width: 160px; }
	select.text-input { cursor: pointer; }
	.advanced-toggle { display: flex; align-items: center; gap: 6px; padding: 10px 14px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; color: var(--color-text-muted); font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
	.advanced-toggle:hover { color: var(--color-text); border-color: var(--color-accent); }
	.toggle-group { display: flex; flex-direction: column; gap: 4px; background: var(--color-surface); border: 1px solid var(--color-border); border-radius: 8px; padding: 4px 16px; }
	.toggle-row { display: flex; align-items: center; justify-content: space-between; padding: 12px 0; cursor: default; }
	.toggle-row + .toggle-row { border-top: 1px solid var(--color-border-subtle); }
	.toggle-info { display: flex; flex-direction: column; gap: 2px; }
	.toggle-label { font-size: 14px; font-weight: 500; color: var(--color-text); }
	.toggle-hint { font-size: 12px; color: var(--color-text-muted); }
	.toggle { border: none; background: none; padding: 0; cursor: pointer; }
	.toggle-track { display: flex; align-items: center; width: 42px; height: 24px; padding: 2px; background: var(--color-border); border-radius: 12px; transition: background 0.2s; }
	.toggle.active .toggle-track { background: var(--color-accent); }
	.toggle-thumb { width: 20px; height: 20px; background: white; border-radius: 50%; transition: transform 0.2s; box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2); }
	.toggle.active .toggle-thumb { transform: translateX(18px); }
</style>
