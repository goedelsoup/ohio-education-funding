/**
 * The routes the chart gate visits, held against the build they are read from.
 *
 * `tests/e2e/charted.ts` reads the gate's route list off `dist/` and collapses a templated family
 * to one instance. That collapse is the one place the gate can still be blind, in two ways: a
 * family pattern that swallows a page it should not, and a new templated family nobody named, which
 * the gate would visit hundreds of times over. Both are facts about the build, so they are here.
 *
 * The other assertion is #608's second defect, read off the bytes: a panel is a member of a small
 * multiple, and a "panel" drawn at the whole wide frame is one chart with no narrow copy.
 */

import { readFileSync } from "node:fs";

import { describe, expect, test } from "vitest";

import { CLIENT_DRAWN, chartedRoutes, FAMILIES, familyOf, plottedRoutes } from "../e2e/charted.ts";

import { DIST, pages } from "./artefact.ts";

describe("the chart gate's routes", () => {
  const plotted = plottedRoutes(DIST);

  test("every built route with a chart is visited, itself or at its family's instance", () => {
    const visited = new Set(chartedRoutes(DIST));
    const missed = plotted.filter((route) => !visited.has(familyOf(route)?.visit ?? route));
    expect(missed).toEqual([]);
    // And the browser-drawn routes, which the build cannot name, are visited as well.
    for (const route of CLIENT_DRAWN) expect(visited.has(route), route).toBe(true);
    // The pages #608 found leaving their frames, which the hand list never reached.
    for (const route of [
      "/bounds",
      "/wiki/formula-component/temporary-transitional-aid-guarantee",
      "/wiki/formula-component/fsfp-base-cost-calculation",
    ]) {
      expect(visited.has(route), route).toBe(true);
    }
  });

  test("each family is a family, and is visited at a page that draws it", () => {
    for (const family of FAMILIES) {
      const members = plotted.filter((route) => family.pattern.test(route));
      // A pattern matching one page is a page given a family's name, and collapses nothing.
      expect(members.length, String(family.pattern)).toBeGreaterThan(1);
      expect(members, `${family.pattern} is visited where it is drawn`).toContain(family.visit);
    }
  });

  test("the pages that are only themselves are few enough to be pages, not a family", () => {
    /*
     * Everything no family matches is visited one by one. Today that is twelve routes. A new
     * templated route that draws a chart — a page per school, say — would arrive here as hundreds,
     * and the gate would load every one of them in every test; the fix is a line in `FAMILIES`.
     */
    const singles = plotted.filter((route) => familyOf(route) == null);
    expect(singles.length, singles.join("\n")).toBeLessThanOrEqual(40);
  });
});

describe("a panel", () => {
  test("is never drawn wider than a panel of two", () => {
    /*
     * The TTAG node's one-panel spread was drawn once at 640 through the panel renderer and shown
     * to a phone at a scale of 0.46, its 11px type painted at 5.1px — #103's defect, returned by
     * the one path that does not make a narrow drawing. 340 sits over the 311 a panel of two is
     * drawn at and well under the 640 of a whole chart.
     */
    const wide: string[] = [];
    for (const file of pages()) {
      const html = readFileSync(file, "utf8");
      if (!html.includes('data-at="panel"')) continue;
      for (const [, at] of html.matchAll(
        /<div class="chart-at" data-at="panel"><svg\b[^>]*viewBox="0 0 ([\d.]+) /g,
      )) {
        if (Number(at) > 340) wide.push(`${file.slice(DIST.length)}: ${at}`);
      }
    }
    expect(wide.slice(0, 10)).toEqual([]);
  });
});
