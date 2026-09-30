/**
 * The answer before the apparatus, on every data route (#549).
 *
 * A data page has three tiers: the answer (a chart, a table, the tiles), how to read it, and the
 * apparatus — where the numbers come from and what the page is not. #549 found the third tier
 * ahead of the first on `/history`, whose survey provenance card sat above both of its charts, and
 * marked every such card `.card.apparatus` so that the order is a property of the bytes rather
 * than of whoever last edited the page.
 *
 * # What counts as an answer
 *
 * A chart (`svg.plot`), a data table (any `<table>` not marked `.prose` — a prose table is a
 * sentence laid out in cells, not a figure), the headline `.tiles`, or anything else drawn as a
 * picture and named with `role="img"`, which is how the district's aid bar is exposed. Start tags
 * in document order, over `<main>` only, so the header and footer are never either tier.
 *
 * # What it does not check
 *
 * Pages that carry no apparatus at all — `/statewide`, `/districts`, `/compare` and the counties
 * today — can only be checked for having an answer, and are. The homepage is a way in rather than
 * an answer (its first block finds a district), so it is held to the second rule only: nothing it
 * carries is apparatus ahead of anything else. Wiki nodes are not data routes; their apparatus is
 * the revisions list, and it is last by construction in the node template.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, test } from "vitest";

import { DIST, pages } from "./artefact.ts";

const ANSWER = /<svg\b[^>]*\bclass="plot\b|<table\b(?![^>]*\bclass="[^"]*\bprose\b)|\bclass="tiles"|\brole="img"/;
const APPARATUS = /\bclass="[^"]*\bapparatus\b/;

/** The data routes #549 names, by the file a host serves for each. */
const NAMED = [
  "statewide.html",
  "districts.html",
  "method.html",
  "outcomes.html",
  "history.html",
  "bounds.html",
  "compare.html",
];

const DASHBOARD = /^district\/\d{6}\.html$/;
const COUNTY = /^county\/[a-z-]+\.html$/;

/** Where, in `<main>`, the first answer and the first apparatus block start; -1 for neither. */
function anatomy(html: string): { answer: number; apparatus: number } {
  const main = html.slice(html.indexOf("<main"), html.indexOf("</main>"));
  return { answer: main.search(ANSWER), apparatus: main.search(APPARATUS) };
}

/** What is wrong with a data page's order, or `null`. */
function misordered(html: string): string | null {
  const { answer, apparatus } = anatomy(html);
  if (answer === -1) return "has no chart, data table, tiles or drawn figure in its main";
  if (apparatus !== -1 && apparatus < answer) return "opens with apparatus ahead of its first answer";
  return null;
}

const relative = (file: string) => file.slice(DIST.length + 1);
const read = (route: string) => readFileSync(join(DIST, route), "utf8");

describe("answer before apparatus", () => {
  test("holds on every data route, every dashboard and every county", () => {
    const routes = pages()
      .map(relative)
      .filter((route) => NAMED.includes(route) || DASHBOARD.test(route) || COUNTY.test(route));
    // 7 named + 609 dashboards + 88 counties. A pattern that silently matched nothing would pass.
    expect(routes.length, "the routes this is about were all found").toBe(NAMED.length + 609 + 88);

    const wrong = routes.flatMap((route) => {
      const problem = misordered(read(route));
      return problem == null ? [] : [`${route}: ${problem}`];
    });
    expect(wrong.slice(0, 5)).toEqual([]);
  });

  test("the apparatus it holds down is there to hold", () => {
    // Every route #549 tagged, so a tag lost in a rewrite reads here rather than as a green sweep
    // over pages that no longer carry anything to order.
    for (const route of ["method.html", "outcomes.html", "history.html", "bounds.html", "district/043786.html"]) {
      expect(anatomy(read(route)).apparatus, route).toBeGreaterThan(-1);
    }
  });

  test("and the homepage carries no apparatus ahead of anything", () => {
    const { answer, apparatus } = anatomy(read("index.html"));
    expect(apparatus === -1 || (answer !== -1 && answer < apparatus)).toBe(true);
  });

  test("flags a real page with its apparatus moved back to the top", () => {
    // `/history` as it was before #549: the Census provenance card first, the charts after it.
    const page = read("history.html");
    expect(misordered(page)).toBeNull();
    const doctored = page.replace(/<\/h1>/, '</h1><div class="card apparatus"><p>Provenance.</p></div>');
    expect(doctored).not.toBe(page);
    expect(misordered(doctored)).toBe("opens with apparatus ahead of its first answer");
  });

  test("flags a real page with its answer taken away", () => {
    const page = read("compare.html");
    const doctored = page.replace(/<table\b/g, "<div");
    expect(doctored).not.toBe(page);
    expect(misordered(doctored)).toBe("has no chart, data table, tiles or drawn figure in its main");
  });

  test("does not take a prose table for an answer", () => {
    // `/method`'s pupil-count table is a table of sentences. If that counted, a page could put its
    // apparatus below a prose table and above every figure it has.
    const { answer, apparatus } = anatomy(
      '<main><table class="prose"></table><div class="card apparatus"></div><svg class="plot"></svg></main>',
    );
    expect(apparatus).toBeGreaterThan(-1);
    expect(answer).toBeGreaterThan(apparatus);
  });
});
