<script lang="ts">
	let {
		avatarUrl = null,
		displayName = null,
		handle = null,
		index,
		total,
		focused = false,
		overLimit = false
	}: {
		avatarUrl?: string | null;
		displayName?: string | null;
		handle?: string | null;
		index: number;
		total: number;
		focused?: boolean;
		overLimit?: boolean;
	} = $props();
</script>

<div class="card-identity" class:focused class:over-limit={overLimit}>
	{#if avatarUrl}
		<img src={avatarUrl} alt="" class="identity-avatar" />
	{:else}
		<div class="identity-avatar-placeholder"></div>
	{/if}
	<div class="identity-meta">
		{#if displayName}
			<span class="identity-name">{displayName}</span>
		{/if}
		{#if handle}
			<span class="identity-handle">@{handle}</span>
		{/if}
	</div>
	<span class="identity-index">#{index + 1}<span class="identity-total">/{total}</span></span>
</div>

<style>
	.card-identity {
		display: flex;
		align-items: center;
		gap: 8px;
		padding-bottom: 6px;
		margin-bottom: 2px;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-subtle) 30%, transparent);
		min-height: 28px;
	}

	.identity-avatar {
		width: 22px;
		height: 22px;
		border-radius: 50%;
		object-fit: cover;
		flex-shrink: 0;
		border: 1.5px solid var(--color-border-subtle);
		transition: border-color 0.15s ease;
	}

	.card-identity.focused .identity-avatar {
		border-color: var(--color-accent);
	}

	.card-identity.over-limit .identity-avatar {
		border-color: var(--color-danger);
	}

	.identity-avatar-placeholder {
		width: 22px;
		height: 22px;
		border-radius: 50%;
		background: var(--color-surface-active);
		border: 1.5px solid var(--color-border-subtle);
		flex-shrink: 0;
		transition: border-color 0.15s ease;
	}

	.card-identity.focused .identity-avatar-placeholder {
		border-color: var(--color-accent);
	}

	.identity-meta {
		display: flex;
		align-items: baseline;
		gap: 5px;
		min-width: 0;
		flex: 1;
	}

	.identity-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 140px;
		line-height: 1;
	}

	.identity-handle {
		font-size: 12px;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 120px;
		line-height: 1;
	}

	.identity-index {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		flex-shrink: 0;
		letter-spacing: -0.02em;
	}

	.identity-total {
		opacity: 0.5;
	}

	@media (max-width: 640px) {
		.identity-name,
		.identity-handle {
			display: none;
		}

		.card-identity {
			gap: 6px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.identity-avatar,
		.identity-avatar-placeholder {
			transition: none;
		}
	}
</style>
