import { writable, derived, get } from 'svelte/store';
import {
	api,
	type TuitbotConfig,
	type SettingsValidationResult,
	type SettingsTestResult
} from '$lib/api';

// --- Writable stores ---

export const config = writable<TuitbotConfig | null>(null);
export const defaults = writable<TuitbotConfig | null>(null);
export const draft = writable<TuitbotConfig | null>(null);
export const loading = writable(true);
export const saving = writable(false);
export const error = writable<string | null>(null);
export const saveError = writable<string | null>(null);
export const validationErrors = writable<SettingsValidationResult['errors']>([]);
export const lastSaved = writable<Date | null>(null);

// --- Derived stores ---

export const isDirty = derived([config, draft], ([$config, $draft]) => {
	if (!$config || !$draft) return false;
	return JSON.stringify($config) !== JSON.stringify($draft);
});

export const fieldErrors = derived(validationErrors, ($errors) => {
	const map: Record<string, string> = {};
	for (const e of $errors) {
		if (e.field) {
			map[e.field] = e.message;
		}
	}
	return map;
});

export const scoringTotal = derived(draft, ($draft) => {
	if (!$draft) return 0;
	const s = $draft.scoring;
	return (
		s.keyword_relevance_max +
		s.follower_count_max +
		s.recency_max +
		s.engagement_rate_max +
		s.reply_count_max +
		s.content_type_max
	);
});

// --- Helper: deep clone ---

function deepClone<T>(obj: T): T {
	return JSON.parse(JSON.stringify(obj));
}

// --- Helper: set a nested value by dot-path ---

function setNestedValue(obj: Record<string, unknown>, path: string, value: unknown): void {
	const keys = path.split('.');
	let current: Record<string, unknown> = obj;
	for (let i = 0; i < keys.length - 1; i++) {
		const key = keys[i];
		if (current[key] === undefined || current[key] === null) {
			current[key] = {};
		}
		current = current[key] as Record<string, unknown>;
	}
	current[keys[keys.length - 1]] = value;
}

// --- Helper: get a nested value by dot-path ---

function getNestedValue(obj: Record<string, unknown>, path: string): unknown {
	const keys = path.split('.');
	let current: unknown = obj;
	for (const key of keys) {
		if (current === undefined || current === null) return undefined;
		current = (current as Record<string, unknown>)[key];
	}
	return current;
}

// --- Actions ---

export async function loadSettings() {
	loading.set(true);
	error.set(null);

	try {
		const [configData, defaultsData] = await Promise.all([
			api.settings.get(),
			api.settings.defaults()
		]);

		config.set(configData);
		defaults.set(defaultsData);
		draft.set(deepClone(configData));
	} catch (e) {
		error.set(e instanceof Error ? e.message : 'Failed to load settings');
	} finally {
		loading.set(false);
	}
}

export function updateDraft(path: string, value: unknown) {
	draft.update(($draft) => {
		if (!$draft) return $draft;
		const clone = deepClone($draft);
		setNestedValue(clone as unknown as Record<string, unknown>, path, value);
		return clone;
	});

	// Clear validation error for this field when user edits it
	validationErrors.update(($errors) => $errors.filter((e) => e.field !== path));
}

export function resetDraft() {
	const $config = get(config);
	if ($config) {
		draft.set(deepClone($config));
		validationErrors.set([]);
		saveError.set(null);
	}
}

export function resetField(path: string) {
	const $defaults = get(defaults);
	if (!$defaults) return;
	const defaultValue = getNestedValue(
		$defaults as unknown as Record<string, unknown>,
		path
	);
	if (defaultValue !== undefined) {
		updateDraft(path, deepClone(defaultValue));
	}
}

export async function saveSettings(): Promise<boolean> {
	const $draft = get(draft);
	if (!$draft) return false;

	saving.set(true);
	saveError.set(null);
	validationErrors.set([]);

	try {
		// Validate first
		const validation = await api.settings.validate($draft);
		if (!validation.valid) {
			validationErrors.set(validation.errors);
			saving.set(false);
			return false;
		}

		// Save
		const updated = await api.settings.patch($draft);
		config.set(updated);
		draft.set(deepClone(updated));
		lastSaved.set(new Date());
		return true;
	} catch (e) {
		saveError.set(e instanceof Error ? e.message : 'Failed to save settings');
		return false;
	} finally {
		saving.set(false);
	}
}

export async function testLlmConnection(): Promise<SettingsTestResult> {
	const $draft = get(draft);
	if (!$draft) {
		return { success: false, error: 'No settings loaded' };
	}

	return api.settings.testLlm({
		provider: $draft.llm.provider,
		api_key: $draft.llm.api_key,
		model: $draft.llm.model,
		base_url: $draft.llm.base_url
	});
}

/** Clear all settings stores (used after factory reset). */
export function resetStores(): void {
	config.set(null);
	defaults.set(null);
	draft.set(null);
	loading.set(true);
	error.set(null);
	saveError.set(null);
	validationErrors.set([]);
	lastSaved.set(null);
}

export function hasDangerousChanges(): boolean {
	const $config = get(config);
	const $draft = get(draft);
	if (!$config || !$draft) return false;

	return (
		$config.llm.provider !== $draft.llm.provider ||
		$config.llm.api_key !== $draft.llm.api_key ||
		$config.x_api.client_id !== $draft.x_api.client_id ||
		$config.x_api.client_secret !== $draft.x_api.client_secret
	);
}
