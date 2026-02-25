<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Briefcase,
		MessageCircle,
		Target,
		Shield,
		Clock,
		Brain,
		Key,
		Database,
		Save,
		RotateCcw,
		Check,
		AlertTriangle,
		Eye,
		EyeOff,
		Plus,
		X,
		Power
	} from 'lucide-svelte';
	import SettingsSection from '$lib/components/settings/SettingsSection.svelte';
	import TagInput from '$lib/components/settings/TagInput.svelte';
	import SliderInput from '$lib/components/settings/SliderInput.svelte';
	import TimeRangeBar from '$lib/components/settings/TimeRangeBar.svelte';
	import ConnectionTest from '$lib/components/settings/ConnectionTest.svelte';
	import {
		config,
		defaults,
		draft,
		loading,
		saving,
		error,
		saveError,
		isDirty,
		fieldErrors,
		scoringTotal,
		lastSaved,
		loadSettings,
		updateDraft,
		resetDraft,
		saveSettings,
		hasDangerousChanges,
		testLlmConnection
	} from '$lib/stores/settings';

	// --- Section nav ---

	const sections = [
		{ id: 'business', label: 'Business', icon: Briefcase },
		{ id: 'persona', label: 'Persona', icon: MessageCircle },
		{ id: 'scoring', label: 'Scoring', icon: Target },
		{ id: 'limits', label: 'Limits', icon: Shield },
		{ id: 'schedule', label: 'Schedule', icon: Clock },
		{ id: 'llm', label: 'LLM', icon: Brain },
		{ id: 'xapi', label: 'X API', icon: Key },
		{ id: 'storage', label: 'Storage', icon: Database }
	];

	let activeSection = $state('business');
	let showSaved = $state(false);
	let showConfirm = $state(false);
	let showApiKey = $state(false);
	let showClientSecret = $state(false);
	let savedTimeout: ReturnType<typeof setTimeout> | null = null;

	// --- IntersectionObserver for section highlighting ---

	let observers: IntersectionObserver[] = [];

	function setupObservers() {
		observers.forEach((o) => o.disconnect());
		observers = [];

		for (const section of sections) {
			const el = document.getElementById(section.id);
			if (!el) continue;

			const observer = new IntersectionObserver(
				(entries) => {
					for (const entry of entries) {
						if (entry.isIntersecting) {
							activeSection = section.id;
						}
					}
				},
				{ rootMargin: '-80px 0px -60% 0px', threshold: 0 }
			);
			observer.observe(el);
			observers.push(observer);
		}
	}

	onMount(() => {
		loadSettings().then(() => {
			// Small delay so DOM is ready
			setTimeout(setupObservers, 100);
		});
		loadAutoStartState();
	});

	onDestroy(() => {
		observers.forEach((o) => o.disconnect());
		if (savedTimeout) clearTimeout(savedTimeout);
	});

	function scrollToSection(id: string) {
		const el = document.getElementById(id);
		if (el) {
			el.scrollIntoView({ behavior: 'smooth', block: 'start' });
		}
	}

	// --- Save flow ---

	async function handleSave() {
		if (hasDangerousChanges()) {
			showConfirm = true;
			return;
		}
		await doSave();
	}

	async function doSave() {
		showConfirm = false;
		const ok = await saveSettings();
		if (ok) {
			showSaved = true;
			if (savedTimeout) clearTimeout(savedTimeout);
			savedTimeout = setTimeout(() => {
				showSaved = false;
			}, 3000);
		}
	}

	// --- Timezone list ---

	const commonTimezones = [
		'UTC',
		'America/New_York',
		'America/Chicago',
		'America/Denver',
		'America/Los_Angeles',
		'America/Anchorage',
		'Pacific/Honolulu',
		'America/Toronto',
		'America/Vancouver',
		'America/Mexico_City',
		'America/Sao_Paulo',
		'America/Argentina/Buenos_Aires',
		'Europe/London',
		'Europe/Paris',
		'Europe/Berlin',
		'Europe/Madrid',
		'Europe/Rome',
		'Europe/Amsterdam',
		'Europe/Stockholm',
		'Europe/Moscow',
		'Europe/Istanbul',
		'Africa/Cairo',
		'Africa/Johannesburg',
		'Asia/Dubai',
		'Asia/Kolkata',
		'Asia/Bangkok',
		'Asia/Singapore',
		'Asia/Shanghai',
		'Asia/Tokyo',
		'Asia/Seoul',
		'Australia/Sydney',
		'Australia/Melbourne',
		'Pacific/Auckland'
	];

	// --- Auto-start ---
	let autoStartEnabled = $state(false);
	let autoStartLoading = $state(false);

	async function loadAutoStartState() {
		try {
			const { isEnabled } = await import('@tauri-apps/plugin-autostart');
			autoStartEnabled = await isEnabled();
		} catch {
			// Not in Tauri — ignore.
		}
	}

	async function toggleAutoStart() {
		autoStartLoading = true;
		try {
			if (autoStartEnabled) {
				const { disable } = await import('@tauri-apps/plugin-autostart');
				await disable();
				autoStartEnabled = false;
			} else {
				const { enable } = await import('@tauri-apps/plugin-autostart');
				await enable();
				autoStartEnabled = true;
			}
		} catch {
			// Not in Tauri — ignore.
		}
		autoStartLoading = false;
	}

	let timezoneSearch = $state('');

	const filteredTimezones = $derived(
		timezoneSearch
			? commonTimezones.filter((tz) =>
					tz.toLowerCase().includes(timezoneSearch.toLowerCase())
				)
			: commonTimezones
	);

	// --- LLM provider model suggestions ---

	const modelSuggestions: Record<string, string> = {
		openai: 'gpt-4o-mini',
		anthropic: 'claude-sonnet-4-5-20250514',
		ollama: 'llama3.1'
	};

	const baseUrlPlaceholders: Record<string, string> = {
		openai: 'https://api.openai.com/v1',
		anthropic: 'https://api.anthropic.com/v1',
		ollama: 'http://localhost:11434/v1'
	};

	// --- Preferred times ---

	function addPreferredTime() {
		if (!$draft) return;
		const times = [...$draft.schedule.preferred_times, '12:00'];
		updateDraft('schedule.preferred_times', times);
	}

	function removePreferredTime(index: number) {
		if (!$draft) return;
		const times = $draft.schedule.preferred_times.filter((_: string, i: number) => i !== index);
		updateDraft('schedule.preferred_times', times);
	}

	function updatePreferredTime(index: number, value: string) {
		if (!$draft) return;
		const times = [...$draft.schedule.preferred_times];
		times[index] = value;
		updateDraft('schedule.preferred_times', times);
	}

	// --- Scoring bar segments ---

	const scoringSignals = $derived(
		$draft
			? [
					{
						key: 'keyword_relevance_max',
						label: 'Keywords',
						value: $draft.scoring.keyword_relevance_max,
						color: '#58a6ff'
					},
					{
						key: 'follower_count_max',
						label: 'Followers',
						value: $draft.scoring.follower_count_max,
						color: '#3fb950'
					},
					{
						key: 'recency_max',
						label: 'Recency',
						value: $draft.scoring.recency_max,
						color: '#d29922'
					},
					{
						key: 'engagement_rate_max',
						label: 'Engagement',
						value: $draft.scoring.engagement_rate_max,
						color: '#f78166'
					},
					{
						key: 'reply_count_max',
						label: 'Reply count',
						value: $draft.scoring.reply_count_max,
						color: '#bc8cff'
					},
					{
						key: 'content_type_max',
						label: 'Content type',
						value: $draft.scoring.content_type_max,
						color: '#f85149'
					}
				]
			: []
	);

	const allDays = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
</script>

<svelte:head>
	<title>Settings — Tuitbot</title>
</svelte:head>

{#if $loading}
	<div class="page-header">
		<h1>Settings</h1>
		<p class="subtitle">Configure automation behavior and preferences</p>
	</div>
	<div class="loading-skeleton">
		{#each Array(4) as _}
			<div class="skeleton-card"></div>
		{/each}
	</div>
{:else if $error}
	<div class="page-header">
		<h1>Settings</h1>
		<p class="subtitle">Configure automation behavior and preferences</p>
	</div>
	<div class="error-banner">
		<p>{$error}</p>
		<button onclick={() => loadSettings()}>Retry</button>
	</div>
{:else if $draft}
	<div class="settings-layout">
		<!-- Section nav -->
		<nav class="section-nav">
			<div class="nav-title">Settings</div>
			{#each sections as section}
				{@const Icon = section.icon}
				<button
					class="nav-item"
					class:active={activeSection === section.id}
					onclick={() => scrollToSection(section.id)}
				>
					<Icon size={15} />
					<span>{section.label}</span>
				</button>
			{/each}
		</nav>

		<!-- Content area -->
		<div class="settings-content">
			<div class="page-header">
				<h1>Settings</h1>
				<p class="subtitle">Configure automation behavior and preferences</p>
			</div>

			<div class="sections">
				<!-- ============================================================ -->
				<!-- BUSINESS PROFILE -->
				<!-- ============================================================ -->
				<SettingsSection
					id="business"
					title="Business Profile"
					description="Your product details and keywords for discovery"
					icon={Briefcase}
				>
					<div class="field-grid">
						<div class="field">
							<label class="field-label" for="product_name">
								Product Name <span class="required">*</span>
							</label>
							<input
								id="product_name"
								type="text"
								class="text-input"
								class:has-error={$fieldErrors['business.product_name']}
								value={$draft.business.product_name}
								oninput={(e) => updateDraft('business.product_name', e.currentTarget.value)}
								placeholder="My Product"
							/>
							{#if $fieldErrors['business.product_name']}
								<span class="field-error">{$fieldErrors['business.product_name']}</span>
							{/if}
						</div>

						<div class="field">
							<label class="field-label" for="product_url">Product URL</label>
							<input
								id="product_url"
								type="url"
								class="text-input"
								value={$draft.business.product_url ?? ''}
								oninput={(e) =>
									updateDraft(
										'business.product_url',
										e.currentTarget.value || null
									)}
								placeholder="https://example.com"
							/>
						</div>

						<div class="field full-width">
							<label class="field-label" for="product_description">
								Product Description <span class="required">*</span>
							</label>
							<textarea
								id="product_description"
								class="textarea-input"
								class:has-error={$fieldErrors['business.product_description']}
								value={$draft.business.product_description}
								oninput={(e) =>
									updateDraft('business.product_description', e.currentTarget.value)}
								placeholder="A one-line description of what your product does"
								rows="2"
							></textarea>
							{#if $fieldErrors['business.product_description']}
								<span class="field-error">{$fieldErrors['business.product_description']}</span>
							{/if}
						</div>

						<div class="field full-width">
							<label class="field-label" for="target_audience">
								Target Audience <span class="required">*</span>
							</label>
							<textarea
								id="target_audience"
								class="textarea-input"
								value={$draft.business.target_audience}
								oninput={(e) =>
									updateDraft('business.target_audience', e.currentTarget.value)}
								placeholder="Describe who your product is for"
								rows="2"
							></textarea>
						</div>

						<div class="field full-width">
							<TagInput
								value={$draft.business.product_keywords}
								label="Product Keywords"
								placeholder="Add keywords and press Enter"
								helpText="Keywords used for tweet discovery"
								error={$fieldErrors['business.product_keywords'] ?? ''}
								defaultValue={$defaults?.business.product_keywords}
								onchange={(tags) => updateDraft('business.product_keywords', tags)}
							/>
						</div>

						<div class="field full-width">
							<TagInput
								value={$draft.business.competitor_keywords}
								label="Competitor Keywords"
								placeholder="Add competitor keywords"
								helpText="Keywords related to competitors for discovery"
								defaultValue={$defaults?.business.competitor_keywords}
								onchange={(tags) =>
									updateDraft('business.competitor_keywords', tags)}
							/>
						</div>

						<div class="field full-width">
							<TagInput
								value={$draft.business.industry_topics}
								label="Industry Topics"
								placeholder="Add topics and press Enter"
								helpText="Topics for content generation"
								error={$fieldErrors['business.industry_topics'] ?? ''}
								defaultValue={$defaults?.business.industry_topics}
								onchange={(tags) =>
									updateDraft('business.industry_topics', tags)}
							/>
						</div>
					</div>
				</SettingsSection>

				<!-- ============================================================ -->
				<!-- CONTENT PERSONA -->
				<!-- ============================================================ -->
				<SettingsSection
					id="persona"
					title="Content Persona"
					description="Shape the personality and voice of your generated content"
					icon={MessageCircle}
				>
					<div class="field-grid">
						<div class="field full-width">
							<label class="field-label" for="brand_voice">Brand Voice</label>
							<textarea
								id="brand_voice"
								class="textarea-input"
								value={$draft.business.brand_voice ?? ''}
								oninput={(e) =>
									updateDraft(
										'business.brand_voice',
										e.currentTarget.value || null
									)}
								placeholder="Describe the personality and tone for all generated content"
								rows="3"
							></textarea>
						</div>

						<div class="field full-width">
							<label class="field-label" for="reply_style">Reply Style</label>
							<textarea
								id="reply_style"
								class="textarea-input"
								value={$draft.business.reply_style ?? ''}
								oninput={(e) =>
									updateDraft(
										'business.reply_style',
										e.currentTarget.value || null
									)}
								placeholder="Style guidelines specific to replies"
								rows="2"
							></textarea>
						</div>

						<div class="field full-width">
							<label class="field-label" for="content_style">Content Style</label>
							<textarea
								id="content_style"
								class="textarea-input"
								value={$draft.business.content_style ?? ''}
								oninput={(e) =>
									updateDraft(
										'business.content_style',
										e.currentTarget.value || null
									)}
								placeholder="Style guidelines for original tweets and threads"
								rows="2"
							></textarea>
						</div>

						<div class="field full-width">
							<TagInput
								value={$draft.business.persona_opinions}
								label="Persona Opinions"
								placeholder="Add opinions the persona holds"
								helpText="Used to add variety and authenticity to content"
								onchange={(tags) =>
									updateDraft('business.persona_opinions', tags)}
							/>
						</div>

						<div class="field full-width">
							<TagInput
								value={$draft.business.persona_experiences}
								label="Persona Experiences"
								placeholder="Add experiences the persona can reference"
								helpText="Keeps content authentic and relatable"
								onchange={(tags) =>
									updateDraft('business.persona_experiences', tags)}
							/>
						</div>

						<div class="field full-width">
							<TagInput
								value={$draft.business.content_pillars}
								label="Content Pillars"
								placeholder="Add core themes"
								helpText="Broad themes the account focuses on"
								onchange={(tags) =>
									updateDraft('business.content_pillars', tags)}
							/>
						</div>
					</div>
				</SettingsSection>

				<!-- ============================================================ -->
				<!-- SCORING ENGINE -->
				<!-- ============================================================ -->
				<SettingsSection
					id="scoring"
					title="Scoring Engine"
					description="Tune the 6-signal scoring system that decides which tweets to reply to"
					icon={Target}
				>
					<div class="field-grid">
						<div class="field full-width">
							<SliderInput
								value={$draft.scoring.threshold}
								label="Reply Threshold"
								min={0}
								max={100}
								unit=" pts"
								helpText="Minimum score needed for a tweet to trigger a reply"
								defaultValue={$defaults?.scoring.threshold}
								onchange={(v) => updateDraft('scoring.threshold', v)}
							/>
						</div>

						<div class="field full-width scoring-bar-section">
							<div class="scoring-bar-header">
								<span class="field-label">Signal Weights</span>
								<span class="scoring-total">
									Total max: <strong>{$scoringTotal.toFixed(1)}</strong> pts
								</span>
							</div>
							<div class="scoring-bar">
								{#each scoringSignals as signal}
									{#if signal.value > 0}
										<div
											class="scoring-segment"
											style="flex: {signal.value}; background: {signal.color}"
											title="{signal.label}: {signal.value}"
										></div>
									{/if}
								{/each}
							</div>
							<div class="scoring-legend">
								{#each scoringSignals as signal}
									<div class="legend-item">
										<span class="legend-dot" style="background: {signal.color}"></span>
										<span class="legend-label">{signal.label}</span>
										<span class="legend-value">{signal.value}</span>
									</div>
								{/each}
							</div>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.scoring.keyword_relevance_max}
								label="Keyword Relevance"
								min={0}
								max={50}
								step={0.5}
								unit=" pts"
								helpText="How well tweet matches your keywords"
								defaultValue={$defaults?.scoring.keyword_relevance_max}
								onchange={(v) =>
									updateDraft('scoring.keyword_relevance_max', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.scoring.follower_count_max}
								label="Follower Count"
								min={0}
								max={50}
								step={0.5}
								unit=" pts"
								helpText="Author follower count (bell curve scoring)"
								defaultValue={$defaults?.scoring.follower_count_max}
								onchange={(v) =>
									updateDraft('scoring.follower_count_max', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.scoring.recency_max}
								label="Recency"
								min={0}
								max={50}
								step={0.5}
								unit=" pts"
								helpText="How recently the tweet was posted"
								defaultValue={$defaults?.scoring.recency_max}
								onchange={(v) => updateDraft('scoring.recency_max', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.scoring.engagement_rate_max}
								label="Engagement Rate"
								min={0}
								max={50}
								step={0.5}
								unit=" pts"
								helpText="Ratio of engagement to followers"
								defaultValue={$defaults?.scoring.engagement_rate_max}
								onchange={(v) =>
									updateDraft('scoring.engagement_rate_max', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.scoring.reply_count_max}
								label="Reply Count"
								min={0}
								max={50}
								step={0.5}
								unit=" pts"
								helpText="Fewer existing replies = higher score"
								defaultValue={$defaults?.scoring.reply_count_max}
								onchange={(v) =>
									updateDraft('scoring.reply_count_max', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.scoring.content_type_max}
								label="Content Type"
								min={0}
								max={50}
								step={0.5}
								unit=" pts"
								helpText="Text-only originals score highest"
								defaultValue={$defaults?.scoring.content_type_max}
								onchange={(v) =>
									updateDraft('scoring.content_type_max', v)}
							/>
						</div>
					</div>
				</SettingsSection>

				<!-- ============================================================ -->
				<!-- SAFETY & LIMITS -->
				<!-- ============================================================ -->
				<SettingsSection
					id="limits"
					title="Safety & Limits"
					description="Rate limits, delays, and content safety rules"
					icon={Shield}
				>
					<div class="field-grid">
						<div class="field">
							<SliderInput
								value={$draft.limits.max_replies_per_day}
								label="Max Replies / Day"
								min={1}
								max={50}
								defaultValue={$defaults?.limits.max_replies_per_day}
								onchange={(v) =>
									updateDraft('limits.max_replies_per_day', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.limits.max_tweets_per_day}
								label="Max Tweets / Day"
								min={1}
								max={20}
								defaultValue={$defaults?.limits.max_tweets_per_day}
								onchange={(v) =>
									updateDraft('limits.max_tweets_per_day', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.limits.max_threads_per_week}
								label="Max Threads / Week"
								min={0}
								max={7}
								defaultValue={$defaults?.limits.max_threads_per_week}
								onchange={(v) =>
									updateDraft('limits.max_threads_per_week', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.limits.max_replies_per_author_per_day}
								label="Max Replies / Author / Day"
								min={1}
								max={10}
								defaultValue={$defaults?.limits.max_replies_per_author_per_day}
								onchange={(v) =>
									updateDraft('limits.max_replies_per_author_per_day', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.limits.min_action_delay_seconds}
								label="Min Action Delay"
								min={10}
								max={600}
								unit="s"
								helpText="Minimum seconds between actions"
								defaultValue={$defaults?.limits.min_action_delay_seconds}
								onchange={(v) =>
									updateDraft('limits.min_action_delay_seconds', v)}
							/>
						</div>

						<div class="field">
							<SliderInput
								value={$draft.limits.max_action_delay_seconds}
								label="Max Action Delay"
								min={10}
								max={600}
								unit="s"
								helpText="Maximum seconds between actions"
								defaultValue={$defaults?.limits.max_action_delay_seconds}
								onchange={(v) =>
									updateDraft('limits.max_action_delay_seconds', v)}
							/>
						</div>

						<div class="field full-width">
							<SliderInput
								value={$draft.limits.product_mention_ratio}
								label="Product Mention Ratio"
								min={0}
								max={1}
								step={0.05}
								unit=""
								helpText="Fraction of replies that may mention your product ({(
									$draft.limits.product_mention_ratio * 100
								).toFixed(0)}%)"
								defaultValue={$defaults?.limits.product_mention_ratio}
								onchange={(v) =>
									updateDraft('limits.product_mention_ratio', v)}
							/>
						</div>

						<div class="field full-width">
							<TagInput
								value={$draft.limits.banned_phrases}
								label="Banned Phrases"
								placeholder="Add phrases that should never appear in content"
								helpText="Content containing these phrases will be regenerated"
								defaultValue={$defaults?.limits.banned_phrases}
								onchange={(tags) =>
									updateDraft('limits.banned_phrases', tags)}
							/>
						</div>

						<div class="field full-width">
							<label class="field-label" for="mode">Operating Mode</label>
							<select
								id="mode"
								class="text-input"
								value={$draft.mode ?? 'autopilot'}
								onchange={(e) => updateDraft('mode', e.currentTarget.value)}
							>
								<option value="autopilot">Autopilot — fully autonomous posting</option>
								<option value="composer">Composer — you control what gets posted</option>
							</select>
							<span class="field-hint">
								{#if ($draft.mode ?? 'autopilot') === 'composer'}
									Composer mode disables all autonomous loops. Use AI Assist to generate content, then approve manually.
								{:else}
									Autopilot runs discovery, content generation, and posting automatically.
								{/if}
							</span>
						</div>

						<div class="field full-width">
							<div class="toggle-row">
								<div class="toggle-info">
									<span class="field-label">Approval Mode</span>
									<span class="field-hint">
										Queue all posts for manual review before publishing{#if ($draft.mode ?? 'autopilot') === 'composer'} (always on in Composer mode){/if}
									</span>
								</div>
								<button
									type="button"
									class="toggle"
									class:active={$draft.approval_mode}
									onclick={() =>
										updateDraft('approval_mode', !$draft.approval_mode)}
									role="switch"
									aria-checked={$draft.approval_mode}
									aria-label="Toggle approval mode"
								>
									<span class="toggle-track">
										<span class="toggle-thumb"></span>
									</span>
								</button>
							</div>
						</div>
					</div>
				</SettingsSection>

				<!-- ============================================================ -->
				<!-- SCHEDULE -->
				<!-- ============================================================ -->
				<SettingsSection
					id="schedule"
					title="Schedule"
					description="Active hours, posting times, and timezone"
					icon={Clock}
				>
					<div class="field-grid">
						<div class="field full-width">
							<label class="field-label" for="timezone">Timezone</label>
							<input
								id="timezone-search"
								type="text"
								class="text-input"
								placeholder="Search timezones..."
								bind:value={timezoneSearch}
							/>
							<select
								id="timezone"
								class="select-input"
								value={$draft.schedule.timezone}
								onchange={(e) =>
									updateDraft('schedule.timezone', e.currentTarget.value)}
							>
								{#each filteredTimezones as tz}
									<option value={tz}>{tz}</option>
								{/each}
							</select>
						</div>

						<div class="field full-width">
							<TimeRangeBar
								start={$draft.schedule.active_hours_start}
								end={$draft.schedule.active_hours_end}
								activeDays={$draft.schedule.active_days}
								timezone={$draft.schedule.timezone}
								onchange={(s, e) => {
									updateDraft('schedule.active_hours_start', s);
									updateDraft('schedule.active_hours_end', e);
								}}
								onDaysChange={(days) =>
									updateDraft('schedule.active_days', days)}
							/>
						</div>

						<div class="field full-width">
							<div class="preferred-times-header">
								<span class="field-label">Preferred Posting Times</span>
								<button type="button" class="add-btn" onclick={addPreferredTime}>
									<Plus size={14} />
									Add Time
								</button>
							</div>
							{#if $draft.schedule.preferred_times.length === 0}
								<span class="field-hint">
									No preferred times set. The content loop will use interval-based posting. Add times for specific scheduling, or use "auto" for research-backed defaults (09:15, 12:30, 17:00).
								</span>
							{/if}
							<div class="time-list">
								{#each $draft.schedule.preferred_times as time, i}
									<div class="time-row">
										{#if time === 'auto'}
											<span class="auto-badge">auto</span>
											<span class="field-hint">09:15, 12:30, 17:00</span>
										{:else}
											<input
												type="time"
												class="time-input"
												value={time}
												onchange={(e) =>
													updatePreferredTime(i, e.currentTarget.value)}
											/>
										{/if}
										<button
											type="button"
											class="remove-btn"
											onclick={() => removePreferredTime(i)}
											aria-label="Remove time"
										>
											<X size={14} />
										</button>
									</div>
								{/each}
							</div>
						</div>

						<div class="field">
							<label class="field-label" for="thread_day">Thread Preferred Day</label>
							<select
								id="thread_day"
								class="select-input"
								value={$draft.schedule.thread_preferred_day ?? ''}
								onchange={(e) =>
									updateDraft(
										'schedule.thread_preferred_day',
										e.currentTarget.value || null
									)}
							>
								<option value="">None (interval mode)</option>
								{#each allDays as day}
									<option value={day}>{day}</option>
								{/each}
							</select>
						</div>

						<div class="field">
							<label class="field-label" for="thread_time">Thread Preferred Time</label>
							<input
								id="thread_time"
								type="time"
								class="time-input wide"
								value={$draft.schedule.thread_preferred_time}
								onchange={(e) =>
									updateDraft(
										'schedule.thread_preferred_time',
										e.currentTarget.value
									)}
							/>
							{#if $defaults}
								<span class="field-default">
									Default: {$defaults.schedule.thread_preferred_time}
								</span>
							{/if}
						</div>
					</div>
				</SettingsSection>

				<!-- ============================================================ -->
				<!-- LLM PROVIDER -->
				<!-- ============================================================ -->
				<SettingsSection
					id="llm"
					title="LLM Provider"
					description="AI model configuration for content generation"
					icon={Brain}
				>
					<div class="field-grid">
						<div class="field">
							<label class="field-label" for="llm_provider">Provider</label>
							<select
								id="llm_provider"
								class="select-input"
								value={$draft.llm.provider}
								onchange={(e) =>
									updateDraft('llm.provider', e.currentTarget.value)}
							>
								<option value="">Select provider...</option>
								<option value="openai">OpenAI</option>
								<option value="anthropic">Anthropic</option>
								<option value="ollama">Ollama</option>
							</select>
						</div>

						<div class="field">
							<label class="field-label" for="llm_model">Model</label>
							<input
								id="llm_model"
								type="text"
								class="text-input"
								value={$draft.llm.model}
								oninput={(e) =>
									updateDraft('llm.model', e.currentTarget.value)}
								placeholder={modelSuggestions[$draft.llm.provider] ?? 'Model name'}
							/>
							{#if $draft.llm.provider && modelSuggestions[$draft.llm.provider]}
								<span class="field-hint">
									Suggested: {modelSuggestions[$draft.llm.provider]}
								</span>
							{/if}
						</div>

						{#if $draft.llm.provider === 'openai' || $draft.llm.provider === 'anthropic'}
							<div class="field full-width">
								<label class="field-label" for="llm_api_key">API Key</label>
								<div class="password-wrapper">
									<input
										id="llm_api_key"
										type={showApiKey ? 'text' : 'password'}
										class="text-input password-input"
										value={$draft.llm.api_key ?? ''}
										oninput={(e) =>
											updateDraft(
												'llm.api_key',
												e.currentTarget.value || null
											)}
										placeholder="sk-..."
									/>
									<button
										type="button"
										class="password-toggle"
										onclick={() => (showApiKey = !showApiKey)}
										aria-label={showApiKey ? 'Hide' : 'Show'}
									>
										{#if showApiKey}
											<EyeOff size={16} />
										{:else}
											<Eye size={16} />
										{/if}
									</button>
								</div>
							</div>
						{/if}

						<div class="field full-width">
							<label class="field-label" for="llm_base_url">Base URL</label>
							<input
								id="llm_base_url"
								type="text"
								class="text-input"
								value={$draft.llm.base_url ?? ''}
								oninput={(e) =>
									updateDraft(
										'llm.base_url',
										e.currentTarget.value || null
									)}
								placeholder={baseUrlPlaceholders[$draft.llm.provider] ??
									'Custom endpoint URL'}
							/>
							<span class="field-hint">
								Leave empty to use the default endpoint
							</span>
						</div>

						<div class="field full-width">
							<ConnectionTest
								label="Test Connection"
								ontest={testLlmConnection}
							/>
						</div>
					</div>
				</SettingsSection>

				<!-- ============================================================ -->
				<!-- X API -->
				<!-- ============================================================ -->
				<SettingsSection
					id="xapi"
					title="X API"
					description="Twitter/X API credentials for posting and discovery"
					icon={Key}
				>
					<div class="field-grid">
						<div class="field full-width info-banner">
							<p>
								OAuth authentication is managed via <code>tuitbot auth</code> in the CLI. Configure your Client ID and Secret below, then run the auth command to complete the OAuth flow.
							</p>
						</div>

						<div class="field">
							<label class="field-label" for="client_id">Client ID</label>
							<input
								id="client_id"
								type="text"
								class="text-input"
								value={$draft.x_api.client_id}
								oninput={(e) =>
									updateDraft('x_api.client_id', e.currentTarget.value)}
								placeholder="Your X API Client ID"
							/>
						</div>

						<div class="field">
							<label class="field-label" for="client_secret">
								Client Secret
							</label>
							<div class="password-wrapper">
								<input
									id="client_secret"
									type={showClientSecret ? 'text' : 'password'}
									class="text-input password-input"
									value={$draft.x_api.client_secret ?? ''}
									oninput={(e) =>
										updateDraft(
											'x_api.client_secret',
											e.currentTarget.value || null
										)}
									placeholder="Optional for public clients"
								/>
								<button
									type="button"
									class="password-toggle"
									onclick={() => (showClientSecret = !showClientSecret)}
									aria-label={showClientSecret ? 'Hide' : 'Show'}
								>
									{#if showClientSecret}
										<EyeOff size={16} />
									{:else}
										<Eye size={16} />
									{/if}
								</button>
							</div>
						</div>

						<div class="field">
							<label class="field-label" for="auth_mode">Auth Mode</label>
							<select
								id="auth_mode"
								class="select-input"
								value={$draft.auth.mode}
								onchange={(e) =>
									updateDraft('auth.mode', e.currentTarget.value)}
							>
								<option value="manual">Manual</option>
								<option value="local_callback">Local Callback</option>
							</select>
						</div>
					</div>
				</SettingsSection>

				<!-- ============================================================ -->
				<!-- STORAGE -->
				<!-- ============================================================ -->
				<SettingsSection
					id="storage"
					title="Storage"
					description="Database location and data retention"
					icon={Database}
				>
					<div class="field-grid">
						<div class="field full-width">
							<label class="field-label" for="db_path">Database Path</label>
							<input
								id="db_path"
								type="text"
								class="text-input"
								value={$draft.storage.db_path}
								disabled
								title="Database path is read-only in the UI"
							/>
							<span class="field-hint">
								Database path cannot be changed from the UI. Edit config.toml directly to modify.
							</span>
						</div>

						<div class="field full-width">
							<SliderInput
								value={$draft.storage.retention_days}
								label="Data Retention"
								min={0}
								max={365}
								unit=" days"
								helpText="How long to keep data. 0 = keep forever."
								defaultValue={$defaults?.storage.retention_days}
								onchange={(v) =>
									updateDraft('storage.retention_days', v)}
							/>
						</div>

						<div class="field full-width">
							<div class="toggle-row">
								<div class="toggle-info">
									<span class="field-label">Auto-start on Login</span>
									<span class="field-hint">
										Launch Tuitbot automatically when you log in to your computer
									</span>
								</div>
								<button
									type="button"
									class="toggle"
									class:active={autoStartEnabled}
									onclick={toggleAutoStart}
									disabled={autoStartLoading}
									role="switch"
									aria-checked={autoStartEnabled}
									aria-label="Toggle auto-start"
								>
									<span class="toggle-track">
										<span class="toggle-thumb"></span>
									</span>
								</button>
							</div>
						</div>
					</div>
				</SettingsSection>
			</div>
		</div>
	</div>

	<!-- Save bar -->
	{#if $isDirty || showSaved || $saveError}
		<div class="save-bar" class:has-error={!!$saveError}>
			<div class="save-bar-content">
				{#if showSaved}
					<div class="save-status success">
						<Check size={16} />
						Settings saved
					</div>
				{:else if $saveError}
					<div class="save-status error">
						<AlertTriangle size={16} />
						{$saveError}
					</div>
				{:else}
					<span class="unsaved-text">You have unsaved changes</span>
				{/if}

				<div class="save-actions">
					{#if !showSaved}
						<button type="button" class="discard-btn" onclick={resetDraft} disabled={$saving}>
							<RotateCcw size={14} />
							Discard
						</button>
						<button
							type="button"
							class="save-btn"
							onclick={handleSave}
							disabled={$saving}
						>
							{#if $saving}
								Saving...
							{:else}
								<Save size={14} />
								Save Changes
							{/if}
						</button>
					{/if}
				</div>
			</div>
		</div>
	{/if}

	<!-- Dangerous change confirmation -->
	{#if showConfirm}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="modal-backdrop" onclick={() => (showConfirm = false)}>
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="modal" onclick={(e) => e.stopPropagation()}>
				<div class="modal-icon">
					<AlertTriangle size={24} />
				</div>
				<h3>Credential Change Warning</h3>
				<p>
					You're changing API credentials or LLM provider settings. This will affect
					active automation. Are you sure you want to save?
				</p>
				<div class="modal-actions">
					<button
						type="button"
						class="discard-btn"
						onclick={() => (showConfirm = false)}
					>
						Cancel
					</button>
					<button type="button" class="save-btn" onclick={doSave}>
						Save Anyway
					</button>
				</div>
			</div>
		</div>
	{/if}
{/if}

<style>
	/* ================================================================ */
	/* Layout */
	/* ================================================================ */

	.settings-layout {
		display: flex;
		gap: 24px;
	}

	.section-nav {
		width: 160px;
		flex-shrink: 0;
		position: sticky;
		top: 16px;
		align-self: flex-start;
	}

	.nav-title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-text-subtle);
		padding: 0 8px 8px;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 7px 8px;
		border: none;
		background: none;
		color: var(--color-text-muted);
		font-size: 13px;
		border-radius: 6px;
		cursor: pointer;
		text-align: left;
		transition:
			background 0.15s,
			color 0.15s;
	}

	.nav-item:hover {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.nav-item.active {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}

	.settings-content {
		flex: 1;
		min-width: 0;
		padding-bottom: 80px;
	}

	.sections {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	/* ================================================================ */
	/* Page header */
	/* ================================================================ */

	.page-header {
		margin-bottom: 24px;
	}

	h1 {
		font-size: 24px;
		font-weight: 700;
		color: var(--color-text);
		margin: 0 0 4px;
	}

	.subtitle {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0;
	}

	/* ================================================================ */
	/* Loading & error */
	/* ================================================================ */

	.loading-skeleton {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.skeleton-card {
		height: 120px;
		background: var(--color-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 8px;
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.5;
		}
	}

	.error-banner {
		padding: 16px 20px;
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-surface));
		border: 1px solid color-mix(in srgb, var(--color-danger) 30%, transparent);
		border-radius: 8px;
		color: var(--color-danger);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.error-banner p {
		margin: 0;
		font-size: 13px;
	}

	.error-banner button {
		padding: 6px 14px;
		background: var(--color-danger);
		color: white;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-size: 13px;
	}

	/* ================================================================ */
	/* Field grid */
	/* ================================================================ */

	.field-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 20px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.full-width {
		grid-column: 1 / -1;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text);
	}

	.required {
		color: var(--color-danger);
	}

	.field-error {
		font-size: 12px;
		color: var(--color-danger);
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-subtle);
	}

	.field-default {
		font-size: 11px;
		color: var(--color-text-subtle);
		font-style: italic;
	}

	/* ================================================================ */
	/* Form inputs */
	/* ================================================================ */

	.text-input,
	.textarea-input,
	.select-input,
	.time-input {
		padding: 8px 12px;
		background: var(--color-base);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		color: var(--color-text);
		font-size: 13px;
		font-family: var(--font-sans);
		outline: none;
		transition: border-color 0.15s;
	}

	.text-input:focus,
	.textarea-input:focus,
	.select-input:focus,
	.time-input:focus {
		border-color: var(--color-accent);
	}

	.text-input:disabled,
	.textarea-input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.text-input.has-error,
	.textarea-input.has-error {
		border-color: var(--color-danger);
	}

	.textarea-input {
		resize: vertical;
		min-height: 60px;
		line-height: 1.5;
	}

	.select-input {
		cursor: pointer;
		appearance: auto;
	}

	.time-input {
		width: fit-content;
	}

	.time-input.wide {
		width: 100%;
	}

	/* ================================================================ */
	/* Password input */
	/* ================================================================ */

	.password-wrapper {
		position: relative;
		display: flex;
	}

	.password-input {
		flex: 1;
		padding-right: 40px;
	}

	.password-toggle {
		position: absolute;
		right: 8px;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		background: none;
		color: var(--color-text-muted);
		cursor: pointer;
		border-radius: 4px;
		transition: color 0.15s;
	}

	.password-toggle:hover {
		color: var(--color-text);
	}

	/* ================================================================ */
	/* Toggle switch */
	/* ================================================================ */

	.toggle-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 0;
	}

	.toggle-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.toggle {
		border: none;
		background: none;
		padding: 0;
		cursor: pointer;
	}

	.toggle-track {
		display: flex;
		align-items: center;
		width: 42px;
		height: 24px;
		padding: 2px;
		background: var(--color-border);
		border-radius: 12px;
		transition: background 0.2s;
	}

	.toggle.active .toggle-track {
		background: var(--color-accent);
	}

	.toggle-thumb {
		width: 20px;
		height: 20px;
		background: white;
		border-radius: 50%;
		transition: transform 0.2s;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
	}

	.toggle.active .toggle-thumb {
		transform: translateX(18px);
	}

	/* ================================================================ */
	/* Scoring bar */
	/* ================================================================ */

	.scoring-bar-section {
		gap: 10px;
	}

	.scoring-bar-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.scoring-total {
		font-size: 13px;
		color: var(--color-text-muted);
		font-family: var(--font-mono);
	}

	.scoring-total strong {
		color: var(--color-accent);
	}

	.scoring-bar {
		display: flex;
		height: 10px;
		border-radius: 5px;
		overflow: hidden;
		gap: 1px;
	}

	.scoring-segment {
		border-radius: 2px;
		transition: flex 0.3s;
	}

	.scoring-legend {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.legend-dot {
		width: 8px;
		height: 8px;
		border-radius: 2px;
		flex-shrink: 0;
	}

	.legend-value {
		font-family: var(--font-mono);
		color: var(--color-text-subtle);
	}

	/* ================================================================ */
	/* Preferred times */
	/* ================================================================ */

	.preferred-times-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.add-btn {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		background: none;
		color: var(--color-accent);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 12px;
		cursor: pointer;
		transition:
			background 0.15s,
			border-color 0.15s;
	}

	.add-btn:hover {
		background: var(--color-surface-hover);
		border-color: var(--color-accent);
	}

	.time-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.time-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.auto-badge {
		padding: 4px 10px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
		border-radius: 4px;
		font-size: 12px;
		font-weight: 500;
	}

	.remove-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		background: none;
		color: var(--color-text-subtle);
		cursor: pointer;
		border-radius: 4px;
		transition:
			color 0.15s,
			background 0.15s;
	}

	.remove-btn:hover {
		color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	/* ================================================================ */
	/* Info banner */
	/* ================================================================ */

	.info-banner {
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-accent) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-accent) 20%, transparent);
		border-radius: 6px;
	}

	.info-banner p {
		margin: 0;
		font-size: 13px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.info-banner code {
		background: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
		padding: 1px 6px;
		border-radius: 3px;
		font-size: 12px;
		font-family: var(--font-mono);
	}

	/* ================================================================ */
	/* Save bar */
	/* ================================================================ */

	.save-bar {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		z-index: 50;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border);
		padding: 12px 24px;
		animation: slideUp 0.2s ease-out;
	}

	@keyframes slideUp {
		from {
			transform: translateY(100%);
		}
		to {
			transform: translateY(0);
		}
	}

	.save-bar-content {
		max-width: 960px;
		margin: 0 auto;
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.unsaved-text {
		font-size: 13px;
		color: var(--color-warning);
		font-weight: 500;
	}

	.save-status {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 500;
	}

	.save-status.success {
		color: var(--color-success);
	}

	.save-status.error {
		color: var(--color-danger);
	}

	.save-actions {
		display: flex;
		gap: 8px;
	}

	.discard-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 14px;
		background: none;
		color: var(--color-text-muted);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 13px;
		cursor: pointer;
		transition:
			background 0.15s,
			color 0.15s;
	}

	.discard-btn:hover:not(:disabled) {
		background: var(--color-surface-hover);
		color: var(--color-text);
	}

	.discard-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.save-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 7px 16px;
		background: var(--color-accent);
		color: white;
		border: none;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition:
			background 0.15s,
			opacity 0.15s;
	}

	.save-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* ================================================================ */
	/* Confirmation modal */
	/* ================================================================ */

	.modal-backdrop {
		position: fixed;
		inset: 0;
		z-index: 100;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.modal {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 12px;
		padding: 24px;
		max-width: 420px;
		text-align: center;
	}

	.modal-icon {
		color: var(--color-warning);
		margin-bottom: 12px;
	}

	.modal h3 {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0 0 8px;
	}

	.modal p {
		font-size: 13px;
		color: var(--color-text-muted);
		margin: 0 0 20px;
		line-height: 1.5;
	}

	.modal-actions {
		display: flex;
		gap: 8px;
		justify-content: center;
	}
</style>
