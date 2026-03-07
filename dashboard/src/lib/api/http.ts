export function resolveBaseUrl(): string {
	if (typeof window === 'undefined') return '';
	if ('__TAURI_INTERNALS__' in window) return 'http://localhost:3001';
	if (window.location.port === '5173') return 'http://localhost:3001';
	return '';
}

export const BASE_URL = resolveBaseUrl();
let token: string = '';
let accountId: string = '00000000-0000-0000-0000-000000000000';
let authMode: 'bearer' | 'cookie' = 'bearer';
let csrfToken: string = '';

export function setToken(t: string) {
	token = t;
}

export function getToken(): string {
	return token;
}

export function setAccountId(id: string) {
	accountId = id;
}

export function getAccountId(): string {
	return accountId;
}

export function setAuthMode(mode: 'bearer' | 'cookie') {
	authMode = mode;
}

export function getAuthMode(): 'bearer' | 'cookie' {
	return authMode;
}

export function setCsrfToken(t: string) {
	csrfToken = t;
}

export function getCsrfToken(): string {
	return csrfToken;
}

export async function request<T>(path: string, options?: RequestInit): Promise<T> {
	const headers: Record<string, string> = {
		'Content-Type': 'application/json',
		'X-Account-Id': accountId
	};

	if (authMode === 'bearer' && token) {
		headers['Authorization'] = `Bearer ${token}`;
	}
	if (authMode === 'cookie' && csrfToken) {
		const method = (options?.method || 'GET').toUpperCase();
		if (method !== 'GET' && method !== 'HEAD') {
			headers['X-CSRF-Token'] = csrfToken;
		}
	}

	const fetchOptions: RequestInit = {
		...options,
		headers: {
			...headers,
			...options?.headers
		}
	};

	// Include cookies for cookie-based auth
	if (authMode === 'cookie') {
		fetchOptions.credentials = 'include';
	}

	const res = await fetch(`${BASE_URL}${path}`, fetchOptions);
	if (!res.ok) {
		const body = await res.json().catch(() => ({ error: res.statusText }));
		throw new Error(body.error || res.statusText);
	}
	return res.json();
}

export async function uploadFile<T>(path: string, file: File): Promise<T> {
	const formData = new FormData();
	formData.append('file', file);

	const headers: Record<string, string> = {
		'X-Account-Id': accountId
		// No Content-Type — browser sets multipart boundary automatically.
	};

	if (authMode === 'bearer' && token) {
		headers['Authorization'] = `Bearer ${token}`;
	}
	if (authMode === 'cookie' && csrfToken) {
		headers['X-CSRF-Token'] = csrfToken;
	}

	const res = await fetch(`${BASE_URL}${path}`, {
		method: 'POST',
		headers,
		body: formData,
		...(authMode === 'cookie' ? { credentials: 'include' as RequestCredentials } : {})
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({ error: res.statusText }));
		throw new Error(body.error || res.statusText);
	}
	return res.json();
}
