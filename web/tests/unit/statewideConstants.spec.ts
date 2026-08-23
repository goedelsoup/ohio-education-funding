/**
 * The four statewide figures the district page prints, checked against the panel they describe.
 *
 * # Why these four and not the rest of `statewide`
 *
 * Because these four were typed into `district.ts` rather than read from anywhere. Two statutory
 * medians copied out of `crates/project::panel::categoricals`, and the preschool sheet's
 * appropriation, factor and statewide total written into a paragraph that renders on 609 pages.
 * #107 moved them into the feed, and moving a figure into the feed is only half the fix: the feed
 * can carry a stale constant exactly as happily as a paragraph can.
 *
 * What makes it not stale is that each of the four has a relationship to the 609 districts beside
 * it, and that relationship is checkable here. `format.ts` renders `null`, `NaN` and `Infinity` as
 * an em dash, so a *missing* figure announces itself on the page; a constant that is merely wrong
 * is always present, always finite, and cannot. These are the assertions that stand in for that.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";

const { bundle } = loadFeed();
const w = bundle.statewide;

test("the preschool total is the sum of the districts the feed carries", () => {
  // Summed in Rust over the same 609 districts, for the reason `Statewide::finances` is: the page
  // and the feed cannot then disagree about which districts are in the total. The card built on
  // this figure states it as "the program totals X statewide", and X is over this population.
  const summed = bundle.districts.reduce(
    (a, d) => a + d.preschool_special_education.total,
    0,
  );
  expect(Math.abs(w.preschool_total - summed)).toBeLessThan(1);
});

test("the preschool program exceeds the appropriation printed beside its factor", () => {
  // The whole of what the card says. If a regenerated feed ever made this false the paragraph
  // would still render, and it would render "$0 over" in a sentence beginning "And this program
  // is over its appropriation."
  expect(w.preschool_appropriation).toBeGreaterThan(0);
  expect(w.preschool_total).toBeGreaterThan(w.preschool_appropriation);
});

test("the stated proration factor is the one the districts were paid at", () => {
  // `unprorated` is `total / factor` in the Rust, so recovering the factor from any district that
  // received the program checks the number the card prints against the arithmetic it describes.
  const paid = bundle.districts.filter((d) => d.preschool_special_education.total > 0);
  expect(paid.length).toBeGreaterThan(0);
  for (const d of paid) {
    const p = d.preschool_special_education;
    expect(Math.abs(p.total / p.unprorated - w.preschool_proration)).toBeLessThan(1e-6);
  }
});

test("the targeted assistance medians are the medians the indices were computed with", () => {
  /*
   * `[B]` is the median district's weighted wealth over this district's, and `[E]` is the median
   * per resident pupil over this district's. Both arrive from the Rust already computed, so
   * dividing them back out recovers whichever median the Rust used — which is exactly the figure
   * the page needs and exactly the one it used to type.
   *
   * Only districts with a positive index, because the tiers switch off and a zero index carries
   * no information about the median.
   */
  const capacity = bundle.districts.filter(
    (d) => d.targeted_assistance.capacity_index > 0 && d.targeted_assistance.weighted_wealth > 0,
  );
  expect(capacity.length).toBeGreaterThan(0);
  for (const d of capacity) {
    const t = d.targeted_assistance;
    const implied = t.capacity_index * t.weighted_wealth;
    // The feed rounds the index to four places, so the tolerance is what that rounding is worth
    // at this magnitude rather than a number chosen to make the test pass.
    expect(Math.abs(implied - w.targeted_assistance_median_weighted_wealth)).toBeLessThan(
      t.weighted_wealth * 1e-4,
    );
  }

  const wealth = bundle.districts.filter(
    (d) => d.targeted_assistance.wealth_index > 0 && d.targeted_assistance.wealth_per_pupil > 0,
  );
  expect(wealth.length).toBeGreaterThan(0);
  for (const d of wealth) {
    const t = d.targeted_assistance;
    const implied = t.wealth_index * t.wealth_per_pupil;
    expect(Math.abs(implied - w.targeted_assistance_median_wealth_per_pupil)).toBeLessThan(
      t.wealth_per_pupil * 1e-4,
    );
  }
});
