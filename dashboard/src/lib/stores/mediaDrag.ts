/**
 * Shared module-level state for intra-page media drag-and-drop.
 *
 * Uses mouse events instead of HTML5 DnD because WKWebView (Tauri on macOS)
 * doesn't reliably support intra-page HTML5 drag-and-drop for custom elements.
 */

let current: { path: string; sourceBlockId: string } | null = null;

type TransferFn = (targetBlockId: string, mediaPath: string, sourceBlockId: string) => void;
let transferHandler: TransferFn | null = null;

export function startMediaDrag(path: string, sourceBlockId: string) {
	current = { path, sourceBlockId };
}

export function getMediaDrag() {
	return current;
}

export function endMediaDrag() {
	current = null;
}

export function isMediaDragActive(): boolean {
	return current !== null;
}

export function registerTransferHandler(fn: TransferFn | null) {
	transferHandler = fn;
}

export function performTransfer(targetBlockId: string): boolean {
	if (!current || !transferHandler || current.sourceBlockId === targetBlockId) return false;
	transferHandler(targetBlockId, current.path, current.sourceBlockId);
	current = null;
	return true;
}
