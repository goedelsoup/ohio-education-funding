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
  /**
   * The bar the chart was built to locate — Ohio, in a chart of the states.
   *
   * `statewide.ts` set this on the national chart and nothing read it: `Bar` did not declare the
   * field and `barSpec` ignored it, so the one mark the chart exists to point at carried no mark
   * at all. Honoured on two channels, because one of them is colour — see {@link barSpec}.
   *
   * Not a general "highlight this" flag. It says *this row is the subject*, which is a fact about
   * the page rather than about the data, and a chart with two subjects has none.
   */
  current?: boolean;
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
  /**
   * Which ordered band of a *third* measure this district falls in, 0 lowest.
   *
   * Drawn in the ordinal ramp, which is three steps wide — see `plot/tokens.ts` for why three and
   * not five. Only worth spending where the banding variable is not already an axis: banding the
   * poverty scatter by poverty would repaint the x axis in a gradient and say nothing.
   */
  band?: number;
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
  /** Draw in this step of the ordinal ramp instead, for a trace summarising one band. */
  band?: number;
  points: { x: number; y: number }[];
}

/**
 * One item with two values on the same measure — a low end and a high end.
 *
 * The shape a ratio compresses. `/counties` ranked its 88 counties by richest ÷ poorest valuation
 * per pupil, which is one number standing for two, and the two are not recoverable from it: Brown
 * spans $124k–$259k and Wood $198k–$407k at the same 2.1×, so Wood's *poorest* district stands on
 * more valuation per pupil than Brown's richest. Ordering the counties by disparity and ordering
 * them by floor agree for 29 of 84.
 */
export interface Range {
  /** Printed at the row, because eighty-four unlabelled bars is a texture. */
  label: string;
  low: number;
  high: number;
  hover: string;
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
 * What a chart is worth, to a pointer, to a finger and to a keyboard.
 *
 * # What was wrong
 *
 * This bound `mousemove` and nothing else. There are 159,530 marks in the build carrying a
 * `data-hover` string, and every one of those values was reachable only by a mouse: no focus
 * handler, so a keyboard could not reach them; no touch handler, so a phone could not either.
 * The forms whose stated purpose is "which dot is mine" answered that question for pointers only.
 *
 * # The three ways in
 *
 * **Pointer.** Delegated `mousemove`, unchanged, so it survives the scenario charts being replaced
 * on every slider tick and is positioned against the viewport rather than clipped by a card.
 *
 * **Touch.** A tap. There is no hover to precede it, so `click` is the whole of the interaction —
 * and the tooltip is placed against the mark rather than the finger, which is under it.
 *
 * **Keyboard.** One tab stop per chart, not one per mark: 159,530 tab stops would be a worse
 * defect than the one being fixed. Focus the chart and the arrow keys walk its marks, `Home` and
 * `End` jump to the ends, `Escape` steps out. The value shows in the same tooltip and is written
 * to a polite live region.
 *
 * # What this does not fix, stated rather than implied
 *
 * A screen reader in browse mode takes the arrow keys for itself before the page sees them, so
 * the cursor below is reached by a sighted keyboard reader, a switch user, and a screen-reader
 * user who has switched to focus mode — and not by one who has not. Exposing every mark to the
 * accessibility tree instead would mean 159,530 nodes and would undo the `role="img"` naming that
 * makes a chart announce itself as one thing. Where a chart sits beside a table of the same
 * figures — 4,501 of the 7,593 do — that table is still the better route, and it is now a keyboard
 * region of its own. The rest is the honest remainder.
 */

/** Text a chart is given when it becomes operable, so the affordance is announced with it. */
const CURSOR_HINT = "Use the arrow keys to read each value.";

/**
 * Make one drawing a single tab stop, and say so in its name.
 *
 * Applied from script and never baked into the build, because the cursor it advertises is script.
 * A `tabindex` in the HTML would be a stop that goes nowhere for a reader with none running, which
 * is the trade `BasisToggle.astro` already decided once for this site.
 *
 * Skips a chart with no values to walk and a chart hidden from assistive technology — a
 * presentational strip is one whose `.note` already says the same thing in words, and giving it a
 * tab stop would be adding an affordance to a graphic that is deliberately not content.
 */
export function openToKeyboard(svg: Element): void {
  if (svg.getAttribute("aria-hidden") === "true") return;
  if (!svg.querySelector("[data-hover]")) return;
  svg.setAttribute("tabindex", "0");
  const named = svg.getAttribute("aria-label");
  if (named && !named.endsWith(CURSOR_HINT)) {
    svg.setAttribute("aria-label", `${named}. ${CURSOR_HINT}`);
  }
}

/**
 * Attach the value layer to everything under `root`.
 *
 * `tip` is the tooltip a reader sees; `said` is the visually hidden live region a screen reader
 * hears. Two elements rather than one because the tooltip is written on every mouse move, and a
 * live region that announced each of those would be unusable.
 */
export function attachValues(root: HTMLElement, tip: HTMLElement, said: HTMLElement): void {
  /** The mark the keyboard cursor is on, if a keyboard put it there. */
  let at: Element | null = null;

  const put = (left: number, top: number) => {
    const pad = 12;
    tip.style.left = `${Math.min(left + pad, window.innerWidth - tip.offsetWidth - pad)}px`;
    tip.style.top = `${top + pad}px`;
  };

  const show = (mark: Element, left: number, top: number) => {
    tip.textContent = mark.getAttribute("data-hover") ?? "";
    tip.hidden = false;
    put(left, top);
  };

  /** Show the value at a mark's own position, for the two ways in that have no cursor position. */
  const showAtMark = (mark: Element) => {
    const box = mark.getBoundingClientRect();
    show(mark, box.left + box.width / 2, box.bottom);
  };

  const drop = () => {
    at?.classList.remove("at");
    at = null;
    said.textContent = "";
  };

  root.addEventListener("mousemove", (event) => {
    const target = (event.target as Element | null)?.closest("[data-hover]");
    if (!target) {
      tip.hidden = true;
      return;
    }
    // A reader who has picked up the mouse has left the cursor behind, and two highlighted marks
    // would be two answers to "which one am I on".
    if (at && at !== target) drop();
    show(target, event.clientX, event.clientY);
  });

  root.addEventListener("mouseleave", () => {
    tip.hidden = true;
  });

  /*
   * Touch. `click` rather than `pointerdown`, so a drag that happens to begin on a mark scrolls
   * the page instead of firing a tooltip at the reader.
   */
  root.addEventListener("click", (event) => {
    const target = (event.target as Element | null)?.closest?.("[data-hover]");
    if (!target) {
      if (at == null) tip.hidden = true;
      return;
    }
    showAtMark(target);
  });

  root.addEventListener("keydown", (event) => {
    const svg = (event.target as Element | null)?.closest?.("svg.plot[tabindex]");
    if (!svg) return;
    const marks = [...svg.querySelectorAll("[data-hover]")];
    if (marks.length === 0) return;

    const here = at ? marks.indexOf(at) : -1;
    const last = marks.length - 1;
    let next: number;
    switch (event.key) {
      case "ArrowRight":
      case "ArrowDown":
        next = here < 0 ? 0 : Math.min(last, here + 1);
        break;
      case "ArrowLeft":
      case "ArrowUp":
        // From nowhere, leftwards means the far end — the same reading a reader would give it.
        next = here < 0 ? last : Math.max(0, here - 1);
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = last;
        break;
      case "Escape":
        drop();
        tip.hidden = true;
        return;
      default:
        return;
    }
    // Only once a key is one this handles: arrows still scroll a chart nobody is reading.
    event.preventDefault();
    at?.classList.remove("at");
    at = marks[next] ?? null;
    if (!at) return;
    at.classList.add("at");
    showAtMark(at);
    said.textContent = at.getAttribute("data-hover") ?? "";
  });

  // Tabbing out of a chart takes the cursor with it, or the highlight outlives the reading.
  root.addEventListener("focusout", (event) => {
    const svg = (event.target as Element | null)?.closest?.("svg.plot[tabindex]");
    if (!svg) return;
    drop();
    tip.hidden = true;
  });
}
