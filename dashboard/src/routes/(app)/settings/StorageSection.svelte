<script lang="ts">
	import { onMount } from 'svelte';
	import { Database } from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import SliderInput from '$lib/components/settings/SliderInput.svelte';
	import { defaults, draft, updateDraft } from '$lib/stores/settings';

	let autoStartEnabled = $state(false);
	let autoStartLoading = $state(false);

	onMount(() => {
		loadAutoStartState();
	});

	async function loadAutoStartState() {
		try {
			const { isEnabled } = await import('@tauri-apps/plugin-autostart');
			autoStartEnabled = await isEnabled();
		} catch {
			// Not in Tauri — ignore.
		}
	}

	async function toggleAutoStart() {
		autoStartLoading = true;
		try {
			if (autoStartEnabled) {
				const { disable } = await import('@tauri-apps/plugin-autostart');
				await disable();
				autoStartEnabled = false;
			} else {
				const { enable } = await import('@tauri-apps/plugin-autostart');
				await enable();
				autoStartEnabled = true;
			}
		} catch {
			// Not in Tauri — ignore.
		}
		autoStartLoading = false;
	}
</script>

{#if $draft}
<SettingsSection
	id="storage"
	title="Storage"
	description="Database location and data retention"
	icon={Database}
	scope="instance"
>
	<div class="field-grid">
		<div class="field full-width">
			<label class="field-label" for="db_path">Database Path</label>
			<input
				id="db_path"
				type="text"
				class="text-input"
				value={$draft.storage.db_path}
				disabled
				title="Database path is read-only in the UI"
			/>
			<span class="field-hint">
				Database path cannot be changed from the UI. Edit config.toml directly to modify.
			</span>
		</div>

		<div class="field full-width">
			<SliderInput
				value={$draft.storage.retention_days}
				label="Data Retention"
				min={0}
				max={365}
				unit=" days"
				helpText="How long to keep data. 0 = keep forever."
				defaultValue={$defaults?.storage.retention_days}
				onchange={(v) =>
					updateDraft('storage.retention_days', v)}
			/>
		</div>

		<div class="field full-width">
			<div class="toggle-row">
				<div class="toggle-info">
					<span class="field-label">Auto-start on Login</span>
					<span class="field-hint">
						Launch Tuitbot automatically when you log in to your computer
					</span>
				</div>
				<button
					type="button"
					class="toggle"
					class:active={autoStartEnabled}
					onclick={toggleAutoStart}
					disabled={autoStartLoading}
					role="switch"
					aria-checked={autoStartEnabled}
					aria-label="Toggle auto-start"
				>
					<span class="toggle-track">
						<span class="toggle-thumb"></span>
					</span>
				</button>
			</div>
		</div>
	</div>
</SettingsSection>
{/if}

<style>
	.field-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.text-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		outline: none;
		transition: border-color 0.15s;
	}

	.text-input:focus {
		border-color: var(--color-accent);
	}

	.text-input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.toggle {
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.toggle-track {
		display: flex;
		align-items: center;
		width: 42px;
		height: 24px;
		padding: 2px;
		background: var(--color-border);
		border-radius: 12px;
		transition: background 0.2s;
	}

	.toggle.active .toggle-track {
		background: var(--color-accent);
	}

	.toggle-thumb {
		width: 20px;
		height: 20px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
	}

	.toggle.active .toggle-thumb {
		transform: translateX(18px);
	}
</style>
