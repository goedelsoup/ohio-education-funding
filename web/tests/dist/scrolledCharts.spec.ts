/**
 * A chart sits in a `.chartwrap`, never in a table's `.scroll` (#707).
 *
 * The phone bleed (#660) gives a chart the card's side padding by selecting `.card .chartwrap`.
 * Four series charts — two on `/history`, the appropriation and the meal program — were wrapped
 * in `<div class="scroll">`, the table wrapper, so the rule never reached them: at 375 they were
 * drawn 295px wide where every other chart is 313. Nothing about the drawing was wrong, which is
 * why no chart test saw it; the wrapper is the defect, so the wrapper is what is read.
 *
 * Read as text rather than parsed, as `stripHead.spec.ts` is and for its reason (#646), with a
 * bite test on a doctored real page and a count of what it read.
 */

import { readFileSync } from "node:fs";
import { relative } from "node:path";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** Attribute order is the serializer's, so `class` need not come first. */
const SCROLL = /<div\b[^>]*\bclass="scroll"[^>]*>/g;
const DIV = /<\/?div\b[^>]*>/g;
const PLOT = /<svg\b[^>]*\bclass="plot"/;

/** How many `.scroll` wrappers a page has, and how many hold a chart and no table. */
function scrolled(html: string): { scrolls: number; charts: number } {
  let scrolls = 0;
  let charts = 0;
  for (const open of html.matchAll(SCROLL)) {
    scrolls += 1;
    // The wrapper's own extent: a chart's drawings are `div.chart-at`s, so count depth.
    const from = open.index + open[0].length;
    DIV.lastIndex = from;
    let depth = 1;
    let to = html.length;
    for (let tag = DIV.exec(html); tag; tag = DIV.exec(html)) {
      depth += tag[0].startsWith("</") ? -1 : 1;
      if (depth === 0) {
        to = tag.index;
        break;
      }
    }
    const inside = html.slice(from, to);
    if (PLOT.test(inside) && !inside.includes("<table")) charts += 1;
  }
  return { scrolls, charts };
}

describe("a chart's wrapper", () => {
  const files = pages(DIST);

  test("no chart is wrapped in a table's scroller", () => {
    const wrong: string[] = [];
    let scrolls = 0;
    for (const file of files) {
      const found = scrolled(readFileSync(file, "utf8"));
      scrolls += found.scrolls;
      if (found.charts > 0) wrong.push(`${relative(DIST, file)}: ${found.charts}`);
    }
    expect(wrong, wrong.slice(0, 40).join("\n")).toEqual([]);
    // Every district page has tables in scrollers. A sweep that read none would pass on none.
    expect(scrolls).toBeGreaterThan(1000);
  });

  test("and the sweep bites: /history with a chart put back in a scroller fails", () => {
    const history = files.find((file) => /(^|\/)history(\/index)?\.html$/.test(relative(DIST, file)));
    expect(history).toBeDefined();
    const html = readFileSync(history!, "utf8");
    expect(scrolled(html).charts).toBe(0);
    const doctored = html.replace(/<div class="chartwrap" data-chart="revenue-mix">/, '<div class="scroll">');
    expect(doctored).not.toBe(html);
    expect(scrolled(doctored).charts).toBe(1);
  });
});
