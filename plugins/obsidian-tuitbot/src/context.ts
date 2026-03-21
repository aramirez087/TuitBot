// Pure helper functions for payload construction.
// No Obsidian imports — safe to test in Node.js directly.

// ---------------------------------------------------------------------------
// Heading types (mirrors the subset of obsidian.HeadingCache we use)
// ---------------------------------------------------------------------------

export interface HeadingInfo {
  level: number;
  heading: string;
  line: number;
}

// ---------------------------------------------------------------------------
// Payload types
// ---------------------------------------------------------------------------

export interface GhostwriterPayload {
  vault_name: string;
  file_path: string;
  selected_text: string;
  heading_context: string | null;
  selection_start_line: number;
  selection_end_line: number;
  note_title: string | null;
  frontmatter_tags: string[] | null;
}

export interface SendSelectionResponse {
  status: string;
  session_id: string;
  composer_url: string;
}

// ---------------------------------------------------------------------------
// Heading resolution
// ---------------------------------------------------------------------------

/**
 * Build the heading context string from an array of headings and a cursor line.
 * Returns the deepest heading path (e.g. "## Setup > ### Config") or null if
 * no heading precedes the cursor.
 */
export function resolveHeadingPath(
  headings: HeadingInfo[],
  cursorLine: number,
): string | null {
  if (headings.length === 0) return null;

  const preceding = headings.filter((h) => h.line <= cursorLine);
  if (preceding.length === 0) return null;

  // Build a heading hierarchy path. Walk backwards from the deepest heading,
  // collecting the chain where each parent has a strictly smaller level.
  const chain: HeadingInfo[] = [];
  let maxLevel = Infinity;

  for (let i = preceding.length - 1; i >= 0; i--) {
    const h = preceding[i];
    if (h.level < maxLevel) {
      chain.unshift(h);
      maxLevel = h.level;
      if (maxLevel === 1) break;
    }
  }

  if (chain.length === 0) return null;

  return chain.map((h) => `${"#".repeat(h.level)} ${h.heading}`).join(" > ");
}

// ---------------------------------------------------------------------------
// Block extraction
// ---------------------------------------------------------------------------

/**
 * Extract the contiguous block of non-blank lines surrounding a given line.
 */
export function extractBlock(
  lines: string[],
  cursorLine: number,
): { text: string; startLine: number; endLine: number } {
  // If the cursor line itself is blank, return it as-is (empty block).
  if (lines[cursorLine].trim() === "") {
    return { text: "", startLine: cursorLine, endLine: cursorLine };
  }

  let start = cursorLine;
  let end = cursorLine;

  while (start > 0 && lines[start - 1].trim() !== "") {
    start--;
  }
  while (end < lines.length - 1 && lines[end + 1].trim() !== "") {
    end++;
  }

  const text = lines.slice(start, end + 1).join("\n");
  return { text, startLine: start, endLine: end };
}
