/**
 * The guard against comparing two per-pupil figures that divide by different pupils.
 *
 * This site has shipped that error twice. Both times the two numbers were correct, both were per
 * pupil, and the pairing was wrong — a failure with no signature that a type or a schema can see.
 * `src/lib/denominators.ts` writes down which count each field uses; these tests make the writing
 * down mandatory and check the pairings the site actually renders.
 */

import { expect, test } from "vitest";

import { median } from "../../src/lib/stats.ts";

import {
  BLOCK_DENOMINATORS,
  DENOMINATORS,
  FIELD_DENOMINATORS,
  RENDERED_PAIRS,
} from "../../src/lib/denominators.ts";
import { admSeamGap, loadFeed } from "../../src/lib/feed.ts";

/** Every path in the feed that looks like a per-pupil quantity or a pupil count. */
function denominatorBearingPaths(): string[] {
  const { bundle } = loadFeed();
  const found = new Set<string>();

  const walk = (value: unknown, path: string): void => {
    if (value === null || typeof value !== "object") return;
    if (Array.isArray(value)) {
      // One element is enough: the array is homogeneous by construction, and the schema proves it.
      if (value[0] !== undefined) walk(value[0], `${path}[]`);
      return;
    }
    for (const [key, child] of Object.entries(value)) {
      const here = `${path}.${key}`.replace(/^\./, "");
      // Deliberately loose. The first version matched `per_pupil` literally and so missed
      // `per_enrolled_pupil` and `per_equivalent_pupil` — the two fields whose denominators
      // differ from each other, which is the pair the guard most needs to see. `adm` is bounded
      // so that `general_admin` and `school_admin` do not come along.
      //
      // `fte` is here because the department counts pupils that way in three of the six
      // categoricals — career-technical FTE and identified gifted FTE are pupil counts under
      // another name, and a guard that only recognises `adm` would let a per-FTE figure onto a
      // page next to a per-ADM one without asking. It is bounded the same way `adm` is, so the
      // teacher counts in the base cost build-up are out of scope: those are staff, not pupils,
      // and nothing on the site divides by them.
      if (/pupil|per_mill|(^|_)(adm|fte)($|_)|enroll/i.test(key)) found.add(here);
      walk(child, here);
    }
  };

  walk(bundle, "");
  return [...found].sort();
}

test("every per-pupil field in the feed declares its denominator", () => {
  /*
   * The point of the whole file. A field added in Rust reaches this test before it reaches a
   * page, and the build stops until someone says which pupil count it divides by — which is the
   * moment that question is cheapest to answer and the only moment it is reliably asked.
   */
  const undeclared = denominatorBearingPaths().filter((p) => !(p in FIELD_DENOMINATORS));
  expect(undeclared).toEqual([]);
});

test("the declaration has no entries for fields that no longer exist", () => {
  // The other direction: a stale declaration is a claim about the feed that has stopped being
  // true, and a reader consulting this table would be misled rather than merely unhelped.
  const live = new Set(denominatorBearingPaths());
  const stale = Object.keys(FIELD_DENOMINATORS).filter((p) => !live.has(p));
  expect(stale).toEqual([]);
});

test("a field inside a declared block agrees with the block", () => {
  /*
   * `spending_by_function` is entirely per unweighted pupil and most of its fields are named for
   * a function rather than for a denominator — `instruction`, `food_service`. A name-keyed walk
   * cannot see those, so the block is declared whole; this checks the two the walk *does* find
   * were not declared inconsistently with it.
   */
  for (const [prefix, expected] of Object.entries(BLOCK_DENOMINATORS)) {
    const inside = Object.entries(FIELD_DENOMINATORS).filter(([path]) => path.startsWith(prefix));
    expect(inside.length, `${prefix} declares a block but no field inside it`).toBeGreaterThan(0);
    for (const [path, key] of inside) {
      // A block carries its own pupil count, and that field is the denominator rather than a
      // quantity over one. `spending_by_function.adm` is the unweighted count itself.
      if (key === null) continue;
      expect(key, `${path} disagrees with its block`).toBe(expected);
    }
  }
});

test("every declared denominator is one of the six the sources actually use", () => {
  for (const [field, key] of Object.entries(FIELD_DENOMINATORS)) {
    if (key === null) continue;
    expect(Object.keys(DENOMINATORS), `${field} names an unknown denominator`).toContain(key);
  }
});

test("each pair the site renders side by side shares a denominator", () => {
  /*
   * The concrete bug. `districts[].property_tax[].value_per_pupil` was displayed against
   * `statewide.median_valuation_per_pupil` on 609 pages: Taxation's numerator over Taxation's
   * pupils, compared to a median over Education's. The medians happened to sit 1.4% apart, so it
   * looked right everywhere except the big-city districts a reader is most likely to open.
   */
  const mismatched = RENDERED_PAIRS.filter(
    ([district, statewide]) => FIELD_DENOMINATORS[district] !== FIELD_DENOMINATORS[statewide],
  ).map(
    ([district, statewide]) =>
      `${district} (${FIELD_DENOMINATORS[district]}) against ${statewide} (${FIELD_DENOMINATORS[statewide]})`,
  );
  expect(mismatched).toEqual([]);
});

test("both sides of every rendered pair are declared at all", () => {
  for (const path of RENDERED_PAIRS.flat()) {
    expect(Object.keys(FIELD_DENOMINATORS), `${path} is paired but not declared`).toContain(path);
  }
});

test("the six pupil counts are genuinely different numbers", () => {
  /*
   * If they agreed there would be nothing to guard. Columbus is the case that makes the file
   * worth having: the same district, the same property, four counts of its children.
   */
  const { byIrn } = loadFeed();
  const columbus = byIrn.get("043802")!;

  const counts = {
    "base-cost-adm": columbus.adm,
    "enrolled-adm-fy24": columbus.adm_history[0],
    "unweighted-adm-fy25": columbus.spending_by_function!.adm,
    "sd1-adm": columbus.property_tax[columbus.property_tax.length - 1]!.adm,
  };

  const values = Object.values(counts);
  expect(Math.max(...values) / Math.min(...values)).toBeGreaterThan(1.5);

  // And the direction is stable: Taxation counts the most, because it counts children who live
  // in the district rather than children it teaches.
  expect(counts["sd1-adm"]).toBeGreaterThan(counts["enrolled-adm-fy24"] * 1.5);
});

test("the enrolled-versus-funded seam is one pair on both halves of the sentence that states it", () => {
  /*
   * The taxes page names the seam the charge-off counterfactual sits on — a deemed local share
   * per **enrolled** ADM subtracted from a base cost per **funded** ADM — and gave a per-district
   * figure beside a statewide median. The two were computed from different pairs of counts: the
   * district figure from `adm` against `adm_history[0]`, the median from `adm` against
   * `current_year_adm`. A reader was comparing their district against a distribution it was not
   * drawn from, and the two figures are 1.95% and 1.6%, close enough that nothing looked wrong.
   *
   * `crates/regime-diff::at_fy2027` decides which pair is right: the deemed share is built from
   * `valuation_per_pupil`, which is the profile report's and is per enrolled ADM — the count
   * `FIELD_DENOMINATORS` maps that very field to — and the base cost it comes off is per base
   * cost ADM, the three-year average. So the seam is `adm` against `adm_history[0]`, and this
   * pins the whole sentence to that one pair.
   */
  const { bundle, tax } = loadFeed();

  expect(FIELD_DENOMINATORS["districts[].valuation_per_pupil"]).toBe("enrolled-adm-fy24");
  expect(DENOMINATORS["enrolled-adm-fy24"].field).toBe("districts[].adm_history[0]");
  expect(DENOMINATORS["base-cost-adm"].field).toBe("districts[].adm");

  const gaps = bundle.districts
    .map((d) => admSeamGap(d))
    .filter((v): v is number => v != null);
  expect(gaps).toHaveLength(bundle.districts.length);

  const sorted = [...gaps].sort((a, b) => a - b);
  expect(tax.admSeam.median).toBeCloseTo(median(sorted), 12);
  expect(tax.admSeam.max).toBeCloseTo(sorted[sorted.length - 1]!, 12);

  // And it is not the other pair, which is what the sentence used to quote. Both round to "1.6%"
  // and "1.9%" at one decimal — a tenth of a percent apart, and about a different question.
  const other = median(
    bundle.districts
      .filter((d) => d.current_year_adm > 0)
      .map((d) => Math.abs(d.adm / d.current_year_adm - 1)),
  );
  expect(tax.admSeam.median).not.toBeCloseTo(other, 3);
});

test("the pupil-count table's divergence figures are read, not typed", () => {
  /*
   * `denominators.ts` made the same claim as the taxes page, with the same wrong median typed
   * into its note: "a median 1.6% and by 27% at the extreme". The extreme was right and the
   * median was the other pair's. Neither figure is written down now; `method.astro` renders them
   * from `TaxStatewide.admSeam`, which is one computation for both places that state it.
   */
  const entry = DENOMINATORS["enrolled-adm-fy24"];
  expect("divergence" in entry).toBe(true);
  expect(entry.note).not.toMatch(/\d+(\.\d+)?%/);
});
