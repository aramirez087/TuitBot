<script lang="ts">
	/**
	 * ThreadComposer — legacy wrapper, kept for backward compatibility.
	 * All logic now lives in ThreadFlowLane which absorbed this component's
	 * responsibilities in Session 3. ComposeWorkspace renders ThreadFlowLane directly.
	 */
	import type { ThreadBlock } from '$lib/api';
	import ThreadFlowLane from './composer/ThreadFlowLane.svelte';

	let {
		blocks = [],
		onchange,
		onvalidchange
	}: {
		blocks?: ThreadBlock[];
		onchange: (blocks: ThreadBlock[]) => void;
		onvalidchange: (valid: boolean) => void;
	} = $props();

	let laneRef: ThreadFlowLane | undefined = $state();

	export function getBlocks(): ThreadBlock[] { return laneRef?.getBlocks() ?? []; }
	export function setBlocks(newBlocks: ThreadBlock[]) { laneRef?.setBlocks(newBlocks); }
	export async function handleInlineAssist(voiceCue?: string): Promise<void> {
		await laneRef?.handleInlineAssist(voiceCue);
	}
	export function handlePaletteAction(actionId: string) {
		laneRef?.handlePaletteAction(actionId);
	}
</script>

<ThreadFlowLane
	bind:this={laneRef}
	{blocks}
	{onchange}
	{onvalidchange}
/>
