<script lang="ts">
	import { CheckCircle2, XCircle } from 'lucide-svelte';
</script>

<div class="feature-matrix">
	<p class="matrix-heading">What you can do</p>
	<ul class="matrix-list">
		<li class="available"><CheckCircle2 size={14} /> Search and discover tweets</li>
		<li class="available">
			<CheckCircle2 size={14} /> Score conversations for relevance
		</li>
		<li class="available">
			<CheckCircle2 size={14} /> Draft replies and original content
		</li>
		<li class="available">
			<CheckCircle2 size={14} /> Plan and preview threads
		</li>
		<li class="unavailable">
			<XCircle size={14} /> Post tweets and replies
		</li>
		<li class="unavailable">
			<XCircle size={14} /> Mentions and home timeline
		</li>
	</ul>
	<p class="matrix-footer">
		You can switch to the Official X API anytime in Settings.
	</p>
</div>

<style>
	.feature-matrix {
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 14px 16px;
	}

	.matrix-heading {
		font-size: 12px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		margin: 0 0 10px;
	}

	.matrix-list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.matrix-list li {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
		line-height: 1.4;
	}

	.matrix-list li.available {
		color: var(--color-success, #22c55e);
	}

	.matrix-list li.unavailable {
		color: var(--color-text-subtle);
	}

	.matrix-footer {
		margin: 12px 0 0;
		font-size: 12px;
		color: var(--color-text-subtle);
		line-height: 1.4;
	}
</style>
