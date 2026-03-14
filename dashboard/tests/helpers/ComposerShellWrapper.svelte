<script lang="ts">
	import ComposerShell from '$lib/components/composer/ComposerShell.svelte';

	let {
		open = true,
		focusMode = false,
		inspectorOpen = false,
		onclose = () => {}
	} = $props();
</script>

<ComposerShell {open} {focusMode} {inspectorOpen} {onclose}>
	<div data-testid="shell-content">Shell test content</div>
</ComposerShell>
