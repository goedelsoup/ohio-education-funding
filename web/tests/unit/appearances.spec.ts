/**
 * How one built page is read for the corpus nodes it cites (#551). The whole-build half — that the
 * committed inversion is the build's, and that node pages render it — is
 * `tests/dist/appearances.spec.ts`.
 */

import { expect, test } from "vitest";

import { appearancesOf, citationsOf, routeOf } from "../../src/lib/appearances.ts";

test("a route folds only the segments that multiply it", () => {
  expect(routeOf("district/043786/outcome.html")).toEqual({
    path: "/district/043786/outcome",
    route: "/district/[irn]/outcome",
  });
  expect(routeOf("county/cuyahoga.html").route).toBe("/county/[slug]");
  expect(routeOf("house/78.html").route).toBe("/house/[n]");
  expect(routeOf("index.html")).toEqual({ path: "/", route: "/" });
  expect(routeOf("legislation.html").route).toBe("/legislation");
});

/** A page as `Base.astro` shapes it: a menu outside `<main>`, cards inside it. */
const page = (main: string, menu = "") =>
  `<html><head><title>Cleveland Municipal — Outcome — Ohio school funding</title></head><body>` +
  `<nav>${menu}</nav><main id="main">${main}</main></body></html>`;

test("a citation is read from main only, and named by the card it sits in", () => {
  const html = page(
    `<nav class="contents"><ul><li><a href="#pi">Performance, by year</a></li></ul></nav>` +
      `<div class="card" id="pi" data-part="pi"><h2>Ignored when the contents names it</h2>` +
      `<a href="/wiki/metric/performance-index">the index</a>` +
      `<a href="/wiki/metric/performance-index#description">again</a></div>` +
      `<div class="card" id="aside" data-part="aside"><h2><a class="section-anchor" href="#aside">#</a>` +
      `Poverty <span class="year-chip-wrap"><button class="year-chip">2024-25</button>` +
      `<span class="year-chip-def" aria-hidden="true">school year</span></span></h2>` +
      `<a href="/wiki/metric/performance-index">once more</a>` +
      `<a href="/wiki/source/ode-report-card">a source is not a node</a></div>`,
    `<a href="/wiki/metric/graduation-rate">in the menu</a>`,
  );
  expect([...citationsOf(html)]).toEqual([
    [
      "metric/performance-index",
      [
        { id: "pi", label: "Performance, by year" },
        { id: "aside", label: "Poverty" },
      ],
    ],
  ]);
});

test("the inversion counts a route's pages and prefers an exemplar as its example", () => {
  const cites = page(`<div class="card" id="pi"><h2>PI</h2><a href="/wiki/metric/performance-index">x</a></div>`);
  const silent = page(`<div class="card" id="pi"><h2>PI</h2></div>`);
  const built = [
    { file: "district/000001/outcome.html", html: cites },
    { file: "district/043786/outcome.html", html: cites },
    { file: "district/000002/outcome.html", html: silent },
    { file: "wiki/metric/performance-index.html", html: cites }, // the corpus is not inverted
  ];
  const [place] = appearancesOf(built, ["043786"])["metric/performance-index"]!;
  expect(place).toEqual({
    route: "/district/[irn]/outcome",
    pages: 2,
    of: 3,
    example: { href: "/district/043786/outcome", title: "Cleveland Municipal — Outcome" },
    sections: [{ id: "pi", label: "PI" }],
  });
  // With no exemplar among them, the first in path order.
  expect(appearancesOf(built)["metric/performance-index"]![0]!.example.href).toBe("/district/000001/outcome");
});
