<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Briefcase,
		MessageCircle,
		Target,
		Shield,
		Clock,
		Brain,
		Key,
		Database,
		Wifi
	} from 'lucide-svelte';
	import {
		loading,
		error,
		draft,
		isDirty,
		saveError,
		loadSettings,
		resetDraft,
		saveSettings,
		hasDangerousChanges
	} from '$lib/stores/settings';

	import BusinessProfileSection from './BusinessProfileSection.svelte';
	import ContentPersonaSection from './ContentPersonaSection.svelte';
	import ScoringEngineSection from './ScoringEngineSection.svelte';
	import SafetyLimitsSection from './SafetyLimitsSection.svelte';
	import ScheduleSection from './ScheduleSection.svelte';
	import LlmProviderSection from './LlmProviderSection.svelte';
	import XApiSection from './XApiSection.svelte';
	import StorageSection from './StorageSection.svelte';
	import LanAccessSection from './LanAccessSection.svelte';
	import SaveBar from './SaveBar.svelte';
	import ConfirmModal from './ConfirmModal.svelte';

	// --- Section nav ---

	const sections = [
		{ id: 'business', label: 'Business', icon: Briefcase },
		{ id: 'persona', label: 'Persona', icon: MessageCircle },
		{ id: 'scoring', label: 'Scoring', icon: Target },
		{ id: 'limits', label: 'Limits', icon: Shield },
		{ id: 'schedule', label: 'Schedule', icon: Clock },
		{ id: 'llm', label: 'LLM', icon: Brain },
		{ id: 'xapi', label: 'X API', icon: Key },
		{ id: 'storage', label: 'Storage', icon: Database },
		{ id: 'lan', label: 'LAN', icon: Wifi }
	];

	let activeSection = $state('business');
	let showSaved = $state(false);
	let showConfirm = $state(false);
	let savedTimeout: ReturnType<typeof setTimeout> | null = null;

	// --- IntersectionObserver for section highlighting ---

	let observers: IntersectionObserver[] = [];

	function setupObservers() {
		observers.forEach((o) => o.disconnect());
		observers = [];

		for (const section of sections) {
			const el = document.getElementById(section.id);
			if (!el) continue;

			const observer = new IntersectionObserver(
				(entries) => {
					for (const entry of entries) {
						if (entry.isIntersecting) {
							activeSection = section.id;
						}
					}
				},
				{ rootMargin: '-80px 0px -60% 0px', threshold: 0 }
			);
			observer.observe(el);
			observers.push(observer);
		}
	}

	onMount(() => {
		loadSettings().then(() => {
			// Small delay so DOM is ready
			setTimeout(setupObservers, 100);
		});
	});

	onDestroy(() => {
		observers.forEach((o) => o.disconnect());
		if (savedTimeout) clearTimeout(savedTimeout);
	});

	function scrollToSection(id: string) {
		const el = document.getElementById(id);
		if (el) {
			el.scrollIntoView({ behavior: 'smooth', block: 'start' });
		}
	}

	// --- Save flow ---

	async function handleSave() {
		if (hasDangerousChanges()) {
			showConfirm = true;
			return;
		}
		await doSave();
	}

	async function doSave() {
		showConfirm = false;
		const ok = await saveSettings();
		if (ok) {
			showSaved = true;
			if (savedTimeout) clearTimeout(savedTimeout);
			savedTimeout = setTimeout(() => {
				showSaved = false;
			}, 3000);
		}
	}
</script>

<svelte:head>
	<title>Settings — Tuitbot</title>
</svelte:head>

{#if $loading}
	<div class="page-header">
		<h1>Settings</h1>
		<p class="subtitle">Configure automation behavior and preferences</p>
	</div>
	<div class="loading-skeleton">
		{#each Array(4) as _}
			<div class="skeleton-card"></div>
		{/each}
	</div>
{:else if $error}
	<div class="page-header">
		<h1>Settings</h1>
		<p class="subtitle">Configure automation behavior and preferences</p>
	</div>
	<div class="error-banner">
		<p>{$error}</p>
		<button onclick={() => loadSettings()}>Retry</button>
	</div>
{:else if $draft}
	<div class="settings-layout">
		<!-- Section nav -->
		<nav class="section-nav">
			<div class="nav-title">Settings</div>
			{#each sections as section}
				{@const Icon = section.icon}
				<button
					class="nav-item"
					class:active={activeSection === section.id}
					onclick={() => scrollToSection(section.id)}
				>
					<Icon size={15} />
					<span>{section.label}</span>
				</button>
			{/each}
		</nav>

		<!-- Content area -->
		<div class="settings-content">
			<div class="page-header">
				<h1>Settings</h1>
				<p class="subtitle">Configure automation behavior and preferences</p>
			</div>

			<div class="sections">
				<BusinessProfileSection />
				<ContentPersonaSection />
				<ScoringEngineSection />
				<SafetyLimitsSection />
				<ScheduleSection />
				<LlmProviderSection />
				<XApiSection />
				<StorageSection />
				<LanAccessSection />
			</div>
		</div>
	</div>

	<SaveBar {showSaved} onSave={handleSave} onDiscard={resetDraft} />

	{#if showConfirm}
		<ConfirmModal onConfirm={doSave} onCancel={() => (showConfirm = false)} />
	{/if}
{/if}

<style>
	/* ================================================================ */
	/* Layout */
	/* ================================================================ */

	.settings-layout {
		display: flex;
		gap: 24px;
	}

	.section-nav {
		width: 160px;
		flex-shrink: 0;
		position: sticky;
		top: 16px;
		align-self: flex-start;
	}

	.nav-title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		padding: 0 8px 8px;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 7px 8px;
		border: none;
		background: none;
		color: var(--color-text-muted);
		font-size: 13px;
		border-radius: 6px;
		cursor: pointer;
		text-align: left;
		transition:
			background 0.15s,
			color 0.15s;
	}

	.nav-item:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.nav-item.active {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}

	.settings-content {
		flex: 1;
		min-width: 0;
		padding-bottom: 80px;
	}

	.sections {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	/* ================================================================ */
	/* Page header */
	/* ================================================================ */

	.page-header {
		margin-bottom: 24px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
	}

	/* ================================================================ */
	/* Loading & error */
	/* ================================================================ */

	.loading-skeleton {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.skeleton-card {
		height: 120px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	.error-banner {
		padding: 16px 20px;
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-danger) 30%, transparent);
		border-radius: 8px;
		color: var(--color-danger);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.error-banner p {
		margin: 0;
		font-size: 13px;
	}

	.error-banner button {
		padding: 6px 14px;
		background: var(--color-danger);
		color: white;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-size: 13px;
	}
</style>
