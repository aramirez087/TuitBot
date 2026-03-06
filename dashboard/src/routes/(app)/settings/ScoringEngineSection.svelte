<script lang="ts">
	import { Target } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import SliderInput from '$lib/components/settings/SliderInput.svelte';
	import { defaults, draft, scoringTotal, updateDraft } from '$lib/stores/settings';

	const scoringSignals = $derived(
		$draft
			? [
					{
						key: 'keyword_relevance_max',
						label: 'Keywords',
						value: $draft.scoring.keyword_relevance_max,
						color: '#58a6ff'
					},
					{
						key: 'follower_count_max',
						label: 'Followers',
						value: $draft.scoring.follower_count_max,
						color: '#3fb950'
					},
					{
						key: 'recency_max',
						label: 'Recency',
						value: $draft.scoring.recency_max,
						color: '#d29922'
					},
					{
						key: 'engagement_rate_max',
						label: 'Engagement',
						value: $draft.scoring.engagement_rate_max,
						color: '#f78166'
					},
					{
						key: 'reply_count_max',
						label: 'Reply count',
						value: $draft.scoring.reply_count_max,
						color: '#bc8cff'
					},
					{
						key: 'content_type_max',
						label: 'Content type',
						value: $draft.scoring.content_type_max,
						color: '#f85149'
					}
				]
			: []
	);
</script>

{#if $draft}
<SettingsSection
	id="scoring"
	title="Scoring Engine"
	description="Tune the 6-signal scoring system that decides which tweets to reply to"
	icon={Target}
	scope="account"
	scopeKey="scoring"
>
	<div class="field-grid">
		<div class="field full-width">
			<SliderInput
				value={$draft.scoring.threshold}
				label="Reply Threshold"
				min={0}
				max={100}
				unit=" pts"
				helpText="Minimum score needed for a tweet to trigger a reply"
				defaultValue={$defaults?.scoring.threshold}
				onchange={(v) => updateDraft('scoring.threshold', v)}
			/>
		</div>

		<div class="field full-width scoring-bar-section">
			<div class="scoring-bar-header">
				<span class="field-label">Signal Weights</span>
				<span class="scoring-total">
					Total max: <strong>{$scoringTotal.toFixed(1)}</strong> pts
				</span>
			</div>
			<div class="scoring-bar">
				{#each scoringSignals as signal}
					{#if signal.value > 0}
						<div
							class="scoring-segment"
							style="flex: {signal.value}; background: {signal.color}"
							title="{signal.label}: {signal.value}"
						></div>
					{/if}
				{/each}
			</div>
			<div class="scoring-legend">
				{#each scoringSignals as signal}
					<div class="legend-item">
						<span class="legend-dot" style="background: {signal.color}"></span>
						<span class="legend-label">{signal.label}</span>
						<span class="legend-value">{signal.value}</span>
					</div>
				{/each}
			</div>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.scoring.keyword_relevance_max}
				label="Keyword Relevance"
				min={0}
				max={50}
				step={0.5}
				unit=" pts"
				helpText="How well tweet matches your keywords"
				defaultValue={$defaults?.scoring.keyword_relevance_max}
				onchange={(v) =>
					updateDraft('scoring.keyword_relevance_max', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.scoring.follower_count_max}
				label="Follower Count"
				min={0}
				max={50}
				step={0.5}
				unit=" pts"
				helpText="Author follower count (bell curve scoring)"
				defaultValue={$defaults?.scoring.follower_count_max}
				onchange={(v) =>
					updateDraft('scoring.follower_count_max', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.scoring.recency_max}
				label="Recency"
				min={0}
				max={50}
				step={0.5}
				unit=" pts"
				helpText="How recently the tweet was posted"
				defaultValue={$defaults?.scoring.recency_max}
				onchange={(v) => updateDraft('scoring.recency_max', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.scoring.engagement_rate_max}
				label="Engagement Rate"
				min={0}
				max={50}
				step={0.5}
				unit=" pts"
				helpText="Ratio of engagement to followers"
				defaultValue={$defaults?.scoring.engagement_rate_max}
				onchange={(v) =>
					updateDraft('scoring.engagement_rate_max', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.scoring.reply_count_max}
				label="Reply Count"
				min={0}
				max={50}
				step={0.5}
				unit=" pts"
				helpText="Fewer existing replies = higher score"
				defaultValue={$defaults?.scoring.reply_count_max}
				onchange={(v) =>
					updateDraft('scoring.reply_count_max', v)}
			/>
		</div>

		<div class="field">
			<SliderInput
				value={$draft.scoring.content_type_max}
				label="Content Type"
				min={0}
				max={50}
				step={0.5}
				unit=" pts"
				helpText="Text-only originals score highest"
				defaultValue={$defaults?.scoring.content_type_max}
				onchange={(v) =>
					updateDraft('scoring.content_type_max', v)}
			/>
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

	.scoring-bar-section {
		gap: 10px;
	}

	.scoring-bar-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.scoring-total {
		font-size: 13px;
		color: var(--color-text-muted);
		font-family: var(--font-mono);
	}

	.scoring-total strong {
		color: var(--color-accent);
	}

	.scoring-bar {
		display: flex;
		height: 10px;
		border-radius: 5px;
		overflow: hidden;
		gap: 1px;
	}

	.scoring-segment {
		border-radius: 2px;
		transition: flex 0.3s;
	}

	.scoring-legend {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.legend-dot {
		width: 8px;
		height: 8px;
		border-radius: 2px;
		flex-shrink: 0;
	}

	.legend-value {
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
	}
</style>
