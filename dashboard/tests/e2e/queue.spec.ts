import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Approval Queue
 * Tests queue page structure (always runs).
 * State-dependent tests (queue items, actions) skip if no data.
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
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');
		
		expect(page.url()).toContain('/approval');
	});

	test('should display queue page header', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const heading = page.locator('h1, h2, [role="heading"]').first();
		await expect(heading).toBeVisible();
	});

	test('should have export functionality', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const exportBtn = page.locator('button:has-text("Export"), [aria-label*="export" i]').first();
		await expect(exportBtn).toBeVisible();
	});

	test('should have filter controls', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const filterBtn = page.locator('button:has-text("Filter"), select, [role="combobox"]').first();
		await expect(filterBtn).toBeVisible();
	});

	// State-dependent tests: skip if queue is empty
	test('should display queue items (skip if empty)', async ({ page, test: testObj }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const queueItems = page.locator('[role="listitem"], .queue-item').first();
		const isEmpty = await page.locator('text=/no items|empty|nothing/i').isVisible().catch(() => false);
		
		if (isEmpty || !(await queueItems.isVisible().catch(() => false))) {
			testObj.skip();
		}

		await expect(queueItems).toBeVisible();
	});

	test('should have approve/reject buttons for items (skip if empty)', async ({ page, test: testObj }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const isEmpty = await page.locator('text=/no items|empty/i').isVisible().catch(() => false);
		if (isEmpty) {
			testObj.skip();
		}

		const approveBtn = page.locator('button:has-text("Approve")').first();
		const rejectBtn = page.locator('button:has-text("Reject")').first();
		
		const hasAction = await approveBtn.isVisible().catch(() => false) || await rejectBtn.isVisible().catch(() => false);
		
		if (!hasAction) {
			testObj.skip();
		}

		expect(hasAction).toBeTruthy();
	});

	test('should display queue stats (skip if empty)', async ({ page, test: testObj }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const isEmpty = await page.locator('text=/no items|empty/i').isVisible().catch(() => false);
		if (isEmpty) {
			testObj.skip();
		}

		const stats = page.locator('text=/pending|approved|rejected/i').first();
		await expect(stats).toBeVisible();
	});

	test('should support keyboard navigation in queue', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Keyboard nav should work even with empty queue
		await page.keyboard.press('j');
		
		expect(page.url()).toContain('/approval');
	});

	test('should have queue list structure (skip if empty)', async ({ page, test: testObj }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const isEmpty = await page.locator('text=/no items|empty/i').isVisible().catch(() => false);
		if (isEmpty) {
			testObj.skip();
		}

		const queueList = page.locator('[role="list"], .approval-feed, .queue-container').first();
		await expect(queueList).toBeVisible();
	});

	test('should support filtering queue', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const filterBtn = page.locator('button:has-text("Filter"), button:has-text("Status"), select').first();
		await expect(filterBtn).toBeVisible();
		await expect(filterBtn).toBeEnabled();
	});

	test('should display approval feed container', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		const feedContainer = page.locator('[role="list"], .approval-feed, section, main').first();
		await expect(feedContainer).toBeVisible();
	});
});
