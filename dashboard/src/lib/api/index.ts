// Barrel re-export — preserves all existing `$lib/api` imports.
export {
	resolveBaseUrl,
	setToken,
	getToken,
	setAccountId,
	getAccountId,
	setAuthMode,
	getAuthMode,
	setCsrfToken,
	getCsrfToken
} from './http';

export * from './types';
export { api } from './client';
