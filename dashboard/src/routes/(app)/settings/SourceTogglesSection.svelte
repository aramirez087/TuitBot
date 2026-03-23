<script lang="ts">
	let {
		sourceType,
		sourceWatch,
		sourceLoopBack,
		sourceAnalyticsSync,
		toggleWatch,
		toggleLoopBack,
		toggleAnalyticsSync
	}: {
		sourceType: string;
		sourceWatch: boolean;
		sourceLoopBack: boolean;
		sourceAnalyticsSync: boolean;
		toggleWatch: () => void;
		toggleLoopBack: () => void;
		toggleAnalyticsSync: () => void;
	} = $props();
</script>

<div class="field full-width">
	<div class="toggle-row">
		<div class="toggle-info">
			<span class="field-label">{sourceType === 'google_drive' ? 'Poll for Changes' : 'Watch for Changes'}</span>
			<span class="field-hint">{sourceType === 'google_drive' ? 'Polls your Drive folder at the configured interval for new or modified files' : 'Automatically re-index when local files are added or modified'}</span>
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
				<span class="field-hint">Write publish metadata (tweet ID, URL, timestamp) back into note frontmatter after posting. This records which notes were used and when.</span>
			</div>
			<button type="button" class="toggle" class:active={sourceLoopBack} onclick={toggleLoopBack} role="switch" aria-checked={sourceLoopBack} aria-label="Toggle loop back">
				<span class="toggle-track"><span class="toggle-thumb"></span></span>
			</button>
		</div>
	</div>
	{#if sourceLoopBack}
		<div class="field full-width">
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="field-label">Analytics Sync</span>
					<span class="field-hint">Periodically enrich note frontmatter with engagement metrics (impressions, likes, retweets, engagement rate, performance score). Writes are local-only — data stays in your vault files. Stats typically arrive 15–60 minutes after posting.</span>
				</div>
				<button type="button" class="toggle" class:active={sourceAnalyticsSync} onclick={toggleAnalyticsSync} role="switch" aria-checked={sourceAnalyticsSync} aria-label="Toggle analytics sync">
					<span class="toggle-track"><span class="toggle-thumb"></span></span>
				</button>
			</div>
		</div>
	{/if}
{/if}
{#if sourceType === 'google_drive'}
	<div class="field full-width">
		<div class="notice notice-info">
			Analytics sync (writing performance data back to notes) is only available for local filesystem sources. Google Drive sources receive publish metadata only.
		</div>
	</div>
{/if}

<style>
	.field { display: flex; flex-direction: column; gap: 6px; }
	.full-width { grid-column: 1 / -1; }
	.field-label { font-size: 13px; font-weight: 500; color: var(--color-text); }
	.field-hint { font-size: 12px; color: var(--color-text-subtle); }
	.notice { padding: 10px 14px; border-radius: 6px; font-size: 12px; line-height: 1.5; }
	.notice-info { background: color-mix(in srgb, var(--color-accent) 8%, transparent); border: 1px solid color-mix(in srgb, var(--color-accent) 15%, transparent); color: var(--color-text-subtle); }
	.toggle-row { display: flex; align-items: center; justify-content: space-between; padding: 8px 0; }
	.toggle-info { display: flex; flex-direction: column; gap: 2px; }
	.toggle { border: none; background: none; padding: 0; cursor: pointer; }
	.toggle-track { display: flex; align-items: center; width: 42px; height: 24px; padding: 2px; background: var(--color-border); border-radius: 12px; transition: background 0.2s; }
	.toggle.active .toggle-track { background: var(--color-accent); }
	.toggle-thumb { width: 20px; height: 20px; background: white; border-radius: 50%; transition: transform 0.2s; box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2); }
	.toggle.active .toggle-thumb { transform: translateX(18px); }
</style>
