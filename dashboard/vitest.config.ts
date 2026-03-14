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
			// Threshold enforcement: scoped to the four fully-tested writable
			// stores (approvalStore, analyticsStore, settingsStore, discoveryStore).
			// draftStudio.svelte.ts uses Svelte-5 runes and is tracked but not
			// included in the enforceable threshold.
			// Targets: ≥70% lines/branches, ≥55% functions (WS handlers not
			// reachable in unit tests).  Raise these as Task 3.2–3.5 land.
			// Per-file thresholds for the four fully-tested writable stores.
			// Raised to spec levels (Tasks 3.2–3.5 complete, thresholds now enforced).
			// Target: statements/lines ≥75%, branches ≥70%, functions ≥75%.
			thresholds: {
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
