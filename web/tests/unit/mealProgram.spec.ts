/**
 * The meal-program series, and the one way it misleads that the other long series do not.
 *
 * `history` can mislead by population, by dollars, and by a hole in the middle. This one has a
 * fourth failure available to it: its denominator changes definition at FY2010, so a chart that
 * draws it as a single line reports a step in the data that is partly a step in the divisor.
 * Every test here is about keeping that break visible.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { basisChange, renderMealProgram, splitByBasis } from "../../src/lib/mealProgram.ts";

const { bundle } = loadFeed();
const meal = bundle.meal_program;

test("the feed carries the series at all", () => {
  // The failure this guards is the one that already happened twice: a panel computed in Rust,
  // tested in Rust, and exported to no reader. `history` was the first; this was the second.
  expect(meal.length).toBe(11);
  expect(meal[0]!.fiscal_year).toBe(2001);
  expect(meal[meal.length - 1]!.fiscal_year).toBe(2011);
});

test("years are ordered and unique, because a chart reads them as an axis", () => {
  const years = meal.map((y) => y.fiscal_year);
  expect(new Set(years).size).toBe(years.length);
  expect([...years].sort((a, b) => a - b)).toEqual(years);
});

test("every row's share is its own two counts divided, not a figure from elsewhere", () => {
  /*
   * The counts are in the feed so the share can be checked. If they ever stop agreeing, the
   * block is carrying a number whose provenance nothing on the page can establish — which is
   * the whole objection this project has to publishing a bare rate.
   */
  for (const y of meal) {
    expect(y.enrollment, `FY${y.fiscal_year}`).toBeGreaterThan(0);
    expect(y.approved / y.enrollment, `FY${y.fiscal_year}`).toBeCloseTo(y.share, 4);
  }
});

test("the denominator changes exactly once, at FY2010", () => {
  // Pinned rather than inferred. If a future extraction adds FY2012 onward, the report splits
  // into three streams with a different population again — and this test is where that arrives.
  expect(basisChange(meal)).toBe(2010);
  expect(meal.filter((y) => y.basis === "adm").map((y) => y.fiscal_year)).toEqual([
    2001, 2002, 2003, 2004, 2005, 2006, 2007, 2008, 2009,
  ]);
  expect(meal.filter((y) => y.basis === "ce").map((y) => y.fiscal_year)).toEqual([2010, 2011]);
});

test("the two eras are two series with nothing joining them", () => {
  /*
   * The load-bearing test. `a` and `b` must never both be non-null in one year: if they were,
   * the chart would connect the eras and draw a continuous line across a redefinition. A single
   * series with a null gap would be wrong in the other direction — a gap says "not measured",
   * and FY2010 was measured, on something else.
   */
  const points = splitByBasis(meal);
  expect(points.length).toBe(meal.length);
  for (const p of points) {
    expect(p.a == null || p.b == null, `FY${p.year} is in both eras`).toBe(true);
    expect(p.a != null || p.b != null, `FY${p.year} is in neither`).toBe(true);
  }
  // And each era is contiguous, so "two lines" is two lines rather than a dashed one.
  const eras = points.map((p) => (p.a != null ? "a" : "b")).join("");
  expect(eras).toBe("aaaaaaaaabb");
});

test("the page says the two lines are not one line", () => {
  // The break is drawn, but a reader who reads before looking still has to be told. This is the
  // sentence that stops the eleven-year trend from being quoted as a single figure.
  const html = renderMealProgram(meal);
  expect(html).toContain("The two lines are not one line");
  expect(html).toContain("CECount");
  expect(html).toContain("AdmCount");
});

test("the page states the population and refuses the comparison a reader will want", () => {
  /*
   * The site already carries a poverty share: the report card's, on every district page, one
   * year and top-coded by community eligibility. This series looks like its history and is not.
   * The page has to say so, because nothing else can — the two never appear on one route.
   */
  const html = renderMealProgram(meal);
  expect(html).toContain("sponsors");
  expect(html).toMatch(/top-coded by community\s+eligibility/);
  expect(html).toContain("formula side");
});

test("a feed without the series renders nothing rather than an empty chart", () => {
  expect(renderMealProgram([])).toBe("");
  expect(renderMealProgram(meal.slice(0, 1))).toBe("");
  expect(basisChange([])).toBeNull();
  expect(basisChange(meal.slice(0, 3))).toBeNull();
});
