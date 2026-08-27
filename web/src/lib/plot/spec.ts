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

/**
 * How a chart is announced to a reader who cannot see it.
 *
 * Every build-time SVG on this site shipped without one of these for as long as there have been
 * charts: 7,605 unnamed graphics, each read out as its own internal group names — "dot", "line",
 * "rule" — followed by whatever text marks happened to sit in it. The district fan chart's entire
 * accessible content was five numbers in a row with nothing saying what they measured.
 *
 * This is a required argument to both renderers rather than a field on `Spec`, and the placement
 * is the point. `barSpec(bars)` builds six different charts here — cash balance, casino receipts,
 * categorical aid, base cost, tax base, spending by function — so the name is a property of the
 * USE and not of the shape. A spec cannot know what it is about; the call site always does.
 *
 * Being required is what makes it hold. A new chart does not compile until it says what it is,
 * which is the same shape as `ensureThemeable` refusing to emit one with a baked colour.
 */
export type Naming =
  /**
   * Announced as one graphic with this name.
   *
   * `role="img"` goes on beside the label, so assistive technology reads the name instead of
   * walking the SVG. That is the right trade here: the numbers inside are positions rather than a
   * reading order, and every chart on this site sits beside prose or a table carrying the same
   * figures. The label is therefore the whole of what a screen reader gets — write it as what is
   * plotted against what, with the basis, not as a description of a picture.
   */
  | { label: string; description?: string }
  /**
   * Hidden from assistive technology, because the text beside it already says what it says.
   *
   * Only where that is literally true — a 46px strip whose `.note` states the same position in
   * words — and never merely because a chart is secondary. A hidden chart is one a reader cannot
   * reach at all.
   */
  | "presentational";

/** Anything a renderer can put attributes on, so this file needs no DOM lib. */
type Nameable = { setAttribute: (name: string, value: string) => void };

/**
 * Put the naming onto a rendered root. Shared by both renderers so they cannot disagree.
 *
 * The description is folded into the label rather than set as `aria-description`: that property
 * is ARIA 1.3 and is not reliably announced yet, and a second sentence that may go unread is
 * worse than one that is certainly read, because nobody writing it would know.
 */
/**
 * Take out what Plot emits that this platform supplies itself.
 *
 * Two things, and both are Plot describing its own rendering rather than the chart.
 *
 * **The mark labels.** Plot writes `aria-label="bar"`, `aria-label="rule"`, `aria-label="text"`
 * onto the `<g>` it wraps each mark in. On a `g` with no role that attribute is not permitted —
 * `aria-label` needs a role that supports naming — so every chart carried a handful of invalid
 * ARIA, which is what `axe` reports as `aria-prohibited-attr`. It was useless before it was
 * invalid: the whole SVG is `role="img"`, so nothing inside is exposed to be named, and "bar" is
 * not a name for anything a reader wanted.
 *
 * **The stylesheet.** Plot inlines a 198-byte `<style>` into every SVG it draws, and every one of
 * them is the same 198 bytes: 15,210 copies, **3.01 MB** of the built HTML, for five declarations.
 * They live in `app.css` under `.plot` now, which is where the rest of this site's chart styling
 * already is — see the `.plot` block there for what each one is doing and which of them is
 * load-bearing.
 *
 * Removed rather than given a role, and hoisted rather than deduplicated by the host: a `g` per
 * mark type is Plot's rendering structure and a stylesheet repeated per element is a stylesheet in
 * the wrong place, whatever compresses it on the wire.
 */
function untangle(node: Element): void {
  for (const group of node.querySelectorAll("g[aria-label]")) {
    if (!group.hasAttribute("role")) group.removeAttribute("aria-label");
  }
  for (const style of node.querySelectorAll("style")) style.remove();
  round(node);
}

/**
 * The geometry attributes, and only those.
 *
 * A whitelist rather than a pass over the whole serialized SVG, because two of the attributes that
 * carry long decimals are *text*: `data-hover` holds a district's figures — `1499.2266` enrolled
 * ADM — and `aria-label` holds the sentence a screen reader is read. Rounding those would be
 * rounding what the chart says rather than where it puts it, silently, in the one place a reader
 * cannot check it against the table.
 *
 * `d`, `transform` and `points` hold several numbers each; the rest hold one. The same rule
 * applies inside all of them, which is why the substitution is on the value and not the attribute.
 */
const GEOMETRY = new Set([
  "cx", "cy", "d", "dx", "dy", "height", "points", "r", "rx", "ry",
  "transform", "width", "x", "x1", "x2", "y", "y1", "y2",
]);

/**
 * Round coordinates to two decimal places.
 *
 * Plot serialises float64 verbatim, so a scatter dot is placed at `cx="171.26003133728457"` — 18
 * characters to say something the frame can express in six. Across the build that is **15.2 MB of
 * the 79.6 MB** of SVG spent on digits below the visible threshold, and `/outcomes` alone is 29%
 * decimal noise.
 *
 * Two places is not a guess. A chart's `viewBox` is 320 or 640 units wide and is drawn between
 * about 293px and 1,100px, so one user unit is 0.5px at its smallest — and a hundredth of that is
 * 0.005px, which is a fortieth of a device pixel on a three-times display. Nothing at that scale
 * reaches a screen, and the end-to-end tests measure the rendered geometry rather than trusting
 * this: the identity line on `/method` is still asserted square to within 2%.
 */
function round(node: Element): void {
  for (const element of node.querySelectorAll("*")) {
    for (const name of element.getAttributeNames()) {
      if (!GEOMETRY.has(name)) continue;
      const value = element.getAttribute(name);
      if (value == null || !value.includes(".")) continue;
      element.setAttribute(
        name,
        value.replace(/-?\d+\.\d+/g, (n) => String(Number(Number(n).toFixed(2)))),
      );
    }
  }
}

export function applyNaming(node: Nameable, naming: Naming): void {
  // `Nameable` is deliberately the smallest surface this file needs, so the tidy-up is guarded
  // rather than assumed: `ssr.ts` and `client.ts` both hand over a real element.
  if ("querySelectorAll" in node) untangle(node as unknown as Element);
  if (naming === "presentational") {
    node.setAttribute("aria-hidden", "true");
    return;
  }
  node.setAttribute("role", "img");
  node.setAttribute(
    "aria-label",
    naming.description ? `${naming.label}. ${naming.description}` : naming.label,
  );
}

/**
 * How a chart delivers the *second* of the keyboard cursor's two channels.
 *
 * The site's rule is an outline around the mark and a brightening of it, so the cursor survives a
 * forced-colours mode that discards the second. `filter: brightness()` on a `fill: transparent`
 * element brightens nothing, and most charts here point the reader at an invisible hit layer
 * drawn above the marks — so on 81.5% of the site's 319,060 hover targets the rule was a
 * description of one channel. That is #220, and this type is the fix: each chart says which of
 * three situations it is in, so the answer is declared and checkable rather than inferred from
 * a class name.
 */
export type Cursor =
  /** The target is the mark. `filter` acts on it directly — the bar charts and the histogram. */
  | { second: "the mark itself" }
  /**
   * The target is invisible, and these layers hold the marks it names. Index-aligned with the hit
   * layer because both are drawn from the same array, which is what makes the pairing safe to
   * follow by DOM position; {@link declareCursor} writes it into the document and
   * `attachValues` reads it back.
   */
  | { second: "paired marks"; layers: string[] }
  /**
   * There is no mark to brighten. `because` is not documentation — `tests/e2e/cursor.spec.ts`
   * requires every exempt layer to be named there with its reason, so an exemption cannot be
   * taken silently.
   */
  | { second: "none"; because: string };

/** A chart, and the tooltip text for the marks a reader can point at. */
export interface Spec {
  options: Plot.PlotOptions;
  /** CSS selector for the hoverable marks, and one string per mark in data order. */
  hovers?: { selector: string; text: string[]; cursor: Cursor };
}

/**
 * Write the mark pairing onto the hit layer, for the cursor to follow at read time.
 *
 * One attribute per chart rather than one per mark. Stamping an index onto every hoverable
 * element and its twin would be 485,000 attributes across the built site — about 2.5 KB a page on
 * `/districts` — to encode something the DOM already says: a hit mark's position among its
 * siblings *is* its index, because Plot draws one element per datum in order.
 */
export function declareCursor(root: Element, hovers: Spec["hovers"]): string | null {
  if (!hovers || hovers.cursor.second !== "paired marks") return null;
  const marks = root.querySelectorAll(hovers.selector);
  const layer = marks[0]?.parentElement;
  if (!layer) return null;
  /*
   * The alignment is checked here rather than asserted in a test over `dist/`, on the same
   * argument `attachHovers` makes one function above: a pairing followed by index that does not
   * line up brightens the *wrong* mark and looks entirely correct doing it. The build is where
   * that can be caught for all 3,506 pages at once.
   */
  for (const selector of hovers.cursor.layers) {
    const twin = root.querySelector(selector);
    const held = twin?.childElementCount ?? 0;
    if (held !== marks.length) {
      return (
        `The cursor's paired layer "${selector}" holds ${held} marks against ${marks.length} ` +
        `hit targets matching "${hovers.selector}". Following that by index would brighten the ` +
        `wrong mark.`
      );
    }
  }
  layer.setAttribute("data-paired", hovers.cursor.layers.join(","));
  return null;
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

/**
 * The two widths every chart is drawn at.
 *
 * # Why a chart is drawn twice rather than scaled
 *
 * A build-time SVG has one size, and the stylesheet used to make it fit by scaling: `width: 100%`
 * over a 640-unit `viewBox`. On a 375px phone the content box is 293px, so the whole drawing was
 * multiplied by 0.46 — and the axis text, which carries its size as a presentation attribute in
 * user units, came out near **4.6px**. Half the size at which text is legible, on the viewport
 * most first visits arrive at.
 *
 * Scaling cannot be fixed by enlarging the type. Every gutter in this file is computed from the
 * text that goes in it, so doubling the font to survive the scale would need every gutter doubled
 * with it — on a frame that has not grown. The labels would collide instead of shrinking, which
 * is the same defect wearing different clothes.
 *
 * So the layout is recomputed at the width it will be shown at. `NARROW` is sized so that a phone
 * scales it by at least 0.9 rather than 0.46; `WIDE` is the width these forms were designed and
 * tuned at, unchanged. `renderToString` draws both and the stylesheet shows one — see
 * `.chart-pair` in `app.css` for the container query that picks, and why the swap is at 576px.
 *
 * The cost is real and was weighed: two SVGs per chart is about 31% more built HTML. It buys
 * legible axis text on the width most of this site's readers are on.
 */
export const WIDTHS = {
  /**
   * For a phone. 293px is the content box at 375px, the narrowest viewport worth drawing for, so
   * 320 is scaled by 0.92 there rather than shrunk.
   */
  narrow: 320,
  /** What every chart in this file was drawn at before there were two, and still is. */
  wide: 640,
} as const;

/**
 * A chart, as a function of the width it is drawn at.
 *
 * Required by both renderers rather than optional, on the same reasoning as {@link Naming}: a
 * builder that defaulted its width would silently draw the phone variant at desktop proportions,
 * and the two SVGs would be identical with nothing saying so. A caller cannot express that here.
 */
export type Drawing = (width: number) => Spec | null;

/**
 * Whether a drawing has anything to draw.
 *
 * Several cards render a chart only if there is one, and the surrounding prose — "one dot each,
 * poorest on the left" — is written for a chart that exists. The builders answer that with `null`,
 * and the decision is about the data every time: three values are not a distribution and two
 * points are not a series at any width. So this asks at one width and the answer stands for both.
 */
export function draws(drawing: Drawing): boolean {
  return drawing(WIDTHS.wide) != null;
}

/**
 * A gutter, bounded so it cannot eat the frame it is a margin of.
 *
 * The margins in this file are sized to their contents — the longest category name, the longest
 * direct label — which is right at 640 and is how a 260px label gutter arises. At 320 that same
 * gutter leaves 60px for the bars, so the chart becomes its own axis labels. This caps any one
 * gutter at a share of the width; a name that no longer fits is drawn shorter by Plot rather than
 * given the frame.
 *
 * 0.45 rather than a half: two gutters at the cap still leave a tenth of the frame for marks, and
 * no form here draws two capped gutters at once. The figure is what the widest direct label on the
 * site needs at `WIDTHS.narrow` — `the formula $10.13B` on `/history`, 135px including its offset
 * — rather than a round number chosen first and checked afterwards.
 */
function gutter(width: number, wanted: number): number {
  return Math.min(Math.round(wanted), Math.round(width * 0.45));
}

/**
 * How wide a string is drawn, in the ems Plot measures `lineWidth` in.
 *
 * Plot wraps text against its own character-width table, which averages about 0.5em. Measured
 * across the 241 text marks this site actually draws, the median is 0.519em and the ninetieth
 * percentile 0.667em — so a budget set from Plot's own estimate lets a label of capitals and wide
 * glyphs through at its nominal length and off the edge of the frame. `Career-technical education`
 * is 26 characters, fitted a 26.8-character budget, and painted 27px outside the viewBox.
 *
 * So the budget is set from the ninetieth percentile rather than the median: a label that wraps a
 * word early costs a line, and one that does not wrap at all is cut in half.
 */
const EM_PER_CHAR = 0.667;

/** A `lineWidth` in ems that holds a line to `px` pixels of drawn text. */
function lineWidth(px: number, fontSize = 10): number {
  return (px / fontSize) * (0.5 / EM_PER_CHAR);
}

/** How wide a string is drawn, in pixels, at the annotation type size. */
function textPx(text: string, fontSize = 11): number {
  return text.length * EM_PER_CHAR * fontSize;
}

/**
 * The annotation along the foot of a chart: an end of the scale at each corner, and between them
 * the one sentence the chart has to say about itself.
 *
 * Four forms drew this row and all four drew it the same way — three text marks at one `dy`,
 * anchored bottom-left, bottom and bottom-right. On a 640 frame the three fit. On a 320 one they
 * do not: `/history` drew `FY2009`, `axis starts at 32%, not zero` and `FY2022` across 186px of
 * frame and the centre ran through both years, so the chart's statement that its axis is truncated
 * — the whole reason that statement is on the chart rather than in the caption — arrived as a
 * smear of overlapping type.
 *
 * So the row measures itself. Where the three fit, nothing changes and the wide drawings are
 * byte-identical to what they were. Where they do not, the centre drops to its own line under the
 * two ends, and the caller is told how much more bottom margin that costs.
 */
function axisFoot(options: {
  width: number;
  marginLeft: number;
  marginRight: number;
  dy: number;
  /** The low end of the scale, at the left corner. */
  low: string;
  /** What the chart says about itself, between them. */
  says: string;
  /** The high end, at the right corner. */
  high: string;
}): { marks: Plot.Markish[]; extraBottom: number } {
  const { width, marginLeft, marginRight, dy, low, says, high } = options;
  const frame = width - marginLeft - marginRight;
  // A gap either side of the centre, so "fits" means legibly rather than exactly.
  const fits = textPx(low) + textPx(says) + textPx(high) + 24 <= frame;
  /*
   * The drop to a second line, and why it is not the type size.
   *
   * This was 13, which is what an 11px line occupies — so the two rows were laid exactly touching
   * and the gap between them was whatever the font happened to leave. On the machine this was
   * written on that was 0.1px; under DejaVu Sans, which is what `system-ui` resolves to on a great
   * many Linux and Android readers, the glyph boxes are a pixel taller and the rows overlap. The
   * chart's statement about its own truncated axis then runs through the year labels — the exact
   * defect this function exists to prevent, moved from one font to another.
   *
   * 16 leaves three user units of clear space at the tallest metrics measured across `system-ui`,
   * DejaVu Sans, Liberation Sans, Arial, Verdana and Tahoma.
   */
  const line = 16;
  const at = (anchor: "bottom-left" | "bottom" | "bottom-right", y: number, text: string) =>
    Plot.text([0], {
      frameAnchor: anchor,
      dy: y,
      text: () => text,
      ...(anchor === "bottom-left" ? { textAnchor: "start" as const } : {}),
      ...(anchor === "bottom-right" ? { textAnchor: "end" as const } : {}),
      fill: INK.muted,
      fontSize: 11,
    });
  return {
    marks: [
      at("bottom-left", dy, low),
      at("bottom-right", dy, high),
      at("bottom", fits ? dy : dy + line, says),
    ],
    extraBottom: fits ? 0 : line,
  };
}

/**
 * A horizontal bar chart: magnitude compared across a handful of named categories.
 *
 * Horizontal because the categories are text and vertical bars would need rotated labels, which
 * are harder to read than they are worth. Direct labels are selective — Plot is given only the
 * bars that asked for one, never a number on every mark.
 */
export function barSpec(bars: Bar[], options: { width: number; max?: number }): Spec {
  const { width } = options;
  const max = options.max ?? Math.max(...bars.map((b) => Math.abs(b.value)), 1);
  const labelled = bars.filter((b) => b.direct != null);
  /*
   * A negative value is drawn as a negative value, on the one chart in the build that has one.
   *
   * This mark took `Math.abs(b.value)` and filled every bar with the same colour, so a deficit
   * and a surplus of the same size were the same picture. Springfield Local held
   * −$2,812,534 at 30 June FY2021 and its bar was indistinguishable from a $2.8M surplus —
   * one bar out of the whole site, which is why nobody caught it. Its own tooltip said
   * `$-2,812,534` while the bar said the opposite.
   *
   * Signed mode is entered only when a value is actually below zero, so the eight other charts
   * built on this spec keep their exact geometry: same domain, same rounding, same fill. Inside
   * it the bar runs from zero to the value, the fill takes the polarity pair the palette already
   * licenses for gain against loss, and a rule marks the baseline the bars now sit on both sides
   * of. The rounded data end goes, because with two directions a single `rx2` would round the
   * baseline of a negative bar and square its data end — the opposite of what the rounding means.
   */
  const lowest = Math.min(0, ...bars.map((b) => b.value));
  const signed = lowest < 0;
  const negativeLabelled = labelled.filter((b) => b.value < 0);
  // A direct label on a negative bar is written leftwards from the bar's end, so the domain gets
  // room for it rather than letting it collide with the category names outside the frame.
  const floor = signed ? lowest - (negativeLabelled.length > 0 ? (max - lowest) * 0.08 : 0) : 0;
  const longest = Math.max(0, ...bars.map((b) => b.direct?.length ?? 0));
  // Sized to the longest category name, as the right gutter is sized to the longest direct label.
  // This was a fixed 160, which silently clipped anything longer — "Building leadership and
  // operation" rendered as "g leadership and operation", which reads as a rendering fault rather
  // than a truncation and is exactly the kind of thing nobody reports.
  const longestLabel = Math.max(0, ...bars.map((b) => b.label.length));
  /*
   * The name gutter, and what it costs when the frame is a phone wide.
   *
   * The width a name wants does not shrink with the frame — the type is the same size either way
   * — so at `WIDTHS.narrow` the 241px "Building leadership and operation" asks for three quarters
   * of the chart. Capped, it gets 42% and wraps inside it; `lineWidth` is what makes Plot wrap
   * rather than run the name off the left edge of the viewBox, where it is simply cut in half.
   *
   * The row then has to grow to hold two lines, which is why `rowHeight` is decided here rather
   * than at the top of the function. See {@link EM_PER_CHAR} for why the wrap budget is not simply
   * the gutter divided by the type size.
   */
  const wanted = Math.max(120, Math.min(260, Math.round(longestLabel * 7.1) + 14));
  const nameGutter = gutter(width, wanted);
  const wraps = nameGutter < wanted;
  const rowHeight = wraps ? 40 : 30;

  /*
   * The bar the chart was built to locate, on two channels.
   *
   * The national chart on `/statewide` ranks the states by local share and draws Ohio among them.
   * It set `current: true` on Ohio's bar; `Bar` did not declare the field and this function did
   * not read it, so **Ohio carried no mark in a chart built to show Ohio's position** and a reader
   * had to find it by reading the category names — which is the work the chart was drawn to save.
   *
   * Colour is one channel and never the only one, the rule the print stylesheet applies to every
   * ground-encoded mark on the site. So the fill takes the contrasting half of the validated pair
   * *and* the category name is drawn in primary ink at 600 rather than in secondary at normal
   * weight. Weight survives a monochrome print and a forced-colours mode, which is the point.
   *
   * In signed mode the fill already carries polarity — a deficit against a surplus — and a hue
   * cannot mean two things at once, so there the subject bar keeps the label channel alone. Only
   * one chart in the build is signed and none of its bars is a subject.
   */
  const marked = bars.filter((b) => b.current);
  const plain = bars.filter((b) => !b.current);

  return {
    options: {
      width,
      height: bars.length * rowHeight,
      // Bounded below so short-label charts keep their existing proportions, and above so a very
      // long name costs the bars width rather than running off the plot.
      marginLeft: nameGutter,
      // Room at the right for the longest direct label actually present. Without it the largest
      // bar's value runs off the viewBox and is clipped — and it is the one most worth reading.
      marginRight: gutter(width, longest > 0 ? 16 + longest * 7.2 : 20),
      marginTop: 0,
      marginBottom: 0,
      x: { axis: null, domain: [floor, max] },
      y: { axis: null, domain: bars.map((b) => b.label), padding: 0.47 },
      marks: [
        Plot.barX(bars, {
          y: "label",
          // One mark, not two: `attachHovers` maps tooltips onto `.bar-fill > *` by index and
          // throws if the counts disagree, so splitting positives from negatives here would
          // reorder the marks out from under the hover layer.
          ...(signed
            ? { x1: 0, x2: (b: Bar) => b.value }
            : { x: (b: Bar) => Math.abs(b.value) }),
          // A constant where the chart has no subject, and not merely as an optimisation: Plot
          // hoists a constant fill onto the group and pushes a channel down onto each rect, so a
          // function here would move where the colour lives on every one of the nine charts built
          // on this spec — including the two the theme tests read `.bar-fill`'s computed fill from.
          fill: signed
            ? (b: Bar) => (b.value < 0 ? SERIES.guarantee : SERIES.formula)
            : marked.length > 0
              ? (b: Bar) => (b.current ? SERIES.guarantee : SERIES.formula)
              : SERIES.formula,
          className: "bar-fill",
          // Rounded at the data end, square at the baseline: the bar grows from the axis and
          // rounding that end would detach it from the thing it is measured against. Not in
          // signed mode — see above.
          ...(signed ? {} : { rx2: 4 }),
        }),
        ...(signed ? [Plot.ruleX([0], { stroke: INK.rule })] : []),
        Plot.text(plain, {
          y: "label",
          frameAnchor: "left",
          dx: -10,
          text: "label",
          textAnchor: "end",
          fill: INK.secondary,
          lineWidth: lineWidth(nameGutter),
          className: "bar-label",
        }),
        // The second channel. Split into its own mark because `fill` and `fontWeight` are
        // constants in Plot rather than channels; text is not in the hover selector, so unlike the
        // fill above these may be split without reordering anything the tooltips index into.
        ...(marked.length > 0
          ? [
              Plot.text(marked, {
                y: "label",
                frameAnchor: "left",
                dx: -10,
                text: "label",
                textAnchor: "end",
                fill: INK.primary,
                fontWeight: 600,
                lineWidth: lineWidth(nameGutter),
                className: "bar-label current",
              }),
            ]
          : []),
        // `dx` and `textAnchor` are constants in Plot rather than channels, so a chart with bars
        // on both sides of zero needs one text mark per direction. Text is not in the hover
        // selector, so unlike the fill above these may be split.
        Plot.text(signed ? labelled.filter((b) => b.value >= 0) : labelled, {
          y: "label",
          x: (b: Bar) => (signed ? b.value : Math.abs(b.value)),
          dx: 8,
          text: "direct",
          textAnchor: "start",
          fill: INK.primary,
          className: "bar-value",
        }),
        ...(signed && negativeLabelled.length > 0
          ? [
              Plot.text(negativeLabelled, {
                y: "label",
                x: (b: Bar) => b.value,
                dx: -8,
                text: "direct",
                textAnchor: "end",
                fill: INK.primary,
                className: "bar-value",
              }),
            ]
          : []),
      ],
    },
    hovers: {
      selector: ".bar-fill > *",
      text: bars.map((b) => escapeHtml(b.hover ?? `${b.label}: ${b.value}`)),
      cursor: { second: "the mark itself" },
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
    width: number;
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
    /**
     * Draw the x axis on this domain instead of on the points' own range.
     *
     * For a **small multiple**: two clouds a card asks a reader to compare across. The spending
     * pair on `/outcomes` is drawn from the same numerator over two denominators, and its own
     * prose says the bands "separate on the horizontal axis too" — a statement about horizontal
     * distance. Fitted to their own ranges those two axes differed by **1.64×**, so part of the
     * separation the sentence points at was the scale rather than the data.
     *
     * The y axis needs no equivalent: it is already shared wherever it matters, because the two
     * charts plot the same measure and their ranges coincide. This exists for the axis where the
     * two denominators genuinely differ, which is the one a reader must not read as data.
     */
    xDomain?: [number, number];
  },
): Spec | null {
  // Two points are not a cloud. Same rule as the line forms, for the same reason: a scatter of
  // three districts would read as a finding about a population that has not been measured.
  if (points.length < 12) return null;

  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const pad = (lo: number, hi: number) => (hi - lo) * 0.04 || Math.abs(hi) * 0.02 || 1;
  const both = options.identity != null;
  const xMin = both ? Math.min(...xs, ...ys) : (options.xDomain?.[0] ?? Math.min(...xs));
  const xMax = both ? Math.max(...xs, ...ys) : (options.xDomain?.[1] ?? Math.max(...xs));
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
  const { width } = options;
  const marginLeft = 62;
  /*
   * Sized to the labels that are actually drawn, which is not every trace.
   *
   * A banded trace carries its identity in the legend and this function deliberately draws no end
   * label for it — see the trace marks below. The gutter was sized off `traces` all the same, so
   * both banded scatters on `/outcomes` gave up **22% of a 640px frame** to labels that were never
   * rendered: three bands whose longest name is "least poor third", 139px of white space beside a
   * cloud that had been squeezed to make room for it.
   */
  const labelledTraces = traces.filter((t) => t.band == null);
  const marginRight = gutter(
    width,
    labelledTraces.length > 0
      ? 24 + Math.max(...labelledTraces.map((t) => t.label.length)) * 7.2
      : 24,
  );
  const marginTop = 28;
  const foot = axisFoot({
    width,
    marginLeft,
    marginRight,
    dy: 20,
    low: axes.x.format(xMin),
    says: axes.x.label + (axes.x.log ? " (log scale)" : ""),
    high: axes.x.format(xMax),
  });
  const marginBottom = 40 + foot.extraBottom;

  /*
   * Square where the identity line is drawn, so that y = x is drawn at 45°. Everywhere else the
   * caller's height, or a default that suits a wide cloud.
   */
  const height = options.identity
    ? width - marginLeft - marginRight + marginTop + marginBottom
    : (options.height ?? 420);
  return {
    options: {
      width,
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
        ...foot.marks,
        Plot.text([0], {
          frameAnchor: "top-left",
          dx: -marginLeft + 4,
          // Clear of the axis name above it. Both sat on the frame's top edge, so
          // "Performance Index" and the 113 it labels were drawn through each other at every
          // width — the one collision in this file that predates there being two of them.
          dy: 3,
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
      cursor: { second: "paired marks", layers: [".scatter-dot"] },
    },
  };
}

/**
 * The end dots of a range row. Named because the hit band has to know it: a band that stops where
 * the dot's centre is leaves the dot's outer half outside its own hit area.
 */
const DOT_RADIUS = 3;

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
  options: { width: number },
): Spec | null {
  if (rows.length < 2) return null;

  const values = rows.flatMap((r) => [r.low, r.high]);
  const min = Math.min(...values);
  const max = Math.max(...values);

  const rowHeight = 14;
  const longest = Math.max(...rows.map((r) => r.label.length));
  // A 14px row cannot hold a second line, so this gutter is capped rather than wrapped: a name
  // too long for a phone's frame is drawn shorter, not folded into the row below it.
  const { width } = options;
  const marginLeft = gutter(width, Math.max(70, Math.min(150, Math.round(longest * 6.2) + 10)));
  const foot = axisFoot({
    width,
    marginLeft,
    marginRight: 16,
    dy: 16,
    low: axis.format(min),
    says: axis.label + (axis.log ? " (log scale)" : ""),
    high: axis.format(max),
  });

  return {
    options: {
      width,
      height: rows.length * rowHeight,
      marginLeft,
      marginRight: 16,
      marginTop: 0,
      marginBottom: 22 + foot.extraBottom,
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
          r: DOT_RADIUS,
          fill: ORDINAL[0],
          stroke: "none",
          className: "range-low",
        }),
        Plot.dot(rows, {
          y: "label",
          x: "high",
          r: DOT_RADIUS,
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
        ...foot.marks,
        /*
         * One band per row, above everything: the hit target is the row, not the 3px dot at
         * either end of it.
         *
         * The negative insets are the fix for #198 and are not cosmetic. The band ran `min` to
         * `max`, so the row holding the maximum had its high dot **centred on the band's right
         * edge** and overhanging it by the dot's radius — measured at 5.08 device pixels on
         * `/counties`, which is `DOT_RADIUS` times the 1.69 that SVG is scaled up by.
         *
         * Two things followed from that. The dot's outer half was not hoverable, on the one row
         * a reader is most likely to point at. And the keyboard cursor's outline, drawn 1px
         * outside this band, ran straight through a **full-opacity `--ordinal-3` dot** — the one
         * place on the site where the cursor is adjacent to the single mark the ink does not
         * clear 3:1 against. See the cursor rule in `app.css`.
         *
         * Insets rather than a padded domain, because this chart's scale is sometimes log and
         * padding a log domain by a share of its span is not a thing the span means. An inset is
         * pixels, and the overhang is pixels.
         */
        Plot.rect(rows, {
          y: "label",
          x1: min,
          x2: max,
          insetLeft: -DOT_RADIUS,
          insetRight: -DOT_RADIUS,
          fill: "transparent",
          className: "range-hit",
        }),
      ],
    },
    hovers: {
      selector: ".range-hit > *",
      text: rows.map((r) => escapeHtml(r.hover)),
      cursor: { second: "paired marks", layers: [".range-low", ".range-high"] },
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
 * The `q`th quantile of an ascending series by **nearest rank**, or zero for an empty one.
 *
 * Deliberately not `stats.percentile`, which interpolates and is the definition every median this
 * site *publishes* now uses. A box plot is different in kind: its rule and its hinges are drawn at
 * observations, and a caller describing the box in prose has to name the same three values the
 * marks sit on. Interpolating here would place the rule between two dots and then describe it as
 * a figure the population does not contain.
 *
 * Exported for exactly that reason. `district.ts` writes the `description` for the strip charts —
 * *"Quartiles run $X to $Y, median $Z"* — and had its own copy of this expression, which is a
 * sentence and a mark agreeing by coincidence rather than by construction. See `stats.ts` on the
 * distinction between the two conventions and why this repository now keeps both, named.
 */
export function nearestRank(sorted: number[], q: number): number {
  if (sorted.length === 0) return 0;
  return sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))]!;
}

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
    width: number;
    /** The one this page is about. Drawn last, above every other mark. */
    marker?: { value: number; label: string } | null;
    /**
     * Draw every value rather than only the outliers.
     *
     * Defaults on up to {@link DOTS_UP_TO}. Pass it explicitly only to override that for a reason
     * the population size does not carry.
     */
    dots?: boolean;
  },
): Spec | null {
  // Two values are a pair, not a distribution. A box drawn over them would put quartiles on a
  // population that has none, which reads as a finding about a spread nobody measured.
  if (values.length < 3) return null;

  const sorted = [...values].sort((a, b) => a.value - b.value);
  const at = (q: number) => nearestRank(sorted.map((v) => v.value), q);
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
  /*
   * A signed distribution states where zero is, on the same terms `histogramSpec` does.
   *
   * Five of the six strips on `/districts` measure a quantity that cannot be negative — aid,
   * valuation, a poverty share — and for those the left edge is the smallest value and nothing is
   * being crossed. The sixth is enrollment change, the site's one signed distribution, and it drew
   * no zero: a dot two thirds along could have been a district that grew or one that shrank, and
   * the strip carried nothing to say which. `histogramSpec` draws a dashed rule at zero and labels
   * it "no change" for exactly that reason, and gives a bin straddling it a neutral fill.
   *
   * Detected from the domain rather than passed in, so it appears wherever the condition holds and
   * cannot be forgotten at a call site. The rule is dashed and in muted ink: it is a reference, not
   * a value, and must not be confused with the marker rule, which is solid, hued and full height.
   */
  const min0 = sorted[0]!.value;
  const max0 = sorted[sorted.length - 1]!.value;
  const crossesZero = min0 < 0 && max0 > 0;
  // Room under the strip for the label, and only where there is a label — the five unsigned strips
  // keep their exact geometry, which is what the six of them being one row apart depends on.
  const height = crossesZero ? 64 : 46;
  const mid = 0;
  const outliers = dots ? [] : sorted.filter((v) => v.value < whiskerLow || v.value > whiskerHigh);
  // Five lanes, so a run of equal values is countable rather than one mark. Deterministic: this
  // module is pure, and a jitter that moved between builds would redraw one county two ways.
  const lane = (i: number) => ((i % 5) - 2) * 4.2;
  const drawn = dots ? sorted : outliers;

  return {
    options: {
      width: options.width,
      height,
      marginLeft: 2,
      marginRight: 2,
      marginTop: 4,
      marginBottom: crossesZero ? 22 : 4,
      x: { axis: null, domain: [min - pad, max + pad] },
      y: { axis: null, domain: [-19, 19] },
      marks: [
        // Under everything, because it is what the dots are read against rather than a mark
        // among them.
        ...(crossesZero
          ? [
              Plot.ruleX([0], { stroke: INK.muted, strokeDasharray: "3 3" }),
              Plot.text([0], {
                x: 0,
                frameAnchor: "bottom",
                dy: 15,
                text: () => "no change",
                fill: INK.muted,
                fontSize: 11,
              }),
            ]
          : []),

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
      cursor: { second: "paired marks", layers: [".dist-dot"] },
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
export function histogramSpec(
  bins: Bin[],
  format: (v: number) => string,
  options: { width: number },
): Spec {
  const first = bins[0]!;
  const last = bins[bins.length - 1]!;
  const crossesZero = first.from < 0 && last.to > 0;

  const side = (b: Bin) =>
    b.to <= 0 ? SERIES.guarantee : b.from >= 0 ? SERIES.formula : SERIES.neutral;

  return {
    options: {
      width: options.width,
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
      cursor: { second: "the mark itself" },
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
  options: { width: number },
): Spec | null {
  // One point is not a series. Returning null draws nothing rather than a degenerate axis with a
  // single mark on it, which would read as a finding about a quantity that has not been measured
  // twice.
  if (points.length < 2) return null;
  const references = points.map((p) => p.reference).filter((v): v is number => v != null);
  // Every year or none. A reference line drawn across a partial series would bridge the years it
  // has no value for, which is the same claim `seriesSpec` refuses to make about a missing year.
  const hasReference = references.length === points.length;

  /*
   * The domain fits the marks that are drawn, and the reference is one of those only when it is
   * drawn in full.
   *
   * It was folded in unconditionally, so a series carrying references for *some* years stretched
   * its y axis to contain values that never reached the frame — and this axis is truncated to the
   * band's own range precisely because the band is narrow. Padding it out for an invisible value
   * flattens the one thing the chart is for.
   */
  const inDomain = hasReference ? references : [];
  let min = Math.min(...points.map((p) => p.low), ...inDomain);
  let max = Math.max(...points.map((p) => p.high), ...inDomain);
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

  // The two bound labels live here. Capped like every other gutter, because 104px is a sixth of
  // the wide frame and a third of the narrow one.
  const marginRight = gutter(options.width, 104);
  const foot = axisFoot({
    width: options.width,
    marginLeft: 0,
    marginRight,
    dy: 18,
    low: `FY${points[0]!.year}`,
    // The truncated axis, stated on the chart rather than in the caption underneath it. A reader
    // who takes the shape at face value has been misled by the time they reach prose.
    says: `axis starts at ${format(min)}, not zero`,
    high: `FY${last.year}`,
  });

  return {
    options: {
      width: options.width,
      height: 220,
      marginTop: 14,
      marginBottom: 26 + foot.extraBottom,
      marginLeft: 0,
      marginRight,
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
        ...foot.marks,
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
      cursor: {
        second: "none",
        because:
          "A full-height column over a continuous line. The band and both series are paths, so there is no per-year mark to brighten, and the outline is the height of the plot.",
      },
    },
  };
}

/**
 * The truncated domain a line form draws on: the values' range, padded, never zero-based.
 *
 * Exported because the annotation that states it — *"axis starts at $1.24B, not zero"* — is the
 * only mark on the chart that says how far from zero the frame begins, and it is rendered with the
 * **caller's** format. A format with too few places for its own axis start does not merely round:
 * it understates the truncation, which is the specific way this annotation can be worse than
 * absent. `$1bn` for an axis starting at $1.24bn understated it by a fifth on the appropriations
 * chart, in an invented unit that appears nowhere else on the site.
 *
 * A caller cannot check that against a number this function used to keep to itself, so it does not
 * keep it: `appropriations.spec.ts` asserts the annotation round-trips to within a percent of what
 * it annotates.
 */
export function truncatedDomain(values: number[]): [number, number] {
  const low = Math.min(...values);
  const high = Math.max(...values);
  const pad = (high - low) * 0.12 || Math.abs(high) * 0.02 || 1;
  return [low - pad, high + pad];
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
  options: { width: number },
): Spec | null {
  // One point is not a series, exactly as in `fanSpec`.
  if (points.length < 2) return null;
  const values = points.flatMap((p) => [p.a, p.b]).filter((v): v is number => v != null);
  if (values.length === 0) return null;

  const [min, max] = truncatedDomain(values);

  const first = points[0]!;
  const last = points[points.length - 1]!;
  // The direct label goes on the last year that has a value, which is not necessarily the last
  // year: a series ending in a gap would otherwise be labelled at a point it does not occupy.
  const endOf = (key: "a" | "b") => [...points].reverse().find((p) => p[key] != null);
  const endA = endOf("a");
  const endB = endOf("b");
  /** What the end labels say, which is what the right gutter has to hold. */
  const endText = (point: SeriesPoint | undefined, key: "a" | "b"): string =>
    point ? `${key === "a" ? labels.a : labels.b} ${format(point[key] ?? 0)}` : "";
  const longestEnd = Math.max(endText(endA, "a").length, endText(endB, "b").length);

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
            text: () => endText(point, key),
            textAnchor: "start",
            fill: stroke,
            className: "series-end",
          }),
        ]
      : [];

  /*
   * Room for the longer of the two direct labels, which carry the series identity.
   *
   * Sized to the string actually drawn and not to the series name alone: the label reads
   * `unclosed $4,229`, and sizing off `unclosed` cut the gutter short by the width of the number.
   * At 640 the shortfall was absorbed by the slack in the 7.2px-per-character estimate; at
   * `WIDTHS.narrow` it put both of `/history`'s end labels outside the frame.
   */
  const marginRight = gutter(options.width, 32 + longestEnd * 7.2);
  const foot = axisFoot({
    width: options.width,
    marginLeft: 0,
    marginRight,
    dy: 18,
    low: `FY${first.year}`,
    says: `axis starts at ${format(min)}, not zero`,
    high: `FY${last.year}`,
  });

  return {
    options: {
      width: options.width,
      height: 220,
      marginTop: 14,
      marginBottom: 26 + foot.extraBottom,
      marginLeft: 0,
      marginRight,
      x: { axis: null, domain: [first.year, last.year] },
      y: { axis: null, domain: [min, max] },
      marks: [
        line("a", SERIES.formula, "series-a"),
        line("b", SERIES.guarantee, "series-b"),
        ...endLabel(endA, "a", SERIES.formula),
        ...endLabel(endB, "b", SERIES.guarantee),
        Plot.ruleY([min], { stroke: INK.rule, className: "axis" }),
        ...foot.marks,
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
      cursor: {
        second: "none",
        because:
          "A full-height column over two continuous lines, as the fan chart is. Nothing per-year exists to brighten.",
      },
    },
  };
}
