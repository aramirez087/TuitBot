<script lang="ts">
	import {
		List,
		Clock,
		CheckCircle,
		XCircle,
		AlertCircle,
		MessageSquare,
		FileText,
		BookOpen
	} from 'lucide-svelte';

	interface Props {
		selectedStatus: string;
		selectedType: string;
		reviewerFilter: string;
		dateFilter: string;
		onStatusChange: (status: string) => void;
		onTypeChange: (type: string) => void;
		onReviewerChange: (reviewer: string) => void;
		onDateChange: (range: string) => void;
	}

	let {
		selectedStatus,
		selectedType,
		reviewerFilter,
		dateFilter,
		onStatusChange,
		onTypeChange,
		onReviewerChange,
		onDateChange
	}: Props = $props();

	const dateRanges = [
		{ value: 'all', label: 'All Time' },
		{ value: '24h', label: '24h' },
		{ value: '7d', label: '7d' },
		{ value: '30d', label: '30d' }
	] as const;

	const statusFilters = [
		{ value: 'all', label: 'All', icon: List },
		{ value: 'pending', label: 'Pending', icon: Clock },
		{ value: 'approved', label: 'Approved', icon: CheckCircle },
		{ value: 'rejected', label: 'Rejected', icon: XCircle },
		{ value: 'failed', label: 'Failed', icon: AlertCircle }
	] as const;

	const typeFilters = [
		{ value: 'all', label: 'All Types', icon: List },
		{ value: 'reply', label: 'Replies', icon: MessageSquare },
		{ value: 'tweet', label: 'Tweets', icon: FileText },
		{ value: 'thread_tweet', label: 'Threads', icon: BookOpen }
	] as const;
</script>

<div class="filters">
	<div class="filter-row">
		<span class="filter-label">Status</span>
		<div class="filter-chips">
			{#each statusFilters as filter}
				{@const Icon = filter.icon}
				<button
					class="chip"
					class:active={selectedStatus === filter.value}
					onclick={() => onStatusChange(filter.value)}
				>
					<Icon size={14} />
					<span>{filter.label}</span>
				</button>
			{/each}
		</div>
	</div>

	<div class="filter-row">
		<span class="filter-label">Type</span>
		<div class="filter-chips">
			{#each typeFilters as filter}
				{@const Icon = filter.icon}
				<button
					class="chip"
					class:active={selectedType === filter.value}
					onclick={() => onTypeChange(filter.value)}
				>
					<Icon size={14} />
					<span>{filter.label}</span>
				</button>
			{/each}
		</div>
	</div>

	<div class="filter-row">
		<span class="filter-label">Period</span>
		<div class="filter-chips">
			{#each dateRanges as range}
				<button
					class="chip"
					class:active={dateFilter === range.value}
					onclick={() => onDateChange(range.value)}
				>
					<span>{range.label}</span>
				</button>
			{/each}
		</div>
	</div>

	<div class="filter-row">
		<span class="filter-label">Reviewer</span>
		<input
			type="text"
			class="reviewer-input"
			placeholder="Filter by reviewer..."
			value={reviewerFilter}
			oninput={(e) => onReviewerChange((e.target as HTMLInputElement).value)}
		/>
	</div>
</div>

<style>
	.filters {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.filter-row {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.filter-label {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		min-width: 48px;
	}

	.filter-chips {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.chip {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background-color: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.chip:hover {
		background-color: var(--color-surface-hover);
		color: var(--color-text);
	}

	.chip.active {
		background-color: var(--color-accent);
		border-color: var(--color-accent);
		color: white;
	}

	.reviewer-input {
		padding: 6px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background-color: var(--color-surface);
		color: var(--color-text);
		font-size: 13px;
		width: 200px;
		outline: none;
		transition: border-color 0.15s ease;
	}

	.reviewer-input:focus {
		border-color: var(--color-accent);
	}

	.reviewer-input::placeholder {
		color: var(--color-text-subtle);
	}
</style>
