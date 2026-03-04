<script lang="ts">
	import { RefreshCw, X, ArrowRight } from 'lucide-svelte';

	let {
		visible,
		mode,
		ondismiss,
		onuseexample
	}: {
		visible: boolean;
		mode: 'tweet' | 'thread';
		ondismiss: () => void;
		onuseexample: (text: string) => void;
	} = $props();

	const TWEET_PROMPTS = [
		"What's a common misconception in your industry?",
		'What did you learn this week that surprised you?',
		'The best advice I got early in my career was...',
		'3 tools I can\'t work without:',
		"Here's what most people get wrong about...",
		"What's one small change that had the biggest impact on your work?",
		'The most underrated skill in my field is...',
		'Something I changed my mind about recently:',
		'A mistake I made that taught me the most:',
		'If I had to start over, the first thing I\'d do is...',
		'One habit that changed how I work:',
		'The question I wish more people would ask:'
	];

	const THREAD_PROMPTS = [
		'5 lessons from a recent project',
		"How I approached a problem — a breakdown",
		"The beginner's guide to something I know well",
		'Common mistakes in my field and how to avoid them',
		'My framework for making decisions',
		'What I wish I knew when starting out',
		'A step-by-step guide to a process I use daily',
		'The tools and workflows behind my best work'
	];

	let promptIndex = $state(0);

	const prompts = $derived(mode === 'tweet' ? TWEET_PROMPTS : THREAD_PROMPTS);
	const currentPrompt = $derived(prompts[promptIndex % prompts.length]);

	function nextPrompt() {
		promptIndex = (promptIndex + 1) % prompts.length;
	}

	function usePrompt() {
		onuseexample(currentPrompt);
	}
</script>

{#if visible}
	<div class="prompt-card">
		<div class="prompt-header">
			<span class="prompt-label">Need inspiration?</span>
			<button class="prompt-close" onclick={ondismiss} aria-label="Dismiss prompts">
				<X size={14} />
			</button>
		</div>
		<p class="prompt-text">{currentPrompt}</p>
		<div class="prompt-actions">
			<button class="prompt-btn primary" onclick={usePrompt}>
				Use this
				<ArrowRight size={12} />
			</button>
			<button class="prompt-btn" onclick={nextPrompt}>
				<RefreshCw size={12} />
				Another
			</button>
		</div>
	</div>
{/if}

<style>
	.prompt-card {
		margin: 12px 16px 0;
		padding: 14px 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		background: var(--color-surface);
		animation: prompt-in 0.2s ease;
	}

	.prompt-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	.prompt-label {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--color-text-subtle);
	}

	.prompt-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.prompt-close:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.prompt-text {
		font-size: 14px;
		line-height: 1.5;
		color: var(--color-text);
		margin: 0 0 12px;
		font-style: italic;
	}

	.prompt-actions {
		display: flex;
		gap: 6px;
	}

	.prompt-btn {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 12px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 5px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.prompt-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.prompt-btn.primary {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		border-color: color-mix(in srgb, var(--color-accent) 20%, transparent);
		color: var(--color-accent);
	}

	.prompt-btn.primary:hover {
		background: color-mix(in srgb, var(--color-accent) 18%, transparent);
		border-color: color-mix(in srgb, var(--color-accent) 35%, transparent);
	}

	@keyframes prompt-in {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.prompt-card {
			animation: none;
		}
	}

</style>
