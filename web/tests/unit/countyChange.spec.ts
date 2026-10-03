/**
 * The county page's who-gained card, pinned on Allen County (#640).
 *
 * The four readings of "Lima was one of only two Allen districts cut" that #642 worked by hand —
 * each a step and a measure of the same comparison, so each is pinned here as the card computes it.
 */
import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { signedPct } from "../../src/lib/change.ts";
import { counties } from "../../src/lib/county.ts";
import {
  finding,
  renderWhoGained,
  stepSentence,
  tally,
  withoutTargetedAssistance,
} from "../../src/lib/countyChange.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import * as routes from "../../src/lib/routes.ts";
import type { District } from "../../src/lib/types.ts";

const all = counties(loadFeed().bundle.districts);
const ALLEN = all.find((c) => c.name === "Allen")!;
const irns = (list: readonly District[]) => list.map((d) => d.irn).sort();
const byIrn = (irn: string) => ALLEN.districts.find((d) => d.irn === irn)!;

const LIMA = "044222";
const ELIDA = "045773";
const DELPHOS = "043885";

test("Allen has the nine districts #638 pinned", () => {
  expect(ALLEN.districts).toHaveLength(9);
});

test("on total state support FY2025 to FY2026, Lima and Elida are the two cut", () => {
  const t = tally(ALLEN.districts, "total", 0, 1);
  expect(irns(t.fall)).toEqual([LIMA, ELIDA].sort());
  expect(t.rise).toHaveLength(7);
  expect(stepSentence(ALLEN.districts, "total", 0, 1)).toContain("The 2 that fell: Elida Local and Lima City.");
});

test("on foundation aid the same step, Lima does not fall and four neighbours do", () => {
  const t = tally(ALLEN.districts, "foundation", 0, 1);
  expect(t.fall).toHaveLength(4);
  expect(irns(t.fall)).not.toContain(LIMA);
  expect(irns(t.rise)).toContain(LIMA);
});

test("across the whole biennium, five of nine are cut on total state support", () => {
  expect(tally(ALLEN.districts, "total", 0, 2).fall).toHaveLength(5);
});

test("without the repealed supplement, Lima rose — and it is Allen's only recipient", () => {
  const recipients = ALLEN.districts.filter((d) => withoutTargetedAssistance(d) != null);
  expect(irns(recipients)).toEqual([LIMA]);
  const lima = withoutTargetedAssistance(byIrn(LIMA))!;
  // The same figures `without_targeted_assistance_lima_rose_by_under_two_percent` pins in
  // crates/project: +1.95%, not the +1.7% #642 printed (#649).
  expect(lima.dollars).toBeCloseTo(723_412.53, 2);
  expect(lima.ratio).toBeCloseTo(0.019458, 6);
});

test("a district with no supplement in FY2025 has no without-it figure", () => {
  expect(withoutTargetedAssistance(byIrn(ELIDA))).toBeNull();
});

const { document } = parseHTML(`<main>${renderWhoGained(ALLEN)}</main>`);
const card = document.querySelector(`#${routes.SECTIONS.county.whoGained}`)!;
const text = (n: Element | null | undefined) => (n?.textContent ?? "").replace(/\s+/g, " ").trim();

test("the card has both measures under a no-script toggle, total checked", () => {
  expect(card).toBeTruthy();
  expect(card.querySelector(".year-chip")).toBeTruthy();
  expect(card.querySelectorAll(".measure-panel")).toHaveLength(2);
  expect(card.querySelector('input[data-measure="total"]')?.hasAttribute("checked")).toBe(true);
});

test("each table is sortable, written largest biennium rise first", () => {
  for (const table of card.querySelectorAll("table[data-sortable]")) {
    const sorted = [...table.querySelectorAll('th[aria-sort="descending"] button')].map((b) => b.getAttribute("data-sort"));
    expect(sorted).toEqual(["step-0-2"]);
    const rows = [...table.querySelectorAll("tbody tr")];
    expect(rows).toHaveLength(9);
    const keys = rows.map((r) => Number(r.querySelectorAll("td")[5]!.getAttribute("data-sort-value")));
    expect(keys).toEqual([...keys].sort((a, b) => b - a));
  }
});

test("Lima's row names the repealed supplement as the line that moved it most", () => {
  const total = card.querySelector('.measure-panel[data-measure="total"]')!;
  const lima = [...total.querySelectorAll("tbody tr")].find((r) => text(r.querySelector("th")) === "Lima City")!;
  expect(lima.querySelector("th a")?.getAttribute("href")).toBe(routes.districtChange(LIMA));
  expect(text(lima)).toContain("Supplemental targeted assistance (repealed by H.B. 96)");
  expect(text(lima)).toContain("−$1,859,367");
  // The foundation panel has no line column: foundation aid is one line.
  expect(card.querySelectorAll('.measure-panel[data-measure="foundation"] thead th')).toHaveLength(9);
});

test("each measure draws its county between the sentences and the table, in the table's order", () => {
  // #661: the card was a table and a bullet list, where the district tab draws the same comparison.
  for (const measure of ["total", "foundation"] as const) {
    const panel = card.querySelector(`.measure-panel[data-measure="${measure}"]`)!;
    const parts = [...panel.children].map((el) =>
      el.matches("ul") ? "sentences" : el.matches("[data-chart]") ? "chart" : el.matches(".scroll") ? "table" : null,
    );
    expect(parts.filter(Boolean)).toEqual(["sentences", "chart", "table"]);
    const chart = panel.querySelector(`[data-chart="neighbors-${measure}"]`)!;
    // No district is the subject on a county page, so none is marked and the ends alone carry values.
    expect(chart.innerHTML).not.toContain("is marked");
    const wide = chart.querySelector('[data-at="wide"]')!;
    const values = [...wide.querySelectorAll(".bar-value text, text.bar-value")].map((t) => t.textContent);
    const rows = [...panel.querySelectorAll("tbody tr")];
    const step = (r: Element) => signedPct(Number(r.querySelectorAll("td")[5]!.getAttribute("data-sort-value")));
    expect(values).toEqual([step(rows[0]!), step(rows.at(-1)!)]);
    expect(chart.innerHTML).toContain("FY2025 to FY2027");
  }
});

test("the card leads on what its first chart draws, in a number", () => {
  // #653's rule, which the card owes once it draws (#661): five of Allen's nine are cut.
  expect(text(card.querySelector("p.note strong"))).toBe(
    "On total state support from FY2025 to FY2027, 4 of the 9 districts here rose and 5 fell.",
  );
  const single = all.find((c) => c.districts.length === 1)!;
  expect(finding(single)).toMatch(/^.+'s total state support (rose|fell|did not move).* from FY2025 to FY2027\.$/);
});

test("a county of one or two districts draws no chart", () => {
  for (const c of all.filter((c) => c.districts.length < 3)) {
    expect(renderWhoGained(c), c.name).not.toContain("<svg");
  }
});

test("state share falls in every Allen district, and Delphos reaches the minimum", () => {
  expect(text(card)).toContain("falls in every district here");
  expect(text(card)).toContain("Delphos City is at the formula's minimum state share in FY2027");
  expect(byIrn(DELPHOS).at_minimum_state_share).toBe(true);
});

test("a missing year is a word, never a zero", () => {
  // FY2025's payment report publishes no state share, so the card has no FY2025 share column.
  expect(text(card.querySelector("thead"))).not.toContain("State share, FY2025");
});

test("every county renders, the single-district ones included", () => {
  expect(all).toHaveLength(88);
  for (const c of all) {
    const html = renderWhoGained(c).replace(/\s+/g, " ");
    expect(html, c.name).toContain(`id="${routes.SECTIONS.county.whoGained}"`);
    expect(html, c.name).toContain("filed under the one county the department assigns");
  }
  const single = all.find((c) => c.districts.length === 1)!;
  expect(stepSentence(single.districts, "total", 0, 1)).toMatch(/(rose|fell|did not move)\.<\/li>$/);
});

test("the card sends a reader to the statewide reading of the same steps (#694)", () => {
  for (const c of all) {
    const hrefs = [...parseHTML(renderWhoGained(c)).document.querySelectorAll("a")].map((a) => a.getAttribute("href"));
    expect(hrefs, c.name).toContain(routes.WHAT_CHANGED);
  }
});
