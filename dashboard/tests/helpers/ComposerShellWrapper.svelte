<script lang="ts">
	import ComposerShell from '$lib/components/composer/ComposerShell.svelte';

	interface Props {
		open?: boolean;
		focusMode?: boolean;
		inspectorOpen?: boolean;
		onclose?: () => void;
	}

	let {
		open = true,
		focusMode = false,
		inspectorOpen = false,
		onclose = () => {}
	}: Props = $props();
</script>

<ComposerShell {open} {focusMode} {inspectorOpen} {onclose}>
	<div data-testid="shell-content">Shell test content</div>
</ComposerShell>
