<script lang="ts">
	import { LayoutDashboard } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import {
		homeSurface,
		loadHomeSurface,
		setHomeSurface,
		type HomeSurface
	} from '$lib/stores/homeSurface';
	import { onMount, onDestroy } from 'svelte';

	let current = $state<HomeSurface>('composer');
	let loaded = $state(false);

	let unsub: (() => void) | null = null;

	onMount(async () => {
		await loadHomeSurface();
		unsub = homeSurface.subscribe((v) => {
			current = v;
		});
		loaded = true;
	});

	onDestroy(() => {
		unsub?.();
	});

	async function handleChange(value: HomeSurface) {
		current = value;
		await setHomeSurface(value);
	}
</script>

<SettingsSection
	id="workspace"
	title="Workspace"
	description="Configure your default workspace experience"
	icon={LayoutDashboard}
>
	{#if !loaded}
		<div class="loading-text">Loading...</div>
	{:else}
		<div class="field-group">
			<h3 class="group-title">Default Landing Page</h3>
			<p class="group-desc">Choose what you see when you open Tuitbot</p>

			<div class="option-cards">
				<button
					class="option-card"
					class:selected={current === 'composer'}
					onclick={() => handleChange('composer')}
					aria-pressed={current === 'composer'}
				>
					<div class="option-radio" class:checked={current === 'composer'}>
						{#if current === 'composer'}
							<div class="radio-dot"></div>
						{/if}
					</div>
					<div class="option-content">
						<span class="option-label">
							Composer home
							<span class="recommended-badge">Recommended</span>
						</span>
						<span class="option-desc">Jump straight into writing</span>
					</div>
				</button>

				<button
					class="option-card"
					class:selected={current === 'analytics'}
					onclick={() => handleChange('analytics')}
					aria-pressed={current === 'analytics'}
				>
					<div class="option-radio" class:checked={current === 'analytics'}>
						{#if current === 'analytics'}
							<div class="radio-dot"></div>
						{/if}
					</div>
					<div class="option-content">
						<span class="option-label">Analytics overview</span>
						<span class="option-desc">See your growth dashboard first</span>
					</div>
				</button>
			</div>

			<p class="field-hint">Takes effect on next visit to the home page</p>
		</div>
	{/if}
</SettingsSection>

<style>
	.loading-text {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.field-group {
		margin-bottom: 0;
	}

	.group-title {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-subtle);
		margin: 0 0 4px;
	}

	.group-desc {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0 0 12px;
		line-height: 1.4;
	}

	.option-cards {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.option-card {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		padding: 12px 14px;
		background: var(--color-base);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		cursor: pointer;
		text-align: left;
		transition:
			border-color 0.15s,
			background 0.15s;
	}

	.option-card:hover {
		border-color: var(--color-border);
		background: var(--color-surface-hover);
	}

	.option-card.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-base));
	}

	.option-card:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: 2px;
	}

	.option-radio {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		border: 2px solid var(--color-border);
		flex-shrink: 0;
		transition: border-color 0.15s;
	}

	.option-radio.checked {
		border-color: var(--color-accent);
	}

	.radio-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-accent);
	}

	.option-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.option-label {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.recommended-badge {
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		padding: 1px 6px;
		border-radius: 4px;
		background: color-mix(in srgb, var(--color-accent) 14%, transparent);
		color: var(--color-accent);
		line-height: 1.5;
	}

	.option-desc {
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.4;
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
		margin: 10px 0 0;
	}
</style>
