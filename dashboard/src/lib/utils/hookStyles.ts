/**
 * Hook style display names and confidence badge utilities.
 * Maps backend TweetFormat style keys to human-readable labels.
 */

const STYLE_LABELS: Record<string, string> = {
	question: 'Question',
	contrarian_take: 'Hot Take',
	tip: 'Quick Tip',
	list: 'List',
	most_people_think_x: 'Myth Buster',
	storytelling: 'Story',
	before_after: 'Before/After',
	general: 'General',
};

export function getStyleLabel(style: string): string {
	return (
		STYLE_LABELS[style] ??
		style
			.replace(/_/g, ' ')
			.replace(/\b\w/g, (c) => c.toUpperCase())
	);
}

export function getConfidenceBadge(confidence: string): { label: string; cssClass: string } {
	if (confidence === 'high') return { label: 'Strong', cssClass: 'confidence-high' };
	return { label: 'Good', cssClass: 'confidence-medium' };
}
