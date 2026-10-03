/**
 * The appropriation series, and the two ways a chart of it misleads.
 *
 * It is the only block in the feed that is an input to the funding system rather than an output,
 * and the only long series whose nominal and real readings support opposite sentences. Both are
 * tested here; the third hazard — dividing it by a pupil count — has no test because the block
 * carries no denominator, and `denominators.ts` records why.
 */

import { readFileSync } from "node:fs";

import { expect, test } from "vitest";

import {
  BILLIONS,
  fromActs,
  fromCatalog,
  growth,
  inBase,
  ONE_TIME,
  RECENT,
  renderAppropriations,
} from "../../src/lib/appropriations.ts";
import { parseHTML } from "linkedom";

import { truncatedDomain } from "../../src/lib/plot/spec.ts";
import { INK, SERIES } from "../../src/lib/plot/tokens.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { baseYear } from "../../src/lib/real.ts";

const { bundle } = loadFeed();
const rows = bundle.appropriations;
const base = baseYear(
  bundle.deflator!,
  rows.map((r) => r.fiscal_year),
);

test("the series is continuous from FY1998, and three publications make it so", () => {
  // It began at FY2002 with four holes in it — FY2006, FY2007, FY2012, FY2013 — which the Catalog
  // closed. The acts themselves then took it back four more years.
  expect(rows.length).toBe(30);
  const years = rows.map((r) => r.fiscal_year);
  expect(years[0]).toBe(1998);
  expect(years[years.length - 1]).toBe(2027);
  expect(years).toEqual(Array.from({ length: 30 }, (_, i) => 1998 + i));
});

test("exactly the four years before the workbook series come from the acts", () => {
  /*
   * And no further. The legislature's own index of what it holds stops at the 122nd General
   * Assembly, so FY1998 is a wall rather than a stopping point — if this list ever grows
   * downward, something has been read from a source that does not exist.
   */
  expect(fromActs(rows)).toEqual([1998, 1999, 2000, 2001]);
  for (const row of rows.filter((r) => r.source === "act")) {
    expect(row.enacted, `FY${row.fiscal_year}`).toBeGreaterThan(5e9);
    expect(row.foundation_funding, `FY${row.fiscal_year}`).toBeGreaterThan(2e9);
    expect(row.items, `FY${row.fiscal_year}`).toBeGreaterThan(50);
  }
});

test("the page says the four act years are read from the acts and why they stop", () => {
  // A reader meeting FY1998 beside FY2002 has to be told they come from different documents, and
  // that the series ends where it does because the publisher ends rather than because nobody
  // looked. Both sentences are on the card.
  const html = renderAppropriations(rows, bundle.deflator!, base, "nominal");
  expect(html).toContain("read from the acts");
  expect(html).toContain("stops at the 122nd");
  expect(html).toContain("appropriated twice");
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

test("the real view names the years it drops and the span the index covers", () => {
  // It said "2 years are absent" and left a reader to find which (#705).
  const kept = new Set(inBase(rows, bundle.deflator, base!, "real").map((r) => r.fiscal_year));
  const dropped = rows.filter((r) => !kept.has(r.fiscal_year)).map((r) => r.fiscal_year);
  const covered = bundle.deflator!.points.map((p) => p.fiscal_year);
  const html = renderAppropriations(rows, bundle.deflator, base, "real").replace(/\s+/g, " ");
  expect(dropped.length).toBeGreaterThan(0);
  for (const year of dropped) expect(html).toContain(`FY${year}`);
  expect(html).toContain(`covers FY${Math.min(...covered)} through FY${Math.max(...covered)}`);
  expect(renderAppropriations(rows, bundle.deflator, base, "nominal")).not.toContain("absent here");
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

test("the truncated-axis annotation does not understate the truncation", () => {
  /*
   * The chart's y axis does not start at zero, and the only thing on the chart that says so is one
   * line of text rendered with the card's own dollar format. So the format has to resolve the
   * number it is annotating.
   *
   * It did not. The card wrote billions as `$15bn` — zero decimal places, and a unit that appears
   * nowhere else on this site, where `millions()` writes `$7.28B`. Applied to an axis starting at
   * $1.24bn that produced *"axis starts at $1bn, not zero"*, understating the truncation by a
   * fifth. An annotation that misstates the thing it exists to disclose is worse than one that is
   * missing, because a reader who reads it stops looking.
   *
   * Checked by parsing the annotation back, which is what a reader does with it.
   */
  const values = rows.flatMap((r) => [r.enacted, r.foundation_funding]);
  const [min] = truncatedDomain(values);

  const annotation = BILLIONS(min);
  const parsed = Number(annotation.replace(/[$,BM]/g, "")) * (annotation.endsWith("B") ? 1e9 : 1e6);
  expect(Math.abs(parsed / min - 1)).toBeLessThan(0.01);

  // And it is the site's unit, not one of the card's own.
  expect(annotation).toMatch(/^\$[\d,]+\.\d{1,2}[BM]$/);
});

/** The first drawing on a rendered card, and its foot's words with the whitespace taken out. */
function drawing(html: string): { svg: SVGSVGElement; foot: string } {
  const svg = parseHTML(`<div>${html}</div>`).document.querySelector("svg")! as unknown as SVGSVGElement;
  const foot = [...svg.querySelectorAll("g.axis-foot text")].map((t) => t.textContent).join(" ");
  return { svg, foot: foot.replace(/\s+/g, "") };
}

test("the line labelled the formula is the formula's hue, and the whole appropriation is not", () => {
  /*
   * #706. `seriesSpec` gave its second line the guarantee's hue whatever it was, and the second
   * line here is the formula's own appropriation: "the formula" was drawn orange.
   */
  const { svg } = drawing(renderAppropriations(rows, bundle.deflator, base, "nominal"));
  expect(svg.querySelector(".series-b")!.getAttribute("stroke")).toBe(SERIES.formula);
  expect(svg.querySelector(".series-a")!.getAttribute("stroke")).toBe(INK.muted);
});

test("the nominal and constant-dollar drawings share one frame, and each foot names its basis", () => {
  /*
   * #706. Each basis was drawn on a domain truncated to its own line, so switching rescaled the
   * frame and the line kept nearly the same shape — the change the switch exists to show was
   * absorbed by the axis. One domain over both bases means one axis start on both.
   */
  const nominal = drawing(renderAppropriations(rows, bundle.deflator, base, "nominal")).foot;
  const real = drawing(renderAppropriations(rows, bundle.deflator, base, "real")).foot;
  const start = (foot: string) => /axisstartsat(.+?),notzero/i.exec(foot)?.[1];
  expect(start(nominal)).toBeDefined();
  expect(start(real)).toBe(start(nominal));
  expect(nominal).toContain("Inthedollarsofeachyear");
  expect(real).toContain(`InconstantFY${base}dollars`);
});

/** Each line's enacted amount by fiscal year, out of the crate's fixture. */
function enactedLines(): Map<string, Map<number, { amount: number; bill: string; title: string }>> {
  const text = readFileSync(new URL("../../../crates/project/fixtures/appropriation-lines.csv", import.meta.url), "utf8");
  const [head, ...body] = text.trim().split("\n");
  const cols = head!.split(",");
  const at = (name: string) => cols.indexOf(name);
  const out = new Map<string, Map<number, { amount: number; bill: string; title: string }>>();
  for (const line of body) {
    const cells = line.split(",");
    expect(cells, line).toHaveLength(cols.length);
    if (cells[at("kind")] !== "enacted") continue;
    const ali = cells[at("line_item")]!;
    const year = Number(cells[at("fiscal_year")]);
    const years = out.get(ali) ?? new Map();
    const prior = years.get(year)?.amount ?? 0;
    years.set(year, {
      amount: prior + Number(cells[at("amount")]),
      bill: cells[at("bill")]!,
      title: cells[at("title")]!,
    });
    out.set(ali, years);
  }
  return out;
}

test("the FY2024 spike the card names is the line the fixture says, and the year's largest rise", () => {
  /*
   * #708. The card names one line as the whole of the spike, from a constant, because the feed
   * carries totals and not lines. This is what keeps the constant honest: the amount, the act,
   * that it ends the next year, and that no other line rose as far into the same year.
   */
  const lines = enactedLines();
  const relief = lines.get(ONE_TIME.line)!;
  const year = relief.get(ONE_TIME.fiscal_year)!;
  expect(year.amount).toBe(ONE_TIME.amount);
  expect(year.title.toLowerCase()).toBe(ONE_TIME.title);
  expect(`H.B. ${year.bill.replace(/^hb/, "")}`).toBe(ONE_TIME.act);
  expect(relief.get(ONE_TIME.fiscal_year + 1)?.amount).toBe(0);

  const rise = (years: Map<number, { amount: number }>) =>
    (years.get(ONE_TIME.fiscal_year)?.amount ?? 0) - (years.get(ONE_TIME.fiscal_year - 1)?.amount ?? 0);
  const [largest] = [...lines].sort(([, a], [, b]) => rise(b) - rise(a));
  expect(largest![0]).toBe(ONE_TIME.line);
});

test("the card names the spike in its note and marks it on its chart", () => {
  const html = renderAppropriations(rows, bundle.deflator, base, "nominal").replace(/\s+/g, " ");
  expect(html).toContain(`<code>${ONE_TIME.line}</code>`);
  expect(html).toContain(`which ${ONE_TIME.act} appropriated for that year alone`);
  expect(html).toContain('class="series-callout"');
});

test("the table opens on the newest years and folds the rest, so the chart after it is near", () => {
  // #708: thirty rows put 2,900px between this chart and the next at 1280.
  const { document } = parseHTML(renderAppropriations(rows, bundle.deflator, base, "nominal"));
  const [open, rest] = [...document.querySelectorAll("table")];
  expect(open!.closest("details")).toBeNull();
  expect(rest!.closest("details.table-rest")).not.toBeNull();
  const years = (t: Element) => [...t.querySelectorAll("tbody th")].map((th) => th.textContent);
  expect(years(open!)).toHaveLength(RECENT);
  expect(years(open!)[0]).toBe(`FY${rows[rows.length - 1]!.fiscal_year}`);
  // Every year is still in the document, once.
  expect([...years(open!), ...years(rest!)].sort()).toEqual(rows.map((r) => `FY${r.fiscal_year}`));
});
