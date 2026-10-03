/**
 * What the page looks like, as numbers.
 *
 * # Why this exists
 *
 * The unit suite ran 395 tests before this file and the browser suite runs 303, and not one of the
 * 698 looks at the page. They assert semantics, structure, contrast arithmetic and copy — which is
 * the right thing to assert and is why the palette is the best-argued part of this repository — but
 * a stylesheet can be entirely correct under all of them and still be unreadable. That is how the
 * 2026-08 audit found 96 findings behind a green gate.
 *
 * Those are the runners' counts and not `grep -c 'test('` over the files. Several of these suites
 * generate cases in a loop — the 20-route axe sweep is one `test()` and twenty tests — so the two
 * numbers disagree by 13 in the browser suite alone. This repository has a documented history of
 * comments carrying figures that reproduce under nothing, and a grep count presented as a test
 * count is that same defect with a smaller blast radius.
 *
 * So this measures the reading experience the way `palette.spec.ts` measures colour: from the
 * artefact, in numbers, reproducibly. See #183.
 *
 * # The two kinds of metric, and why the distinction is load-bearing
 *
 * **Deterministic** metrics read computed CSS and are the same on every machine: the census of
 * font sizes, the heading ratio, the count and nesting depth of bordered boxes, the count of
 * right-aligned cells carrying prose. These can become hard thresholds and later phases will make
 * them so.
 *
 * **Font-sensitive** metrics depend on how wide the platform's UI face draws a glyph: the reading
 * measure in `ch`, the height of the wrapping header, and the y-offset of the first content. SF
 * Pro, Segoe UI and DejaVu Sans do not agree, and this repository has already shipped a chart
 * defect that only reproduced under one of them — see the note in `plot/spec.ts`.
 *
 * A font-sensitive figure is therefore recorded WITH the advance width that produced it
 * ({@link Measured.zeroAdvance}) and the family that was resolved ({@link Measured.bodyFont}).
 * Without those a 75ch measured on a laptop and an 85ch measured on a runner look like a
 * regression rather than like two different fonts, and somebody spends an afternoon on it. A
 * threshold over one of these has to state the font it is a threshold under; none does yet,
 * which is the honest state and is why {@link THRESHOLDS} starts empty.
 *
 * # What is deliberately not measured
 *
 * Text inside `<svg>`. A chart's `font-size` attribute is not what gets painted — the SVG is
 * `width: 100%` over a `viewBox`, so the rendered size is `computed × (boxWidth / viewBox.width)`
 * and the e2e suite already checks that separately at 375px. Counting those numbers in the size
 * census would mix two scales and make the census say nothing.
 */

/** One `<p>` long enough that its line length is a reading decision rather than an accident. */
export const PROSE_MIN_CHARS = 140;

/** Above this many words, a table cell holds a sentence rather than a value. */
export const PROSE_CELL_MIN_WORDS = 6;

/** A box counts as a box at this radius or above; below it the corner is a hairline artefact. */
export const BOX_MIN_RADIUS = 4;

/**
 * The routes the report walks.
 *
 * Chosen to cover each genre exactly once rather than to be representative by volume: the reading
 * column, the 609-row wide table, the corpus prose genre, the decision-record genre, and the
 * chart-bearing card. 3,492 pages collapse into these five shapes, and a sixth route of a shape
 * already here would cost a browser page load and tell nobody anything.
 *
 * The Explained topic (#712) is a sixth shape and earns its route: a reading page held to a
 * reading grade, whose answer is a sentence rather than a figure, and whose whole length is the
 * thing its template promises to keep short ({@link PAGE_HEIGHT_CEILINGS}). The pilot topic stands
 * for the class; every topic renders through the same template.
 *
 * `build.format` is `"file"`, so these are the paths a host serves.
 */
export const ROUTES = [
  "/index.html",
  "/district/043786.html",
  "/district/043786/finances.html",
  "/districts.html",
  "/statewide.html",
  "/method.html",
  "/wiki/funding-regime/fair-school-funding-plan.html",
  "/wiki/decision/the-four-kinds-of-parameter.html",
  "/explained/phase-in.html",
] as const;

/**
 * The widths, and why these three.
 *
 * 375 is the narrow phone the chart suite already uses. 1280 is the desktop the site is composed
 * for. 768 is between them and is where the header stops wrapping to three rows and has not yet
 * reached the one-row layout — the width at which the chrome measurement in #189 is worst per
 * pixel of screen.
 */
export const WIDTHS = [375, 768, 1280] as const;

/** One route at one width. */
export interface Measured {
  route: string;
  width: number;

  /* Deterministic — computed CSS only. */

  /** Every distinct rendered `font-size`, ascending, outside `<svg>`. */
  sizes: number[];
  /** `h1` size over `body` size. Below about 2 there is no hierarchy, only a rounding error. */
  headingRatio: number | null;
  /**
   * Bordered, radiused boxes: how many, how deep they nest, and how many are decoration.
   *
   * `count` is what a reader sees. `decorative` is the subset a redesign may actually remove —
   * everything that is not an operable edge and not a floating panel.
   *
   * The split is not a convenience. WCAG 1.4.11 requires a control's boundary to clear 3:1 against
   * its ground, which is what `--border-control` was solved for, so the border on a chip, a tab, a
   * select or the skip link is an accessibility requirement rather than a style. Of the 30 boxes
   * on a district page, 19 are those. A threshold on `count` would therefore be a threshold that
   * can only be met by removing affordances, which is the opposite of the intent — see #185.
   */
  boxes: { count: number; decorative: number; maxDepth: number };
  /** `<td>` carrying more than {@link PROSE_CELL_MIN_WORDS} words and set `text-align: right`. */
  rightAlignedProse: number;
  /**
   * `<a>` elements inside `main`, in document order, before the first paragraph long enough to
   * count as {@link PROSE_MIN_CHARS} of reading — the same bar the reading measure already holds
   * a paragraph to. A structure question, not a layout one: a breadcrumb, an infobox and a
   * page-contents list are all links a reader passes before the page says anything, and counting
   * them by DOM position rather than by pixel position is what keeps this out of the
   * font-sensitive half below. `null` when the page never reaches a paragraph that long.
   */
  linksBeforeContent: number | null;

  /* Font-sensitive — layout, and therefore the platform's UI face. */

  /** Reading measure of substantial paragraphs, in `ch`. */
  measure: { median: number; p90: number; max: number; over78: number; count: number } | null;
  /** Height of the sticky header, which wraps and so depends on how wide the labels draw. */
  headerHeight: number | null;
  /** Where the page's own `h1` starts. The chrome above it is the phone's opening screen. */
  firstContentY: number | null;
  /**
   * Where the first `figure`, `svg` chart or `table` below the `h1` starts. The gap between this
   * and {@link firstContentY} is how much prose a reader crosses before the page shows them
   * anything other than a sentence — `null` when the route draws none of the three.
   */
  firstFigureY: number | null;
  /** Total height of the document. How long the page is, at this width. */
  pageHeight: number;
  /** The advance width of `0` in the body font, which is what a `ch` figure above is divided by. */
  zeroAdvance: number | null;
  /** The family the platform actually resolved, so a `ch` figure can be compared across machines. */
  bodyFont: string | null;
}

/** A whole run. */
export interface Report {
  /** ISO 8601. Passed in rather than read from the clock, so a report is reproducible. */
  measuredAt: string;
  rows: Measured[];
  /** Build-wide, unlike every field above it — one count over every wiki page rather than eight. */
  wikiToData?: WikiToData;
}

/** How many built wiki pages link a data route, out of how many wiki pages there are. */
export interface WikiToData {
  wikiPages: number;
  linking: number;
}

/**
 * A data route: an individual district, a county, or either of the two statewide summaries.
 *
 * Matched as an `href` prefix rather than as a full path, because `district` and `county` carry a
 * slug — `/district/043786`, never `/district` alone — while `statewide` and `outcomes` are each
 * one page. `/districts`, the index, is deliberately not one of these: it is the page a reader
 * already reaches from a wiki node's own nav, and this count is asking about the ones that are
 * not — a single district's dashboard, a county, or either whole-state view.
 */
const DATA_ROUTE_HREF = /^\/(district\/|county\/|statewide(?:$|\/)|outcomes(?:$|\/))/;

/**
 * How many wiki pages, out of every wiki page in a build, link a data route from their own
 * content.
 *
 * Scoped to `main` for the reason {@link Measured.linksBeforeContent} is: the sticky header
 * repeats the same handful of links on every route, and a page's own content is not what the
 * header carries. `pages` takes an already-scoped `main` per route because the caller — a Node
 * script walking `dist/` with `linkedom`, or a unit test handing this a fabricated document — is
 * the one that knows how to parse HTML, and this function should not have to.
 *
 * Today: 5 of 274 wiki pages link a district page (each names the district the node is about),
 * and none link a county or either statewide view — see #546.
 */
export function countWikiToData(pages: Array<{ path: string; main: ParentNode | null }>): WikiToData {
  const wiki = pages.filter((p) => p.path.startsWith("/wiki/"));
  const linking = wiki.filter((p) => {
    if (p.main == null) return false;
    for (const a of p.main.querySelectorAll("a[href]")) {
      const href = a.getAttribute("href");
      if (href != null && DATA_ROUTE_HREF.test(href)) return true;
    }
    return false;
  });
  return { wikiPages: wiki.length, linking: linking.length };
}

/**
 * The thresholds, and the fact that there are none yet.
 *
 * #183 builds the instrument and fails nothing — a check that starts red teaches whoever added it
 * to pass `--no-verify`. Each later phase turns one row of this report into a limit here:
 *
 * - #185 set `boxDecorative` to a hard zero once the card stopped being a bordered box
 * - #186 sets `sizeCount`, `sizeGap` and `headingRatio`, and `measureMax` under a stated font
 * - #188 sets `rightAlignedProse` to a hard zero
 * - #189 sets `firstContentY` at 375px, under a stated font
 *
 * A font-sensitive key must not be set without naming the font it holds under. See the header.
 */
export interface Thresholds {
  /** Most distinct font sizes a route may render. Deterministic. */
  sizeCount?: number;
  /** Largest permitted ratio between two adjacent sizes in the census. Deterministic. */
  sizeGap?: number;
  /** Smallest permitted `h1`-to-body ratio. Deterministic. */
  headingRatio?: number;
  /** Most bordered boxes a route may carry, operable edges included. Deterministic. */
  boxCount?: number;
  /** Most bordered boxes that are neither an operable edge nor a floating panel. Deterministic. */
  boxDecorative?: number;
  /** Deepest a bordered box may nest inside another. Deterministic. */
  boxDepth?: number;
  /** Most right-aligned cells carrying prose. Deterministic, and the target is zero. */
  rightAlignedProse?: number;
  /** Widest permitted paragraph, in `ch`. FONT-SENSITIVE — state the font. */
  measureMax?: number;
  /** Most chrome permitted above the `h1`, in px, at the narrowest width. FONT-SENSITIVE. */
  firstContentY?: number;
}

export const THRESHOLDS: Thresholds = {
  /*
   * #185. A bordered, radiused box that is neither an operable edge nor a floating panel is
   * decoration, and there are none left: the card and the tile both gave theirs up, and what
   * separates a section from the next one is now the interval.
   *
   * A hard zero rather than a budget, because there is no case where a document section needs an
   * outline to say it has ended — the ground and the space already say it. Anything that genuinely
   * needs an edge is a control or floats, and neither is counted here.
   *
   * `boxCount` is deliberately NOT set. 19 of the 30 boxes a district page used to draw are
   * operable edges answering to WCAG 1.4.11, so a threshold on the total could only be met by
   * removing affordances. `boxDepth` is not set either: it fell from 2 to 1 as a consequence of
   * this, not as an independent constraint, and a floating menu panel legitimately contains
   * bordered links — asserting depth would forbid that for no reason.
   */
  boxDecorative: 0,

  /*
   * #186. The reading ramp and the apparatus ramp, held apart.
   *
   * `sizeCount` at 10 is the wiki node, which is the busiest page here: eight named steps and `body`
   * at the apparatus size. The em-relative inline marks that used to scale with whatever sentence
   * they sat in are ramp tokens since #573, and `typography.spec.ts` sweeps the routes for any size
   * off the ramp. It was thirteen, eleven of them crowded inside 3.7px.
   *
   * `headingRatio` at 2 against the 2.56 measured. Below about 2 a heading is not a rank, it is a
   * rounding error — it was 1.49.
   */
  sizeCount: 10,
  headingRatio: 2,

  /*
   * `measureMax` is FONT-SENSITIVE and the 80 carries the platform spread rather than ignoring it.
   *
   * Measured 75ch here and 77 on the widest route; a runner's UI sans draws `0` at 9.54px against
   * this machine's 9.21, which is 3.6% wider per glyph and therefore ~2% FEWER characters on the
   * same column. So the runner reads lower than this, and 80 clears both with room. The report
   * prints the advance width beside every `ch` figure for exactly this comparison.
   *
   * It was 146ch on `/districts` and 110 in the reading column.
   */
  measureMax: 80,

  /*
   * #188. A hard zero, and it is deterministic — a word count and a computed `text-align`, neither
   * of which moves with the platform's font.
   *
   * The defect it closes was 21,333 cells over six words set right-aligned, on 1,433 of 3,492
   * pages, because `td { text-align: right }` applied to every table and was corrected only for
   * the ones tagged `.prose` — which was none of the fourteen on the district dashboard.
   *
   * Zero rather than a budget for the same reason `boxDecorative` is: there is no table on this
   * site where a sentence wants to be ragged-left. `alignColumns` in `src/lib/semantics.ts` decides
   * it per column from what the column holds, so a new table is covered the day it is written and
   * nobody has to remember a class.
   *
   * This report walks nine routes and the defect was on 1,433 pages, so the threshold alone would
   * have been satisfied by fixing five of them. The build-wide sweep in `tests/dist/semantics.spec.ts` is
   * the half that reads every page.
   */
  rightAlignedProse: 0,

  /*
   * #189. How far down the page the `h1` starts at the narrowest width, which is how much of a
   * phone screen is spent before the reader learns what they are looking at.
   *
   * It was 194px on a district page at 390px — the header wrapped to three rows and measured
   * 166px. The five section menus now fold into one disclosure and the district sub-navigation
   * scrolls rather than wrapping, so the header is 52px at every width and the `h1` starts at 80.
   *
   * 110 against a measured 80 on the six page-level routes and 107 on the two wiki routes, whose
   * breadcrumb sits above the heading and is content rather than chrome. A header that wrapped
   * back to two rows would add about 40px and put every route over.
   *
   * # The font it holds under
   *
   * ui-sans-serif / system-ui at 375px, where a digit advance is 9.21px. Unlike `measureMax`, this
   * one turned out NOT to move: measured under DejaVu Sans and Georgia at the same width, every
   * route reports the same 80 and 107.
   *
   * The sensitivity is real but it lives below this width. At 320px, under Georgia or Courier New,
   * the brand and the disclosure no longer fit on one row and the header goes to 92px — see
   * `--sticky-chrome` in `tokens/space.css`, which records the same boundary. 320px is narrower
   * than any width this report walks and narrower than any phone the site is built for, so the
   * threshold is stated at 375px and the edge is written down rather than guarded.
   */
  firstContentY: 110,

  /*
   * `sizeGap` is deliberately NOT set, and the metric it would have used is why.
   *
   * It was built to catch "a cluster and then a leap" — eleven sizes inside 3.7px and then 22.4px.
   * The crowding was the defect; the leap never was. A display size IS a leap, and after this
   * phase the widest step on `/districts` is 2.57x precisely because the `h1` is doing its job. A
   * threshold here would forbid the thing the phase exists to create, so `sizeCount` carries the
   * finding alone and this stays unset.
   */
};

/**
 * How far down a phone screen the first figure may start, by the kind of page (#549).
 *
 * `firstContentY` asks how much chrome sits above the `h1`; this asks how much of everything else
 * does — the lede, the facts row, the notes and the disclaimers — before the reader reaches the
 * first chart, table or figure that answers what the page is for. It was 1,726px on a Cleveland
 * dashboard at 375px, which is two and a half phone screens of preamble, and 1,993 on the worst
 * district. #549 moved the apparatus below the answer and folded the dashboard's contents list,
 * and these are the ceilings that keep it there.
 *
 * # Why per class and not one number
 *
 * The pages are different shapes and one number would either forbid the dashboard or permit a
 * listing to grow a screen of preamble unnoticed. A class is a set of routes that share a template
 * and so move together; each carries the worst route of its class that was measured and a sample
 * the browser suite loads, and the worst route is always in the sample.
 *
 * # The font it holds under, and the headroom
 *
 * FONT-SENSITIVE: the y of the first figure is the height of every line of prose above it, and a
 * wider face wraps more. Each `measured` is the WORSE of two runs at 375px over the same build —
 * this machine's system-ui (the stack resolves to it, since IBM Plex is not installed) and DejaVu
 * Sans / Serif / Sans Mono loaded in place of the three stacks, which is what an Ubuntu runner
 * resolves them to. DejaVu is wider and read 0-65px deeper route for route; the worst dashboard reads 1,634 under
 * both, because what sits above its first table is mostly figures rather than prose.
 *
 * Every ceiling is the worse measurement plus ten percent, rounded up to the next hundred. Ten
 * percent is about a paragraph at this width: enough that a reworded sentence does not redden the
 * suite, not enough that a new card above the answer passes. The dashboard, overview and method
 * ceilings are each below what that class's worst route measured before #549, so the check would
 * have failed the site as it was. The listing class did not move — its tables already led — and
 * its ceiling only holds it there.
 *
 * `/index.html` sits in the overview class at its current 948; the homepage is being rebuilt
 * alongside this, and a homepage that puts its first figure lower should move the class or earn
 * its own rather than raise this one silently.
 *
 * Wiki routes and the district sub-pages are not classed: a wiki node is prose by genre and has no
 * answer figure to reach, and the sub-pages each lead with their table already.
 */
export const FIRST_FIGURE_CEILINGS = [
  {
    name: "district dashboard",
    pattern: /^\/district\/\d{6}\.html$/,
    // All 609 dashboards measured. Median 1,325 under system-ui, 1,347 under DejaVu. Was 1,993.
    measured: 1634,
    ceiling: 1800,
    sample: ["/district/046755.html", "/district/043786.html"],
  },
  {
    name: "overview",
    pattern: /^\/(index|statewide|outcomes|history|bounds)\.html$/,
    // /statewide under DejaVu. Before #549: /history 1,681, /statewide 1,302, /bounds 1,241.
    measured: 1072,
    ceiling: 1200,
    sample: ["/index.html", "/statewide.html", "/outcomes.html", "/history.html", "/bounds.html"],
  },
  {
    name: "listing",
    pattern: /^\/(districts|compare|county\/[a-z-]+)\.html$/,
    // /compare under DejaVu. All 88 counties measured; the deepest is Hocking at 744.
    measured: 800,
    ceiling: 900,
    sample: ["/districts.html", "/compare.html", "/county/hocking.html", "/county/cuyahoga.html"],
  },
  {
    name: "method",
    pattern: /^\/method\.html$/,
    // A reading page whose first figure is the base-cost table, after the argument. Was 2,596.
    measured: 2260,
    ceiling: 2500,
    sample: ["/method.html"],
  },
] as const;

/** The first-figure ceiling a route is held to, or `null` for a route no class covers. */
export function firstFigureCeiling(route: string): number | null {
  return FIRST_FIGURE_CEILINGS.find((c) => c.pattern.test(route))?.ceiling ?? null;
}

/**
 * How long an Explained topic may be on a phone, whole page (#712).
 *
 * A topic is one question answered in eight short sections, and the template's promise is that a
 * newcomer reads it to the end. Nothing else in the measure holds a page's length, and length is
 * the way that promise breaks: a second chart, a longer worked example, another card of caveats.
 * The ceiling is on `pageHeight` at 375px, the narrowest width and so the longest page.
 *
 * Measured and rounded as {@link FIRST_FIGURE_CEILINGS} are: the worse of system-ui and DejaVu at
 * 375px, plus ten percent, up to the next hundred. Here system-ui read the longer, 4,438 against
 * DejaVu's 4,089 — the chart is the bulk of the page and its height does not follow the face, so
 * the prose that does is a minority of the length.
 *
 * One class rather than one number per topic, because every topic renders through one template.
 * Each topic joins the sample as it lands; a topic that needs more room than the pilot earned is a
 * topic with too much in it, and the answer is to cut it, not to raise the class.
 */
export const PAGE_HEIGHT_CEILINGS = [
  {
    name: "explained topic",
    pattern: /^\/explained\/[a-z0-9-]+\.html$/,
    // The pilot, /explained/phase-in, under system-ui. /explained/what-a-district-gets reads
    // 4,616 under system-ui and 4,252 under DejaVu, after cutting it from 5,066 to fit.
    measured: 4438,
    ceiling: 4900,
    sample: ["/explained/phase-in.html", "/explained/what-a-district-gets.html"],
  },
] as const;

/** The page-height ceiling a route is held to, or `null` for a route no class covers. */
export function pageHeightCeiling(route: string): number | null {
  return PAGE_HEIGHT_CEILINGS.find((c) => c.pattern.test(route))?.ceiling ?? null;
}

/** One breach of one threshold, named so the message says what to do rather than what happened. */
export interface Violation {
  route: string;
  width: number;
  metric: keyof Thresholds;
  measured: number;
  limit: number;
  message: string;
}

/**
 * The widest ratio between two adjacent sizes in a census.
 *
 * A scale is a scale because its steps are even. Fifteen sizes crowded into 4.2px and then a jump
 * to the `h1` is two clusters rather than a ramp, and the number that says so is the largest gap
 * between neighbours — 22.4 / 15.2 = 1.47 today, against roughly 1.15 between every other pair.
 *
 * Returns 1 for a census too short to have a gap, which is the identity: no gap to be too wide.
 */
export function widestGap(sizes: number[]): number {
  const ascending = [...sizes].sort((a, b) => a - b);
  let widest = 1;
  for (let i = 1; i < ascending.length; i += 1) {
    const previous = ascending[i - 1];
    const current = ascending[i];
    if (previous == null || current == null || previous <= 0) continue;
    widest = Math.max(widest, current / previous);
  }
  return widest;
}

/**
 * Every threshold this report breaches.
 *
 * Deliberately returns the whole list rather than throwing on the first: a redesign phase wants to
 * see all of what it moved, and a check that stops at the first failure turns one run into six.
 */
export function violations(report: Report, thresholds: Thresholds = THRESHOLDS): Violation[] {
  const found: Violation[] = [];
  const at = (row: Measured, metric: keyof Thresholds, measured: number, limit: number, message: string) =>
    found.push({ route: row.route, width: row.width, metric, measured, limit, message });

  for (const row of report.rows) {
    if (thresholds.sizeCount != null && row.sizes.length > thresholds.sizeCount) {
      at(row, "sizeCount", row.sizes.length, thresholds.sizeCount,
        `renders ${row.sizes.length} distinct font sizes; the scale allows ${thresholds.sizeCount}`);
    }
    if (thresholds.sizeGap != null) {
      const gap = widestGap(row.sizes);
      if (gap > thresholds.sizeGap) {
        at(row, "sizeGap", gap, thresholds.sizeGap,
          `the widest step in the size census is ${gap.toFixed(2)}×, over ${thresholds.sizeGap}×`);
      }
    }
    if (thresholds.headingRatio != null && row.headingRatio != null && row.headingRatio < thresholds.headingRatio) {
      at(row, "headingRatio", row.headingRatio, thresholds.headingRatio,
        `the h1 is ${row.headingRatio.toFixed(2)}× body, under ${thresholds.headingRatio}×`);
    }
    if (thresholds.boxCount != null && row.boxes.count > thresholds.boxCount) {
      at(row, "boxCount", row.boxes.count, thresholds.boxCount,
        `carries ${row.boxes.count} bordered boxes, over ${thresholds.boxCount}`);
    }
    if (thresholds.boxDecorative != null && row.boxes.decorative > thresholds.boxDecorative) {
      at(row, "boxDecorative", row.boxes.decorative, thresholds.boxDecorative,
        `carries ${row.boxes.decorative} bordered boxes that are decoration, over ${thresholds.boxDecorative}`);
    }
    if (thresholds.boxDepth != null && row.boxes.maxDepth > thresholds.boxDepth) {
      at(row, "boxDepth", row.boxes.maxDepth, thresholds.boxDepth,
        `nests bordered boxes ${row.boxes.maxDepth} deep, over ${thresholds.boxDepth}`);
    }
    if (thresholds.rightAlignedProse != null && row.rightAlignedProse > thresholds.rightAlignedProse) {
      at(row, "rightAlignedProse", row.rightAlignedProse, thresholds.rightAlignedProse,
        `sets ${row.rightAlignedProse} sentence-bearing cells right-aligned, over ${thresholds.rightAlignedProse}`);
    }
    if (thresholds.measureMax != null && row.measure != null && row.measure.max > thresholds.measureMax) {
      at(row, "measureMax", row.measure.max, thresholds.measureMax,
        `sets a paragraph at ${row.measure.max.toFixed(0)}ch, over ${thresholds.measureMax}ch ` +
        `(measured in ${row.bodyFont ?? "an unrecorded font"})`);
    }
    if (thresholds.firstContentY != null && row.firstContentY != null && row.width === WIDTHS[0]
        && row.firstContentY > thresholds.firstContentY) {
      at(row, "firstContentY", row.firstContentY, thresholds.firstContentY,
        `puts the h1 at y=${row.firstContentY}, over ${thresholds.firstContentY}px of chrome ` +
        `(measured in ${row.bodyFont ?? "an unrecorded font"})`);
    }
  }
  return found;
}

/** `1234.5` as `1,234.5`, and `null` as an em dash, so a column of these stays a column. */
function cell(value: number | null | undefined, decimals = 0): string {
  if (value == null || Number.isNaN(value)) return "—";
  return value.toLocaleString("en", { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
}

/**
 * The report as a table, one block per width.
 *
 * Grouped by width rather than by route because every font-sensitive column only means anything
 * against a stated width, and a reader comparing two routes is nearly always comparing them at the
 * same one.
 */
export function formatReport(report: Report): string {
  const COLUMNS: Array<{ head: string; width: number; of: (row: Measured) => string }> = [
    { head: "route", width: 44, of: (r) => r.route.replace(/\.html$/, "") },
    { head: "sizes", width: 5, of: (r) => cell(r.sizes.length) },
    { head: "gap", width: 5, of: (r) => `${widestGap(r.sizes).toFixed(2)}×` },
    { head: "h1/body", width: 7, of: (r) => (r.headingRatio == null ? "—" : `${r.headingRatio.toFixed(2)}×`) },
    { head: "med", width: 4, of: (r) => cell(r.measure?.median) },
    { head: "p90", width: 4, of: (r) => cell(r.measure?.p90) },
    { head: "max", width: 4, of: (r) => cell(r.measure?.max) },
    { head: ">78", width: 4, of: (r) => cell(r.measure?.over78) },
    { head: "boxes", width: 5, of: (r) => cell(r.boxes.count) },
    { head: "deco", width: 4, of: (r) => cell(r.boxes.decorative) },
    { head: "deep", width: 4, of: (r) => cell(r.boxes.maxDepth) },
    { head: "cells", width: 5, of: (r) => cell(r.rightAlignedProse) },
    { head: "chrome", width: 6, of: (r) => cell(r.firstContentY) },
    { head: "links", width: 5, of: (r) => cell(r.linksBeforeContent) },
    { head: "figY", width: 6, of: (r) => cell(r.firstFigureY) },
    { head: "pageH", width: 6, of: (r) => cell(r.pageHeight) },
  ];

  const lines: string[] = [];
  lines.push(`MEASURE REPORT — ${report.rows.length} rows, measured ${report.measuredAt}`);
  lines.push("");
  lines.push("  med/p90/max/>78 are the reading measure in ch and DEPEND ON THE PLATFORM'S FONT.");
  lines.push("  chrome is the y-offset of the h1, and depends on it too. The rest are computed CSS.");

  for (const width of WIDTHS) {
    const rows = report.rows.filter((r) => r.width === width);
    if (rows.length === 0) continue;
    const font = rows.find((r) => r.bodyFont != null);
    lines.push("");
    lines.push(
      `${width}px` +
        (font?.bodyFont ? `  —  ${font.bodyFont}, 0 draws ${font.zeroAdvance?.toFixed(2)}px` : ""),
    );
    lines.push(
      "  " + COLUMNS.map((c) => (c.head === "route" ? c.head.padEnd(c.width) : c.head.padStart(c.width))).join(" "),
    );
    lines.push("  " + COLUMNS.map((c) => "─".repeat(c.width)).join(" "));
    for (const row of rows) {
      lines.push(
        "  " +
          COLUMNS.map((c) => {
            const text = c.of(row);
            return c.head === "route" ? text.padEnd(c.width) : text.padStart(c.width);
          }).join(" "),
      );
    }
  }

  if (report.wikiToData != null) {
    const { wikiPages, linking } = report.wikiToData;
    lines.push("");
    lines.push(`  wikiToData: ${linking} of ${wikiPages} wiki pages link a data route from their own content.`);
  }

  return lines.join("\n");
}

/**
 * Collect every metric from the page this is evaluated in.
 *
 * Self-contained by necessity: Playwright serialises this function and runs it in the browser, so
 * it may close over nothing. The three constants it needs are therefore arguments rather than the
 * module-level ones above, and the caller passes those — which also makes the coupling visible
 * instead of leaving two copies of `140` in two files.
 */
export function collect(limits: {
  proseMinChars: number;
  proseCellMinWords: number;
  boxMinRadius: number;
}): Omit<Measured, "route" | "width"> {
  const { proseMinChars, proseCellMinWords, boxMinRadius } = limits;

  const visible = (el: Element): boolean => {
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) return false;
    const style = getComputedStyle(el);
    return style.visibility !== "hidden" && style.display !== "none";
  };

  /* The size census. Only elements that themselves paint text, and never inside an <svg>: a
     chart's font-size attribute is scaled by its viewBox and is not the size that lands. */
  const sizes = new Set<number>();
  for (const el of document.querySelectorAll("body *")) {
    if (el.closest("svg") != null) continue;
    let paintsText = false;
    for (const node of el.childNodes) {
      if (node.nodeType === 3 && (node.textContent ?? "").trim().length > 0) paintsText = true;
    }
    if (!paintsText || !visible(el)) continue;
    /* Rounded to the nearest half pixel, because two sizes 0.01px apart are not two sizes.
       `.claim` is `0.68em` of prose and lands at 11.53px where `--text-xs` lands at 11.52 — a
       census that reports those as separate steps is counting float error, not decisions. Half a
       pixel is below what any reader distinguishes and above what rounding produces. */
    sizes.add(Math.round(parseFloat(getComputedStyle(el).fontSize) * 2) / 2);
  }

  const bodyStyle = getComputedStyle(document.body);
  const bodySize = parseFloat(bodyStyle.fontSize);
  const h1 = document.querySelector("h1");
  const headingRatio =
    h1 != null && bodySize > 0 ? parseFloat(getComputedStyle(h1).fontSize) / bodySize : null;

  /* The advance width of `0` in the body font, which is the divisor behind every `ch` below.
     Measured rather than assumed, and reported, because it is what makes two machines comparable. */
  const ruler = document.createElement("span");
  ruler.style.cssText = `position:absolute;visibility:hidden;white-space:pre;font:${bodyStyle.font}`;
  ruler.textContent = "0".repeat(100);
  document.body.appendChild(ruler);
  const zeroAdvance = ruler.getBoundingClientRect().width / 100;
  ruler.remove();

  /* The reading measure. Per-paragraph, in that paragraph's own font — a `<p>` inside a card and
     one inside `.prose-body` are set at different sizes, so one divisor for the page would be
     wrong for half of them. Cached by font string; there are two or three distinct ones. */
  const advances = new Map<string, number>();
  const advanceIn = (font: string): number => {
    const cached = advances.get(font);
    if (cached != null) return cached;
    const span = document.createElement("span");
    span.style.cssText = `position:absolute;visibility:hidden;white-space:pre;font:${font}`;
    span.textContent = "0".repeat(100);
    document.body.appendChild(span);
    const advance = span.getBoundingClientRect().width / 100;
    span.remove();
    advances.set(font, advance);
    return advance;
  };

  const widths: number[] = [];
  for (const p of document.querySelectorAll("p")) {
    if ((p.textContent ?? "").trim().length < proseMinChars || !visible(p)) continue;
    const advance = advanceIn(getComputedStyle(p).font);
    if (advance > 0) widths.push(p.getBoundingClientRect().width / advance);
  }
  widths.sort((a, b) => a - b);
  const at = (q: number): number => widths[Math.min(widths.length - 1, Math.floor(widths.length * q))] ?? 0;
  const measure =
    widths.length === 0
      ? null
      : {
          median: Math.round(at(0.5)),
          p90: Math.round(at(0.9)),
          max: Math.round(widths[widths.length - 1] ?? 0),
          over78: widths.filter((w) => w > 78).length,
          count: widths.length,
        };

  /* Bordered, radiused boxes, and their nesting. A box needs a border that is actually drawn —
     a `border-style: none` at 1px paints nothing and is not a boundary a reader can see. */
  const boxed = new Set<Element>();
  const decorative = new Set<Element>();
  /* An operable edge answers to WCAG 1.4.11 rather than to taste, and a floating panel needs an
     edge because it sits over content rather than in it. Shadow is a sound proxy for floating
     here and not a guess: `stylesheet.spec.ts` asserts that nothing which sits on the page
     carries one, so anything that does is by construction above it. */
  const OPERABLE = "a[href], button, input, select, textarea, summary, label, [tabindex]";
  for (const el of document.querySelectorAll("body *")) {
    const style = getComputedStyle(el);
    if (style.borderTopStyle === "none" || parseFloat(style.borderTopWidth) === 0) continue;
    if (parseFloat(style.borderTopLeftRadius) < boxMinRadius) continue;
    if (!visible(el)) continue;
    boxed.add(el);
    const floating = style.boxShadow !== "none" && style.boxShadow !== "";
    if (!el.matches(OPERABLE) && !floating) decorative.add(el);
  }
  let maxDepth = 0;
  for (const el of boxed) {
    let depth = 0;
    for (let node: Element | null = el; node != null; node = node.parentElement) {
      if (boxed.has(node)) depth += 1;
    }
    maxDepth = Math.max(maxDepth, depth);
  }

  let rightAlignedProse = 0;
  for (const td of document.querySelectorAll("td")) {
    if (getComputedStyle(td).textAlign !== "right") continue;
    const words = (td.textContent ?? "").trim().split(/\s+/).filter(Boolean).length;
    if (words > proseCellMinWords) rightAlignedProse += 1;
  }

  /* `id="main"` is `Base.astro`'s wrapper — everything a route draws that is not the sticky
     header. Both new counts below stop at its boundary for the same reason: the header repeats
     the same links on every route, and counting them would make every page's figure look like
     the reader's own content. */
  const main = document.querySelector("main");

  /* Links before content: walked by DOM position, not pixel position, which is why this sits in
     the deterministic half above rather than beside `firstContentY`. */
  let linksBeforeContent: number | null = null;
  if (main != null) {
    const firstProse = [...main.querySelectorAll("p")].find(
      (p) => (p.textContent ?? "").trim().length >= proseMinChars,
    );
    if (firstProse != null) {
      let count = 0;
      const walker = document.createTreeWalker(main, NodeFilter.SHOW_ELEMENT);
      for (let node = walker.nextNode(); node != null; node = walker.nextNode()) {
        if (node === firstProse) break;
        if ((node as Element).tagName === "A") count += 1;
      }
      linksBeforeContent = count;
    }
  }

  /* The first figure, chart or table below the h1. `querySelectorAll` already returns document
     order, so the first visible match at or after the h1's own position is the one a reader
     reaches first. */
  let firstFigureY: number | null = null;
  if (main != null) {
    const h1Top = h1 == null ? -Infinity : h1.getBoundingClientRect().top + window.scrollY;
    for (const el of main.querySelectorAll("figure, svg, table")) {
      if (!visible(el)) continue;
      const top = el.getBoundingClientRect().top + window.scrollY;
      if (top < h1Top) continue;
      firstFigureY = Math.round(top);
      break;
    }
  }

  const header = document.querySelector("header.site");
  return {
    sizes: [...sizes].sort((a, b) => a - b),
    headingRatio,
    boxes: { count: boxed.size, decorative: decorative.size, maxDepth },
    rightAlignedProse,
    linksBeforeContent,
    measure,
    headerHeight: header == null ? null : Math.round(header.getBoundingClientRect().height),
    firstContentY: h1 == null ? null : Math.round(h1.getBoundingClientRect().top + window.scrollY),
    firstFigureY,
    pageHeight: Math.round(document.documentElement.scrollHeight),
    zeroAdvance,
    bodyFont: bodyStyle.fontFamily,
  };
}
