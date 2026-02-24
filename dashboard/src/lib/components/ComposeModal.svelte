<script lang="ts">
	import type { ScheduleConfig } from '$lib/api';
	import { X, Plus, Trash2, Send } from 'lucide-svelte';
	import TimePicker from './TimePicker.svelte';

	let {
		open,
		prefillTime = null,
		prefillDate = null,
		schedule,
		onclose,
		onsubmit
	}: {
		open: boolean;
		prefillTime?: string | null;
		prefillDate?: Date | null;
		schedule: ScheduleConfig | null;
		onclose: () => void;
		onsubmit: (data: { content_type: string; content: string; scheduled_for?: string }) => void;
	} = $props();

	let mode = $state<'tweet' | 'thread'>('tweet');
	let tweetText = $state('');
	let threadParts = $state<string[]>(['', '']);
	let selectedTime = $state<string | null>(null);
	let submitting = $state(false);
	let submitError = $state<string | null>(null);

	const targetDate = $derived(prefillDate ?? new Date());
	const dateLabel = $derived(
		targetDate.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })
	);

	// Sync prefillTime when modal opens
	$effect(() => {
		if (open) {
			selectedTime = prefillTime ?? null;
			tweetText = '';
			threadParts = ['', ''];
			mode = 'tweet';
			submitting = false;
			submitError = null;
		}
	});

	const TWEET_MAX = 280;
	const tweetChars = $derived(tweetText.length);
	const tweetOverLimit = $derived(tweetChars > TWEET_MAX);

	const canSubmitTweet = $derived(tweetText.trim().length > 0 && !tweetOverLimit);
	const canSubmitThread = $derived(
		threadParts.filter((p) => p.trim().length > 0).length >= 2 &&
			threadParts.every((p) => p.length <= TWEET_MAX)
	);
	const canSubmit = $derived(mode === 'tweet' ? canSubmitTweet : canSubmitThread);

	function addThreadPart() {
		threadParts = [...threadParts, ''];
	}

	function removeThreadPart(index: number) {
		if (threadParts.length <= 2) return;
		threadParts = threadParts.filter((_, i) => i !== index);
	}

	function updateThreadPart(index: number, value: string) {
		threadParts = threadParts.map((p, i) => (i === index ? value : p));
	}

	async function handleSubmit() {
		if (!canSubmit || submitting) return;
		submitting = true;
		submitError = null;

		try {
			const content =
				mode === 'tweet' ? tweetText.trim() : JSON.stringify(threadParts.map((p) => p.trim()).filter(Boolean));

			const data: { content_type: string; content: string; scheduled_for?: string } = {
				content_type: mode,
				content
			};

			if (selectedTime) {
				// Build ISO datetime from target date + selected time
				const scheduled = new Date(targetDate);
				const [h, m] = selectedTime.split(':').map(Number);
				scheduled.setHours(h, m, 0, 0);
				data.scheduled_for = scheduled.toISOString().replace('Z', '');
			}

			onsubmit(data);
		} catch (e) {
			submitError = e instanceof Error ? e.message : 'Failed to submit';
			submitting = false;
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onclose();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			onclose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={handleBackdropClick}>
		<div class="modal">
			<div class="modal-header">
				<div class="modal-title">
					<h2>Compose</h2>
					<span class="date-subtitle">{dateLabel}</span>
				</div>
				<button class="close-btn" onclick={onclose}>
					<X size={16} />
				</button>
			</div>

			<div class="mode-tabs">
				<button class="tab" class:active={mode === 'tweet'} onclick={() => (mode = 'tweet')}>
					Tweet
				</button>
				<button class="tab" class:active={mode === 'thread'} onclick={() => (mode = 'thread')}>
					Thread
				</button>
			</div>

			<div class="modal-body">
				{#if mode === 'tweet'}
					<div class="tweet-compose">
						<textarea
							class="compose-input"
							class:over-limit={tweetOverLimit}
							placeholder="What's on your mind?"
							bind:value={tweetText}
							rows={4}
						></textarea>
						<div class="char-counter" class:over-limit={tweetOverLimit}>
							{tweetChars}/{TWEET_MAX}
						</div>
					</div>
				{:else}
					<div class="thread-compose">
						{#each threadParts as part, i}
							<div class="thread-part">
								<div class="thread-part-header">
									<span class="thread-num">{i + 1}/{threadParts.length}</span>
									{#if threadParts.length > 2}
										<button class="remove-part-btn" onclick={() => removeThreadPart(i)}>
											<Trash2 size={12} />
										</button>
									{/if}
								</div>
								<textarea
									class="compose-input thread-input"
									class:over-limit={part.length > TWEET_MAX}
									placeholder={i === 0 ? 'Start your thread...' : 'Continue...'}
									value={part}
									oninput={(e) => updateThreadPart(i, e.currentTarget.value)}
									rows={3}
								></textarea>
								<div class="char-counter" class:over-limit={part.length > TWEET_MAX}>
									{part.length}/{TWEET_MAX}
								</div>
							</div>
						{/each}
						<button class="add-part-btn" onclick={addThreadPart}>
							<Plus size={14} />
							Add tweet
						</button>
					</div>
				{/if}

				<div class="schedule-section">
					<TimePicker
						{schedule}
						{selectedTime}
						targetDate={targetDate}
						onselect={(time) => (selectedTime = time || null)}
					/>
				</div>

				{#if submitError}
					<div class="error-msg">{submitError}</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="cancel-btn" onclick={onclose}>Cancel</button>
				<button class="submit-btn" onclick={handleSubmit} disabled={!canSubmit || submitting}>
					<Send size={14} />
					{submitting ? 'Submitting...' : selectedTime ? 'Schedule' : 'Post now'}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		width: 520px;
		max-width: 90vw;
		max-height: 85vh;
		overflow-y: auto;
		box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.modal-title {
		display: flex;
		align-items: baseline;
		gap: 8px;
	}

	.modal-header h2 {
		font-size: 16px;
		font-weight: 600;
		margin: 0;
		color: var(--color-text);
	}

	.date-subtitle {
		font-size: 13px;
		font-weight: 400;
		color: var(--color-text-muted);
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.close-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.mode-tabs {
		display: flex;
		gap: 0;
		padding: 0 20px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.tab {
		padding: 10px 16px;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.tab:hover {
		color: var(--color-text);
	}

	.tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	.modal-body {
		padding: 20px;
	}

	.compose-input {
		width: 100%;
		padding: 10px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--color-base);
		color: var(--color-text);
		font-size: 14px;
		font-family: var(--font-sans);
		line-height: 1.5;
		resize: vertical;
		box-sizing: border-box;
		transition: border-color 0.15s ease;
	}

	.compose-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.compose-input.over-limit {
		border-color: var(--color-danger);
	}

	.char-counter {
		text-align: right;
		font-size: 11px;
		color: var(--color-text-subtle);
		margin-top: 4px;
		font-family: var(--font-mono);
	}

	.char-counter.over-limit {
		color: var(--color-danger);
		font-weight: 600;
	}

	.thread-compose {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.thread-part {
		position: relative;
	}

	.thread-part-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 4px;
	}

	.thread-num {
		font-size: 11px;
		font-weight: 500;
		color: var(--color-text-muted);
		font-family: var(--font-mono);
	}

	.remove-part-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: var(--color-text-subtle);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.remove-part-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-danger);
	}

	.thread-input {
		font-size: 13px;
	}

	.add-part-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		border: 1px dashed var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.add-part-btn:hover {
		border-color: var(--color-accent);
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
	}

	.schedule-section {
		margin-top: 16px;
		padding-top: 16px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.error-msg {
		margin-top: 12px;
		padding: 8px 12px;
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 12px;
	}

	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 20px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.cancel-btn {
		padding: 8px 16px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 13px;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.cancel-btn:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.submit-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 20px;
		border: none;
		border-radius: 6px;
		background: var(--color-accent);
		color: #fff;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.submit-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.submit-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
</style>
