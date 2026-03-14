<script lang="ts">
	import { Sparkles, Search } from 'lucide-svelte';
	import { formatCombo, matchEvent } from '$lib/utils/shortcuts';
	import { api } from '$lib/api';
	import { flushSync } from 'svelte';
	import CommandPalette from '../CommandPalette.svelte';
	import type ThreadFlowLane from './ThreadFlowLane.svelte';
	import type TweetEditor from './TweetEditor.svelte';
	import type { AttachedMedia } from './TweetEditor.svelte';

	let {
		mode = 'tweet',
		embedded = false,
		canSubmit = false,
		focusMode = false,
		previewMode = false,
		inspectorOpen = false,
		showFromNotes = false,
		isMobile = false,
		threadFlowRef,
		tweetEditorRef,
		attachedMedia = [],
		onaiassist,
		onaction,
		onmediachange,
		ontextinsert,
	}: {
		mode?: 'tweet' | 'thread';
		embedded?: boolean;
		canSubmit?: boolean;
		focusMode?: boolean;
		previewMode?: boolean;
		inspectorOpen?: boolean;
		showFromNotes?: boolean;
		isMobile?: boolean;
		threadFlowRef?: ThreadFlowLane;
		tweetEditorRef?: TweetEditor;
		attachedMedia?: AttachedMedia[];
		onaiassist: () => void;
		/** Generic action bus — see handleKeydown for action names */
		onaction: (action: string) => void;
		onmediachange?: (media: AttachedMedia[]) => void;
		ontextinsert?: (text: string) => void;
	} = $props();

	let paletteOpen = $state(false);

	async function handleImagePaste() {
		try {
			const { readImage } = await import('@tauri-apps/plugin-clipboard-manager');
			const img = await readImage();
			const { width: w, height: h } = await img.size();
			const rgba = await img.rgba();

			const canvas = document.createElement('canvas');
			canvas.width = w; canvas.height = h;
			const ctx = canvas.getContext('2d');
			if (!ctx) return;
			ctx.putImageData(new ImageData(new Uint8ClampedArray(rgba), w, h), 0, 0);
			const blob: Blob | null = await new Promise((resolve) => canvas.toBlob(resolve, 'image/png'));
			if (!blob) return;

			const file = new File([blob], 'pasted-image.png', { type: 'image/png' });
			const result = await api.media.upload(file);

			if (mode === 'thread') {
				threadFlowRef?.addMediaToFocusedBlock(result.path);
			} else {
				flushSync(() => {
					onmediachange?.([...(attachedMedia ?? []), {
						path: result.path, file,
						previewUrl: api.media.fileUrl(result.path),
						mediaType: result.media_type
					}]);
				});
			}
			return;
		} catch { /* not an image */ }

		try {
			const { readText } = await import('@tauri-apps/plugin-clipboard-manager');
			const text = await readText();
			if (text) document.execCommand('insertText', false, text);
		} catch { /* no text either */ }
	}

	function handleKeydown(e: KeyboardEvent) {
		if (paletteOpen) return;

		if (previewMode) {
			if (e.key === 'Escape') { e.preventDefault(); onaction('close-preview'); return; }
			if (matchEvent(e, 'cmd+shift+p')) { e.preventDefault(); onaction('toggle-preview'); return; }
			return;
		}

		if (matchEvent(e, 'cmd+k')) { e.preventDefault(); paletteOpen = true; return; }
		if (matchEvent(e, 'cmd+shift+f')) { e.preventDefault(); if (!embedded) onaction('focus-mode'); return; }
		if (matchEvent(e, 'cmd+shift+enter')) { e.preventDefault(); onaction('submit'); return; }
		if (matchEvent(e, 'cmd+enter')) {
			if (mode === 'tweet') { e.preventDefault(); onaction('switch-to-thread'); }
			return;
		}
		if (matchEvent(e, 'cmd+v')) { e.preventDefault(); handleImagePaste(); return; }
		if (matchEvent(e, 'cmd+shift+j')) { e.preventDefault(); onaction('ai-inline'); return; }
		if (matchEvent(e, 'cmd+shift+n')) { e.preventDefault(); onaction('mode-tweet'); return; }
		if (matchEvent(e, 'cmd+shift+t')) { e.preventDefault(); onaction('mode-thread'); return; }
		if (matchEvent(e, 'cmd+i')) { e.preventDefault(); onaction('toggle-inspector'); return; }
		if (matchEvent(e, 'cmd+shift+p')) { e.preventDefault(); onaction('toggle-preview'); return; }
		if (e.key === 'Escape') {
			if (showFromNotes) { onaction('close-notes'); return; }
			if (isMobile && inspectorOpen) { onaction('close-mobile-inspector'); return; }
			if (!embedded && focusMode) { onaction('exit-focus'); return; }
			if (!embedded) { onaction('close'); return; }
		}
	}

	function handlePaletteAction(actionId: string) {
		paletteOpen = false;
		switch (actionId) {
			case 'focus-mode': onaction('focus-mode'); break;
			case 'mode-tweet': onaction('mode-tweet'); break;
			case 'mode-thread': onaction('mode-thread'); break;
			case 'submit': onaction('submit'); break;
			case 'ai-improve': onaction('ai-inline'); break;
			case 'ai-from-notes': onaction('ai-from-notes'); break;
			case 'ai-generate': onaction('ai-generate'); break;
			case 'toggle-inspector': onaction('toggle-inspector'); break;
			case 'toggle-preview': onaction('toggle-preview'); break;
			case 'attach-media': tweetEditorRef?.triggerFileSelect(); break;
			case 'add-card': case 'duplicate': case 'split': case 'merge':
			case 'move-up': case 'move-down':
				threadFlowRef?.handlePaletteAction(actionId); break;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="subheader-bar" role="toolbar" aria-label="Composer actions">
	<div class="bar-left">
		<button
			class="bar-btn ai-btn"
			onclick={onaiassist}
			title="AI improve ({formatCombo('cmd+shift+j')})"
			aria-label="AI improve selection or post"
		>
			<Sparkles size={14} />
		</button>
	</div>

	<div class="bar-right">
		<button
			class="shortcut-trigger"
			onclick={() => { paletteOpen = true; }}
			aria-label="Command palette"
		>
			<Search size={12} />
			<kbd>{formatCombo('cmd+k')}</kbd>
			<span class="shortcut-label">All shortcuts</span>
		</button>
	</div>
</div>

{#if paletteOpen}
	<CommandPalette
		open={paletteOpen}
		{mode}
		onclose={() => { paletteOpen = false; }}
		onaction={handlePaletteAction}
	/>
{/if}

<style>
	.subheader-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 4px 20px;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border-subtle) 35%, transparent);
		flex-shrink: 0;
		min-height: 36px;
	}

	.bar-left { display: flex; align-items: center; gap: 2px; }
	.bar-right { display: flex; align-items: center; }

	.bar-btn {
		display: flex; align-items: center; justify-content: center;
		width: 30px; height: 30px;
		border: none; border-radius: 6px;
		background: transparent; color: var(--color-text-muted);
		cursor: pointer; transition: all 0.12s ease; padding: 0;
	}

	.bar-btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		color: var(--color-accent);
	}

	.bar-btn.ai-btn:hover:not(:disabled) {
		color: var(--color-warning, #d29922);
		background: color-mix(in srgb, var(--color-warning, #d29922) 10%, transparent);
	}

	.shortcut-trigger {
		display: inline-flex; align-items: center; gap: 5px;
		padding: 4px 8px; border: none; border-radius: 4px;
		background: transparent; color: var(--color-text-subtle);
		font-size: 11px; cursor: pointer; transition: color 0.12s ease; opacity: 0.7;
	}

	.shortcut-trigger:hover { color: var(--color-text-muted); opacity: 1; }

	.shortcut-trigger kbd {
		display: inline-flex; align-items: center;
		padding: 0 4px; border-radius: 3px;
		background: color-mix(in srgb, var(--color-surface-active) 60%, transparent);
		color: var(--color-text-subtle); font-size: 10px;
		font-family: var(--font-mono); font-weight: 500;
		border: 1px solid color-mix(in srgb, var(--color-border-subtle) 40%, transparent);
		line-height: 1.5;
	}

	@media (max-width: 480px) {
		.shortcut-label { display: none; }
		.subheader-bar { padding: 4px 12px; }
	}

	@media (pointer: coarse) { .bar-btn { min-width: 44px; min-height: 44px; } }
	@media (prefers-reduced-motion: reduce) { .bar-btn, .shortcut-trigger { transition: none; } }
</style>
