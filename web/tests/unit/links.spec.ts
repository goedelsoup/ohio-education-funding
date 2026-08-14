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

import { FROM_CATALOG, FROM_DECISION, loadCorpus, resolveTarget } from "../../src/lib/corpus.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { counties } from "../../src/lib/county.ts";
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
  "/wiki/decision",
  "/counties",
  ...counties(bundle.districts).map((c) => routes.county(c.slug)),
  "/house",
  ...bundle.house_districts.map((h) => routes.houseDistrict(h.number)),
  "/senate",
  ...bundle.senate_districts.map((h) => routes.senateDistrict(h.number)),
  ...bundle.districts.flatMap((d) => [
    routes.district(d.irn),
    routes.districtOutcome(d.irn),
    routes.districtFinances(d.irn),
    routes.districtTaxes(d.irn),
    routes.districtScenario(d.irn),
  ]),
  ...corpus.classes.map((c) => routes.wikiClass(c.className)),
  ...corpus.nodes.map((n) => routes.wikiNode(n.className, n.name)),
  ...corpus.sources.map((s) => routes.wikiSource(s.slug)),
  ...corpus.decisions.map((d) => routes.wikiDecision(d.slug)),
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

/**
 * A link is off the site only if `resolveTarget` decided so, never because it looks that way.
 *
 * `internal(href)` on its own is the hole both tests below used to have. An unresolvable target
 * comes back with `href` set to the raw relative path — `../../skills/deduction.md` — which is not
 * site-absolute, so `if (!internal(href)) continue` filed it under "external, none of our
 * business" and skipped it. That is precisely the set the tests exist to catch, and 40 of them
 * shipped with both tests green.
 */
const placed = (target: string, fromClass: string, where: string, broken: string[]): boolean => {
  const { href, resolved } = resolveTarget(target, fromClass);
  if (!resolved) {
    broken.push(`${where} → ${target} (no rule places this shape)`);
    return false;
  }
  if (internal(href) && !known(href)) {
    broken.push(`${where} → ${target} → ${href} (no such page)`);
    return false;
  }
  return true;
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
  // `linkText`, not `description`: the corpus writes links in property values and `findings:`
  // blocks too, and those were only ever checked by accident, back when the two fields were one.
  for (const node of corpus.nodes) {
    for (const match of node.linkText.matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = match[1]!;
      if (/^(https?:|mailto:|#|\/)/.test(target)) continue;
      placed(target, node.className, node.id, broken);
    }
  }
  expect(broken, "inline corpus links that resolve to nothing").toEqual([]);
});

test("every inline link in a catalog entry resolves", () => {
  // Catalog entries are the largest single writer of link targets and use shapes no node writes:
  // a bare sibling `ocg-white-paper-013.md`, and `../corpus/metric/` for a whole class. Nineteen
  // of the first and four of the second were emitted as raw relative hrefs.
  const broken: string[] = [];
  for (const source of corpus.sources) {
    for (const match of source.body.matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = match[1]!;
      if (/^(https?:|mailto:|#|\/)/.test(target)) continue;
      placed(target, FROM_CATALOG, `catalog/${source.slug}`, broken);
    }
  }
  expect(broken, "catalog links that resolve to nothing").toEqual([]);
});

test("every inline link in a decision record resolves", () => {
  // The third writer of link targets, and it uses a shape neither of the others does: a bare
  // sibling `name.yml` meaning another *decision*. Left to the sibling-node branch that would
  // have become `/wiki/<some class>/the-three-streams-of-mr81` — resolved, and a 404.
  const broken: string[] = [];
  for (const decision of corpus.decisions) {
    for (const match of decision.linkText.matchAll(/\]\(([^)\s]+)\)/g)) {
      const target = match[1]!;
      if (/^(https?:|mailto:|#|\/)/.test(target)) continue;
      placed(target, FROM_DECISION, `decisions/${decision.slug}`, broken);
    }
  }
  expect(broken, "decision links that resolve to nothing").toEqual([]);
});

test("a decision record is a page, and the plural shape that used to be one is not", () => {
  // `../decisions/name.yml` is written from all three places and has to land on the same page from
  // each. It used to resolve to GitHub, which was honest while nothing published these; the shape
  // that was never honest is `/wiki/decisions/…`, plural, from reading the subtree as a class.
  for (const from of [FROM_CATALOG, FROM_DECISION, "metric"]) {
    expect(resolveTarget("../decisions/the-open-item-audit.yml", from)).toMatchObject({
      href: "/wiki/decision/the-open-item-audit",
      resolved: true,
    });
  }
  // A sibling, which only a decision record writes.
  expect(resolveTarget("the-three-streams-of-mr81.yml", FROM_DECISION)).toMatchObject({
    href: "/wiki/decision/the-three-streams-of-mr81",
    resolved: true,
  });
  // And the same string from a node still means a node in that node's own class.
  expect(resolveTarget("twenty-mill-floor.yml", "parameter").href).toBe(
    "/wiki/parameter/twenty-mill-floor",
  );
  // The subtrees that are still unpublished still go to the repository.
  expect(resolveTarget("../../skills/deduction.md", "metric").href).toMatch(/^https:\/\/github\.com/);
});

test("the decisions that carry corrections are found, and the quotations are not", () => {
  /*
   * The rule this whole feature turns on, pinned against the real records.
   *
   * A decision record uses blockquotes for two jobs — quoting something superseded, and saying
   * something above is wrong — and they are told apart by the second opening with strong emphasis.
   * That is a measured property of the corpus, not a convention anyone declared, so it needs a
   * test: the day a correction is written opening with plain prose, this fails instead of the page
   * quietly filing a withdrawal as a quotation.
   */
  const count = (slug: string) =>
    corpus.decisions.find((d) => d.slug === slug)?.corrections ?? -1;

  // Four corrections: a supersession, a correction, and two expired rejections.
  expect(count("the-directory-cannot-say-why")).toBe(4);
  // A `CORRECTED by` and a `RESOLVED by`.
  expect(count("child-nutrition-connector")).toBe(2);
  // Blockquotes, and every one of them a quotation: the previous blocker, a docstring, a proposal.
  expect(count("reading-an-amending-act")).toBe(0);
  expect(count("scenario-models-ohio")).toBe(0);
  expect(count("the-four-kinds-of-parameter")).toBe(0);

  // And the corpus-wide shape: corrections are the minority, which is why hoisting them to the
  // top of a page is worth doing rather than being noise on every record.
  const corrected = corpus.decisions.filter((d) => d.corrections > 0);
  expect(corrected.length).toBeGreaterThan(0);
  expect(corrected.length).toBeLessThan(corpus.decisions.length / 2);
});

test("every decision page is reachable and every citation of one resolves", () => {
  const broken: string[] = [];
  for (const decision of corpus.decisions) {
    if (!PAGES.has(routes.wikiDecision(decision.slug))) broken.push(decision.slug);
    for (const citation of decision.citedBy) {
      if (!known(citation.href)) broken.push(`${decision.slug} ← ${citation.href}`);
    }
  }
  expect(broken).toEqual([]);

  // The inbound index is the half of this a file on disk does not have, and it has to be built
  // from all three writers. These are the counts the corpus itself carries.
  const cited = (slug: string) =>
    corpus.decisions.find((d) => d.slug === slug)?.citedBy.length ?? -1;
  // Cited from four catalog entries, which is more than any node cites it.
  expect(cited("report-card-connector")).toBeGreaterThanOrEqual(4);
  // Cited by another decision record, which is the shape the sibling branch exists for.
  expect(cited("the-three-streams-of-mr81")).toBeGreaterThan(0);
});

test("resolveTarget refuses a shape it cannot place rather than guessing", () => {
  // The negative side of the gate. Without these, "every target resolves" is satisfiable by a
  // resolver that returns `resolved: true` for anything.
  expect(resolveTarget("../../../etc/passwd", "metric").resolved).toBe(false);
  expect(resolveTarget("Some Prose Sentence.pdf", "metric").resolved).toBe(false);

  // A bare `name.yml` needs a class to be a sibling of, and a catalog entry has none. Left
  // unrefused this built `/wiki//twenty-mill-floor`, which is `resolved` and still a 404.
  expect(resolveTarget("twenty-mill-floor.yml", FROM_CATALOG).resolved).toBe(false);
  expect(resolveTarget("twenty-mill-floor.yml", "parameter")).toMatchObject({
    href: "/wiki/parameter/twenty-mill-floor",
    resolved: true,
  });

  // A bare `*.md` means different things from the two places it is written, and both are real.
  expect(resolveTarget("ocg-white-paper-013.md", FROM_CATALOG).href).toBe(
    "/wiki/source/ocg-white-paper-013",
  );
  expect(resolveTarget("ACTIONS.md", "metric").href).toMatch(
    /^https:\/\/github\.com\/.*\/\.yidam\/corpus\/metric\/ACTIONS\.md$/,
  );
  expect(resolveTarget("../../skills/deduction.md", "metric").href).toMatch(
    /^https:\/\/github\.com\/.*\/\.yidam\/skills\/deduction\.md$/,
  );
  expect(resolveTarget("../corpus/metric/", FROM_CATALOG).href).toBe("/wiki/metric");
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
  expect(PAGES.size).toBeGreaterThanOrEqual(bundle.districts.length * 5);
  for (const irn of ["043786", "049056", "044933"]) {
    expect(PAGES.has(routes.district(irn))).toBe(true);
    expect(PAGES.has(routes.districtScenario(irn))).toBe(true);
    expect(PAGES.has(routes.districtTaxes(irn))).toBe(true);
  }
});
