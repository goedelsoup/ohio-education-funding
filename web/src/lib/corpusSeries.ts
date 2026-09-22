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
 * # Sign
 *
 * Rows are signed, because a bar below the zero rule is what a chart is for. The figure manifest
 * exports a correlation as a magnitude with the direction in its key, so {@link rowsAgainstFigures}
 * compares a row's magnitude to the figure it names.
 */

import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import type { Bar } from "./chart.ts";
import type { Node } from "./corpus.ts";
import { citedCrates, proseFields, type Manifest, type Unit } from "./corpusFigures.ts";
import { count, money, pct } from "./format.ts";

/**
 * The series manifest contract this module reads. Versioned apart from the figure manifest's:
 * the two documents have different shapes and different readers, and a field added to a row is no
 * reason for the scalar check to refuse a document it still reads correctly.
 */
export const READS_SERIES_CONTRACT = "1.0.0";

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

/** `crates/series.json`, as written by `cargo run -p figures series`. */
export interface SeriesManifest {
  contract: string;
  series: ManifestSeries[];
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
  const magnitude = Math.abs(value);
  switch (unit) {
    case "count":
      return `${sign}${count(magnitude)}`;
    case "dollars":
      return `${sign}${money(magnitude)}`;
    case "share":
      return `${sign}${pct(magnitude)}`;
    case "pupils":
      return `${sign}${magnitude.toFixed(1)}`;
    case "ratio":
      return `${sign}${magnitude.toFixed(4)}`;
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
  /** The manifest exports a series no node draws. */
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

      const series = byKey.get(entry.key);
      if (!series) {
        at(
          "unknown-key",
          `draws "${entry.key}", which crates/series.json does not carry. Either the key was ` +
            `renamed in crates/figures/src/lib.rs or the manifest is stale.`,
        );
        continue;
      }
      drawn.add(series.key);

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

      if (!cited.has(series.owner)) {
        at(
          "unattributed",
          `draws a ${series.owner} column and cites ${series.owner} in no claim tag. A chart ` +
            `from a crate the node never names is a chart nobody reading the page could trace.`,
        );
      }

      for (const end of endpoints(series)) {
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

  for (const series of manifest.series) {
    if (!drawn.has(series.key)) {
      found.push({
        node: "crates/series.json",
        key: series.key,
        kind: "uncited-series",
        message:
          `exports ${series.key} and no node draws it. A column computed for nobody is a ` +
          `column nothing checks; bind it or remove it.`,
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
