<script lang="ts">
	import type { Snippet } from 'svelte';
	import { RotateCcw, Lock, Layers } from 'lucide-svelte';
	import { currentAccountId } from '$lib/stores/accounts';
	import { overriddenKeys, resetSectionToBase, saving } from '$lib/stores/settings';
	import {
		type SectionScope,
		isNonDefault,
		isSectionOverridden,
		SECTION_SCOPE
	} from '$lib/stores/settingsScope';

	/* eslint-disable @typescript-eslint/no-explicit-any */
	interface Props {
		id: string;
		title: string;
		description: string;
		icon: any;
		scope?: SectionScope;
		scopeKey?: string;
		children: Snippet;
	}
	/* eslint-enable @typescript-eslint/no-explicit-any */

	let { id, title, description, icon, scope, scopeKey, children }: Props = $props();

	const Icon = $derived(icon);

	let nonDefault = $derived(isNonDefault($currentAccountId));
	let overridden = $derived(
		scope === 'account' && scopeKey ? isSectionOverridden(id, $overriddenKeys) : false
	);
	let showInstanceOverlay = $derived(scope === 'instance' && nonDefault);
	let showAccountBadge = $derived(scope === 'account' && nonDefault);

	let resetting = $state(false);

	async function handleReset() {
		if (!scopeKey || resetting) return;
		resetting = true;
		// If multiple keys share the same scopeKey (e.g. business/persona),
		// the SECTION_SCOPE entry already maps to the right top-level key.
		const entry = SECTION_SCOPE[id];
		if (entry) {
			for (const key of entry.keys) {
				await resetSectionToBase(key);
			}
		}
		resetting = false;
	}
</script>

<section {id} class="settings-section" class:instance-locked={showInstanceOverlay}>
	<div class="section-header">
		<div class="section-icon">
			<Icon size={18} />
		</div>
		<div class="section-info">
			<div class="section-title-row">
				<h2>{title}</h2>
				{#if showAccountBadge}
					{#if overridden}
						<span class="scope-badge overridden">
							<Layers size={12} />
							Overridden
						</span>
					{:else}
						<span class="scope-badge inherited">
							<Layers size={12} />
							Inherited
						</span>
					{/if}
				{/if}
			</div>
			<p>{description}</p>
		</div>
		{#if showAccountBadge && overridden && scopeKey}
			<button
				class="reset-btn"
				onclick={handleReset}
				disabled={resetting || $saving}
				title="Reset to base config"
			>
				<RotateCcw size={13} />
				{resetting ? 'Resetting...' : 'Reset to base'}
			</button>
		{/if}
	</div>

	{#if showInstanceOverlay}
		<div class="instance-banner">
			<Lock size={14} />
			<span>Shared across all accounts. Switch to the default account to edit.</span>
		</div>
	{/if}

	<div class="section-content" class:content-locked={showInstanceOverlay}>
		{@render children()}
	</div>
</section>

<style>
	.settings-section {
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		overflow: hidden;
	}

	.settings-section.instance-locked {
		border-color: color-mix(in srgb, var(--color-border-subtle) 60%, transparent);
	}

	.section-header {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 20px 24px 0;
	}

	.section-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: 8px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		flex-shrink: 0;
	}

	.section-info {
		flex: 1;
		min-width: 0;
	}

	.section-title-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	h2 {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 2px;
	}

	p {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
		line-height: 1.4;
	}

	.scope-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 4px;
		font-size: 11px;
		font-weight: 500;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.scope-badge.overridden {
		background: color-mix(in srgb, var(--color-accent) 14%, transparent);
		color: var(--color-accent);
	}

	.scope-badge.inherited {
		background: color-mix(in srgb, var(--color-text-muted) 10%, transparent);
		color: var(--color-text-subtle);
	}

	.reset-btn {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border: 1px solid var(--color-border);
		border-radius: 5px;
		background: none;
		color: var(--color-text-muted);
		font-size: 12px;
		cursor: pointer;
		white-space: nowrap;
		flex-shrink: 0;
		transition:
			background 0.15s,
			color 0.15s,
			border-color 0.15s;
	}

	.reset-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.reset-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.instance-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 12px 24px 0;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-warning) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-warning) 20%, transparent);
		color: var(--color-text-muted);
		font-size: 12px;
	}

	.section-content {
		padding: 20px 24px 24px;
	}

	.section-content.content-locked {
		pointer-events: none;
		opacity: 0.45;
		user-select: none;
	}
</style>
