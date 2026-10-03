/**
 * `/outcomes` draws the guarantee trap, and counts its poverty ceiling once (#703).
 *
 * The second of the page's four steps — "the guarantee is that trap" — was the only one told by a
 * table. It is the poverty cloud coloured by regime now, with a median line through each, and the
 * table under it. And the wall of dots at the cloud's right edge is named on the chart, by a count
 * the limits card repeats: both read `CEILING_FROM`, so the two numbers are asserted equal here
 * rather than trusted to be.
 *
 * Read off the bytes, as `peerStrip.spec.ts` is.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vitest";

import { DIST } from "./artefact.ts";

const html = readFileSync(join(DIST, "outcomes.html"), "utf8");

/** The markup of the card with this id, up to the next card. */
function card(id: string): string {
  const from = html.indexOf(`id="${id}"`);
  expect(from, `#${id} is on the page`).toBeGreaterThan(-1);
  const next = html.indexOf('<div class="card"', from);
  return html.slice(from, next < 0 ? undefined : next);
}

describe("/outcomes", () => {
  test("the guarantee trap is a chart with a median line for each regime", () => {
    const trap = card("guarantee-trap");
    expect(trap).toContain('<div class="chartwrap" data-chart="guarantee-trap">');
    // Every drawing of the chart carries the same two lines; count them in the first.
    const first = trap.slice(trap.indexOf("<svg"), trap.indexOf("</svg>"));
    expect(first.match(/class="scatter-trace"/g) ?? []).toHaveLength(2);
  });

  test("the ceiling on the chart is the ceiling in the prose", () => {
    const note = /class="scatter-ceiling"[^>]*>[\s\S]*?(\d+) districts at/.exec(
      card("poverty-and-performance"),
    );
    expect(note, "the poverty cloud names its ceiling").not.toBeNull();
    const prose = /(\d+) within a point of it/.exec(card("limits"));
    expect(prose, "the limits card counts it").not.toBeNull();
    expect(Number(note![1])).toBeGreaterThan(0);
    expect(note![1]).toBe(prose![1]);
  });
});
