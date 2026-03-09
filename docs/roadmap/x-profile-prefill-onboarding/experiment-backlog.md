# Experiment Backlog

Experiments to optimize the onboarding and activation funnel. Each entry includes a hypothesis, the metric to watch, an implementation sketch, and priority.

## Priority Legend
- **P1**: High impact, implement next
- **P2**: Medium impact, queue after P1s
- **P3**: Low urgency, exploratory

---

## E1: Inline X Developer App Creation Guide

**Hypothesis**: An embedded step-by-step guide with screenshots reduces drop-off at the X auth step more than the current text-only instructions.

**Metric**: X auth completion rate (`onboarding:x-auth-success` / users reaching X Access step)

**Sketch**: Replace the current `<ol>` guide in XApiStep with an expandable accordion containing annotated screenshots of each Developer Portal step. Add a "I already have a Client ID" shortcut.

**Priority**: P1

## E2: Defer Claim Step to Post-Onboarding

**Hypothesis**: Moving passphrase setup out of the onboarding wizard reduces friction and improves completion rate for web/self-hosted users.

**Metric**: Onboarding completion rate for web users (`onboarding:completed` where `claimed: true`)

**Sketch**: Remove `ClaimStep` from the wizard. After first login to the dashboard, show a persistent banner prompting passphrase setup. Track `activation:claim-deferred` and `activation:claim-completed`.

**Priority**: P1

## E3: Auto-Trigger Profile Re-Analysis When LLM Added in Settings

**Hypothesis**: Users who skipped LLM during onboarding get better profiles if analysis runs automatically when they later configure an LLM.

**Metric**: Profile completeness score after LLM config in Settings

**Sketch**: In the Settings LLM section, after successful save, check if `business_profile` has low-confidence fields. If so, show "Re-analyze profile with AI?" button. Wire to existing `/api/onboarding/analyze-profile` endpoint.

**Priority**: P1

## E4: Skip Welcome Step for Returning Users

**Hypothesis**: Users who have seen the welcome screen before (e.g., abandoned a previous attempt) don't need it again; skipping saves time.

**Metric**: Step 0 -> 1 transition time; onboarding abandon rate

**Sketch**: Check `localStorage` for a `tuitbot:onboarding:seen_welcome` flag. If set, start at step 1. Set the flag when Welcome renders.

**Priority**: P2

## E5: Auto-Detect LLM from Environment

**Hypothesis**: If Ollama is running locally, pre-selecting it in the LLM step increases LLM config rate during onboarding.

**Metric**: LLM configuration rate (`onboarding:submitted` with `has_llm: true`)

**Sketch**: On LlmStep mount, fetch `http://localhost:11434/api/tags`. If it returns models, pre-select Ollama and show the available models. Fall back to manual entry on error.

**Priority**: P2

## E6: Profile Analysis Skip Timer

**Hypothesis**: Showing a "Skip" button after 10 seconds (if analysis is slow) reduces abandonment without hurting analysis utilization.

**Metric**: Analysis abandonment rate (`onboarding:analysis-error` / `onboarding:analysis-started`)

**Sketch**: In ProfileAnalysisState, start a timer on mount. After 10s, show a subtle "Taking longer than expected — Skip" link. Track `onboarding:analysis-skipped { reason: 'timeout_skip' }`.

**Priority**: P2

## E7: Show Sample Content on Welcome

**Hypothesis**: Concrete examples of what Tuitbot produces (sample tweet, sample reply) increase motivation to complete setup.

**Metric**: Step 0 -> 1 conversion (click-through from Welcome)

**Sketch**: Add a "See what Tuitbot creates" expandable section below the feature bullets in WelcomeStep, showing 2-3 example tweets/replies with placeholder branding.

**Priority**: P2

## E8: Single-Page Onboarding (No Wizard)

**Hypothesis**: A single scrollable page with all sections visible reduces perceived complexity and increases completion.

**Metric**: Overall completion rate; time to completion

**Sketch**: Render all steps as collapsible sections on one page. Auto-expand the first incomplete section. Progressive disclosure via accordions instead of step navigation.

**Priority**: P3

## E9: Progressive LLM Suggestion

**Hypothesis**: Suggesting the cheapest model first (Ollama if available, then smallest cloud model) with an upgrade path later improves LLM setup rate.

**Metric**: LLM setup rate during onboarding

**Sketch**: In LlmStep, order provider cards by cost (Ollama > Anthropic Haiku > OpenAI mini). Show estimated cost per 1K operations.

**Priority**: P3

## E10: Checklist Gamification

**Hypothesis**: A persistent progress indicator in the nav bar increases time-to-posting_ready by creating a completion motivation loop.

**Metric**: Time from `onboarding:completed` to `activation:tier-changed{to:posting_ready}`

**Sketch**: Add a small circular progress indicator next to the Tuitbot logo in the sidebar. Shows fraction of checklist items completed. Pulses briefly when a new tier is unlocked.

**Priority**: P3
