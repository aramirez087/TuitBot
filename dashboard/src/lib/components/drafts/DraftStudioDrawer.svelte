<script lang="ts">
	import * as studio from '$lib/stores/draftStudio.svelte';
	import DraftRail from './DraftRail.svelte';

	interface Props {
		open: boolean;
		onSelect: (id: number) => void;
		onCreate: () => void;
		onDelete: (id: number) => void;
		onDuplicate: (id: number) => void;
		onRestore: (id: number) => void;
	}

	const { open, onSelect, onCreate, onDelete, onDuplicate, onRestore }: Props = $props();
</script>

<div class="drafts-drawer" class:open>
	<DraftRail
		drafts={studio.getCurrentTabDrafts()}
		selectedId={studio.getSelectedId()}
		tab={studio.getTab()}
		tabCounts={studio.getTabCounts()}
		loading={studio.isLoading()}
		searchQuery={studio.getSearchQuery()}
		sortBy={studio.getSortBy()}
		tagFilter={studio.getTagFilter()}
		accountTags={studio.getAccountTags()}
		onselect={onSelect}
		ontabchange={(t) => studio.setTab(t)}
		oncreate={onCreate}
		ondelete={onDelete}
		onduplicate={onDuplicate}
		onrestore={onRestore}
		onsearch={(q) => studio.setSearchQuery(q)}
		onsort={(by) =>
			studio.setSortBy(by as 'updated' | 'created' | 'title' | 'scheduled')}
		ontagfilter={(id) => studio.setTagFilter(id)}
	/>
</div>

<style>
	.drafts-drawer {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 280px;
		z-index: 20;
		transform: translateX(-100%);
		transition: transform 0.22s cubic-bezier(0.4, 0, 0.2, 1);
		box-shadow: 4px 0 24px rgba(0, 0, 0, 0.18);
		overflow: hidden;
	}

	.drafts-drawer.open {
		transform: translateX(0);
	}
</style>
