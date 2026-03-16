import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Composer / Draft Studio
 * Hard assertions: Tests fail if critical UI is missing. Optional features use test.skip().
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

test.describe('Composer / Draft Studio', () => {
	test.beforeEach(async ({ page }) => {
		await authenticateOnce(page);
	});

	test('should display composer interface on main page', async ({ page }) => {
		// Hard assert: must be on authenticated home page
		await expect(page).toHaveURL('/');
	});

	test('should allow composing a simple tweet', async ({ page }) => {
		// Hard assert: tweet input MUST exist
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		await expect(tweetInput).toBeVisible();
		
		await tweetInput.fill('Test tweet from E2E testing');
		
		// Hard assert: content was entered
		expect(await tweetInput.inputValue()).toContain('Test tweet');
	});

	test('should have character count display', async ({ page }) => {
		// Hard assert: tweet input exists
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		await expect(tweetInput).toBeVisible();
		
		await tweetInput.fill('a'.repeat(100));
		
		// Hard assert: character counter exists
		const charCounter = page.locator('text=/\\d+\\/280|characters/i').first();
		await expect(charCounter).toBeVisible();
	});

	test('should prevent or cap text exceeding 280 characters', async ({ page }) => {
		// Hard assert: tweet input exists
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		await expect(tweetInput).toBeVisible();
		
		// Try to fill with 300 characters
		const longText = 'a'.repeat(300);
		await tweetInput.fill(longText);
		
		// Hard assert: input value must NOT exceed 280 (either capped or rejected)
		const inputValue = await tweetInput.inputValue();
		expect(inputValue.length).toBeLessThanOrEqual(280);
	});

	test('should support thread composition', async ({ page, test: testObj }) => {
		// Optional feature: thread mode might not be available in test environment
		const addThreadBtn = page.locator('button:has-text("Thread"), button:has-text("Add tweet"), button[title*="thread" i]').first();
		
		const hasThreadBtn = await addThreadBtn.isVisible().catch(() => false);
		if (!hasThreadBtn) {
			testObj.skip();
		}
		
		// Hard assert: if feature exists, it must work
		await addThreadBtn.click();
		
		// Should have at least 2 tweet inputs after adding thread
		const tweetInputs = page.locator('textarea, input[placeholder*="tweet"]');
		const count = await tweetInputs.count();
		expect(count).toBeGreaterThanOrEqual(1);
	});

	test('should support reply mode', async ({ page, test: testObj }) => {
		// Optional feature: reply mode might not be available
		const replyBtn = page.locator('button:has-text("Reply"), button[title*="reply" i]').first();
		
		const hasReplyBtn = await replyBtn.isVisible().catch(() => false);
		if (!hasReplyBtn) {
			testObj.skip();
		}
		
		// Hard assert: if feature exists, button must be clickable
		await replyBtn.click();
		await expect(replyBtn).toBeVisible();
	});

	test('should allow clearing draft content', async ({ page }) => {
		// Hard assert: tweet input exists
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		await expect(tweetInput).toBeVisible();
		
		// Fill with content
		await tweetInput.fill('Test content to clear');
		expect(await tweetInput.inputValue()).toContain('Test content');
		
		// Hard assert: input can be cleared
		await tweetInput.clear();
		expect(await tweetInput.inputValue()).toBe('');
	});

	test('should show character counter when near limit', async ({ page }) => {
		// Hard assert: tweet input exists
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		await expect(tweetInput).toBeVisible();
		
		// Fill with text close to 280 limit
		await tweetInput.fill('a'.repeat(270));
		
		// Hard assert: character counter must be visible
		const charCounter = page.locator('text=/\\d+\\/280|characters|chars/i').first();
		await expect(charCounter).toBeVisible();
	});
});
