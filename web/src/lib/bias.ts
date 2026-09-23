/**
 * What the two published projections were *off by*, at every horizon, in both populations.
 *
 * # The other half of the question the coverage curve opens
 *
 * `coverage.ts` draws the band's **width** and shows that it holds what it claims. That is a
 * statement about the spread and none at all about the centre: a band can hold sixty-eight per
 * cent of forecasts while every one of them sits high. This card is the level, and it is here
 * because a reader shown the first check and not the second has been shown the half that passes.
 *
 * # Two quantities, and a correction right for one is wrong for the other
 *
 * The feed publishes a statewide total and six hundred district figures out of the same
 * projections, and they do not carry the same bias. The mean district's is the mean of the log
 * errors, which weights every district alike; the total's is the log of summed points over summed
 * actuals, which weights by size. At thirteen years the mean district runs about +0.073 and the
 * total about +0.057 — so a level correction sized on the statewide number, applied to a district,
 * leaves it high, and one sized on the mean district, applied to the total, pushes it low.
 *
 * That is the whole reason there are two lines rather than one, and the reason nothing here is
 * corrected. `.yidam/decisions/the-bias-published-beside-the-point.yml` is where that is decided
 * and why.
 *
 * # Two panels rather than four lines
 *
 * The population is the second axis: restricting targets to before the school closures drops
 * whole origins rather than reweighting them, so "before the closure" and "across it" are two
 * different sets of forecasts and not one set seen twice. Four series in total — and
 * {@link seriesSpec} draws exactly two, for the reason its `labels` type states.
 *
 * So the pair that must be read *against each other* stays inside a panel, and the pair that is a
 * population change becomes the two panels. That is the right way round: the finding is that the
 * mean district and the total differ, and a chart that put those two on separate panels would
 * make the comparison the card exists for the one the reader has to do from memory. The panels
 * share nothing but their reference, which is zero.
 *
 * # Nothing here is typed, and the turns least of all
 *
 * Every number below is read off `crates/series.json`, which `crates/figures` computes from
 * `project::backtest::bias_profile`. The horizon at which each line crosses zero is found in the
 * drawn points by {@link turningPoint} rather than written down — it is the sentence the card is
 * for, and the one thing on it that a changed panel would silently falsify.
 *
 * The reference is zero, and like the 68.3% next door it is a definition rather than a
 * measurement, so the crate pins it nowhere.
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
import { escapeHtml, logError } from "./format.ts";
import { panelWidth, seriesSpec } from "./plot/spec.ts";
import { renderPanelToString } from "./plot/ssr.ts";
import * as routes from "./routes.ts";
import { yearOf } from "./year.ts";

/** The two curves this card draws, shallowest population first. */
export const CURVE_KEYS = [
  "project/the-bias-before-the-closure",
  "project/the-bias-across-the-closure",
] as const;

/** One drawn line: the three numbers the endpoint rule pins, and the one it does not. */
export interface Drawn {
  label: string;
  first: number;
  last: number;
  worst: { at: number; gap: number };
  /**
   * The first position at which the line is on the other side of zero from where it started, or
   * `null` if it never crosses.
   *
   * Not a pinned figure: it is a position on the index rather than a value, and the endpoint rule
   * pins values. It is the finding all the same, so it is computed from the points.
   */
  turns: number | null;
}

/** One population, as everything a panel needs to say about it. */
export interface Population {
  curve: ManifestCurve;
  /** What the panel is called above the drawing. Short: the curve's own label is the long form. */
  caption: string;
  points: SeriesPoint[];
  deepest: number;
  meanDistrict: Drawn;
  total: Drawn;
}

/** The bias profile, as the card reads it. */
export interface Bias {
  populations: Population[];
  /** Zero, which no crate computes. */
  reference: ManifestReference;
  /** Where the twelve figures are stated, because a page has no `figures:` block of its own. */
  source: { route: string; node: string };
}

/**
 * The first drawn position on the far side of zero from the line's own start.
 *
 * Read off the points rather than interpolated. A crossing between two horizons is reported at the
 * horizon that is over the line, because the index is years ahead and there is no half year in it:
 * "positive from four years on" is a statement a reader can check against the picture, and "at
 * 3.7 years" is one they cannot.
 */
export function turningPoint(line: ManifestLine): number | null {
  const start = line.points[0];
  if (!start || start.y === 0) return null;
  const crossed = line.points.find((point) => Math.sign(point.y) === -Math.sign(start.y));
  return crossed ? crossed.x : null;
}

/** A line as the card reads it. */
function drawn(line: ManifestLine): Drawn {
  const first = line.points[0];
  const last = line.points[line.points.length - 1];
  if (!first || !last) {
    throw new Error(`"${line.label}" has no points, so it has no ends to state.`);
  }
  return {
    label: line.label,
    first: first.y,
    last: last.y,
    worst: { at: line.worst.at, gap: line.worst.gap },
    turns: turningPoint(line),
  };
}

/**
 * Read both bias curves out of the series manifest.
 *
 * Throws where either is missing, on `coverage()`'s rule and for a sharper version of its reason:
 * a forecast card that has dropped its bias panel is a card claiming the projection was checked
 * for its level when nothing checked it, and an empty frame says that more confidently than no
 * frame at all.
 */
export function bias(): Bias {
  const manifest = loadSeriesManifest();
  const captions = ["Before the closure", "Across it"];
  const populations = CURVE_KEYS.map((key, at) => {
    const curve = manifest.curves.find((c) => c.key === key);
    if (!curve) {
      throw new Error(
        `The series manifest carries no ${key}, which is what /method's forecast card draws ` +
          `beside the coverage profile. Regenerate it: cargo run --manifest-path ` +
          `crates/Cargo.toml -p figures series > crates/series.json`,
      );
    }
    const points = pointsOf(curve);
    return {
      curve,
      caption: captions[at]!,
      points,
      deepest: points[points.length - 1]?.at ?? 0,
      meanDistrict: drawn(curve.lines[0]!),
      total: drawn(curve.lines[1]!),
    };
  });

  const reference = populations[0]!.curve.reference;
  if (!reference) {
    throw new Error(
      `${CURVE_KEYS[0]} carries no reference, and these lines are read for nothing but their ` +
        `distance from zero. A bias curve without it is four lines about nothing.`,
    );
  }
  const source = PAGE_SERIES[CURVE_KEYS[0]];
  if (!source) {
    throw new Error(
      `${CURVE_KEYS[0]} is not in PAGE_SERIES, so nothing in the corpus answers for the figures ` +
        `at its ends and this card would be citing itself.`,
    );
  }
  return { populations, reference, source };
}

/** `nine years`, or `one year` — the horizon written the way the prose around it reads. */
function years(at: number): string {
  return at === 1 ? "one year" : `${at} years`;
}

/** The corpus address of the node that states the twelve figures. See `coverage.ts`. */
function nodeHref(b: Bias): string {
  const [className, node] = b.source.node.split("/");
  return routes.wikiNode(className ?? "", node ?? "");
}

/** One population's panel: the caption, the drawing, and nothing else. */
function panel(p: Population, width: number, panelText: string): string {
  const chart = renderPanelToString(
    () =>
      seriesSpec(
        p.points,
        /*
         * Short, because the gutter that holds them is capped at 0.45 of a 311px panel and
         * `mean district +0.0272` is painted 135px wide on the runner against the 132 it has.
         * These two words are the card's own framing of the pair — "a statewide total, and six
         * hundred district figures" — and every place with room for the longer form says the
         * longer form: the hover, the alternative text, and the paragraph under the panels.
         */
        { a: "district", b: "statewide" },
        logError,
        (point) =>
          `${years(point.at)} ahead: the mean district ran ${logError(point.a ?? 0)} in logs, ` +
          `the state total ${logError(point.b ?? 0)}`,
        {
          width,
          tick: (at) => years(at),
          reference: { value: 0, label: "No bias" },
        },
      ),
    {
      label: `${p.curve.label}${panelText}. Two lines: the mean of the districts' log errors, ` +
        `and the log of summed forecasts over summed actuals. The dashed rule is zero, which is ` +
        `a forecast right on average`,
      description:
        `The mean district runs ${logError(p.meanDistrict.first)} at one year and ` +
        `${logError(p.meanDistrict.last)} at ${p.deepest}; the state total ` +
        `${logError(p.total.first)} and ${logError(p.total.last)}. Both start under zero and end ` +
        `over it`,
    },
    width,
  );
  return `<div class="panel"><p class="panel-label">${escapeHtml(p.caption)}</p>
      <div class="chartwrap" data-chart="projection-bias">${chart}</div></div>`;
}

/**
 * The two panels and the sentence they are drawn for.
 *
 * Server-rendered like the coverage chart above it, and placed after that card's own footnote
 * rather than inside the band's argument: this is not a correction to the band, and #431 is
 * explicit that it must not read as one.
 */
export function renderBias(b: Bias): string {
  /* The year every one of these figures is measured in, read off the feed. See `coverage.ts`. */
  const dated = yearOf("history");
  const panelText = dated ? `, ${dated}` : "";
  const width = panelWidth(b.populations.length);
  const [before, across] = b.populations as [Population, Population];

  /*
   * The turn is the finding, so the sentence is built around whether there is one to state. Both
   * lines cross zero on both populations today; the wording below degrades to the ends alone if a
   * future panel ever produces a line that does not, rather than printing "at null years".
   */
  const turns =
    before.meanDistrict.turns != null && before.total.turns != null
      ? `Before the closure the district line crosses at
        ${years(before.meanDistrict.turns)} and the total's not until
        ${years(before.total.turns)}, so there is a horizon where the average district is already
        being over-forecast and the statewide figure is still low. `
      : "";

  return `
    <p class="note"><strong>And the band's level is a second question, with two answers.</strong>
      The width above holds; that is a fact about the spread and none at all about the centre. The
      centre is drawn below, and it is two lines rather than one because this site publishes two
      quantities out of the same projections — a statewide total, and six hundred district figures
      — and they do not carry the same bias.</p>

    <div class="panels">${panel(before, width, panelText)}${panel(across, width, panelText)}</div>

    <p class="note">Both quantities start <em>under</em> the truth and end over it. Across the
      closure the mean district runs ${logError(across.meanDistrict.first)} in logs at one year and
      ${logError(across.meanDistrict.last)} at ${across.deepest}, while the state total runs
      ${logError(across.total.first)} and ${logError(across.total.last)} — the same forecasts,
      summed by size rather than averaged over districts, and about three quarters of the drift.
      ${turns}A level correction sized on either one is the wrong size for the other, which is why
      neither is corrected here and both are published instead: the feed carries all four numbers
      at every horizon on the projection's <code>bias</code> table, and a consumer correcting a
      figure can take the one that belongs to it.</p>

    <p class="note">The <em>${escapeHtml(before.caption)}</em> panel stops at
      ${years(before.deepest)} because ${dated ? `the F-33 panel (${dated})` : "the F-33 panel"}
      cannot answer past it without a school closure inside the forecast, and an absence is the
      honest answer rather than the deeper figure wearing the shallower label. Neither line on either
      panel turns back: each one's furthest distance from zero is its last point — ${across.meanDistrict.worst.gap.toFixed(
        4,
      )} and ${across.total.worst.gap.toFixed(4)} at ${years(across.total.worst.at)} across the
      closure — so the drift is not a middle-distance artefact that washes out, and the deepest
      horizon is where a correction would matter most and is trusted least. All twelve numbers are
      pinned in <code>crates/figures.json</code> and stated by
      <a href="${escapeHtml(nodeHref(b))}">the corpus node for the projection</a>.</p>`;
}
