<script lang="ts">
	import type { ScheduleConfig, ComposeRequest } from '$lib/api';
	import ComposeWorkspace from './composer/ComposeWorkspace.svelte';

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
		onsubmit: (data: ComposeRequest) => void;
	} = $props();

	let triggerElement: Element | null = null;

	$effect(() => {
		if (open) {
			triggerElement = document.activeElement;
		}
	});

	function handleClose() {
		onclose();
		if (triggerElement instanceof HTMLElement) triggerElement.focus();
	}
</script>

{#if open}
	<ComposeWorkspace
		{schedule}
		{onsubmit}
		onclose={handleClose}
		{prefillTime}
		{prefillDate}
		embedded={false}
	/>
{/if}
