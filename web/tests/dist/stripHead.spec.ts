/**
 * A strip that marks a subject states the subject's value above it (#654).
 *
 * Three pages draw `distributionSpec` with a marker, and only the district dashboard printed the
 * marked value: `Strip.astro`'s `.strip-head` said "$405,350 · 91st percentile", while the county
 * strip drew Franklin and the outcomes strip drew Columbus as a rule and nothing else. The value
 * is what a reader repeats; the strip is what makes it mean something.
 *
 * The rule read off the bytes: every `.chartwrap` whose drawing carries a `.dist-marker` is
 * immediately preceded by a `div.strip-head`. Read as text rather than parsed — a full parse of
 * every page is most of what this project costs (#646) — so it carries a bite test on a doctored
 * real page and a count of what it found.
 */

import { readFileSync } from "node:fs";
import { relative } from "node:path";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

/** Attribute order is the serializer's: a wrapper given its own address (#616) opens `<div id=… class=…`. */
const CHARTWRAP = /<div\b[^>]*\bclass="chartwrap"[^>]*>/g;
const MARKER = /<g\b[^>]*\bclass="dist-marker"/;
/** A `.strip-head` ending exactly where the chart begins. It holds no `<div>` of its own. */
const HEAD_BEFORE = /<div class="strip-head"[^>]*>(?:(?!<\/?div\b)[\s\S])*<\/div>\s*$/;

/** How many marked strips a page draws, and how many of them have no head before them. */
function strips(html: string): { marked: number; headless: number } {
  let marked = 0;
  let headless = 0;
  const opens = [...html.matchAll(CHARTWRAP)];
  opens.forEach((open, i) => {
    const body = html.slice(open.index, opens[i + 1]?.index ?? html.length);
    // A chart's drawings hold no `<div>`, so the wrapper closes at the first `</div>` after it.
    const inside = body.slice(0, body.indexOf("</div>"));
    if (!MARKER.test(inside)) return;
    marked += 1;
    if (!HEAD_BEFORE.test(html.slice(Math.max(0, open.index - 2000), open.index))) headless += 1;
  });
  return { marked, headless };
}

describe("a marked strip's head", () => {
  const files = pages(DIST).filter((file) => readFileSync(file, "utf8").includes('class="dist-marker"'));

  test("every strip that marks a subject states its value above it", () => {
    const missing: string[] = [];
    let marked = 0;
    for (const file of files) {
      const found = strips(readFileSync(file, "utf8"));
      marked += found.marked;
      if (found.headless > 0) missing.push(`${relative(DIST, file)}: ${found.headless}`);
    }
    expect(missing, missing.slice(0, 40).join("\n")).toEqual([]);
    // District dashboards (two each), counties and the outcomes peer strips. A sweep that found
    // none would pass on none.
    expect(marked).toBeGreaterThan(1800);
  });

  test("and the sweep bites: a county page with its head removed fails", () => {
    const county = files.find((file) => /county\/franklin\.html$/.test(file));
    expect(county).toBeDefined();
    const html = readFileSync(county!, "utf8");
    expect(strips(html)).toEqual({ marked: 1, headless: 0 });
    const doctored = html.replace(/<div class="strip-head"[^>]*>(?:(?!<\/?div\b)[\s\S])*<\/div>/, "");
    expect(strips(doctored)).toEqual({ marked: 1, headless: 1 });
  });
});
