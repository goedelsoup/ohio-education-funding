/**
 * Two chart choices on every district page, held against the build (#659).
 *
 * The base-cost and categoricals cards each drew a bar chart labelled in shares and then a table
 * with a Share column repeating those labels, clipped to "SHA" at 375. The categoricals chart
 * dropped its $0 programs under a heading saying six, which on 316 of 609 dashboards drew five
 * bars or fewer. And cash held — a fiscal-year series — was drawn as category bars running down
 * the page, on `/statewide` and on every district's finances page.
 *
 * Read off the bytes rather than in a browser because every district carries both cards, and the
 * defect was a population fact: one district page in a browser sees one district's zeroes.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vitest";

import { DIST } from "./artefact.ts";

/** The district dashboards, as `<irn>.html` beside each district's directory of sub-pages. */
const irns = readdirSync(join(DIST, "district"))
  .filter((name) => /^\d{6}\.html$/.test(name))
  .map((name) => name.slice(0, 6));

/** The text of a page from `marker` onwards, which is where a card's own markup starts. */
function from(html: string, marker: string, page: string): string {
  const at = html.indexOf(marker);
  expect(at, `${page} carries ${marker}`).toBeGreaterThanOrEqual(0);
  return html.slice(at);
}

/** The header cells of the first table in `html`, as text. */
function head(html: string): string[] {
  const thead = /<thead>([\s\S]*?)<\/thead>/.exec(html)![1]!;
  return [...thead.matchAll(/<th\b[^>]*>([\s\S]*?)<\/th>/g)].map(([, cell]) =>
    cell!.replace(/<[^>]+>/g, "").trim(),
  );
}

describe("a district's base-cost and categoricals tables", () => {
  test("carry the dollars, and leave the shares to the chart above them", () => {
    expect(irns.length).toBeGreaterThan(600);
    const wrong: string[] = [];
    for (const irn of irns) {
      const html = readFileSync(join(DIST, "district", `${irn}.html`), "utf8");
      const base = head(from(html, 'data-chart="base-cost"', irn));
      if (base.join("|") !== "Element|Amount") wrong.push(`${irn} base cost: ${base.join("|")}`);
      // A district with no categorical funding has no card, and nothing to repeat.
      if (!html.includes('data-chart="categoricals"')) continue;
      const programs = head(from(html, 'data-chart="categoricals"', irn));
      if (programs.join("|") !== "Program|Amount") wrong.push(`${irn} categoricals: ${programs.join("|")}`);
    }
    expect(wrong.slice(0, 10)).toEqual([]);
  });

  test("the categoricals chart draws the six parts its heading counts, a $0 program as a stub", () => {
    const short: string[] = [];
    let stubbed = 0;
    for (const irn of irns) {
      const html = readFileSync(join(DIST, "district", `${irn}.html`), "utf8");
      if (!html.includes('data-chart="categoricals"')) continue;
      const chart = from(html, 'data-chart="categoricals"', irn);
      // The first drawing of the chart; the narrow and wide copies draw the same bars.
      const fill = /<g[^>]*class="bar-fill"[^>]*>([\s\S]*?)<\/g>/.exec(chart)![1]!;
      const bars = fill.match(/<path\b/g)?.length ?? 0;
      if (bars !== 6) short.push(`${irn}: ${bars}`);
      const values = /<g[^>]*class="bar-value"[^>]*>([\s\S]*?)<\/g>/.exec(chart)![1]!;
      if (values.includes(">$0<")) stubbed++;
    }
    expect(short.slice(0, 10)).toEqual([]);
    // Over half the districts have a zero program, so a sweep that met none read a stale build.
    expect(stubbed).toBeGreaterThan(300);
  });
});

describe("cash held", () => {
  /** A cash chart's markup, up to the close of its wrapper. */
  const chartOf = (html: string, name: string, page: string): string => {
    const chart = from(html, `data-chart="${name}"`, page);
    return chart.slice(0, chart.indexOf('<p class="note">'));
  };

  test("is drawn as a line across the years, on /statewide and every district's finances page", () => {
    const bars: string[] = [];
    let lines = 0;
    const pages: [string, string][] = [
      [join(DIST, "statewide.html"), "statewide-cash"],
      ...irns.map((irn): [string, string] => [join(DIST, "district", irn, "finances.html"), "cash"]),
    ];
    for (const [file, name] of pages) {
      const html = readFileSync(file, "utf8");
      if (!html.includes(`data-chart="${name}"`)) continue;
      const chart = chartOf(html, name, file.slice(DIST.length));
      if (chart.includes('class="bar-fill"') || !chart.includes("series-a")) {
        bars.push(file.slice(DIST.length));
      } else {
        lines++;
      }
    }
    expect(bars.slice(0, 10)).toEqual([]);
    expect(lines).toBeGreaterThan(600);
  });
});
