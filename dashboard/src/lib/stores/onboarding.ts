import { writable } from 'svelte/store';

export interface OnboardingData {
	// X API
	client_id: string;
	client_secret: string;
	// Business
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
	// Settings
	approval_mode: boolean;
}

function createOnboardingStore() {
	const { subscribe, update, set } = writable<OnboardingData>({
		client_id: '',
		client_secret: '',
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
		approval_mode: true,
	});

	return {
		subscribe,
		set,
		updateField: <K extends keyof OnboardingData>(key: K, value: OnboardingData[K]) => {
			update((data) => ({ ...data, [key]: value }));
		},
		reset: () => {
			set({
				client_id: '',
				client_secret: '',
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
				approval_mode: true,
			});
		},
	};
}

export const onboardingData = createOnboardingStore();
