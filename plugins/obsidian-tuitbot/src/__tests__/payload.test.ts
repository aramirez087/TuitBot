import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { resolveHeadingPath, extractBlock, type HeadingInfo } from "../context.js";

// ---------------------------------------------------------------------------
// resolveHeadingPath
// ---------------------------------------------------------------------------

function h(level: number, heading: string, line: number): HeadingInfo {
  return { level, heading, line };
}

describe("resolveHeadingPath", () => {
  it("returns null when there are no headings", () => {
    assert.equal(resolveHeadingPath([], 10), null);
  });

  it("returns null when cursor is before all headings", () => {
    assert.equal(resolveHeadingPath([h(1, "Title", 5)], 2), null);
  });

  it("returns single heading when cursor is after one heading", () => {
    assert.equal(resolveHeadingPath([h(2, "Setup", 3)], 10), "## Setup");
  });

  it("returns heading path for nested headings", () => {
    const headings = [h(1, "Title", 0), h(2, "Setup", 5), h(3, "Config", 10)];
    assert.equal(
      resolveHeadingPath(headings, 15),
      "# Title > ## Setup > ### Config",
    );
  });

  it("skips sibling headings and builds correct chain", () => {
    const headings = [
      h(1, "Title", 0),
      h(2, "Introduction", 5),
      h(2, "Setup", 15),
      h(3, "Dependencies", 20),
    ];
    assert.equal(
      resolveHeadingPath(headings, 25),
      "# Title > ## Setup > ### Dependencies",
    );
  });

  it("handles cursor exactly on a heading line", () => {
    assert.equal(resolveHeadingPath([h(2, "Setup", 5)], 5), "## Setup");
  });

  it("returns deepest heading when multiple same-level exist", () => {
    const headings = [h(2, "Part One", 0), h(2, "Part Two", 10), h(2, "Part Three", 20)];
    assert.equal(resolveHeadingPath(headings, 25), "## Part Three");
  });

  it("handles deep nesting (4 levels)", () => {
    const headings = [
      h(1, "Root", 0),
      h(2, "Chapter", 5),
      h(3, "Section", 10),
      h(4, "Subsection", 15),
    ];
    assert.equal(
      resolveHeadingPath(headings, 20),
      "# Root > ## Chapter > ### Section > #### Subsection",
    );
  });
});

// ---------------------------------------------------------------------------
// extractBlock
// ---------------------------------------------------------------------------

describe("extractBlock", () => {
  it("extracts a single-line block", () => {
    const result = extractBlock(["", "hello world", ""], 1);
    assert.equal(result.text, "hello world");
    assert.equal(result.startLine, 1);
    assert.equal(result.endLine, 1);
  });

  it("extracts a multi-line paragraph", () => {
    const lines = ["", "first line", "second line", "third line", ""];
    const result = extractBlock(lines, 2);
    assert.equal(result.text, "first line\nsecond line\nthird line");
    assert.equal(result.startLine, 1);
    assert.equal(result.endLine, 3);
  });

  it("handles block at start of document", () => {
    const result = extractBlock(["first line", "second line", "", "other"], 0);
    assert.equal(result.text, "first line\nsecond line");
    assert.equal(result.startLine, 0);
    assert.equal(result.endLine, 1);
  });

  it("handles block at end of document", () => {
    const result = extractBlock(["other", "", "last line", "very last"], 3);
    assert.equal(result.text, "last line\nvery last");
    assert.equal(result.startLine, 2);
    assert.equal(result.endLine, 3);
  });

  it("returns empty text for cursor on blank line", () => {
    const result = extractBlock(["content", "", "more content"], 1);
    assert.equal(result.text, "");
    assert.equal(result.startLine, 1);
    assert.equal(result.endLine, 1);
  });

  it("handles single line document", () => {
    const result = extractBlock(["only line"], 0);
    assert.equal(result.text, "only line");
    assert.equal(result.startLine, 0);
    assert.equal(result.endLine, 0);
  });

  it("stops at blank lines as block boundaries", () => {
    const lines = [
      "block one line 1",
      "block one line 2",
      "",
      "block two line 1",
      "block two line 2",
      "",
      "block three",
    ];
    const result = extractBlock(lines, 3);
    assert.equal(result.text, "block two line 1\nblock two line 2");
    assert.equal(result.startLine, 3);
    assert.equal(result.endLine, 4);
  });
});
