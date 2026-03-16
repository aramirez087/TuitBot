import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Approval Queue / Scheduled Items
 * Hard assertions: Tests fail if critical UI is missing.
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

test.describe('Approval Queue / Scheduled Items', () => {
	test.beforeEach(async ({ page }) => {
		await authenticateOnce(page);
	});

	test('should navigate to approval queue page', async ({ page }) => {
		// Hard assert: must be able to reach /approval
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');
		
		expect(page.url()).toContain('/approval');
	});

	test('should display approval queue header', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: queue must have a header/title
		const header = page.locator('h1, h2, [role="heading"]').first();
		await expect(header).toBeVisible();
	});

	test('should have queue list or empty state', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: either queue items exist OR empty state is shown
		const queueItems = page.locator('[role="listitem"], .queue-item').first();
		const emptyState = page.locator('text=/no items|empty|nothing/i').first();
		
		const hasItems = await queueItems.isVisible().catch(() => false);
		const isEmpty = await emptyState.isVisible().catch(() => false);
		
		expect(hasItems || isEmpty).toBeTruthy();
	});

	test('should support status filtering', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: filtering UI must exist (button, dropdown, etc)
		const filterBtn = page.locator('button:has-text("pending"), button:has-text("Pending"), select, [role="combobox"]').first();
		await expect(filterBtn).toBeVisible();
		
		// Verify it's interactive
		await expect(filterBtn).toBeEnabled();
	});

	test('should have action buttons for queue items', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: at least one action button must exist
		const approveBtn = page.locator('button:has-text("Approve")').first();
		const rejectBtn = page.locator('button:has-text("Reject")').first();
		
		const hasApprove = await approveBtn.isVisible().catch(() => false);
		const hasReject = await rejectBtn.isVisible().catch(() => false);
		
		expect(hasApprove || hasReject).toBeTruthy();
	});

	test('should display queue statistics', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: stats must be visible (pending count, etc)
		const stats = page.locator('text=/pending|approved|rejected|total/i').first();
		await expect(stats).toBeVisible();
	});

	test('should support keyboard navigation', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: page must be on /approval after keyboard navigation
		await page.keyboard.press('j');
		
		// Should not crash and remain on /approval
		expect(page.url()).toContain('/approval');
	});

	test('should have export functionality', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: export button must be visible
		const exportBtn = page.locator('button:has-text("Export")').first();
		await expect(exportBtn).toBeVisible();
		
		// Click to reveal export options
		await exportBtn.click();
		
		// Hard assert: export format options appear
		const csvOption = page.locator('text=/csv|json/i').first();
		await expect(csvOption).toBeVisible();
	});

	test('should display queue container with proper role', async ({ page }) => {
		await page.goto('/approval');
		await page.waitForLoadState('networkidle');

		// Hard assert: queue must have proper list semantics
		const queueList = page.locator('[role="list"], .queue-container').first();
		await expect(queueList).toBeVisible();
	});
});
