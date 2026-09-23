/**
 * What the plus-or-minus one sigma band actually held, at every horizon the backtest reaches.
 *
 * # Why the card needed a picture
 *
 * `/method`'s forecast card argues two measured things in prose. The first is this one: the band
 * is not a district's own variability, it is a cross-sectional spread widened by the horizon, and
 * the question a reader is entitled to ask of it is whether it holds what it claims. The answer
 * is two lines over thirteen horizons — pooled across origins, and with each origin's mean removed
 * — and the whole of the finding is a *shape*: the second line is flat, within three points of
 * the target at every horizon, and the first sags in the middle distance. Flatness is the one
 * thing a thirteen-column table is worst at showing, and there was no table here anyway.
 *
 * # Nothing here is typed
 *
 * Every number on the card below is read off `crates/series.json`'s
 * `project/what-the-band-held-at-every-horizon`, which `crates/figures` computes from
 * `project::backtest::profile`. The deepest horizon is the length of a line rather than a
 * constant, the two ends are the ends of the drawn points, and the worst departure is the crate's
 * own maximum. A panel that grows another origin moves all of them without an edit here.
 *
 * The one number that is *not* read off anything is the reference: 68.3% is the share of a normal
 * distribution inside one standard deviation, which is a definition rather than a measurement.
 * The crate carries it as the curve's reference and pins it nowhere, for the reason
 * `ManifestReference` gives.
 *
 * # Where the numbers are answered for
 *
 * A page has no `figures:` block, so `PAGE_SERIES` names the corpus node that states the six
 * figures at the lines' ends and worst departures — the same redirection `/bounds` uses — and the
 * card links to it. Six, rather than the two an ordinary column owes, because a line over an
 * index is read where it starts, where it ends, and at its worst distance from the reference.
 */

import type { SeriesPoint } from "./chart.ts";
import {
  loadSeriesManifest,
  pointsOf,
  PAGE_SERIES,
  type ManifestCurve,
  type ManifestLine,
  type ManifestReference,
} from "./corpusSeries.ts";
import { escapeHtml, pct } from "./format.ts";
import { seriesSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import * as routes from "./routes.ts";
import { yearOf } from "./year.ts";

/** The one curve this card draws. Named here because three things below look it up. */
export const CURVE_KEY = "project/what-the-band-held-at-every-horizon";

/** One drawn line, and the three numbers a reader takes off it. */
export interface Held {
  label: string;
  first: number;
  last: number;
  worst: { at: number; gap: number };
}

/** The coverage profile, as everything the card needs to say about it. */
export interface Coverage {
  curve: ManifestCurve;
  points: SeriesPoint[];
  /** What a plus-or-minus one sigma band claims to hold, which no crate computes. */
  reference: ManifestReference;
  /** Every error, origins pooled — the first line, and the one that sags. */
  pooled: Held;
  /** The same errors with each origin's mean removed — the flat one, and the claim. */
  crossDistrict: Held;
  /** The deepest horizon drawn, counted off the line rather than written down. */
  deepest: number;
  /** Where the six figures are stated, because they are not stated on this page. */
  source: { route: string; node: string };
}

/** A line as the card reads it: its two ends and its worst departure. */
function held(line: ManifestLine): Held {
  const first = line.points[0];
  const last = line.points[line.points.length - 1];
  if (!first || !last) {
    throw new Error(`${CURVE_KEY}: "${line.label}" has no points, so it has no ends to state.`);
  }
  return {
    label: line.label,
    first: first.y,
    last: last.y,
    worst: { at: line.worst.at, gap: line.worst.gap },
  };
}

/**
 * Read the coverage profile out of the series manifest.
 *
 * Throws where the manifest does not carry it, or carries it without a reference. That is
 * `loadSeriesManifest`'s rule and `/bounds`'s: a card whose subject has gone missing must fail the
 * build rather than render an empty frame, because an empty frame reads as "the band was never
 * checked" — which is the one thing this card exists to deny.
 */
export function coverage(): Coverage {
  const curve = loadSeriesManifest().curves.find((c) => c.key === CURVE_KEY);
  if (!curve) {
    throw new Error(
      `The series manifest carries no ${CURVE_KEY}, which is what /method's forecast card draws. ` +
        `Regenerate it: cargo run --manifest-path crates/Cargo.toml -p figures series > crates/series.json`,
    );
  }
  if (!curve.reference) {
    throw new Error(
      `${CURVE_KEY} carries no reference, and the two lines on it are read for their distance ` +
        `from one. A coverage curve without the coverage it claims is two lines about nothing.`,
    );
  }
  const source = PAGE_SERIES[CURVE_KEY];
  if (!source) {
    throw new Error(
      `${CURVE_KEY} is not in PAGE_SERIES, so nothing in the corpus answers for the figures at ` +
        `its ends and this card would be citing itself.`,
    );
  }
  const points = pointsOf(curve);
  return {
    curve,
    points,
    reference: curve.reference,
    pooled: held(curve.lines[0]!),
    crossDistrict: held(curve.lines[1]!),
    deepest: points[points.length - 1]?.at ?? 0,
    source,
  };
}

/** `nine years`, or `one year` — the horizon written the way the prose around it reads. */
function years(at: number): string {
  return at === 1 ? "one year" : `${at} years`;
}

/** A gap from the reference, in points of coverage rather than as a share of it. */
function points(gap: number): string {
  return `${(gap * 100).toFixed(1)} points`;
}

/**
 * The corpus address of the node that states the six figures.
 *
 * `PAGE_SERIES` holds the id as `class/node`, the form every corpus id takes, so the two halves
 * are split rather than stored twice. `crossCheckPageSeries` has already required the node to
 * exist and to bind all six before this renders.
 */
function nodeHref(c: Coverage): string {
  const [className, node] = c.source.node.split("/");
  return routes.wikiNode(className ?? "", node ?? "");
}

/**
 * The chart, and what the two lines say.
 *
 * Server-rendered like every other static chart here: `check:dist` reads the built HTML, and the
 * six endpoint figures it has to find are inside this SVG and the paragraph under it.
 */
export function renderCoverage(c: Coverage): string {
  /*
   * The one thing on this card the series manifest does not carry, and the reason it is here.
   *
   * A curve's index is a horizon, so nothing in `crates/series.json` names a fiscal year — and
   * every figure on a page has to be able to reach the year it was measured in, which is what
   * `tests/e2e/figures.spec.ts` holds. The measurement is the F-33 panel, which the feed already
   * dates as `history`: same Census survey, same years, read rather than typed. Empty where the
   * feed was built without that fixture, and the sentence closes over the gap rather than
   * printing a comma with nothing between its sides.
   */
  const dated = yearOf("history");
  const panel = dated ? `, ${dated}` : "";

  const chart = renderToString(
    (width) =>
      seriesSpec(
        c.points,
        { a: "pooled", b: "cross-district" },
        (v) => pct(v, 0),
        (p) =>
          `${years(p.at)} ahead: the band held ${pct(p.a, 1)} of forecasts pooled across ` +
          `origins, ${pct(p.b, 1)} with each origin's mean removed`,
        {
          width,
          tick: (at) => years(at),
          reference: { value: c.reference.value, label: c.reference.label },
        },
      ),
    {
      label:
        `The share of district forecasts that fell inside the plus-or-minus one sigma band, at ` +
        `every horizon from one year to ${c.deepest}, over every origin and district the F-33 ` +
        `panel admits${panel}. Two lines: every error with origins pooled, and the same errors ` +
        `with each origin's own mean removed. The dashed rule is the ${pct(c.reference.value, 1)} such a ` +
        `band claims to hold.`,
      description:
        `Pooled, coverage is ${pct(c.pooled.first, 1)} at one year and ` +
        `${pct(c.pooled.last, 1)} at ${c.deepest}, at worst ${points(c.pooled.worst.gap)} under ` +
        `the target at ${years(c.pooled.worst.at)}; with each origin's mean removed it stays ` +
        `within ${points(c.crossDistrict.worst.gap)} of it at every horizon`,
    },
  );

  return `
    <div class="chartwrap" data-chart="band-coverage">${chart}</div>
    <p class="note">The band is checked rather than asserted. Backtested over the F-33
      panel${panel} — every origin and district it admits — it held
      ${pct(c.crossDistrict.first, 1)} of forecasts at one year and
      ${pct(c.crossDistrict.last, 1)} at ${c.deepest} — within
      ${points(c.crossDistrict.worst.gap)} of the ${pct(c.reference.value, 1)} such a band claims,
      at every horizon, which is the flat line and the whole of the claim. The other line is the
      same errors with the origins left in: ${pct(c.pooled.first, 1)} at one year and
      ${pct(c.pooled.last, 1)} at ${c.deepest}, and ${points(c.pooled.worst.gap)} short of the
      target at ${years(c.pooled.worst.at)}, which is its worst and is at neither end — the ends
      are the two numbers that state this line least well, and are the reason it is drawn.
      The gap between the two lines is not the band being wrong about
      districts; it is a <em>year</em> effect — at a fixed horizon every origin runs the same
      method over the same districts, so an origin's own mean is a year's, and the years are the
      school closures. All six numbers are pinned in <code>crates/figures.json</code> and stated by
      <a href="${escapeHtml(nodeHref(c))}">the corpus node for the projection</a>, which is where a
      claim this page draws but does not own is answered for.</p>`;
}
