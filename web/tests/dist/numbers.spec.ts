/**
 * Every figure cell in the build, written the way `format.ts` writes a number.
 *
 * # Why this reads the artefact
 *
 * The HTML is built in template literals, so a number reaches the page through whatever the line
 * that interpolated it happened to call. `format.ts` normalises the minus to U+2212 and groups
 * thousands; `toFixed` does neither, and there were ninety calls to it in the renderers. Before
 * #503 the build carried `-0.03` in the growth rows of 326 outcome pages and `-0.662` on
 * /statewide, beside cells that read `−$1,318`, and Columbus's Category 2 pupils as `8376.39`.
 *
 * `astro check` cannot see inside a string, and a source rule can only name the spellings it
 * already knows about. The cells are the thing a reader sees, so the cells are what this holds.
 */

import { readFileSync } from "node:fs";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** The text a `.tnum` element opens with — up to its first child tag, which is enough for a figure. */
const TNUM = /<(?:td|th|span|div)\b[^>]*\bclass="[^"]*\btnum\b[^"]*"[^>]*>([^<]*)/g;

/** A cell that cites the Revised Code carries section numbers, which are not quantities. */
const CITATION = /R\.C\.|§/;

describe("figure cells", () => {
  const cells: { page: string; text: string }[] = [];
  for (const file of pages()) {
    const page = readFileSync(file, "utf8");
    for (const match of page.matchAll(TNUM)) {
      const text = (match[1] ?? "").trim();
      if (text) cells.push({ page: file.slice(DIST.length + 1), text });
    }
  }

  test("the sweep reads the figure cells at all", () => {
    /*
     * Without this the two rules below pass on a build with no `.tnum` in it, or on a pattern that
     * stopped matching the markup. 252,986 cells carried text when this was written.
     */
    expect(cells.length).toBeGreaterThan(200_000);
    expect(cells.filter((c) => c.text.includes("−")).length, "some cell is negative").toBeGreaterThan(1_000);
  });

  test("no figure cell writes its minus as a hyphen", () => {
    const hyphens = cells.filter((c) => /(^|[\s($])-\d/.test(c.text));
    expect(
      hyphens.slice(0, 5).map((c) => `${c.page}: ${c.text}`),
      "format the value through `fixed`, `signed` or `money` in src/lib/format.ts",
    ).toEqual([]);
  });

  test("no figure cell leaves a quantity over a thousand ungrouped", () => {
    /*
     * A decimal quantity only: a four-digit run with no point is a year, and a six-digit one is an
     * IRN or a line item's number, and neither takes a separator.
     */
    const ungrouped = cells.filter((c) => !CITATION.test(c.text) && /(?<![\d,.])\d{4,}\.\d/.test(c.text));
    expect(
      ungrouped.slice(0, 5).map((c) => `${c.page}: ${c.text}`),
      "format the value through `fixed` or `count` in src/lib/format.ts",
    ).toEqual([]);
  });
});
