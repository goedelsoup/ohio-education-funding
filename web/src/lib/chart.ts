/**
 * The chart vocabulary: what a chart is made of, and the one piece of it that runs in a browser.
 *
 * The drawing moved to Observable Plot and lives in `plot/`. What is left here is deliberately
 * everything that neither Plot nor a DOM is needed for — the shapes the renderers take as input,
 * the binning that produces one of them, and the hover layer, which is the only part of a chart
 * on this site that is not a static string by the time a reader sees it.
 *
 * That split is what keeps `linkedom` and Plot out of the client bundle on every page except the
 * two scenario routes. This module is imported by both halves and depends on neither.
 */

/** One bar. */
export interface Bar {
  label: string;
  value: number;
  /** Text shown on hover. Falls back to the label and value. */
  hover?: string;
  /** Printed at the end of the bar. Use sparingly — never a number on every mark. */
  direct?: string;
}

/** One bin of a distribution. */
export interface Bin {
  /** Inclusive lower edge, in the value's own units. */
  from: number;
  /** Exclusive upper edge. */
  to: number;
  count: number;
}

/** One year of a fan chart. */
export interface FanPoint {
  year: number;
  point: number;
  low: number;
  high: number;
  /** An observed year: the band has no width and the line is drawn solid, outside it. */
  observed: boolean;
  /**
   * A second quantity in the same units, drawn as a bare line in the contrasting hue.
   *
   * Used where the banded series alone would say nothing. For a district the guarantee pays,
   * realized aid is flat by construction and the formula's own answer is the thing that moves;
   * the vertical distance between the two is what the guarantee costs.
   */
  reference?: number;
}

/**
 * One district in a scatter: two measures of it, and which half of a split it is on.
 *
 * The unit here is a *district* rather than a bin or a year, which is what makes this the first
 * form on the site that shows the population rather than a summary of it. 606 of the 609 carry
 * both members of every pair the relationship cards are about.
 */
export interface ScatterPoint {
  x: number;
  y: number;
  /** Text shown on hover. There is no direct label on a scatter — 606 of them is not a chart. */
  hover: string;
  /**
   * Which half of a two-way split this district is on, if the scatter is split at all.
   *
   * There is one split worth drawing on this site and it is `on_guarantee`, which is near enough
   * balanced — 294 against 312 — that neither half is a rounding error on the other. A third
   * category is not available and would not be drawn if it were: the palette is two hues.
   */
  series?: "formula" | "guarantee";
}

/**
 * A line through a scatter, and what it is a line *of*.
 *
 * Never a fitted model. Every trace on this site is a median of the points in a bin of the x
 * axis — the same computation `povertyQuintiles` and `guaranteeRateByQuintile` already do for the
 * bar charts, drawn as a line instead of as five bars. That distinction is the whole reason the
 * traces are allowed to live in the web layer at all: a regression line is a claim about a model
 * and would belong in `crates/` with a checkpoint behind it, and a median is a description of the
 * points a reader can already see.
 */
export interface Trace {
  /** Printed at the end of the line, because a trace is a series and identity is never hue alone. */
  label: string;
  series: "formula" | "guarantee";
  points: { x: number; y: number }[];
}

/**
 * One value in a distribution, with the text shown when a reader points at it.
 *
 * Carried as a pair rather than as a bare number because the populations these draw are small
 * enough to be pointed at individually — a county has six districts at the median and thirty-one
 * at the largest, and "which dot is mine" is the question the form exists to answer.
 */
export interface DistributionValue {
  value: number;
  hover: string;
}

/** One year of a two-series time series. */
export interface SeriesPoint {
  year: number;
  /** The first series. `null` is a year the source does not publish — see {@link seriesSpec}. */
  a: number | null;
  /** The second, in the same units. There is no third and no second axis. */
  b: number | null;
}

/**
 * Bin a set of values into `n` equal-width bins spanning their range.
 *
 * Returned rather than drawn so it can be tested without a DOM — which is also why it stayed
 * here when the drawing left.
 */
export function bin(values: number[], n: number): Bin[] {
  if (values.length === 0 || n < 1) return [];
  const min = Math.min(...values);
  const max = Math.max(...values);
  if (min === max) return [{ from: min, to: min, count: values.length }];
  const width = (max - min) / n;
  const bins: Bin[] = Array.from({ length: n }, (_, i) => ({
    from: min + i * width,
    to: min + (i + 1) * width,
    count: 0,
  }));
  for (const v of values) {
    // The top edge belongs to the last bin, not to a bin past the end.
    const index = Math.min(n - 1, Math.floor((v - min) / width));
    bins[index]!.count++;
  }
  return bins;
}

/**
 * Attach a hover tooltip to every mark carrying `data-hover` inside `root`.
 *
 * Delegated from the container so it survives a re-render — the scenario charts are replaced on
 * every slider tick — and positioned against the viewport so it is not clipped by a card's
 * overflow.
 */
export function attachHover(root: HTMLElement, tip: HTMLElement): void {
  root.addEventListener("mousemove", (event) => {
    const target = (event.target as Element | null)?.closest("[data-hover]");
    if (!target) {
      tip.hidden = true;
      return;
    }
    tip.textContent = target.getAttribute("data-hover") ?? "";
    tip.hidden = false;
    const pad = 12;
    tip.style.left = `${Math.min(event.clientX + pad, window.innerWidth - tip.offsetWidth - pad)}px`;
    tip.style.top = `${event.clientY + pad}px`;
  });
  root.addEventListener("mouseleave", () => {
    tip.hidden = true;
  });
}
