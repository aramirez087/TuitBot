import { writable } from 'svelte/store';

export interface OnboardingXUser {
	id: string;
	username: string;
	name: string;
	profile_image_url: string | null;
	description: string | null;
	location: string | null;
	url: string | null;
}

export interface OnboardingSession {
	/** Whether the user has connected their X account during onboarding. */
	x_connected: boolean;
	/** The authenticated X user's profile (populated after successful OAuth). */
	x_user: OnboardingXUser | null;
	/** The CSRF state parameter for the current OAuth flow. */
	oauth_state: string;
	/** The X authorization URL to open. */
	auth_url: string;
	/** Error message from the last auth attempt. */
	auth_error: string;
	/** Whether an auth operation is in progress. */
	auth_loading: boolean;
}

function createOnboardingSession() {
	const initial: OnboardingSession = {
		x_connected: false,
		x_user: null,
		oauth_state: '',
		auth_url: '',
		auth_error: '',
		auth_loading: false
	};

	const { subscribe, update, set } = writable<OnboardingSession>(initial);

	return {
		subscribe,
		set,
		update,
		setConnected: (user: OnboardingXUser) => {
			update((s) => ({
				...s,
				x_connected: true,
				x_user: user,
				auth_loading: false,
				auth_error: ''
			}));
		},
		setAuthUrl: (url: string, state: string) => {
			update((s) => ({
				...s,
				auth_url: url,
				oauth_state: state,
				auth_loading: false
			}));
		},
		setError: (error: string) => {
			update((s) => ({ ...s, auth_error: error, auth_loading: false }));
		},
		setLoading: (loading: boolean) => {
			update((s) => ({ ...s, auth_loading: loading }));
		},
		reset: () => set(initial)
	};
}

export const onboardingSession = createOnboardingSession();
