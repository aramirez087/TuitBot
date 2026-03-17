/**
 * Persist and restore onboarding form state via localStorage.
 * Allows users to resume mid-onboarding if they navigate away.
 */

export interface OnboardingFormSnapshot {
	step: number;
	data: Record<string, any>;
	timestamp: number;
}

const STORAGE_KEY = 'tuitbot_onboarding_form_state';
const RETENTION_HOURS = 24;

export function saveOnboardingState(currentStep: number, formData: Record<string, any>): void {
	try {
		const snapshot: OnboardingFormSnapshot = {
			step: currentStep,
			data: formData,
			timestamp: Date.now(),
		};
		localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
	} catch (e) {
		// Silently fail if localStorage is unavailable (private mode, quota exceeded, etc.)
		console.debug('Failed to save onboarding state:', e);
	}
}

export function loadOnboardingState(): OnboardingFormSnapshot | null {
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (!stored) return null;

		const snapshot: OnboardingFormSnapshot = JSON.parse(stored);
		const now = Date.now();
		const ageHours = (now - snapshot.timestamp) / (1000 * 60 * 60);

		// Discard if older than retention period
		if (ageHours > RETENTION_HOURS) {
			clearOnboardingState();
			return null;
		}

		return snapshot;
	} catch (e) {
		// Silently fail on parse errors or quota issues
		console.debug('Failed to load onboarding state:', e);
		return null;
	}
}

export function clearOnboardingState(): void {
	try {
		localStorage.removeItem(STORAGE_KEY);
	} catch {
		// Silently fail
	}
}
