import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'path';

export default defineConfig({
	plugins: [
		svelte({ hot: !process.env.VITEST })
	],
	test: {
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./tests/setup.ts'],
		include: ['src/**/*.{test,spec}.{ts,js}', 'tests/unit/**/*.{test,spec}.{ts,js}'],
		exclude: ['tests/e2e/**', 'node_modules/**'],
		coverage: {
			provider: 'v8',
			reporter: ['text', 'json', 'html', 'lcov'],
			reportsDirectory: './coverage',
			// Broad measurement scope — all lib files appear in reports.
			include: ['src/lib/**'],
			exclude: ['src/lib/assets/**', 'src/**/*.d.ts'],
			// GLOBAL threshold (Task 4.1): Enforce 70% lines coverage across entire frontend.
			// This is the hard gate for CI — all PRs must meet this minimum.
			// Per-file thresholds below are stricter for the four fully-tested stores.
			// Targets: ≥70% lines (global), ≥75% on core stores.
			thresholds: {
				// Global thresholds (enforced across all src/lib/**):
				// Minimum coverage required for any PR to pass CI.
				lines: 70,
				statements: 70,
				branches: 60,
				functions: 65,

				// Per-file thresholds for the four fully-tested writable stores.
				// Raised to spec levels (Tasks 3.2–3.5 complete, thresholds now enforced).
				// Target: statements/lines ≥75%, branches ≥70%, functions ≥75%.
				'src/lib/stores/approval.ts': {
					statements: 75,
					branches: 70,
					functions: 75,
					lines: 75
				},
				'src/lib/stores/analytics.ts': {
					statements: 85,
					branches: 85,
					functions: 80,
					lines: 85
				},
				'src/lib/stores/settings.ts': {
					statements: 75,
					branches: 70,
					functions: 80,
					lines: 75
				},
				'src/lib/stores/targets.ts': {
					statements: 75,
					branches: 70,
					functions: 75,
					lines: 75
				}
			}
		}
	},
	resolve: {
		// Ensure browser exports are used instead of SSR exports.
		// Without this, Svelte 5 resolves to index-server.js in vitest.
		conditions: ['browser', 'import', 'module', 'default'],
		alias: {
			$lib: resolve('./src/lib'),
			$app: resolve('./node_modules/@sveltejs/kit/src/runtime/app')
		}
	}
});
