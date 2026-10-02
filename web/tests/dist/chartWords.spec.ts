/**
 * The words every built chart prints, against the list it may not (#658).
 *
 * `BANNED` in `src/lib/chartWords.ts` is the list and says why each is on it. This reads the
 * markup a reader is served rather than the renderers, because a chart's text is assembled from
 * the feed, the series manifest and Rust's bound names, and no one source file holds all of it.
 * Charts drawn in the browser are not in this markup; `tests/unit/chartWords.spec.ts` holds the
 * reach chart's axes instead.
 */

import { readFileSync } from "node:fs";

import { describe, expect, test } from "vitest";

import { banned } from "../../src/lib/chartWords.ts";
import { DIST, pages } from "./artefact.ts";

describe("chart words", () => {
  const texts: { page: string; text: string }[] = [];
  for (const file of pages()) {
    const page = readFileSync(file, "utf8");
    for (const svg of page.match(/<svg\b[^>]*class="plot[^"]*"[^>]*>[\s\S]*?<\/svg>/g) ?? []) {
      for (const t of svg.matchAll(/<text\b[^>]*>([\s\S]*?)<\/text>/g)) {
        const text = t[1]!.replace(/<[^>]+>/g, " ").replace(/&#160;|&nbsp;/g, " ").replace(/\s+/g, " ").trim();
        if (text) texts.push({ page: file.slice(DIST.length + 1), text });
      }
    }
  }

  test("the sweep reads chart text at all", () => {
    // Without this the rule below passes on a build whose markup stopped matching.
    expect(texts.length).toBeGreaterThan(10_000);
    expect(texts.some((t) => t.page === "bounds.html")).toBe(true);
  });

  test("no built chart prints a banned word", () => {
    const found = new Set(texts.flatMap((t) => banned(t.text).map((b) => `${t.page}: ${b}`)));
    expect([...found].slice(0, 20)).toEqual([]);
  });
});
