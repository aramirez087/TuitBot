// Smoke test — verifies the Vitest + coverage toolchain is wired correctly.
// Intentionally self-contained: no imports from project files so that an
// empty src/lib/index.ts does not break svelte-check or vitest.
import { describe, it, expect } from 'vitest';

describe('vitest toolchain', () => {
	it('arithmetic works (smoke test)', () => {
		expect(1 + 1).toBe(2);
	});
});
