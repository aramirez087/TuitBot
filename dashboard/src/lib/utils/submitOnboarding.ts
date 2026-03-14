import { api } from '$lib/api';
import { claimSession } from '$lib/stores/auth';
import { connectWs } from '$lib/stores/websocket';
import { trackFunnel } from '$lib/analytics/funnel';

export type OnboardingSubmitResult =
	| { kind: 'redirect'; to: string }
	| { kind: 'error'; message: string };

export interface OnboardingSubmitParams {
	data: {
		provider_backend: string;
		client_id: string;
		client_secret: string;
		product_name: string;
		product_description: string;
		product_url?: string;
		target_audience: string;
		product_keywords: string[];
		industry_topics: string[];
		approval_mode: boolean;
		llm_provider: string;
		llm_api_key: string;
		llm_model: string;
		llm_base_url?: string;
		source_type: string;
		connection_id: number | null;
		folder_id: string;
		vault_path: string;
		vault_watch: boolean;
		vault_loop_back: boolean;
		poll_interval_seconds?: number;
		x_user_id: string;
		x_username: string;
		x_display_name: string;
		x_avatar_url?: string;
	};
	showClaimStep: boolean;
	claimPassphrase: string;
	alreadyClaimed: boolean;
}

export async function submitOnboarding(
	params: OnboardingSubmitParams
): Promise<OnboardingSubmitResult> {
	const { data, showClaimStep, claimPassphrase, alreadyClaimed } = params;

	const hasLlm =
		data.llm_provider === 'ollama' ||
		(data.llm_api_key.trim().length > 0 && data.llm_model.trim().length > 0);
	const hasX = data.provider_backend === 'scraper' || data.client_id.trim().length > 0;
	const tierLabel = hasLlm
		? hasX ? 'generation_ready' : 'profile_ready'
		: hasX ? 'exploration_ready' : 'profile_ready';

	trackFunnel('onboarding:submitted', {
		has_x_auth: !!data.x_user_id,
		has_llm: hasLlm,
		has_vault: data.vault_path.length > 0 || data.connection_id !== null || data.folder_id.length > 0,
		tier: tierLabel,
	});

	let config: Record<string, unknown> = {
		x_api:
			data.provider_backend === 'scraper'
				? { provider_backend: 'scraper' }
				: {
						...(data.client_id ? { client_id: data.client_id } : {}),
						...(data.client_secret ? { client_secret: data.client_secret } : {}),
					},
		business: {
			product_name: data.product_name,
			product_description: data.product_description,
			...(data.product_url ? { product_url: data.product_url } : {}),
			target_audience: data.target_audience,
			product_keywords: data.product_keywords,
			industry_topics: data.industry_topics,
		},
		approval_mode: data.approval_mode,
	};

	if (data.llm_provider && (data.llm_provider === 'ollama' || data.llm_api_key)) {
		config.llm = {
			provider: data.llm_provider,
			...(data.llm_api_key ? { api_key: data.llm_api_key } : {}),
			model: data.llm_model,
			...(data.llm_base_url ? { base_url: data.llm_base_url } : {}),
		};
	}

	if (data.source_type === 'google_drive' && (data.connection_id || data.folder_id)) {
		config.content_sources = {
			sources: [
				{
					source_type: 'google_drive',
					path: null,
					folder_id: data.folder_id || null,
					service_account_key: null,
					connection_id: data.connection_id,
					watch: data.vault_watch,
					file_patterns: ['*.md', '*.txt'],
					loop_back_enabled: false,
					poll_interval_seconds: data.poll_interval_seconds || 300,
				},
			],
		};
	} else if (data.vault_path) {
		config.content_sources = {
			sources: [
				{
					source_type: 'local_fs',
					path: data.vault_path,
					folder_id: null,
					service_account_key: null,
					watch: data.vault_watch,
					file_patterns: ['*.md', '*.txt'],
					loop_back_enabled: data.vault_loop_back,
					poll_interval_seconds: null,
				},
			],
		};
	}

	if (data.x_user_id) {
		config.x_profile = {
			x_user_id: data.x_user_id,
			x_username: data.x_username,
			x_display_name: data.x_display_name,
			x_avatar_url: data.x_avatar_url || null,
		};
	}

	if (showClaimStep && claimPassphrase.trim()) {
		config.claim = { passphrase: claimPassphrase.trim() };
	}

	try {
		const result = await api.settings.init(config);

		if (result.status === 'validation_failed' && result.errors) {
			return {
				kind: 'error',
				message: result.errors
					.map((e: { field: string; message: string }) => `${e.field}: ${e.message}`)
					.join('; '),
			};
		}

		if (result.csrf_token) {
			claimSession(result.csrf_token);
			connectWs();
		}

		trackFunnel('onboarding:completed', { tier: tierLabel, claimed: showClaimStep });

		const exampleText = 'Just discovered an interesting take on this — what do you all think? 🧵';
		return {
			kind: 'redirect',
			to: alreadyClaimed
				? '/login'
				: `/drafts?new=true&prefill_content=${encodeURIComponent(exampleText)}`,
		};
	} catch (e) {
		const msg = e instanceof Error ? e.message : '';

		if (e instanceof TypeError && msg.includes('fetch')) {
			trackFunnel('onboarding:error', { error: 'network', step: 'submit' });
			return { kind: 'error', message: "Can't reach the Tuitbot server. Check that it's running and try again." };
		}

		if (msg.toLowerCase().includes('already exists')) {
			trackFunnel('onboarding:409-recovery', { reason: 'already_exists' });
			const exampleText = 'Just discovered an interesting take on this — what do you all think? 🧵';
			return { kind: 'redirect', to: `/drafts?new=true&prefill_content=${encodeURIComponent(exampleText)}` };
		}

		if (msg.toLowerCase().includes('already claimed') && config.claim) {
			trackFunnel('onboarding:409-recovery', { reason: 'already_claimed' });
			try {
				delete config.claim;
				await api.settings.init(config);
				return { kind: 'redirect', to: '/login' };
			} catch (retryErr) {
				const retryMsg = retryErr instanceof Error ? retryErr.message : '';
				if (retryMsg.toLowerCase().includes('already exists')) {
					return { kind: 'redirect', to: '/login' };
				}
			}
		}

		const errorMsg = msg || 'Failed to create configuration';
		trackFunnel('onboarding:error', { error: errorMsg, step: 'submit' });
		return { kind: 'error', message: errorMsg };
	}
}
