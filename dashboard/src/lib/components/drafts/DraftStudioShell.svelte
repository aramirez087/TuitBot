<script lang="ts">
	import { page } from "$app/stores";
	import { onMount } from "svelte";
	import { ACCOUNT_SWITCHED_EVENT } from "$lib/stores/accounts";
	import * as studio from "$lib/stores/draftStudio.svelte";
	import { api, type ThreadBlock } from "$lib/api";
	import type { ScheduledContentItem } from "$lib/api/types";
	import type { SyncStatus } from "$lib/utils/composerAutosave";
	import type { AttachedMedia } from "$lib/components/composer/TweetEditor.svelte";
	import type { ComposeRequest } from "$lib/api";
	import type { PaletteAction } from "$lib/components/CommandPalette.svelte";
	import { matchEvent } from "$lib/utils/shortcuts";
	import { Plus, Copy, Trash2, Files } from "lucide-svelte";
	import DraftRail from "./DraftRail.svelte";
	import DraftEmptyState from "./DraftEmptyState.svelte";
	import DraftSyncBadge from "./DraftSyncBadge.svelte";
	import DraftDetailsPanel from "./DraftDetailsPanel.svelte";
	import DraftHistoryPanel from "./DraftHistoryPanel.svelte";
	import ComposeWorkspace from "$lib/components/composer/ComposeWorkspace.svelte";

	let loadingDraft = $state(false);
	let syncStatus = $state<SyncStatus>("saved");
	let conflictDraftId = $state<number | null>(null);
	let railComponent: DraftRail | undefined = $state();
	let composerZoneEl: HTMLDivElement | undefined = $state();
	let detailsPanelOpen = $state(false);
	let activePanel = $state<"details" | "history">("details");
	let prefillSchedule = $state<string | null>(null);
	let drawerOpen = $state(false);

	interface HydrationPayload {
		mode: "tweet" | "thread";
		tweetText: string;
		threadBlocks: ThreadBlock[];
		attachedMedia: AttachedMedia[];
		updatedAt: string;
	}

	let hydration = $state<HydrationPayload | null>(null);
	let hydrationDraftId = $state<number | null>(null);

	const draftStudioPaletteActions: PaletteAction[] = [
		{
			id: "ds-new-draft",
			label: "New draft",
			icon: Plus,
			category: "DraftStudio",
			shortcut: "n",
			when: "always",
		},
		{
			id: "ds-duplicate",
			label: "Duplicate current draft",
			icon: Copy,
			category: "DraftStudio",
			shortcut: "d",
			when: "always",
		},
		{
			id: "ds-delete",
			label: "Delete current draft",
			icon: Trash2,
			category: "DraftStudio",
			shortcut: "backspace",
			when: "always",
		},
	];

	onMount(() => {
		studio.initFromUrl($page.url);
		studio.loadDrafts();
		studio.loadTags();

		// Handle ?new=true param (from Cmd+N or external redirect)
		if ($page.url.searchParams.get("new") === "true") {
			const url = new URL(window.location.href);
			url.searchParams.delete("new");
			history.replaceState(null, "", url.toString());
			studio.createDraft().then((newId) => {
				if (newId !== null) {
					console.info("[draft-studio]", {
						event: "draft_created",
						id: newId,
						source: "cmd-n",
					});
				}
			});
		}

		// Handle ?prefill_schedule param (from calendar time-slot clicks)
		const prefillParam = $page.url.searchParams.get("prefill_schedule");
		if (prefillParam) {
			const parsed = new Date(prefillParam);
			if (!isNaN(parsed.getTime())) {
				prefillSchedule = prefillParam;
			}
			const url = new URL(window.location.href);
			url.searchParams.delete("prefill_schedule");
			history.replaceState(null, "", url.toString());
		}

		const handler = () => {
			studio.reset();
			hydration = null;
			hydrationDraftId = null;
			studio.loadDrafts();
			studio.loadTags();
		};
		window.addEventListener(ACCOUNT_SWITCHED_EVENT, handler);
		return () =>
			window.removeEventListener(ACCOUNT_SWITCHED_EVENT, handler);
	});

	$effect(() => {
		const id = studio.getSelectedId();
		if (id === null) {
			hydration = null;
			hydrationDraftId = null;
			studio.setFullDraft(null);
			return;
		}
		if (id === hydrationDraftId) return;

		loadingDraft = true;
		syncStatus = "saved";
		conflictDraftId = null;
		fetchDraft(id);
	});

	// Load tags for selected draft when selection changes
	$effect(() => {
		void studio.getSelectedId();
		studio.loadSelectedDraftTags();
	});

	// Reload history when draft changes while history panel is open
	$effect(() => {
		void studio.getSelectedId();
		if (activePanel === "history" && detailsPanelOpen) {
			studio.loadRevisions();
			studio.loadActivity();
		}
	});

	async function fetchDraft(id: number) {
		try {
			const draft = await api.draftStudio.get(id);
			if (studio.getSelectedId() !== id) return;
			studio.setFullDraft(draft);
			console.info("[draft-studio]", {
				event: "draft_selected",
				id,
				source: "fetch",
			});
			hydration = parseServerDraft(draft);
			hydrationDraftId = id;
		} catch {
			if (studio.getSelectedId() !== id) return;
			studio.setFullDraft(null);
			hydration = null;
			hydrationDraftId = null;
		} finally {
			if (studio.getSelectedId() === id) loadingDraft = false;
		}
	}

	function parseServerDraft(draft: ScheduledContentItem): HydrationPayload {
		if (draft.content_type === "thread") {
			let texts: string[] = [];
			try {
				const parsed = JSON.parse(draft.content || "[]");
				texts = Array.isArray(parsed)
					? parsed.filter((t): t is string => typeof t === "string")
					: [];
			} catch {
				texts = draft.content ? [draft.content] : [];
			}
			return {
				mode: "thread",
				tweetText: "",
				threadBlocks:
					texts.length > 0
						? texts.map((text, i) => ({
								id: crypto.randomUUID(),
								text,
								media_paths: [],
								order: i,
							}))
						: [
								{
									id: crypto.randomUUID(),
									text: "",
									media_paths: [],
									order: 0,
								},
								{
									id: crypto.randomUUID(),
									text: "",
									media_paths: [],
									order: 1,
								},
							],
				attachedMedia: [],
				updatedAt: draft.updated_at,
			};
		}
		return {
			mode: "tweet",
			tweetText: draft.content || "",
			threadBlocks: [],
			attachedMedia: [],
			updatedAt: draft.updated_at,
		};
	}

	async function handleCreate() {
		const newId = await studio.createDraft();
		if (newId !== null) {
			console.info("[draft-studio]", {
				event: "draft_created",
				id: newId,
				source: "rail-button",
			});
			drawerOpen = false;
		}
	}

	async function handleDelete(id: number) {
		await studio.deleteDraft(id);
	}

	function handleDuplicate(id: number) {
		studio.duplicateDraft(id);
	}

	function handleRestore(id: number) {
		studio.restoreDraft(id);
	}

	function handleSyncStatus(status: SyncStatus) {
		syncStatus = status;
		if (status === "conflict") {
			conflictDraftId = studio.getSelectedId();
		}
		if (status === "offline") {
			console.info("[draft-studio]", {
				event: "save_failed",
				id: studio.getSelectedId(),
				syncStatus: status,
			});
		}
	}

	async function handleConflictResolution(
		resolution: "use-mine" | "reload-server",
	) {
		const id = studio.getSelectedId();
		if (id === null) return;

		if (resolution === "reload-server") {
			hydration = null;
			hydrationDraftId = null;
			loadingDraft = true;
			syncStatus = "saved";
			conflictDraftId = null;
			await fetchDraft(id);
		}
		if (resolution === "use-mine") {
			try {
				const draft = await api.draftStudio.get(id);
				if (studio.getSelectedId() !== id) return;
				if (hydration) {
					hydration = { ...hydration, updatedAt: draft.updated_at };
					syncStatus = "unsaved";
					conflictDraftId = null;
				}
			} catch {
				syncStatus = "offline";
			}
		}
	}

	async function handleDraftSubmit(data: ComposeRequest) {
		const id = studio.getSelectedId();
		if (id === null) return;
		if (data.scheduled_for) {
			const success = await studio.scheduleDraft(id, data.scheduled_for);
			if (success) await fetchDraft(id);
		}
	}

	async function handleSchedule(scheduledFor: string) {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.scheduleDraft(id, scheduledFor);
		if (success) {
			console.info("[draft-studio]", {
				event: "transition",
				id,
				from: "draft",
				to: "scheduled",
			});
			await fetchDraft(id);
		}
	}

	async function handleUnschedule() {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.unscheduleDraft(id);
		if (success) {
			console.info("[draft-studio]", {
				event: "transition",
				id,
				from: "scheduled",
				to: "draft",
			});
			await fetchDraft(id);
		}
	}

	async function handleReschedule(scheduledFor: string) {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.rescheduleDraft(id, scheduledFor);
		if (success) await fetchDraft(id);
	}

	function handleDuplicateFromDetails() {
		const id = studio.getSelectedId();
		if (id !== null) handleDuplicate(id);
	}

	async function handleRestoreFromRevision(revisionId: number) {
		const id = studio.getSelectedId();
		if (id === null) return;
		const success = await studio.restoreFromRevision(revisionId);
		if (success) {
			console.info("[draft-studio]", {
				event: "restore_executed",
				id,
				revisionId,
			});
			hydration = null;
			hydrationDraftId = null;
			loadingDraft = true;
			await fetchDraft(id);
		}
	}

	function switchToHistory() {
		activePanel = "history";
		detailsPanelOpen = true;
		studio.loadRevisions();
		studio.loadActivity();
	}

	function handleDraftAction(actionId: string) {
		const id = studio.getSelectedId();
		switch (actionId) {
			case "ds-new-draft":
				handleCreate();
				break;
			case "ds-duplicate":
				if (id !== null) handleDuplicate(id);
				break;
			case "ds-delete":
				if (id !== null) handleDelete(id);
				break;
		}
	}

	async function handleMetaUpdate(data: { title?: string; notes?: string }) {
		const id = studio.getSelectedId();
		if (id === null) return;
		await api.draftStudio.updateMeta(id, data);
		if (data.title !== undefined) {
			studio.updateDraftInCollection(id, { title: data.title || null });
		}
	}

	function handleAssignTag(tagId: number) {
		studio.assignTag(tagId);
	}

	function handleUnassignTag(tagId: number) {
		studio.unassignTag(tagId);
	}

	function handleCreateTag(name: string) {
		studio.createAndAssignTag(name);
	}

	function handleShellKeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			if (drawerOpen) {
				e.preventDefault();
				drawerOpen = false;
				return;
			}
			if (composerZoneEl?.contains(document.activeElement)) {
				e.preventDefault();
				e.stopPropagation();
			}
		}
		// Cmd+Shift+D toggles details panel
		if (matchEvent(e, "cmd+shift+d")) {
			e.preventDefault();
			activePanel = "details";
			detailsPanelOpen = !detailsPanelOpen;
		}
		// Cmd+Shift+H toggles history panel
		if (matchEvent(e, "cmd+shift+h")) {
			e.preventDefault();
			if (activePanel === "history" && detailsPanelOpen) {
				detailsPanelOpen = false;
			} else {
				switchToHistory();
			}
		}
		// Cmd+Shift+O opens/closes drafts drawer
		if (matchEvent(e, "cmd+shift+o")) {
			e.preventDefault();
			drawerOpen = !drawerOpen;
		}
	}

	function handleDraftSelect(id: number) {
		studio.selectDraft(id);
		drawerOpen = false;
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="studio-shell"
	class:details-open={detailsPanelOpen && studio.getSelectedId() !== null}
	onkeydown={handleShellKeydown}
>
	<!-- Drawer overlay backdrop -->
	{#if drawerOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="drawer-backdrop" onclick={() => (drawerOpen = false)}></div>
	{/if}

	<!-- Sliding Drafts Drawer -->
	<div class="drafts-drawer" class:open={drawerOpen}>
		<DraftRail
			bind:this={railComponent}
			drafts={studio.getCurrentTabDrafts()}
			selectedId={studio.getSelectedId()}
			tab={studio.getTab()}
			tabCounts={studio.getTabCounts()}
			loading={studio.isLoading()}
			searchQuery={studio.getSearchQuery()}
			sortBy={studio.getSortBy()}
			tagFilter={studio.getTagFilter()}
			accountTags={studio.getAccountTags()}
			onselect={handleDraftSelect}
			ontabchange={(t) => studio.setTab(t)}
			oncreate={handleCreate}
			ondelete={handleDelete}
			onduplicate={handleDuplicate}
			onrestore={handleRestore}
			onsearch={(q) => studio.setSearchQuery(q)}
			onsort={(by) =>
				studio.setSortBy(
					by as "updated" | "created" | "title" | "scheduled",
				)}
			ontagfilter={(id) => studio.setTagFilter(id)}
		/>
	</div>

	<!-- Main Composer Area -->
	<div class="composer-zone" bind:this={composerZoneEl}>
		<!-- Composer header bar -->
		<div class="composer-bar">
			<button
				class="drafts-toggle-btn"
				type="button"
				onclick={() => (drawerOpen = !drawerOpen)}
				title="Toggle drafts (⌘⇧O)"
				class:active={drawerOpen}
			>
				<Files size={15} />
				<span>Drafts</span>
				{#if studio.getTabCounts().active > 0}
					<span class="draft-count"
						>{studio.getTabCounts().active}</span
					>
				{/if}
			</button>

			{#if studio.getSelectedId() !== null}
				<DraftSyncBadge
					status={syncStatus}
					onresolveconflict={handleConflictResolution}
				/>
			{/if}

			<div class="bar-spacer"></div>

			{#if studio.getSelectedId() !== null}
				<button
					class="bar-action"
					type="button"
					onclick={() => studio.createDraft()}
					title="New draft"
				>
					<Plus size={14} />
					<span>New</span>
				</button>
			{/if}
		</div>

		{#if studio.getError()}
			<div class="error-banner">
				<span>{studio.getError()}</span>
				<button type="button" onclick={() => studio.clearError()}
					>Dismiss</button
				>
			</div>
		{/if}

		{#if studio.isLoading() && !studio.getSelectedDraft()}
			<div class="zone-loading">
				<div class="zone-spinner"></div>
			</div>
		{:else if studio.getSelectedId() !== null}
			{#if loadingDraft}
				<div class="zone-loading">
					<div class="zone-spinner"></div>
				</div>
			{:else if hydration && hydrationDraftId !== null}
				{#key hydrationDraftId}
					<ComposeWorkspace
						draftId={hydrationDraftId}
						initialContent={hydration}
						embedded={true}
						schedule={null}
						canPublish={false}
						onsubmit={handleDraftSubmit}
						onsyncstatus={handleSyncStatus}
						extraPaletteActions={draftStudioPaletteActions}
						ondraftaction={handleDraftAction}
					/>
				{/key}
			{:else}
				<div class="zone-error">
					<p>Failed to load draft content.</p>
					<button
						type="button"
						onclick={() => {
							if (studio.getSelectedId() !== null)
								fetchDraft(studio.getSelectedId()!);
						}}
					>
						Retry
					</button>
				</div>
			{/if}
		{:else if studio.getTabCounts().active === 0 && studio.getTabCounts().scheduled === 0}
			<DraftEmptyState variant="no-drafts" oncreate={handleCreate} />
		{:else}
			<DraftEmptyState variant="no-selection" oncreate={handleCreate} />
		{/if}
	</div>

	{#if detailsPanelOpen && studio.getSelectedId() !== null}
		<div class="details-zone">
			<div class="panel-switcher">
				<button
					type="button"
					class="panel-tab"
					class:active={activePanel === "details"}
					onclick={() => (activePanel = "details")}>Details</button
				>
				<button
					type="button"
					class="panel-tab"
					class:active={activePanel === "history"}
					onclick={switchToHistory}>History</button
				>
			</div>
			{#if activePanel === "details"}
				<DraftDetailsPanel
					draft={studio.getFullDraft()}
					draftSummary={studio.getSelectedDraft()}
					tags={studio.getSelectedDraftTags()}
					allTags={studio.getAccountTags()}
					{prefillSchedule}
					onupdatemeta={handleMetaUpdate}
					onassigntag={handleAssignTag}
					onunassigntag={handleUnassignTag}
					oncreatetag={handleCreateTag}
					onschedule={handleSchedule}
					onunschedule={handleUnschedule}
					onreschedule={handleReschedule}
					onduplicate={handleDuplicateFromDetails}
					onclose={() => (detailsPanelOpen = false)}
				/>
			{:else}
				<DraftHistoryPanel
					revisions={studio.getRevisions()}
					activity={studio.getActivity()}
					onrestore={handleRestoreFromRevision}
					onclose={() => (detailsPanelOpen = false)}
				/>
			{/if}
		</div>
	{/if}
</div>

<style>
	.studio-shell {
		display: grid;
		grid-template-columns: 1fr;
		height: calc(100vh - 48px);
		margin: -24px -32px;
		overflow: hidden;
		position: relative;
	}

	.studio-shell.details-open {
		grid-template-columns: 1fr 280px;
	}

	/* === Drawer === */
	.drawer-backdrop {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.25);
		z-index: 10;
		backdrop-filter: blur(1px);
	}

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

	/* === Composer Zone === */
	.composer-zone {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow-y: auto;
		background: var(--color-base);
		grid-column: 1;
	}

	/* === Composer Header Bar === */
	.composer-bar {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 16px;
		border-bottom: 1px solid var(--color-border-subtle);
		background: var(--color-surface);
		flex-shrink: 0;
	}

	.drafts-toggle-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 5px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.drafts-toggle-btn:hover,
	.drafts-toggle-btn.active {
		background: var(--color-surface-active);
		color: var(--color-text);
		border-color: var(--color-border);
	}

	.draft-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 16px;
		height: 16px;
		padding: 0 4px;
		border-radius: 8px;
		background: var(--color-accent);
		color: #fff;
		font-size: 10px;
		font-weight: 700;
		line-height: 1;
	}

	.bar-spacer {
		flex: 1;
	}

	.bar-action {
		display: flex;
		align-items: center;
		gap: 5px;
		padding: 5px 10px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text-muted);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
	}

	.bar-action:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	/* === Details Zone === */
	.details-zone {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow: hidden;
		border-left: 1px solid var(--color-border-subtle);
		background: var(--color-surface);
	}

	.panel-switcher {
		display: flex;
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
		background: var(--color-surface);
	}

	.panel-tab {
		flex: 1;
		padding: 8px 0;
		border: none;
		border-bottom: 2px solid transparent;
		background: transparent;
		color: var(--color-text-subtle);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s;
	}

	.panel-tab:hover {
		color: var(--color-text);
	}

	.panel-tab.active {
		color: var(--color-accent);
		border-bottom-color: var(--color-accent);
	}

	/* === Loading / Error states === */
	.zone-loading {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	.zone-spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--color-border-subtle);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	.zone-error {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: 12px;
		color: var(--color-text-subtle);
		font-size: 14px;
	}

	.zone-error button {
		padding: 6px 16px;
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		background: transparent;
		color: var(--color-text);
		font-size: 13px;
		cursor: pointer;
	}

	.zone-error button:hover {
		background: var(--color-surface);
	}

	.error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
		color: var(--color-danger);
		font-size: 13px;
		border-bottom: 1px solid
			color-mix(in srgb, var(--color-danger) 20%, transparent);
		flex-shrink: 0;
	}

	.error-banner button {
		border: none;
		background: transparent;
		color: var(--color-danger);
		font-size: 12px;
		font-weight: 500;
		cursor: pointer;
		text-decoration: underline;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
