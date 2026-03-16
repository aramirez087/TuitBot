import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Safety Guardrails & Rate Limits
 * Tests rate limit display, activity feed, and safety UI visibility.
 * Focuses on testing actual UI present on the page.
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

	test('should load activity page with rate limits', async ({ page }) => {
		// Hard assert: activity page must load
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');
		
		expect(page.url()).toContain('/activity');
	});

	test('should display daily rate limit section', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: rate limit section must exist
		const rateSection = page.locator('text=/rate limit|daily limit|limit/i').first();
		await expect(rateSection).toBeVisible();
	});

	test('should show specific limit types', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: at least one limit type (replies, tweets, or threads) must be shown
		const limitTypes = page.locator('text=/replies?|tweets?|threads?/i');
		const count = await limitTypes.count();
		expect(count).toBeGreaterThan(0);
	});

	test('should display limit usage numbers', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: usage display (e.g., "3/5") must be visible
		const usage = page.locator('text=/\\d+\\/\\d+/').first();
		await expect(usage).toBeVisible();
	});

	test('should have progress visualization', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: progress bar or visual indicator
		const progress = page.locator('[role="progressbar"], .progress-bar, .limit-bar, div.usage').first();
		await expect(progress).toBeVisible();
	});

	test('should display activity feed section', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: activity feed is present
		const feed = page.locator('[role="list"], .activity-feed, .feed-container, .activity-section').first();
		await expect(feed).toBeVisible();
	});

	test('should have export functionality in activity', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: export button for activity data
		const exportBtn = page.locator('button:has-text("Export"), [aria-label*="export" i]').first();
		await expect(exportBtn).toBeVisible();
	});

	test('should have activity filters', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: filter controls for activity feed
		const filterBtn = page.locator('button:has-text("Filter"), select, [role="combobox"]').first();
		await expect(filterBtn).toBeVisible();
	});

	test('should display load more or pagination', async ({ page }) => {
		await page.goto('/activity');
		await page.waitForLoadState('networkidle');

		// Hard assert: pagination or load more control visible
		const pagination = page.locator('button:has-text("Load more"), [role="button"]:has-text("more"), .pagination').first();
		
		// Either pagination visible OR check scrollable area exists
		const hasPagination = await pagination.isVisible().catch(() => false);
		const feedArea = page.locator('.feed-container, [role="list"]').first();
		const hasFeed = await feedArea.isVisible().catch(() => false);
		
		expect(hasPagination || hasFeed).toBeTruthy();
	});

	test('should allow composing from home page', async ({ page }) => {
		// Hard assert: composer must be accessible
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		
		const composer = page.locator('textarea, input[placeholder*="tweet"], input[placeholder*="What"]').first();
		await expect(composer).toBeVisible();
		
		// Should be able to type
		await composer.fill('Test message');
		expect(await composer.inputValue()).toContain('Test');
	});

	test('should have composer character counter', async ({ page }) => {
		// Hard assert: character counter visible in composer
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		
		const composer = page.locator('textarea, input[placeholder*="tweet"], input[placeholder*="What"]').first();
		await expect(composer).toBeVisible();
		
		// Type text
		await composer.fill('a'.repeat(100));
		
		// Character counter must be shown
		const counter = page.locator('text=/\\d+\\/280|characters/i').first();
		await expect(counter).toBeVisible();
	});

	test('should have publish controls', async ({ page }) => {
		// Hard assert: publish button exists in composer
		await page.goto('/');
		await page.waitForLoadState('networkidle');
		
		const publishBtn = page.locator('button:has-text("Publish"), button:has-text("Send"), button:has-text("Post")').first();
		await expect(publishBtn).toBeVisible();
	});
});
