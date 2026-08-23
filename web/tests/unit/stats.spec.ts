/**
 * One definition of the median across the workspace, and the figures that were two.
 *
 * The site published two statistics under one phrase. `crates/dispersion::median` interpolates —
 * R's type 7 — and computes the eight `median_*` fields in `bundle.statewide`; `web/src/lib/stats`
 * took the upper of the two middle observations, with a reason of its own. So `/statewide` said
 * the median district was $47 per pupil worse off under the regime counterfactual and
 * `/district/…/taxes` said $45, and neither was stale or mistyped.
 *
 * These pin the resolution rather than the arithmetic: the web definition is now the crates', and
 * nothing in this layer computes a median any other way.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { median, percentile } from "../../src/lib/stats.ts";

const { bundle, tax } = loadFeed();

test("the median interpolates between the two middle observations", () => {
  // The whole disagreement, in four numbers. The upper-middle definition returns 3.
  expect(median([1, 2, 3, 4])).toBe(2.5);
  // Odd lengths never distinguished the two definitions, which is why this went unnoticed.
  expect(median([1, 2, 3])).toBe(2);
  expect(median([7])).toBe(7);
  // Zero and not `undefined`: every caller here interpolates the result into prose or a chart.
  // `dispersion::median` returns `None` instead, for reasons that hold on the crate side only.
  expect(median([])).toBe(0);
  // Order-independent — callers pass unsorted arrays and the crate's takes a sorted slice.
  expect(median([4, 1, 3, 2])).toBe(2.5);
});

test("percentile is R type 7, on rank q * (n - 1)", () => {
  const series = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  expect(percentile(series, 0)).toBeCloseTo(0, 12);
  expect(percentile(series, 0.5)).toBeCloseTo(5, 12);
  expect(percentile(series, 1)).toBeCloseTo(10, 12);
  // Between observations rather than at one: rank 0.25 * 10 = 2.5.
  expect(percentile([0, 1, 2, 3, 4], 0.625)).toBeCloseTo(2.5, 12);
});

test("this layer's median agrees with the feed's on a field the feed already carries", () => {
  /*
   * The check that would have caught the defect. `median_regime_difference` is computed by
   * `crates/dispersion::median` over exactly these values, so recomputing them here and comparing
   * is a direct test of whether the two sides share a definition. Under the upper-middle
   * definition this is off by $2.40.
   */
  const differences = bundle.districts
    .map((d) => d.regime?.difference)
    .filter((v): v is number => v != null);

  expect(differences.length).toBeGreaterThan(0);
  expect(median(differences)).toBeCloseTo(bundle.statewide.median_regime_difference, 3);
});

test("the correction the taxes page describes is derived from the same definition on both sides", () => {
  /*
   * The pair that used to be the literals `$289` and `$45`. `overstated_by` is the charge-off the
   * recognised-valuation deferral removes, so adding it back reconstructs the uncorrected
   * comparison — and the sentence's claim, that the correction moved a finding and not only a
   * figure, is exactly the claim that the two signs differ.
   */
  const uncorrected = bundle.districts
    .filter((d) => d.regime?.difference != null && d.regime?.overstated_by != null)
    .map((d) => d.regime!.difference! + d.regime!.overstated_by!);

  expect(tax.medianRegimeDifferenceUncorrected).toBeCloseTo(median(uncorrected), 6);
  expect(tax.medianRegimeDifferenceUncorrected).toBeGreaterThan(0);
  expect(bundle.statewide.median_regime_difference).toBeLessThan(0);
});

test("nothing in the rendered layer hand-rolls a median", () => {
  /*
   * The ratchet. Five copies of `sorted[Math.floor(n / 2)]` existed across `county.ts`,
   * `relationships.ts`, `feed.ts`, `statewide.ts` and this module, and the drift between them and
   * the crates is what published two answers. A sixth would do it again silently.
   *
   * `plot/spec.ts` is exempt and says why: its box takes quartiles by nearest rank because the box
   * is drawn at an observation. That convention has a name — `nearestRank` — so a caller
   * describing a box in prose shares it rather than writing the expression out again.
   */
  const SRC = join(import.meta.dirname, "../../src");
  const walk = (dir: string): string[] =>
    readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
      const path = join(dir, e.name);
      if (e.isDirectory()) return walk(path);
      return /\.(ts|astro)$/.test(e.name) ? [path] : [];
    });

  const MIDDLE = /\[\s*Math\.floor\(\s*[\w.]+(?:\.length)?\s*(?:\/\s*2|\*\s*0?\.5)\s*\)\s*\]/;
  const offenders = walk(SRC)
    .filter((f) => !f.endsWith(join("plot", "spec.ts")))
    .filter((f) => MIDDLE.test(readFileSync(f, "utf8")))
    .map((f) => f.slice(SRC.length + 1));

  expect(offenders, "use `median` from src/lib/stats.ts").toEqual([]);
});
