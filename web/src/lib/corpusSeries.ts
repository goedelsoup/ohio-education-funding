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
 * # Sign
 *
 * Rows are signed, because a bar below the zero rule is what a chart is for, and so are slopes —
 * −0.0429 is the finding on the second panel of the first cloud. The figure manifest exports a
 * signed quantity as a magnitude with the direction in its key, so {@link rowsAgainstFigures} and
 * {@link fitsAgainstFigures} compare a magnitude to the figure it names.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import type { Bar, Fit, Place, Rank, ScatterPoint } from "./chart.ts";
import type { Node } from "./corpus.ts";
import { citedCrates, proseFields, type Manifest, type Unit } from "./corpusFigures.ts";
import { count, money, pct } from "./format.ts";
import * as routes from "./routes.ts";

/**
 * The series manifest contract this module reads. Versioned apart from the figure manifest's:
 * the two documents have different shapes and different readers, and a field added to a row is no
 * reason for the scalar check to refuse a document it still reads correctly.
 */
export const READS_SERIES_CONTRACT = "4.0.0";

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

/** `crates/series.json`, as written by `cargo run -p figures series`. */
export interface SeriesManifest {
  contract: string;
  series: ManifestSeries[];
  scatters: ManifestScatter[];
  planes: ManifestPlane[];
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
 * `null` where any row carries no group: a multiple of one panel is a bar chart with a heading on
 * it, and the route draws that with {@link barsOf} instead.
 */
export function multiplesOf(
  series: ManifestSeries,
): { panels: SeriesMultiple[]; max: number; min: number } | null {
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
  return {
    panels,
    max: Math.max(...series.rows.map((row) => Math.abs(row.value)), 1),
    min: Math.min(0, ...series.rows.map((row) => row.value)),
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
  /** The manifest exports a series, a cloud or a plane no node draws. */
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
       * One `series:` block, three shapes behind it.
       *
       * A node binds what it draws by key and does not say which array the key is in — a binding
       * is "draw this computation under this field", and whether the crate answered with a column,
       * a cloud or a plane is the crate's business. The three differ in what has to be bound with
       * them, and that is the whole of the difference below.
       */
      const series = byKey.get(entry.key);
      const cloud = cloudByKey.get(entry.key);
      const plane = planeByKey.get(entry.key);
      if (!series && !cloud && !plane) {
        at(
          "unknown-key",
          `draws "${entry.key}", which crates/series.json does not carry. Either the key was ` +
            `renamed in crates/figures/src/lib.rs or the manifest is stale.`,
        );
        continue;
      }
      const owner = (series ?? cloud ?? plane)!.owner;
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
  const byId = new Map(nodes.map((node) => [node.id, node]));
  const found: SeriesDiscrepancy[] = [];

  for (const [key, source] of Object.entries(PAGE_SERIES)) {
    const series = byKey.get(key);
    if (!series) {
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
    for (const end of endpoints(series)) {
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
