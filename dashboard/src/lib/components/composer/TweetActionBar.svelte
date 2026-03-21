<script lang="ts">
	import { MessageCircle, Repeat2, Heart, Bookmark, Share } from 'lucide-svelte';

	let {
		compact = false,
	}: {
		compact?: boolean;
	} = $props();
</script>

<div class="action-bar" class:compact aria-hidden="true">
	<div class="action-item">
		<MessageCircle size={compact ? 15 : 17} />
	</div>
	<div class="action-item">
		<Repeat2 size={compact ? 15 : 17} />
	</div>
	<div class="action-item">
		<Heart size={compact ? 15 : 17} />
	</div>
	<div class="action-item">
		<Bookmark size={compact ? 15 : 17} />
	</div>
	<div class="action-item">
		<Share size={compact ? 15 : 17} />
	</div>
</div>

<style>
	.action-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		max-width: 400px;
		padding: 8px 0 4px;
		color: var(--color-text-subtle);
		opacity: 0.5;
	}

	.action-bar.compact {
		padding: 6px 0 2px;
		max-width: 340px;
	}

	.action-item {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
	}

	.action-bar.compact .action-item {
		width: 28px;
		height: 28px;
	}
</style>
