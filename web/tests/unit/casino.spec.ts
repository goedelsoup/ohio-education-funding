/**
 * The casino block, and the four joins it invites that would be wrong.
 *
 * This is the first block in the feed that is money a district receives and that **no other figure
 * in the feed counts**. Everything else per-district is either the department's model or the
 * treasurer's general fund, and both of those are populations and constructions the rest of the
 * feed already agrees with. This one agrees with nothing:
 *
 * - its population is about a thousand districts against the feed's 609;
 * - its denominator is a fifth pupil count, defined by R.C. 5753.11 and shared with nothing here;
 * - its fiscal year is the year of the *payment*, one year later than the half-year earned;
 * - and it is booked outside the general fund, so the `finances` rows do not contain it either.
 *
 * Every test below is about keeping one of those four from being quietly crossed.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { renderCasino } from "../../src/lib/district.ts";
import { seriesYear } from "../../src/lib/year.ts";

const { bundle } = loadFeed();
const statewide = bundle.casino;

test("the feed carries the series at all", () => {
  // The failure this guards is the one that has now happened three times: a panel computed in
  // Rust, tested in Rust, and exported to no reader. `history` was the first and `meal_program`
  // the second.
  expect(statewide.length).toBe(9);
  expect(statewide[0]!.fiscal_year).toBe(2016);
  expect(statewide[statewide.length - 1]!.fiscal_year).toBe(2024);
});

test("years are ordered and unique, because a chart reads them as an axis", () => {
  const years = statewide.map((y) => y.fiscal_year);
  expect(new Set(years).size).toBe(years.length);
  expect([...years].sort((a, b) => a - b)).toEqual(years);
});

test("every district carries the same nine years, so a chart has no holes to interpolate", () => {
  const expected = statewide.map((y) => y.fiscal_year);
  for (const d of bundle.districts) {
    expect(
      d.casino.map((y) => y.fiscal_year),
      `${d.irn} ${d.name}`,
    ).toEqual(expected);
  }
});

test("the districts do not sum to the statewide figure, and the gap is the point", () => {
  /*
   * R.C. 5753.11 counts community schools, STEM schools and joint vocational districts as public
   * school districts, so the fund reaches about a thousand agencies. This feed carries 609. A
   * consumer that summed the district rows and presented the result as the fund would understate
   * it by an eighth — and, worse, would be describing a population it had not named.
   */
  const last = statewide[statewide.length - 1]!;
  const summed = bundle.districts.reduce(
    (total, d) => total + (d.casino[d.casino.length - 1]?.amount ?? 0),
    0,
  );
  expect(summed).toBeLessThan(last.amount);
  expect(summed / last.amount).toBeGreaterThan(0.85);
  expect(summed / last.amount).toBeLessThan(0.9);
});

test("the closure lands in FY2021, which is a year later than the half-year that closed", () => {
  // The casinos shut in March 2020. The August 2020 payment settles January-June 2020 and falls in
  // FY2021, so a reader looking for the pandemic in FY2020 finds a normal year. This is the one
  // place the payment basis is load-bearing rather than a convention.
  const by = new Map(statewide.map((y) => [y.fiscal_year, y.amount]));
  const closed = by.get(2021)!;
  const others = statewide.filter((y) => y.fiscal_year !== 2021).map((y) => y.amount);
  expect(closed).toBeLessThan(Math.min(...others));
  expect(by.get(2020)!).toBeGreaterThan(by.get(2019)!);
});

test("the two statewide figures the card types are the ones the feed produces", () => {
  /*
   * `renderCasino` prints a median share and a single-county count that are not in the feed —
   * one is a median over a join the feed does not make, the other a count over a nullable field.
   * Typed figures in prose are what `yearLiterals.spec.ts` exists to police for years; these are
   * the same hazard in a different shape, so they are checked here against the data they describe.
   */
  const shares = bundle.districts
    .map((d) => {
      const last = d.casino[d.casino.length - 1];
      const booked = d.finances.find((f) => f.fiscal_year === last?.fiscal_year);
      return last && booked && booked.state_aid > 0 ? last.amount / booked.state_aid : null;
    })
    .filter((s): s is number => s != null)
    .sort((a, b) => a - b);
  const median = shares[Math.floor(shares.length / 2)]!;
  expect(median).toBeCloseTo(0.0115, 4);

  const single = bundle.districts.filter((d) => d.casino_counties === 1).length;
  expect(single).toBe(178);
});

test("a county span is a catchment and never a district's home county", () => {
  // Every district in this feed is paid from at least one county fund, and the ones paid from
  // several are the ones whose territory crosses a county line — not an error and not a duplicate.
  for (const d of bundle.districts) {
    expect(d.casino_counties, `${d.irn} ${d.name}`).not.toBeNull();
    expect(d.casino_counties!).toBeGreaterThanOrEqual(1);
    expect(d.casino_counties!).toBeLessThanOrEqual(7);
  }
});

test("the chip is derived from the block rather than typed beside it", () => {
  const chip = seriesYear("casino", bundle);
  expect(chip).not.toBeNull();
  expect(chip!.kind).toBe("fiscal");
  expect(chip!.label).toBe(
    `FY${statewide[0]!.fiscal_year}-FY${statewide[statewide.length - 1]!.fiscal_year}`,
  );
});

test("the card refuses a per-pupil figure and says why", () => {
  /*
   * The strongest thing this card does is decline to divide. Four per-pupil figures already sit on
   * a district's pages and a fifth would read as one of them, so the refusal has to survive
   * someone later deciding the card looks incomplete without one.
   */
  const d = bundle.districts.find((x) => x.irn === "043802")!;
  const html = renderCasino(bundle, d);
  expect(html).toContain("no per-pupil figure here");
  expect(html).toContain("R.C. 5753.11");
  expect(html).not.toMatch(/per pupil<\/div>/);
  // And it states the thing the rest of the page cannot: that nothing above it counts this money.
  expect(html).toContain("nothing above this card counts it");
});

test("a district with no distribution renders nothing rather than an empty card", () => {
  const d = bundle.districts[0]!;
  expect(renderCasino(bundle, { ...d, casino: [] })).toBe("");
});
