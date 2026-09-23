/**
 * The corpus-to-crate cross-check for the columns a node draws, beside `corpusFigures.ts` for the
 * scalars it quotes.
 *
 * # What a series is, and what holds it
 *
 * `crates/series.json` is the second document `crates/figures` writes: short labelled columns the
 * wiki renders as charts, computed from the same inputs as the figure manifest. A node binds one
 * with a `series:` entry naming the key and the prose field the chart is drawn under, and the
 * route renders it there.
 *
 * A chart is not quoted, so none of the phrase checks the figure gate runs apply. What holds it
 * to the prose instead is the **endpoint rule**: every series' largest and smallest row name an
 * ordinary figure (asserted on the crate side, in `crates/figures/tests/`), and a node that draws
 * the series must bind those figures in its own `figures:` block (asserted here). The numbers a
 * reader takes off a chart are its extremes, and the extremes are then numbers the prose states,
 * the crate pins, and the figure gate checks three ways.
 *
 * # What a cloud is, and what holds it
 *
 * The same document carries a second array. A **cloud** is a population drawn as it is rather than
 * summarised — six hundred districts under one or more panels sharing a predictor — and it has no
 * rows, so it has no endpoints. What a reader can take off it is the pair of numbers printed on
 * each panel: the **slope** of the line the crate fitted through it and that line's **r-squared**.
 * So those are what the rule holds. Every panel names the figure that pins each, the crate asserts
 * it (`crates/figures/tests/`), and a node drawing the cloud must bind all of them here.
 *
 * It is the same rule. A bar chart's extremes and a scatter's fit are both "the numbers a reader
 * quotes off the picture", and both end up as ordinary figures the prose states and the figure
 * gate checks three ways. Nothing else on a panel is a claim: the four corner numbers are the
 * frame, which the crate computes so that a slope is drawn as an angle.
 *
 * # What a plane is, and what holds it
 *
 * A third array, and the strictest form of the same rule. A **plane** is a handful of named things
 * placed on two measures at once — seven anchor rules on (guarantee written, total state support)
 * — and it exists because the two axes disagree for three of the seven, which is a finding no
 * column reporting one of them at a time can carry.
 *
 * A cloud pins two numbers per panel because two is what a reader can quote off six hundred dots.
 * A plane is seven dots with their names printed on them, so every coordinate is quotable and
 * **every coordinate is pinned**: each position names a figure for its x and a figure for its y,
 * the crate asserts that both reproduce, and a node drawing the plane must bind all fourteen.
 *
 * # What a curve is, and what holds it
 *
 * A fourth array, and the one shape here whose index is **ordered**. A **curve** is two or more
 * named lines over one monotone index — the thirteen horizons a backtest reaches — drawn against
 * a **reference** the reader is asked to measure them by. Its claim is a shape along the index:
 * that the pooled line sinks away from 68.3% while the cross-district line stays flat beside it.
 *
 * So its extremes are not "largest and smallest": reordering the index would destroy what the
 * chart says, and the numbers a reader quotes are each line's **first and last** point, plus the
 * **worst departure** from the reference — which is a maximum over the whole index and not a
 * value at any position on it. Three figures per line, and a node drawing a curve must bind all
 * of them. The reference itself is pinned by nothing, and cannot be: 68.3% is the area under a
 * normal curve inside one standard deviation, which no crate computes and nothing could make
 * stale.
 *
 * # What a spread is, and what holds it
 *
 * A fifth array, and the first whose claim is about a region of the picture being **empty**. A
 * **spread** is a whole population on two signed axes that sum to a third quantity — the
 * guarantee's two terms, six hundred districts, one mark each — cut into regions by a
 * classification drawn as hue and by boundaries where that classification stops being defined.
 *
 * It has no extremes worth quoting, no fitted line, and no coordinate anybody reads off. What a
 * reader takes off it is **how many are in that part of it**, so that is the rule: a **census**,
 * one row per region, each naming an ordinary figure, and a node drawing the spread must bind all
 * of them. "Nobody is off the floor because the formula pays less per child" is a count of four
 * out of three hundred and fourteen in one wedge, and a reader will take a full region on trust
 * from prose long before they take an empty one.
 *
 * The second half of that rule is **unreached**: a count is a count of a population, and a
 * population is the thing that silently changes when a fixture is extended. So a subject the
 * computation could not place is carried to the drawing and named under it rather than dropped
 * out of the denominator. A spread of 607 captioned 609 is a small lie that is easy to ship.
 *
 * Boundaries name no figure, on the same argument the curve's reference does not: "the multiple
 * is one" is the line `y = -x` on two terms that sum to a log multiple, which is arithmetic and
 * not a measurement. They arrive as **two endpoints in the axes' own units** for the reason
 * {@link Fit} does — evaluating a line in this layer would be this layer computing a claim again.
 *
 * # Sign
 *
 * Rows are signed, because a bar below the zero rule is what a chart is for, and so are slopes —
 * −0.0429 is the finding on the second panel of the first cloud. The figure manifest exports a
 * signed quantity as a magnitude with the direction in its key, so {@link rowsAgainstFigures} and
 * {@link fitsAgainstFigures} compare a magnitude to the figure it names. A curve's departure is
 * the one quantity here that is unsigned before it starts — it is a distance from the reference —
 * so for it the two comparisons coincide.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import type { Bar, Fit, Place, Rank, ScatterPoint, SeriesPoint } from "./chart.ts";
import type { Node } from "./corpus.ts";
import { citedCrates, proseFields, type Manifest, type Unit } from "./corpusFigures.ts";
import { count, money, pct } from "./format.ts";
import * as routes from "./routes.ts";

/**
 * The series manifest contract this module reads. Versioned apart from the figure manifest's:
 * the two documents have different shapes and different readers, and a field added to a row is no
 * reason for the scalar check to refuse a document it still reads correctly.
 */
export const READS_SERIES_CONTRACT = "6.0.0";

/** One row of a series: a category and its signed value. */
export interface SeriesRow {
  label: string;
  value: number;
  /** What the mark says on hover, when the label and the value are not enough on their own. */
  hover?: string;
  /**
   * The run of rows this one belongs to, where the axis has a second level — a family, a class.
   * A table drawn from the series heads its rows with it.
   */
  group?: string;
  /**
   * The denominator, where the value is a count out of a population that is not the whole. `212`
   * against `604` and `212` against `609` are different claims and the manifest makes the
   * difference sayable; see the crate-side field for the census this exists for.
   */
  of?: number;
  /** The authority this row's value rests on, where the row is a claim with a source of its own. */
  cites?: string;
  /**
   * Why this row is the one the chart was drawn to point at, in a phrase, when one of them is the
   * finding. At most one row a series carries. Becomes {@link Rank.marked}.
   */
  marked?: string;
  /** The `crates/figures.json` key whose pin this row reproduces in magnitude. */
  figure?: string;
}

/** One column, computed from the crate that owns it. */
export interface ManifestSeries {
  key: string;
  owner: string;
  unit: Unit;
  /** What the rows are indexed by, in words. */
  axis: string;
  label: string;
  rows: SeriesRow[];
}

/** One continuous axis of a cloud: what it measures, and the domain the crate drew it on. */
export interface ManifestAxis {
  label: string;
  unit: Unit;
  min: number;
  max: number;
  /** Every axis of the first cloud is: the fits are in logs, and a line fitted in logs is a line
      only where the axes are. */
  log: boolean;
}

/**
 * A fitted line, as two endpoints in the data's own units plus the two numbers it is a line of.
 *
 * See {@link Fit} in `chart.ts` for why it arrives as a segment rather than as a model, and why a
 * fitted line is allowed on this site at all when {@link Bar}'s sibling `Trace` forbids one.
 */
export interface ManifestFit {
  from: [number, number];
  to: [number, number];
  slope: number;
  rSquared: number;
  /** The `crates/figures.json` key whose pin the slope reproduces in magnitude. */
  slopeFigure: string;
  /** As `slopeFigure`, for the r-squared, which is never negative and so is pinned as it stands. */
  rSquaredFigure: string;
}

/** One outcome drawn against the cloud's shared predictor. */
export interface ManifestPanel {
  label: string;
  y: ManifestAxis;
  fit: ManifestFit;
}

/** One subject of a cloud: where it sits on the predictor, and under every panel. */
export interface ManifestPoint {
  label: string;
  x: number;
  /** Aligned to the cloud's `panels`, one value each. */
  ys: number[];
}

/** One population drawn as itself, computed from the crate that owns it. */
export interface ManifestScatter {
  key: string;
  owner: string;
  /** What one point is, singular and lower case — "district". */
  subject: string;
  label: string;
  /** The shared predictor. Every panel is drawn against it, at its own span. */
  x: ManifestAxis;
  panels: ManifestPanel[];
  points: ManifestPoint[];
}

/**
 * One named thing on a plane: where it sits on both measures, and the figure pinning each.
 *
 * Both figures, not one and not the extremes. A cloud's points are anonymous and only its fit is
 * quotable; these carry their names on the picture, so every number on it is a number a reader
 * will write down — and the rule the whole manifest is built on is that such a number is pinned.
 */
export interface ManifestPosition {
  label: string;
  x: number;
  y: number;
  /** The `crates/figures.json` key whose pin the x coordinate reproduces in magnitude. */
  xFigure: string;
  /** As `xFigure`, for the y coordinate. */
  yFigure: string;
}

/** A handful of named things on two measures at once, computed from the crate that owns it. */
export interface ManifestPlane {
  key: string;
  owner: string;
  /** What one position is, singular and lower case — "anchor rule". */
  subject: string;
  label: string;
  /** Both axes hold zero, which the crate computes them to and `planeSpec` refuses a plane without:
      zero is the rule in force and the boundary the quadrant reading turns on. */
  x: ManifestAxis;
  y: ManifestAxis;
  points: ManifestPosition[];
}

/** One point of a line: a position on the shared index and a value there. */
export interface ManifestCoordinate {
  x: number;
  y: number;
}

/**
 * The worst a line departs from its curve's reference, and where along the index.
 *
 * Unsigned, and a maximum over the whole index rather than a value at any position on it — which
 * is why it is pinned separately from the ends. "Within three points at every horizon" is a claim
 * no coordinate states.
 */
export interface ManifestDeparture {
  at: number;
  gap: number;
  /** The `crates/figures.json` key whose pin the gap reproduces. */
  figure: string;
}

/** One line of a curve: a value at every position on the shared index, with its three pins. */
export interface ManifestLine {
  label: string;
  points: ManifestCoordinate[];
  /** The `crates/figures.json` key whose pin the first point's value reproduces in magnitude. */
  firstFigure: string;
  /** As `firstFigure`, for the last point. */
  lastFigure: string;
  worst: ManifestDeparture;
}

/**
 * The line a curve's lines are read against, which is not one of them.
 *
 * It names no figure, and that absence is deliberate rather than an omission: a reference is a
 * definition — the share of a normal distribution inside ±1σ — and a pin duplicating a definition
 * checks nothing. It is the one number on a curve that is not a measurement, which is exactly why
 * the lines are measured against it.
 */
export interface ManifestReference {
  label: string;
  value: number;
}

/** Named lines over one ordered index, computed from the crate that owns them. */
export interface ManifestCurve {
  key: string;
  owner: string;
  /** What one step along the index is, singular and lower case — "horizon". */
  subject: string;
  label: string;
  /** The shared index, always linear: a curve is read for distances along it. */
  x: ManifestAxis;
  /** Framed to hold every line *and* the reference, which the crate computes it to. */
  y: ManifestAxis;
  reference?: ManifestReference;
  lines: ManifestLine[];
}

/** One subject of a spread: where it sits, and which class it is drawn in. */
export interface ManifestMark {
  label: string;
  x: number;
  y: number;
  /** An index into {@link ManifestSpread.classes}. */
  class: number;
}

/** One panel of a spread: a sub-population, drawn on the frame every panel shares. */
export interface ManifestSlice {
  label: string;
  marks: ManifestMark[];
}

/**
 * A line on a spread where a definition changes, as the segment across the frame.
 *
 * Two endpoints rather than a slope and an intercept, for {@link Fit}'s reason: a line given as a
 * model has to be evaluated somewhere, and evaluating it here would be a second arithmetic in a
 * second language for the crate's one claim. The crate constrains it to a whole-frame line on the
 * way in — a caller able to supply two points could draw one through part of a cloud, and an
 * arbitrary line through a cloud is a claim rather than a definition.
 *
 * It names no figure. See the module note on spreads for why a definition is not pinned.
 */
export interface ManifestBoundary {
  label: string;
  from: [number, number];
  to: [number, number];
}

/** One region of a spread's census: what it is, how many are in it, and its pin. */
export interface ManifestTally {
  label: string;
  subjects: number;
  /** The `crates/figures.json` key whose pin `subjects` reproduces. */
  figure: string;
}

/**
 * A subject the computation could not place, named under the drawing rather than dropped.
 *
 * It carries no figure because there is no number in it to pin — the point is the name. It is
 * named unambiguously enough that a reader cannot look for it in the cloud and find something
 * else: district names are not unique, so the crate writes the IRN alongside.
 */
export interface ManifestMissing {
  label: string;
  why: string;
}

/** A whole population on two signed axes, cut into regions the census counts. */
export interface ManifestSpread {
  key: string;
  owner: string;
  /** What one mark is, singular and lower case — "district". */
  subject: string;
  label: string;
  /** Linear: both axes are signed logarithms already, which a log scale cannot place. */
  x: ManifestAxis;
  y: ManifestAxis;
  /**
   * What each hue means, in the order the marks index. Three at most, and **the first is drawn
   * neutral** — a spread whose classification has a "neither" state puts it first.
   */
  classes: string[];
  boundaries: ManifestBoundary[];
  census: ManifestTally[];
  unreached: ManifestMissing[];
  /** All framed together, which is what makes a difference between them a difference. */
  panels: ManifestSlice[];
}

/** `crates/series.json`, as written by `cargo run -p figures series`. */
export interface SeriesManifest {
  contract: string;
  series: ManifestSeries[];
  scatters: ManifestScatter[];
  planes: ManifestPlane[];
  curves: ManifestCurve[];
  spreads: ManifestSpread[];
}

/** Where `crates/figures series` writes, from `web/` and from the repository root. */
const CANDIDATES = ["../crates/series.json", "crates/series.json"];

let cached: SeriesManifest | null = null;

/**
 * Read and check the series manifest. Memoized.
 *
 * @throws if it is absent, unparseable, declares a contract this module does not read, or is
 * empty — on the same rule as `loadFigureManifest`: a gate with nothing to compare must fail
 * rather than pass, because "the artefact did not build" would otherwise look like "the corpus
 * is correct".
 */
export function loadSeriesManifest(): SeriesManifest {
  if (cached) return cached;
  const path =
    CANDIDATES.map((candidate) => resolve(process.cwd(), candidate)).find((candidate) =>
      existsSync(candidate),
    ) ?? resolve(process.cwd(), CANDIDATES[0]!);

  let parsed: SeriesManifest;
  try {
    parsed = JSON.parse(readFileSync(path, "utf8")) as SeriesManifest;
  } catch (cause) {
    throw new Error(
      `Could not read the series manifest at ${path}. Regenerate it with:\n` +
        `  cargo run --manifest-path crates/Cargo.toml -p figures series > crates/series.json\n\n` +
        `${String(cause)}`,
    );
  }

  if (parsed.contract !== READS_SERIES_CONTRACT) {
    throw new Error(
      `This check reads series manifest contract ${READS_SERIES_CONTRACT}; ${path} declares ` +
        `${String(parsed.contract)}. Either crates/figures changed the series shape and this ` +
        `module has not, or the manifest is from another tree.`,
    );
  }
  if (!Array.isArray(parsed.series) || parsed.series.length === 0) {
    throw new Error(`${path} carries no series, so this check would pass against any corpus.`);
  }
  if (!Array.isArray(parsed.scatters) || parsed.scatters.length === 0) {
    throw new Error(`${path} carries no clouds, so this check would pass against any corpus.`);
  }
  if (!Array.isArray(parsed.planes) || parsed.planes.length === 0) {
    throw new Error(`${path} carries no planes, so this check would pass against any corpus.`);
  }
  if (!Array.isArray(parsed.curves) || parsed.curves.length === 0) {
    throw new Error(`${path} carries no curves, so this check would pass against any corpus.`);
  }
  if (!Array.isArray(parsed.spreads) || parsed.spreads.length === 0) {
    throw new Error(`${path} carries no spreads, so this check would pass against any corpus.`);
  }
  cached = parsed;
  return parsed;
}

/** The rows a reader would quote: the largest and the smallest, by signed value. */
export function endpoints(series: ManifestSeries): SeriesRow[] {
  if (series.rows.length === 0) return [];
  let largest = series.rows[0]!;
  let smallest = series.rows[0]!;
  for (const row of series.rows) {
    if (row.value > largest.value) largest = row;
    if (row.value < smallest.value) smallest = row;
  }
  return largest === smallest ? [largest] : [largest, smallest];
}

/**
 * The fields a chart can be drawn under.
 *
 * Each of these is its own card on the node route, and the chart goes at the end of it. A property
 * is a row of a table and a summary is a lead nothing renders marks on, so neither has anywhere
 * for a chart to go.
 */
export const DRAWABLE_FIELDS: readonly string[] = ["description", "findings"];

/**
 * The series a **page** draws rather than a node, and the node each one's endpoints are bound on.
 *
 * # Why a series can belong to a page at all
 *
 * A `series:` binding sits on a corpus node, and for most charts that is exactly right: a column
 * of four correlations is a claim about property classes, and the node about property classes is
 * where it belongs. The census of the plan's bounds is not a claim about any one component. It is
 * thirty-eight edges of one plan seen at once — the shape of the whole instrument — and there is
 * no node it is *about* that is not simply "the plan". Drawing it on a node would be filing a map
 * of a country under one of its provinces.
 *
 * # What is given up, and what replaces it
 *
 * A page has no `figures:` block, so the endpoint rule has nothing on the page to hold to. This
 * table is what replaces it: it names, per series, the route that draws it and **the node whose
 * prose states its endpoints**. {@link crossCheckSeries} then holds that node to the same
 * requirement a drawing node is held to — it must bind the figures at the series' ends — and the
 * page links to it as the chart's provenance, so a reader who wants the numbers checked is one
 * click from the prose that states them and the claim tags behind it.
 *
 * Declared here rather than in the corpus because it is a fact about the *site*: which route
 * renders which computation is not something a node knows, and `routes.ts` is the module that
 * says where things live.
 */
export const PAGE_SERIES: Readonly<Record<string, { route: string; node: string }>> = {
  "project/bounds-census": { route: routes.BOUNDS, node: "funding-regime/fair-school-funding-plan" },
  "project/what-the-band-held-at-every-horizon": {
    route: routes.METHOD,
    node: "scenario/guarantee-phase-out",
  },
  "project/the-bias-before-the-closure": {
    route: routes.METHOD,
    node: "scenario/guarantee-phase-out",
  },
  "project/the-bias-across-the-closure": {
    route: routes.METHOD,
    node: "scenario/guarantee-phase-out",
  },
};

/**
 * A series as the ranked rows {@link rankSpec} draws.
 *
 * The counterpart of {@link barsOf} for a long column on one count. Where that form has no
 * subject, this one may: the manifest's `marked` is carried straight through, because the row a
 * census is drawn for can be the row at zero and a length cannot encode that.
 *
 * The hover is composed rather than taken from the manifest's `hover`, which these rows mostly do
 * not carry: the count, the population it is out of where that is not the whole, and the
 * provision. That is the row of the table beside the chart, said once.
 */
export function ranksOf(series: ManifestSeries): Rank[] {
  return series.rows.map((row) => {
    const rank: Rank = { label: row.label, value: row.value, hover: hoverForRow(series, row) };
    return row.marked == null ? rank : { ...rank, marked: row.marked };
  });
}

/**
 * What a census row says when a reader points at it: the count, the population it is counted out
 * of, and the authority it rests on.
 *
 * The denominator is printed wherever the manifest gives one, including where it is the whole
 * model. A tooltip is read one row at a time with nothing beside it, so a bare count is a figure
 * the reader has to supply the basis for — and the rows where the basis is *not* the whole are
 * exactly the ones a reader would not think to check.
 */
function hoverForRow(series: ManifestSeries, row: SeriesRow): string {
  const value = formatValue(series.unit, row.value);
  const head =
    row.of === undefined
      ? `${row.label}: ${value}`
      : `${row.label}: ${value} of ${formatValue(series.unit, row.of)}`;
  return [head, row.hover, row.cites].filter(Boolean).join(" — ");
}

/**
 * A row's value, written the way its unit is read: signed, because the chart is.
 *
 * `count` and `money` are the site's own formatters; a ratio is written to four places because
 * that is the precision the correlations the first series carries are pinned at, and a sign is
 * put on every non-zero value so that the tooltip says what the bar draws.
 */
export function formatRow(unit: Unit, value: number): string {
  const sign = value > 0 ? "+" : value < 0 ? "−" : "";
  return `${sign}${formatValue(unit, Math.abs(value))}`;
}

/** A pupil count: one decimal place, because ADM has one, and separated, because it reaches five
    figures. */
const PUPILS = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 1,
  maximumFractionDigits: 1,
});

/**
 * A value written the way the site writes that unit, unsigned.
 *
 * Split out of {@link formatRow} when the clouds arrived: an axis end and a tooltip coordinate are
 * positions rather than changes, and a `+` on one would read as a claim about direction that a
 * coordinate is not making. A ratio is written to four places because that is the precision the
 * slopes and correlations these charts carry are pinned at, and a reader quoting one off a chart
 * should be quoting the figure rather than a rounding of it.
 */
export function formatValue(unit: Unit, value: number): string {
  switch (unit) {
    case "count":
      return count(value);
    case "dollars":
      return money(value);
    case "share":
      return pct(value);
    case "pupils":
      return PUPILS.format(value);
    case "ratio":
      return value.toFixed(4);
  }
}

/**
 * A series as the bars `barSpec` draws.
 *
 * No `direct` labels — a number on every mark is what that field's own doc warns against, and the
 * endpoints are stated in the prose the chart sits under — and no `current`, because a column
 * has no subject. The hover is the manifest's when it gives one and `label: value` otherwise.
 */
export function barsOf(series: ManifestSeries): Bar[] {
  return series.rows.map((row) => ({
    label: row.label,
    value: row.value,
    hover: row.hover ?? `${row.label}: ${formatRow(series.unit, row.value)}`,
  }));
}

/** One small multiple of a grouped series: the run's name, and its rows as bars. */
export interface SeriesMultiple {
  label: string;
  bars: Bar[];
}

/**
 * A grouped series as small multiples drawn on one scale, or `null` where it has no second level.
 *
 * # What the shared scale is for
 *
 * `Row.group` is a second level of the axis, and a series carrying one is seven charts of five
 * bars rather than one chart of thirty-five: the rows within a run are comparable and the runs are
 * comparable to each other, which a single column of thirty-five bars with its categories repeated
 * seven times says neither of. Small multiples say both — but only if they share a scale, because
 * a panel fitted to its own rows draws its largest bar at full width whatever that bar is worth.
 *
 * So both ends are computed over **every** row of the series and handed to every panel. That is
 * also why they are computed here rather than in the route: the scale is a property of the series,
 * and a caller who worked one out per panel would have drawn exactly the picture this prevents.
 *
 * # Why not one scale across two series
 *
 * Because the two this was written for differ by two orders of magnitude — the first fifth of one
 * anchor rule is $138.58 on the wealth axis and $4.06 on the disadvantaged-share axis — and a
 * scale wide enough for both flattens the second into a row of hairlines. They are separate
 * documents so that "scale per axis" is structural rather than a caller's discipline, and the
 * prose that draws them says the scales differ.
 *
 * # The panel where every bar is zero
 *
 * One of the seven anchor rules does nothing at all, on either axis, so each of the two sets has a
 * panel of five zeros in it: five row names against a zero rule and no ink whatsoever. That is the
 * correct picture and it reads as a broken one — the reader's question is *did this fail to
 * render*, not *is this rule inert* — and the shared scale that makes the other six panels legible
 * is exactly what guarantees the zeros have nowhere to show.
 *
 * So a panel whose every bar is zero direct-labels all of them. `Bar.direct` says never a number
 * on every mark, and that rule is about a chart where the bars already carry the quantity; here
 * there are no bars, so the number is the only thing the panel has to say. It is the one panel in
 * a set that can want this, because a panel with a single non-zero bar draws that bar and the
 * emptiness beside it is then a comparison rather than an absence.
 *
 * `null` where any row carries no group: a multiple of one panel is a bar chart with a heading on
 * it, and the route draws that with {@link barsOf} instead.
 */
export function multiplesOf(
  series: ManifestSeries,
): { panels: SeriesMultiple[]; max: number; min: number; labelChars: number } | null {
  if (series.rows.length === 0 || series.rows.some((row) => row.group === undefined)) return null;
  const panels: SeriesMultiple[] = [];
  for (const row of series.rows) {
    const label = row.group!;
    let panel = panels.find((p) => p.label === label);
    if (!panel) {
      panel = { label, bars: [] };
      panels.push(panel);
    }
    panel.bars.push({
      label: row.label,
      value: row.value,
      // The run's name as well as the row's, because a tooltip is read with nothing beside it and
      // the name of the panel it belongs to is above the chart rather than in it.
      hover: row.hover ?? `${label} — ${row.label}: ${formatRow(series.unit, row.value)}`,
    });
  }
  for (const panel of panels) {
    if (panel.bars.every((bar) => bar.value === 0)) {
      for (const bar of panel.bars) bar.direct = formatRow(series.unit, bar.value);
    }
  }
  return {
    panels,
    max: Math.max(...series.rows.map((row) => Math.abs(row.value)), 1),
    min: Math.min(0, ...series.rows.map((row) => row.value)),
    // Part of the shared scale rather than a detail of the panels that happen to carry a label:
    // a panel reserving label room its siblings do not draws zero at a different pixel from
    // theirs. See `barSpec`'s `labelChars`.
    labelChars: Math.max(0, ...panels.flatMap((p) => p.bars.map((b) => b.direct?.length ?? 0))),
  };
}

/**
 * One panel of a cloud, as `scatterSpec` wants it.
 *
 * The frame is the crate's — `xDomain` and `yDomain` come straight out of the manifest, and
 * `scatterSpec` refuses a fit without both, because a y axis fitted to its own points would redraw
 * the slope that is the whole claim. The panels of one cloud therefore share an x domain and span
 * the same number of log units on y, which is what makes them comparable to each other and a slope
 * of one a diagonal.
 *
 * No `series` and no `band` on the points: the cloud is one population with nothing to split it
 * into, and a second channel drawn where there is no second fact is decoration. #376's `muted` is
 * for saying *this point is outside the population you asked about* alongside a colouring, and
 * these panels keep every district, so hue alone carries them.
 */
export interface CloudPanel {
  label: string;
  points: ScatterPoint[];
  x: { label: string; format: (v: number) => string; log: boolean };
  y: { label: string; format: (v: number) => string; log: boolean };
  fit: Fit;
  /** Handed to `scatterSpec` as `xDomain`/`yDomain`. */
  xDomain: [number, number];
  yDomain: [number, number];
}

/** The hover a point carries: which subject it is, and where it sits on both axes. */
function hoverFor(cloud: ManifestScatter, panel: ManifestPanel, point: ManifestPoint, at: number) {
  const y = point.ys[at];
  return (
    `${point.label}: ${cloud.x.label} ${formatValue(cloud.x.unit, point.x)}, ` +
    `${panel.y.label} ${y === undefined ? "—" : formatValue(panel.y.unit, y)}`
  );
}

/**
 * A cloud as the panels a card draws left to right.
 *
 * @throws if a point carries no value under some panel. The crate refuses to write such a
 * document — every point is asserted to carry one value per panel on both sides of the manifest —
 * so this fires only on a hand-edited or truncated file, and a dot silently dropped from one panel
 * and not the other is the one defect that would make the pair say something false.
 */
export function panelsOf(cloud: ManifestScatter): CloudPanel[] {
  return cloud.panels.map((panel, at) => {
    const points = cloud.points.map((point) => {
      const y = point.ys[at];
      if (y === undefined) {
        throw new Error(
          `${cloud.key}: "${point.label}" carries no value under "${panel.label}". Regenerate ` +
            `crates/series.json; a cloud missing a point from one panel and not the other would ` +
            `compare two different populations.`,
        );
      }
      return { x: point.x, y, hover: hoverFor(cloud, panel, point, at) };
    });
    return {
      label: panel.label,
      points,
      x: {
        label: cloud.x.label,
        format: (v: number) => formatValue(cloud.x.unit, v),
        log: cloud.x.log,
      },
      y: {
        label: panel.y.label,
        format: (v: number) => formatValue(panel.y.unit, v),
        log: panel.y.log,
      },
      fit: {
        from: { x: panel.fit.from[0], y: panel.fit.from[1] },
        to: { x: panel.fit.to[0], y: panel.fit.to[1] },
        slope: panel.fit.slope,
        rSquared: panel.fit.rSquared,
      },
      xDomain: [cloud.x.min, cloud.x.max],
      yDomain: [panel.y.min, panel.y.max],
    };
  });
}

/**
 * A plane's positions as `planeSpec` draws them.
 *
 * Signed, through {@link formatRow} rather than {@link formatValue}: a cloud's coordinates are
 * positions and a plus on one would assert a direction it is not claiming, where every coordinate
 * here is a difference against the rule in force and its sign is the finding.
 */
export function placesOf(plane: ManifestPlane): Place[] {
  return plane.points.map((point) => ({
    label: point.label,
    x: point.x,
    y: point.y,
    hover:
      `${point.label}: ${plane.x.label} ${formatRow(plane.x.unit, point.x)}, ` +
      `${plane.y.label} ${formatRow(plane.y.unit, point.y)}`,
  }));
}

/**
 * The three numbers one line of a curve owes a figure, in the order they are read.
 *
 * A column's endpoint rule picks the largest and smallest row, which on a curve would pick two
 * positions nobody quotes: what a reader takes off a line drawn over an index is where it starts,
 * where it ends, and how far it ever gets from the reference. That is the rule, and it is per
 * line rather than per curve — two lines on one picture are two claims.
 */
function endsOf(line: ManifestLine): { what: string; value: number; figure: string }[] {
  const first = line.points[0];
  const last = line.points[line.points.length - 1];
  return [
    { what: `starts at ${first?.y ?? "nothing"}`, value: first?.y ?? 0, figure: line.firstFigure },
    { what: `ends at ${last?.y ?? "nothing"}`, value: last?.y ?? 0, figure: line.lastFigure },
    {
      what: `departs from the reference by ${line.worst.gap} at ${line.worst.at}`,
      value: line.worst.gap,
      figure: line.worst.figure,
    },
  ];
}

/**
 * A curve's two lines as {@link seriesSpec} draws them.
 *
 * The same pairing the historical charts use, which is why the form is reused rather than added
 * to: two quantities in one unit over one ordered index. What differs is only that the index is a
 * horizon, and `SeriesPoint.at` is the field that stopped assuming otherwise.
 *
 * Exactly two lines, because that is what the form admits. A curve carrying a third would be a
 * crate change the caller cannot paper over, so this throws rather than silently dropping one.
 */
export function pointsOf(curve: ManifestCurve): SeriesPoint[] {
  const [a, b] = curve.lines;
  if (!a || !b || curve.lines.length !== 2) {
    throw new Error(
      `${curve.key} has ${curve.lines.length} lines and seriesSpec draws two. A third quantity ` +
        `goes in the table under the chart, which is what the table is for.`,
    );
  }
  return a.points.map((point, index) => ({
    at: point.x,
    a: point.y,
    b: b.points[index]?.y ?? null,
  }));
}

/**
 * Which hue each class of a spread is drawn in, by index.
 *
 * Positional, and it has to be: the palette's two hues are named for the site's one canonical
 * split — formula against guarantee — and a spread classifies by whatever its own subject is.
 * Both of the guarantee's origins are districts the *guarantee* pays, so neither has a claim on
 * either name. What makes a hue mean something here is the legend, which the crate supplies as
 * {@link ManifestSpread.classes} and the page must render.
 *
 * Neutral leads, because the crate's contract puts the unclassified state first — "any mark with
 * no polarity. Never a hue," which is exactly what a subject outside a classification is. The two
 * hues follow in the order `plot/tokens.ts` declares them.
 */
const CLASS_HUES = ["neutral", "formula", "guarantee"] as const;

/** One panel of a spread, as a card draws it. */
export interface SpreadPanel {
  label: string;
  points: ScatterPoint[];
  x: { label: string; format: (v: number) => string; log: boolean };
  y: { label: string; format: (v: number) => string; log: boolean };
  /** Handed to `scatterSpec` as `xDomain`/`yDomain`, and the same pair for every panel. */
  xDomain: [number, number];
  yDomain: [number, number];
  /** Handed to `scatterSpec` as `rules`. The same lines on every panel, for the same reason. */
  rules: { label: string; from: { x: number; y: number }; to: { x: number; y: number } }[];
}

/**
 * A spread as the panels a card draws.
 *
 * Every panel carries the whole spread's frame and the whole spread's boundaries, because that is
 * the only thing that makes a difference between two panels a difference in the data. The crate
 * computes the frame across all of them together; this hands the same pair to each.
 *
 * @throws if a mark names a class the spread does not have. The crate asserts it on the way in, so
 * this fires only on a hand-edited or truncated file — and a mark silently drawn in the wrong hue
 * is the one defect that would make the picture say the opposite of what the census counts.
 */
export function regionsOf(spread: ManifestSpread): SpreadPanel[] {
  const rules = spread.boundaries.map((boundary) => ({
    label: boundary.label,
    from: { x: boundary.from[0], y: boundary.from[1] },
    to: { x: boundary.to[0], y: boundary.to[1] },
  }));
  return spread.panels.map((panel) => ({
    label: panel.label,
    points: panel.marks.map((mark) => {
      const what = spread.classes[mark.class];
      if (what === undefined) {
        throw new Error(
          `${spread.key}: "${mark.label}" is class ${mark.class} and the spread declares ` +
            `${spread.classes.length}. Regenerate crates/series.json; a mark drawn in a hue the ` +
            `legend does not name is a mark saying something nothing counted.`,
        );
      }
      return {
        x: mark.x,
        y: mark.y,
        series: CLASS_HUES[Math.min(mark.class, CLASS_HUES.length - 1)]!,
        hover:
          `${mark.label}: ${spread.x.label} ${formatRow(spread.x.unit, mark.x)}, ` +
          `${spread.y.label} ${formatRow(spread.y.unit, mark.y)} — ${what}`,
      } satisfies ScatterPoint;
    }),
    x: {
      label: spread.x.label,
      format: (v: number) => formatRow(spread.x.unit, v),
      log: spread.x.log,
    },
    y: {
      label: spread.y.label,
      format: (v: number) => formatRow(spread.y.unit, v),
      log: spread.y.log,
    },
    xDomain: [spread.x.min, spread.x.max],
    yDomain: [spread.y.min, spread.y.max],
    rules,
  }));
}

/** Each position the check can fail in. Every one is produced on purpose in the spec. */
export type SeriesDiscrepancyKind =
  /** The node binds a key the series manifest does not carry. */
  | "unknown-key"
  /** The node draws the series under a field it does not have, or one no chart can go under. */
  | "field-missing"
  /** The node draws a crate's column and cites that crate in no claim tag. */
  | "unattributed"
  /** An endpoint of the series is a figure this node does not bind. */
  | "endpoints-unbound"
  /** A number printed on a panel of a cloud is a figure this node does not bind. */
  | "fit-unbound"
  /** A coordinate of a position on a plane is a figure this node does not bind. */
  | "position-unbound"
  /** An end or the worst departure of a line on a curve is a figure this node does not bind. */
  | "line-unbound"
  /** A region of a spread's census is a figure this node does not bind. */
  | "census-unbound"
  /** The manifest exports a series, a cloud, a plane, a curve or a spread no node draws. */
  | "uncited-series"
  /** A series {@link PAGE_SERIES} says a page draws names a node the corpus does not hold. */
  | "page-source-missing";

/** One thing wrong between a node's `series:` block and the manifests. */
export interface SeriesDiscrepancy {
  node: string;
  key: string;
  kind: SeriesDiscrepancyKind;
  message: string;
}

/**
 * Check every `series:` binding in the corpus against the series manifest and the node's own
 * figure bindings.
 *
 * Pure over its arguments, for the reason `crossCheck` is: each kind above is broken on purpose
 * in `tests/unit/corpusSeries.spec.ts`, and a gate whose failures are only hypothesised is a gate
 * nobody has run.
 */
export function crossCheckSeries(nodes: Node[], manifest: SeriesManifest): SeriesDiscrepancy[] {
  const byKey = new Map(manifest.series.map((series) => [series.key, series]));
  const cloudByKey = new Map(manifest.scatters.map((cloud) => [cloud.key, cloud]));
  const planeByKey = new Map(manifest.planes.map((plane) => [plane.key, plane]));
  const curveByKey = new Map(manifest.curves.map((curve) => [curve.key, curve]));
  const spreadByKey = new Map(manifest.spreads.map((spread) => [spread.key, spread]));
  const found: SeriesDiscrepancy[] = [];
  const drawn = new Set<string>();

  for (const node of nodes) {
    if (node.series.length === 0) continue;
    const fields = proseFields(node);
    const cited = citedCrates(node);
    const bound = new Set(node.figures.map((figure) => figure.key));

    for (const entry of node.series) {
      const at = (kind: SeriesDiscrepancyKind, message: string) =>
        found.push({ node: node.id, key: entry.key, kind, message });

      /*
       * One `series:` block, five shapes behind it.
       *
       * A node binds what it draws by key and does not say which array the key is in — a binding
       * is "draw this computation under this field", and whether the crate answered with a column,
       * a cloud, a plane, a curve or a spread is the crate's business. The five differ in what has
       * to be bound with them, and that is the whole of the difference below.
       */
      const series = byKey.get(entry.key);
      const cloud = cloudByKey.get(entry.key);
      const plane = planeByKey.get(entry.key);
      const curve = curveByKey.get(entry.key);
      const spread = spreadByKey.get(entry.key);
      if (!series && !cloud && !plane && !curve && !spread) {
        at(
          "unknown-key",
          `draws "${entry.key}", which crates/series.json does not carry. Either the key was ` +
            `renamed in crates/figures/src/lib.rs or the manifest is stale.`,
        );
        continue;
      }
      const owner = (series ?? cloud ?? plane ?? curve ?? spread)!.owner;
      drawn.add(entry.key);

      if (!DRAWABLE_FIELDS.includes(entry.field)) {
        at(
          "field-missing",
          `draws it under "${entry.field}", which no chart can be drawn under. A chart goes at ` +
            `the end of one of: ${DRAWABLE_FIELDS.join(", ")}.`,
        );
      } else if (!fields.has(entry.field)) {
        at(
          "field-missing",
          `draws it under a field "${entry.field}" this node does not have. Fields available: ` +
            `${[...fields.keys()].filter((field) => DRAWABLE_FIELDS.includes(field)).join(", ")}.`,
        );
      }

      if (!cited.has(owner)) {
        at(
          "unattributed",
          `draws a ${owner} chart and cites ${owner} in no claim tag. A chart from a crate the ` +
            `node never names is a chart nobody reading the page could trace.`,
        );
      }

      if (cloud) {
        for (const panel of cloud.panels) {
          for (const [what, key] of [
            ["slope", panel.fit.slopeFigure],
            ["r-squared", panel.fit.rSquaredFigure],
          ] as const) {
            if (!bound.has(key)) {
              at(
                "fit-unbound",
                `prints the ${what} of "${panel.label}" on the panel, which is ${key}, and this ` +
                  `node does not bind that figure. Bind it in figures: first — the slope and the ` +
                  `r-squared are the only numbers a reader can take off a cloud, so they are the ` +
                  `cloud's endpoints.`,
              );
            }
          }
        }
        continue;
      }

      if (plane) {
        for (const position of plane.points) {
          for (const [which, axis, key] of [
            ["x", plane.x, position.xFigure],
            ["y", plane.y, position.yFigure],
          ] as const) {
            if (!bound.has(key)) {
              at(
                "position-unbound",
                `places "${position.label}" at ${which} = ${which === "x" ? position.x : position.y} ` +
                  `on "${axis.label}", which is ${key}, and this node does not bind that figure. ` +
                  `Bind it in figures: first — a plane prints its subjects' names on the picture, ` +
                  `so every coordinate on it is a number a reader will quote by name and every ` +
                  `coordinate is pinned.`,
              );
            }
          }
        }
        continue;
      }

      if (curve) {
        for (const line of curve.lines) {
          for (const end of endsOf(line)) {
            if (!bound.has(end.figure)) {
              at(
                "line-unbound",
                `draws "${line.label}", which ${end.what}, and that is ${end.figure}, a figure ` +
                  `this node does not bind. Bind it in figures: first — a line over an index is ` +
                  `read at its two ends and at its worst departure from the reference, so those ` +
                  `three are the numbers a reader will quote off it.`,
              );
            }
          }
        }
        continue;
      }

      if (spread) {
        for (const tally of spread.census) {
          if (!bound.has(tally.figure)) {
            at(
              "census-unbound",
              `counts ${tally.subjects} in "${tally.label}", which is ${tally.figure}, a figure ` +
                `this node does not bind. Bind it in figures: first — a spread has no extreme ` +
                `and no fitted line, so how many are in a region is the only number a reader ` +
                `can take off one, and an empty region is what the picture exists to show.`,
            );
          }
        }
        continue;
      }

      for (const end of endpoints(series!)) {
        if (end.figure === undefined) {
          at(
            "endpoints-unbound",
            `has "${end.label}" at ${end.value} as an endpoint and the manifest names no figure ` +
              `for it. crates/figures/tests/ should have refused this; the manifest is stale.`,
          );
        } else if (!bound.has(end.figure)) {
          at(
            "endpoints-unbound",
            `has "${end.label}" at ${end.value} as an endpoint, which is ${end.figure}, and ` +
              `this node does not bind that figure. Bind it in figures: first — the numbers a ` +
              `reader takes off a chart have to be numbers the prose states.`,
          );
        }
      }
    }
  }

  /*
   * A series a page draws is drawn.
   *
   * The rest of the page's obligation — that the key exists and that the node it names binds the
   * figures at its ends — is {@link crossCheckPageSeries}, because it is a question about
   * {@link PAGE_SERIES} and not about the nodes and manifest handed in here. Only this one line
   * belongs in this function: without it every page-drawn series is reported as computed for
   * nobody, which is the one thing it is not.
   */
  for (const key of Object.keys(PAGE_SERIES)) drawn.add(key);

  const exported = [
    ...manifest.series.map((series) => [series.key, "column"] as const),
    ...manifest.scatters.map((cloud) => [cloud.key, "cloud"] as const),
    ...manifest.planes.map((plane) => [plane.key, "plane"] as const),
    ...manifest.curves.map((curve) => [curve.key, "curve"] as const),
    ...manifest.spreads.map((spread) => [spread.key, "spread"] as const),
  ];
  for (const [key, what] of exported) {
    if (!drawn.has(key)) {
      found.push({
        node: "crates/series.json",
        key,
        kind: "uncited-series",
        message:
          `exports ${key} and no node draws it. A ${what} computed for nobody is a ${what} ` +
          `nothing checks; bind it on a node, declare the page that draws it in PAGE_SERIES, ` +
          `or remove it.`,
      });
    }
  }

  return found;
}

/**
 * The series a page draws, held on the node that states their ends.
 *
 * The endpoint rule, moved rather than waived: a node that draws a series must bind the figures at
 * its largest and smallest rows, and a page has no `figures:` block to bind anything with — so
 * {@link PAGE_SERIES} names the node that answers for it, and that node is held to exactly what a
 * drawing node would be.
 *
 * Separate from {@link crossCheckSeries} because it asks a different question. That one is pure
 * over the nodes and manifest it is given, and every test of it hands over a two-node fixture;
 * this one is about a table in this module, so asking it of a fixture would report the whole of
 * `PAGE_SERIES` as missing from every fixture manifest ever written. Both are run over the real
 * corpus by the same test.
 */
export function crossCheckPageSeries(
  nodes: Node[],
  manifest: SeriesManifest,
): SeriesDiscrepancy[] {
  const byKey = new Map(manifest.series.map((series) => [series.key, series]));
  const curveByKey = new Map(manifest.curves.map((curve) => [curve.key, curve]));
  const byId = new Map(nodes.map((node) => [node.id, node]));
  const found: SeriesDiscrepancy[] = [];

  for (const [key, source] of Object.entries(PAGE_SERIES)) {
    const series = byKey.get(key);
    const curve = curveByKey.get(key);
    if (!series && !curve) {
      found.push({
        node: source.node,
        key,
        kind: "unknown-key",
        message:
          `is declared in PAGE_SERIES as drawn by ${source.route}, and crates/series.json does ` +
          `not carry it. Either the key was renamed in crates/figures/src/lib.rs or the page no ` +
          `longer draws it.`,
      });
      continue;
    }
    const node = byId.get(source.node);
    if (!node) {
      found.push({
        node: source.node,
        key,
        kind: "page-source-missing",
        message:
          `is drawn by ${source.route}, which names this node as where its endpoints are stated ` +
          `— and the corpus has no such node. A page's chart is held by the node it cites; ` +
          `without one it is held by nothing.`,
      });
      continue;
    }
    const bound = new Set(node.figures.map((figure) => figure.key));

    if (curve) {
      for (const line of curve.lines) {
        for (const end of endsOf(line)) {
          if (!bound.has(end.figure)) {
            found.push({
              node: source.node,
              key,
              kind: "line-unbound",
              message:
                `is drawn by ${source.route}, whose line "${line.label}" ${end.what}, and that ` +
                `is ${end.figure}, a figure this node does not bind. Bind it in figures: first ` +
                `— a page has no figures: block of its own, so the node it cites is what holds ` +
                `the numbers a reader takes off the chart.`,
            });
          }
        }
      }
      continue;
    }

    for (const end of endpoints(series!)) {
      if (end.figure === undefined) {
        found.push({
          node: source.node,
          key,
          kind: "endpoints-unbound",
          message:
            `has "${end.label}" at ${end.value} as an endpoint and the manifest names no figure ` +
            `for it. crates/figures/tests/ should have refused this; the manifest is stale.`,
        });
      } else if (!bound.has(end.figure)) {
        found.push({
          node: source.node,
          key,
          kind: "endpoints-unbound",
          message:
            `is drawn by ${source.route}, whose endpoint "${end.label}" at ${end.value} is ` +
            `${end.figure}, and this node does not bind that figure. Bind it in figures: first — ` +
            `a page has no figures: block of its own, so the node it cites is what holds the ` +
            `numbers a reader takes off the chart.`,
        });
      }
    }
  }
  return found;
}

/**
 * Every row that names a figure reproduces it in magnitude, manifest against manifest.
 *
 * The crate test holds a row to the figure's *pin*; this holds the committed series document to
 * the committed figure document, which is the staleness two artefacts from one generator can
 * have between them. The two are one computation, so they agree to the bit or not at all.
 */
export function rowsAgainstFigures(manifest: SeriesManifest, figures: Manifest): string[] {
  const byKey = new Map(figures.figures.map((figure) => [figure.key, figure]));
  const out: string[] = [];
  for (const series of manifest.series) {
    for (const row of series.rows) {
      if (row.figure === undefined) continue;
      const figure = byKey.get(row.figure);
      if (!figure) {
        out.push(`${series.key}: "${row.label}" names ${row.figure}, which figures.json lacks`);
        continue;
      }
      if (figure.unit !== series.unit) {
        out.push(
          `${series.key}: "${row.label}" is a ${series.unit} and ${row.figure} is a ${figure.unit}`,
        );
      }
      if (Math.abs(Math.abs(row.value) - figure.value) > Number.EPSILON) {
        out.push(
          `${series.key}: "${row.label}" is ${row.value} and ${row.figure} is ${figure.value}; ` +
            `one of the two manifests is stale`,
        );
      }
    }
  }
  return out;
}

/**
 * Every panel's two numbers reproduce the figures they name, manifest against manifest.
 *
 * {@link rowsAgainstFigures}'s sibling and the same check: the crate holds a fit to the figure's
 * *pin*, this holds the committed series document to the committed figure document, which is the
 * staleness two artefacts from one generator can have between them. They are one computation, so
 * they agree to the bit or not at all.
 *
 * A slope is compared in magnitude and an r-squared as it stands. That asymmetry is the sign
 * convention rather than a leniency: the figure manifest exports a signed quantity unsigned with
 * the direction in its key, because the corpus's numeral reader has no sign group, and an
 * r-squared has no direction to put there.
 */
export function fitsAgainstFigures(manifest: SeriesManifest, figures: Manifest): string[] {
  const byKey = new Map(figures.figures.map((figure) => [figure.key, figure]));
  const out: string[] = [];
  for (const cloud of manifest.scatters) {
    for (const panel of cloud.panels) {
      for (const [what, key, value] of [
        ["slope", panel.fit.slopeFigure, Math.abs(panel.fit.slope)],
        ["r-squared", panel.fit.rSquaredFigure, panel.fit.rSquared],
      ] as const) {
        const figure = byKey.get(key);
        if (!figure) {
          out.push(`${cloud.key}: "${panel.label}" names ${key}, which figures.json lacks`);
          continue;
        }
        if (figure.unit !== "ratio") {
          out.push(`${cloud.key}: "${panel.label}" has a ${what} and ${key} is a ${figure.unit}`);
        }
        if (Math.abs(value - figure.value) > Number.EPSILON) {
          out.push(
            `${cloud.key}: "${panel.label}" has ${what} ${value} and ${key} is ${figure.value}; ` +
              `one of the two manifests is stale`,
          );
        }
      }
    }
  }
  return out;
}

/**
 * Every position reproduces both figures it names, manifest against manifest.
 *
 * {@link rowsAgainstFigures}'s and {@link fitsAgainstFigures}'s third sibling and the same check:
 * the crate holds a coordinate to the figure's *pin*, this holds the committed series document to
 * the committed figure document, which is the staleness two artefacts from one generator can have
 * between them. They are one computation, so they agree to the bit or not at all.
 *
 * Both coordinates in magnitude, because both are signed: a rule that writes $107,611,728.73 less
 * guarantee than the anchor in force is pinned as a cut, with the direction in the key, on the
 * convention the corpus's numeral reader forces and the rest of this module already follows.
 */
export function positionsAgainstFigures(manifest: SeriesManifest, figures: Manifest): string[] {
  const byKey = new Map(figures.figures.map((figure) => [figure.key, figure]));
  const out: string[] = [];
  for (const plane of manifest.planes) {
    for (const point of plane.points) {
      for (const [which, axis, key, value] of [
        ["x", plane.x, point.xFigure, point.x],
        ["y", plane.y, point.yFigure, point.y],
      ] as const) {
        const figure = byKey.get(key);
        if (!figure) {
          out.push(`${plane.key}: "${point.label}" names ${key} for ${which}, which figures.json lacks`);
          continue;
        }
        if (figure.unit !== axis.unit) {
          out.push(
            `${plane.key}: "${point.label}" has a ${which} in ${axis.unit} and ${key} is a ${figure.unit}`,
          );
        }
        if (Math.abs(Math.abs(value) - figure.value) > Number.EPSILON) {
          out.push(
            `${plane.key}: "${point.label}" is at ${which} = ${value} and ${key} is ${figure.value}; ` +
              `one of the two manifests is stale`,
          );
        }
      }
    }
  }
  return out;
}

/**
 * Every line reproduces the three figures it names, manifest against manifest.
 *
 * The fourth sibling of {@link rowsAgainstFigures}, {@link fitsAgainstFigures} and
 * {@link positionsAgainstFigures}, and the same check: the crate holds a line to the figures'
 * *pins*, this holds the committed series document to the committed figure document, which is the
 * staleness two artefacts from one generator can have between them. They are one computation, so
 * they agree to the bit or not at all.
 *
 * In magnitude, as the others are, though nothing on a curve is signed today: a share is a share
 * and a departure is a distance. Written the same way regardless, because the convention is the
 * figure manifest's — it exports magnitudes with the direction in the key — and a check that
 * happens to be right because its inputs are all positive is a check that breaks on the first
 * curve over a signed quantity.
 *
 * The reference is not checked here, because it names no figure. See {@link ManifestReference}.
 */
export function linesAgainstFigures(manifest: SeriesManifest, figures: Manifest): string[] {
  const byKey = new Map(figures.figures.map((figure) => [figure.key, figure]));
  const out: string[] = [];
  for (const curve of manifest.curves) {
    for (const line of curve.lines) {
      for (const end of endsOf(line)) {
        const figure = byKey.get(end.figure);
        if (!figure) {
          out.push(`${curve.key}: "${line.label}" names ${end.figure}, which figures.json lacks`);
          continue;
        }
        if (figure.unit !== curve.y.unit) {
          out.push(
            `${curve.key}: "${line.label}" is drawn in ${curve.y.unit} and ${end.figure} is a ` +
              `${figure.unit}`,
          );
        }
        if (Math.abs(Math.abs(end.value) - figure.value) > Number.EPSILON) {
          out.push(
            `${curve.key}: "${line.label}" ${end.what} and ${end.figure} is ${figure.value}; ` +
              `one of the two manifests is stale`,
          );
        }
      }
    }
  }
  return out;
}

/**
 * Every region of every spread reproduces the figure it names, and every panel is a region.
 *
 * The staleness leg of the spread rule, beside {@link rowsAgainstFigures} and the three like it.
 * Compared **exactly** where those compare in magnitude: a count is unsigned already and it is an
 * integer, so there is no rounding to tolerate and no direction that could have been dropped into
 * the key. A tolerance here would only be somewhere for a district to hide.
 *
 * The panel half is the cheap failure this exists to stop: a panel drawing a hundred and two
 * marks under a caption saying a hundred and three. A panel's size is the first number a reader
 * takes off a small multiple, so it has to be one of the counts the figure manifest stands behind
 * rather than whatever the loop happened to produce — and so does the total, which is the
 * denominator every other region is read against.
 *
 * The boundaries are not checked, because they name no figure. See {@link ManifestBoundary}.
 */
export function censusAgainstFigures(manifest: SeriesManifest, figures: Manifest): string[] {
  const byKey = new Map(figures.figures.map((figure) => [figure.key, figure]));
  const out: string[] = [];
  for (const spread of manifest.spreads) {
    for (const tally of spread.census) {
      const figure = byKey.get(tally.figure);
      if (!figure) {
        out.push(
          `${spread.key}: "${tally.label}" names ${tally.figure}, which figures.json lacks`,
        );
        continue;
      }
      if (figure.unit !== "count") {
        out.push(
          `${spread.key}: "${tally.label}" counts ${spread.subject}s and ${tally.figure} is a ` +
            `${figure.unit}`,
        );
      }
      if (Math.abs(tally.subjects - figure.value) >= 0.5) {
        out.push(
          `${spread.key}: "${tally.label}" holds ${tally.subjects} and ${tally.figure} is ` +
            `${figure.value}; one of the two manifests is stale`,
        );
      }
    }

    const counted = new Set(spread.census.map((tally) => tally.subjects));
    for (const panel of spread.panels) {
      if (!counted.has(panel.marks.length)) {
        out.push(
          `${spread.key}: the panel "${panel.label}" draws ${panel.marks.length} and no region ` +
            `of the census counts that many, so its caption is a number nothing pins`,
        );
      }
    }
    const drawn = spread.panels.reduce((sum, panel) => sum + panel.marks.length, 0);
    if (!counted.has(drawn)) {
      out.push(
        `${spread.key}: draws ${drawn} ${spread.subject}s and no region counts the whole of ` +
          `them, which is the denominator every other region is read against`,
      );
    }
    if (spread.unreached.length === 0) {
      out.push(
        `${spread.key}: names nobody it could not place. A count is a count of a population, ` +
          `and an empty list is also what a dropped filter looks like`,
      );
    }
  }
  return out;
}
