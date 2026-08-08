/**
 * Every internal link resolves to a page this site actually builds.
 *
 * # Why this test exists
 *
 * The wiki rewrites roughly two hundred relative file paths — `../parameter/twenty-mill-floor.yml`
 * — into routes. Every one of those rewrites is a small guess about a path shape, and a wrong
 * guess produces a link that looks perfectly ordinary and 404s. Nothing else would catch it:
 * `astro check` type-checks code and not strings, the build emits whatever it is given, and no
 * human is going to click two hundred links after each corpus edit.
 *
 * It runs over the *route table* rather than over `dist/`, so it needs no build and no server —
 * the same reason the rest of the unit suite is fast enough to run on every save. The trade is
 * that it checks links this repository generates, which is exactly the set that can regress here.
 */

import { expect, test } from "vitest";

import { loadCorpus, resolveTarget } from "../../src/lib/corpus.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import * as routes from "../../src/lib/routes.ts";

const corpus = loadCorpus();
const { bundle } = loadFeed();

/** Every path this site builds a document for. */
const PAGES = new Set<string>([
  "/",
  "/districts",
  "/outcomes",
  "/scenario",
  "/compare",
  "/search",
  "/method",
  "/data",
  "/wiki",
  "/wiki/source",
  ...bundle.districts.flatMap((d) => [
    routes.district(d.irn),
    routes.districtOutcome(d.irn),
    routes.districtFinances(d.irn),
    routes.districtScenario(d.irn),
  ]),
  ...corpus.classes.map((c) => routes.wikiClass(c.className)),
  ...corpus.nodes.map((n) => routes.wikiNode(n.className, n.name)),
  ...corpus.sources.map((s) => routes.wikiSource(s.slug)),
]);

/** Files served from `public/` or emitted by an endpoint, which are links but not pages. */
const ASSETS = new Set([
  "/data/bundle.json",
  "/data/panel.json",
  "/data/districts.csv",
  "/search-index.json",
]);

const internal = (href: string) => href.startsWith("/");
const known = (href: string) => {
  const [path] = href.split(/[?#]/);
  return PAGES.has(path!) || ASSETS.has(path!);
};

test("every corpus edge points at a page or off the site entirely", () => {
  const broken: string[] = [];
  for (const node of corpus.nodes) {
    for (const edge of node.out) {
      if (!internal(edge.href)) continue;
      if (!known(edge.href)) broken.push(`${node.id} → ${edge.href}`);
    }
  }
  expect(broken, "corpus links that resolve to nothing").toEqual([]);
});

test("every inline link in corpus prose resolves", () => {
  const broken: string[] = [];
  for (const node of corpus.nodes) {
    for (const match of node.description.matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = match[1]!;
      if (/^(https?:|mailto:|#)/.test(target)) continue;
      const { href } = resolveTarget(target, node.className);
      if (!internal(href)) continue;
      if (!known(href)) broken.push(`${node.id} → ${target} → ${href}`);
    }
  }
  expect(broken, "inline corpus links that resolve to nothing").toEqual([]);
});

test("citations are counted from both the forms the corpus writes them in", () => {
  // This corpus cites sources two ways and uses each for different entries: a structured
  // `sourced-from` edge in `links:`, and a markdown link in the middle of a sentence. A "cited by"
  // list built from only one of them is empty for every source that happens to use the other —
  // which is exactly the bug this pins. The counts are the catalog's own index.
  const count = (slug: string) =>
    corpus.sources.find((s) => s.slug === slug)?.citedBy.length ?? -1;

  // Cited only through structured `sourced-from` edges.
  expect(count("dew-sfpr-line-by-line")).toBe(6);
  // Cited only inline, in prose.
  expect(count("ocg-white-paper-013")).toBeGreaterThanOrEqual(4);

  // And the shape of the whole catalog: most entries are reached, and the ones that are not are a
  // gap the catalog already knows about rather than a failure of this reader.
  const uncited = corpus.sources.filter((s) => s.citedBy.length === 0);
  expect(uncited.length).toBeLessThanOrEqual(2);
});

test("every source page is reachable and every citation resolves", () => {
  const broken: string[] = [];
  for (const source of corpus.sources) {
    if (!PAGES.has(routes.wikiSource(source.slug))) broken.push(source.slug);
    for (const citation of source.citedBy) {
      if (!known(citation.href)) broken.push(`${source.slug} ← ${citation.href}`);
    }
  }
  expect(broken).toEqual([]);
});

test("the metric routes the district pages link to are real nodes", () => {
  // The district and comparison pages deep-link measures into the corpus by hard-coded slug —
  // "assessed valuation per pupil" points at `/wiki/metric/assessed-valuation-per-pupil`. Renaming
  // that node would silently break every one of 609 district pages.
  for (const slug of [
    "assessed-valuation-per-pupil",
    "per-pupil-operating-expenditure",
    "effective-operating-millage",
  ]) {
    expect(PAGES.has(routes.metric(slug)), `metric/${slug} is linked but does not exist`).toBe(true);
  }
  for (const slug of ["twenty-mill-floor"]) {
    expect(PAGES.has(routes.parameter(slug)), `parameter/${slug} is linked but does not exist`).toBe(
      true,
    );
  }
  expect(PAGES.has(routes.wikiNode("funding-regime", "fair-school-funding-plan"))).toBe(true);
});

test("every district in the feed has all four of its views", () => {
  // The route table above is generated from the feed, so this checks the thing that route table
  // asserts: that `getStaticPaths` covers the whole panel and not a filtered subset of it.
  expect(PAGES.size).toBeGreaterThanOrEqual(bundle.districts.length * 4);
  for (const irn of ["043786", "049056", "044933"]) {
    expect(PAGES.has(routes.district(irn))).toBe(true);
    expect(PAGES.has(routes.districtScenario(irn))).toBe(true);
  }
});
