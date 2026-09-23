/**
 * The nine chart forms, as Observable Plot specifications.
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
  Fit,
  Place,
  Range,
  Rank,
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

/** The interval between two panels of a small multiple: `--space-6`, 1.1rem, at the 16px root. */
const PANEL_GAP = 18;

/**
 * The narrowest a panel is laid out at before the row wraps: `.panel`'s flex basis in `app.css`.
 *
 * The two numbers have to agree. This function works out the width a panel is *drawn* at and the
 * stylesheet works out the width it is *shown* at, and a drawing wider than its box is scaled down
 * by the same `width: 100%` that {@link WIDTHS} exists to stop.
 */
const PANEL_MIN = 280;

/**
 * The width one panel of an `n`-panel small multiple is drawn at.
 *
 * A panel is laid out against its sibling rather than against the page: two of them share the
 * wide frame, so each is a little under half of it, and on a phone they stack and each is the
 * narrow frame. Both of those are near enough {@link WIDTHS.narrow} that a panel is drawn **once**
 * rather than at two widths — which is the reason this is a function and not a call to
 * `renderToString`. The cloud these draw is 609 dots; a second copy for a layout it is never shown
 * in would be the largest dead mark in the built HTML.
 *
 * # Why `n` is not the divisor
 *
 * `.panels` wraps, and it wraps at {@link PANEL_MIN}. Seven panels do not share one row — they
 * make four — so dividing the frame by seven would draw each at 76px and the stylesheet would
 * then show it at 311, scaled up by four and blurred, with its 11px type at 45. The divisor is
 * therefore how many actually fit on a row, which for one and for two panels is one and two: the
 * cloud drawn before there was a third form is unchanged to the pixel.
 */
export function panelWidth(panels: number): number {
  const perRow = Math.max(
    1,
    Math.min(panels, Math.floor((WIDTHS.wide + PANEL_GAP) / (PANEL_MIN + PANEL_GAP))),
  );
  return Math.floor((WIDTHS.wide - PANEL_GAP * (perRow - 1)) / perRow);
}

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
 * byte-identical to what they were. Where they do not, the centre drops to a line of its own under
 * the two ends — and stops being a centre, for the reason given where it is drawn — and the caller
 * is told how much more bottom margin that costs.
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
  /*
   * The dropped line starts at the axis rather than staying centred, and that is not cosmetic.
   *
   * `frameAnchor: "bottom"` is the centre of the *plot frame*, which is only the centre of the
   * drawing where the two margins match. `seriesSpec` draws its direct labels in a right gutter
   * and nothing in a left one, so on a 311px panel the frame is 0..171 and a centred annotation
   * hangs 2px off the left edge of the SVG — measured on the runner, where `system-ui` is wider
   * than it is here, and invisible on this machine.
   *
   * Nudging it by half the margin difference would fix that panel and leave the next asymmetric
   * one to be discovered the same way, because the correction would be sized on an estimate of
   * painted width and the estimate is the thing that was wrong. A line of its own has the whole
   * width available and no reason to be centred in anything: anchored to the frame's left edge it
   * sits under the low end of the scale it is talking about, and where it starts stops depending
   * on which font the reader has.
   */
  return {
    marks: [
      at("bottom-left", dy, low),
      at("bottom-right", dy, high),
      fits ? at("bottom", dy, says) : at("bottom-left", dy + line, says),
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
 *
 * # `max` and `min`, which exist for small multiples and for nothing else
 *
 * A single chart fits its domain to its own bars and neither option is passed. Seven panels of
 * one grouped series are a different object: the shared axis is the whole reason they are drawn
 * side by side rather than as seven charts, and a panel scaled to itself says its largest bar is
 * as large as the largest bar anywhere. So a caller drawing a small multiple computes both ends
 * over **every** panel's rows and hands the same pair to each.
 *
 * `min` also decides signed mode, which is why it is a floor rather than a clamp: one panel of
 * the anchor-rule multiple is all zeroes, and fitted to itself it would draw an unsigned frame
 * with no zero rule in it — the one panel of seven whose baseline is not where the others' is.
 */
export function barSpec(
  bars: Bar[],
  options: { width: number; max?: number; min?: number; labelChars?: number },
): Spec {
  const { width } = options;
  const max = options.max ?? Math.max(...bars.map((b) => Math.abs(b.value)), 1);
  const labelled = bars.filter((b) => b.direct != null);
  /*
   * A negative value is drawn as a negative value, on the charts in the build that have one.
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
  const lowest = Math.min(options.min ?? 0, 0, ...bars.map((b) => b.value));
  const signed = lowest < 0;
  const negativeLabelled = labelled.filter((b) => b.value < 0);
  // A direct label on a negative bar is written leftwards from the bar's end, so the domain gets
  // room for it rather than letting it collide with the category names outside the frame.
  const floor = signed ? lowest - (negativeLabelled.length > 0 ? (max - lowest) * 0.08 : 0) : 0;
  /*
   * How much room the direct labels get at the right — this chart's own longest, or the set's.
   *
   * A set of panels shares a scale, and a scale is a domain *and* a frame. The right gutter is
   * sized to the labels a panel actually carries, so a panel that direct-labels its rows when its
   * siblings do not gets ten pixels less frame than they do, and everything inside it — the zero
   * rule most visibly — lands at a different pixel from the same value in the panel beside it.
   * The baselines of panels drawn side by side then fail to line up, which is the one thing the
   * shared scale exists to guarantee. So a caller drawing a set passes the widest label anywhere
   * in the set and every panel reserves the same room, whether it has a label in it or not.
   */
  const longest = options.labelChars ?? Math.max(0, ...bars.map((b) => b.direct?.length ?? 0));
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
   * cannot mean two things at once, so there the subject bar keeps the label channel alone. Two
   * charts in the build are signed — Springfield's balance, and the correlation-by-class column a
   * corpus node draws from `crates/series.json` — and none of their bars is a subject.
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
 * The fewest points this form will draw a cloud of.
 *
 * Exported because callers reason about it. `/reach` cannot subset its cloud to a county — 79 of
 * Ohio's 88 counties hold fewer districts than this — and the page says so in prose, so the number
 * it says has to be this number rather than a second copy of it.
 */
export const MIN_CLOUD = 12;

/**
 * How a scatter's dots are drawn, and the one place the three alphas are written down.
 *
 * `banded` is more opaque than `plain` because banded dots carry a third measure and have to be
 * legible one against another, where the neutral cloud only has to be legible against the card.
 *
 * `muted` is the de-emphasised mark — see `ScatterPoint.muted` — and it spends **two** channels
 * rather than one. Measured against `--surface-1`, a neutral dot is 1.42:1 light and 1.65:1 dark at
 * `plain`; at the muted alpha it is 1.18 and 1.25, which separates the two populations by only 1.20
 * and 1.32. That is not a difference a reader picks out of six hundred overlapping dots, so the
 * radius comes down too: 1.6 against 2.4 is 44% of the area, and alpha and area together leave
 * about a fifth of the ink. `palette.spec.ts` holds those figures, and holds the ordering — a muted
 * mark that stopped being the fainter of the two would be a de-emphasis that emphasised.
 *
 * The bar is separation from the un-muted mark and visibility above the surface, not 3:1. Nothing
 * in this cloud has ever met 3:1: `--neutral-mark` is 2.35:1 light against `--surface-1` at full
 * opacity, before the 0.45 the form draws it at. Small, partly transparent marks are what make
 * density legible as density here, and the relief for that is the legend the banded forms carry.
 */
export const DOT = {
  radius: { plain: 2.4, muted: 1.6 },
  opacity: { banded: 0.62, plain: 0.45, muted: 0.22 },
} as const;

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
/**
 * The two numbers a fitted panel prints, and the only two it does.
 *
 * Four places, because that is where the corpus writes them and where `crates/figures.json` pins
 * them — a reader taking `0.9855` off the picture is taking the figure, not a rounding of it. The
 * typographic minus rather than the hyphen, as the rest of the site's numerals use, and no plus:
 * a positive slope is the unmarked case and the line already says which way it goes.
 */
function printedFit(fit: Fit): string {
  const slope = `${fit.slope < 0 ? "\u2212" : ""}${Math.abs(fit.slope).toFixed(4)}`;
  return `slope ${slope} \u00b7 r\u00b2 ${fit.rSquared.toFixed(4)}`;
}

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
     * The y axis needs an equivalent now, and the second reason is stronger than the first. On
     * `/reach` the cloud is redrawn on every slider tick, and an axis fitted to the run it is
     * drawing refits with it — so dragging base cost holds the points still and moves the *frame*,
     * which is the exact reading a movement chart must not produce. A domain the caller has fixed
     * across the whole reachable lever space makes a district that did not move look like a
     * district that did not move.
     *
     * A supplied domain is **authoritative**, and the marks are clipped to it. The first version
     * made it a floor unioned with the data, on the theory that a bound which turned out too small
     * should grow rather than clip silently — and that defeated the whole purpose: Kelleys Island
     * Local has four pupils and $26,700 of aid per pupil, so the union put the axis back on the
     * outlier at every tick and the frame moved exactly as before. Clipping is not silent where
     * the caller counts what it clipped, which is what `/reach` does.
     *
     * `identity` shares the two across both axes, for the reason its own note gives.
     */
    xDomain?: [number, number];
    /** As `xDomain`, for the vertical axis. */
    yDomain?: [number, number];
    /**
     * Draw a line the crates fitted, and print its slope and r-squared.
     *
     * The second reference line this form draws, and the note on `identity` above is the argument
     * for why there is not a third: an arbitrary line through a cloud is a claim, and the claims
     * on this site are computed in `crates/` with a checkpoint behind them. This one is. The
     * segment arrives in the data's own units from `crates/series.json`, its two numbers are
     * ordinary pinned figures the corpus node quotes, and nothing here fits anything — see
     * {@link Fit}.
     *
     * Both domains must be supplied, and this **throws** when they are not. A fitted line is an
     * angle, and an angle is a statement only if the frame is the one the slope was computed in:
     * a y axis fitted to its own points redraws a slope of one at whatever angle the data's range
     * happens to give, which was measured at 1.296 on the first version of the pair this was
     * written for. The frame is the crate's, and the plot area is squared for the same reason
     * `identity` squares it, so that the units per pixel are equal on the two axes.
     *
     * The slope and the r-squared are the only quantities printed *on* the cloud, which is the
     * scatter's form of the endpoint rule the bars have: what a reader can take off a chart has to
     * be a number the prose states and a figure pins. The four corner numbers are the frame rather
     * than the finding — they say where the cloud sits, and a reader who quotes one has quoted an
     * axis end.
     */
    fit?: Fit;
  },
): Spec | null {
  // Two points are not a cloud. Same rule as the line forms, for the same reason: a scatter of
  // three districts would read as a finding about a population that has not been measured.
  if (points.length < MIN_CLOUD) return null;

  // Both ends of a displacement, so a trail cannot run out of the frame it is drawn in.
  const xs = points.flatMap((p) => (p.from ? [p.x, p.from.x] : [p.x]));
  const ys = points.flatMap((p) => (p.from ? [p.y, p.from.y] : [p.y]));
  const pad = (lo: number, hi: number) => (hi - lo) * 0.04 || Math.abs(hi) * 0.02 || 1;
  /* A supplied domain wins outright; the marks are clipped to it. See `xDomain`. */
  const xLo = options.xDomain?.[0] ?? Math.min(...xs);
  const xHi = options.xDomain?.[1] ?? Math.max(...xs);
  const yLo = options.yDomain?.[0] ?? Math.min(...ys);
  const yHi = options.yDomain?.[1] ?? Math.max(...ys);
  if (options.fit && (options.xDomain == null || options.yDomain == null)) {
    throw new Error(
      `A fitted line was given without both domains. The line was fitted in a frame the crate ` +
        `computed and drawing it in a frame fitted to these points would redraw its slope; pass ` +
        `the manifest's own x and y bounds. See the note on scatterSpec's fit option.`,
    );
  }
  const both = options.identity != null;
  const xMin = both ? Math.min(xLo, yLo) : xLo;
  const xMax = both ? Math.max(xHi, yHi) : xHi;
  const yMin = both ? xMin : yLo;
  const yMax = both ? xMax : yHi;
  /*
   * A fitted axis is padded so the extreme points are not on the frame. A supplied one is not.
   *
   * The padding is a tenth of the reason a fitted axis reads well and the whole of the reason a
   * supplied one would not: a caller who asks for `[0, 19175]` because a dollar figure is read as
   * a height above zero gets `[-767, 19942]` instead, and the baseline floats above the axis line
   * under a label still reading `$0`. The caller chose the bound; this draws it.
   *
   * With `identity` the two axes are one domain, so a domain supplied for either end fixes both.
   */
  const supplied = both
    ? options.xDomain != null || options.yDomain != null
    : options.xDomain != null;
  const xPad = supplied ? 0 : pad(xMin, xMax);
  const yPad = (both ? supplied : options.yDomain != null) ? 0 : pad(yMin, yMax);

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
  /*
   * The displaced. Compared in the data's own units rather than in pixels, because the scale is
   * not built yet here — and a district whose movement is under a cent is one the run did not
   * move, which is a fact about the formula and not about the rendering.
   */
  const moved = points.filter(
    (p) => p.from != null && (Math.abs(p.from.x - p.x) > 0.005 || Math.abs(p.from.y - p.y) > 0.005),
  );
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
   * Square where a reference line's *angle* is the claim — `identity`, so that y = x is drawn at
   * 45°, and `fit`, so that a slope of one is. Everywhere else the caller's height, or a default
   * that suits a wide cloud.
   *
   * The two get there differently and end in the same place. `identity` puts both axes on one
   * domain here; `fit` is handed two domains the crate has already given the same span, and a
   * square plot area is then what makes a unit of the outcome as long as a unit of the predictor.
   */
  const height =
    options.identity || options.fit
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
      /*
       * The radius is a pixel count, not a measure — which has to be said, because `r` became a
       * channel when `muted` arrived and a channel goes through a scale.
       *
       * With `r` a constant Plot takes it as a literal radius. With `r` a function it is data, and
       * the default `r` scale is a square-root one fitted to the values it is given: 2.4 and 1.6
       * came out of the renderer as **3.67 and 3**. The ordering survived, so the picture still
       * looked right, and every figure written down about it was wrong — the area ratio the
       * de-emphasis is built on went from 44% to 67%.
       *
       * `identity` is what says these are already in the units the mark wants.
       */
      r: { type: "identity" },
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

        // The fitted line, drawn where the identity line is drawn and for the same reason: it is
        // what the cloud is read against rather than a mark in it. Neutral, and asserting no
        // polarity — the polarity is the slope printed beside it.
        ...(options.fit
          ? [
              Plot.line([options.fit.from, options.fit.to], {
                x: "x",
                y: "y",
                stroke: INK.rule,
                strokeWidth: 1.5,
                className: "scatter-fit",
                clip: true,
              }),
            ]
          : []),

        /*
         * Displacement, where the caller supplied a before position.
         *
         * Drawn under the cloud and over the identity line: a trail is where a district came
         * from, and the dot is where it is now. Only the moved are drawn — a zero-length segment
         * is 609 invisible marks and, more to the point, a district that did not move should look
         * like one that did not move.
         *
         * This is the second reference geometry this form draws and the note on `identity` says
         * why there is not a general one: an arbitrary line through a cloud is a claim. A
         * displacement is not that. It is per-point, it is computed from the same run that placed
         * the dot, and it is the subject rather than an assertion laid over it — the whole reason
         * the chart exists is that some of these segments have zero length.
         */
        ...(moved.length > 0
          ? [
              Plot.link(moved, {
                x1: (p: ScatterPoint) => p.from?.x ?? p.x,
                y1: (p: ScatterPoint) => p.from?.y ?? p.y,
                x2: "x",
                y2: "y",
                stroke: hue,
                strokeWidth: 1,
                /* A muted district normally draws no trail at all — `/reach` withholds the before
                   position rather than drawing a faint segment — but a caller that supplies one
                   gets it at the muted alpha rather than at the cloud's. */
                strokeOpacity: (p: ScatterPoint) =>
                  p.muted ? DOT.opacity.muted : DOT.opacity.plain,
                className: "scatter-trail",
                /* See `xDomain`. Where a caller has fixed the frame, a point beyond it must be
                   held at the edge rather than drawn over the card — and where the domain is
                   fitted to the data, as on every baked chart, nothing is outside to clip. */
                clip: true,
              }),
            ]
          : []),

        Plot.dot(points, {
          x: "x",
          y: "y",
          /* Both as channels rather than constants, so a muted point can be drawn as context. The
             radius carries as much of that as the alpha does — see `DOT`. */
          r: (p: ScatterPoint) => (p.muted ? DOT.radius.muted : DOT.radius.plain),
          fill: hue,
          fillOpacity: (p: ScatterPoint) =>
            p.muted ? DOT.opacity.muted : banded ? DOT.opacity.banded : DOT.opacity.plain,
          stroke: "none",
          className: "scatter-dot",
          // As the trails: a fixed frame holds an outlier at the edge rather than painting it
          // over the card, and a fitted one has nothing outside to clip.
          clip: true,
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

        /*
         * The two numbers, in the one corner of a fitted panel that is empty in both of them.
         *
         * Bottom-right: a cloud whose slope is one leaves the corner under its own diagonal
         * clear, and a cloud with no slope sits in a band across the middle and leaves the floor
         * clear. Top-right is empty in the second and is exactly where the first one's line ends.
         */
        ...(options.fit
          ? [
              Plot.text([0], {
                frameAnchor: "bottom-right",
                dx: -6,
                dy: -6,
                text: () => printedFit(options.fit!),
                textAnchor: "end",
                fill: INK.muted,
                fontSize: 11,
                className: "scatter-fit-label",
              }),
            ]
          : []),

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
 * Where a direct label may go beside its mark, in the order the placer tries them.
 *
 * Right first because that is where a label reads from — the eye is already moving that way — and
 * left second because it is the same line of type. Above and below cost a change of line and are
 * what a crowded frame falls back to.
 *
 * Each of `dx`, `dy` and `textAnchor` is a Plot *constant* rather than a channel, which is why
 * this is a small closed table and the marks are drawn one per entry: `barSpec` learned the same
 * thing two forms above.
 */
const LABEL_AT = {
  right: { dx: 9, dy: 0, textAnchor: "start" },
  left: { dx: -9, dy: 0, textAnchor: "end" },
  above: { dx: 0, dy: -13, textAnchor: "middle" },
  below: { dx: 0, dy: 13, textAnchor: "middle" },
} as const;

type LabelAt = keyof typeof LABEL_AT;

/** The order {@link placeLabels} tries them in. Fixed, so the placement is deterministic. */
const LABEL_ORDER: readonly LabelAt[] = ["right", "left", "above", "below"];

/** One line of annotation type, in pixels. `axisFoot`'s `line` less the clear space it leaves. */
const LABEL_LINE = 13;

/** Half a dot's keep-out square, in pixels: the mark's radius plus its ring. */
const DOT_KEEP_OUT = 6;

/** A rectangle in the frame's pixel coordinates. */
interface Box {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** Where a label drawn at `at` would land, given its mark's position and its text. */
function labelBox(at: LabelAt, px: number, py: number, text: string): Box {
  const { dx, dy, textAnchor } = LABEL_AT[at];
  const w = textPx(text);
  const left = px + dx + (textAnchor === "start" ? 0 : textAnchor === "end" ? -w : -w / 2);
  const top = py + dy - LABEL_LINE / 2;
  return { left, top, right: left + w, bottom: top + LABEL_LINE };
}

/** How much of each other two boxes cover, in square pixels. Zero where they merely touch. */
function overlap(a: Box, b: Box): number {
  const w = Math.min(a.right, b.right) - Math.max(a.left, b.left);
  const h = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
  return w > 0 && h > 0 ? w * h : 0;
}

/**
 * Where each mark's direct label goes, so that no two collide and none leaves the frame.
 *
 * # Why this is computed rather than chosen
 *
 * Every other direct label on this site is drawn at a fixed offset, and that works because those
 * forms lay their subjects out on a grid the offset was chosen against: a bar's label is beside
 * its row and rows do not move. A plane places its marks wherever the measurement puts them, so a
 * fixed offset is a bet that the data will not put two of them near each other — and the two this
 * form was written for, a rolling pupil count and `[M]` mirrored inside `[H]`, sit 4 pixels apart
 * on the outcome axis at `WIDTHS.narrow`. Their labels were drawn through each other.
 *
 * So the offsets are a closed set of four and the choice between them is greedy: take the first
 * that lands wholly inside the frame and touches neither another label already placed nor **any**
 * mark, including marks not yet labelled. Where none is clear, take the one that overlaps least,
 * which degrades a crowded frame rather than dropping a name out of it — a plane of seven named
 * policies with one unlabelled dot on it is worse than a tight one.
 *
 * Deterministic in the order the marks arrive, which is the manifest's order, so the same document
 * draws the same picture on every build.
 */
function placeLabels(
  marks: readonly { label: string; px: number; py: number }[],
  frame: Box,
): LabelAt[] {
  const taken: Box[] = marks.map((m) => ({
    left: m.px - DOT_KEEP_OUT,
    top: m.py - DOT_KEEP_OUT,
    right: m.px + DOT_KEEP_OUT,
    bottom: m.py + DOT_KEEP_OUT,
  }));
  const out: LabelAt[] = [];
  for (const mark of marks) {
    let best: LabelAt = LABEL_ORDER[0]!;
    let least = Infinity;
    for (const at of LABEL_ORDER) {
      const box = labelBox(at, mark.px, mark.py, mark.label);
      if (
        box.left < frame.left ||
        box.right > frame.right ||
        box.top < frame.top ||
        box.bottom > frame.bottom
      ) {
        continue;
      }
      const cost = taken.reduce((sum, b) => sum + overlap(box, b), 0);
      if (cost < least) {
        least = cost;
        best = at;
      }
      if (cost === 0) break;
    }
    out.push(best);
    taken.push(labelBox(best, mark.px, mark.py, mark.label));
  }
  return out;
}

/**
 * A handful of named things, each at a point on two signed measures at once.
 *
 * # Why this is not `scatterSpec` with fewer dots
 *
 * A cloud is a population. Its subject is the *shape* six hundred districts make, no one dot is
 * quotable, and what a reader takes off it is the fitted line's two numbers. A plane is a **list**:
 * seven anchor rules, every one of which a reader will name and quote, and the finding is which
 * quadrant each one is in. Nothing is fitted, because a line through seven policies chosen by a
 * legislature is not a relationship — it is a shape read into a list.
 *
 * That difference runs all the way down. The marks are large rather than small, because density is
 * not the information and there is nothing to overplot. Every one carries its name on the picture
 * rather than in a tooltip. And the frame is drawn as **two rules through the origin** rather than
 * as edges at the domain's corners: on this form zero is not a floor but the boundary the reading
 * turns on — left of the vertical is *less than the rule in force*, above the horizontal is *more*
 * — so the four quadrants are the chart, and a bounding box would draw attention to the extent,
 * which means nothing here.
 *
 * # Why the marks carry no hue
 *
 * The quadrant is the encoding. Colouring the three points that disagree between their axes would
 * state in a second channel exactly what their position already states, and this site's two series
 * mean formula aid and guarantee — a third meaning hung on them would be a third meaning on every
 * other chart too.
 *
 * @throws if either domain excludes zero. The crate computes these domains from `(0, 0)` outwards
 * for that reason; a plane whose origin is off the picture has no quadrants and would draw the
 * disagreement as agreement.
 */
export function planeSpec(
  places: Place[],
  axes: {
    x: { label: string; format: (v: number) => string };
    y: { label: string; format: (v: number) => string };
  },
  options: {
    width: number;
    height?: number;
    /** The crate's frame, taken as given: see {@link scatterSpec}'s `xDomain` for why. */
    xDomain: [number, number];
    yDomain: [number, number];
  },
): Spec | null {
  if (places.length === 0) return null;
  const { width } = options;
  const [xLo, xHi] = options.xDomain;
  const [yLo, yHi] = options.yDomain;
  for (const [which, lo, hi] of [
    ["x", xLo, xHi],
    ["y", yLo, yHi],
  ] as const) {
    if (!(lo <= 0 && hi >= 0)) {
      throw new Error(
        `planeSpec was given a ${which} domain of [${lo}, ${hi}], which does not hold zero. ` +
          `The two zero rules are this form's axes; without the origin on the picture there are ` +
          `no quadrants to read it in.`,
      );
    }
  }

  const marginLeft = 62;
  const marginTop = 28;
  const marginRight = gutter(width, 40);
  const foot = axisFoot({
    width,
    marginLeft,
    marginRight,
    dy: 20,
    low: axes.x.format(xLo),
    says: axes.x.label,
    high: axes.x.format(xHi),
  });
  const marginBottom = 40 + foot.extraBottom;
  const height = options.height ?? 420;

  const frame: Box = {
    left: marginLeft,
    top: marginTop,
    right: width - marginRight,
    bottom: height - marginBottom,
  };
  const at = places.map((place) => ({
    label: place.label,
    px: frame.left + ((place.x - xLo) / (xHi - xLo)) * (frame.right - frame.left),
    py: frame.bottom - ((place.y - yLo) / (yHi - yLo)) * (frame.bottom - frame.top),
  }));
  const placed = placeLabels(at, frame);

  return {
    options: {
      width,
      height,
      marginLeft,
      marginRight,
      marginTop,
      marginBottom,
      // As `scatterSpec`: a radius is a pixel count, and a channel would send it through a scale.
      r: { type: "identity" },
      x: { axis: null, type: "linear", domain: [xLo, xHi] },
      y: { axis: null, type: "linear", domain: [yLo, yHi] },
      marks: [
        // The two axes, through the origin. Recessive like every other rule on the site, and the
        // only frame this form draws — see the note above on why there is no bounding box.
        Plot.ruleX([0], { stroke: INK.rule, className: "plane-zero" }),
        Plot.ruleY([0], { stroke: INK.rule, className: "plane-zero" }),

        Plot.dot(places, {
          x: "x",
          y: "y",
          r: 4.5,
          fill: SERIES.neutral,
          // The ring the mark-size guidance asks for on anything that can overlap. Two of these
          // seven share an x of exactly zero, which is a fact about the rules rather than an
          // accident of scale, so the pair has to stay countable where the scale brings them near.
          stroke: INK.surface,
          strokeWidth: 1.5,
          className: "plane-dot",
        }),

        // One text mark per placement, because `dx`, `dy` and `textAnchor` are constants in Plot.
        // Text is not in the hover selector, so splitting these reorders nothing that is indexed.
        ...LABEL_ORDER.flatMap((which) => {
          const mine = places.filter((_, i) => placed[i] === which);
          if (mine.length === 0) return [];
          return [
            Plot.text(mine, {
              x: "x",
              y: "y",
              ...LABEL_AT[which],
              text: "label",
              fill: INK.primary,
              fontSize: 11,
              className: "plane-label",
            }),
          ];
        }),

        // Both ends of both scales, as a cloud carries them: a frame with no numbers on it is a
        // texture, and here the numbers are what say how far from the rule in force these get.
        ...foot.marks,
        Plot.text([0], {
          frameAnchor: "top-left",
          dx: -marginLeft + 4,
          dy: 3,
          text: () => axes.y.format(yHi),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom-left",
          dx: -marginLeft + 4,
          text: () => axes.y.format(yLo),
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "top-left",
          dx: -marginLeft + 4,
          dy: -12,
          text: () => axes.y.label,
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),

        // The hit layer, above everything, as every pointed-at form here draws one.
        Plot.dot(places, {
          x: "x",
          y: "y",
          r: 10,
          fill: "transparent",
          stroke: "none",
          className: "plane-hit",
        }),
      ],
    },
    hovers: {
      selector: ".plane-hit > *",
      text: places.map((p) => escapeHtml(p.hover)),
      cursor: { second: "paired marks", layers: [".plane-dot"] },
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
 * Many items on one count, ranked, on a logarithmic axis — with the item at zero drawn.
 *
 * # Why this is not `barSpec`
 *
 * The census of the modelled formula's bounds is thirty-eight rows running from 554 districts to
 * one, and one row at **nought**. On a linear axis the bottom third of that is a row of stubs
 * indistinguishable from each other and from zero: the two bounds reached by exactly one district
 * — the cap on the DPIA blended count and the clamp of the guarantee's floor — are the finding
 * at that end of the sort, and a bar 0.18% of the frame long does not state a finding. A log axis
 * gives every one of the thirty-eight a length its own eye can compare, which is the whole reason
 * the axis is logarithmic and is the same argument {@link rangeSpec} makes for `/counties`.
 *
 * # The row a log axis cannot draw
 *
 * Zero is nowhere on a log scale, and a zero-length bar says nothing on any scale — but the bound
 * that **cannot bind** is the most interesting row in the census and dropping it would be a chart
 * that reports thirty-seven bounds and calls itself a map of thirty-eight. So a row at zero is
 * drawn at the axis floor, half a decade below the smallest value that can be placed, in the
 * contrasting hue and with its own phrase printed beside it — see {@link Rank.marked}. It is
 * visibly outside the run of the data rather than being the shortest member of it, which is what
 * a reader has to understand about it.
 *
 * # Dot and stem, not a bar
 *
 * A bar's *length* encodes its value, and on a log axis a length is a ratio to wherever the frame
 * happens to start — so a bar drawn from an arbitrary floor would be a magnitude a reader could
 * read off it and would be wrong. The datum is therefore the **dot**, at its own position, and
 * the stem behind it is drawn in rule ink as a guide across the row rather than as a measure.
 * The same vocabulary, and for the same reason, as `rangeSpec`'s span and ends.
 */
export function rankSpec(
  rows: Rank[],
  axis: { label: string; format: (v: number) => string },
  options: { width: number },
): Spec | null {
  if (rows.length < 2) return null;
  if (rows.some((r) => r.value < 0)) return null;

  const values = rows.map((r) => r.value);
  const max = Math.max(...values);
  const smallest = Math.min(...values);
  const positive = values.filter((v) => v > 0);
  if (positive.length === 0 || max <= 0) return null;
  /*
   * The left end of the drawn domain.
   *
   * Where nothing is at zero it is simply the smallest value. Where something is, it is half the
   * smallest value that *can* be drawn — a clear step below the axis's own bottom mark, so the
   * zero row reads as off the scale rather than as the last rung of it.
   */
  const floor = smallest > 0 ? smallest : Math.min(...positive) / 2;
  const at = (r: Rank) => (r.value > 0 ? r.value : floor);

  const longest = Math.max(...rows.map((r) => r.label.length));
  const { width } = options;
  // Sized to the longest name, like `barSpec`'s, and to 260 rather than `rangeSpec`'s 150: these
  // names are sentences about a provision and not county names. Where the cap bites, the row
  // grows to hold two lines rather than the name being cut — the 14px row `rangeSpec` draws
  // cannot, which is why that form truncates and this one wraps.
  const wanted = Math.max(90, Math.min(260, Math.round(longest * 6.2) + 12));
  const marginLeft = gutter(width, wanted);
  const rowHeight = marginLeft < wanted ? 28 : 16;

  const marked = rows.filter((r) => r.marked != null);
  const plain = rows.filter((r) => r.marked == null);
  /*
   * Room at the right for the marked row's phrase, which is the one direct label this form draws.
   *
   * Not simply the phrase's width. It is printed outward from its own mark, so what it needs
   * *past the frame* is its width less whatever empty plot already lies to the right of that
   * mark — and the row this form exists for sits at the axis floor, where the whole width of the
   * plot is empty to its right and the reserve is nothing at all. Reserving for it anyway cost
   * 95px of a 320px frame: a third of the narrow drawing, held blank, for text drawn at the
   * opposite edge.
   *
   * Measured against a first pass with no reserve, which is the smaller frame, so the answer errs
   * toward leaving room rather than toward cutting the phrase.
   */
  const phrase = Math.max(0, ...marked.map((r) => textPx(r.marked ?? "", 10)));
  const inner = Math.max(1, width - marginLeft - gutter(width, 16));
  const decades = Math.log(max / floor);
  const rightmost = Math.max(floor, ...marked.map(at));
  const beyond = inner * (1 - (decades > 0 ? Math.log(rightmost / floor) / decades : 0));
  const marginRight = gutter(width, 16 + Math.max(0, 8 + phrase - beyond));

  const foot = axisFoot({
    width,
    marginLeft,
    marginRight,
    dy: 16,
    low: axis.format(smallest),
    says: `${axis.label} (log scale)`,
    high: axis.format(max),
  });

  return {
    options: {
      width,
      height: rows.length * rowHeight,
      marginLeft,
      marginRight,
      marginTop: 0,
      marginBottom: 22 + foot.extraBottom,
      x: { axis: null, type: "log", domain: [floor, max] },
      y: { axis: null, domain: rows.map((r) => r.label), padding: 0.2 },
      marks: [
        // The guide, not the measure. Under the dot, and absent from the zero row, which has no
        // distance to cover and whose position is the claim.
        Plot.ruleY(plain, {
          y: "label",
          x1: floor,
          x2: at,
          stroke: INK.rule,
          strokeWidth: 1,
          className: "rank-stem",
        }),
        /*
         * One dot mark for every row, with the hue as a channel.
         *
         * Split by subject the way the labels below are and `attachHovers` would index the wrong
         * tooltip onto it: the cursor pairs `.rank-hit`'s children with `.rank-dot`'s by
         * position, and `declareCursor` refuses a chart where the two layers disagree in count.
         * `barSpec` takes the same channel for the same reason.
         */
        Plot.dot(rows, {
          y: "label",
          x: at,
          r: DOT_RADIUS,
          fill:
            marked.length > 0
              ? (r: Rank) => (r.marked == null ? SERIES.formula : SERIES.guarantee)
              : SERIES.formula,
          stroke: "none",
          className: "rank-dot",
        }),
        Plot.text(plain, {
          y: "label",
          frameAnchor: "left",
          dx: -8,
          text: "label",
          textAnchor: "end",
          fill: INK.secondary,
          fontSize: 10,
          lineWidth: lineWidth(marginLeft),
          className: "rank-label",
        }),
        // The subject's second channel. Weight and ink survive a monochrome print and a
        // forced-colours mode; the hue above does not, and colour is never the only channel.
        ...(marked.length > 0
          ? [
              Plot.text(marked, {
                y: "label",
                frameAnchor: "left",
                dx: -8,
                text: "label",
                textAnchor: "end",
                fill: INK.primary,
                fontSize: 10,
                fontWeight: 600,
                lineWidth: lineWidth(marginLeft),
                className: "rank-label current",
              }),
              Plot.text(marked, {
                y: "label",
                x: at,
                dx: 8,
                text: "marked",
                textAnchor: "start",
                fill: INK.primary,
                fontSize: 10,
                className: "rank-mark",
              }),
            ]
          : []),
        ...foot.marks,
        // The hit target is the row rather than the dot, and overhangs the ends by the dot's
        // radius — `rangeSpec`'s insets, for the defect recorded there.
        Plot.rect(rows, {
          y: "label",
          x1: floor,
          x2: max,
          insetLeft: -DOT_RADIUS,
          insetRight: -DOT_RADIUS,
          fill: "transparent",
          className: "rank-hit",
        }),
      ],
    },
    hovers: {
      selector: ".rank-hit > *",
      text: rows.map((r) => escapeHtml(r.hover)),
      cursor: { second: "paired marks", layers: [".rank-dot"] },
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

/** What a caller tells {@link seriesSpec} beyond the points themselves. */
export interface SeriesOptions {
  width: number;
  /**
   * How to write a position on the index, for the two corner labels under the axis.
   *
   * Required rather than defaulted, because every default here would be a guess about what the
   * index counts and the wrong one prints `FY13` under a thirteen-year horizon.
   */
  tick: (at: number) => string;
  /** A value the lines are measured against, drawn as a rule and held inside the frame. */
  reference?: { value: number; label: string };
}

/**
 * Two quantities in the same units, over one ordered index.
 *
 * The fourth form, and the one the historical view needed: a fan chart draws an interval and a
 * bar chart draws categories, and neither says what a pair of series did across fourteen years.
 *
 * The rules it inherits, and the three it adds:
 *
 * - **Two series, and there cannot be a third.** The categorical palette is a validated pair and
 *   nothing here generates beyond it. A page needing a third quantity puts it in the table
 *   underneath, which is what the table is for.
 * - **One axis.** Both series are in the same units by construction — the caller passes shares or
 *   dollars, never one of each — so there is no second scale to mislead with.
 * - **A missing position is a break, not a bridge.** FY2014 is absent from the Census archive,
 *   and a line drawn straight through it would assert a measurement nobody made. Plot breaks a
 *   line at a null, and passing `null` rather than omitting the position is what keeps the gap on
 *   the x axis where a reader can see it.
 * - The axis is truncated to the data's own range and says so, as the fan chart does, because a
 *   share moving from 46% to 34% is invisible against a zero baseline.
 *
 * # The index is not necessarily a year
 *
 * It was, for the three years this form existed for the three pages that draw fiscal years, and
 * `FY${'{'}point.year{'}'}` was written into the axis foot here rather than supplied by the caller. The
 * coverage curve on `/method` runs over **horizons**, so the caller now writes both end labels
 * through {@link SeriesOptions.tick} and this function makes no claim about what the index
 * counts. The rename of `SeriesPoint.year` to `at` is the same change said in the type.
 *
 * # The reference line
 *
 * A curve may be drawn against a value the lines are *measured by* rather than compared to each
 * other — 68.3% is what a ±1σ band claims to hold, and what the two coverage lines mean is their
 * distance from it. Passed as {@link SeriesOptions.reference}, it is drawn as a labelled rule and,
 * more importantly, **included in the truncated domain**: a reference outside the frame would be
 * a comparison the reader has been asked to make and not shown. It is not a third series — it has
 * no per-position value, takes the muted ink rather than a categorical hue, and nothing hovers it.
 */
export function seriesSpec(
  points: SeriesPoint[],
  labels: { a: string; b: string },
  format: (v: number) => string,
  hover: (p: SeriesPoint) => string,
  options: SeriesOptions,
): Spec | null {
  // One point is not a series, exactly as in `fanSpec`.
  if (points.length < 2) return null;
  const values = points.flatMap((p) => [p.a, p.b]).filter((v): v is number => v != null);
  if (values.length === 0) return null;

  // The reference is inside the domain by construction rather than by luck: the lines are read
  // for their distance from it, and a rule drawn off the frame states no distance at all.
  const [min, max] = truncatedDomain(
    options.reference ? [...values, options.reference.value] : values,
  );

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
      x: "at",
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
            x: point.at,
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
   * Under everything, because it is what the two lines are read against rather than a third line
   * among them. Muted and dashed for the same reason: the categorical pair is the quantities, and
   * a reference in a third hue would read as a quantity that has no per-position value.
   */
  const ref = options.reference;
  const reference = ref
    ? [
        Plot.ruleY([ref.value], {
          stroke: INK.muted,
          strokeDasharray: "3 3",
          className: "series-reference",
        }),
        Plot.text([ref.value], {
          x: first.at,
          y: ref.value,
          dx: 2,
          dy: -6,
          text: () => ref.label,
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
          className: "series-reference",
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
    low: options.tick(first.at),
    says: `axis starts at ${format(min)}, not zero`,
    high: options.tick(last.at),
  });

  return {
    options: {
      width: options.width,
      height: 220,
      marginTop: 14,
      marginBottom: 26 + foot.extraBottom,
      marginLeft: 0,
      marginRight,
      x: { axis: null, domain: [first.at, last.at] },
      y: { axis: null, domain: [min, max] },
      marks: [
        ...reference,
        line("a", SERIES.formula, "series-a"),
        line("b", SERIES.guarantee, "series-b"),
        ...endLabel(endA, "a", SERIES.formula),
        ...endLabel(endB, "b", SERIES.guarantee),
        Plot.ruleY([min], { stroke: INK.rule, className: "axis" }),
        ...foot.marks,
        // One full-height column per year, above every mark, as the fan chart does.
        Plot.rect(points, {
          x1: (p: SeriesPoint) => p.at - 0.5,
          x2: (p: SeriesPoint) => p.at + 0.5,
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
