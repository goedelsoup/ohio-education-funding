/**
 * The `/what-changed` page, pinned on the figures #641 states (#642's phase 3).
 *
 * Each provision card computes its figures from the feed; these are the values the issue worked
 * by hand and `crates/project/tests/a_districts_years_can_be_read_side_by_side.rs` pins in Rust,
 * read here through the page's own functions so a renderer that summed the wrong line fails.
 */
import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { levelChange, SHARE_COLUMN, shareChange, signedPct, signedPoints } from "../../src/lib/change.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { millions } from "../../src/lib/format.ts";
import * as routes from "../../src/lib/routes.ts";
import { median } from "../../src/lib/stats.ts";
import {
  dpia,
  foundationIdentity,
  phaseIn,
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

test("the fall in foundation aid is the phase-in's: the bases do not move (#695)", () => {
  const f = foundationIdentity(districts);
  // One base under all three years, so foundation aid moves only through what is paid above it.
  for (const x of f) expect(x.base).toBeCloseTo(6_395_281_649.73, 0);
  // What is left is the open enrollment clawback, in the districts it holds below their base. A
  // new term in foundation aid, or a clawback that grew, fails here.
  expect(f.map((x) => x.below)).toEqual([16, 0, 22]);
  for (const [i, v] of [3_073_454.87, 0, 3_037_536.64].entries()) expect(f[i]!.clawback).toBeCloseTo(v, 1);
  for (const x of f) expect(x.aid).toBeCloseTo(x.base + x.above - x.clawback, 2);
  // So the two −$114.5M on the page are one figure, to within the clawback's own change.
  const aid = f[2]!.aid - f[0]!.aid;
  const above = f[2]!.above - f[0]!.above;
  expect(millions(aid)).toBe("−$114.5M");
  expect(millions(above)).toBe("−$114.5M");
  expect(Math.abs(aid - above)).toBeLessThan(50_000);
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
  // By IRN, not by name: 607 districts share 580 names.
  const elida = allen.find((d) => d.irn === "045773")!;
  expect(signedPct(fall(elida))).toBe("−6.4%");
});

const { document } = parseHTML(
  `<main>${[
    renderTwoMeasures,
    renderDistribution,
    renderTargetedAssistance,
    renderDirectCertification,
    renderReappraisal,
    renderPhaseIn,
  ]
    .map((render) => render(districts))
    .join("")}</main>`,
);

test("the page says the two −$114.5M are one figure, and names the clawback", () => {
  const phase = text(document.querySelector(`#${routes.SECTIONS.changed.phaseIn}`));
  expect(phase).toContain("The funding bases sum to $6.40B in all three years");
  expect(phase).toContain("foundation aid fell $114.5M and the phase-in's payment above bases fell $114.5M");
  expect(phase).toContain(
    "the same money, to within $35,918. That difference is the open enrollment clawback, which holds some districts below their base: $3.1M across 16 districts in FY2025 and $3.0M across 22 in FY2027, and nothing in the FY2026 model.",
  );
  const first = document.querySelector(`#${routes.SECTIONS.changed.twoMeasures}`)!;
  expect(first.querySelector(`a[href="#${routes.SECTIONS.changed.phaseIn}"]`)).toBeTruthy();
});

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

test("no card lists every district: the directory and the county pages do (#693)", () => {
  // The by-county table was 609 rows and 437 KB of a 469 KB page, repeating both.
  for (const table of document.querySelectorAll("table")) {
    expect(table.querySelectorAll("tbody tr").length).toBeLessThan(20);
  }
  const link = document.querySelector(`#${routes.SECTIONS.changed.reappraisal} a[href^="/districts"]`);
  expect(link?.getAttribute("href")).toBe(routes.directorySorted(SHARE_COLUMN, "ascending"));
});

test("the state share change is in points between the two models, which publish it for all 609", () => {
  expect(districts.filter((d) => shareChange(d.biennium) == null)).toHaveLength(0);
  // Elida again: its share fell nearly seven points across the model years.
  const elida = districts.find((d) => d.irn === "045773")!;
  expect(signedPoints(shareChange(elida.biennium)!)).toBe("−6.7 pts");
  expect(signedPoints(-0.004)).toBe("−0.4 pts");
  expect(signedPoints(0.012)).toBe("+1.2 pts");
});

test("a year the files do not publish is a word, never a zero", () => {
  const card = text(document.querySelector(`#${routes.SECTIONS.changed.directCertification}`));
  expect(card).toContain("does not itemize DPIA");
});
