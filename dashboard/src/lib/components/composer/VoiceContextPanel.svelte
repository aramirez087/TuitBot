<script lang="ts">
	import { ChevronDown, ChevronRight, Settings } from 'lucide-svelte';
	import { config, loadSettings } from '$lib/stores/settings';
	import { get } from 'svelte/store';

	let {
		cue,
		oncuechange
	}: {
		cue: string;
		oncuechange: (cue: string) => void;
	} = $props();

	const EXPANDED_KEY = 'tuitbot:voice:expanded';
	const SAVED_CUES_KEY = 'tuitbot:voice:saved-cues';
	const MAX_SAVED_CUES = 5;

	let expanded = $state(loadExpandedState());
	let savedCues = $state<string[]>(loadSavedCues());
	let showSavedDropdown = $state(false);

	function loadExpandedState(): boolean {
		try {
			return localStorage.getItem(EXPANDED_KEY) === 'true';
		} catch {
			return false;
		}
	}

	function loadSavedCues(): string[] {
		try {
			const raw = localStorage.getItem(SAVED_CUES_KEY);
			if (!raw) return [];
			const parsed = JSON.parse(raw);
			return Array.isArray(parsed) ? parsed.slice(0, MAX_SAVED_CUES) : [];
		} catch {
			return [];
		}
	}

	function toggleExpanded() {
		expanded = !expanded;
		try {
			localStorage.setItem(EXPANDED_KEY, String(expanded));
		} catch {
			/* quota exceeded */
		}
	}

	function selectCue(selected: string) {
		oncuechange(selected);
		showSavedDropdown = false;
	}

	export function saveCueToHistory() {
		if (!cue.trim()) return;
		const trimmed = cue.trim();
		const updated = [trimmed, ...savedCues.filter((c) => c !== trimmed)].slice(0, MAX_SAVED_CUES);
		savedCues = updated;
		try {
			localStorage.setItem(SAVED_CUES_KEY, JSON.stringify(updated));
		} catch {
			/* quota exceeded */
		}
	}

	// Ensure settings are loaded
	$effect(() => {
		const current = get(config);
		if (!current) {
			loadSettings();
		}
	});

	const brandVoice = $derived($config?.business?.brand_voice ?? '');
	const contentStyle = $derived($config?.business?.content_style ?? '');
	const pillars = $derived($config?.business?.content_pillars ?? []);
	const hasVoiceSettings = $derived(!!brandVoice || !!contentStyle || pillars.length > 0);

	function truncate(text: string, max: number): string {
		if (text.length <= max) return text;
		return text.slice(0, max) + '\u2026';
	}
</script>

<div class="voice-context-panel" class:expanded>
	<button class="voice-toggle" onclick={toggleExpanded}>
		{#if expanded}
			<ChevronDown size={14} />
		{:else}
			<ChevronRight size={14} />
		{/if}
		<span class="voice-toggle-label">Voice context</span>
		{#if cue.trim()}
			<span class="active-cue-badge">{truncate(cue.trim(), 20)}</span>
		{/if}
	</button>

	{#if expanded}
		<div class="voice-body">
			{#if hasVoiceSettings}
				<div class="voice-summary">
					{#if brandVoice}
						<span class="voice-chip" title={brandVoice}>{truncate(brandVoice, 40)}</span>
					{/if}
					{#if contentStyle}
						<span class="voice-chip" title={contentStyle}>{truncate(contentStyle, 30)}</span>
					{/if}
					{#each pillars.slice(0, 3) as pillar}
						<span class="pillar-chip">{pillar}</span>
					{/each}
				</div>
			{:else}
				<div class="voice-empty">
					<Settings size={12} />
					<span>Set voice in Settings &rarr; Content Persona</span>
				</div>
			{/if}

			<div class="quick-cue-row">
				<input
					class="cue-input"
					type="text"
					placeholder="Tone cue (e.g., 'more casual', 'add a hot take')"
					value={cue}
					oninput={(e) => oncuechange(e.currentTarget.value)}
					onfocus={() => { if (savedCues.length > 0) showSavedDropdown = true; }}
					onblur={() => { setTimeout(() => { showSavedDropdown = false; }, 150); }}
				/>
				{#if showSavedDropdown && savedCues.length > 0}
					<div class="saved-cues-dropdown">
						{#each savedCues as saved}
							<button class="saved-cue-item" onmousedown={() => selectCue(saved)}>
								{saved}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.voice-context-panel {
		margin-bottom: 12px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: var(--color-base);
		overflow: hidden;
	}

	.voice-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 8px 10px;
		border: none;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: color 0.15s ease;
	}

	.voice-toggle:hover {
		color: var(--color-text);
	}

	.voice-toggle-label {
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.active-cue-badge {
		margin-left: auto;
		padding: 1px 6px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
		font-size: 11px;
		font-weight: 400;
	}

	.voice-body {
		padding: 0 10px 10px;
	}

	.voice-summary {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-bottom: 8px;
	}

	.voice-chip {
		padding: 2px 8px;
		border-radius: 4px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
		font-size: 11px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 200px;
	}

	.pillar-chip {
		padding: 2px 8px;
		border-radius: 4px;
		background: var(--color-surface-hover);
		color: var(--color-text-muted);
		font-size: 11px;
	}

	.voice-empty {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 8px;
		font-size: 11px;
		color: var(--color-text-subtle);
	}

	.quick-cue-row {
		position: relative;
	}

	.cue-input {
		width: 100%;
		padding: 6px 10px;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		background: var(--color-surface);
		color: var(--color-text);
		font-size: 12px;
		font-family: var(--font-sans);
		box-sizing: border-box;
		transition: border-color 0.15s ease;
	}

	.cue-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.cue-input::placeholder {
		color: var(--color-text-subtle);
	}

	.saved-cues-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 2px;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		z-index: 10;
		overflow: hidden;
	}

	.saved-cue-item {
		display: block;
		width: 100%;
		padding: 6px 10px;
		border: none;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		text-align: left;
		cursor: pointer;
		transition: background 0.1s ease;
	}

	.saved-cue-item:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}
</style>
