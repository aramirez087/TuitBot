import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: './tests/e2e',
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 2 : 0,
	workers: process.env.CI ? 1 : undefined,
	reporter: process.env.CI ? 'github' : 'list',
	use: {
		baseURL: 'http://localhost:5173',
		trace: 'on-first-retry',
		screenshot: 'only-on-failure'
	},
	// CI installs only Chromium (npx playwright install --with-deps chromium).
	// Run all browsers locally; restrict to chromium in CI to match what's installed.
	projects: process.env.CI
		? [
				{
					name: 'chromium',
					use: { ...devices['Desktop Chrome'] }
				}
		  ]
		: [
				{
					name: 'chromium',
					use: { ...devices['Desktop Chrome'] }
				},
				{
					name: 'firefox',
					use: { ...devices['Desktop Firefox'] }
				},
				{
					name: 'webkit',
					use: { ...devices['Desktop Safari'] }
				}
		  ],
	webServer: {
		command: 'npm run dev',
		url: 'http://localhost:5173',
		reuseExistingServer: !process.env.CI,
		timeout: 30_000
	}
});
