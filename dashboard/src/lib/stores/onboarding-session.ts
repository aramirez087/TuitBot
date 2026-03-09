import { writable } from 'svelte/store';
import type { InferredProfile } from '$lib/api/types';

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
	/** Profile analysis results from the server. */
	inferred_profile: InferredProfile | null;
	/** Whether profile analysis is in progress. */
	analyzing: boolean;
	/** Warnings from partial analysis. */
	analysis_warnings: string[];
}

function createOnboardingSession() {
	const initial: OnboardingSession = {
		x_connected: false,
		x_user: null,
		oauth_state: '',
		auth_url: '',
		auth_error: '',
		auth_loading: false,
		inferred_profile: null,
		analyzing: false,
		analysis_warnings: []
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
		setAnalyzing: (analyzing: boolean) => {
			update((s) => ({ ...s, analyzing }));
		},
		setInferredProfile: (profile: InferredProfile, warnings: string[]) => {
			update((s) => ({
				...s,
				inferred_profile: profile,
				analysis_warnings: warnings,
				analyzing: false
			}));
		},
		reset: () => set(initial)
	};
}

export const onboardingSession = createOnboardingSession();
