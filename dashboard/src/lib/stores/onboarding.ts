import { writable } from 'svelte/store';
import type { InferredProfile } from '$lib/api/types';

export interface OnboardingData {
	// X Access
	provider_backend: string;
	client_id: string;
	client_secret: string;
	// Profile
	account_type: 'individual' | 'business';
	product_name: string;
	product_description: string;
	product_url: string;
	target_audience: string;
	product_keywords: string[];
	industry_topics: string[];
	// LLM
	llm_provider: string;
	llm_api_key: string;
	llm_model: string;
	llm_base_url: string;
	// Language & Brand
	language: string;
	brand_voice: string;
	// Content Sources
	source_type: string;
	vault_path: string;
	vault_watch: boolean;
	vault_loop_back: boolean;
	folder_id: string;
	connection_id: number | null;
	poll_interval_seconds: number;
	// Settings
	approval_mode: boolean;
}

function createOnboardingStore() {
	const { subscribe, update, set } = writable<OnboardingData>({
		provider_backend: '',
		client_id: '',
		client_secret: '',
		account_type: 'individual',
		product_name: '',
		product_description: '',
		product_url: '',
		target_audience: '',
		product_keywords: [],
		industry_topics: [],
		llm_provider: 'openai',
		llm_api_key: '',
		llm_model: 'gpt-4o-mini',
		llm_base_url: '',
		language: 'en',
		brand_voice: 'balanced',
		source_type: 'local_fs',
		vault_path: '',
		vault_watch: true,
		// Loop-back defaults off: provenance tracking works but file write-back is not yet complete.
		vault_loop_back: false,
		folder_id: '',
		connection_id: null,
		poll_interval_seconds: 300,
		approval_mode: true,
	});

	return {
		subscribe,
		set,
		updateField: <K extends keyof OnboardingData>(key: K, value: OnboardingData[K]) => {
			update((data) => ({ ...data, [key]: value }));
		},
		prefillFromInference: (profile: InferredProfile) => {
			update((data) => {
				const voiceMap: Record<string, string> = {
					professional: 'balanced',
					casual: 'bold',
					formal: 'conservative',
					witty: 'bold'
				};
				const inferredVoice = profile.brand_voice.value;
				const mappedVoice = inferredVoice ? voiceMap[inferredVoice] ?? 'balanced' : data.brand_voice;

				return {
					...data,
					account_type: profile.account_type.value,
					product_name: data.product_name || profile.product_name.value,
					product_description: data.product_description || profile.product_description.value,
					product_url: data.product_url || profile.product_url.value || '',
					target_audience: data.target_audience || profile.target_audience.value,
					product_keywords: data.product_keywords.length > 0 ? data.product_keywords : profile.product_keywords.value,
					industry_topics: data.industry_topics.length > 0 ? data.industry_topics : profile.industry_topics.value,
					brand_voice: mappedVoice
				};
			});
		},
		reset: () => {
			set({
				provider_backend: '',
				client_id: '',
				client_secret: '',
				account_type: 'individual',
				product_name: '',
				product_description: '',
				product_url: '',
				target_audience: '',
				product_keywords: [],
				industry_topics: [],
				llm_provider: 'openai',
				llm_api_key: '',
				llm_model: 'gpt-4o-mini',
				llm_base_url: '',
				language: 'en',
				brand_voice: 'balanced',
				source_type: 'local_fs',
				vault_path: '',
				vault_watch: true,
				vault_loop_back: false,
				folder_id: '',
				connection_id: null,
				poll_interval_seconds: 300,
				approval_mode: true,
			});
		},
	};
}

export const onboardingData = createOnboardingStore();
