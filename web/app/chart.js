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
import { escapeHtml } from "./format.js";
/**
 * A horizontal bar chart: magnitude compared across a handful of named categories.
 *
 * Horizontal because the categories are text and vertical bars would need rotated labels, which
 * are harder to read than they are worth.
 */
export function barChart(bars, options = {}) {
    if (bars.length === 0)
        return "";
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
        ${b.direct
            ? `<text class="bar-value" x="${labelWidth + w + 8}" y="${i * rowHeight + rowHeight / 2}"
                     dominant-baseline="middle">${escapeHtml(b.direct)}</text>`
            : ""}
      </g>`;
    })
        .join("");
    return `<svg class="chart" viewBox="0 0 ${width} ${height}" role="img"
    aria-label="Bar chart of ${bars.length} categories">${rows}</svg>`;
}
/**
 * Bin a set of values into `n` equal-width bins spanning their range.
 *
 * Returned rather than drawn so it can be tested without a DOM.
 */
export function bin(values, n) {
    if (values.length === 0 || n < 1)
        return [];
    const min = Math.min(...values);
    const max = Math.max(...values);
    if (min === max)
        return [{ from: min, to: min, count: values.length }];
    const width = (max - min) / n;
    const bins = Array.from({ length: n }, (_, i) => ({
        from: min + i * width,
        to: min + (i + 1) * width,
        count: 0,
    }));
    for (const v of values) {
        // The top edge belongs to the last bin, not to a bin past the end.
        const index = Math.min(n - 1, Math.floor((v - min) / width));
        bins[index].count++;
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
export function divergingHistogram(bins, format) {
    if (bins.length === 0)
        return "";
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
        const hover = escapeHtml(`${b.count} district${b.count === 1 ? "" : "s"}: ${format(b.from)} to ${format(b.to)}`);
        return `<g class="bar-row" data-hover="${hover}">
        <rect class="hist ${side}" x="${x}" y="${height - 20 - h}" width="${barWidth}"
              height="${h}" rx="4" ry="4"></rect>
      </g>`;
    })
        .join("");
    const first = bins[0];
    const last = bins[bins.length - 1];
    // Where zero falls, so the diverging midpoint is visible rather than inferred from the hues.
    // Without it a reader cannot tell whether a bar just left of the tall one is a small loss or
    // a small gain, which is the only question the chart is for.
    const span = last.to - first.from;
    const zero = first.from < 0 && last.to > 0 ? ((0 - first.from) / span) * width : null;
    return `<svg class="chart" viewBox="0 0 ${width} ${height}" role="img"
    aria-label="Distribution of per-district change, ${bins.length} bins">
    ${marks}
    <line class="axis" x1="0" y1="${height - 20}" x2="${width}" y2="${height - 20}"></line>
    ${zero == null
        ? ""
        : `<line class="zero" x1="${zero.toFixed(1)}" y1="0" x2="${zero.toFixed(1)}"
                 y2="${height - 20}"></line>
           <text class="axis-label" x="${zero.toFixed(1)}" y="${height - 6}"
                 text-anchor="middle">no change</text>`}
    <text class="axis-label" x="0" y="${height - 6}">${escapeHtml(format(first.from))}</text>
    <text class="axis-label" x="${width}" y="${height - 6}"
          text-anchor="end">${escapeHtml(format(last.to))}</text>
  </svg>`;
}
/**
 * Attach a hover tooltip to every mark carrying `data-hover` inside `root`.
 *
 * Delegated from the container so it survives a re-render, and positioned against the viewport
 * so it is not clipped by a card's overflow.
 */
export function attachHover(root, tip) {
    root.addEventListener("mousemove", (event) => {
        const target = event.target?.closest("[data-hover]");
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
