/**
 * The browser formula, checked against the real feed.
 *
 * Vitest, through Astro's Vite config — pure functions over the committed bundle, with no
 * browser and no build in the way. The page is Playwright's job; this is the arithmetic.
 *
 * The important test is the checkpoint one: it runs every Rust-computed reference scenario
 * through the TypeScript against all 609 districts. If the two implementations drift, this
 * fails here rather than at a legislator's desk.
 */

import { expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { apply, applyAll, currentLaw, currentRealizedAid, totals } from "../../src/lib/policy.ts";
import { compare, isVerified, toPolicy, verify } from "../../src/lib/verify.ts";
import { bin } from "../../src/lib/chart.ts";
import { money, millions, ordinal, pct, percentileOf, signedMoney } from "../../src/lib/format.ts";
import { guaranteeRateByQuintile, wealthQuintiles } from "../../src/lib/statewide.ts";
import { performanceByPoverty, povertyQuintiles } from "../../src/lib/outcomes.ts";
import { REQUIRED_CONTRACT, type Bundle } from "../../src/lib/types.ts";

const bundle: Bundle = JSON.parse(
  readFileSync(join(import.meta.dirname, "../../public/data/bundle.json"), "utf8"),
);
const model = bundle.statewide.minimum_state_share;

test("the committed feed is the contract this page reads", () => {
  expect(bundle.contract_version).toBe(REQUIRED_CONTRACT);
  expect(bundle.districts.length).toBeGreaterThan(600);
  expect(bundle.checkpoints.length).toBeGreaterThan(0);
});

test("every Rust-computed checkpoint is reproduced by the browser formula", () => {
  const verification = verify(bundle);
  const failures = verification.comparisons.filter((c) => !c.agrees);
  expect(failures.map((f) => `${f.label}: ${f.differences.join("; ")}`)).toEqual([]);
  expect(isVerified(verification)).toBe(true);
});

test("current law is the identity, district by district", () => {
  // Not just in aggregate: an error that cancels across 609 districts is still an error.
  for (const d of bundle.districts) {
    const outcome = apply(d, currentLaw(model), d.current_year_adm, model);
    expect(Math.abs(outcome.delta), `${d.name} moved by ${outcome.delta} under current law`)
      .toBeLessThan(0.02);
    expect(outcome.onGuarantee, d.name).toBe(d.on_guarantee);
  }
});

test("removing the guarantee saves exactly what the guarantee costs", () => {
  const outcomes = applyAll(
    bundle.districts,
    { ...currentLaw(model), guarantee: { kind: "removed" } },
    model,
  );
  const t = totals(outcomes);
  expect(Math.abs(t.cost + bundle.statewide.guarantee_total)).toBeLessThan(1);
  expect(t.onGuarantee).toBe(0);
  expect(t.gainers).toBe(0);
  expect(t.losers).toBe(bundle.statewide.on_guarantee);
});

test("no guarantee policy reaches a district the formula already pays", () => {
  // The structural fact: a floor cannot catch anyone it was not already holding.
  for (const rule of [
    { kind: "removed" } as const,
    { kind: "rebase", factor: 0.5 } as const,
    { kind: "phase-out", remaining: 0.25 } as const,
  ]) {
    const outcomes = applyAll(bundle.districts, { ...currentLaw(model), guarantee: rule }, model);
    const movedOnFormula = outcomes.filter(
      (o, i) => Math.abs(o.delta) > 0.02 && !bundle.districts[i]!.on_guarantee,
    );
    expect(movedOnFormula.map((o) => o.name), `${rule.kind} reached a formula district`).toEqual([]);
  }
});

test("a phase-out at zero is removal and at one is current law", () => {
  const at = (remaining: number) =>
    totals(
      applyAll(
        bundle.districts,
        { ...currentLaw(model), guarantee: { kind: "phase-out", remaining } },
        model,
      ),
    ).cost;
  expect(Math.abs(at(1) - 0)).toBeLessThan(1);
  expect(Math.abs(at(0) + bundle.statewide.guarantee_total)).toBeLessThan(1);
  expect(Math.abs(at(0.5) + bundle.statewide.guarantee_total / 2)).toBeLessThan(1);
});

test("raising base cost cannot cut any district", () => {
  const outcomes = applyAll(bundle.districts, { ...currentLaw(model), baseCostScale: 1.05 }, model);
  expect(totals(outcomes).losers).toBe(0);
});

test("the marginal cost of base cost rises as districts come off the guarantee", () => {
  const cost = (scale: number) =>
    totals(applyAll(bundle.districts, { ...currentLaw(model), baseCostScale: scale }, model)).cost;
  const perPoint = (from: number, to: number) => (cost(to) - cost(from)) / ((to - from) * 100);
  expect(perPoint(1.0, 1.02), "the guarantee is a shock absorber that wears out")
    .toBeLessThan(perPoint(1.1, 1.2));
});

test("current-law realized aid is summed from components, not round-tripped", () => {
  const total = bundle.districts.reduce((sum, d) => sum + currentRealizedAid(d), 0);
  expect(Math.abs(total - bundle.statewide.realized_aid_total), `${total}`).toBeLessThan(1);
});

test("a serialized policy round-trips into a runnable one", () => {
  const checkpoint = bundle.checkpoints.find((c) => c.policy.guarantee === "phase-out")!;
  const policy = toPolicy(checkpoint.policy);
  expect(policy.guarantee.kind).toBe("phase-out");
  expect(compare(bundle, checkpoint).agrees).toBe(true);
});

test("a feed with no checkpoints is unverified, not verified", () => {
  // `every()` over an empty list is true; without this guard a feed that lost its checkpoints
  // would pass silently and let the scenario builder run unchecked.
  const stripped: Bundle = { ...bundle, checkpoints: [] };
  const verification = verify(stripped);
  expect(verification.ok).toBe(true);
  expect(isVerified(verification)).toBe(false);
});

test("a tampered checkpoint is caught", () => {
  const tampered: Bundle = {
    ...bundle,
    checkpoints: [{ ...bundle.checkpoints[1]!, cost: bundle.checkpoints[1]!.cost + 1000 }],
  };
  expect(isVerified(verify(tampered))).toBe(false);
});

test("wealth quintiles keep every district with a valuation", () => {
  const groups = wealthQuintiles(bundle.districts);
  const kept = groups.reduce((n, g) => n + g.length, 0);
  const withValuation = bundle.districts.filter((d) => d.valuation_per_pupil != null).length;
  expect(kept, "integer division must not drop the remainder").toBe(withValuation);
  expect(groups.length).toBe(5);
});

test("guarantee rate rises with property wealth", () => {
  const bars = guaranteeRateByQuintile(bundle.districts);
  expect(bars.length).toBe(5);
  expect(bars[4]!.value, `poorest ${bars[0]!.value}, wealthiest ${bars[4]!.value}`)
    .toBeGreaterThan(bars[0]!.value * 2);
});

test("binning puts the top edge in the last bin rather than past the end", () => {
  const bins = bin([0, 1, 2, 3, 4], 4);
  expect(bins.length).toBe(4);
  expect(bins.reduce((n, b) => n + b.count, 0)).toBe(5);
  expect(bins[3]!.count, "4 belongs in the last bin, and so does 3").toBe(2);
});

test("binning a constant series does not divide by zero", () => {
  const bins = bin([7, 7, 7], 10);
  expect(bins.length).toBe(1);
  expect(bins[0]!.count).toBe(3);
});

test("ordinals handle the teens", () => {
  expect(ordinal(1)).toBe("1st");
  expect(ordinal(2)).toBe("2nd");
  expect(ordinal(3)).toBe("3rd");
  expect(ordinal(4)).toBe("4th");
  expect(ordinal(11)).toBe("11th");
  expect(ordinal(12)).toBe("12th");
  expect(ordinal(13)).toBe("13th");
  expect(ordinal(21)).toBe("21st");
  expect(ordinal(63)).toBe("63rd");
  expect(ordinal(100)).toBe("100th");
});

test("missing values render as an em dash, never as zero", () => {
  expect(money(null)).toBe("—");
  expect(pct(null)).toBe("—");
  expect(millions(null)).toBe("—");
  expect(money(null)).not.toBe(money(0));
});

test("a zero change reads as zero rather than as a gain", () => {
  expect(signedMoney(0)).toBe("$0");
  expect(signedMoney(0.0001)).toBe("$0");
  expect(signedMoney(120)).toBe("+$120");
  expect(signedMoney(-120)).toBe("−$120");
});

test("percentile is the share strictly below", () => {
  expect(percentileOf([1, 2, 3, 4], 3)).toBe(0.5);
  expect(percentileOf([1, 2, 3, 4], 1)).toBe(0);
  expect(percentileOf([], 1)).toBe(0);
});

test("outcomes: the feed carries an outcome block for every district with a report card", () => {
  const withOutcome = bundle.districts.filter((d) => d.outcome != null);
  expect(withOutcome.length).toBe(bundle.statewide.outcomes!.districts);
  // And the three without one are null rather than an object of nulls, so "no report card"
  // stays distinguishable from "a report card with nothing in it".
  const without = bundle.districts.filter((d) => d.outcome == null);
  expect(without.length).toBe(3);
});

test("outcomes: carries the raw guarantee association and the controlled one together", () => {
  // A page showing +0.187 without +0.035 beside it would be stating the confound as a
  // finding, which is the one thing this axis exists to prevent.
  const o = bundle.statewide.outcomes!;
  expect(Math.abs(o.guarantee_vs_performance)).toBeGreaterThan(0.15);
  expect(Math.abs(o.guarantee_vs_performance_controlled)).toBeLessThan(0.06);
});

test("outcomes: carries both spending denominators, and they disagree", () => {
  const o = bundle.statewide.outcomes!;
  expect(Math.abs(o.weighted_spending_vs_performance)).toBeLessThan(0.05);
  expect(o.enrolled_spending_vs_performance).toBeLessThan(-0.25);
});

test("outcomes: poverty quintiles keep every district that has both variables", () => {
  const groups = povertyQuintiles(bundle.districts);
  const kept = groups.reduce((n, g) => n + g.length, 0);
  const eligible = bundle.districts.filter(
    (d) => d.economically_disadvantaged != null && d.outcome?.performance_index != null,
  ).length;
  expect(kept).toBe(eligible);
  expect(groups.length).toBe(5);
});

test("outcomes: the Performance Index falls monotonically across poverty fifths", () => {
  // The relationship the whole view is built to foreground. If it ever stopped being
  // monotone the copy on that card would be wrong, not just the chart.
  const medians = performanceByPoverty(bundle.districts).map((b) => b.value);
  for (let i = 1; i < medians.length; i++) {
    expect(medians[i]!).toBeLessThan(medians[i - 1]!);
  }
});

test("outcomes: every district with a report card has a spending figure on both denominators", () => {
  const broken = bundle.districts.filter(
    (d) =>
      d.outcome != null &&
      (d.outcome.per_enrolled_pupil == null || d.outcome.per_equivalent_pupil == null),
  );
  expect(broken.map((d) => d.name)).toEqual([]);
});

test("outcomes: spending per enrolled pupil is never below spending per weighted pupil", () => {
  // The weighted count scales the enrolled one upward, so the weighted per-pupil figure is
  // always the smaller. Inverting this would flip the denominator finding.
  for (const d of bundle.districts) {
    if (!d.outcome?.per_enrolled_pupil || !d.outcome.per_equivalent_pupil) continue;
    expect(d.outcome.per_enrolled_pupil).toBeGreaterThanOrEqual(
      d.outcome.per_equivalent_pupil - 0.01,
    );
  }
});
