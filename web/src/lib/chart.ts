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
   * *category* is not available: the palette is two hues and generates no others.
   *
   * `"neutral"` is not a third category. It is a point the split does not classify, and it is
   * spelled out rather than left as an absence because the two are different claims: a scatter
   * with no `series` at all is an unsplit population drawn in one colour, whereas a `"neutral"`
   * point sits in a split population that has nothing to say about *this* one. The guarantee's
   * two terms are the case — below a multiple of one there is no majority to take, so those
   * districts are a third state — and `plot/tokens.ts` has the slot waiting: "any mark with no
   * polarity. Never a hue."
   */
  series?: "formula" | "guarantee" | "neutral";
  /**
   * Which ordered band of a *third* measure this district falls in, 0 lowest.
   *
   * Drawn in the ordinal ramp, which is three steps wide — see `plot/tokens.ts` for why three and
   * not five. Only worth spending where the banding variable is not already an axis: banding the
   * poverty scatter by poverty would repaint the x axis in a gradient and say nothing.
   */
  band?: number;
  /**
   * Where this district sat before the change being drawn — its position under current law.
   *
   * Present only on a scatter whose subject is **movement**, where the pair of positions is the
   * finding and neither one alone is. `scatterSpec` draws a segment from here to `{x, y}`, so a
   * district that did not move draws nothing and reads as a dot with no tail.
   *
   * A **flat** tail is the point. Under Ohio's guarantee a district is paid `max(formula, floor)`,
   * so raising what the formula computes moves what the guarantee absorbs and not what the
   * district receives: the district travels along the x axis and its payment does not change.
   * 253 of 609 are paid the same under a $220M base cost refresh, and a chart of after-positions
   * alone cannot show that they are the same 253 every time.
   */
  from?: { x: number; y: number };
  /**
   * Draw this district as context rather than as subject.
   *
   * A second channel for a second fact, and the reason it is not a third hue: the palette has two
   * and a three-step ordinal ramp, and the three steps are a measured limit rather than a
   * stylistic one. Where a caller needs to say *this point is outside the population you asked
   * about* **alongside** what it is already saying with colour, hue has nothing left to spend.
   *
   * `/reach`'s scope is the case. Under its regime and typology colourings an out-of-scope
   * district could be left neutral and read correctly, but under "gained or lost" neutral already
   * means *unmoved* — so one fill would be carrying two unrelated claims and the legend would have
   * to pick which one to name.
   *
   * Drawn on `r` **and** opacity, because either alone is too weak here. `--neutral-mark` is close
   * to its surface by design — 1.42:1 at the cloud's own 0.45, which is what makes density legible
   * as density — so muting it further separates the two populations by only 1.20:1, and that is not
   * a difference a reader picks out of six hundred overlapping dots. Halving the area as well takes
   * the ink to roughly a fifth. See `DOT` in `plot/spec.ts` for the figures.
   *
   * The hit layer does not shrink. It is a separate mark at `r: 7` for every point, so a muted
   * district is still something a reader can point at and still carries its whole tooltip — which
   * is what keeps this a change of emphasis rather than a removal.
   */
  muted?: boolean;
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
 * A fitted line through a scatter, and the two numbers it is the line of.
 *
 * The exception {@link Trace} says does not exist, and it exists on exactly the terms that note
 * lays down: "a regression line is a claim about a model and would belong in `crates/` with a
 * checkpoint behind it". So this one does. `crates/figures` fits it, `crates/series.json` carries
 * it, and the slope and r-squared are ordinary pinned figures the corpus node quotes in prose and
 * the figure gate checks three ways. Nothing in the web layer fits anything.
 *
 * It is carried as **two endpoints in the data's own units** rather than as a slope and an
 * intercept, and that is not a convenience. A line given as a model has to be evaluated somewhere,
 * and evaluating it here would be this layer computing a claim again — in a different language,
 * against a centred design it cannot see (the intercept of `dispersion::least_squares` is the mean
 * of the outcome, not the value at zero, which is a mistake already made once). Two points are a
 * segment. There is nothing to get wrong.
 *
 * `slope` and `rSquared` are the only numbers printed on the panel, which is the cloud's form of
 * the endpoint rule: what a reader can quote off a chart has to be a number the corpus states and
 * a figure pins. They are signed, because a slope of −0.0429 is the finding.
 */
export interface Fit {
  from: { x: number; y: number };
  to: { x: number; y: number };
  slope: number;
  rSquared: number;
}

/**
 * One named thing at a point on two measures at once.
 *
 * A {@link ScatterPoint} without the population around it, and the difference is what may be
 * drawn. Six hundred districts are a texture and a dot in one carries its name only in a tooltip;
 * seven named policies are a *list*, every one of which a reader will want to quote by name, so
 * the name is printed beside the mark and the mark is drawn at a size a reader can point at.
 *
 * Both coordinates are signed and zero is a position rather than a floor — the two zero rules are
 * what divide the frame into the four quadrants the form is read in. Which is why this is its own
 * shape rather than a flag on `ScatterPoint`: that one's `muted`, `band` and `series` channels
 * are for saying which part of a population a dot is in, and a plane of seven has no parts.
 */
export interface Place {
  label: string;
  x: number;
  y: number;
  hover: string;
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
 * One item of a long ranking: a name, a count, and whether it is the row the chart is about.
 *
 * A {@link Bar} with two of its fields renamed would have done for a short column. It is its own
 * shape because of what {@link marked} is for. `Bar.current` is a flag — *this row is the
 * subject* — and a flag is enough when the subject is one of six bars a reader can already see.
 * On thirty-eight rows drawn on a log axis, the subject of the census of the plan's bounds is the
 * row at **zero**, which is the one position a log axis cannot draw and a length cannot encode.
 * So the mark carries a phrase rather than a boolean: it is drawn at the axis floor with that
 * phrase printed beside it, which is the only way a null result appears on a chart of magnitudes
 * as something other than an absence.
 */
export interface Rank {
  label: string;
  /** The count. Zero is a legitimate value here and is the one this form was written for. */
  value: number;
  hover: string;
  /**
   * Why this row is the one the chart was drawn to point at, in two or three words.
   *
   * At most one row of a ranking may carry it, for {@link Bar.current}'s reason: a chart with two
   * subjects has none. Printed beside the mark, so the phrase is read rather than inferred from
   * a colour, and the colour is the second channel rather than the only one.
   */
  marked?: string;
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

/**
 * One position of a two-series line chart, and the pair of values there.
 *
 * The index is a bare number and is deliberately not called a year. It is a fiscal year on
 * `/history`, `/appropriations` and `/data`, and a **horizon** on `/method`'s coverage curve — the
 * form is the same in both cases (two quantities in one unit over one ordered index) and what
 * differs is only how the ends are written, which the caller supplies. A field called `year`
 * would have made the one non-year caller read as a bug.
 */
export interface SeriesPoint {
  /** Where on the index this point sits. Ordered, ascending, and evenly spaced. */
  at: number;
  /** The first series. `null` is a position the source does not publish — see {@link seriesSpec}. */
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

  /**
   * The visible marks a hit target names, if its layer declares any.
   *
   * The pairing is written on the layer by `declareCursor` and followed here by DOM position: a
   * hit mark's index among its siblings is its index in the data, because Plot draws one element
   * per datum in order and both layers are drawn from the same array. #220's own measurement of
   * that alignment was 84 of 84 on `/counties`; `tests/e2e/cursor.spec.ts` now checks it over the
   * whole build rather than one page, by child count on every paired layer.
   */
  const paired = (mark: Element): Element[] => {
    const layer = mark.parentElement;
    const declared = layer?.getAttribute("data-paired");
    if (!layer || !declared) return [];
    const index = [...layer.children].indexOf(mark);
    const svg = layer.closest("svg.plot");
    if (!svg || index < 0) return [];
    return declared.split(",").flatMap((selector) => {
      const twin = svg.querySelector(selector.trim())?.children[index];
      return twin ? [twin] : [];
    });
  };

  /** Put the cursor on a mark, or take it off: both channels move together or neither does. */
  const cursor = (mark: Element, on: boolean) => {
    mark.classList.toggle("at", on);
    for (const twin of paired(mark)) twin.classList.toggle("at-mark", on);
  };

  const drop = () => {
    if (at) cursor(at, false);
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
    if (at) cursor(at, false);
    at = marks[next] ?? null;
    if (!at) return;
    cursor(at, true);
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
