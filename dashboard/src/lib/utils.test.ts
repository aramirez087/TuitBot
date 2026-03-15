// Minimal smoke-test for the lib/index barrel export.
// Ensures Vitest + coverage toolchain is wired correctly.
// Add real tests alongside feature code as the dashboard grows.
import { describe, it, expect } from 'vitest';
import * as lib from './index';

describe('lib barrel', () => {
	it('exports an object (smoke test)', () => {
		expect(typeof lib).toBe('object');
	});
});
