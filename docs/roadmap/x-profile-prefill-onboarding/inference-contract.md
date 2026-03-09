# Inferred Field Contract

## Overview

The `InferredProfile` type is the response contract for the profile analysis endpoint (`POST /api/onboarding/analyze-profile`). It contains per-field inference results with confidence scores and provenance tracking.

## TypeScript Contract (Frontend)

```typescript
type Confidence = "high" | "medium" | "low";

interface InferredField<T> {
  /** The inferred value. */
  value: T;
  /** How confident the inference is. */
  confidence: Confidence;
  /** Where the inference came from. */
  provenance: "bio" | "tweets" | "bio_and_tweets" | "profile_url" | "display_name" | "default";
}

interface InferredProfile {
  account_type: InferredField<"individual" | "business">;
  product_name: InferredField<string>;
  product_description: InferredField<string>;
  product_url: InferredField<string | null>;
  target_audience: InferredField<string>;
  product_keywords: InferredField<string[]>;
  industry_topics: InferredField<string[]>;
  brand_voice: InferredField<string | null>;
}
```

## Rust Contract (Backend)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    Bio,
    Tweets,
    BioAndTweets,
    ProfileUrl,
    DisplayName,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredField<T: Serialize> {
    pub value: T,
    pub confidence: Confidence,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredProfile {
    pub account_type: InferredField<String>,
    pub product_name: InferredField<String>,
    pub product_description: InferredField<String>,
    pub product_url: InferredField<Option<String>>,
    pub target_audience: InferredField<String>,
    pub product_keywords: InferredField<Vec<String>>,
    pub industry_topics: InferredField<Vec<String>>,
    pub brand_voice: InferredField<Option<String>>,
}
```

## Inference Rules Per Field

### account_type

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| Bio contains company/product indicators (CEO, founder, "we build", product names with TM/R, .com domain) | `"business"` | high | bio |
| Bio mentions job title or company name but ambiguous | `"business"` | medium | bio |
| Bio is personal, no company indicators | `"individual"` | high | bio |
| Bio is empty or too short to determine | `"individual"` | low | default |

### product_name

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| Business account: product/company name extracted from bio | Extracted name | high | bio |
| Business account: name inferred from tweet patterns | Inferred name | medium | tweets |
| Individual account: X display name | Display name | high | display_name |
| Fallback | X display name | low | display_name |

### product_description

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| Bio has clear product/person description (>20 chars) | Bio text (cleaned) | high | bio |
| Bio is short but meaningful (10-20 chars) | Bio text | medium | bio |
| Bio empty, description synthesized from tweets | Synthesized text | medium | tweets |
| Bio empty, no tweets | Empty string | low | default |

### product_url

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| X profile has URL field set | Profile URL | high | profile_url |
| URL found in bio text | Extracted URL | medium | bio |
| No URL found anywhere | `null` | low | default |

### target_audience

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| LLM identifies clear audience from bio + tweet engagement patterns | Inferred audience | high | bio_and_tweets |
| LLM identifies audience from bio only | Inferred audience | medium | bio |
| LLM identifies audience from tweets only | Inferred audience | medium | tweets |
| Cannot determine | Generic default (e.g., "Twitter users interested in [topic]") | low | default |

### product_keywords

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| 5+ keywords extracted from bio + tweets with high frequency | Keywords (3-7) | high | bio_and_tweets |
| Keywords from bio only | Keywords (2-5) | medium | bio |
| Keywords from tweets only | Keywords (3-7) | medium | tweets |
| Too few keywords found | Whatever was found (1-2) | low | bio_and_tweets |

### industry_topics

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| 3+ distinct themes identified from tweet content | Topics (3-5) | high | tweets |
| Topics from bio + limited tweets | Topics (2-4) | medium | bio_and_tweets |
| Topics from bio only (no tweets) | Topics (1-3) | low | bio |

### brand_voice

| Condition | Value | Confidence | Provenance |
|-----------|-------|------------|------------|
| 10+ tweets analyzed, clear tone pattern | One of: `"casual"`, `"balanced"`, `"professional"`, `"witty"`, `"technical"` | high | tweets |
| 5-9 tweets, some tone signal | Detected tone | medium | tweets |
| <5 tweets or ambiguous | `null` (use default "balanced") | low | default |

## LLM Prompt Structure

The inference module sends a single structured prompt to the LLM:

**System prompt:**
```
You are analyzing an X (Twitter) profile to configure a content automation tool. Extract structured profile information from the user's bio and recent tweets. Return a JSON object with the specified fields. Be specific and actionable — generic descriptions are not useful.
```

**User prompt template:**
```
Analyze this X profile and return a JSON object with the following fields.

## Profile
- Display name: {display_name}
- Username: @{username}
- Bio: {description}
- URL: {url}
- Location: {location}
- Followers: {followers_count}
- Following: {following_count}

## Recent Tweets (most recent first)
{tweets_formatted}

## Required Output (JSON)
{
  "account_type": "individual" or "business",
  "product_name": "name of the person or product",
  "product_description": "one-sentence description",
  "target_audience": "who they want to reach",
  "product_keywords": ["keyword1", "keyword2", ...],
  "industry_topics": ["topic1", "topic2", ...],
  "brand_voice": "casual" | "balanced" | "professional" | "witty" | "technical" | null
}

Rules:
- product_keywords: 3-7 terms that would find relevant tweets to reply to
- industry_topics: 2-5 broader themes for original content creation
- Keywords and topics should not overlap significantly
- If bio is empty or uninformative, rely more on tweet content
- If you cannot determine a field, use null
```

**Response parsing:**
- Parse JSON from LLM response (strip markdown fences if present)
- Map each field to the `InferredProfile` struct
- Assign confidence based on available input data and field presence (see rules above)
- Assign provenance based on which inputs contributed

## Confidence Assignment Logic

Confidence is determined post-inference based on input availability:

```
if bio.length > 20 AND tweets.length >= 10:
    base_confidence = "high"
elif bio.length > 0 OR tweets.length >= 5:
    base_confidence = "medium"
else:
    base_confidence = "low"
```

Individual fields may be downgraded from the base:
- `product_url`: downgraded to "low" if not found in profile URL or bio
- `brand_voice`: downgraded if fewer than 5 tweets
- `target_audience`: downgraded if neither bio nor tweets contain audience signals
- Any field where LLM returned null: set to "low" confidence with default provenance

## Error Responses

```typescript
// Full success
{ status: "ok", profile: InferredProfile }

// Partial success (some fields failed)
{ status: "partial", profile: InferredProfile, warnings: string[] }

// X API failure
{ status: "x_api_error", error: string }

// LLM failure
{ status: "llm_error", error: string }
```

On any error, the frontend falls back to manual entry (current BusinessStep behavior).

## Future Fields (Not in MVP)

These fields exist in `BusinessProfile` but are excluded from initial inference to keep the prompt focused:

- `content_pillars: InferredField<string[]>` — requires deeper content analysis
- `persona_opinions: InferredField<string[]>` — requires opinion extraction from tweets
- `persona_experiences: InferredField<string[]>` — requires biographical analysis
- `reply_style: InferredField<string>` — requires reply-specific tweet analysis
- `content_style: InferredField<string>` — requires content-type classification

These can be added in later iterations after validating the core inference quality.
