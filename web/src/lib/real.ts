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

/** The year a real-terms view is denominated in: the earliest the index covers. */
export function baseYear(deflator: Deflator): number | null {
  return deflator.points[0]?.fiscal_year ?? null;
}

/**
 * Restate one figure from `from` dollars into `to` dollars.
 *
 * Returns `null` rather than the nominal figure when the index cannot cover both years — a
 * silently un-deflated number in a column labelled "real" is the failure this guards.
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
 */
export function yearIn(
  deflator: Deflator,
  year: FinanceYear,
  base: number,
): FinanceYear | null {
  const at = (value: number) => convert(deflator, value, year.fiscal_year, base);
  const [aid, local, revenue, spend, cash] = [
    at(year.state_aid),
    at(year.local_tax),
    at(year.total_revenue),
    at(year.total_expenditure),
    at(year.ending_cash),
  ];
  if (
    aid == null ||
    local == null ||
    revenue == null ||
    spend == null ||
    cash == null
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
  const base = baseYear(deflator);
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
  pick: (y: FinanceYear) => number,
): number | null {
  const first = years[0];
  const last = years[years.length - 1];
  if (!deflator || !first || !last || pick(first) <= 0) return null;
  const rebased = convert(deflator, pick(first), first.fiscal_year, last.fiscal_year);
  if (rebased == null || rebased === 0) return null;
  return pick(last) / rebased - 1;
}
