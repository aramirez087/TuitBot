<script lang="ts">
	import { onMount } from 'svelte';
	import { FolderOpen, Cloud, ChevronDown, ChevronRight } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import DriveConnectCard from '$lib/components/onboarding/DriveConnectCard.svelte';
	import { draft, updateDraft } from '$lib/stores/settings';
	import { capabilities, deploymentMode, loadCapabilities } from '$lib/stores/runtime';
	import { loadConnections, expiredGoogleDrive } from '$lib/stores/connectors';

	let browseError = $state('');
	let advancedOpen = $state(false);

	const currentSource = $derived($draft?.content_sources?.sources?.[0]);
	const sourceType = $derived(currentSource?.source_type ?? 'local_fs');
	const sourcePath = $derived(currentSource?.path ?? '');
	const folderId = $derived(currentSource?.folder_id ?? '');
	const connectionId = $derived(currentSource?.connection_id ?? null);
	const serviceAccountKey = $derived(currentSource?.service_account_key ?? '');
	const sourceWatch = $derived(currentSource?.watch ?? true);
	const sourceLoopBack = $derived(currentSource?.loop_back_enabled ?? true);
	const filePatterns = $derived(currentSource?.file_patterns ?? ['*.md', '*.txt']);
	const pollInterval = $derived(currentSource?.poll_interval_seconds ?? 300);

	const canLocalFs = $derived($capabilities?.local_folder ?? true);
	const canManualPath = $derived($capabilities?.manual_local_path ?? true);
	const canNativePicker = $derived($capabilities?.file_picker_native ?? false);
	const canGoogleDrive = $derived($capabilities?.google_drive ?? true);
	const mode = $derived($deploymentMode);
	const isCloud = $derived(mode === 'cloud');
	const isSelfHost = $derived(mode === 'self_host');
	const isDesktop = $derived(mode === 'desktop');
	const hasLegacySaKey = $derived(serviceAccountKey != null && serviceAccountKey !== '' && !connectionId);
	const SectionIcon = $derived(sourceType === 'google_drive' ? Cloud : FolderOpen);

	onMount(() => { loadCapabilities(); loadConnections(); });

	$effect(() => {
		if ($capabilities && !$capabilities.local_folder && sourceType === 'local_fs') {
			updateSource({ source_type: 'google_drive', path: null, loop_back_enabled: false, poll_interval_seconds: 300 });
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
				connection_id: current?.connection_id ?? null,
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
			updateSource({ source_type: 'google_drive', path: null, loop_back_enabled: false, poll_interval_seconds: 300 });
		} else {
			updateSource({ source_type: 'local_fs', folder_id: null, service_account_key: null, connection_id: null, poll_interval_seconds: null });
		}
	}

	function handlePathInput(e: Event) { updateSource({ path: (e.target as HTMLInputElement).value || null }); }
	function handleFolderIdInput(e: Event) { updateSource({ folder_id: (e.target as HTMLInputElement).value || null }); }
	function handlePollIntervalInput(e: Event) {
		const v = parseInt((e.target as HTMLInputElement).value, 10);
		updateSource({ poll_interval_seconds: isNaN(v) ? 300 : v });
	}

	async function browseFolder() {
		browseError = '';
		try {
			const { open } = await import('@tauri-apps/plugin-dialog');
			const selected = await open({ directory: true, title: 'Select Obsidian vault or notes folder' });
			if (selected) updateSource({ path: selected as string });
		} catch { browseError = 'Could not open folder picker'; }
	}

	function toggleWatch() { updateSource({ watch: !sourceWatch }); }
	function toggleLoopBack() { updateSource({ loop_back_enabled: !sourceLoopBack }); }
	function handleConnected(connId: number, _email: string) { updateSource({ connection_id: connId, service_account_key: null }); }
	function handleDisconnected() { updateSource({ connection_id: null }); }
</script>

{#if $draft}
<SettingsSection id="sources" title="Content Sources" description="Connect a content source for the Watchtower to index" icon={SectionIcon}>
	<div class="field-grid">
		{#if !isCloud}
			<div class="field full-width">
				<label class="field-label" for="source_type">Source Type</label>
				{#if isDesktop}
					<select id="source_type" class="text-input" value={sourceType} onchange={handleSourceTypeChange}>
						{#if canLocalFs}<option value="local_fs">Obsidian Vault / Notes Folder</option>{/if}
						{#if canGoogleDrive}<option value="google_drive">Google Drive</option>{/if}
					</select>
				{:else}
					<select id="source_type" class="text-input" value={sourceType} onchange={handleSourceTypeChange}>
						{#if canGoogleDrive}<option value="google_drive">Google Drive</option>{/if}
						{#if canLocalFs}<option value="local_fs">Local Server Folder</option>{/if}
					</select>
				{/if}
				<span class="field-hint">Choose where your content lives</span>
			</div>
		{:else}
			<div class="field full-width">
				<div class="notice notice-info">Local folder sources are not available in cloud deployments. Use Google Drive to provide content.</div>
			</div>
		{/if}

		{#if sourceType === 'local_fs' && (canLocalFs || canManualPath)}
			{#if isSelfHost && !advancedOpen}
				<div class="field full-width">
					<button type="button" class="advanced-toggle" onclick={() => (advancedOpen = true)}>
						<ChevronRight size={14} /> Advanced: Local Server Folder
					</button>
				</div>
			{:else}
				{#if isSelfHost}
					<div class="field full-width">
						<button type="button" class="advanced-toggle" onclick={() => (advancedOpen = false)}>
							<ChevronDown size={14} /> Advanced: Local Server Folder
						</button>
					</div>
				{/if}
				<div class="field full-width">
					<label class="field-label" for="source_path">{isDesktop ? 'Obsidian Vault / Notes Folder' : 'Server Folder Path'}</label>
					<div class="path-row">
						<input id="source_path" type="text" class="text-input path-input" value={sourcePath} oninput={handlePathInput} placeholder={isDesktop ? '~/Documents/my-vault' : '/data/content'} />
						{#if canNativePicker}
							<button type="button" class="browse-btn" onclick={browseFolder}><FolderOpen size={14} /> Browse</button>
						{/if}
					</div>
					<span class="field-hint">{canNativePicker ? 'Click Browse to select your Obsidian vault or notes folder.' : 'Enter the full server-side path to your content folder.'}</span>
					{#if browseError}<span class="field-error">{browseError}</span>{/if}
				</div>
			{/if}
		{:else if sourceType === 'google_drive'}
			<div class="field full-width">
				<DriveConnectCard onconnected={handleConnected} ondisconnected={handleDisconnected} />
			</div>
			{#if hasLegacySaKey}
				<div class="field full-width"><div class="notice notice-warning">Using legacy service account key. Connect a Google account above to upgrade to the linked account flow.</div></div>
			{/if}
			{#if $expiredGoogleDrive && connectionId}
				<div class="field full-width"><div class="notice notice-danger">Your Google Drive authorization has expired. Click "Reconnect" above to restore syncing.</div></div>
			{/if}
			<div class="field full-width">
				<label class="field-label" for="folder_id">Google Drive Folder ID (optional)</label>
				<input id="folder_id" type="text" class="text-input" value={folderId} oninput={handleFolderIdInput} placeholder="1aBcD_eFgHiJkLmNoPqRsTuVwXyZ" />
				<span class="field-hint">The folder ID from your Google Drive URL. Leave blank to index your entire Drive.</span>
			</div>
			<div class="field full-width">
				<label class="field-label" for="poll_interval">Poll Interval (seconds)</label>
				<input id="poll_interval" type="number" class="text-input poll-input" value={pollInterval} oninput={handlePollIntervalInput} min="60" max="86400" />
				<span class="field-hint">How often to check Google Drive for changes (minimum 60s)</span>
			</div>
		{/if}

		<div class="field full-width">
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="field-label">{sourceType === 'google_drive' ? 'Poll for Changes' : 'Watch for Changes'}</span>
					<span class="field-hint">{sourceType === 'google_drive' ? 'Periodically check for new or modified files' : 'Automatically re-index when files are added or modified'}</span>
				</div>
				<button type="button" class="toggle" class:active={sourceWatch} onclick={toggleWatch} role="switch" aria-checked={sourceWatch} aria-label="Toggle file watching">
					<span class="toggle-track"><span class="toggle-thumb"></span></span>
				</button>
			</div>
		</div>
		{#if sourceType === 'local_fs'}
			<div class="field full-width">
				<div class="toggle-row">
					<div class="toggle-info">
						<span class="field-label">Loop Back</span>
						<span class="field-hint">Write performance metadata back into source file frontmatter</span>
					</div>
					<button type="button" class="toggle" class:active={sourceLoopBack} onclick={toggleLoopBack} role="switch" aria-checked={sourceLoopBack} aria-label="Toggle loop back">
						<span class="toggle-track"><span class="toggle-thumb"></span></span>
					</button>
				</div>
			</div>
		{/if}
		<div class="field full-width">
			<span class="field-label">File Patterns</span>
			<div class="patterns">{#each filePatterns as pattern}<span class="pattern-tag">{pattern}</span>{/each}</div>
			<span class="field-hint">File patterns are configured in config.toml. Default: *.md, *.txt</span>
		</div>
	</div>
</SettingsSection>
{/if}

<style>
	.field-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
	.field { display: flex; flex-direction: column; gap: 6px; }
	.full-width { grid-column: 1 / -1; }
	.field-label { font-size: 13px; font-weight: 500; color: var(--color-text); }
	.field-hint { font-size: 12px; color: var(--color-text-subtle); }
	.field-error { font-size: 12px; color: var(--color-danger); }
	.notice { padding: 10px 14px; border-radius: 6px; font-size: 12px; line-height: 1.5; }
	.notice-info { background: color-mix(in srgb, var(--color-accent) 8%, transparent); border: 1px solid color-mix(in srgb, var(--color-accent) 15%, transparent); color: var(--color-text-subtle); }
	.notice-warning { background: color-mix(in srgb, var(--color-warning, #f59e0b) 8%, transparent); border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 20%, transparent); color: var(--color-text-subtle); }
	.notice-danger { background: color-mix(in srgb, var(--color-danger) 8%, transparent); border: 1px solid color-mix(in srgb, var(--color-danger) 20%, transparent); color: var(--color-danger); }
	.advanced-toggle { display: flex; align-items: center; gap: 6px; padding: 8px 12px; background: var(--color-surface-hover); border: 1px solid var(--color-border); border-radius: 6px; color: var(--color-text-muted); font-size: 13px; font-weight: 500; cursor: pointer; transition: all 0.15s; }
	.advanced-toggle:hover { color: var(--color-text); border-color: var(--color-accent); }
	.path-row { display: flex; gap: 8px; }
	.text-input { padding: 8px 12px; background: var(--color-base); border: 1px solid var(--color-border); border-radius: 6px; color: var(--color-text); font-size: 13px; font-family: var(--font-sans); outline: none; transition: border-color 0.15s; }
	.text-input:focus { border-color: var(--color-accent); }
	.path-input { flex: 1; }
	.poll-input { max-width: 160px; }
	select.text-input { cursor: pointer; }
	.browse-btn { display: inline-flex; align-items: center; gap: 6px; padding: 8px 14px; background: var(--color-surface-hover); border: 1px solid var(--color-border); border-radius: 6px; color: var(--color-text); font-size: 13px; cursor: pointer; white-space: nowrap; transition: all 0.15s; }
	.browse-btn:hover { background: var(--color-accent); border-color: var(--color-accent); color: white; }
	.toggle-row { display: flex; align-items: center; justify-content: space-between; padding: 8px 0; }
	.toggle-info { display: flex; flex-direction: column; gap: 2px; }
	.toggle { border: none; background: none; padding: 0; cursor: pointer; }
	.toggle-track { display: flex; align-items: center; width: 42px; height: 24px; padding: 2px; background: var(--color-border); border-radius: 12px; transition: background 0.2s; }
	.toggle.active .toggle-track { background: var(--color-accent); }
	.toggle-thumb { width: 20px; height: 20px; background: white; border-radius: 50%; transition: transform 0.2s; box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2); }
	.toggle.active .toggle-thumb { transform: translateX(18px); }
	.patterns { display: flex; gap: 6px; flex-wrap: wrap; }
	.pattern-tag { padding: 4px 10px; background: color-mix(in srgb, var(--color-accent) 10%, transparent); border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent); border-radius: 4px; font-size: 12px; font-family: monospace; color: var(--color-accent); }
</style>
