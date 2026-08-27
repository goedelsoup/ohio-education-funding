/**
 * The held-fixed card's worked example, run rather than typed.
 *
 * # What was wrong
 *
 * The card said *"A 3.1% refresh moves $113.0M of base cost aid and about $25.2M more through those
 * three."* `$113.0M` is `3.1% × $3,645.9M`, the panel's total base cost state share — a
 * proportional product, and `policy.ts` rejects proportionality in as many words:
 *
 * > Dollar-for-dollar per pupil, not proportional: local capacity does not move when base cost
 * > does, so the state's residual absorbs the whole per-pupil increase.
 *
 * The two numbers were also different vintages. `$113.0M` came from the pre-fix table in
 * `.yidam/corpus/scenario/ACTIONS.md`, which reported what the site delivered *before*
 * `base_cost_scale` reached the denominated categoricals; `$25.2M` was recomputed after. So the
 * sentence beside them — "until recently this page showed only the first number" — was false, since
 * the page never showed `$113.0M` at all.
 *
 * It was catchable with the slider directly below the card: at `+3%` the tile read `+$163.1M`
 * against the card's `$138.2M`.
 *
 * # What is asserted here
 *
 * Not the values. They move when the feed does, and pinning them would rebuild the defect in a
 * test. What is asserted is the *relationship* the sentence claims — that these are delivered
 * dollars from the same `applyAll` the tiles run, and that the base cost channel is not the
 * proportional product it used to be.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { applyAll, currentLaw, totals } from "../../src/lib/policy.ts";
import { heldFixed, refreshEffect } from "../../src/lib/refresh.ts";

const { bundle } = loadFeed();
const MODEL = bundle.statewide.minimum_state_share;

const effect = refreshEffect(bundle.districts, bundle.drafts, MODEL)!;

test("the scale is the one a draft in this feed prices", () => {
  const carriers = bundle.drafts.filter((d) => d.provisions.some((p) => p.lever === "base-cost"));
  expect(carriers.map((d) => d.slug)).toContain(effect.slug);
  const chosen = carriers.find((d) => d.slug === effect.slug)!;
  expect(effect.scale).toBe(
    Number(chosen.provisions.find((p) => p.lever === "base-cost")!.proposed),
  );
});

test("a one-clause draft is preferred over a bill that merely contains a refresh", () => {
  /*
   * Both bills in this feed price the same restatement, and one is five clauses of which this is
   * the first. A sentence naming *that* bill names something that contains a refresh, and a reader
   * following the link meets four other provisions — special education weights, a transportation
   * floor, a scholarship hold-harmless — none of which the sentence is about.
   */
  const chosen = bundle.drafts.find((d) => d.slug === effect.slug)!;
  expect(chosen.provisions).toHaveLength(1);

  // And where no such draft exists the broader bill is still used rather than nothing.
  const onlyMulti = bundle.drafts.filter((d) => d.provisions.length > 1);
  expect(onlyMulti.length).toBeGreaterThan(0);
  const fallback = refreshEffect(bundle.districts, onlyMulti, MODEL)!;
  expect(fallback.scale).toBe(effect.scale);
  expect(fallback.slug).not.toBe(effect.slug);
});

test("the two channels sum to what the lever actually delivers", () => {
  /*
   * The claim the sentence makes, and the one the old figures broke. Whatever the split, the total
   * has to be the number the tile shows when a reader puts the slider there — which is the check
   * that would have failed on `$113.0M + $25.2M = $138.2M` against `+$169.1M`.
   */
  const at = (scale: number) =>
    totals(applyAll(bundle.districts, { ...currentLaw(MODEL), baseCostScale: scale }, MODEL))
      .realizedAid;
  const delivered = at(effect.scale) - at(1);
  expect(effect.throughBaseCost + effect.throughCategoricals).toBeCloseTo(delivered, 2);
});

test("the base cost channel is not the proportional product it used to be", () => {
  /*
   * `3.1% × base cost state share` was the old arithmetic. The formula absorbs the per-pupil
   * increase dollar-for-dollar into the state's residual instead, so the delivered figure is far
   * from that product — and a change that made them agree again would mean `apply` had stopped
   * doing what its own comment says.
   */
  const shareTotal = bundle.districts.reduce((sum, d) => sum + d.base_cost_state_share, 0);
  const proportional = shareTotal * (effect.scale - 1);
  expect(effect.throughBaseCost).toBeGreaterThan(proportional * 1.2);
});

test("the categorical channel is smaller than the arithmetic uplift, because the guarantee absorbs some", () => {
  // `denominated × (scale − 1)` is what the three programs gain on paper. What reaches a district
  // is less, because a district under its guarantee floor delivers none of it.
  const { denominated } = heldFixed(bundle.districts);
  expect(effect.throughCategoricals).toBeGreaterThan(0);
  expect(effect.throughCategoricals).toBeLessThan(denominated * (effect.scale - 1));
});

test("a feed with no priced base-cost provision prices no refresh", () => {
  // The card drops the worked example rather than inventing a scale for it.
  expect(refreshEffect(bundle.districts, [], MODEL)).toBeNull();
  const unpriced = bundle.drafts.map((d) => ({
    ...d,
    provisions: d.provisions.map((p) => ({ ...p, lever: "" as const })),
  }));
  expect(refreshEffect(bundle.districts, unpriced, MODEL)).toBeNull();
});

test("the three held-fixed totals agree with what the Rust asserts about them", () => {
  /*
   * `crates/project/tests/what_a_scenario_holds_fixed.rs` pins the reachable exposure at $812.5m
   * and preschool's unreached weighted half at $45.7m, each to within a million, and asserts that
   * the index-driven pair is the larger of the two groups — which is the finding that makes the
   * whole card worth writing. These were typed into the page in a second language on the other side
   * of that process boundary; this is the same claim, on this side.
   */
  const { denominated, indexDriven, preschoolWeighted } = heldFixed(bundle.districts);
  expect(denominated / 1e6).toBeCloseTo(812.5, 0);
  expect(preschoolWeighted / 1e6).toBeCloseTo(45.7, 0);
  expect(indexDriven).toBeGreaterThan(denominated + preschoolWeighted);
});
