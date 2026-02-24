<script lang="ts">
	import { Zap } from 'lucide-svelte';
</script>

<div class="welcome">
	<div class="welcome-icon">
		<Zap size={48} strokeWidth={1.5} />
	</div>
	<h1 class="welcome-title">Welcome to Tuitbot</h1>
	<p class="welcome-subtitle">
		Your autonomous growth co-pilot for X (Twitter). Tuitbot discovers relevant
		conversations, builds relationships, and grows your account organically.
	</p>
	<div class="welcome-features">
		<div class="feature">
			<span class="feature-dot"></span>
			<span>Discover and reply to relevant conversations</span>
		</div>
		<div class="feature">
			<span class="feature-dot"></span>
			<span>Post educational tweets and weekly threads</span>
		</div>
		<div class="feature">
			<span class="feature-dot"></span>
			<span>Track analytics to optimize your strategy</span>
		</div>
		<div class="feature">
			<span class="feature-dot"></span>
			<span>All data stays on your machine</span>
		</div>
	</div>
	<p class="welcome-hint">
		This setup takes about 3 minutes. You'll need your X API credentials and
		an LLM API key.
	</p>
</div>

<style>
	.welcome {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: 16px;
		padding: 24px 0;
	}

	.welcome-icon {
		color: var(--color-accent);
		margin-bottom: 8px;
	}

	.welcome-title {
		font-size: 28px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0;
	}

	.welcome-subtitle {
		font-size: 15px;
		color: var(--color-text-muted);
		max-width: 480px;
		line-height: 1.6;
		margin: 0;
	}

	.welcome-features {
		display: flex;
		flex-direction: column;
		gap: 10px;
		text-align: left;
		margin: 16px 0;
	}

	.feature {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 14px;
		color: var(--color-text);
	}

	.feature-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background-color: var(--color-accent);
		flex-shrink: 0;
	}

	.welcome-hint {
		font-size: 13px;
		color: var(--color-text-subtle);
		margin: 8px 0 0 0;
	}
</style>
