import { readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

// In dev mode, read the API token from ~/.tuitbot/api_token so the
// dashboard can authenticate without Tauri's invoke.
function readDevToken(): string {
	try {
		return readFileSync(join(homedir(), '.tuitbot', 'api_token'), 'utf-8').trim();
	} catch {
		return '';
	}
}

export default defineConfig(({ mode }) => ({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 5173,
		strictPort: true
	},
	define: {
		__DEV_API_TOKEN__: JSON.stringify(mode === 'development' ? readDevToken() : '')
	},
	test: {
		// Vitest config — runs unit tests outside SvelteKit's SSR context.
		// Uses jsdom so Svelte components can mount without a real browser.
		environment: 'jsdom',
		globals: true,
		include: ['src/**/*.{test,spec}.{js,ts}'],
		coverage: {
			// v8 provider is fast and requires no extra instrumentation.
			provider: 'v8',
			reporter: ['text', 'lcov', 'json-summary'],
			// lcov.info is what codecov parses; written to coverage/ by default.
			reportsDirectory: './coverage',
			include: ['src/**/*.{js,ts,svelte}'],
			exclude: ['src/**/*.{test,spec}.{js,ts}', 'src/app.d.ts']
		}
	}
}));
