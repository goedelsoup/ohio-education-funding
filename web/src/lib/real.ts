/**
 * Restating the financial panel in constant dollars.
 *
 * # Why this is not optional
 *
 * The panel spans FY2020 to FY2025, in which CPI-U rose **25.1%** — the sharpest price change in
 * forty years. Across that span a nominal statement can have the wrong sign, not merely the
 * wrong magnitude: statewide cash balances end **9% above** where they started in nominal
 * dollars and **13% below** in real ones, and those two sentences support opposite arguments.
 *
 * So the page offers both and says which is showing. It never shows a real figure without the
 * index's name attached, because the choice of index is itself a claim — CPI-U is a general
 * consumer index, and school costs are majority compensation, for which the Employment Cost
 * Index would be better and has shorter coverage.
 */

import type { Deflator, FinanceYear } from "./types.ts";

/** Which dollars a figure is denominated in. */
export type Basis = "nominal" | "real";

/** The index level for a fiscal year, or `null` if the series does not cover it. */
export function indexFor(deflator: Deflator, fiscalYear: number): number | null {
  return deflator.points.find((p) => p.fiscal_year === fiscalYear)?.index ?? null;
}

/**
 * The year a real-terms view is denominated in: the earliest year of the series being restated.
 *
 * # Why this takes the series and not just the index
 *
 * It used to return the first point of the index, full stop, which was the same answer for as
 * long as the index covered exactly one panel. Extending the deflator back to FY2009 for the
 * historical view moved it — and every district's finances page silently began denominating its
 * FY2020–FY2025 figures in FY2009 dollars, because a wider index is not a claim about what any
 * particular card is showing.
 *
 * The base year is a property of the series, so it is computed from the series. A caller with no
 * series to pass has nothing to denominate.
 */
export function baseYear(deflator: Deflator, years?: number[]): number | null {
  const covered = deflator.points.map((p) => p.fiscal_year);
  const candidates = years == null ? covered : years.filter((y) => covered.includes(y));
  return candidates.length === 0 ? null : Math.min(...candidates);
}

/**
 * Restate one figure from `from` dollars into `to` dollars.
 *
 * Returns `null` rather than the nominal figure when the index cannot cover both years — a
 * silently un-deflated number in a column labelled "real" is the failure this guards.
 *
 * The years this can happen for are named: `deflator.uncovered` is the list the feed carries and
 * the index cannot reach, so a caller with a year in hand can say *why* there is no real figure
 * instead of only that there isn't one.
 */
export function convert(
  deflator: Deflator,
  value: number,
  from: number,
  to: number,
): number | null {
  const at = indexFor(deflator, from);
  const target = indexFor(deflator, to);
  if (at == null || target == null || at <= 0) return null;
  return value * (target / at);
}

/**
 * Restate a whole year of the panel into `base` dollars.
 *
 * The cash balance is converted on the same index as the flows. That is the right treatment for
 * a question about what a balance *buys*, which is the question anyone asks of a reserve.
 *
 * A line the filing never carried stays absent. Returns `null` only when the index cannot cover
 * the year, which is the one case where the year cannot honestly be labelled real.
 */
export function yearIn(
  deflator: Deflator,
  year: FinanceYear,
  base: number,
): FinanceYear | null {
  // Two different failures, kept apart. `null` in means the filing carried no such line, and it
  // comes back out as `null`: there is nothing to restate and an absence is the same absence in
  // any year's dollars. `undefined` means the index could not reach the year, which is a failure
  // to restate and disqualifies the whole year — deflating four lines of five and labelling the
  // result "real" is the thing this module exists to prevent.
  const at = (value: number | null): number | null | undefined =>
    value == null ? null : (convert(deflator, value, year.fiscal_year, base) ?? undefined);
  const [aid, local, revenue, spend, cash] = [
    at(year.state_aid),
    at(year.local_tax),
    at(year.total_revenue),
    at(year.total_expenditure),
    at(year.ending_cash),
  ];
  if (
    aid === undefined ||
    local === undefined ||
    revenue === undefined ||
    spend === undefined ||
    cash === undefined
  ) {
    return null;
  }
  return {
    fiscal_year: year.fiscal_year,
    state_aid: aid,
    local_tax: local,
    total_revenue: revenue,
    total_expenditure: spend,
    ending_cash: cash,
  };
}

/**
 * A series in the requested basis.
 *
 * `nominal` returns the input untouched. `real` returns it in the earliest covered year's
 * dollars, or the input untouched if the index cannot cover it — with `converted` saying which
 * happened, so a caller never labels an un-deflated series as real.
 */
export function series(
  deflator: Deflator | null,
  years: FinanceYear[],
  basis: Basis,
): { years: FinanceYear[]; converted: boolean; base: number | null } {
  if (basis === "nominal" || deflator == null) {
    return { years, converted: false, base: null };
  }
  const base = baseYear(
    deflator,
    years.map((year) => year.fiscal_year),
  );
  if (base == null) return { years, converted: false, base: null };
  const restated = years.map((year) => yearIn(deflator, year, base));
  if (restated.some((year) => year == null)) {
    return { years, converted: false, base: null };
  }
  return { years: restated as FinanceYear[], converted: true, base };
}

/**
 * Real change in a measure across a series, as a fraction.
 *
 * The operation most Ohio school finance arguments actually need and the one most often skipped.
 * Returns `null` where the index cannot cover both endpoints or the first value is not positive.
 */
export function realChange(
  deflator: Deflator | null,
  years: FinanceYear[],
  pick: (y: FinanceYear) => number | null,
): number | null {
  const first = years[0];
  const last = years[years.length - 1];
  if (!deflator || !first || !last) return null;
  // An endpoint the filing did not report has no change to state. Reading it as zero would make
  // the change -100% or +infinity depending on which end it fell on, and both would be drawn.
  const [start, end] = [pick(first), pick(last)];
  if (start == null || end == null || start <= 0) return null;
  const rebased = convert(deflator, start, first.fiscal_year, last.fiscal_year);
  if (rebased == null || rebased === 0) return null;
  return end / rebased - 1;
}
