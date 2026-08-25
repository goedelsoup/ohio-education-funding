/**
 * What the page looks like, as numbers.
 *
 * # Why this exists
 *
 * The unit suite ran 395 tests before this file and the browser suite runs 303, and not one of the
 * 698 looks at the page. They assert semantics, structure, contrast arithmetic and copy — which is
 * the right thing to assert and is why the palette is the best-argued part of this repository — but
 * a stylesheet can be entirely correct under all of them and still be unreadable. That is how the
 * 2026-08 audit found 96 findings behind a green gate.
 *
 * Those are the runners' counts and not `grep -c 'test('` over the files. Several of these suites
 * generate cases in a loop — the 20-route axe sweep is one `test()` and twenty tests — so the two
 * numbers disagree by 13 in the browser suite alone. This repository has a documented history of
 * comments carrying figures that reproduce under nothing, and a grep count presented as a test
 * count is that same defect with a smaller blast radius.
 *
 * So this measures the reading experience the way `palette.spec.ts` measures colour: from the
 * artefact, in numbers, reproducibly. See #183.
 *
 * # The two kinds of metric, and why the distinction is load-bearing
 *
 * **Deterministic** metrics read computed CSS and are the same on every machine: the census of
 * font sizes, the heading ratio, the count and nesting depth of bordered boxes, the count of
 * right-aligned cells carrying prose. These can become hard thresholds and later phases will make
 * them so.
 *
 * **Font-sensitive** metrics depend on how wide the platform's UI face draws a glyph: the reading
 * measure in `ch`, the height of the wrapping header, and the y-offset of the first content. SF
 * Pro, Segoe UI and DejaVu Sans do not agree, and this repository has already shipped a chart
 * defect that only reproduced under one of them — see the note in `plot/spec.ts`.
 *
 * A font-sensitive figure is therefore recorded WITH the advance width that produced it
 * ({@link Measured.zeroAdvance}) and the family that was resolved ({@link Measured.bodyFont}).
 * Without those a 75ch measured on a laptop and an 85ch measured on a runner look like a
 * regression rather than like two different fonts, and somebody spends an afternoon on it. A
 * threshold over one of these has to state the font it is a threshold under; none does yet,
 * which is the honest state and is why {@link THRESHOLDS} starts empty.
 *
 * # What is deliberately not measured
 *
 * Text inside `<svg>`. A chart's `font-size` attribute is not what gets painted — the SVG is
 * `width: 100%` over a `viewBox`, so the rendered size is `computed × (boxWidth / viewBox.width)`
 * and the e2e suite already checks that separately at 375px. Counting those numbers in the size
 * census would mix two scales and make the census say nothing.
 */

/** One `<p>` long enough that its line length is a reading decision rather than an accident. */
export const PROSE_MIN_CHARS = 140;

/** Above this many words, a table cell holds a sentence rather than a value. */
export const PROSE_CELL_MIN_WORDS = 6;

/** A box counts as a box at this radius or above; below it the corner is a hairline artefact. */
export const BOX_MIN_RADIUS = 4;

/**
 * The routes the report walks.
 *
 * Chosen to cover each genre exactly once rather than to be representative by volume: the reading
 * column, the 609-row wide table, the corpus prose genre, the decision-record genre, and the
 * chart-bearing card. 3,492 pages collapse into these five shapes, and a sixth route of a shape
 * already here would cost a browser page load and tell nobody anything.
 *
 * `build.format` is `"file"`, so these are the paths a host serves.
 */
export const ROUTES = [
  "/index.html",
  "/district/043786.html",
  "/district/043786/finances.html",
  "/districts.html",
  "/statewide.html",
  "/method.html",
  "/wiki/funding-regime/fair-school-funding-plan.html",
  "/wiki/decision/the-four-kinds-of-parameter.html",
] as const;

/**
 * The widths, and why these three.
 *
 * 375 is the narrow phone the chart suite already uses. 1280 is the desktop the site is composed
 * for. 768 is between them and is where the header stops wrapping to three rows and has not yet
 * reached the one-row layout — the width at which the chrome measurement in #189 is worst per
 * pixel of screen.
 */
export const WIDTHS = [375, 768, 1280] as const;

/** One route at one width. */
export interface Measured {
  route: string;
  width: number;

  /* Deterministic — computed CSS only. */

  /** Every distinct rendered `font-size`, ascending, outside `<svg>`. */
  sizes: number[];
  /** `h1` size over `body` size. Below about 2 there is no hierarchy, only a rounding error. */
  headingRatio: number | null;
  /**
   * Bordered, radiused boxes: how many, how deep they nest, and how many are decoration.
   *
   * `count` is what a reader sees. `decorative` is the subset a redesign may actually remove —
   * everything that is not an operable edge and not a floating panel.
   *
   * The split is not a convenience. WCAG 1.4.11 requires a control's boundary to clear 3:1 against
   * its ground, which is what `--border-control` was solved for, so the border on a chip, a tab, a
   * select or the skip link is an accessibility requirement rather than a style. Of the 30 boxes
   * on a district page, 19 are those. A threshold on `count` would therefore be a threshold that
   * can only be met by removing affordances, which is the opposite of the intent — see #185.
   */
  boxes: { count: number; decorative: number; maxDepth: number };
  /** `<td>` carrying more than {@link PROSE_CELL_MIN_WORDS} words and set `text-align: right`. */
  rightAlignedProse: number;

  /* Font-sensitive — layout, and therefore the platform's UI face. */

  /** Reading measure of substantial paragraphs, in `ch`. */
  measure: { median: number; p90: number; max: number; over78: number; count: number } | null;
  /** Height of the sticky header, which wraps and so depends on how wide the labels draw. */
  headerHeight: number | null;
  /** Where the page's own `h1` starts. The chrome above it is the phone's opening screen. */
  firstContentY: number | null;
  /** The advance width of `0` in the body font, which is what a `ch` figure above is divided by. */
  zeroAdvance: number | null;
  /** The family the platform actually resolved, so a `ch` figure can be compared across machines. */
  bodyFont: string | null;
}

/** A whole run. */
export interface Report {
  /** ISO 8601. Passed in rather than read from the clock, so a report is reproducible. */
  measuredAt: string;
  rows: Measured[];
}

/**
 * The thresholds, and the fact that there are none yet.
 *
 * #183 builds the instrument and fails nothing — a check that starts red teaches whoever added it
 * to pass `--no-verify`. Each later phase turns one row of this report into a limit here:
 *
 * - #185 set `boxDecorative` to a hard zero once the card stopped being a bordered box
 * - #186 sets `sizeCount`, `sizeGap` and `headingRatio`, and `measureMax` under a stated font
 * - #188 sets `rightAlignedProse` to a hard zero
 * - #189 sets `firstContentY` at 375px, under a stated font
 *
 * A font-sensitive key must not be set without naming the font it holds under. See the header.
 */
export interface Thresholds {
  /** Most distinct font sizes a route may render. Deterministic. */
  sizeCount?: number;
  /** Largest permitted ratio between two adjacent sizes in the census. Deterministic. */
  sizeGap?: number;
  /** Smallest permitted `h1`-to-body ratio. Deterministic. */
  headingRatio?: number;
  /** Most bordered boxes a route may carry, operable edges included. Deterministic. */
  boxCount?: number;
  /** Most bordered boxes that are neither an operable edge nor a floating panel. Deterministic. */
  boxDecorative?: number;
  /** Deepest a bordered box may nest inside another. Deterministic. */
  boxDepth?: number;
  /** Most right-aligned cells carrying prose. Deterministic, and the target is zero. */
  rightAlignedProse?: number;
  /** Widest permitted paragraph, in `ch`. FONT-SENSITIVE — state the font. */
  measureMax?: number;
  /** Most chrome permitted above the `h1`, in px, at the narrowest width. FONT-SENSITIVE. */
  firstContentY?: number;
}

export const THRESHOLDS: Thresholds = {
  /*
   * #185. A bordered, radiused box that is neither an operable edge nor a floating panel is
   * decoration, and there are none left: the card and the tile both gave theirs up, and what
   * separates a section from the next one is now the interval.
   *
   * A hard zero rather than a budget, because there is no case where a document section needs an
   * outline to say it has ended — the ground and the space already say it. Anything that genuinely
   * needs an edge is a control or floats, and neither is counted here.
   *
   * `boxCount` is deliberately NOT set. 19 of the 30 boxes a district page used to draw are
   * operable edges answering to WCAG 1.4.11, so a threshold on the total could only be met by
   * removing affordances. `boxDepth` is not set either: it fell from 2 to 1 as a consequence of
   * this, not as an independent constraint, and a floating menu panel legitimately contains
   * bordered links — asserting depth would forbid that for no reason.
   */
  boxDecorative: 0,
};

/** One breach of one threshold, named so the message says what to do rather than what happened. */
export interface Violation {
  route: string;
  width: number;
  metric: keyof Thresholds;
  measured: number;
  limit: number;
  message: string;
}

/**
 * The widest ratio between two adjacent sizes in a census.
 *
 * A scale is a scale because its steps are even. Fifteen sizes crowded into 4.2px and then a jump
 * to the `h1` is two clusters rather than a ramp, and the number that says so is the largest gap
 * between neighbours — 22.4 / 15.2 = 1.47 today, against roughly 1.15 between every other pair.
 *
 * Returns 1 for a census too short to have a gap, which is the identity: no gap to be too wide.
 */
export function widestGap(sizes: number[]): number {
  const ascending = [...sizes].sort((a, b) => a - b);
  let widest = 1;
  for (let i = 1; i < ascending.length; i += 1) {
    const previous = ascending[i - 1];
    const current = ascending[i];
    if (previous == null || current == null || previous <= 0) continue;
    widest = Math.max(widest, current / previous);
  }
  return widest;
}

/**
 * Every threshold this report breaches.
 *
 * Deliberately returns the whole list rather than throwing on the first: a redesign phase wants to
 * see all of what it moved, and a check that stops at the first failure turns one run into six.
 */
export function violations(report: Report, thresholds: Thresholds = THRESHOLDS): Violation[] {
  const found: Violation[] = [];
  const at = (row: Measured, metric: keyof Thresholds, measured: number, limit: number, message: string) =>
    found.push({ route: row.route, width: row.width, metric, measured, limit, message });

  for (const row of report.rows) {
    if (thresholds.sizeCount != null && row.sizes.length > thresholds.sizeCount) {
      at(row, "sizeCount", row.sizes.length, thresholds.sizeCount,
        `renders ${row.sizes.length} distinct font sizes; the scale allows ${thresholds.sizeCount}`);
    }
    if (thresholds.sizeGap != null) {
      const gap = widestGap(row.sizes);
      if (gap > thresholds.sizeGap) {
        at(row, "sizeGap", gap, thresholds.sizeGap,
          `the widest step in the size census is ${gap.toFixed(2)}×, over ${thresholds.sizeGap}×`);
      }
    }
    if (thresholds.headingRatio != null && row.headingRatio != null && row.headingRatio < thresholds.headingRatio) {
      at(row, "headingRatio", row.headingRatio, thresholds.headingRatio,
        `the h1 is ${row.headingRatio.toFixed(2)}× body, under ${thresholds.headingRatio}×`);
    }
    if (thresholds.boxCount != null && row.boxes.count > thresholds.boxCount) {
      at(row, "boxCount", row.boxes.count, thresholds.boxCount,
        `carries ${row.boxes.count} bordered boxes, over ${thresholds.boxCount}`);
    }
    if (thresholds.boxDecorative != null && row.boxes.decorative > thresholds.boxDecorative) {
      at(row, "boxDecorative", row.boxes.decorative, thresholds.boxDecorative,
        `carries ${row.boxes.decorative} bordered boxes that are decoration, over ${thresholds.boxDecorative}`);
    }
    if (thresholds.boxDepth != null && row.boxes.maxDepth > thresholds.boxDepth) {
      at(row, "boxDepth", row.boxes.maxDepth, thresholds.boxDepth,
        `nests bordered boxes ${row.boxes.maxDepth} deep, over ${thresholds.boxDepth}`);
    }
    if (thresholds.rightAlignedProse != null && row.rightAlignedProse > thresholds.rightAlignedProse) {
      at(row, "rightAlignedProse", row.rightAlignedProse, thresholds.rightAlignedProse,
        `sets ${row.rightAlignedProse} sentence-bearing cells right-aligned, over ${thresholds.rightAlignedProse}`);
    }
    if (thresholds.measureMax != null && row.measure != null && row.measure.max > thresholds.measureMax) {
      at(row, "measureMax", row.measure.max, thresholds.measureMax,
        `sets a paragraph at ${row.measure.max.toFixed(0)}ch, over ${thresholds.measureMax}ch ` +
        `(measured in ${row.bodyFont ?? "an unrecorded font"})`);
    }
    if (thresholds.firstContentY != null && row.firstContentY != null && row.width === WIDTHS[0]
        && row.firstContentY > thresholds.firstContentY) {
      at(row, "firstContentY", row.firstContentY, thresholds.firstContentY,
        `puts the h1 at y=${row.firstContentY}, over ${thresholds.firstContentY}px of chrome ` +
        `(measured in ${row.bodyFont ?? "an unrecorded font"})`);
    }
  }
  return found;
}

/** `1234.5` as `1,234.5`, and `null` as an em dash, so a column of these stays a column. */
function cell(value: number | null | undefined, decimals = 0): string {
  if (value == null || Number.isNaN(value)) return "—";
  return value.toLocaleString("en", { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
}

/**
 * The report as a table, one block per width.
 *
 * Grouped by width rather than by route because every font-sensitive column only means anything
 * against a stated width, and a reader comparing two routes is nearly always comparing them at the
 * same one.
 */
export function formatReport(report: Report): string {
  const COLUMNS: Array<{ head: string; width: number; of: (row: Measured) => string }> = [
    { head: "route", width: 44, of: (r) => r.route.replace(/\.html$/, "") },
    { head: "sizes", width: 5, of: (r) => cell(r.sizes.length) },
    { head: "gap", width: 5, of: (r) => `${widestGap(r.sizes).toFixed(2)}×` },
    { head: "h1/body", width: 7, of: (r) => (r.headingRatio == null ? "—" : `${r.headingRatio.toFixed(2)}×`) },
    { head: "med", width: 4, of: (r) => cell(r.measure?.median) },
    { head: "p90", width: 4, of: (r) => cell(r.measure?.p90) },
    { head: "max", width: 4, of: (r) => cell(r.measure?.max) },
    { head: ">78", width: 4, of: (r) => cell(r.measure?.over78) },
    { head: "boxes", width: 5, of: (r) => cell(r.boxes.count) },
    { head: "deco", width: 4, of: (r) => cell(r.boxes.decorative) },
    { head: "deep", width: 4, of: (r) => cell(r.boxes.maxDepth) },
    { head: "cells", width: 5, of: (r) => cell(r.rightAlignedProse) },
    { head: "chrome", width: 6, of: (r) => cell(r.firstContentY) },
  ];

  const lines: string[] = [];
  lines.push(`MEASURE REPORT — ${report.rows.length} rows, measured ${report.measuredAt}`);
  lines.push("");
  lines.push("  med/p90/max/>78 are the reading measure in ch and DEPEND ON THE PLATFORM'S FONT.");
  lines.push("  chrome is the y-offset of the h1, and depends on it too. The rest are computed CSS.");

  for (const width of WIDTHS) {
    const rows = report.rows.filter((r) => r.width === width);
    if (rows.length === 0) continue;
    const font = rows.find((r) => r.bodyFont != null);
    lines.push("");
    lines.push(
      `${width}px` +
        (font?.bodyFont ? `  —  ${font.bodyFont}, 0 draws ${font.zeroAdvance?.toFixed(2)}px` : ""),
    );
    lines.push(
      "  " + COLUMNS.map((c) => (c.head === "route" ? c.head.padEnd(c.width) : c.head.padStart(c.width))).join(" "),
    );
    lines.push("  " + COLUMNS.map((c) => "─".repeat(c.width)).join(" "));
    for (const row of rows) {
      lines.push(
        "  " +
          COLUMNS.map((c) => {
            const text = c.of(row);
            return c.head === "route" ? text.padEnd(c.width) : text.padStart(c.width);
          }).join(" "),
      );
    }
  }

  return lines.join("\n");
}

/**
 * Collect every metric from the page this is evaluated in.
 *
 * Self-contained by necessity: Playwright serialises this function and runs it in the browser, so
 * it may close over nothing. The three constants it needs are therefore arguments rather than the
 * module-level ones above, and the caller passes those — which also makes the coupling visible
 * instead of leaving two copies of `140` in two files.
 */
export function collect(limits: {
  proseMinChars: number;
  proseCellMinWords: number;
  boxMinRadius: number;
}): Omit<Measured, "route" | "width"> {
  const { proseMinChars, proseCellMinWords, boxMinRadius } = limits;

  const visible = (el: Element): boolean => {
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 && rect.height === 0) return false;
    const style = getComputedStyle(el);
    return style.visibility !== "hidden" && style.display !== "none";
  };

  /* The size census. Only elements that themselves paint text, and never inside an <svg>: a
     chart's font-size attribute is scaled by its viewBox and is not the size that lands. */
  const sizes = new Set<number>();
  for (const el of document.querySelectorAll("body *")) {
    if (el.closest("svg") != null) continue;
    let paintsText = false;
    for (const node of el.childNodes) {
      if (node.nodeType === 3 && (node.textContent ?? "").trim().length > 0) paintsText = true;
    }
    if (!paintsText || !visible(el)) continue;
    sizes.add(Math.round(parseFloat(getComputedStyle(el).fontSize) * 100) / 100);
  }

  const bodyStyle = getComputedStyle(document.body);
  const bodySize = parseFloat(bodyStyle.fontSize);
  const h1 = document.querySelector("h1");
  const headingRatio =
    h1 != null && bodySize > 0 ? parseFloat(getComputedStyle(h1).fontSize) / bodySize : null;

  /* The advance width of `0` in the body font, which is the divisor behind every `ch` below.
     Measured rather than assumed, and reported, because it is what makes two machines comparable. */
  const ruler = document.createElement("span");
  ruler.style.cssText = `position:absolute;visibility:hidden;white-space:pre;font:${bodyStyle.font}`;
  ruler.textContent = "0".repeat(100);
  document.body.appendChild(ruler);
  const zeroAdvance = ruler.getBoundingClientRect().width / 100;
  ruler.remove();

  /* The reading measure. Per-paragraph, in that paragraph's own font — a `<p>` inside a card and
     one inside `.prose-body` are set at different sizes, so one divisor for the page would be
     wrong for half of them. Cached by font string; there are two or three distinct ones. */
  const advances = new Map<string, number>();
  const advanceIn = (font: string): number => {
    const cached = advances.get(font);
    if (cached != null) return cached;
    const span = document.createElement("span");
    span.style.cssText = `position:absolute;visibility:hidden;white-space:pre;font:${font}`;
    span.textContent = "0".repeat(100);
    document.body.appendChild(span);
    const advance = span.getBoundingClientRect().width / 100;
    span.remove();
    advances.set(font, advance);
    return advance;
  };

  const widths: number[] = [];
  for (const p of document.querySelectorAll("p")) {
    if ((p.textContent ?? "").trim().length < proseMinChars || !visible(p)) continue;
    const advance = advanceIn(getComputedStyle(p).font);
    if (advance > 0) widths.push(p.getBoundingClientRect().width / advance);
  }
  widths.sort((a, b) => a - b);
  const at = (q: number): number => widths[Math.min(widths.length - 1, Math.floor(widths.length * q))] ?? 0;
  const measure =
    widths.length === 0
      ? null
      : {
          median: Math.round(at(0.5)),
          p90: Math.round(at(0.9)),
          max: Math.round(widths[widths.length - 1] ?? 0),
          over78: widths.filter((w) => w > 78).length,
          count: widths.length,
        };

  /* Bordered, radiused boxes, and their nesting. A box needs a border that is actually drawn —
     a `border-style: none` at 1px paints nothing and is not a boundary a reader can see. */
  const boxed = new Set<Element>();
  const decorative = new Set<Element>();
  /* An operable edge answers to WCAG 1.4.11 rather than to taste, and a floating panel needs an
     edge because it sits over content rather than in it. Shadow is a sound proxy for floating
     here and not a guess: `stylesheet.spec.ts` asserts that nothing which sits on the page
     carries one, so anything that does is by construction above it. */
  const OPERABLE = "a[href], button, input, select, textarea, summary, label, [tabindex]";
  for (const el of document.querySelectorAll("body *")) {
    const style = getComputedStyle(el);
    if (style.borderTopStyle === "none" || parseFloat(style.borderTopWidth) === 0) continue;
    if (parseFloat(style.borderTopLeftRadius) < boxMinRadius) continue;
    if (!visible(el)) continue;
    boxed.add(el);
    const floating = style.boxShadow !== "none" && style.boxShadow !== "";
    if (!el.matches(OPERABLE) && !floating) decorative.add(el);
  }
  let maxDepth = 0;
  for (const el of boxed) {
    let depth = 0;
    for (let node: Element | null = el; node != null; node = node.parentElement) {
      if (boxed.has(node)) depth += 1;
    }
    maxDepth = Math.max(maxDepth, depth);
  }

  let rightAlignedProse = 0;
  for (const td of document.querySelectorAll("td")) {
    if (getComputedStyle(td).textAlign !== "right") continue;
    const words = (td.textContent ?? "").trim().split(/\s+/).filter(Boolean).length;
    if (words > proseCellMinWords) rightAlignedProse += 1;
  }

  const header = document.querySelector("header.site");
  return {
    sizes: [...sizes].sort((a, b) => a - b),
    headingRatio,
    boxes: { count: boxed.size, decorative: decorative.size, maxDepth },
    rightAlignedProse,
    measure,
    headerHeight: header == null ? null : Math.round(header.getBoundingClientRect().height),
    firstContentY: h1 == null ? null : Math.round(h1.getBoundingClientRect().top + window.scrollY),
    zeroAdvance,
    bodyFont: bodyStyle.fontFamily,
  };
}
