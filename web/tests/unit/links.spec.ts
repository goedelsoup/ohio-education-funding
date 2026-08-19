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

import { renderBaseCostBuildUp } from "../../src/lib/basecost.ts";
import { FROM_CATALOG, FROM_DECISION, loadCorpus, resolveTarget } from "../../src/lib/corpus.ts";
import {
  renderAidSource,
  renderCategoricals,
  renderEnrollmentYears,
  renderNationalPosition,
  renderSupplements,
  renderWhatThisIsNot,
} from "../../src/lib/district.ts";
import { loadFeed } from "../../src/lib/feed.ts";
import { counties } from "../../src/lib/county.ts";
import * as routes from "../../src/lib/routes.ts";
import { anchor } from "../../src/lib/section.ts";
import { medianTrace, pairs } from "../../src/lib/relationships.ts";
import { BOX_FROM, distributionSpec, scatterSpec } from "../../src/lib/plot/spec.ts";
import { renderToString } from "../../src/lib/plot/ssr.ts";
import { renderSpendingByFunction } from "../../src/lib/spending.ts";
import {
  renderChargeOff,
  renderDenominators,
  renderMillage,
  renderTaxBase,
} from "../../src/lib/tax.ts";

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

/**
 * Every corpus link the district pages actually emit, checked by rendering them.
 *
 * # Why the hand list above is not enough, and what it cost
 *
 * The test above pins four slugs. It is a good test and it checks the wrong set: it enumerates the
 * links someone remembered to add to it, which is not the same as the links the site emits. The
 * renderers write `routes.metric(...)` and `routes.wikiNode(...)` inline in template strings, and
 * a call inside a template string is enumerated by nothing — not by `astro check`, which
 * type-checks the function and not its argument, and not by the corpus link checker, which reads
 * `.yidam/` prose and never looks at `src/lib/`.
 *
 * So `tax.ts` wrote `routes.parameter("state-share-percentage")` for a node that is a **metric**,
 * and `/wiki/parameter/state-share-percentage` shipped as a 404 on all 609 property-tax pages —
 * nine lines from `basecost.ts` linking the identical slug correctly.
 *
 * # Why this renders rather than greps
 *
 * A regular expression over the source finds the fourteen calls whose slug is a literal and misses
 * the thirteen `fsfp-*` slugs that reach `routes.wikiNode("formula-component", node)` through a
 * table — which is the largest single cluster of corpus links on the page and the one a rename is
 * most likely to break. Calling the renderers and reading their output has no such blind spot: it
 * checks what a reader is sent, which is the only thing that can 404.
 *
 * Two districts, because coverage is data-dependent: a large urban one renders supplement and
 * clawback rows a small one does not, and the union of the two is what the suite should hold.
 */
/**
 * The measured size of the set the test below harvests, written down rather than rounded.
 *
 * It is a floor and not an equality: a district that trips a branch the sample pair does not would
 * legitimately raise it, and pinning it exactly would make the test fail on a correct change. It
 * exists so that a renderer whose signature changes and stops producing markup fails here instead
 * of passing on an empty set.
 */
const EMITTED_FLOOR = 25;

test("every corpus link the district renderers emit resolves", () => {
  const districts = bundle.districts;
  const statewide = bundle.statewide;
  // Cleveland is the largest and renders the most branches; Kelleys Island is the smallest in the
  // panel and renders the fewest. Chosen by size rather than by name so the pair keeps meaning
  // something if the feed changes.
  const bySize = [...districts].sort((a, b) => b.adm - a.adm);
  const sample = [bySize[0]!, bySize[bySize.length - 1]!];

  const emitted = new Set<string>();
  for (const d of sample) {
    const html = [
      renderAidSource(bundle, d),
      renderEnrollmentYears(bundle, d, true),
      renderWhatThisIsNot(bundle, d),
      renderBaseCostBuildUp(d, statewide.districts),
      renderCategoricals(d, statewide),
      renderSupplements(d),
      renderNationalPosition(d),
      renderTaxBase(d),
      renderMillage(d, statewide),
      renderDenominators(d),
      renderChargeOff(d, statewide),
      renderSpendingByFunction(d),
    ].join("\n");
    for (const match of html.matchAll(/href="(\/[^"#?]*)/g)) emitted.add(match[1]!);
  }

  // Every site-absolute link, not only the `/wiki/` ones. The reciprocal links between the
  // enrollment card and the district scenario route, and the closing card's pointers at
  // `/finances` and `/taxes`, are route strings written inline in a template — the same shape as
  // the `routes.parameter` call that shipped a 404 on 609 pages, and enumerated by nothing else.
  //
  // Vacuity guard at the measured number rather than a round one. If a renderer signature changes
  // and the calls above stop producing markup, the set shrinks and this fails rather than passing
  // on nothing.
  expect(
    emitted.size,
    "the district renderers emitted fewer internal links than they used to",
  ).toBeGreaterThanOrEqual(EMITTED_FLOOR);

  const broken = [...emitted].filter((href) => !known(href)).sort();
  expect(broken, "links the district pages emit that resolve to nothing").toEqual([]);

  // This and the hand-pinned test above are complementary, not redundant. `renderPosition`,
  // `renderTaxChange` and `renderTaxAgainstSpending` need statewide comparison arguments this test
  // does not assemble, so their three slugs — assessed valuation per pupil, per-pupil operating
  // expenditure, the twenty-mill floor — are covered there and not here. Neither test alone
  // covers the set.
});

test("every district in the feed has all five of its views", () => {
  // The route table above is generated from the feed, so this checks the thing that route table
  // asserts: that `getStaticPaths` covers the whole panel and not a filtered subset of it.
  expect(PAGES.size).toBeGreaterThanOrEqual(bundle.districts.length * 5);
  for (const irn of ["043786", "049056", "044933"]) {
    expect(PAGES.has(routes.district(irn))).toBe(true);
    expect(PAGES.has(routes.districtScenario(irn))).toBe(true);
    expect(PAGES.has(routes.districtTaxes(irn))).toBe(true);
  }
});

test("every section a decision record renders is a name the route table lists", () => {
  /*
   * A decision page addresses each of its cards by the corpus field the card renders — `context`,
   * `rationale`, `amendment` — so the vocabulary in `routes.ts` and `DECISION_SECTIONS` in
   * `corpus.ts` have to agree. They are two tables and not one on purpose: `corpus.ts` reads
   * `.yidam/` off disk and `routes.ts` is a table of strings that must stay free of it.
   *
   * Which leaves them free to drift, and the drift is silent in the direction that matters. A
   * field added to the corpus and rendered as a card would fail the built-site check with a
   * message about `routes.ts` — a whole build, on a name nobody thought of as a route. This says
   * it in the unit suite instead.
   */
  const rendered = new Set(
    loadCorpus().decisions.flatMap((decision) => decision.sections.map((section) => section.name)),
  );
  expect(rendered.size, "the corpus renders decision sections at all").toBeGreaterThan(3);
  expect(
    [...rendered].filter((name) => !routes.SECTION_NAMES.has(name)),
    "a decision section the route table does not list",
  ).toEqual([]);
});

test("a section anchor names its own section, and nothing a template writes escapes it", () => {
  /*
   * The one place the anchor markup is defined. Both halves of the site render through it — the
   * template literals in `src/lib/*.ts` directly, the `.astro` templates through
   * `SectionAnchor.astro`, which renders this string rather than restating it as markup — so this
   * is the whole contract, and `check-dist-links.ts` asserts the built pages honour it.
   */
  expect(anchor("base-cost")).toContain('href="#base-cost"');
  expect(anchor("base-cost")).toContain('class="section-anchor"');
  // The name is read out; the glyph is not. A screen reader announcing "number sign" beside every
  // heading on the site would be worse than no anchor.
  expect(anchor("base-cost")).toContain('aria-label="Link to this section"');
  expect(anchor("base-cost")).toContain('aria-hidden="true"');
  // Separated from the words beside it. `</a>` against a letter is the fused-word defect the e2e
  // suite scans every route for, and inside a flex heading a whitespace-only run costs nothing.
  expect(anchor("base-cost").endsWith("</a> ")).toBe(true);
  // An id is interpolated into an attribute, so it is escaped there like any other.
  expect(anchor('a" onclick="x')).not.toContain('onclick="x"');
});

test("a scatter draws one mark per district and never a colour the theme cannot change", () => {
  /*
   * The two rules the build enforces for every other chart, asserted for this one before it ships
   * six hundred marks: a hover string per district in data order — `attachHovers` throws if those
   * disagree, and this is the cheaper place to find out — and no literal colour, because a
   * build-time SVG cannot re-render when a reader switches theme.
   */
  const points = pairs(
    loadFeed().bundle.districts,
    (d) => d.valuation_per_pupil,
    (d) => d.realized_aid_per_pupil,
    (d) => d.name,
  );
  expect(points.length).toBeGreaterThan(500);

  const spec = scatterSpec(points, {
    x: { label: "x", format: (v) => String(v), log: true },
    y: { label: "y", format: (v) => String(v), log: true },
  });
  expect(spec?.hovers?.text.length).toBe(points.length);

  // Renders, and `ensureThemeable` inside `renderToString` throws on a baked-in colour.
  const svg = renderToString(spec);
  expect(svg).toContain("<svg");
  expect(svg.replace(/<style>[\s\S]*?<\/style>/g, "")).not.toMatch(/#[0-9a-f]{3,8}\b|rgba?\(/i);
});

test("a scatter refuses to draw a population it does not have", () => {
  // Same rule the line forms use: too few points is not a cloud, and an axis with four marks on
  // it would read as a finding about something that has not been measured.
  const few = Array.from({ length: 6 }, (_, i) => ({ x: i, y: i, hover: String(i) }));
  expect(
    scatterSpec(few, { x: { label: "x", format: String }, y: { label: "y", format: String } }),
  ).toBeNull();
});

test("a median trace bins by count and keeps every district", () => {
  /*
   * Equal-count and not equal-width, because every measure here is skewed: valuation per pupil
   * runs $79k to $1.35M against a median of $248k, and equal-width tenths would put two thirds of
   * the districts in the first bin. The last bin takes the remainder, so integer division drops
   * nobody — the same rule the quintile helpers use.
   */
  const values = loadFeed()
    .bundle.districts.filter((d) => d.valuation_per_pupil != null)
    .map((d) => ({ x: d.valuation_per_pupil!, y: d.realized_aid_per_pupil }));
  const trace = medianTrace(values, 10, "as received", "guarantee");
  expect(trace.points.length).toBe(10);

  // The x of each bin is its own median, so the line is drawn where the districts are.
  const xs = trace.points.map((p) => p.x);
  expect([...xs].sort((a, b) => a - b)).toEqual(xs);
  expect(xs[0]).toBeGreaterThanOrEqual(Math.min(...values.map((v) => v.x)));
  expect(xs[xs.length - 1]).toBeLessThanOrEqual(Math.max(...values.map((v) => v.x)));
});

test("the guarantee shows up as a gap between what the formula computes and what is paid", () => {
  /*
   * The finding the wealth-neutrality card now draws, asserted as a fact about the feed rather
   * than as a fact about the chart. The two medians should be near enough identical among the
   * least wealthy districts — where few are on the guarantee — and apart among the wealthiest,
   * where most are. If that ever reversed, the paragraph under the chart would be wrong.
   */
  const districts = loadFeed().bundle.districts.filter((d) => d.valuation_per_pupil != null);
  const realized = medianTrace(
    districts.map((d) => ({ x: d.valuation_per_pupil!, y: d.realized_aid_per_pupil })),
    10,
    "as received",
    "guarantee",
  );
  const formula = medianTrace(
    districts.map((d) => ({ x: d.valuation_per_pupil!, y: d.formula_aid_per_pupil })),
    10,
    "the formula",
    "formula",
  );
  const gap = (i: number) => realized.points[i]!.y - formula.points[i]!.y;
  expect(gap(0)).toBeLessThan(100);
  expect(gap(9)).toBeGreaterThan(500);
  // And the payment is never below the formula's answer at the median: the guarantee only tops up.
  for (let i = 0; i < 10; i += 1) expect(gap(i)).toBeGreaterThanOrEqual(0);
});

test("a distribution draws its members where they fit and its shape where they do not", () => {
  /*
   * Three populations, three answers, one rule — the form decides rather than four call sites
   * deciding four ways. A county is six districts and a poverty fifth is a hundred and twenty:
   * both fit across the strip and both are drawn in full. Ohio's 609 do not, so the box carries
   * the shape and only the districts past the fences are drawn individually.
   */
  const values = (n: number) =>
    Array.from({ length: n }, (_, i) => ({ value: i, hover: `d${i}` }));

  const small = distributionSpec(values(30));
  expect(small?.hovers?.text.length, "a small population is drawn in full").toBe(30);

  const medium = distributionSpec(values(120));
  expect(medium?.hovers?.text.length, "a poverty fifth is drawn in full").toBe(120);

  // 609 evenly spaced values have no outliers, so the box is the whole of what is drawn.
  const large = distributionSpec(values(609));
  expect(large?.hovers?.text.length).toBeLessThan(30);
});

test("a distribution refuses a population too small to have a shape", () => {
  const values = (n: number) => Array.from({ length: n }, (_, i) => ({ value: i, hover: `d${i}` }));
  // A pair is not a distribution. Two of Ohio's legislative seats hold two school districts, and
  // the table on those pages names both.
  expect(distributionSpec(values(2))).toBeNull();
  expect(distributionSpec(values(3))).not.toBeNull();
});

test("the box is drawn only where quartiles summarise something", () => {
  /*
   * A seat with three districts drew a box spanning almost the full width with three dots inside
   * it: the first and third quartiles of three numbers are the first and third numbers. 39 of 132
   * seats and 60 of 88 counties are under the floor, so this is the common case rather than the
   * edge one.
   */
  const values = (n: number) => Array.from({ length: n }, (_, i) => ({ value: i, hover: `d${i}` }));
  const boxes = (spec: ReturnType<typeof distributionSpec>) =>
    (renderToString(spec).match(/<rect/g) ?? []).length;

  expect(boxes(distributionSpec(values(BOX_FROM - 1))), "no box below the floor").toBe(0);
  expect(boxes(distributionSpec(values(BOX_FROM))), "a box at the floor").toBeGreaterThan(0);
});

test("a district's position is drawn against the population it is being placed in", () => {
  /*
   * The flat strip this replaced put a pin on a bar with the minimum at one end and the maximum at
   * the other, which drew the 60th percentile and the 95th identically. Ohio's valuation per pupil
   * reaches five and a half times its median, so those are a dense middle and open country.
   */
  const bundle = loadFeed().bundle;
  const valuations = bundle.districts
    .filter((d) => d.valuation_per_pupil != null)
    .map((d) => ({ value: d.valuation_per_pupil!, hover: d.name }));
  const spec = distributionSpec(valuations, {
    marker: { value: valuations[0]!.value, label: "a district" },
  });
  const svg = renderToString(spec);
  expect(svg).toContain("dist-marker");
  // And it is themeable, like every other chart the build emits.
  expect(svg.replace(/<style>[\s\S]*?<\/style>/g, "")).not.toMatch(/#[0-9a-f]{3,8}\b|rgba?\(/i);

  // The skew is what the form exists for: the fences sit well inside the range, so the districts
  // past them are drawn as themselves rather than as the end of a continuum.
  expect(spec?.hovers?.text.length).toBeGreaterThan(0);
  expect(spec!.hovers!.text.length).toBeLessThan(valuations.length / 4);
});

test("an identity plot is square and shares one domain, or it does not mean what it says", () => {
  /*
   * The reduction-factor card draws the same quantity twice — mills this repository predicts
   * against mills a county auditor charged — so a point's distance from y = x is its residual, in
   * the units already on the axis. Two things have to hold for that reading to be true, and
   * neither is cosmetic: both axes on one domain, because a line through (min, min) and (max, max)
   * of two different ranges is not y = x; and a square plot area, because a shared domain on a
   * 640×420 frame still draws the line at 33°, which reads as a trend the cloud is beating rather
   * than as the equality it is.
   */
  const points = [
    { x: 10, y: 12, hover: "a" },
    { x: 20, y: 20, hover: "b" },
    { x: 30, y: 44, hover: "c" },
    ...Array.from({ length: 12 }, (_, i) => ({ x: 15 + i, y: 15 + i, hover: `d${i}` })),
  ];
  const spec = scatterSpec(
    points,
    { x: { label: "predicted", format: String }, y: { label: "charged", format: String } },
    [],
    { identity: { label: "predicted = charged" } },
  )!;

  const x = spec.options.x as { domain: [number, number] };
  const y = spec.options.y as { domain: [number, number] };
  expect(x.domain, "the two axes are on one domain").toEqual(y.domain);
  // Which is the union of both measures, not either one of them — padded, as every axis here is.
  expect(x.domain[0]).toBeLessThanOrEqual(10);
  expect(x.domain[1]).toBeGreaterThanOrEqual(44);

  const width = spec.options.width as number;
  const height = spec.options.height as number;
  const plotWidth = width - (spec.options.marginLeft as number) - (spec.options.marginRight as number);
  const plotHeight = height - (spec.options.marginTop as number) - (spec.options.marginBottom as number);
  expect(plotWidth, "the plot area is square").toBe(plotHeight);

  expect(renderToString(spec)).toContain("scatter-identity");
});

test("a scatter without an identity line fits each axis to its own measure", () => {
  // The ordinary case, and the reason the squaring is opt-in: valuation against aid has no
  // meaningful diagonal, and forcing one domain on two different quantities would be nonsense.
  const points = Array.from({ length: 20 }, (_, i) => ({ x: i, y: i * 1000, hover: `d${i}` }));
  const spec = scatterSpec(points, {
    x: { label: "x", format: String },
    y: { label: "y", format: String },
  })!;
  const x = spec.options.x as { domain: [number, number] };
  const y = spec.options.y as { domain: [number, number] };
  expect(x.domain).not.toEqual(y.domain);
  expect(renderToString(spec)).not.toContain("scatter-identity");
});

test("the reduction factors reproduce the floor and approximate everything else", () => {
  /*
   * The finding the card states, asserted against the feed rather than against the chart. At the
   * twenty-mill floor the factors have stopped operating and there is nothing left to predict, so
   * the model is near-exact; above it they are what sets the rate, and it is not. If that ever
   * reversed, three sentences on `/method` would be wrong.
   */
  const withMillage = loadFeed().bundle.districts.filter((d) => d.millage != null);
  const exact = (d: (typeof withMillage)[number]) => Math.abs(d.millage!.residual) < 0.01;
  const atFloor = withMillage.filter((d) => d.millage!.at_floor);
  const above = withMillage.filter((d) => !d.millage!.at_floor);

  expect(atFloor.length).toBeGreaterThan(100);
  expect(above.length).toBeGreaterThan(100);
  const floorRate = atFloor.filter(exact).length / atFloor.length;
  const aboveRate = above.filter(exact).length / above.length;
  expect(floorRate, "the floor cases reproduce").toBeGreaterThan(0.5);
  expect(aboveRate, "the rest do not").toBeLessThan(0.1);

  // And the departures run one way: the factors reduce existing levies on existing property and
  // know nothing of a levy passed since.
  const over = withMillage.filter((d) => d.millage!.residual > 0.5).length;
  const under = withMillage.filter((d) => d.millage!.residual < -0.5).length;
  expect(over).toBeGreaterThan(under * 5);
});
