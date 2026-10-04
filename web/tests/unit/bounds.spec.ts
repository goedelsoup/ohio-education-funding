/**
 * `/bounds`'s census card, read off the real series manifest.
 *
 * The card's prose and its chart are both derived from `project/bounds-census`, so what is
 * asserted here is the derivation: that the ends the dek argues the log scale from are ends a log
 * scale can draw, that a row out of fewer districts says so, and that the names the crate writes
 * fit the gutter the chart sizes for them.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { census, renderCensus, renderNulls } from "../../src/lib/bounds.ts";
import { RANK_LABEL_CHARS, rankSpec, WIDTHS } from "../../src/lib/plot/spec.ts";
import { renderToString } from "../../src/lib/plot/ssr.ts";

const c = census();

test("the census dek argues the log scale from two ends a log scale can draw", () => {
  /*
   * #710. "The two ends of this census are 554 districts and 0" — the one value a log axis has no
   * position for, which is why the chart has a floor row at all. The description said the same.
   */
  const text = parseHTML(`<div>${renderCensus(c)}</div>`).document.querySelector("p.note")!.textContent!;
  const ends = /two ends of this census are\s+([\d,]+) districts and\s+([\d,]+)/.exec(text);
  expect(ends, text).not.toBeNull();
  for (const end of ends!.slice(1)) expect(Number(end.replace(/,/g, ""))).toBeGreaterThan(0);
  expect(text).not.toMatch(/about forty/);
  // And the ranking is stated, since nothing else on the card says which way it runs.
  expect(text).toMatch(/Ranked by districts decided, most first/);
});

test("every census row out of fewer districts than the whole carries its population", () => {
  // #710: seven of the thirty-eight, the clawback's 22 of 43 among them.
  const fewer = c.rows.filter((row) => row.of !== undefined && row.of !== c.whole);
  expect(fewer.length).toBeGreaterThan(0);
  c.rows.forEach((row, i) => {
    const note = c.ranks[i]!.note;
    if (fewer.includes(row)) expect(note, row.label).toBe(`of ${row.of}`);
    else expect(note, row.label).toBeUndefined();
  });
});

test("the cannot-bind note describes the dot the chart draws, not a bar", () => {
  // #710. The census is a dot chart, and the zero row is a dot at the floor.
  const html = renderNulls(c);
  expect(html).not.toMatch(/a bar of\s+length nought/);
  if (c.cannotBind.length > 0) expect(html).toMatch(/at the floor of the\s+scale/);
});

test("the crate's short-name budget is the gutter's, and every census name fits it on one line", () => {
  /*
   * #709. The Rust test held names to 40 characters against a gutter that holds 34, so 17 of the
   * 38 rows wrapped and the chart came out 1,425px tall at 1280. The two numbers are written in
   * two languages, so this is what holds them together.
   */
  const rust = readFileSync(
    join(
      import.meta.dirname,
      "../../../crates/project/tests/the_bounds_that_cannot_bind_and_the_ones_that_never_have.rs",
    ),
    "utf8",
  );
  const gutter = /const GUTTER: usize = (\d+);/.exec(rust);
  expect(gutter, "the Rust test states its gutter").not.toBeNull();
  expect(Number(gutter![1])).toBe(RANK_LABEL_CHARS);

  for (const row of c.rows) expect(row.label.length, row.label).toBeLessThanOrEqual(RANK_LABEL_CHARS);

  const drawn = renderToString(
    (width) => rankSpec(c.ranks, { label: "Districts the bound decides", format: String }, { width }),
    { label: "census", description: "census" },
  );
  const wide = /<svg\b[\s\S]*?<\/svg>/g;
  const drawings = [...drawn.matchAll(wide)].map((m) => m[0]);
  expect(drawings).toHaveLength(Object.keys(WIDTHS).length);
  // The wide drawing is the last: `renderToString` lays the widths out narrowest first.
  const { document } = parseHTML(`<div>${drawings.at(-1)}</div>`);
  const wrapped = [...document.querySelectorAll("g.rank-label text")]
    .filter((t) => t.querySelectorAll("tspan").length > 0)
    .map((t) => t.textContent);
  expect(wrapped).toEqual([]);
});
