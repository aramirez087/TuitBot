<script lang="ts">
	import AnalyticsDashboard from "$lib/components/AnalyticsDashboard.svelte";
</script>

<svelte:head>
	<title>Analytics — Tuitbot</title>
</svelte:head>

<div class="page-container">
	<AnalyticsDashboard />
</div>

<style>
	.page-container {
		width: 100%;
		max-width: 1400px;
		margin: 0 auto;
		padding: 24px 0;
	}

	@media (max-width: 768px) {
		.page-container {
			padding: 16px;
		}
	}
</style>
