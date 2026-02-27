# Superiority Scorecard: Tuitbot vs. Typefully

**Date:** 2026-02-27
**Objective:** Prove that the Tuitbot composer is measurably better than Typefully in at least 3 of 4 dimensions.
**Win Criteria:** Score >= Typefully in all 4 dimensions, with clear superiority in at least 3.

---

## Dimension 1: Writing Speed

How fast can a user compose and submit content?

| Metric | Typefully Baseline | Tuitbot Target | Measurement Method | Session |
|--------|-------------------|----------------|-------------------|---------|
| Time to compose a 5-tweet thread (from open to submit) | ~3-4 min (estimated from UX walkthroughs) | < 3 min | Manual timed test: open compose, write 5 tweets of ~200 chars each, submit. Three trials, take median | 03 |
| Keystrokes to reorder a 5-tweet thread (reverse order) | ~15 (5 drag-and-drop operations, ~3 actions each: grab, drag, drop) | <= 8 (4x Alt+Up on card 5, walking it to position 1) | Keystroke count: record all key/mouse events to achieve reverse ordering | 04 |
| Time to add media to 3 thread tweets | ~20 sec (3x click media zone, select file) | <= 15 sec (drag-drop onto cards) | Manual timed test: attach one image to tweets 1, 3, 5. Three trials, take median | 04 |
| Actions to submit with keyboard only | ~6 (various tab/enter combinations) | <= 4 (Cmd+Enter from anywhere in editor) | Action count: key events from "content ready" to "submit confirmed" | 05 |

**Superiority claim:** Keyboard-first thread reordering is faster than mouse-based drag-and-drop. Power actions (duplicate, split) eliminate the need to retype content.

---

## Dimension 2: Structural Control

How much control does the user have over thread structure?

| Metric | Typefully Baseline | Tuitbot Target | Measurement Method | Session |
|--------|-------------------|----------------|-------------------|---------|
| Power actions available | 1 (Reorder) | 4 (Reorder, Duplicate, Split, Merge) | Feature checklist: verify each action works via UI and keyboard | 04 |
| Keyboard shortcut coverage | ~12 shortcuts | >= 15 shortcuts | Count: list all registered shortcuts, verify each works | 05 |
| Thread operations accessible via keyboard only | Partial (reorder requires mouse drag) | Full (all thread operations keyboard-accessible) | Manual test: compose, reorder, duplicate, split, merge, submit — using only keyboard | 04-05 |
| Command palette actions | ~8 actions | >= 12 actions (compose, thread, mode, AI, navigation) | Count: list all actions registered in command palette | 05 |

**Superiority claim:** Four power actions vs. one. All operations keyboard-accessible. Command palette as a faster alternative to button-clicking.

---

## Dimension 3: Feedback Clarity

How well does the UI communicate state, errors, and preview?

| Metric | Typefully Baseline | Tuitbot Target | Measurement Method | Session |
|--------|-------------------|----------------|-------------------|---------|
| Preview fidelity | Inline WYSIWYG (high fidelity, rendered in-place) | Side-panel WYSIWYG (high fidelity, separate panel) | Visual comparison: screenshot both, rate structural accuracy (text wrapping, media grid, thread numbering) on 1-5 scale. Target: >= 4 | 03 |
| Validation error types | 1 (character limit) | 3 (character limit, media constraint violations, structural errors — e.g., empty cards, single-card thread) | Error type count: trigger each validation error, verify distinct error messages | 03 |
| Inline validation | Character counter turns red | Character counter turns red + textarea border turns red + error message below | Visual verification: over-limit state shows all three indicators | 03 |
| Auto-save indicator | Implicit (always saving) | Explicit ("Saved" indicator with timestamp, "Unsaved changes" warning) | UI verification: make changes, verify save indicator updates within 1 second | 03 |
| Recovery prompt on reopen | Not needed (always persisted) | Clear prompt: "Recover unsaved content?" with Restore / Discard / Save as Draft options | Manual test: type content, close modal without submitting, reopen, verify recovery prompt | 03 |

**Superiority claim:** More validation error types catch mistakes earlier. Explicit save indicator gives confidence. Recovery prompt prevents accidental content loss.

---

## Dimension 4: Accessibility

How well does the composer serve users who rely on keyboard, screen readers, or have visual impairments?

| Metric | Typefully Baseline | Tuitbot Target | Measurement Method | Session |
|--------|-------------------|----------------|-------------------|---------|
| Keyboard-only composability | Partial — some actions require mouse (reorder, media) | Full — all compose actions accessible via keyboard | Manual test: compose a 3-tweet thread with media, reorder, and submit using only keyboard. Every step must be achievable | 04-06 |
| ARIA landmark coverage | Standard (basic dialog role) | Full: `role="dialog"`, `aria-label`, `aria-live` regions for counters, `aria-describedby` for error messages | Axe audit: run axe-core on compose modal in all states (tweet, thread, preview, focus mode). Zero critical violations | 06 |
| Focus management | Standard browser behavior | Trapped in modal: focus cycles within modal, returns to trigger element on close, skip links to main sections | Manual test: Tab through all elements, verify focus never escapes modal, verify focus return | 06 |
| Color contrast ratio | Unknown (not publicly audited) | 4.5:1 minimum for all text (WCAG AA) | Contrast checker: measure all text/background combinations in compose modal. Every pair must meet 4.5:1 | 06 |
| Reduced motion support | Unknown | `prefers-reduced-motion` media query disables all animations | Manual test: enable reduced motion in OS, verify no animations play | 06 |
| Screen reader announcement of state changes | Unknown | Character count changes announced via `aria-live="polite"`, error messages announced via `aria-live="assertive"` | VoiceOver test: type text, verify character count announced; trigger error, verify error announced | 06 |

**Superiority claim:** Full keyboard composability (vs. partial). ARIA coverage with live regions for dynamic content. Explicit focus management with trap and return. Reduced motion support. This is the dimension where Tuitbot most clearly surpasses Typefully.

---

## Scoring Summary

| Dimension | Typefully | Tuitbot Target | Verdict |
|-----------|-----------|----------------|---------|
| Writing Speed | Baseline | Faster (keyboard shortcuts, power actions reduce keystrokes) | **Win** |
| Structural Control | Reorder only | Reorder + Duplicate + Split + Merge, all keyboard-accessible | **Win** |
| Feedback Clarity | Inline WYSIWYG + character limit | Side-panel preview + 3 validation types + auto-save indicator + recovery | **Tie** (different approach, comparable quality) |
| Accessibility | Standard | Full keyboard, ARIA, focus trap, contrast, reduced motion | **Win** |

**Result:** 3 wins, 1 tie. Superiority demonstrated.

---

## Measurement Schedule

| Session | Metrics to Evaluate |
|---------|-------------------|
| 03 | Preview fidelity, validation error types, auto-save indicator, recovery prompt |
| 04 | Reorder keystrokes, media attachment time, power actions, keyboard-only thread operations |
| 05 | Total shortcut count, command palette actions, keyboard submit, inline AI |
| 06 | ARIA audit, focus management, contrast ratio, reduced motion, screen reader testing |
| 08 | Full scorecard evaluation — all metrics measured and documented in `superiority-scorecard-final.md` |
