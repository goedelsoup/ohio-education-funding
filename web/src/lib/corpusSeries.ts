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
 * # Sign
 *
 * Rows are signed, because a bar below the zero rule is what a chart is for, and so are slopes —
 * −0.0429 is the finding on the second panel of the first cloud. The figure manifest exports a
 * signed quantity as a magnitude with the direction in its key, so {@link rowsAgainstFigures} and
 * {@link fitsAgainstFigures} compare a magnitude to the figure it names.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import type { Bar, Fit, ScatterPoint } from "./chart.ts";
import type { Node } from "./corpus.ts";
import { citedCrates, proseFields, type Manifest, type Unit } from "./corpusFigures.ts";
import { count, money, pct } from "./format.ts";

/**
 * The series manifest contract this module reads. Versioned apart from the figure manifest's:
 * the two documents have different shapes and different readers, and a field added to a row is no
 * reason for the scalar check to refuse a document it still reads correctly.
 */
export const READS_SERIES_CONTRACT = "2.0.0";

/** One row of a series: a category and its signed value. */
export interface SeriesRow {
  label: string;
  value: number;
  /** What the mark says on hover, when the label and the value are not enough on their own. */
  hover?: string;
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

/** `crates/series.json`, as written by `cargo run -p figures series`. */
export interface SeriesManifest {
  contract: string;
  series: ManifestSeries[];
  scatters: ManifestScatter[];
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
  /** The manifest exports a series or a cloud no node draws. */
  | "uncited-series";

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
       * One `series:` block, two shapes behind it.
       *
       * A node binds what it draws by key and does not say which array the key is in — a binding
       * is "draw this computation under this field", and whether the crate answered with a column
       * or a cloud is the crate's business. The two differ in what has to be bound with them, and
       * that is the whole of the difference below.
       */
      const series = byKey.get(entry.key);
      const cloud = cloudByKey.get(entry.key);
      if (!series && !cloud) {
        at(
          "unknown-key",
          `draws "${entry.key}", which crates/series.json does not carry. Either the key was ` +
            `renamed in crates/figures/src/lib.rs or the manifest is stale.`,
        );
        continue;
      }
      const owner = (series ?? cloud)!.owner;
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

  const exported = [
    ...manifest.series.map((series) => [series.key, "column"] as const),
    ...manifest.scatters.map((cloud) => [cloud.key, "cloud"] as const),
  ];
  for (const [key, what] of exported) {
    if (!drawn.has(key)) {
      found.push({
        node: "crates/series.json",
        key,
        kind: "uncited-series",
        message:
          `exports ${key} and no node draws it. A ${what} computed for nobody is a ${what} ` +
          `nothing checks; bind it or remove it.`,
      });
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
