/**
 * The meal-program series, and the two ways it misleads that the other long series do not.
 *
 * `history` can mislead by population, by dollars, and by a hole in the middle. This one has two
 * further failures available to it. Its denominator changes definition at FY2010, so a chart that
 * draws it as a single line reports a step in the data that is partly a step in the divisor. And
 * from FY2012 the report is published as three files of which only one counts applications, so a
 * line joined across FY2011 falls thirteen points for a reason that is not poverty.
 *
 * Every test here is about keeping both breaks visible.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import {
  basisChange,
  renderMealProgram,
  singleStream,
  splitByBasis,
  splitStream,
} from "../../src/lib/mealProgram.ts";

const { bundle } = loadFeed();
const meal = bundle.meal_program;

test("the feed carries the series at all", () => {
  // The failure this guards is the one that already happened twice: a panel computed in Rust,
  // tested in Rust, and exported to no reader. `history` was the first; this was the second.
  expect(meal.length).toBe(17);
  expect(meal[0]!.fiscal_year).toBe(1998);
  expect(meal[meal.length - 1]!.fiscal_year).toBe(2014);
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
  for (const y of singleStream(meal)) {
    expect(y.enrollment, `FY${y.fiscal_year}`).toBeGreaterThan(0);
    expect(y.approved / y.enrollment, `FY${y.fiscal_year}`).toBeCloseTo(y.share!, 4);
  }
});

test("the split years carry a band and no share, and the band is checkable too", () => {
  /*
   * The strongest claim in this block is a refusal: three publications counting three different
   * things do not have a share between them. `null` says that; a zero or an omitted field would
   * not, and either would be read as a number.
   */
  const split = splitStream(meal);
  expect(split.map((y) => y.fiscal_year)).toEqual([2012, 2013, 2014]);
  for (const y of split) {
    expect(y.share, `FY${y.fiscal_year}`).toBeNull();
    expect(y.streams, `FY${y.fiscal_year}`).toBe(3);
    // The floor is the approvals with the directly certified children added back.
    expect((y.approved + y.identified) / y.enrollment, `FY${y.fiscal_year}`).toBeCloseTo(
      y.floor,
      4,
    );
    expect(y.ceiling, `FY${y.fiscal_year}`).toBeGreaterThan(y.floor);
    // And the naive reading — approvals over the whole enrollment — sits below both.
    expect(y.approved / y.enrollment, `FY${y.fiscal_year}`).toBeLessThan(y.floor);
  }
});

test("the band brackets the last year that has a share, so the direction is what is unsettled", () => {
  const last = singleStream(meal).at(-1)!;
  for (const y of splitStream(meal)) {
    expect(y.floor, `FY${y.fiscal_year}`).toBeLessThan(last.share!);
    expect(y.ceiling, `FY${y.fiscal_year}`).toBeGreaterThan(last.share!);
  }
});

test("the population that stopped filing grows, and the feed says how much", () => {
  // A sixth of the enrollment by FY2014, from nothing three years earlier. This is the size of
  // the hole in `approved`, and it is the reason the naive series falls.
  const split = splitStream(meal);
  expect(split[0]!.without_applications).toBeGreaterThan(0.05);
  expect(split.at(-1)!.without_applications).toBeGreaterThan(0.15);
  for (const y of singleStream(meal)) {
    expect(y.without_applications, `FY${y.fiscal_year}`).toBe(0);
    expect(y.identified, `FY${y.fiscal_year}`).toBe(0);
  }
});

test("the denominator changes exactly once, at FY2010", () => {
  expect(basisChange(meal)).toBe(2010);
  expect(meal.filter((y) => y.basis === "adm").map((y) => y.fiscal_year)).toEqual([
    1998, 1999, 2000, 2001, 2002, 2003, 2004, 2005, 2006, 2007, 2008, 2009,
  ]);
  expect(meal.filter((y) => y.basis === "ce").map((y) => y.fiscal_year)).toEqual([
    2010, 2011, 2012, 2013, 2014,
  ]);
});

test("the two eras are two series with nothing joining them", () => {
  /*
   * The load-bearing test. `a` and `b` must never both be non-null in one year: if they were,
   * the chart would connect the eras and draw a continuous line across a redefinition. A single
   * series with a null gap would be wrong in the other direction — a gap says "not measured",
   * and FY2010 was measured, on something else.
   */
  const points = splitByBasis(meal);
  expect(points.length).toBe(singleStream(meal).length);
  for (const p of points) {
    expect(p.a == null || p.b == null, `FY${p.year} is in both eras`).toBe(true);
    expect(p.a != null || p.b != null, `FY${p.year} is in neither`).toBe(true);
  }
  // And each era is contiguous, so "two lines" is two lines rather than a dashed one.
  const eras = points.map((p) => (p.a != null ? "a" : "b")).join("");
  expect(eras).toBe("aaaaaaaaaaaabb");
});

test("the split years are not on the chart at all", () => {
  // Neither bound is plottable: drawing the floor asserts poverty fell, drawing the ceiling
  // asserts it rose, and drawing their midpoint asserts a precision nothing supports.
  const years = splitByBasis(meal).map((p) => p.year);
  expect(years.at(-1)).toBe(2011);
  for (const y of splitStream(meal)) expect(years).not.toContain(y.fiscal_year);
});

test("the page says the two lines are not one line", () => {
  // The break is drawn, but a reader who reads before looking still has to be told. This is the
  // sentence that stops the fourteen-year trend from being quoted as a single figure.
  const html = renderMealProgram(meal);
  expect(html).toContain("The two lines are not one line");
  expect(html).toContain("CECount");
  expect(html).toContain("AdmCount");
});

test("the page says why the line stops before the data does", () => {
  /*
   * A chart that simply ended at FY2011 would read as the archive ending there. It does not:
   * three more Octobers exist and are in the table. The page has to account for the difference,
   * or the most interesting thing about the source is invisible.
   */
  const html = renderMealProgram(meal);
  expect(html).toContain("the report does");
  expect(html).toContain("community\n        eligibility");
  expect(html).toContain("poverty collapsing");
  // The band is quoted with both ends, and the table prints it as a range rather than a figure.
  expect(html).toMatch(/43\.7% to 48\.4%/);
  expect(html).toMatch(/43\.7%–48\.4%/);
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

/**
 * A series that changes how it counts must show the break in the table, not only in the chart.
 *
 * The chart has split this into two series since it was built — `AdmCount` through FY2009,
 * `CECount` from FY2010 — because "the share steps up across it" and a line drawn through is a
 * lie. The table beside it ran straight through the same break, carrying the distinction only in a
 * per-row "Counted on" cell, which a reader meets after the eye has already gone down the column.
 *
 * Structure and not just styling: a `</tbody><tbody>` is a row group a screen reader announces,
 * where a `border-top` is a line only a sighted reader sees.
 */
test("the table marks the definitional break the chart already draws", () => {
  const html = renderMealProgram(meal);
  expect(
    basisChange(meal.filter((y) => y.streams === 1)),
    "the fixture no longer contains a basis change to mark",
  ).not.toBeNull();

  expect(html).toContain('class="series-break"');
  // Two row groups, because the break closes one and opens another.
  expect(html.match(/<tbody>/g)?.length ?? 0).toBeGreaterThan(1);
  // And it says which side is which rather than only that something happened.
  expect(html).toContain("AdmCount");
  expect(html).toContain("CECount");
  expect(html).toMatch(/two series/);
});
