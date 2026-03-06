/**
 * Settings scope constants and helpers for account-scoped override UX.
 *
 * Maps each settings section to its scope type (account vs instance) and the
 * top-level config key(s) that the backend's `_overrides` array references.
 */

export type SectionScope = 'account' | 'instance';

export interface SectionScopeEntry {
	scope: SectionScope;
	keys: string[];
}

/**
 * Maps section `id` values (as used in SettingsSection) to their scope
 * type and the top-level config keys they correspond to.
 *
 * Source of truth: `settings-scope-matrix.md` and `merge.rs` ACCOUNT_SCOPED_KEYS.
 */
export const SECTION_SCOPE: Record<string, SectionScopeEntry> = {
	business: { scope: 'account', keys: ['business'] },
	persona: { scope: 'account', keys: ['business'] },
	scoring: { scope: 'account', keys: ['scoring'] },
	limits: { scope: 'account', keys: ['limits'] },
	schedule: { scope: 'account', keys: ['schedule'] },
	xapi: { scope: 'account', keys: ['x_api'] },
	sources: { scope: 'account', keys: ['content_sources'] },
	llm: { scope: 'instance', keys: ['llm'] },
	storage: { scope: 'instance', keys: ['storage'] },
	lan: { scope: 'instance', keys: [] },
	danger: { scope: 'instance', keys: [] }
};

const DEFAULT_ACCOUNT_ID = '00000000-0000-0000-0000-000000000000';

/** Check whether the given account ID is non-default. */
export function isNonDefault(accountId: string): boolean {
	return accountId !== DEFAULT_ACCOUNT_ID;
}

/** Check whether any of a section's config keys appear in the overridden list. */
export function isSectionOverridden(sectionId: string, overridden: string[]): boolean {
	const entry = SECTION_SCOPE[sectionId];
	if (!entry || entry.keys.length === 0) return false;
	return entry.keys.some((k) => overridden.includes(k));
}
