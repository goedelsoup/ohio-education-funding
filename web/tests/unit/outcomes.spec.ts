/**
 * The district outcome card, rendered for every district in the feed.
 *
 * The three report-card shares became fractions in bundle 35.0.0 and the card went on printing
 * them with a percent sign appended, so Columbus read 1.0% economically disadvantaged under a
 * Dashboard header of 100.0% (#570). Every district, because the defect was on every district.
 */

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { loadFeed } from "../../src/lib/feed.ts";
import { renderDistrictOutcome, renderOutcomes } from "../../src/lib/outcomes.ts";
import { DOT } from "../../src/lib/plot/spec.ts";
import { bands, medianTrace } from "../../src/lib/relationships.ts";
import { percentile } from "../../src/lib/stats.ts";
import type { District } from "../../src/lib/types.ts";

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

/*
 * The two-denominators pair on `/outcomes` (#702). Its lead-ins state pooled correlations, its
 * closing note states two spreads, and its dek promises the same two axes; each is held against
 * what the card actually draws and prints.
 */
const card = parseHTML(`<div>${renderOutcomes(bundle)}</div>`).document.querySelector(
  '[data-part="two-denominators"]',
)!;
const PAIR = {
  "weighted-spending": (d: District) => d.outcome?.per_equivalent_pupil,
  "enrolled-spending": (d: District) => d.outcome?.per_enrolled_pupil,
} as const;

test("each lead-in's word is true of the pooled line drawn under it", () => {
  for (const [chart, x] of Object.entries(PAIR)) {
    const lead = card.querySelector(`[data-chart="${chart}"]`)!.previousElementSibling!;
    const word = lead.querySelector("strong")?.textContent;
    const values = bundle.districts.flatMap((d) => {
      const xv = x(d);
      const yv = d.outcome?.performance_index;
      return xv != null && yv != null ? [{ x: xv, y: yv }] : [];
    });
    const fifths = medianTrace(values, 5, "", "formula").points;
    const change = fifths.at(-1)!.y - fifths[0]!.y;
    if (word === "Flat:") expect(Math.abs(change), chart).toBeLessThan(3);
    else if (word === "Falls:") expect(change, chart).toBeLessThan(0);
    else throw new Error(`${chart}: lead-in "${word}" is neither Flat nor Falls`);
    expect(lead.textContent, `${chart} names the line it describes`).toContain("dashed line");
  }
});

test("the thirds' percentile spreads are printed as computed", () => {
  const third = bands(bundle.districts, (d) => d.economically_disadvantaged);
  const spread = (q: number) => {
    const at = [0, 1, 2].map((band) =>
      percentile(
        bundle.districts
          .filter((d) => third.get(d) === band)
          .map((d) => d.outcome?.per_equivalent_pupil)
          .filter((v): v is number => v != null),
        q,
      ),
    );
    return Math.round(Math.max(...at) - Math.min(...at));
  };
  const note = [...card.querySelectorAll("p.note")].find((p) => p.textContent?.includes("tenth percentiles"))!;
  const printed = /tenth percentiles are \$([\d,]+) apart and their ninetieths \$([\d,]+)/.exec(
    note.textContent!.replace(/\s+/g, " "),
  );
  expect(printed, "the sentence still reads as written").not.toBeNull();
  expect(Number(printed![1]!.replace(/,/g, ""))).toBe(spread(0.1));
  expect(Number(printed![2]!.replace(/,/g, ""))).toBe(spread(0.9));
});

test("the pair's two plot areas are one height at every width", () => {
  // At 375 one x-axis name wrapped and the other did not, and a fixed total height made the same
  // fifty points of Performance Index 9% taller on one panel.
  const heights = (chart: string) =>
    new Map(
      [...card.querySelectorAll(`[data-chart="${chart}"] .chart-at`)].map((at) => [
        at.getAttribute("data-at"),
        at.querySelector("clipPath rect")?.getAttribute("height"),
      ]),
    );
  const weighted = heights("weighted-spending");
  expect(weighted.size).toBeGreaterThanOrEqual(3);
  expect(heights("enrolled-spending")).toEqual(weighted);
});

test("no dot on the pair is cut by the frame", () => {
  // Batavia Local sat on the shared spending scale's left end and was drawn half outside it.
  for (const chart of Object.keys(PAIR)) {
    for (const at of card.querySelectorAll(`[data-chart="${chart}"] .chart-at`)) {
      const clip = at.querySelector("clipPath rect")!;
      const [x, y, w, h] = ["x", "y", "width", "height"].map((k) => Number(clip.getAttribute(k)));
      const dots = [...at.querySelectorAll(".scatter-dot circle")];
      expect(dots.length).toBeGreaterThan(500);
      for (const dot of dots) {
        const cx = Number(dot.getAttribute("cx"));
        const cy = Number(dot.getAttribute("cy"));
        const where = `${chart} ${at.getAttribute("data-at")} (${cx}, ${cy})`;
        expect(cx - x!, where).toBeGreaterThanOrEqual(DOT.radius.plain);
        expect(x! + w! - cx, where).toBeGreaterThanOrEqual(DOT.radius.plain);
        expect(cy - y!, where).toBeGreaterThanOrEqual(DOT.radius.plain);
        expect(y! + h! - cy, where).toBeGreaterThanOrEqual(DOT.radius.plain);
      }
    }
  }
});
