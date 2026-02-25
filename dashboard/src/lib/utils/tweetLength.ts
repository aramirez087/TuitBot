/**
 * URL-aware tweet length calculation.
 *
 * Twitter/X wraps every URL in a t.co short link (always 23 characters).
 * This module mirrors the Rust `content::length` module so the client
 * validates the same way the server does.
 */

/** Length of a t.co shortened URL on X. */
const TCO_URL_LENGTH = 23;

/** Maximum characters allowed in a single tweet. */
export const MAX_TWEET_CHARS = 280;

/**
 * Regex matching URLs that X will wrap in t.co links.
 *
 * Matches two patterns:
 * 1. Protocol URLs: `https?://...`
 * 2. Bare domains with common TLDs, optional path
 */
const URL_REGEX =
	/https?:\/\/[^\s)>\]]+|\b[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]*[a-zA-Z0-9])?)*\.(?:com|org|net|edu|gov|io|co|dev|app|me|info|biz|xyz|ai|tech|so|to|cc|gg|tv|fm|ly)(?:\/[^\s)>\]]*)?/g;

/**
 * Calculate the weighted length of a tweet accounting for t.co URL wrapping.
 *
 * Every URL (protocol or bare domain) is counted as 23 characters
 * regardless of its actual length.
 */
export function tweetWeightedLen(text: string): number {
	let length = text.length;
	// Reset lastIndex since the regex has the global flag.
	URL_REGEX.lastIndex = 0;
	let match;
	while ((match = URL_REGEX.exec(text)) !== null) {
		length = length - match[0].length + TCO_URL_LENGTH;
	}
	return length;
}

/**
 * Check if text is within the tweet character limit, accounting for t.co URLs.
 */
export function validateTweetLength(text: string, maxChars: number = MAX_TWEET_CHARS): boolean {
	return tweetWeightedLen(text) <= maxChars;
}
