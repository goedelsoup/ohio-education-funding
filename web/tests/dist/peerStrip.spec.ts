/**
 * The outcomes peer strip labels the number its lead-in compares against (#704).
 *
 * The district Outcome tab leads "Columbus City School District scores 13.5 points below the median
 * of its poverty fifth: 60.7 against 74.3", and the strip under it labelled Ohio's median and the
 * district's name. The 74.3 was the box's rule, unlabelled, so the comparison the sentence makes
 * was between a number in bold and a line nobody had named.
 *
 * Read off the bytes, as `stripHead.spec.ts` is. Of the sentence's numbers, the two it compares
 * are held: the fifth's median is printed by a `.dist-median-label` in every drawing of the strip,
 * and the district's own value by the strip head above it (#654). The gap is their difference and
 * is the third tile's to state, not the drawing's.
 */

import { readFileSync } from "node:fs";
import { relative } from "node:path";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** The bold lead-in: a value against a median, or level with one. */
const LEAD = /<strong>[^<]*? scores (?:[\d.]+ points (?:below|above) the median of its poverty fifth: ([\d.]+) against ([\d.]+)|level with the median of its poverty fifth, ([\d.]+))\.<\/strong>/;
const HEAD = /<div class="strip-head"><span>Performance Index<\/span>\s*<strong class="tnum">([\d.]+)/;
const LABEL = /<g\b[^>]*\bclass="dist-median-label"[^>]*>\s*<text\b[^>]*>([^<]*)<\/text>/g;

/** What a page's peer card fails to say, empty where it says it all. */
function unsaid(html: string): string[] {
  const card = html.slice(html.indexOf('id="comparable-poverty"'));
  const lead = LEAD.exec(card);
  if (!lead) return [];
  const [, value, against, level] = lead;
  const median = against ?? level!;
  const strip = card.slice(card.indexOf('data-chart="peer-group"'));
  const labels = [...strip.slice(0, strip.indexOf("</div>")).matchAll(LABEL)].map((m) => m[1]!);
  const gaps: string[] = [];
  if (labels.length === 0 || !labels.every((l) => l.includes(median))) {
    gaps.push(`median ${median} not in its labels ${JSON.stringify(labels)}`);
  }
  const head = HEAD.exec(card)?.[1];
  if (value != null && head !== value) gaps.push(`value ${value} not in the strip head (${head})`);
  return gaps;
}

describe("the outcomes peer strip", () => {
  const files = pages(DIST).filter((file) => /district\/[^/]+\/outcome\.html$/.test(file));

  test("labels the median its lead-in compares against, and heads the strip with the value", () => {
    const missing: string[] = [];
    let led = 0;
    for (const file of files) {
      const html = readFileSync(file, "utf8");
      if (LEAD.test(html)) led += 1;
      const gaps = unsaid(html);
      if (gaps.length > 0) missing.push(`${relative(DIST, file)}: ${gaps.join("; ")}`);
    }
    expect(missing, missing.slice(0, 40).join("\n")).toEqual([]);
    // Most of Ohio's 607 districts have a score and a poverty fifth. A sweep that found no lead-in
    // would pass on none.
    expect(led).toBeGreaterThan(500);
  });

  test("and the sweep bites: Columbus with its median label removed fails", () => {
    const columbus = files.find((file) => /district\/043802\/outcome\.html$/.test(file));
    expect(columbus).toBeDefined();
    const html = readFileSync(columbus!, "utf8");
    expect(unsaid(html)).toEqual([]);
    const doctored = html.replaceAll('class="dist-median-label"', 'class="dist-unnamed"');
    expect(unsaid(doctored)).toHaveLength(1);
  });
});
