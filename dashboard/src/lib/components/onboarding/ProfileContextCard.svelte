<script lang="ts">
	import { CheckCircle2, User, Building2, Loader2, Sparkles } from 'lucide-svelte';
	import type { OnboardingXUser } from '$lib/stores/onboarding-session';

	interface FieldMeta { color: string; label: string; }

	interface Props {
		xConnected: boolean;
		xUser: OnboardingXUser | null;
		accountType: string;
		accountTypeMeta: FieldMeta | null;
		enriching: boolean;
		enrichDone: boolean;
		onAccountTypeChange: (type: 'individual' | 'business') => void;
	}

	const {
		xConnected,
		xUser,
		accountType,
		accountTypeMeta,
		enriching,
		enrichDone,
		onAccountTypeChange,
	}: Props = $props();
</script>

{#if enriching}
	<div class="enrich-banner">
		<span class="spinner"><Loader2 size={14} /></span>
		<span>Enriching profile with AI...</span>
	</div>
{:else if enrichDone}
	<div class="enrich-banner enrich-done">
		<Sparkles size={14} />
		<span>Profile enriched with AI — review the updated fields below.</span>
	</div>
{/if}

{#if xConnected && xUser}
	<div class="connected-card">
		{#if xUser.profile_image_url}
			<img src={xUser.profile_image_url} alt="" class="avatar" />
		{:else}
			<div class="avatar avatar-placeholder"></div>
		{/if}
		<div class="connected-info">
			<span class="display-name">{xUser.name}</span>
			<span class="username">@{xUser.username}</span>
		</div>
		<div class="connected-badge">
			<CheckCircle2 size={16} />
			Connected
		</div>
	</div>
{/if}

<div class="account-type-toggle">
	<button
		class="type-btn"
		class:selected={accountType !== 'business'}
		onclick={() => onAccountTypeChange('individual' as const)}
	>
		<User size={16} />
		<span>Individual</span>
	</button>
	<button
		class="type-btn"
		class:selected={accountType === 'business'}
		onclick={() => onAccountTypeChange('business' as const)}
	>
		<Building2 size={16} />
		<span>Business / Product</span>
	</button>
	{#if accountTypeMeta}
		<span class="toggle-provenance" style="color: var(--color-text-subtle)">
			<span class="confidence-dot" style="background: {accountTypeMeta.color}"></span>
			{accountTypeMeta.label}
		</span>
	{/if}
</div>

<style>
	.enrich-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		background: color-mix(in srgb, var(--color-accent) 6%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, var(--color-border));
		border-radius: 8px;
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.enrich-done {
		background: color-mix(in srgb, var(--color-success, #22c55e) 6%, var(--color-base));
		border-color: color-mix(in srgb, var(--color-success, #22c55e) 25%, var(--color-border));
		color: var(--color-success, #22c55e);
	}

	.spinner {
		display: inline-flex;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.connected-card {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		background: color-mix(in srgb, var(--color-success, #22c55e) 6%, var(--color-base));
		border: 1px solid color-mix(in srgb, var(--color-success, #22c55e) 25%, var(--color-border));
		border-radius: 8px;
	}

	.avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
	}

	.avatar-placeholder {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
	}

	.connected-info {
		display: flex;
		flex-direction: column;
		gap: 1px;
		flex: 1;
		min-width: 0;
	}

	.display-name {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.username {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.connected-badge {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		font-weight: 600;
		color: var(--color-success, #22c55e);
		flex-shrink: 0;
	}

	.account-type-toggle {
		display: flex;
		gap: 8px;
		align-items: center;
		flex-wrap: wrap;
	}

	.type-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 10px 16px;
		border: 2px solid var(--color-border);
		border-radius: 8px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.type-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-text);
	}

	.type-btn.selected {
		border-color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface));
		color: var(--color-text);
	}

	.toggle-provenance {
		font-size: 11px;
		display: flex;
		align-items: center;
		gap: 4px;
		flex-basis: 100%;
	}

	.confidence-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		display: inline-block;
		flex-shrink: 0;
	}
</style>
