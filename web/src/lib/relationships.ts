/**
 * The relationships, as points rather than as coefficients.
 *
 * # What this module is for
 *
 * Ten correlation coefficients were rendered on this site as numbers in two-column tables, on
 * cards whose subject was the relationship the coefficient summarises. This turns the pairs behind
 * them into something a reader can look at. Nothing here computes a new figure: it selects two
 * fields per district and takes medians of them, which is the same arithmetic `povertyQuintiles`
 * and `guaranteeRateByQuintile` already do for the bar charts.
 *
 * # Why there is no regression in this file
 *
 * A fitted line is a model, and this repository has one rule about models: they live in `crates/`,
 * they are pure and deterministic, and the build checks them against reference scenarios before
 * the site is allowed to render. The README states it as "one thing here is computed, and it is
 * the base cost". A least-squares line drawn from 606 points in the web layer would be a second
 * computed thing with nothing behind it, and it would sit on the page looking exactly as
 * authoritative as the checked ones.
 *
 * A **median per bin** is not that. It describes the points the reader is already looking at, it
 * asserts no functional form, and it can be checked by eye against the cloud it sits on. Where a
 * fitted line would say more, the coefficient beside the chart is the one the crates computed and
 * it is stated as what it is.
 */

import type { ScatterPoint, Trace } from "./chart.ts";
import { median } from "./stats.ts";
import type { District } from "./types.ts";

/**
 * The districts carrying both measures, as points.
 *
 * A district missing either one is dropped rather than defaulted: three districts have no report
 * card and three no reported poverty share, and a zero for a missing Performance Index would put
 * six dots on the floor of a chart about attainment.
 */
export function pairs(
  districts: District[],
  x: (d: District) => number | null | undefined,
  y: (d: District) => number | null | undefined,
  hover: (d: District, x: number, y: number) => string,
  extra?: {
    series?: (d: District) => "formula" | "guarantee";
    /**
     * Which ordered band the district is in, from {@link bands}.
     *
     * Taken as a function of the district rather than applied to the returned points, because the
     * points have already dropped whoever was missing a measure and recovering the district from a
     * point's index means re-deriving that filter and trusting the two to agree. `attachHovers`
     * exists because this repository does not trust that kind of index alignment even when it is
     * someone else's renderer doing the aligning.
     */
    band?: (d: District) => number | undefined;
  },
): ScatterPoint[] {
  const points: ScatterPoint[] = [];
  for (const d of districts) {
    const xv = x(d);
    const yv = y(d);
    if (xv == null || yv == null || !Number.isFinite(xv) || !Number.isFinite(yv)) continue;
    const band = extra?.band?.(d);
    points.push({
      x: xv,
      y: yv,
      hover: hover(d, xv, yv),
      ...(extra?.series ? { series: extra.series(d) } : {}),
      ...(band == null ? {} : { band }),
    });
  }
  return points;
}

/**
 * The median of each equal-count bin of the x axis, as a line.
 *
 * Equal-count and not equal-width, because every measure here is skewed — assessed valuation per
 * pupil runs from $79k to $1.35M against a median of $248k — and equal-width bins would put two
 * thirds of the districts in the first bin and one district in the last. The existing quintile
 * bar charts bin the same way for the same reason.
 *
 * The x of each point is the bin's own median x rather than its midpoint, so the line is drawn
 * where the districts are rather than where the bin edges happen to fall.
 */
export function medianTrace(
  values: { x: number; y: number }[],
  bins: number,
  label: string,
  series: "formula" | "guarantee",
): Trace {
  const sorted = [...values].sort((a, b) => a.x - b.x);
  const points: { x: number; y: number }[] = [];

  for (let i = 0; i < bins; i += 1) {
    // The last bin takes the remainder, so integer division drops no district — the same rule
    // `povertyQuintiles` and `wealthQuintiles` use.
    const group = sorted.slice(
      Math.floor((i * sorted.length) / bins),
      i === bins - 1 ? sorted.length : Math.floor(((i + 1) * sorted.length) / bins),
    );
    if (group.length === 0) continue;
    points.push({ x: median(group.map((v) => v.x)), y: median(group.map((v) => v.y)) });
  }

  return { label, series, points };
}

/**
 * Which ordered band each district falls in, by a measure that is not on either axis.
 *
 * # Why bands are worth colouring and why three of them
 *
 * A median line says what the middle of the cloud does. Banding says what the cloud is *made of* —
 * and where the banding measure is a third variable, it shows a structure no line can: on
 * `/outcomes` the three poverty bands occupy the same range of spending per need-weighted pupil,
 * p10 within $500 of each other and p90 within $100, while their median Performance Index differs
 * by eighteen points. Three horizontal bands stacked at one x range is the card's whole argument,
 * drawn.
 *
 * Three and not five is a measurement, not a preference: a scatter is an all-pairs form and five
 * steps of one hue close to a normal-vision ΔE of 10.9. `plot/tokens.ts` has the numbers.
 *
 * # Where it is not worth spending
 *
 * Where the banding measure is already an axis. Banding the poverty-against-attainment scatter by
 * poverty repaints the x axis as a left-to-right gradient and adds nothing a reader could not
 * already see, while spending the one channel a third variable could have used.
 */
export function bands(
  districts: District[],
  by: (d: District) => number | null | undefined,
  count = 3,
): Map<District, number> {
  const ranked = districts
    .map((d) => ({ d, v: by(d) }))
    .filter((row): row is { d: District; v: number } => row.v != null && Number.isFinite(row.v))
    .sort((a, b) => a.v - b.v);

  const assigned = new Map<District, number>();
  ranked.forEach((row, i) => {
    // The last band takes the remainder, so integer division drops nobody — the same rule the
    // quintile helpers and `medianTrace` use.
    assigned.set(row.d, Math.min(count - 1, Math.floor((i * count) / ranked.length)));
  });
  return assigned;
}


