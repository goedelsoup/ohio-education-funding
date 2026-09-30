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
import { relative } from "node:path";

import { parseHTML } from "linkedom";
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

/**
 * Every figure in the visible text, not only the ones in a `.tnum` cell.
 *
 * # What the cell sweep missed
 *
 * A figure in a sentence is not in a cell, and #574 found three kinds of them. Correlations went
 * out through `toFixed(3)` into prose on `/`, `/outcomes` and the Outcome tab, so "-0.846" sat on
 * the page beside "−0.846" from `format.ts`. `/data`'s checkpoint table wrote "7.281B" with no
 * dollar sign where `/statewide` wrote "$7.28B". And "$525m" was typed into a district card, a
 * third money style beside `millions()`'s "$812.5M".
 *
 * # What is exempt, and why
 *
 * `/wiki/**`. It renders the corpus, and the corpus's figures are matched by value by the figure
 * bindings — "$812m" and a table cell reading "-0.024" are what a node says and what its binding
 * checks. Rewriting them would be editing a claim to suit a stylesheet. The same exemption
 * `plainCopy.spec.ts` makes for the same reason.
 *
 * Text a script writes after load is invisible here, as it is to every sweep of `dist/`.
 */
describe("figures in prose", () => {
  const EXEMPT = /^wiki(?:\.html|\/)/;

  const PATTERNS: [string, RegExp][] = [
    /*
     * A hyphen standing where a minus would: not after a word character, another dash, a point or
     * a slash, which is a range ("K-12", "2019-20"), a compound or a path.
     */
    ["a hyphen-minus", /(?<![\w\-–−./#&])-\d[\d,]*(?:\.\d+)?/g],
    ["billions without a currency sign", /(?<![$\d.,])\d+\.\d+B\b/g],
    ["a lowercase money suffix", /\$[\d,]+(?:\.\d+)?(?:m|b|bn|k)\b/g],
  ];

  /**
   * The page's visible text: the title and the body, without scripts, styles or templates.
   *
   * With a space at every cell and block boundary, which `textContent` does not put there: a row
   * reading `<th>H.B. 110</th><td>7.281B</td>` is "H.B. 1107.281B" without it, and the digit
   * before the figure hid the defect this sweep was written for.
   */
  function visibleText(file: string): string {
    const html = readFileSync(file, "utf8").replace(/<\/(?:td|th|p|li|dt|dd|div|h[1-6]|caption)>/g, " $&");
    const { document } = parseHTML(html);
    for (const node of document.querySelectorAll("script, style, template")) node.remove();
    const title = document.querySelector("title")?.textContent ?? "";
    return `${title}\n${document.body?.textContent ?? ""}`.replace(/\s+/g, " ");
  }

  const scanned = pages().filter((file) => !EXEMPT.test(relative(DIST, file)));

  test("the scan reaches the routes it is about", () => {
    expect(scanned.length).toBeGreaterThan(2000);
    const names = new Set(scanned.map((file) => relative(DIST, file)));
    for (const route of ["index.html", "outcomes.html", "data.html", "legislation.html", "statewide.html"]) {
      expect(names.has(route), route).toBe(true);
    }
  });

  test("the guard fires on the defect it exists for", () => {
    // Doctored lines from the pages #574 fixed: a scan that passes everywhere measures nothing
    // unless it can be shown to fail somewhere.
    const fires = (text: string) => PATTERNS.some(([, re]) => new RegExp(re.source).test(text));
    expect(fires("Poverty explains Ohio's attainment measure at -0.846. Every other")).toBe(true);
    expect(fires("H.B. 110 7.281B 312 297")).toBe(true);
    expect(fires("$525m distributed on a curve like that")).toBe(true);
    // And holds its fire on what is right, and on the ranges and compounds that look like it.
    expect(fires("at −0.846, $7.28B and $812.5M, grades K-12, FY2002–FY2003, 2024-25")).toBe(false);
  });

  test("no page outside /wiki writes a figure outside format.ts's shapes", () => {
    // Collected by match, so a string repeated on 609 district pages is reported once.
    const found = new Map<string, { page: string; pages: number }>();
    for (const file of scanned) {
      const text = visibleText(file);
      for (const [kind, re] of PATTERNS) {
        for (const match of text.matchAll(re)) {
          const at = match.index ?? 0;
          const key = `${kind}: "${match[0]}" in "…${text.slice(Math.max(0, at - 50), at + 40)}…"`;
          const seen = found.get(key);
          if (seen) seen.pages += 1;
          else found.set(key, { page: relative(DIST, file), pages: 1 });
        }
      }
    }
    const report = [...found].map(([key, { page, pages }]) => `${key} — ${page}, ${pages} page(s)`);
    expect(report, "format the value through `fixed`, `signed` or `millions` in src/lib/format.ts").toEqual([]);
  });
});
