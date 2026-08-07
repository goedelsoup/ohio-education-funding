/**
 * The browser formula, checked against the real feed.
 *
 * `node --test web/test` runs these. Node 22.18 and later strip types natively, so there is no
 * build step and no test framework to install — the same no-dependencies property the Rust side
 * keeps, for the same reason.
 *
 * The important test is {@link the checkpoint one}: it runs every Rust-computed reference
 * scenario through the TypeScript against all 609 districts. If the two implementations drift,
 * this fails here rather than at a legislator's desk.
 */

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { apply, applyAll, currentLaw, currentRealizedAid, totals } from "../src/policy.ts";
import { compare, isVerified, toPolicy, verify } from "../src/verify.ts";
import { bin } from "../src/chart.ts";
import { money, millions, ordinal, pct, percentileOf, signedMoney } from "../src/format.ts";
import { guaranteeRateByQuintile, wealthQuintiles } from "../src/statewide.ts";
import { REQUIRED_CONTRACT, type Bundle } from "../src/types.ts";

const bundle: Bundle = JSON.parse(
  readFileSync(join(import.meta.dirname, "../data/bundle.json"), "utf8"),
);
const model = bundle.statewide.minimum_state_share;

test("the committed feed is the contract this page reads", () => {
  assert.equal(bundle.contract_version, REQUIRED_CONTRACT);
  assert.ok(bundle.districts.length > 600);
  assert.ok(bundle.checkpoints.length > 0);
});

test("every Rust-computed checkpoint is reproduced by the browser formula", () => {
  const verification = verify(bundle);
  const failures = verification.comparisons.filter((c) => !c.agrees);
  assert.deepEqual(
    failures.map((f) => `${f.label}: ${f.differences.join("; ")}`),
    [],
  );
  assert.ok(isVerified(verification));
});

test("current law is the identity, district by district", () => {
  // Not just in aggregate: an error that cancels across 609 districts is still an error.
  for (const d of bundle.districts) {
    const outcome = apply(d, currentLaw(model), d.current_year_adm, model);
    assert.ok(
      Math.abs(outcome.delta) < 0.02,
      `${d.name} moved by ${outcome.delta} under current law`,
    );
    assert.equal(outcome.onGuarantee, d.on_guarantee, d.name);
  }
});

test("removing the guarantee saves exactly what the guarantee costs", () => {
  const outcomes = applyAll(
    bundle.districts,
    { ...currentLaw(model), guarantee: { kind: "removed" } },
    model,
  );
  const t = totals(outcomes);
  assert.ok(Math.abs(t.cost + bundle.statewide.guarantee_total) < 1);
  assert.equal(t.onGuarantee, 0);
  assert.equal(t.gainers, 0);
  assert.equal(t.losers, bundle.statewide.on_guarantee);
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
    assert.deepEqual(movedOnFormula.map((o) => o.name), [], `${rule.kind} reached a formula district`);
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
  assert.ok(Math.abs(at(1) - 0) < 1);
  assert.ok(Math.abs(at(0) + bundle.statewide.guarantee_total) < 1);
  assert.ok(Math.abs(at(0.5) + bundle.statewide.guarantee_total / 2) < 1);
});

test("raising base cost cannot cut any district", () => {
  const outcomes = applyAll(bundle.districts, { ...currentLaw(model), baseCostScale: 1.05 }, model);
  assert.equal(totals(outcomes).losers, 0);
});

test("the marginal cost of base cost rises as districts come off the guarantee", () => {
  const cost = (scale: number) =>
    totals(applyAll(bundle.districts, { ...currentLaw(model), baseCostScale: scale }, model)).cost;
  const perPoint = (from: number, to: number) =>
    (cost(to) - cost(from)) / ((to - from) * 100);
  assert.ok(
    perPoint(1.0, 1.02) < perPoint(1.1, 1.2),
    "the guarantee is a shock absorber that wears out",
  );
});

test("current-law realized aid is summed from components, not round-tripped", () => {
  const total = bundle.districts.reduce((sum, d) => sum + currentRealizedAid(d), 0);
  assert.ok(Math.abs(total - bundle.statewide.realized_aid_total) < 1, `${total}`);
});

test("a serialized policy round-trips into a runnable one", () => {
  const checkpoint = bundle.checkpoints.find((c) => c.policy.guarantee === "phase-out")!;
  const policy = toPolicy(checkpoint.policy);
  assert.equal(policy.guarantee.kind, "phase-out");
  assert.ok(compare(bundle, checkpoint).agrees);
});

test("a feed with no checkpoints is unverified, not verified", () => {
  // `every()` over an empty list is true; without this guard a feed that lost its checkpoints
  // would pass silently and let the scenario builder run unchecked.
  const stripped: Bundle = { ...bundle, checkpoints: [] };
  const verification = verify(stripped);
  assert.equal(verification.ok, true);
  assert.equal(isVerified(verification), false);
});

test("a tampered checkpoint is caught", () => {
  const tampered: Bundle = {
    ...bundle,
    checkpoints: [{ ...bundle.checkpoints[1]!, cost: bundle.checkpoints[1]!.cost + 1000 }],
  };
  assert.equal(isVerified(verify(tampered)), false);
});

test("wealth quintiles keep every district with a valuation", () => {
  const groups = wealthQuintiles(bundle.districts);
  const kept = groups.reduce((n, g) => n + g.length, 0);
  const withValuation = bundle.districts.filter((d) => d.valuation_per_pupil != null).length;
  assert.equal(kept, withValuation, "integer division must not drop the remainder");
  assert.equal(groups.length, 5);
});

test("guarantee rate rises with property wealth", () => {
  const bars = guaranteeRateByQuintile(bundle.districts);
  assert.equal(bars.length, 5);
  assert.ok(
    bars[4]!.value > bars[0]!.value * 2,
    `poorest ${bars[0]!.value}, wealthiest ${bars[4]!.value}`,
  );
});

test("binning puts the top edge in the last bin rather than past the end", () => {
  const bins = bin([0, 1, 2, 3, 4], 4);
  assert.equal(bins.length, 4);
  assert.equal(bins.reduce((n, b) => n + b.count, 0), 5);
  assert.equal(bins[3]!.count, 2, "4 belongs in the last bin, and so does 3");
});

test("binning a constant series does not divide by zero", () => {
  const bins = bin([7, 7, 7], 10);
  assert.equal(bins.length, 1);
  assert.equal(bins[0]!.count, 3);
});

test("ordinals handle the teens", () => {
  assert.equal(ordinal(1), "1st");
  assert.equal(ordinal(2), "2nd");
  assert.equal(ordinal(3), "3rd");
  assert.equal(ordinal(4), "4th");
  assert.equal(ordinal(11), "11th");
  assert.equal(ordinal(12), "12th");
  assert.equal(ordinal(13), "13th");
  assert.equal(ordinal(21), "21st");
  assert.equal(ordinal(63), "63rd");
  assert.equal(ordinal(100), "100th");
});

test("missing values render as an em dash, never as zero", () => {
  assert.equal(money(null), "—");
  assert.equal(pct(null), "—");
  assert.equal(millions(null), "—");
  assert.notEqual(money(null), money(0));
});

test("a zero change reads as zero rather than as a gain", () => {
  assert.equal(signedMoney(0), "$0");
  assert.equal(signedMoney(0.0001), "$0");
  assert.equal(signedMoney(120), "+$120");
  assert.equal(signedMoney(-120), "−$120");
});

test("percentile is the share strictly below", () => {
  assert.equal(percentileOf([1, 2, 3, 4], 3), 0.5);
  assert.equal(percentileOf([1, 2, 3, 4], 1), 0);
  assert.equal(percentileOf([], 1), 0);
});
