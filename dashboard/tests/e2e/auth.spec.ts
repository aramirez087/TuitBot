import { test, expect, Page } from '@playwright/test';

/**
 * E2E Tests: Authentication Flow
 * Covers: Login with passphrase, session persistence, logout
 */

test.describe('Authentication', () => {
	// Use a known test passphrase — assumes server is running with this set
	// In real CI, this would come from test credentials via env var
	const TEST_PASSPHRASE = 'test test test test';

	test('should display login form on initial access', async ({ page }) => {
		await page.goto('/');
		// Should redirect to /login if not authenticated
		await expect(page).toHaveURL(/login/);
		
		// Should show form elements
		await expect(page.locator('label:has-text("Passphrase")')).toBeVisible();
		await expect(page.locator('input[id="passphrase"]')).toBeVisible();
		await expect(page.locator('button[aria-label="Sign in to Tuitbot"]')).toBeVisible();
	});

	test('should login successfully with correct passphrase', async ({ page }) => {
		await page.goto('/login');
		
		// Enter passphrase and submit
		const passphraseInput = page.locator('input[id="passphrase"]');
		await passphraseInput.fill(TEST_PASSPHRASE);
		
		const submitBtn = page.locator('button[aria-label="Sign in to Tuitbot"]');
		await submitBtn.click();
		
		// Should redirect to main app (content/calendar view)
		await expect(page).toHaveURL('/');
		
		// Should show main app content
		// Check for presence of key app elements (will vary based on app state)
		await page.waitForLoadState('networkidle');
	});

	test('should display error for invalid passphrase', async ({ page }) => {
		await page.goto('/login');
		
		// Enter wrong passphrase
		const passphraseInput = page.locator('input[id="passphrase"]');
		await passphraseInput.fill('wrong wrong wrong wrong');
		
		const submitBtn = page.locator('button[aria-label="Sign in to Tuitbot"]');
		await submitBtn.click();
		
		// Should display error message
		const errorMsg = page.locator('div[role="alert"]');
		await expect(errorMsg).toBeVisible();
		await expect(errorMsg).toContainText(/incorrect|unauthorized|failed/i);
		
		// Should stay on login page
		await expect(page).toHaveURL(/login/);
	});

	test('should persist session across page reloads', async ({ page, context }) => {
		// Login
		await page.goto('/login');
		const passphraseInput = page.locator('input[id="passphrase"]');
		await passphraseInput.fill(TEST_PASSPHRASE);
		const submitBtn = page.locator('button[aria-label="Sign in to Tuitbot"]');
		await submitBtn.click();
		
		// Wait for redirect
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');
		
		// Reload page
		await page.reload();
		
		// Should still be authenticated (not redirected to login)
		await expect(page).not.toHaveURL(/login/);
		await page.waitForLoadState('networkidle');
	});

	test('should accept passphrase via Enter key', async ({ page }) => {
		await page.goto('/login');
		
		const passphraseInput = page.locator('input[id="passphrase"]');
		await passphraseInput.fill(TEST_PASSPHRASE);
		
		// Press Enter instead of clicking button
		await passphraseInput.press('Enter');
		
		// Should submit and redirect
		await expect(page).toHaveURL('/');
		await page.waitForLoadState('networkidle');
	});

	test('should disable login button when passphrase is empty', async ({ page }) => {
		await page.goto('/login');
		
		const submitBtn = page.locator('button[aria-label="Sign in to Tuitbot"]');
		await expect(submitBtn).toBeDisabled();
		
		// Fill input
		const passphraseInput = page.locator('input[id="passphrase"]');
		await passphraseInput.fill('a');
		await expect(submitBtn).toBeEnabled();
		
		// Clear input
		await passphraseInput.clear();
		await expect(submitBtn).toBeDisabled();
	});
});
