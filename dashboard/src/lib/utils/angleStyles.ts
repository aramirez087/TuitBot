/**
 * Angle type display names and evidence type badge utilities.
 * Used by the AngleCards component for mined angle rendering.
 */

const ANGLE_TYPE_LABELS: Record<string, string> = {
	story: 'Story',
	listicle: 'Listicle',
	hot_take: 'Hot Take',
};

const EVIDENCE_TYPE_CONFIG: Record<string, { label: string; cssVar: string }> = {
	contradiction: { label: 'Contradiction', cssVar: '--color-warning' },
	data_point: { label: 'Data Point', cssVar: '--color-accent' },
	aha_moment: { label: 'Aha Moment', cssVar: '--color-success' },
};

export function getAngleTypeLabel(type: string): string {
	return (
		ANGLE_TYPE_LABELS[type] ??
		type
			.replace(/_/g, ' ')
			.replace(/\b\w/g, (c) => c.toUpperCase())
	);
}

export function getEvidenceTypeConfig(type: string): { label: string; cssVar: string } {
	return (
		EVIDENCE_TYPE_CONFIG[type] ?? {
			label: type
				.replace(/_/g, ' ')
				.replace(/\b\w/g, (c) => c.toUpperCase()),
			cssVar: '--color-text-subtle',
		}
	);
}

export function truncateCitation(text: string, max: number = 40): string {
	if (text.length <= max) return text;
	return text.slice(0, max - 1) + '\u2026';
}
