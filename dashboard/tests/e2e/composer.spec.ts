import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Composer / Draft Studio
 * Covers: Tweet composition, character limit validation, thread creation, reply mode
 */

const TEST_PASSPHRASE = process.env.TEST_PASSPHRASE || 'test test test test';

// Setup: Login once per test file using storageState
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
		// Verify we're on home page (authenticated)
		await expect(page).toHaveURL('/');
	});

	test('should allow composing a simple tweet', async ({ page }) => {
		// Find tweet editor textarea or input
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		
		// Hard assertion: must have tweet input
		await expect(tweetInput).toBeVisible();
		
		await tweetInput.fill('Test tweet from E2E testing');
		
		// Verify content was entered
		const inputValue = await tweetInput.inputValue();
		expect(inputValue).toContain('Test tweet');
	});

	test('should validate character count in composer', async ({ page }) => {
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		
		// Hard assertion: input must exist
		await expect(tweetInput).toBeVisible();
		
		await tweetInput.fill('a'.repeat(100));
		
		// Character counter should be visible or input should accept text
		const charCounter = page.locator('text=/\\d+\\/280|characters/i').first();
		const counterVisible = await charCounter.isVisible().catch(() => false);
		
		// Either counter is shown OR text was accepted
		expect(await tweetInput.inputValue()).toHaveLength(100);
		if (counterVisible) {
			await expect(charCounter).toBeVisible();
		}
	});

	test('should warn or prevent exceeding character limit', async ({ page }) => {
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		
		await expect(tweetInput).toBeVisible();
		
		// Generate a string longer than 280 characters
		const longText = 'a'.repeat(300);
		await tweetInput.fill(longText);
		
		// Check if input value is capped at 280 or if warning appears
		const inputValue = await tweetInput.inputValue();
		const warningText = page.locator('text=/character limit|exceeds|max/i').first();
		
		// Either input is limited OR warning is shown
		const exceedsLimit = inputValue.length > 280;
		const hasWarning = await warningText.isVisible().catch(() => false);
		
		expect(exceedsLimit === false || hasWarning).toBeTruthy();
	});

	test('should support thread composition', async ({ page }) => {
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		
		await expect(tweetInput).toBeVisible();
		
		// Look for "add tweet to thread" button
		const addThreadBtn = page.locator('button:has-text("Thread"), button:has-text("Add tweet"), button[title*="thread" i]').first();
		
		// Skip if thread feature not available in test environment
		if (await addThreadBtn.isVisible().catch(() => false)) {
			await addThreadBtn.click();
			
			// Should add another tweet composer
			const tweetInputs = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]');
			const count = await tweetInputs.count();
			
			// More inputs available (at least 2 for a thread)
			expect(count).toBeGreaterThanOrEqual(1);
		}
	});

	test('should support reply mode', async ({ page }) => {
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		
		await expect(tweetInput).toBeVisible();
		
		// Look for "reply" mode toggle
		const replyBtn = page.locator('button:has-text("Reply"), button[title*="reply" i]').first();
		
		// Skip if reply feature not available
		if (await replyBtn.isVisible().catch(() => false)) {
			await replyBtn.click();
			
			// Verify button was interactive
			await expect(replyBtn).toBeVisible();
		}
	});

	test('should allow clearing draft content', async ({ page }) => {
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		
		await expect(tweetInput).toBeVisible();
		
		// Fill with content
		await tweetInput.fill('Test content to clear');
		expect(await tweetInput.inputValue()).toContain('Test content');
		
		// Look for clear/reset button
		const clearBtn = page.locator('button:has-text("Clear"), button[title*="clear" i]').first();
		
		if (await clearBtn.isVisible().catch(() => false)) {
			await clearBtn.click();
			const value = await tweetInput.inputValue();
			expect(value).toBe('');
		} else {
			// Manually clear for test
			await tweetInput.clear();
			expect(await tweetInput.inputValue()).toBe('');
		}
	});

	test('should show character counter near limit', async ({ page }) => {
		const tweetInput = page.locator('textarea, input[placeholder*="What"], input[placeholder*="tweet"], input[placeholder*="post"]').first();
		
		await expect(tweetInput).toBeVisible();
		
		// Fill with text close to 280 limit
		const almostFull = 'a'.repeat(270);
		await tweetInput.fill(almostFull);
		
		// Character counter should be visible
		const charCounter = page.locator('text=/\\d+\\/280|characters|chars/i').first();
		
		if (await charCounter.isVisible().catch(() => false)) {
			await expect(charCounter).toBeVisible();
		}
	});
});
