<script lang="ts">
	import type { CalendarItem } from '$lib/api';
	import { FileText, GitBranch, MessageSquare } from 'lucide-svelte';

	let {
		items,
		days,
		currentDate,
		ondayclick
	}: {
		items: CalendarItem[];
		days: Date[];
		currentDate: Date;
		ondayclick?: (date: Date) => void;
	} = $props();

	const dayLabels = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

	const currentMonth = $derived(currentDate.getMonth());

	function dateKey(d: Date): string {
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	}

	const todayKey = $derived(() => {
		const now = new Date();
		return dateKey(now);
	});

	function isToday(d: Date): boolean {
		return dateKey(d) === todayKey();
	}

	function isCurrentMonth(d: Date): boolean {
		return d.getMonth() === currentMonth;
	}

	/** Group items by date. */
	const itemsByDate = $derived(() => {
		const map = new Map<string, CalendarItem[]>();
		for (const item of items) {
			const d = new Date(item.timestamp);
			const key = dateKey(d);
			if (!map.has(key)) map.set(key, []);
			map.get(key)!.push(item);
		}
		return map;
	});

	interface DayStats {
		total: number;
		tweets: number;
		threads: number;
		replies: number;
	}

	function getDayStats(d: Date): DayStats {
		const dayItems = itemsByDate().get(dateKey(d)) ?? [];
		return {
			total: dayItems.length,
			tweets: dayItems.filter((i) => i.content_type === 'tweet').length,
			threads: dayItems.filter((i) => i.content_type === 'thread').length,
			replies: dayItems.filter((i) => i.content_type === 'reply').length
		};
	}

	function handleDayClick(d: Date) {
		if (ondayclick) {
			ondayclick(d);
		}
	}
</script>

<div class="month-view">
	<div class="month-header">
		{#each dayLabels as label}
			<div class="month-day-label">{label}</div>
		{/each}
	</div>

	<div class="month-grid">
		{#each days as day}
			{@const stats = getDayStats(day)}
			<button
				class="month-cell"
				class:today={isToday(day)}
				class:other-month={!isCurrentMonth(day)}
				class:has-items={stats.total > 0}
				onclick={() => handleDayClick(day)}
			>
				<span class="cell-date">{day.getDate()}</span>

				{#if stats.total > 0}
					<div class="cell-dots">
						{#if stats.tweets > 0}
							<span class="dot tweet" title="{stats.tweets} tweet{stats.tweets !== 1 ? 's' : ''}">
								<FileText size={8} />
								{#if stats.tweets > 1}
									<span class="dot-count">{stats.tweets}</span>
								{/if}
							</span>
						{/if}
						{#if stats.threads > 0}
							<span class="dot thread" title="{stats.threads} thread{stats.threads !== 1 ? 's' : ''}">
								<GitBranch size={8} />
								{#if stats.threads > 1}
									<span class="dot-count">{stats.threads}</span>
								{/if}
							</span>
						{/if}
						{#if stats.replies > 0}
							<span class="dot reply" title="{stats.replies} repl{stats.replies !== 1 ? 'ies' : 'y'}">
								<MessageSquare size={8} />
								{#if stats.replies > 1}
									<span class="dot-count">{stats.replies}</span>
								{/if}
							</span>
						{/if}
					</div>
				{/if}
			</button>
		{/each}
	</div>
</div>

<style>
	.month-view {
		border: 1px solid var(--color-border);
		border-radius: 8px;
		overflow: hidden;
	}

	.month-header {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border);
	}

	.month-day-label {
		text-align: center;
		padding: 8px 4px;
		font-size: 10px;
		font-weight: 500;
		color: var(--color-text-subtle);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.month-grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
	}

	.month-cell {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 8px 4px;
		min-height: 64px;
		border: none;
		border-right: 1px solid var(--color-border-subtle);
		border-bottom: 1px solid var(--color-border-subtle);
		background: transparent;
		cursor: pointer;
		transition: background 0.15s ease;
		color: var(--color-text);
	}

	.month-cell:nth-child(7n) {
		border-right: none;
	}

	.month-cell:hover {
		background: var(--color-surface-hover);
	}

	.month-cell.today {
		background: color-mix(in srgb, var(--color-accent) 5%, transparent);
	}

	.month-cell.today:hover {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
	}

	.month-cell.other-month {
		opacity: 0.35;
	}

	.cell-date {
		font-size: 13px;
		font-weight: 500;
	}

	.month-cell.today .cell-date {
		color: var(--color-accent);
		font-weight: 700;
	}

	.cell-dots {
		display: flex;
		gap: 4px;
		flex-wrap: wrap;
		justify-content: center;
	}

	.dot {
		display: flex;
		align-items: center;
		gap: 1px;
		padding: 1px 3px;
		border-radius: 6px;
		font-size: 9px;
		font-weight: 500;
	}

	.dot.tweet {
		color: var(--color-accent);
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
	}

	.dot.thread {
		color: #a371f7;
		background: color-mix(in srgb, #a371f7 15%, transparent);
	}

	.dot.reply {
		color: var(--color-success);
		background: color-mix(in srgb, var(--color-success) 15%, transparent);
	}

	.dot-count {
		font-family: var(--font-mono);
	}
</style>
