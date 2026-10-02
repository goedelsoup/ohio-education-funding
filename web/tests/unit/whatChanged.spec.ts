/**
 * The `/what-changed` page, pinned on the figures #641 states (#642's phase 3).
 *
 * Each provision card computes its figures from the feed; these are the values the issue worked
 * by hand and `crates/project/tests/a_districts_years_can_be_read_side_by_side.rs` pins in Rust,
 * read here through the page's own functions so a renderer that summed the wrong line fails.
 */
import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { levelChange } from "../../src/lib/change.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { millions } from "../../src/lib/format.ts";
import * as routes from "../../src/lib/routes.ts";
import { median } from "../../src/lib/stats.ts";
import {
  dpia,
  phaseIn,
  renderByCounty,
  renderDirectCertification,
  renderDistribution,
  renderPhaseIn,
  renderReappraisal,
  renderTargetedAssistance,
  renderTwoMeasures,
  shareBands,
  statewideLevels,
  summary,
  targetedAssistance,
} from "../../src/lib/whatChanged.ts";

const districts = loadFeed().alphabetical;
const text = (n: Element | null | undefined) => (n?.textContent ?? "").replace(/\s+/g, " ").trim();

test("the population is the 609 the three files share", () => {
  expect(districts).toHaveLength(609);
});

test("the two measures move in opposite directions: +$145.0M and −$114.5M", () => {
  const change = (m: "total" | "foundation") => {
    const l = statewideLevels(districts, m);
    return millions(l[2] - l[0]);
  };
  expect(change("total")).toBe("+$145.0M");
  expect(change("foundation")).toBe("−$114.5M");
  expect(summary(districts)).toBe(
    "From FY2025 to FY2027, total state support to Ohio's 609 districts rose $145.0M and foundation aid fell $114.5M, provision by provision.",
  );
});

test("the summary is a meta description: one sentence, at most 160 characters", () => {
  const s = summary(districts);
  expect(s.length).toBeLessThanOrEqual(160);
  expect(s).toMatch(/[.]$/);
  expect(s).not.toContain("…");
});

test("the repeal: $52.3M to 36 districts, 19 of them cut against 135 of the other 573", () => {
  const t = targetedAssistance(districts);
  expect(millions(t.dollars)).toBe("+$52.3M");
  expect([t.recipientsCut, t.recipients]).toEqual([19, 36]);
  expect([t.restCut, t.rest]).toEqual([135, 573]);
  // With the supplement out of both years, two recipients are still cut.
  expect(t.cutWithout).toBe(2);
});

test("DPIA falls 7.5% across the model years, in 562 districts", () => {
  const x = dpia(districts);
  expect(x.all.levels[0]).toBeCloseTo(567_673_868.76, 0);
  expect(x.all.levels[1]).toBeCloseTo(525_094_312.31, 0);
  expect(x.fell).toBe(562);
  // The two groups partition the 609 and their changes sum to the whole.
  expect(x.formula.districts + x.held.districts).toBe(609);
  const change = (g: { levels: readonly [number, number] }) => g.levels[1] - g.levels[0];
  expect(change(x.formula) + change(x.held)).toBeCloseTo(change(x.all), 2);
});

test("the phase-in rate is two-thirds, five-sixths and whole, read off the files", () => {
  const p = phaseIn(districts);
  expect(p.map((x) => x.rate.toFixed(4))).toEqual(["0.6667", "0.8333", "1.0000"]);
  expect(p.map((x) => x.above)).toEqual([425, 375, 314]);
  expect(p.map((x) => x.atBase)).toEqual([168, 234, 273]);
  // The rate rose and the phase-in paid less above bases, because the gap shrank faster.
  expect(p[2]!.paid).toBeLessThan(p[0]!.paid);
  expect(p[2]!.gap).toBeLessThan(p[0]!.gap);
});

test("the lower a district's state share, the larger the share of it a year takes away", () => {
  const bands = shareBands(districts);
  expect(bands.reduce((n, b) => n + b.districts.length, 0)).toBe(609);
  const fall = bands.map((b) =>
    median(b.districts.map((d) => d.biennium.observed[2]!.state_share! / d.biennium.observed[1]!.state_share! - 1)),
  );
  // At the minimum the share cannot fall; above it, the fall shrinks band by band.
  expect(fall[0]).toBe(0);
  for (let i = 2; i < fall.length; i++) expect(fall[i]!).toBeGreaterThan(fall[i - 1]!);
});

test("in Allen County, Elida and Shawnee fall most on foundation aid across the reappraisal year", () => {
  // FY2025 to FY2026, the step tax year 2024's reappraisal reaches. Both start on a low share.
  const allen = districts.filter((d) => d.county === "Allen");
  const fall = (d: (typeof allen)[number]) => levelChange(d.biennium, "foundation", 0, 1).ratio;
  const largest = [...allen].sort((a, b) => fall(a) - fall(b)).slice(0, 2);
  expect(largest.map((d) => d.name)).toEqual(["Elida Local", "Shawnee Local"]);
  const rows = [...document.querySelectorAll(`#${routes.SECTIONS.changed.byCounty} tbody tr[data-county="Allen"]`)];
  const elida = rows.find((r) => r.querySelector("th a")?.getAttribute("href") === routes.districtChange("045773"))!;
  expect(text(elida)).toContain("−6.4%");
});

const { document } = parseHTML(
  `<main>${[
    renderTwoMeasures,
    renderDistribution,
    renderTargetedAssistance,
    renderDirectCertification,
    renderReappraisal,
    renderPhaseIn,
    renderByCounty,
  ]
    .map((render) => render(districts))
    .join("")}</main>`,
);

test("every card is addressed and dated", () => {
  for (const id of Object.values(routes.SECTIONS.changed)) {
    const card = document.querySelector(`#${id}`);
    expect(card, id).toBeTruthy();
    expect(card!.querySelector(".year-chip"), id).toBeTruthy();
  }
});

test("every money figure in the measures card names its measure", () => {
  const rows = [...document.querySelectorAll(`#${routes.SECTIONS.changed.twoMeasures} tbody th`)].map(text);
  expect(rows).toEqual(["Total state support", "Foundation aid"]);
});

test("the distribution card carries both measures under the toggle", () => {
  const card = document.querySelector(`#${routes.SECTIONS.changed.distribution}`)!;
  expect(card.querySelectorAll(".measure-panel")).toHaveLength(2);
  expect(card.querySelector('input[data-measure="total"]')?.hasAttribute("checked")).toBe(true);
});

test("the county table is every district, each linking to its change tab", () => {
  const rows = [...document.querySelectorAll(`#${routes.SECTIONS.changed.byCounty} tbody tr`)];
  expect(rows).toHaveLength(609);
  // By IRN, not by name: 607 districts share 580 names.
  const hrefs = rows.map((row) => row.querySelector("th a")?.getAttribute("href"));
  expect(new Set(hrefs)).toEqual(new Set(districts.map((d) => routes.districtChange(d.irn))));
  expect(rows.filter((r) => r.getAttribute("data-county") === "Allen")).toHaveLength(9);
  const options = document.querySelectorAll(`#${routes.SECTIONS.changed.byCounty}-county option`);
  expect(options).toHaveLength(89);
});

test("a year the files do not publish is a word, never a zero", () => {
  const card = text(document.querySelector(`#${routes.SECTIONS.changed.directCertification}`));
  expect(card).toContain("does not itemize DPIA");
});
