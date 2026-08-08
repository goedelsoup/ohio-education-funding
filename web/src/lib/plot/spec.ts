/**
 * The three chart forms, as Observable Plot specifications.
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
 */

import * as Plot from "@observablehq/plot";

import { escapeHtml } from "../format.ts";
import type { Bar, Bin, FanPoint } from "../chart.ts";
import { INK, SERIES } from "./tokens.ts";

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

  return {
    options: {
      width: WIDTH,
      height: bars.length * rowHeight,
      marginLeft: 160,
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
