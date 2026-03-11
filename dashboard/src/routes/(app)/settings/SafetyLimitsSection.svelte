<script lang="ts">
	import { Shield } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import SliderInput from '$lib/components/settings/SliderInput.svelte';
	import TagInput from '$lib/components/settings/TagInput.svelte';
	import { defaults, draft, updateDraft } from '$lib/stores/settings';
</script>

{#if $draft}
<SettingsSection
	id="limits"
	title="Safety & Limits"
	description="Rate limits, delays, and content safety rules"
	icon={Shield}
	scope="account"
	scopeKey="limits"
>
	<div class="field-grid">
		<div class="field">
			<SliderInput
				value={$draft.limits.max_replies_per_day}
				label="Max Replies / Day"
				min={1}
				max={50}
				defaultValue={$defaults?.limits.max_replies_per_day}
				onchange={(v) =>
					updateDraft('limits.max_replies_per_day', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.limits.max_tweets_per_day}
				label="Max Tweets / Day"
				min={1}
				max={20}
				defaultValue={$defaults?.limits.max_tweets_per_day}
				onchange={(v) =>
					updateDraft('limits.max_tweets_per_day', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.limits.max_threads_per_week}
				label="Max Threads / Week"
				min={0}
				max={7}
				defaultValue={$defaults?.limits.max_threads_per_week}
				onchange={(v) =>
					updateDraft('limits.max_threads_per_week', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.limits.max_replies_per_author_per_day}
				label="Max Replies / Author / Day"
				min={1}
				max={10}
				defaultValue={$defaults?.limits.max_replies_per_author_per_day}
				onchange={(v) =>
					updateDraft('limits.max_replies_per_author_per_day', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.limits.min_action_delay_seconds}
				label="Min Action Delay"
				min={10}
				max={600}
				unit="s"
				helpText="Minimum seconds between actions"
				defaultValue={$defaults?.limits.min_action_delay_seconds}
				onchange={(v) =>
					updateDraft('limits.min_action_delay_seconds', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.limits.max_action_delay_seconds}
				label="Max Action Delay"
				min={10}
				max={600}
				unit="s"
				helpText="Maximum seconds between actions"
				defaultValue={$defaults?.limits.max_action_delay_seconds}
				onchange={(v) =>
					updateDraft('limits.max_action_delay_seconds', v)}
			/>
		</div>

		<div class="field full-width">
			<SliderInput
				value={$draft.limits.product_mention_ratio}
				label="Product Mention Ratio"
				min={0}
				max={1}
				step={0.05}
				unit=""
				helpText="Fraction of replies that may mention your product ({(
					$draft.limits.product_mention_ratio * 100
				).toFixed(0)}%)"
				defaultValue={$defaults?.limits.product_mention_ratio}
				onchange={(v) =>
					updateDraft('limits.product_mention_ratio', v)}
			/>
		</div>

		<div class="field full-width">
			<TagInput
				value={$draft.limits.banned_phrases}
				label="Banned Phrases"
				placeholder="Add phrases that should never appear in content"
				helpText="Content containing these phrases will be regenerated"
				defaultValue={$defaults?.limits.banned_phrases}
				onchange={(tags) =>
					updateDraft('limits.banned_phrases', tags)}
			/>
		</div>

		<div class="field full-width">
			<label class="field-label" for="mode">Operating Mode</label>
			<select
				id="mode"
				class="text-input"
				value={$draft.mode ?? 'autopilot'}
				onchange={(e) => updateDraft('mode', e.currentTarget.value)}
			>
				<option value="autopilot">Autopilot — fully autonomous posting</option>
				<option value="composer">Composer — you control what gets posted</option>
			</select>
			<span class="field-hint">
				{#if ($draft.mode ?? 'autopilot') === 'composer'}
					Composer mode disables autonomous loops. Manual compose and scheduling are always available.
				{:else}
					Autopilot runs discovery, content generation, and posting automatically. Manual scheduling is always available.
				{/if}
			</span>
		</div>

		<div class="field full-width">
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="field-label">Approval Mode</span>
					<span class="field-hint">
						Queue all posts for manual review before publishing. Scheduled posts retain their target time and will post at the scheduled time after approval.{#if ($draft.mode ?? 'autopilot') === 'composer'} Always on in Composer mode.{/if}
					</span>
				</div>
				<button
					type="button"
					class="toggle"
					class:active={$draft.approval_mode}
					onclick={() =>
						updateDraft('approval_mode', !$draft.approval_mode)}
					role="switch"
					aria-checked={$draft.approval_mode}
					aria-label="Toggle approval mode"
				>
					<span class="toggle-track">
						<span class="toggle-thumb"></span>
					</span>
				</button>
			</div>
		</div>
	</div>
</SettingsSection>
{/if}

<style>
	.field-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.text-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		outline: none;
		transition: border-color 0.15s;
		cursor: pointer;
		appearance: auto;
	}

	.text-input:focus {
		border-color: var(--color-accent);
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.toggle {
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.toggle-track {
		display: flex;
		align-items: center;
		width: 42px;
		height: 24px;
		padding: 2px;
		background: var(--color-border);
		border-radius: 12px;
		transition: background 0.2s;
	}

	.toggle.active .toggle-track {
		background: var(--color-accent);
	}

	.toggle-thumb {
		width: 20px;
		height: 20px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
	}

	.toggle.active .toggle-thumb {
		transform: translateX(18px);
	}
</style>
