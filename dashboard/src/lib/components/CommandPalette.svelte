<script lang="ts">
	import { formatCombo, SHORTCUT_CATALOG } from '$lib/utils/shortcuts';
	import { focusTrap } from '$lib/actions/focusTrap';
	import {
		Maximize2,
		MessageSquare,
		List,
		Send,
		Sparkles,
		FileText,
		Plus,
		Copy,
		Scissors,
		Merge,
		ArrowUp,
		ArrowDown,
		Image,
		Search,
		PanelRight
	} from 'lucide-svelte';
	let {
		open,
		mode,
		onclose,
		onaction
	}: {
		open: boolean;
		mode: 'tweet' | 'thread';
		onclose: () => void;
		onaction: (actionId: string) => void;
	} = $props();

	interface PaletteAction {
		id: string;
		label: string;
		icon: typeof Maximize2;
		category: 'Mode' | 'Compose' | 'AI' | 'Thread';
		shortcut?: string;
		when?: 'thread' | 'tweet' | 'always';
	}

	const allActions: PaletteAction[] = [
		{ id: 'focus-mode', label: 'Toggle focus mode', icon: Maximize2, category: 'Mode', shortcut: 'cmd+shift+f', when: 'always' },
		{ id: 'toggle-inspector', label: 'Toggle inspector', icon: PanelRight, category: 'Mode', shortcut: 'cmd+i', when: 'always' },
		{ id: 'mode-tweet', label: 'Switch to Tweet', icon: MessageSquare, category: 'Mode', shortcut: 'cmd+shift+n', when: 'always' },
		{ id: 'mode-thread', label: 'Switch to Thread', icon: List, category: 'Mode', shortcut: 'cmd+shift+t', when: 'always' },
		{ id: 'submit', label: 'Submit / Post now', icon: Send, category: 'Compose', shortcut: 'cmd+enter', when: 'always' },
		{ id: 'ai-improve', label: 'AI Improve', icon: Sparkles, category: 'AI', shortcut: 'cmd+j', when: 'always' },
		{ id: 'ai-generate', label: 'AI Generate / Improve', icon: Sparkles, category: 'AI', when: 'always' },
		{ id: 'ai-from-notes', label: 'Generate from notes', icon: FileText, category: 'AI', when: 'always' },
		{ id: 'attach-media', label: 'Attach media', icon: Image, category: 'Compose', when: 'always' },
		{ id: 'add-card', label: 'Add tweet card', icon: Plus, category: 'Thread', when: 'thread' },
		{ id: 'duplicate', label: 'Duplicate card', icon: Copy, category: 'Thread', shortcut: 'cmd+d', when: 'thread' },
		{ id: 'split', label: 'Split at cursor', icon: Scissors, category: 'Thread', shortcut: 'cmd+shift+s', when: 'thread' },
		{ id: 'merge', label: 'Merge with next', icon: Merge, category: 'Thread', shortcut: 'cmd+shift+m', when: 'thread' },
		{ id: 'move-up', label: 'Move card up', icon: ArrowUp, category: 'Thread', shortcut: 'alt+arrowup', when: 'thread' },
		{ id: 'move-down', label: 'Move card down', icon: ArrowDown, category: 'Thread', shortcut: 'alt+arrowdown', when: 'thread' }
	];

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement | undefined = $state();

	const filteredActions = $derived.by(() => {
		const visible = allActions.filter((a) => {
			if (a.when === 'thread' && mode !== 'thread') return false;
			if (a.when === 'tweet' && mode !== 'tweet') return false;
			return true;
		});
		if (!query.trim()) return visible;
		const q = query.toLowerCase();
		return visible.filter(
			(a) => a.label.toLowerCase().includes(q) || a.category.toLowerCase().includes(q)
		);
	});

	// Group by category for display
	const categoryOrder: PaletteAction['category'][] = ['Mode', 'Compose', 'AI', 'Thread'];

	const groupedActions = $derived.by(() => {
		const groups: { category: string; actions: PaletteAction[] }[] = [];
		for (const cat of categoryOrder) {
			const items = filteredActions.filter((a) => a.category === cat);
			if (items.length > 0) groups.push({ category: cat, actions: items });
		}
		return groups;
	});

	// Flat list for keyboard navigation indexing
	const flatActions = $derived(groupedActions.flatMap((g) => g.actions));

	$effect(() => {
		if (open && inputEl) {
			inputEl.focus();
			query = '';
			selectedIndex = 0;
		}
	});

	// Reset selectedIndex when query changes
	$effect(() => {
		void query;
		selectedIndex = 0;
	});

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			onclose();
			return;
		}
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (flatActions.length > 0) {
				selectedIndex = (selectedIndex + 1) % flatActions.length;
			}
			return;
		}
		if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (flatActions.length > 0) {
				selectedIndex = (selectedIndex - 1 + flatActions.length) % flatActions.length;
			}
			return;
		}
		if (e.key === 'Enter') {
			e.preventDefault();
			if (flatActions[selectedIndex]) {
				onaction(flatActions[selectedIndex].id);
			}
			return;
		}
	}

	function executeAction(actionId: string) {
		onaction(actionId);
	}

	function getActionDomId(action: PaletteAction): string {
		return `palette-action-${action.id}`;
	}

	const activeDescendant = $derived(
		flatActions[selectedIndex] ? getActionDomId(flatActions[selectedIndex]) : undefined
	);
</script>

{#if open}
	<div
		class="palette-backdrop"
		onclick={(e) => { if (e.target === e.currentTarget) onclose(); }}
		onkeydown={handleKeydown}
		role="presentation"
	>
		<div
			class="palette-panel"
			role="dialog"
			aria-modal="true"
			aria-label="Command palette"
			use:focusTrap
		>
			<div class="palette-search">
				<Search size={14} />
				<input
					bind:this={inputEl}
					type="text"
					class="palette-input"
					placeholder="Type a command..."
					bind:value={query}
					role="combobox"
					aria-expanded="true"
					aria-controls="palette-listbox"
					aria-activedescendant={activeDescendant}
					aria-autocomplete="list"
				/>
			</div>
			<div class="palette-results" id="palette-listbox" role="listbox" aria-label="Commands">
				{#if flatActions.length === 0}
					<div class="palette-empty">No matching commands</div>
				{:else}
					{#each groupedActions as group}
						<div class="palette-category-label">{group.category}</div>
						{#each group.actions as action}
							{@const globalIdx = flatActions.indexOf(action)}
							{@const ActionIcon = action.icon}
							<div
								id={getActionDomId(action)}
								class="palette-item"
								class:selected={globalIdx === selectedIndex}
								role="option"
								tabindex={-1}
								aria-selected={globalIdx === selectedIndex}
								onclick={() => executeAction(action.id)}
								onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); executeAction(action.id); } }}
								onmouseenter={() => { selectedIndex = globalIdx; }}
							>
								<span class="palette-item-icon">
									<ActionIcon size={14} />
								</span>
								<span class="palette-item-label">{action.label}</span>
								{#if action.shortcut}
									<span class="palette-item-shortcut">{formatCombo(action.shortcut)}</span>
								{/if}
							</div>
						{/each}
					{/each}
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.palette-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.3);
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 60px;
		z-index: 10;
		border-radius: inherit;
	}

	.palette-panel {
		width: 420px;
		max-width: 90%;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 10px;
		box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
		overflow: hidden;
	}

	.palette-search {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 14px;
		border-bottom: 1px solid var(--color-border-subtle);
		color: var(--color-text-muted);
	}

	.palette-input {
		flex: 1;
		border: none;
		background: transparent;
		color: var(--color-text);
		font-size: 14px;
		font-family: var(--font-sans);
		outline: none;
	}

	.palette-input::placeholder {
		color: var(--color-text-subtle);
	}

	.palette-results {
		max-height: 320px;
		overflow-y: auto;
		padding: 6px 0;
	}

	.palette-empty {
		padding: 20px 14px;
		text-align: center;
		font-size: 13px;
		color: var(--color-text-subtle);
	}

	.palette-category-label {
		padding: 6px 14px 4px;
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--color-text-subtle);
	}

	.palette-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 14px;
		cursor: pointer;
		transition: background 0.1s ease;
	}

	.palette-item.selected {
		background: var(--color-surface-hover);
	}

	.palette-item-icon {
		display: flex;
		align-items: center;
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.palette-item-label {
		flex: 1;
		font-size: 13px;
		color: var(--color-text);
	}

	.palette-item-shortcut {
		font-size: 11px;
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
		padding: 1px 6px;
		border-radius: 4px;
		background: var(--color-base);
		border: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	/* Mobile layout */
	@media (max-width: 640px) {
		.palette-backdrop {
			padding-top: 20px;
		}

		.palette-panel {
			width: 100%;
			max-width: 100%;
			border-radius: 0;
			max-height: calc(100vh - 40px);
		}

		.palette-item {
			padding: 12px 14px;
			min-height: 44px;
		}
	}

	/* Touch targets */
	@media (pointer: coarse) {
		.palette-item {
			min-height: 44px;
		}
	}
</style>
