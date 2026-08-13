/**
 * The appropriation series, and the two ways a chart of it misleads.
 *
 * It is the only block in the feed that is an input to the funding system rather than an output,
 * and the only long series whose nominal and real readings support opposite sentences. Both are
 * tested here; the third hazard — dividing it by a pupil count — has no test because the block
 * carries no denominator, and `denominators.ts` records why.
 */

import { expect, test } from "vitest";

import { fromCatalog, growth, inBase, renderAppropriations } from "../../src/lib/appropriations.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { baseYear } from "../../src/lib/real.ts";

const { bundle } = loadFeed();
const rows = bundle.appropriations;
const base = baseYear(
  bundle.deflator!,
  rows.map((r) => r.fiscal_year),
);

test("the series is continuous from FY2002, which it was not before the Catalog", () => {
  // The workbook series had four holes in it: FY2006, FY2007, FY2012 and FY2013.
  expect(rows.length).toBe(26);
  const years = rows.map((r) => r.fiscal_year);
  expect(years[0]).toBe(2002);
  expect(years[years.length - 1]).toBe(2027);
  expect(years).toEqual(Array.from({ length: 26 }, (_, i) => 2002 + i));
});

test("exactly the four gap years come from the Catalog", () => {
  /*
   * Not "some years". The Catalog covers FY2006-FY2027, so a join that did not check what the
   * workbooks already had would relabel eighteen years and — worse upstream — double their totals.
   */
  expect(fromCatalog(rows)).toEqual([2006, 2007, 2012, 2013]);
});

test("the nominal series grows and the real one grows far less", () => {
  /*
   * The finding the card exists for. Both are true sentences about the same appropriations, and a
   * page showing only the first is making the claim this site was built to check.
   */
  const nominal = growth(rows)!;
  const real = growth(inBase(rows, bundle.deflator, base!, "real"))!;
  expect(nominal).toBeGreaterThan(1.5);
  expect(real).toBeLessThan(nominal * 0.75);
});

test("a year the index cannot reach is dropped from the real view, not shown un-deflated", () => {
  // FY2027 is the year that loses: the index cannot reach a June that has not happened.
  const real = inBase(rows, bundle.deflator, base!, "real");
  expect(real.length).toBeLessThan(rows.length);
  expect(real.some((r) => r.fiscal_year === 2027)).toBe(false);
  // And nothing was silently carried through at its nominal value.
  for (const r of real) {
    const nominal = rows.find((n) => n.fiscal_year === r.fiscal_year)!;
    if (r.fiscal_year !== base) expect(r.enacted).not.toBe(nominal.enacted);
  }
});

test("the card says the other basis tells a different story, on both bases", () => {
  // Whichever one a reader lands on. `BasisToggle` is symmetric by design, so neither panel can
  // rely on the other having been read first.
  for (const basis of ["nominal", "real"] as const) {
    const html = renderAppropriations(rows, bundle.deflator, base, basis);
    expect(html).toContain("tells a different story");
    expect(html).toContain("both sentences are true");
  }
});

test("the card names which publication answers for the four borrowed years", () => {
  const html = renderAppropriations(rows, bundle.deflator, base, "nominal");
  expect(html).toContain("FY2006, FY2007, FY2012, FY2013");
  expect(html).toContain("Catalog of Budget Line Items");
  expect(html).toContain("agree to the cent");
});

test("the card refuses to render a real view with no base year", () => {
  expect(renderAppropriations(rows, null, null, "real")).toBe("");
  expect(renderAppropriations([], bundle.deflator, base, "nominal")).toBe("");
});
