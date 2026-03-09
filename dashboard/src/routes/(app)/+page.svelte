<script lang="ts">
	import DraftStudioShell from "$lib/components/drafts/DraftStudioShell.svelte";
	import ActivationChecklist from "$lib/components/onboarding/ActivationChecklist.svelte";
	import { capabilityTier } from "$lib/stores/capability";
</script>

<svelte:head>
	<title>Home — Tuitbot</title>
</svelte:head>

{#if $capabilityTier !== 'posting_ready'}
	<ActivationChecklist />
{/if}
<DraftStudioShell />
