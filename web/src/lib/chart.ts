/**
 * The two chart forms this platform needs, built as plain SVG.
 *
 * Both follow the same rules: thin marks, 4px rounded data-ends anchored to the baseline, a 2px
 * surface gap between adjacent fills so they read as separate quantities, recessive axes, and a
 * hover layer. Colors come from CSS custom properties rather than being written in here, so the
 * dark-mode steps — which are chosen, not flipped — apply without a second code path.
 *
 * Neither form is a dual-axis chart, and there is no code here that could make one.
 */

import { escapeHtml } from "./format.ts";

/** One bar. */
export interface Bar {
  label: string;
  value: number;
  /** Text shown on hover. Falls back to the label and value. */
  hover?: string;
  /** Printed at the end of the bar. Use sparingly — not on every mark. */
  direct?: string;
}

/**
 * A horizontal bar chart: magnitude compared across a handful of named categories.
 *
 * Horizontal because the categories are text and vertical bars would need rotated labels, which
 * are harder to read than they are worth.
 */
export function barChart(bars: Bar[], options: { max?: number } = {}): string {
  if (bars.length === 0) return "";
  const max = options.max ?? Math.max(...bars.map((b) => Math.abs(b.value)), 1);
  const rowHeight = 30;
  const barHeight = 14;
  const labelWidth = 160;
  const width = 640;
  const height = bars.length * rowHeight;

  const rows = bars
    .map((b, i) => {
      const y = i * rowHeight + (rowHeight - barHeight) / 2;
      const w = Math.max(2, (Math.abs(b.value) / max) * (width - labelWidth - 60));
      const hover = escapeHtml(b.hover ?? `${b.label}: ${b.value}`);
      return `
      <g class="bar-row" data-hover="${hover}">
        <text class="bar-label" x="${labelWidth - 10}" y="${i * rowHeight + rowHeight / 2}"
              text-anchor="end" dominant-baseline="middle">${escapeHtml(b.label)}</text>
        <rect class="bar-fill" x="${labelWidth}" y="${y}" width="${w}" height="${barHeight}"
              rx="4" ry="4"></rect>
        ${
          b.direct
            ? `<text class="bar-value" x="${labelWidth + w + 8}" y="${i * rowHeight + rowHeight / 2}"
                     dominant-baseline="middle">${escapeHtml(b.direct)}</text>`
            : ""
        }
      </g>`;
    })
    .join("");

  return `<svg class="chart" viewBox="0 0 ${width} ${height}" role="img"
    aria-label="Bar chart of ${bars.length} categories">${rows}</svg>`;
}

/** One bin of a distribution. */
export interface Bin {
  /** Inclusive lower edge, in the value's own units. */
  from: number;
  /** Exclusive upper edge. */
  to: number;
  count: number;
}

/**
 * Bin a set of values into `n` equal-width bins spanning their range.
 *
 * Returned rather than drawn so it can be tested without a DOM.
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
 * A histogram of a signed quantity, coloured by which side of zero it falls on.
 *
 * Diverging, so: two hues and a neutral midpoint, never a hue at zero. A bin straddling zero is
 * drawn neutral rather than assigned to a side, because assigning it would state a polarity the
 * data does not have.
 */
export function divergingHistogram(
  bins: Bin[],
  format: (v: number) => string,
): string {
  if (bins.length === 0) return "";
  const width = 640;
  const height = 150;
  const gap = 2;
  const barWidth = width / bins.length - gap;
  const maxCount = Math.max(...bins.map((b) => b.count), 1);

  const marks = bins
    .map((b, i) => {
      const h = b.count === 0 ? 0 : Math.max(2, (b.count / maxCount) * (height - 26));
      const x = i * (barWidth + gap);
      const side = b.to <= 0 ? "loss" : b.from >= 0 ? "gain" : "neutral";
      const hover = escapeHtml(
        `${b.count} district${b.count === 1 ? "" : "s"}: ${format(b.from)} to ${format(b.to)}`,
      );
      return `<g class="bar-row" data-hover="${hover}">
        <rect class="hist ${side}" x="${x}" y="${height - 20 - h}" width="${barWidth}"
              height="${h}" rx="4" ry="4"></rect>
      </g>`;
    })
    .join("");

  const first = bins[0]!;
  const last = bins[bins.length - 1]!;

  // Where zero falls, so the diverging midpoint is visible rather than inferred from the hues.
  // Without it a reader cannot tell whether a bar just left of the tall one is a small loss or
  // a small gain, which is the only question the chart is for.
  const span = last.to - first.from;
  const zero =
    first.from < 0 && last.to > 0 ? ((0 - first.from) / span) * width : null;

  return `<svg class="chart" viewBox="0 0 ${width} ${height}" role="img"
    aria-label="Distribution of per-district change, ${bins.length} bins">
    ${marks}
    <line class="axis" x1="0" y1="${height - 20}" x2="${width}" y2="${height - 20}"></line>
    ${
      zero == null
        ? ""
        : `<line class="zero" x1="${zero.toFixed(1)}" y1="0" x2="${zero.toFixed(1)}"
                 y2="${height - 20}"></line>
           <text class="axis-label" x="${zero.toFixed(1)}" y="${height - 6}"
                 text-anchor="middle">no change</text>`
    }
    <text class="axis-label" x="0" y="${height - 6}">${escapeHtml(format(first.from))}</text>
    <text class="axis-label" x="${width}" y="${height - 6}"
          text-anchor="end">${escapeHtml(format(last.to))}</text>
  </svg>`;
}

/** One year of a fan chart. */
export interface FanPoint {
  year: number;
  point: number;
  low: number;
  high: number;
  /** An observed year: the band has no width and the line is drawn solid. */
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
 * A fan chart: a quantity over time whose **interval is the subject**.
 *
 * Every other chart on this page draws a point. This one draws a range, and the drawing rules
 * follow from that rather than from convention:
 *
 * - The band is a filled area with a 2px stroke on each edge, so it reads as a mark rather than
 *   as shading behind one.
 * - The central estimate is **dashed**. A solid centre line reads as the answer with error bars
 *   around it; a dashed one reads as one path through a band, which is what it is.
 * - Both bounds are direct-labelled at the terminal year. The point is not, because the whole
 *   claim of this figure is that the two ends are the finding.
 * - The y axis is truncated to the band's own range — a band a tenth of a percent wide would be
 *   invisible against a zero baseline — so both y bounds are labelled to say so outright.
 *
 * # The seam
 *
 * Observed years are drawn **solid and outside the band**, and the band opens from the last of
 * them. Starting a chart at the forecast leaves the reader nothing to judge the trend against
 * and asks them to take the first value on faith; drawing the observed run-up inside a
 * zero-width band would instead imply it was estimated. So the two halves are separate marks —
 * measurement to the left of the seam, forecast to the right, and a ringed dot on the seam
 * itself — and the difference is visible without consulting the legend.
 *
 * Not a dual-axis chart, and there is nothing here that could make one.
 */
export function fanChart(
  points: FanPoint[],
  format: (v: number) => string,
  hover: (p: FanPoint) => string,
): string {
  if (points.length < 2) return "";
  const width = 640;
  const height = 220;
  const padTop = 14;
  const padBottom = 26;
  const padRight = 104;
  const plot = width - padRight;

  const references = points
    .map((p) => p.reference)
    .filter((v): v is number => v != null);
  const lows = points.map((p) => p.low).concat(references);
  const highs = points.map((p) => p.high).concat(references);
  let min = Math.min(...lows);
  let max = Math.max(...highs);
  const pad = (max - min) * 0.12 || Math.abs(max) * 0.02 || 1;
  min -= pad;
  max += pad;

  const x = (i: number) => (i / (points.length - 1)) * plot;
  const y = (v: number) =>
    padTop + (1 - (v - min) / (max - min)) * (height - padTop - padBottom);
  const at = (i: number, v: number) => `${x(i).toFixed(1)},${y(v).toFixed(1)}`;

  // The seam: the last year with a published enrollment behind it. Everything to its left is
  // measurement, everything to its right is forecast, and the band starts here rather than at
  // the origin so it visibly emerges from the last thing that was actually observed.
  const lastObserved = points.reduce((found, p, i) => (p.observed ? i : found), -1);
  const seam = Math.max(0, lastObserved);
  const observed = points.slice(0, seam + 1);
  const projected = points.slice(seam);

  const seg = (slice: FanPoint[], offset: number, pick: (p: FanPoint) => number) =>
    slice.map((p, i) => at(offset + i, pick(p))).join(" ");

  const upper = projected.map((p, i) => at(seam + i, p.high));
  const lower = projected.map((p, i) => at(seam + i, p.low)).reverse();
  const band = `M ${upper.join(" L ")} L ${lower.join(" L ")} Z`;

  const last = points[points.length - 1]!;
  // A band that never opens. Not a degenerate chart to hide — for a district the guarantee pays,
  // aid does not move with enrollment at all, and the flat line is the finding.
  const degenerate = last.high - last.low < Math.max(1e-9, Math.abs(last.point) * 1e-6);
  const columns = points
    .map((p, i) => {
      const w = plot / (points.length - 1);
      return `<rect class="fan-hit" x="${(x(i) - w / 2).toFixed(1)}" y="0"
        width="${w.toFixed(1)}" height="${height - padBottom}"
        data-hover="${escapeHtml(hover(p))}"></rect>`;
    })
    .join("");

  return `<svg class="chart" viewBox="0 0 ${width} ${height}" role="img"
    aria-label="Observed from ${points[0]!.year} to ${points[seam]!.year}, then projected to
      ${last.year} in a range of ${format(last.low)} to ${format(last.high)}">
    ${projected.length > 1 ? `<path class="fan-band" d="${band}"></path>` : ""}
    ${
      projected.length > 1
        ? `<polyline class="fan-edge" points="${seg(projected, seam, (p) => p.high)}"></polyline>
           <polyline class="fan-edge" points="${seg(projected, seam, (p) => p.low)}"></polyline>
           <polyline class="fan-mid" points="${seg(projected, seam, (p) => p.point)}"></polyline>`
        : ""
    }
    ${
      // Solid, and outside the band: these years were measured, not estimated. Drawing them
      // inside a zero-width band would say the opposite.
      observed.length > 1
        ? `<polyline class="fan-observed" points="${seg(observed, 0, (p) => p.point)}"></polyline>`
        : ""
    }
    ${
      references.length === points.length
        ? `<polyline class="fan-reference" points="${seg(points, 0, (p) => p.reference ?? 0)}"></polyline>
           <text class="fan-bound reference" x="${(plot + 8).toFixed(1)}"
                 y="${y(last.reference ?? 0).toFixed(1)}"
                 dominant-baseline="middle">${escapeHtml(format(last.reference ?? 0))}</text>`
        : ""
    }
    ${
      lastObserved >= 0
        ? `<circle class="fan-anchor" cx="${x(seam).toFixed(1)}" cy="${y(points[seam]!.point).toFixed(1)}" r="4"></circle>`
        : ""
    }
    <text class="fan-bound" x="${(plot + 8).toFixed(1)}" y="${y(last.high).toFixed(1)}"
          dominant-baseline="middle">${escapeHtml(format(last.high))}</text>
    ${
      // One label, not two identical ones, when the band has collapsed — which happens for a
      // district whose aid the guarantee makes independent of its enrollment.
      degenerate
        ? ""
        : `<text class="fan-bound" x="${(plot + 8).toFixed(1)}" y="${y(last.low).toFixed(1)}"
             dominant-baseline="middle">${escapeHtml(format(last.low))}</text>`
    }
    <line class="axis" x1="0" y1="${height - padBottom}" x2="${plot}" y2="${height - padBottom}"></line>
    <text class="axis-label" x="0" y="${height - 8}">FY${points[0]!.year}</text>
    <text class="axis-label" x="${plot}" y="${height - 8}" text-anchor="end">FY${last.year}</text>
    <text class="axis-label" x="${(plot / 2).toFixed(1)}" y="${height - 8}"
          text-anchor="middle">axis starts at ${escapeHtml(format(min))}, not zero</text>
    ${columns}
  </svg>`;
}

/**
 * Attach a hover tooltip to every mark carrying `data-hover` inside `root`.
 *
 * Delegated from the container so it survives a re-render, and positioned against the viewport
 * so it is not clipped by a card's overflow.
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
