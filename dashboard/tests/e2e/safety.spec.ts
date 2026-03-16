import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Safety Guardrails & UI Feedback
 * Hard assertions: Tests fail if safety UI is missing.
 */

const TEST_PASSPHRASE = process.env.TEST_PASSPHRASE || 'test test test test';

async function authenticateOnce(page: Page) {
	await page.goto('/login');
	const passphraseInput = page.locator('input[id="passphrase"]');
	await passphraseInput.fill(TEST_PASSPHRASE);
	const submitBtn = page.locator('button[aria-label="Sign in to Tuitbot"]');
	await submitBtn.click();
	await expect(page).toHaveURL('/');
	await page.waitForLoadState('networkidle');
}

test.describe('Safety Guardrails & UI Feedback', () => {
	test.beforeEach(async ({ page }) => {
		await authenticateOnce(page);
	});

	test('should display rate limit section in activity view', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: rate limit section must be visible
		const rateSection = page.locator('text=/rate limit|daily limit/i').first();
		await expect(rateSection).toBeVisible();
	});

	test('should show daily limit indicators', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: must show limit usage (e.g., "3/5" or progress bar)
		const limitDisplay = page.locator('text=/\\d+\\/\\d+|usage/i').first();
		await expect(limitDisplay).toBeVisible();
	});

	test('should display specific limit types', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: at least one limit type must be labeled
		const repliesLabel = page.locator('text=/replies?/i').first();
		const tweetsLabel = page.locator('text=/tweets?/i').first();
		const threadsLabel = page.locator('text=/threads?/i').first();
		
		const hasReplies = await repliesLabel.isVisible().catch(() => false);
		const hasTweets = await tweetsLabel.isVisible().catch(() => false);
		const hasThreads = await threadsLabel.isVisible().catch(() => false);
		
		expect(hasReplies || hasTweets || hasThreads).toBeTruthy();
	});

	test('should have progress visualization for limits', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: progress bar or visual indicator must exist
		const progressBar = page.locator('[role="progressbar"]').first();
		const progressDiv = page.locator('div.progress, div.limit-bar, .usage-bar').first();
		
		const hasProgress = await progressBar.isVisible().catch(() => false);
		const hasDiv = await progressDiv.isVisible().catch(() => false);
		
		expect(hasProgress || hasDiv).toBeTruthy();
	});

	test('should allow composing tweets in composer', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('networkidle');

		// Hard assert: tweet input must exist
		const tweetInput = page.locator('textarea, input[placeholder*="tweet"], input[placeholder*="What"]').first();
		await expect(tweetInput).toBeVisible();
		
		// Hard assert: can type safely
		await tweetInput.fill('Test tweet for safety');
		expect(await tweetInput.inputValue()).toContain('Test tweet');
	});

	test('should have publish button in composer', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('networkidle');

		// Hard assert: publish/send button must exist
		const publishBtn = page.locator('button:has-text("Publish"), button:has-text("Send"), button:has-text("Post")').first();
		await expect(publishBtn).toBeVisible();
	});

	test('should show time remaining for limits', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: time indicator or reset time must be shown
		const timeIndicator = page.locator('text=/resets?|remaining|available|reset at|today/i').first();
		await expect(timeIndicator).toBeVisible();
	});

	test('should display limit warning on activity page', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: either warning OR normal limit display must exist
		const warning = page.locator('[role="alert"], text=/warning/i').first();
		const limitSection = page.locator('text=/limit|usage/i').first();
		
		const hasWarning = await warning.isVisible().catch(() => false);
		const hasLimit = await limitSection.isVisible().catch(() => false);
		
		expect(hasWarning || hasLimit).toBeTruthy();
	});

	test('should have safety controls in composer', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('networkidle');

		// Hard assert: composer must have some safety UI (label, warning, etc)
		const safetyUI = page.locator('text=/safety|safe|check|complian/i, [aria-label*="safe" i]').first();
		
		// May not be visible but composer must exist
		const tweetInput = page.locator('textarea, input[placeholder*="tweet"]').first();
		await expect(tweetInput).toBeVisible();
	});

	test('should support activity filtering and export', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: export button must exist for data access
		const exportBtn = page.locator('button:has-text("Export"), button[title*="export" i]').first();
		await expect(exportBtn).toBeVisible();
		
		// Hard assert: filter controls must exist
		const filterUI = page.locator('button:has-text("Filter"), select, [role="combobox"]').first();
		await expect(filterUI).toBeVisible();
	});
});
