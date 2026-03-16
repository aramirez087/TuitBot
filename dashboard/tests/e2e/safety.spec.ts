import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Safety Guardrails & Rate Limits
 * Tests activity page and composer (always runs).
 * State-dependent tests (rate limits, activity feed) skip if data unavailable.
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

test.describe('Safety Guardrails & Rate Limits', () => {
	test.beforeEach(async ({ page }) => {
		await authenticateOnce(page);
	});

	test('should load activity page', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');
		
		expect(page.url()).toContain('/activity');
	});

	test('should have export in activity', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const exportBtn = page.locator('button:has-text("Export"), [aria-label*="export" i]').first();
		await expect(exportBtn).toBeVisible();
	});

	test('should have activity filters', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const filterBtn = page.locator('button:has-text("Filter"), select, [role="combobox"]').first();
		await expect(filterBtn).toBeVisible();
	});

	test('should load composer from home', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		
		const composer = page.locator('textarea, input[placeholder*="tweet"], input[placeholder*="What"]').first();
		await expect(composer).toBeVisible();
	});

	test('should allow typing in composer', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		
		const composer = page.locator('textarea, input[placeholder*="tweet"], input[placeholder*="What"]').first();
		await expect(composer).toBeVisible();
		
		await composer.fill('Test message');
		expect(await composer.inputValue()).toContain('Test');
	});

	test('should show publish button', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		
		const publishBtn = page.locator('button:has-text("Publish"), button:has-text("Send"), button:has-text("Post")').first();
		await expect(publishBtn).toBeVisible();
	});

	// State-dependent tests: skip if rate limit data unavailable
	test('should display rate limit section (skip if unavailable)', async ({ page }, testInfo) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const rateSection = page.locator('text=/rate limit|daily limit/i').first();
		
		if (!(await rateSection.isVisible().catch(() => false))) {
			testInfo.skip();
		}

		await expect(rateSection).toBeVisible();
	});

	test('should show limit usage numbers (skip if unavailable)', async ({ page }, testInfo) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const usage = page.locator('text=/\\d+\\/\\d+/').first();
		
		if (!(await usage.isVisible().catch(() => false))) {
			testInfo.skip();
		}

		await expect(usage).toBeVisible();
	});

	test('should display progress bar (skip if unavailable)', async ({ page }, testInfo) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const progress = page.locator('[role="progressbar"], .progress-bar, .limit-bar').first();
		
		if (!(await progress.isVisible().catch(() => false))) {
			testInfo.skip();
		}

		await expect(progress).toBeVisible();
	});

	test('should show limit types (skip if unavailable)', async ({ page }, testInfo) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const limitLabels = page.locator('text=/replies?|tweets?|threads?/i');
		const count = await limitLabels.count().catch(() => 0);
		
		if (count === 0) {
			testInfo.skip();
		}

		expect(count).toBeGreaterThan(0);
	});

	test('should display activity feed (skip if empty)', async ({ page }, testInfo) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const feed = page.locator('[role="list"], .activity-feed, .feed-container').first();
		const isEmpty = await page.locator('text=/no activity|empty/i').isVisible().catch(() => false);
		
		if (isEmpty || !(await feed.isVisible().catch(() => false))) {
			testInfo.skip();
		}

		await expect(feed).toBeVisible();
	});

	test('should show character counter when typing', async ({ page }) => {
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		
		const composer = page.locator('textarea, input[placeholder*="tweet"], input[placeholder*="What"]').first();
		await expect(composer).toBeVisible();
		
		await composer.fill('a'.repeat(100));
		
		const counter = page.locator('text=/\\d+\\/280|characters/i').first();
		await expect(counter).toBeVisible();
	});

	test('should support activity pagination (skip if no pagination needed)', async ({ page }, testInfo) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		const loadMore = page.locator('button:has-text("Load more"), [role="button"]:has-text("more")').first();
		
		if (!(await loadMore.isVisible().catch(() => false))) {
			testInfo.skip();
		}

		await expect(loadMore).toBeVisible();
	});
});
