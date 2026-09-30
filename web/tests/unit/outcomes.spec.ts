/**
 * The district outcome card, rendered for every district in the feed.
 *
 * The three report-card shares became fractions in bundle 35.0.0 and the card went on printing
 * them with a percent sign appended, so Columbus read 1.0% economically disadvantaged under a
 * Dashboard header of 100.0% (#570). Every district, because the defect was on every district.
 */

import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { renderDistrictOutcome } from "../../src/lib/outcomes.ts";

const { bundle } = loadFeed();

const ROWS = [
  "Economically disadvantaged (report card)",
  "English learners",
  "Students with disabilities",
] as const;

/** The figure in the row headed `label`, as a number of percent, or null for a dash. */
function row(html: string, label: string): number | null {
  const at = html.indexOf(`<th>${label}</th>`);
  expect(at, `row "${label}"`).toBeGreaterThan(-1);
  const cell = /<td[^>]*>([^<]*)<\/td>/.exec(html.slice(at))?.[1] ?? "";
  if (cell === "—") return null;
  expect(cell, `row "${label}" reads as a percentage`).toMatch(/^\d+\.\d%$/);
  return Number(cell.slice(0, -1));
}

test("each report-card share is printed as the percentage its fraction is", () => {
  let read = 0;
  for (const d of bundle.districts) {
    const o = d.outcome;
    if (!o) continue;
    const html = renderDistrictOutcome(d, bundle.statewide.outcomes);
    const fields = [o.economically_disadvantaged, o.english_learner, o.students_with_disabilities];
    ROWS.forEach((label, i) => {
      const shown = row(html, label);
      const v = fields[i];
      if (v == null) return expect(shown, `${d.irn} ${label}`).toBeNull();
      expect(shown, `${d.irn} ${label}`).toBeCloseTo(v * 100, 1);
      read++;
    });
  }
  expect(read).toBeGreaterThan(1500);
});

test("no district over half disadvantaged by its profile reads under 1% on its report card", () => {
  // The issue's own symptom, stated against the other measure so it cannot pass by agreeing with
  // itself: the two shares differ by top-coding, never by two orders of magnitude.
  const poor = bundle.districts.filter(
    (d) => d.outcome?.economically_disadvantaged != null && (d.economically_disadvantaged ?? 0) > 0.5,
  );
  expect(poor.length).toBeGreaterThan(50);
  for (const d of poor) {
    const shown = row(renderDistrictOutcome(d, bundle.statewide.outcomes), ROWS[0]);
    expect(shown, d.irn).toBeGreaterThanOrEqual(1);
  }
});
