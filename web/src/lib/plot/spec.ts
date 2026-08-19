/**
 * The seven chart forms, as Observable Plot specifications.
 *
 * # Why the spec is separate from the rendering
 *
 * These charts are drawn in two places. Almost every one is rendered at build time into static
 * SVG — see `ssr.ts` — so the reader's browser downloads no charting library at all. The scenario
 * routes are the exception: their charts change when a slider moves, so they draw in the browser
 * with `client.ts`. Both take the specifications from this module, which means there is exactly
 * one description of what each chart looks like and no chance of the interactive copy drifting
 * away from the static one.
 *
 * Nothing here touches a DOM, so it is testable without one.
 *
 * # The design rules these encode
 *
 * Two series appear anywhere in this platform — formula aid and guarantee, reused as gain and
 * loss — in slots validated for both light and dark surfaces (all six checks pass in both; see
 * `app.css`). They carry direct labels and a legend as well as colour, so identity never rests on
 * hue. Marks are thin, data-ends are rounded 4px and anchored square to the baseline, axes are
 * recessive, and text wears text tokens rather than a series colour. There is no dual-axis chart
 * here and nothing in this file could produce one.
 *
 * `scatterSpec` is the one form that breaks the mark-size rule, and it says why at its own
 * definition: six hundred 8px markers is a blob, and the density is the information.
 */

import * as Plot from "@observablehq/plot";

import { escapeHtml } from "../format.ts";
import type {
  Bar,
  Bin,
  DistributionValue,
  FanPoint,
  Range,
  ScatterPoint,
  SeriesPoint,
  Trace,
} from "../chart.ts";
import { INK, ORDINAL, SERIES } from "./tokens.ts";

/** A chart, and the tooltip text for the marks a reader can point at. */
export interface Spec {
  options: Plot.PlotOptions;
  /** CSS selector for the hoverable marks, and one string per mark in data order. */
  hovers?: { selector: string; text: string[] };
}

/**
 * Options shared by every chart: transparent surface, inherited type, no Plot-supplied colour.
 *
 * Lives here rather than beside either renderer because both use it, and `ssr.ts` imports
 * `linkedom` — a module the browser must never be handed.
 */
export const BASE: Plot.PlotOptions = {
  style: {
    background: "transparent",
    color: INK.primary,
    fontFamily: "inherit",
    fontSize: "12px",
    overflow: "visible",
  },
  className: "plot",
};

/** Every chart is this wide; the container scrolls if the viewport is narrower. */
const WIDTH = 640;

/**
 * A horizontal bar chart: magnitude compared across a handful of named categories.
 *
 * Horizontal because the categories are text and vertical bars would need rotated labels, which
 * are harder to read than they are worth. Direct labels are selective — Plot is given only the
 * bars that asked for one, never a number on every mark.
 */
export function barSpec(bars: Bar[], options: { max?: number } = {}): Spec {
  const rowHeight = 30;
  const max = options.max ?? Math.max(...bars.map((b) => Math.abs(b.value)), 1);
  const labelled = bars.filter((b) => b.direct != null);
  const longest = Math.max(0, ...bars.map((b) => b.direct?.length ?? 0));
  // Sized to the longest category name, as the right gutter is sized to the longest direct label.
  // This was a fixed 160, which silently clipped anything longer — "Building leadership and
  // operation" rendered as "g leadership and operation", which reads as a rendering fault rather
  // than a truncation and is exactly the kind of thing nobody reports.
  const longestLabel = Math.max(0, ...bars.map((b) => b.label.length));

  return {
    options: {
      width: WIDTH,
      // Bounded below so short-label charts keep their existing proportions, and above so a very
      // long name costs the bars width rather than running off the plot.
      height: bars.length * rowHeight,
      marginLeft: Math.max(120, Math.min(260, Math.round(longestLabel * 7.1) + 14)),
      // Room at the right for the longest direct label actually present. Without it the largest
      // bar's value runs off the viewBox and is clipped — and it is the one most worth reading.
      marginRight: longest > 0 ? 16 + longest * 7.2 : 20,
      marginTop: 0,
      marginBottom: 0,
      x: { axis: null, domain: [0, max] },
      y: { axis: null, domain: bars.map((b) => b.label), padding: 0.47 },
      marks: [
        Plot.barX(bars, {
          y: "label",
          x: (b: Bar) => Math.abs(b.value),
          fill: SERIES.formula,
          className: "bar-fill",
          // Rounded at the data end, square at the baseline: the bar grows from the axis and
          // rounding that end would detach it from the thing it is measured against.
          rx2: 4,
        }),
        Plot.text(bars, {
          y: "label",
          frameAnchor: "left",
          dx: -10,
          text: "label",
          textAnchor: "end",
          fill: INK.secondary,
          className: "bar-label",
        }),
        Plot.text(labelled, {
          y: "label",
          x: (b: Bar) => Math.abs(b.value),
          dx: 8,
          text: "direct",
          textAnchor: "start",
          fill: INK.primary,
          className: "bar-value",
        }),
      ],
    },
    hovers: {
      selector: ".bar-fill > *",
      text: bars.map((b) => escapeHtml(b.hover ?? `${b.label}: ${b.value}`)),
    },
  };
}

/**
 * Two measures of every district, one dot each.
 *
 * # Why this form had to exist
 *
 * The site carried ten correlation coefficients and no scatterplot. A coefficient is the least
 * informative summary of a relationship there is — it is one number standing for six hundred
 * pairs, and two very different clouds produce the same one. The cards it appeared on were the
 * ones whose whole subject *is* the relationship: "Does state aid offset property wealth?"
 * answered with two numbers in a table.
 *
 * The near-zero ones are the strongest case rather than the weakest. Spending per need-weighted
 * pupil against attainment is −0.004, and the card spends two paragraphs explaining that dividing
 * a spending figure by a need index and correlating it against a need-driven outcome measures the
 * weighting rather than the spending. A flat cloud says that in one look.
 *
 * # Overplotting, and why the marks break the usual size rule
 *
 * Six hundred marks in 640×420 overlap, and the usual guidance — markers of 8px or more, a 2px
 * surface ring on anything overlapping — is written for a handful of points and produces a solid
 * blob here. So the marks are small and partly transparent, which makes density legible as
 * density: where the cloud is dark, districts are stacked. What does *not* shrink is the hit
 * target, which is a separate transparent mark at a size a reader can actually point at.
 *
 * # What the line is
 *
 * A median per bin of the x axis, and never a fitted model — see {@link Trace}. Drawn over the
 * cloud rather than in place of it: the cloud is the evidence and the line is the summary, and
 * showing the summary alone is how the site got here.
 *
 * # Axes
 *
 * Corner labels rather than Plot's axes, as `seriesSpec` and `fanSpec` do. A scatter needs both
 * scales stated where a line chart can get away with the ends, so both ends of both axes are
 * labelled and the exact pair for any one district is in its tooltip.
 */
export function scatterSpec(
  points: ScatterPoint[],
  axes: {
    x: { label: string; format: (v: number) => string; log?: boolean };
    y: { label: string; format: (v: number) => string; log?: boolean };
  },
  traces: Trace[] = [],
  options: {
    height?: number;
    /**
     * Draw the line y = x, and put both axes on one domain so that it is a diagonal.
     *
     * For the one shape where the two measures are the same quantity arrived at two ways — a rate
     * this repository predicts against the rate a county auditor charged. A point on the line is a
     * district the model reproduces; the vertical distance off it is the residual, in the units
     * the axis is already in.
     *
     * Two things follow from it and both are load-bearing. The **domain is shared**, because with
     * each axis fitted to its own range the line through (min, min) and (max, max) is not y = x at
     * all. And the **plot area is squared**, because a shared domain on a 640×420 frame still
     * places every point correctly and still draws the line at 33° — which reads as a trend the
     * cloud is beating rather than as the equality it is. Squaring costs a fixed height and is the
     * only way the picture means what it says.
     *
     * This is the only reference line the form draws, and it is deliberately not a general
     * "draw a line here" API: an arbitrary line through a cloud is a claim, and the claims on this
     * site are computed in `crates/` with a checkpoint behind them.
     */
    identity?: { label: string };
  } = {},
): Spec | null {
  // Two points are not a cloud. Same rule as the line forms, for the same reason: a scatter of
  // three districts would read as a finding about a population that has not been measured.
  if (points.length < 12) return null;

  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const pad = (lo: number, hi: number) => (hi - lo) * 0.04 || Math.abs(hi) * 0.02 || 1;
  const both = options.identity != null;
  const xMin = both ? Math.min(...xs, ...ys) : Math.min(...xs);
  const xMax = both ? Math.max(...xs, ...ys) : Math.max(...xs);
  const yMin = both ? xMin : Math.min(...ys);
  const yMax = both ? xMax : Math.max(...ys);
  const xPad = pad(xMin, xMax);
  const yPad = pad(yMin, yMax);

  const hue = (p: ScatterPoint) =>
    p.band != null
      ? (ORDINAL[Math.min(ORDINAL.length - 1, Math.max(0, p.band))] as string)
      : p.series === "guarantee"
        ? SERIES.guarantee
        : p.series === "formula"
          ? SERIES.formula
          : SERIES.neutral;
  /*
   * Banded dots carry a third measure and have to be legible one against another, so they are
   * drawn a shade more opaque than the neutral cloud, which only has to be legible against the
   * card. The ramp's end steps sit near 2.2:1 against their surface; this is part of what the
   * legend those cards carry is relieving.
   */
  const banded = points.some((p) => p.band != null);
  const traceHue = (t: Trace) =>
    t.band != null
      ? (ORDINAL[Math.min(ORDINAL.length - 1, Math.max(0, t.band))] as string)
      : t.series === "guarantee"
        ? SERIES.guarantee
        : SERIES.formula;

  /*
   * Shorter where a card draws two of these to be compared with each other — the spending pair on
   * `/outcomes` is a small-multiple and stacking two full-height clouds puts the second below the
   * fold, which is where a comparison goes to die.
   */
  const marginLeft = 62;
  const marginRight = traces.length > 0 ? 24 + Math.max(...traces.map((t) => t.label.length)) * 7.2 : 24;
  const marginTop = 28;
  const marginBottom = 40;

  /*
   * Square where the identity line is drawn, so that y = x is drawn at 45°. Everywhere else the
   * caller's height, or a default that suits a wide cloud.
   */
  const height = options.identity
    ? WIDTH - marginLeft - marginRight + marginTop + marginBottom
    : (options.height ?? 420);
  return {
    options: {
      width: WIDTH,
      height,
      marginLeft,
      marginRight,
      marginTop,
      marginBottom,
      x: {
        axis: null,
        type: axes.x.log ? "log" : "linear",
        domain: axes.x.log ? [xMin, xMax] : [xMin - xPad, xMax + xPad],
      },
      y: {
        axis: null,
        type: axes.y.log ? "log" : "linear",
        domain: axes.y.log ? [yMin, yMax] : [yMin - yPad, yMax + yPad],
      },
      marks: [
        // The frame, drawn as two rules rather than Plot's axes: recessive, and the same two
        // strokes every other chart here bounds itself with.
        Plot.ruleY([axes.y.log ? yMin : yMin - yPad], { stroke: INK.rule, className: "axis" }),
        Plot.ruleX([axes.x.log ? xMin : xMin - xPad], { stroke: INK.rule, className: "axis" }),

        // Under the cloud, because it is what the cloud is being read against rather than a mark
        // in it. Neutral: it asserts no polarity and is not one of the two series.
        ...(options.identity
          ? [
              Plot.line(
                [
                  { x: xMin, y: xMin },
                  { x: xMax, y: xMax },
                ],
                { x: "x", y: "y", stroke: INK.rule, strokeWidth: 1.5, className: "scatter-identity" },
              ),
            ]
          : []),

        Plot.dot(points, {
          x: "x",
          y: "y",
          r: 2.4,
          fill: hue,
          fillOpacity: banded ? 0.62 : 0.45,
          stroke: "none",
          className: "scatter-dot",
        }),

        ...traces.flatMap((trace) => [
          Plot.line(trace.points, {
            x: "x",
            y: "y",
            stroke: traceHue(trace),
            strokeWidth: 2,
            className: "scatter-trace",
          }),
          /*
           * A direct label, except where the bands already have one.
           *
           * A banded chart carries a legend by construction — the ramp's end steps are near 2.2:1
           * against their surface and the legend is the relief that buys — so labelling each trace
           * as well says the same thing twice and says it on top of the cloud. Three of them ran
           * across the densest part of this one. Identity is still not hue alone; it is hue and a
           * legend, which is what the legend is for.
           */
          ...(trace.band != null
            ? []
            : [
                Plot.text([trace.points[trace.points.length - 1]!], {
                  x: "x",
                  y: "y",
                  dx: 8,
                  text: () => trace.label,
                  textAnchor: "start",
                  fill: traceHue(trace),
                  className: "scatter-trace-end",
                }),
              ]),
        ]),

        ...(options.identity
          ? [
              Plot.text([0], {
                x: xMax,
                y: xMax,
                dx: -6,
                dy: 12,
                text: () => options.identity!.label,
                textAnchor: "end",
                fill: INK.muted,
                fontSize: 11,
                className: "scatter-identity-label",
              }),
            ]
          : []),

        // Both ends of both scales. A cloud with no numbers on it is a texture.
        Plot.text([0], {
          frameAnchor: "bottom-left",
          dy: 20,
          text: () => axes.x.format(xMin),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom",
          dy: 20,
          text: () => axes.x.label + (axes.x.log ? " (log scale)" : ""),
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom-right",
          dy: 20,
          text: () => axes.x.format(xMax),
          textAnchor: "end",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "top-left",
          dx: -marginLeft + 4,
          text: () => axes.y.format(yMax),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom-left",
          dx: -marginLeft + 4,
          text: () => axes.y.format(yMin),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "top-left",
          dx: -marginLeft + 4,
          dy: -12,
          text: () => axes.y.label + (axes.y.log ? " (log scale)" : ""),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),

        // The hit layer. Bigger than the mark and invisible, so a reader can point at a district
        // rather than at a 2.4px dot — and drawn last so it is above every other mark.
        Plot.dot(points, {
          x: "x",
          y: "y",
          r: 7,
          fill: "transparent",
          stroke: "none",
          className: "scatter-hit",
        }),
      ],
    },
    hovers: {
      selector: ".scatter-hit > *",
      text: points.map((p) => escapeHtml(p.hover)),
    },
  };
}

/**
 * Many items, each with a low end and a high end, on one measure.
 *
 * # Why the ratio was not enough
 *
 * `/counties` ranked 88 counties by richest ÷ poorest valuation per pupil and printed the ratio.
 * A ratio is one number standing for two and the two are not recoverable from it. Brown and Wood
 * are both 2.1×, and Wood's *poorest* district stands on more valuation per pupil than Brown's
 * richest — the same "internal disparity" over non-overlapping wealth. Ordering the counties by
 * disparity and ordering them by floor agree for 29 of 84, so the page was showing one of two
 * nearly independent rankings and calling it the shape of the state.
 *
 * # Why the axis is logarithmic, which is the whole design
 *
 * On a log axis a bar's **length is its ratio** and its **position is its level**. Sorted by ratio
 * the lengths step down monotonically while the positions scatter freely, so "the same disparity
 * in a different place" is not a sentence a reader has to be told — it is the picture. On a linear
 * axis the same sort produces bars whose lengths have no fixed relationship to the number they are
 * sorted by, which is worse than the table it replaced.
 *
 * # Two shades of one hue, not two hues
 *
 * The ends of a range are the same measure at two points, not two series, so they take one hue in
 * steps — the ordinal ramp's first and last, already validated all-pairs. Two categorical hues
 * would say the low end and the high end are different kinds of thing.
 */
export function rangeSpec(
  rows: Range[],
  axis: { label: string; format: (v: number) => string; log?: boolean },
): Spec | null {
  if (rows.length < 2) return null;

  const values = rows.flatMap((r) => [r.low, r.high]);
  const min = Math.min(...values);
  const max = Math.max(...values);

  const rowHeight = 14;
  const longest = Math.max(...rows.map((r) => r.label.length));
  const marginLeft = Math.max(70, Math.min(150, Math.round(longest * 6.2) + 10));

  return {
    options: {
      width: WIDTH,
      height: rows.length * rowHeight,
      marginLeft,
      marginRight: 16,
      marginTop: 0,
      marginBottom: 22,
      x: {
        axis: null,
        type: axis.log ? "log" : "linear",
        domain: axis.log ? [min, max] : [min - (max - min) * 0.04, max + (max - min) * 0.04],
      },
      y: { axis: null, domain: rows.map((r) => r.label), padding: 0.2 },
      marks: [
        // The span. Thin, and under both ends: it is the distance, and the ends are what it runs
        // between.
        Plot.ruleY(rows, {
          y: "label",
          x1: "low",
          x2: "high",
          stroke: INK.rule,
          strokeWidth: 1,
          className: "range-span",
        }),
        Plot.dot(rows, {
          y: "label",
          x: "low",
          r: 3,
          fill: ORDINAL[0],
          stroke: "none",
          className: "range-low",
        }),
        Plot.dot(rows, {
          y: "label",
          x: "high",
          r: 3,
          fill: ORDINAL[2],
          stroke: "none",
          className: "range-high",
        }),
        Plot.text(rows, {
          y: "label",
          frameAnchor: "left",
          dx: -8,
          text: "label",
          textAnchor: "end",
          fill: INK.secondary,
          fontSize: 10,
          className: "range-label",
        }),
        Plot.text([0], {
          frameAnchor: "bottom-left",
          dy: 16,
          text: () => axis.format(min),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom",
          dy: 16,
          text: () => axis.label + (axis.log ? " (log scale)" : ""),
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom-right",
          dy: 16,
          text: () => axis.format(max),
          textAnchor: "end",
          fill: INK.muted,
          fontSize: 11,
        }),
        // One full-width band per row, above everything: the hit target is the row, not the 3px
        // dot at either end of it.
        Plot.rect(rows, {
          y: "label",
          x1: min,
          x2: max,
          fill: "transparent",
          className: "range-hit",
        }),
      ],
    },
    hovers: {
      selector: ".range-hit > *",
      text: rows.map((r) => escapeHtml(r.hover)),
    },
  };
}

/**
 * One population, along one axis, with the member this page is about marked on it.
 *
 * # What it replaces
 *
 * A marker on a flat neutral bar with the minimum at one end and the maximum at the other. That
 * says where a district sits and nothing about what it sits among, and for these measures the
 * difference is the whole thing: assessed valuation per pupil runs $79k to $1.35M against a median
 * of $248k, so "the 60th percentile" is a dense neighbourhood and "the 95th" is open country, and
 * the flat strip drew them identically. The same defect applied wherever a peer group was reduced
 * to its extremes — a county page named its richest and poorest district and drew neither the
 * fifteen between them nor how they were spread.
 *
 * # Box, dots, or both
 *
 * Members are drawn individually up to a population the strip can hold, and the threshold is here
 * rather than at each call site so that four cards cannot answer it four ways. A county has six
 * districts at the median and a poverty fifth has a hundred and twenty; both fit across 640px and
 * both are worth seeing, and a box plot of six values is five statistics standing in for six
 * numbers the reader wanted. Ohio's 609 do not fit — six hundred marks on a 46px strip is a rule,
 * not a distribution — so above the threshold the box carries the shape and only the outliers are
 * drawn, which is where the individual districts are worth pointing at anyway.
 *
 * The first version of this drew a box with nothing in it for the poverty fifth: 122 districts,
 * none beyond the fences, so "outliers only" meant no marks at all. A form whose default renders
 * an empty frame for an ordinary input has the wrong default.
 *
 * The vertical spread on the dots is deterministic and means nothing — it is index-based rather
 * than random, both because this module is pure and because a jitter that moved between builds
 * would make two renderings of one county disagree. It exists so that ties are countable.
 *
 * # The fences
 *
 * Whiskers reach the last value inside 1.5 IQR of the box and stop there; anything beyond is drawn
 * as its own mark. That is the ordinary convention and it is worth naming because the alternative
 * — whiskers at the extremes — would draw Ohio's one $1.35M district as the end of a continuum it
 * is nowhere near.
 */
/**
 * The largest population whose members are all drawn.
 *
 * 640px across five lanes: a hundred and fifty marks is one per four pixels per lane, which is
 * still countable. Ohio's 609 districts are four times that and become a rule.
 */
const DOTS_UP_TO = 150;

/**
 * The smallest population that gets a box.
 *
 * Below this the quartiles are not a summary of anything. A seat with three school districts drew
 * a box spanning almost the full width with three dots inside it, because the first and third
 * quartiles of three numbers are the first and third numbers — five statistics standing in for
 * three values, presented with all the authority of a distribution. 39 of Ohio's 132 legislative
 * seats and 60 of its 88 counties are under this, and every one of them is better served by the
 * dots alone: the reader wanted the six districts, and six dots on a line is six districts.
 *
 * Exported because the two cards that draw small populations have to say what the reader is
 * looking at, and a sentence promising a box where there is none is worse than no sentence.
 */
export const BOX_FROM = 8;

export function distributionSpec(
  values: DistributionValue[],
  options: {
    /** The one this page is about. Drawn last, above every other mark. */
    marker?: { value: number; label: string } | null;
    /**
     * Draw every value rather than only the outliers.
     *
     * Defaults on up to {@link DOTS_UP_TO}. Pass it explicitly only to override that for a reason
     * the population size does not carry.
     */
    dots?: boolean;
  } = {},
): Spec | null {
  // Two values are a pair, not a distribution. A box drawn over them would put quartiles on a
  // population that has none, which reads as a finding about a spread nobody measured.
  if (values.length < 3) return null;

  const sorted = [...values].sort((a, b) => a.value - b.value);
  const at = (q: number) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))]!.value;
  const q1 = at(0.25);
  const med = at(0.5);
  const q3 = at(0.75);
  const iqr = q3 - q1;
  const lowFence = q1 - 1.5 * iqr;
  const highFence = q3 + 1.5 * iqr;
  const whiskerLow = sorted.find((v) => v.value >= lowFence)?.value ?? sorted[0]!.value;
  const whiskerHigh = [...sorted].reverse().find((v) => v.value <= highFence)?.value ?? sorted[sorted.length - 1]!.value;

  const min = sorted[0]!.value;
  const max = sorted[sorted.length - 1]!.value;
  const span = max - min || Math.abs(max) || 1;
  const pad = span * 0.03;

  const dots = options.dots ?? values.length <= DOTS_UP_TO;
  const box = values.length >= BOX_FROM;
  const height = 46;
  const mid = 0;
  const outliers = dots ? [] : sorted.filter((v) => v.value < whiskerLow || v.value > whiskerHigh);
  // Five lanes, so a run of equal values is countable rather than one mark. Deterministic: this
  // module is pure, and a jitter that moved between builds would redraw one county two ways.
  const lane = (i: number) => ((i % 5) - 2) * 4.2;
  const drawn = dots ? sorted : outliers;

  return {
    options: {
      width: WIDTH,
      height,
      marginLeft: 2,
      marginRight: 2,
      marginTop: 4,
      marginBottom: 4,
      x: { axis: null, domain: [min - pad, max + pad] },
      y: { axis: null, domain: [-19, 19] },
      marks: [
        // The whisker, drawn first and thin: it is the range, not the mass.
        Plot.ruleY([mid], {
          x1: whiskerLow,
          x2: whiskerHigh,
          stroke: INK.rule,
          strokeWidth: 1,
        }),
        ...(box
          ? [
              // The middle half. Filled rather than outlined — a border drawn to separate marks is
              // what the fill and the surface gap are for.
              Plot.rect([{ q1, q3 }], {
                x1: "q1",
                x2: "q3",
                y1: -11,
                y2: 11,
                fill: SERIES.neutral,
                fillOpacity: 0.28,
                rx: 3,
              }),
              Plot.ruleX([med], { y1: -11, y2: 11, stroke: INK.secondary, strokeWidth: 2 }),
            ]
          : []),

        Plot.dot(drawn, {
          x: "value",
          y: (_d: DistributionValue, i: number) => (dots ? lane(i) : 0),
          r: dots ? 2.8 : 2.4,
          fill: SERIES.neutral,
          fillOpacity: dots ? 0.5 : 0.7,
          stroke: "none",
          className: "dist-dot",
        }),

        // The member the page is about: full height, full hue, above everything, and labelled.
        ...(options.marker
          ? [
              Plot.ruleX([options.marker.value], {
                y1: -17,
                y2: 17,
                stroke: SERIES.formula,
                strokeWidth: 2.5,
                className: "dist-marker",
              }),
            ]
          : []),

        // The hit layer, wider than the marks, above them, and only where there are marks to hit.
        Plot.dot(drawn, {
          x: "value",
          y: (_d: DistributionValue, i: number) => (dots ? lane(i) : 0),
          r: 8,
          fill: "transparent",
          stroke: "none",
          className: "dist-hit",
        }),
      ],
    },
    hovers: {
      selector: ".dist-hit > *",
      text: drawn.map((v) => escapeHtml(v.hover)),
    },
  };
}

/**
 * A histogram of a signed quantity, coloured by which side of zero it falls on.
 *
 * Diverging, so: two hues and a neutral midpoint, never a hue at zero. A bin straddling zero is
 * drawn neutral rather than assigned to a side, because assigning it would state a polarity the
 * data does not have. The zero rule is drawn and labelled "no change" — a reader has to be able
 * to see where zero is, not infer it from the hues.
 */
export function histogramSpec(bins: Bin[], format: (v: number) => string): Spec {
  const first = bins[0]!;
  const last = bins[bins.length - 1]!;
  const crossesZero = first.from < 0 && last.to > 0;

  const side = (b: Bin) =>
    b.to <= 0 ? SERIES.guarantee : b.from >= 0 ? SERIES.formula : SERIES.neutral;

  return {
    options: {
      width: WIDTH,
      height: 150,
      marginTop: 4,
      marginBottom: 26,
      marginLeft: 0,
      marginRight: 0,
      x: { axis: null, domain: [first.from, last.to] },
      y: { axis: null, domain: [0, Math.max(...bins.map((b) => b.count), 1)] },
      marks: [
        Plot.rectY(bins, {
          x1: "from",
          x2: "to",
          y: "count",
          fill: side,
          // A 2px surface gap between adjacent fills, so they read as separate quantities.
          insetLeft: 1,
          insetRight: 1,
          rx2: 4,
          className: "hist",
        }),
        Plot.ruleY([0], { stroke: INK.rule }),
        ...(crossesZero
          ? [
              Plot.ruleX([0], { stroke: INK.muted, strokeDasharray: "3 3" }),
              Plot.text([0], {
                x: 0,
                frameAnchor: "bottom",
                dy: 18,
                text: () => "no change",
                fill: INK.muted,
                fontSize: 11,
              }),
            ]
          : []),
        Plot.text([first.from], {
          frameAnchor: "bottom-left",
          dy: 18,
          text: (v: number) => format(v),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([last.to], {
          frameAnchor: "bottom-right",
          dy: 18,
          text: (v: number) => format(v),
          textAnchor: "end",
          fill: INK.muted,
          fontSize: 11,
        }),
      ],
    },
    hovers: {
      selector: ".hist > *",
      text: bins.map((b) =>
        escapeHtml(
          `${b.count} district${b.count === 1 ? "" : "s"}: ${format(b.from)} to ${format(b.to)}`,
        ),
      ),
    },
  };
}

/**
 * A fan chart: a quantity over time whose **interval is the subject**.
 *
 * Every other chart here draws a point. This one draws a range, and the rules follow from that
 * rather than from convention:
 *
 * - The band is a filled area with a 2px stroke on each edge, so it reads as a mark rather than
 *   as shading behind one.
 * - The central estimate is **dashed**. A solid centre line reads as the answer with error bars
 *   around it; a dashed one reads as one path through a band, which is what it is.
 * - Both bounds are direct-labelled at the terminal year, and the point is not — the whole claim
 *   of the figure is that the two ends are the finding.
 * - The y axis is truncated to the band's own range, because a band a tenth of a percent wide is
 *   invisible against a zero baseline, and it says so on its face rather than in a caption.
 *
 * # The seam
 *
 * Observed years are drawn solid and **outside** the band, which opens from the last of them.
 * Starting at the forecast leaves a reader nothing to judge the trend against; drawing the
 * observed run-up inside a zero-width band would instead claim it was estimated. So the two
 * halves are separate marks with a ringed dot on the seam, and the difference is visible without
 * consulting the legend.
 */
export function fanSpec(
  points: FanPoint[],
  format: (v: number) => string,
  hover: (p: FanPoint) => string,
): Spec | null {
  // One point is not a series. Returning null draws nothing rather than a degenerate axis with a
  // single mark on it, which would read as a finding about a quantity that has not been measured
  // twice.
  if (points.length < 2) return null;
  const references = points.map((p) => p.reference).filter((v): v is number => v != null);
  const hasReference = references.length === points.length;

  let min = Math.min(...points.map((p) => p.low), ...references);
  let max = Math.max(...points.map((p) => p.high), ...references);
  const pad = (max - min) * 0.12 || Math.abs(max) * 0.02 || 1;
  min -= pad;
  max += pad;

  const lastObserved = points.reduce((found, p, i) => (p.observed ? i : found), -1);
  const seam = Math.max(0, lastObserved);
  const observed = points.slice(0, seam + 1);
  const projected = points.slice(seam);
  const last = points[points.length - 1]!;
  // A band that never opens: for a district the guarantee pays, aid does not move with enrollment
  // at all. Not a degenerate chart to hide — the flat line is the finding — but two identical
  // bound labels stacked on each other would be noise.
  const degenerate = last.high - last.low < Math.max(1e-9, Math.abs(last.point) * 1e-6);

  const bounds = degenerate ? [last.high] : [last.high, last.low];

  return {
    options: {
      width: WIDTH,
      height: 220,
      marginTop: 14,
      marginBottom: 26,
      marginLeft: 0,
      marginRight: 104,
      x: { axis: null, domain: [points[0]!.year, last.year] },
      y: { axis: null, domain: [min, max] },
      marks: [
        ...(projected.length > 1
          ? [
              Plot.areaY(projected, {
                x: "year",
                y1: "low",
                y2: "high",
                fill: SERIES.formula,
                fillOpacity: 0.16,
                className: "fan-band",
              }),
              Plot.line(projected, {
                x: "year",
                y: "high",
                stroke: SERIES.formula,
                strokeWidth: 2,
                className: "fan-edge",
              }),
              Plot.line(projected, {
                x: "year",
                y: "low",
                stroke: SERIES.formula,
                strokeWidth: 2,
                className: "fan-edge",
              }),
              Plot.line(projected, {
                x: "year",
                y: "point",
                stroke: SERIES.formula,
                strokeWidth: 2,
                strokeDasharray: "5 4",
                className: "fan-mid",
              }),
            ]
          : []),
        // Measurement, and outside the band. Drawing these inside a zero-width band would say
        // they were estimated.
        ...(observed.length > 1
          ? [
              Plot.line(observed, {
                x: "year",
                y: "point",
                stroke: SERIES.formula,
                strokeWidth: 2,
                className: "fan-observed",
              }),
            ]
          : []),
        ...(hasReference
          ? [
              Plot.line(points, {
                x: "year",
                y: "reference",
                stroke: SERIES.guarantee,
                strokeWidth: 2,
                className: "fan-reference",
              }),
              Plot.text([last], {
                x: last.year,
                // Guarded by `hasReference` above, which the type system cannot see through.
                y: last.reference ?? 0,
                dx: 8,
                text: () => format(last.reference ?? 0),
                textAnchor: "start",
                fill: SERIES.guarantee,
                className: "fan-bound reference",
              }),
            ]
          : []),
        ...(lastObserved >= 0
          ? [
              Plot.dot([points[seam]!], {
                x: "year",
                y: "point",
                r: 4,
                fill: INK.surface,
                stroke: SERIES.formula,
                strokeWidth: 2,
                className: "fan-anchor",
              }),
            ]
          : []),
        Plot.text(bounds, {
          x: last.year,
          y: (v: number) => v,
          dx: 8,
          text: (v: number) => format(v),
          textAnchor: "start",
          fill: INK.primary,
          className: "fan-bound",
        }),
        Plot.ruleY([min], { stroke: INK.rule, className: "axis" }),
        Plot.text([points[0]!.year], {
          frameAnchor: "bottom-left",
          dy: 18,
          text: (v: number) => `FY${v}`,
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        // The truncated axis, stated on the chart rather than in the caption underneath it. A
        // reader who takes the shape at face value has been misled by the time they reach prose.
        Plot.text([0], {
          frameAnchor: "bottom",
          dy: 18,
          text: () => `axis starts at ${format(min)}, not zero`,
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([last.year], {
          frameAnchor: "bottom-right",
          dy: 18,
          text: (v: number) => `FY${v}`,
          textAnchor: "end",
          fill: INK.muted,
          fontSize: 11,
        }),
        // The hit layer, last so it sits above every mark. One full-height column per year: the
        // target a reader aims at is the strip, not the 2px line inside it.
        Plot.rect(points, {
          x1: (p: FanPoint) => p.year - 0.5,
          x2: (p: FanPoint) => p.year + 0.5,
          y1: min,
          y2: max,
          fill: "transparent",
          className: "fan-hit",
        }),
      ],
    },
    hovers: {
      selector: ".fan-hit > *",
      text: points.map((p) => escapeHtml(hover(p))),
    },
  };
}

/**
 * Two quantities in the same units, over years.
 *
 * The fourth form, and the one the historical view needed: a fan chart draws an interval and a
 * bar chart draws categories, and neither says what a pair of series did across fourteen years.
 *
 * The rules it inherits, and the two it adds:
 *
 * - **Two series, and there cannot be a third.** The categorical palette is a validated pair and
 *   nothing here generates beyond it. A page needing a third quantity puts it in the table
 *   underneath, which is what the table is for.
 * - **One axis.** Both series are in the same units by construction — the caller passes shares or
 *   dollars, never one of each — so there is no second scale to mislead with.
 * - **A missing year is a break, not a bridge.** FY2014 is absent from the Census archive, and a
 *   line drawn straight through it would assert a measurement nobody made. Plot breaks a line at
 *   a null, and passing `null` rather than omitting the year is what keeps the gap on the x axis
 *   where a reader can see it.
 * - The axis is truncated to the data's own range and says so, as the fan chart does, because a
 *   share moving from 46% to 34% is invisible against a zero baseline.
 */
export function seriesSpec(
  points: SeriesPoint[],
  labels: { a: string; b: string },
  format: (v: number) => string,
  hover: (p: SeriesPoint) => string,
): Spec | null {
  // One point is not a series, exactly as in `fanSpec`.
  if (points.length < 2) return null;
  const values = points.flatMap((p) => [p.a, p.b]).filter((v): v is number => v != null);
  if (values.length === 0) return null;

  let min = Math.min(...values);
  let max = Math.max(...values);
  const pad = (max - min) * 0.12 || Math.abs(max) * 0.02 || 1;
  min -= pad;
  max += pad;

  const first = points[0]!;
  const last = points[points.length - 1]!;
  // The direct label goes on the last year that has a value, which is not necessarily the last
  // year: a series ending in a gap would otherwise be labelled at a point it does not occupy.
  const endOf = (key: "a" | "b") => [...points].reverse().find((p) => p[key] != null);
  const endA = endOf("a");
  const endB = endOf("b");

  const line = (key: "a" | "b", stroke: string, className: string) =>
    Plot.line(points, {
      x: "year",
      y: key,
      stroke,
      strokeWidth: 2,
      // Plot breaks a line at a non-finite y of its own accord, which is why the missing year
      // reaches here as a point with a null value rather than as an absent point.
      className,
    });

  const endLabel = (point: SeriesPoint | undefined, key: "a" | "b", stroke: string) =>
    point
      ? [
          Plot.text([point], {
            x: point.year,
            y: point[key] ?? 0,
            dx: 8,
            text: () => `${key === "a" ? labels.a : labels.b} ${format(point[key] ?? 0)}`,
            textAnchor: "start",
            fill: stroke,
            className: "series-end",
          }),
        ]
      : [];

  return {
    options: {
      width: WIDTH,
      height: 220,
      marginTop: 14,
      marginBottom: 26,
      marginLeft: 0,
      // Room for the longer of the two direct labels, which carry the series identity.
      marginRight: 32 + Math.max(labels.a.length, labels.b.length) * 7.2,
      x: { axis: null, domain: [first.year, last.year] },
      y: { axis: null, domain: [min, max] },
      marks: [
        line("a", SERIES.formula, "series-a"),
        line("b", SERIES.guarantee, "series-b"),
        ...endLabel(endA, "a", SERIES.formula),
        ...endLabel(endB, "b", SERIES.guarantee),
        Plot.ruleY([min], { stroke: INK.rule, className: "axis" }),
        Plot.text([first.year], {
          frameAnchor: "bottom-left",
          dy: 18,
          text: (v: number) => `FY${v}`,
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom",
          dy: 18,
          text: () => `axis starts at ${format(min)}, not zero`,
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([last.year], {
          frameAnchor: "bottom-right",
          dy: 18,
          text: (v: number) => `FY${v}`,
          textAnchor: "end",
          fill: INK.muted,
          fontSize: 11,
        }),
        // One full-height column per year, above every mark, as the fan chart does.
        Plot.rect(points, {
          x1: (p: SeriesPoint) => p.year - 0.5,
          x2: (p: SeriesPoint) => p.year + 0.5,
          y1: min,
          y2: max,
          fill: "transparent",
          className: "series-hit",
        }),
      ],
    },
    hovers: {
      selector: ".series-hit > *",
      text: points.map((p) => escapeHtml(hover(p))),
    },
  };
}
