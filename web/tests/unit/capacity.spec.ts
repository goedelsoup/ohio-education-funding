/**
 * Local capacity's three terms add to the department's published figure, for every district (#791).
 *
 * # Why here, and not in the Rust
 *
 * `crates/bundle` multiplies the three weighted measures by the rate and writes them beside their
 * sum, so a Rust test that the terms add to the total would test one function against itself. The
 * claim worth holding is about the **shipped feed**: after four-place serialization, the terms the
 * taxes page prints add to the total it prints, and that total is `regime.local_capacity`, the
 * department's `[b1]` read off the calculator by a different path. Both checks run here through
 * `decomposeCapacity`, the function the card itself calls, so the test and the page cannot hold
 * different rules. The crate's agreement with the department is `xcheck`'s to prove; this is the
 * page's.
 *
 * And a guard that passes on every row measures nothing until a doctored row is shown to fail it,
 * so each check is paired with a real district's row bent by one percent in one term.
 */

import { expect, test } from "vitest";

import { CAPACITY_AGREEMENT, CAPACITY_CLOSURE, decomposeCapacity } from "../../src/lib/capacity.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import type { District } from "../../src/lib/types.ts";

const { bundle } = loadFeed();

/** Shawnee Local, Allen County: the district the issue worked by hand. */
const SHAWNEE = "045799";

test("every district carries a capacity, and every one closes on the published figure", () => {
  expect(bundle.districts).toHaveLength(609);
  let worstClosure = 0;
  let worstAgreement = 0;
  for (const d of bundle.districts) {
    const k = decomposeCapacity(d);
    expect(k, `${d.irn} has no decomposition`).not.toBeNull();
    const sum = k!.valuation.term + k!.grossIncome.term + k!.medianIncome.term;
    worstClosure = Math.max(worstClosure, Math.abs(sum - k!.total));
    worstAgreement = Math.max(worstAgreement, Math.abs(k!.total - k!.published) / k!.published);
  }
  // The guard throws past its bounds, so these restate them. They are here so the worst case is
  // on record beside the bound it sits under: 1.0e-4 dollars and 6.364e-6 of the figure.
  expect(worstClosure).toBeLessThanOrEqual(CAPACITY_CLOSURE);
  expect(worstAgreement).toBeLessThanOrEqual(CAPACITY_AGREEMENT);
});

test("the weights recovered from the feed are the statute's, and the shares add to one", () => {
  for (const d of bundle.districts) {
    const k = decomposeCapacity(d)!;
    expect([k.valuation.weight, k.grossIncome.weight, k.medianIncome.weight]).toEqual([0.6, 0.2, 0.2]);
    expect(Math.abs(k.valuation.share + k.grossIncome.share + k.medianIncome.share - 1)).toBeLessThan(1e-6);
  }
});

/** One district with one of its three terms raised by 1%, leaving the total where it was. */
function doctored(field: "valuation_term_per_pupil" | "median_income_term_per_pupil"): District {
  const d = bundle.districts.find((x) => x.irn === SHAWNEE)!;
  return { ...d, capacity: { ...d.capacity!, [field]: d.capacity![field] * 1.01 } };
}

test("a term moved by one percent no longer closes, and the page will not draw it", () => {
  // Shawnee's valuation term is $4,140.03, so 1% is $41 — and the smallest term moved by 1% is
  // still $11.85, four orders of magnitude past the bound.
  expect(() => decomposeCapacity(doctored("valuation_term_per_pupil"))).toThrow(/add to/);
  expect(() => decomposeCapacity(doctored("median_income_term_per_pupil"))).toThrow(/add to/);
});

test("a total that closes on its terms but not on the published figure is refused too", () => {
  // All three terms and the total scaled together: the sum still closes, so only the second check
  // can catch it. The case is a feed whose rate is wrong for every term at once.
  const d = bundle.districts.find((x) => x.irn === SHAWNEE)!;
  const c = d.capacity!;
  const scaled = {
    ...d,
    capacity: {
      ...c,
      valuation_term_per_pupil: c.valuation_term_per_pupil * 1.01,
      gross_income_term_per_pupil: c.gross_income_term_per_pupil * 1.01,
      median_income_term_per_pupil: c.median_income_term_per_pupil * 1.01,
      per_pupil: c.per_pupil * 1.01,
    },
  };
  expect(() => decomposeCapacity(scaled)).toThrow(/published/);
});

test("Shawnee Local's capacity is the figure the charge-off card prints, in three terms", () => {
  /*
   * The issue's hand-worked row was $4,149.34 + $1,861.89 + $1,212.09 = $7,223.31, from the
   * calculator's `fy27-oct2` vintage. This feed is the vintage the crate reproduces, where the
   * statewide median income is $54,546.64 rather than $54,182 and every one of Shawnee's own inputs
   * moved too, so the row reads $4,140.03 + $1,794.83 + $1,184.54 = $7,119.41, and that is the
   * department's published figure for this vintage.
   */
  const d = bundle.districts.find((x) => x.irn === SHAWNEE)!;
  const k = decomposeCapacity(d)!;
  expect(k.valuation.term).toBeCloseTo(4140.03, 2);
  expect(k.grossIncome.term).toBeCloseTo(1794.83, 2);
  expect(k.medianIncome.term).toBeCloseTo(1184.54, 2);
  expect(k.total).toBeCloseTo(7119.41, 2);
  expect(d.regime!.local_capacity).toBeCloseTo(7119.41, 2);
});
