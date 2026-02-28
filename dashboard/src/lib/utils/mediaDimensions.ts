/** Intrinsic dimensions of a media item */
export interface MediaDimensions {
	width: number;
	height: number;
}

/**
 * X's display slot aspect ratios per media count.
 * Each entry is an array of aspect ratios (width / height) for each slot position.
 *
 * 1 image:  single 16:9 landscape crop
 * 2 images: side-by-side, each ~4:5 portrait
 * 3 images: left tall (2:3) + right stacked (1:1 each)
 * 4 images: 2×2 grid, each ~1:1 square
 */
export const X_SLOT_RATIOS: Record<number, number[]> = {
	1: [16 / 9],
	2: [4 / 5, 4 / 5],
	3: [2 / 3, 1, 1],
	4: [1, 1, 1, 1]
};

/**
 * Load intrinsic dimensions from an image URL.
 * Returns null for load failures or non-image content.
 */
export function loadImageDimensions(url: string): Promise<MediaDimensions | null> {
	return new Promise((resolve) => {
		const img = new Image();
		img.onload = () => resolve({ width: img.naturalWidth, height: img.naturalHeight });
		img.onerror = () => resolve(null);
		img.src = url;
	});
}

/**
 * Calculate how severely a slot will crop the image.
 * Returns a value 0–1 where 0 = no crop, 1 = extreme crop.
 * Values > 0.3 are considered "significant" for the crop indicator.
 *
 * The math: compare the image's intrinsic aspect ratio to the slot's.
 * The further apart they are, the more content is hidden by object-fit: cover.
 */
export function calculateCropSeverity(
	intrinsic: MediaDimensions,
	slotAspectRatio: number
): number {
	if (intrinsic.width <= 0 || intrinsic.height <= 0 || slotAspectRatio <= 0) return 0;
	const imageAR = intrinsic.width / intrinsic.height;
	const ratio = imageAR > slotAspectRatio ? imageAR / slotAspectRatio : slotAspectRatio / imageAR;
	// ratio >= 1. Map 1 → 0 (no crop), 2 → ~0.5, 3+ → approaching 1.
	return Math.min(1, (ratio - 1) / 2);
}

/** Check if a file path looks like a video */
export function isVideoPath(path: string): boolean {
	const lower = path.toLowerCase();
	return lower.endsWith('.mp4') || lower.endsWith('.mov') || lower.endsWith('.webm');
}

/** Crop severity threshold above which we show a crop indicator */
export const CROP_SEVERITY_THRESHOLD = 0.3;
