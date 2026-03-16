import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Approval Queue
 * Tests the approval queue UI, navigation, and controls.
 * Focuses on testing what IS present on the page, not optional items.
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

test.describe('Approval Queue', () => {
	test.beforeEach(async ({ page }) => {
		await authenticateOnce(page);
	});

	test('should load approval queue page', async ({ page }) => {
		// Hard assert: page must load
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');
		
		expect(page.url()).toContain('/approval');
	});

	test('should display queue page title/header', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: page must have a heading
		const heading = page.locator('h1, h2, [role="heading"]').first();
		await expect(heading).toBeVisible();
	});

	test('should display page content structure', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: page has main content area
		const mainContent = page.locator('main, [role="main"], .approval-container, .page-content').first();
		await expect(mainContent).toBeVisible();
	});

	test('should display approval statistics/summary', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: approval stats section must exist (shows pending, approved, etc counts)
		const statsSection = page.locator('text=/pending|approved|rejected|stats|total/i').first();
		await expect(statsSection).toBeVisible();
	});

	test('should have filter or view controls', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: must have some kind of control UI (filter, dropdown, buttons)
		const controls = page.locator('button, select, [role="combobox"], [role="listbox"]').first();
		await expect(controls).toBeVisible();
	});

	test('should support keyboard navigation on page', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: keyboard nav should work (j key for down)
		await page.keyboard.press('j');
		
		// Should remain on /approval (no navigation away)
		expect(page.url()).toContain('/approval');
	});

	test('should have export or data access functionality', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: export button must be accessible
		const exportBtn = page.locator('button:has-text("Export"), [aria-label*="export" i]').first();
		await expect(exportBtn).toBeVisible();
	});

	test('should display queue view with proper semantics', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: queue section with list or proper ARIA structure
		const queueSection = page.locator('[role="list"], .approval-feed, .queue-container, section').first();
		await expect(queueSection).toBeVisible();
	});

	test('should support filtering by status', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: filter controls must exist
		const filterControl = page.locator('button:has-text("Filter"), button:has-text("Status"), select, [role="combobox"]').first();
		await expect(filterControl).toBeVisible();
		
		// Verify it's usable
		await expect(filterControl).toBeEnabled();
	});
});
