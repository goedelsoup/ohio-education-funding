/**
 * The historical panel, and the three ways a series like this misleads.
 *
 * The panel spans FY2009-FY2022 with a hole in it, is measured on a population that is not the
 * one every other page uses, and is denominated in dollars that lost a third of their value
 * across the window. Each of those is a way for a correct chart to state something false, and
 * each has a test here.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import {
  gaps,
  inBase,
  renderEqualization,
  renderRevenueMix,
  residual,
  stateShareOfGap,
  withGaps,
} from "../../src/lib/history.ts";
import { baseYear } from "../../src/lib/real.ts";

const { bundle } = loadFeed();
const history = bundle.history;
const historyYears = history.map((y) => y.fiscal_year);

test("the feed carries the panel the page exists to draw", () => {
  // It was computed in Rust and exported nowhere for four phases. The failure this guards is the
  // one that already happened: a fixture in the workspace that no reader could reach.
  expect(history.length).toBeGreaterThan(10);
  expect(history[0]!.fiscal_year).toBe(2009);
  expect(history[history.length - 1]!.fiscal_year).toBe(2022);
});

test("years are ordered and unique, because a chart reads them as an axis", () => {
  const years = history.map((y) => y.fiscal_year);
  expect(new Set(years).size).toBe(years.length);
  expect([...years].sort((a, b) => a - b)).toEqual(years);
});

test("the shares sum to one, so no revenue is unaccounted for", () => {
  for (const year of history) {
    const total = year.local_share + year.state_share + year.federal_share;
    expect(Math.abs(total - 1)).toBeLessThan(0.005);
  }
});

test("FY2014 is a break in the series rather than a bridge across it", () => {
  // It is absent from the Bureau's archive under every naming the other years use. A line drawn
  // straight through it would assert a measurement nobody made, so the year reaches the chart as
  // a point with no value.
  expect(gaps(history)).toEqual([2014]);
  const points = withGaps(
    history,
    (y) => y.local_share,
    (y) => y.state_share,
  );
  const missing = points.find((p) => p.year === 2014);
  expect(missing).toBeDefined();
  expect(missing!.a).toBeNull();
  expect(missing!.b).toBeNull();
  // And every other year is still there, in order.
  expect(points).toHaveLength(2022 - 2009 + 1);
});

test("the residual is the part no level of government closes", () => {
  for (const year of history) {
    expect(residual(year)).toBeCloseTo(
      year.gap_per_pupil - year.state_closes_per_pupil - year.federal_closes_per_pupil,
      6,
    );
  }
});

test("the state share fell across the panel, which is the finding the page states", () => {
  const first = history[0]!;
  const last = history[history.length - 1]!;
  expect(first.state_share).toBeGreaterThan(last.state_share);
  expect(last.local_share).toBeGreaterThan(last.state_share);
});

test("the rate held while the gap grew — both halves, or the page says the wrong thing", () => {
  // The claim on the card. State aid's share of the gap stays in a narrow band while the gap
  // itself grows, so the unclosed part grows with it. If either half stopped being true the
  // sentence beside the chart would be false and nothing else would notice.
  const first = history[0]!;
  const last = history[history.length - 1]!;
  expect(Math.abs(stateShareOfGap(last) - stateShareOfGap(first))).toBeLessThan(0.06);
  expect(last.gap_per_pupil).toBeGreaterThan(first.gap_per_pupil * 1.5);
  expect(residual(last)).toBeGreaterThan(residual(first));
});

test("a real-terms year is deflated on every dollar column or dropped entirely", () => {
  // The failure mode: one un-deflated figure in a column labelled real. Restating must move
  // every dollar field or none of them.
  const base = bundle.deflator ? baseYear(bundle.deflator, historyYears) : null;
  expect(base).not.toBeNull();
  // Not the first year: the base is the earliest year the index covers, and restating that one
  // into its own dollars is the identity — a test that would pass against a function doing nothing.
  const year = history[history.length - 1]!;
  expect(year.fiscal_year).not.toBe(base);
  const real = inBase(bundle.deflator, year, base!, "real");
  expect(real).not.toBeNull();
  for (const key of [
    "gap_per_pupil",
    "state_closes_per_pupil",
    "federal_closes_per_pupil",
    "poorest_local_per_pupil",
    "richest_local_per_pupil",
  ] as const) {
    expect(real![key]).not.toBe(year[key]);
  }
  // Shares are ratios and must survive untouched.
  expect(real!.local_share).toBe(year.local_share);
  expect(real!.districts).toBe(year.districts);
});

test("the deflator covers every year the history carries", () => {
  // It used to cover only the financial panel's FY2020-FY2025. A page offering constant dollars
  // for a series the index cannot reach would silently drop most of it.
  expect(bundle.deflator).not.toBeNull();
  const covered = new Set(bundle.deflator!.points.map((p) => p.fiscal_year));
  for (const year of history) expect(covered.has(year.fiscal_year)).toBe(true);
});

test("no year without an index survives into the real-dollar view", () => {
  const uncovered = { ...history[0]!, fiscal_year: 1975 };
  expect(inBase(bundle.deflator, uncovered, baseYear(bundle.deflator!, historyYears)!, "real")).toBeNull();
  // Nominal asks nothing of the index and passes the year through.
  expect(inBase(null, uncovered, 2009, "nominal")).toEqual(uncovered);
});

test("both cards render, and say which population they measured", () => {
  const mix = renderRevenueMix(history);
  expect(mix).toContain("<svg");
  expect(mix).toContain("FY2022");
  // The federal caveat is the one a reader most needs and least expects.
  expect(mix).toContain("pandemic relief peak");

  const base = baseYear(bundle.deflator!, historyYears)!;
  const real = renderEqualization(history, bundle.deflator, base, "real");
  expect(real).toContain(`In FY${base} dollars.`);
  expect(renderEqualization(history, bundle.deflator, base, "nominal")).toContain(
    "In the dollars of each year.",
  );
});

test("a panel too short to be a series draws nothing rather than a degenerate axis", () => {
  expect(renderRevenueMix([])).toBe("");
  expect(renderRevenueMix(history.slice(0, 1))).toBe("");
  expect(renderEqualization([], bundle.deflator, 2009, "nominal")).toBe("");
});

test("extending the index for this panel did not rebase the financial views", () => {
  /*
   * The regression this page caused, and the reason `baseYear` takes a series. The deflator now
   * reaches back to FY2009 so the history route can be shown in constant dollars — and for one
   * build that silently restated every district's FY2020-FY2025 finances into FY2009 dollars,
   * because the base year was read off the index rather than off the series being restated.
   */
  const financeYears = bundle.statewide.finances.map((y) => y.fiscal_year);
  expect(baseYear(bundle.deflator!, financeYears)).toBe(Math.min(...financeYears));
  expect(baseYear(bundle.deflator!, historyYears)).toBe(Math.min(...historyYears));
  expect(baseYear(bundle.deflator!, financeYears)).not.toBe(
    baseYear(bundle.deflator!, historyYears),
  );
});

test("a series the index cannot reach has no base year rather than a wrong one", () => {
  expect(baseYear(bundle.deflator!, [1899, 1900])).toBeNull();
  // And a partly covered series denominates in the earliest year that *is* covered, never in one
  // the index does not have.
  expect(baseYear(bundle.deflator!, [1899, 2015])).toBe(2015);
});
