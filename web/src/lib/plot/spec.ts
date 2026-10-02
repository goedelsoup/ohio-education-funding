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
 * The builders touch no DOM, so they are testable without one. `draw` touches only the document
 * it is handed, or the browser's where it is handed none.
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

import { sentence, unstop } from "../chartWords.ts";
import { compactMoney, compactMoneyTicks, escapeHtml } from "../format.ts";
import type {
  Bar,
  Bin,
  DistributionValue,
  FanPoint,
  Fit,
  Place,
  Range,
  RangeMarker,
  Rank,
  ScatterPoint,
  SeriesPoint,
  Trace,
} from "../chart.ts";
import { INK, ORDINAL, RULE, SERIES, SERIES_TEXT, SUBJECT } from "./tokens.ts";
import { firstOf, lastOf } from "../ends.ts";
import { fiscalYear } from "../yearLabel.ts";

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

function applyNaming(node: Nameable, naming: Naming): void {
  // `Nameable` is deliberately the smallest surface this file needs, so the tidy-up is guarded
  // rather than assumed: `ssr.ts` and `client.ts` both hand over a real element.
  if ("querySelectorAll" in node) untangle(node as unknown as Element);
  if (naming === "presentational") {
    node.setAttribute("aria-hidden", "true");
    return;
  }
  node.setAttribute("role", "img");
  node.setAttribute("aria-label", unstop(naming.label));
  /*
   * The description is the method — how the axis is cut, what a band means — and it was appended
   * to the name, which a screen reader speaks in full before a reader can move past the chart: 577
   * characters on `/method` (#612). In a `<desc>` it is the chart's accessible description
   * instead, read after the name and only by a reader who stays to hear it.
   */
  if (naming.description && "ownerDocument" in node) {
    const element = node as unknown as Element;
    const desc = element.ownerDocument.createElementNS("http://www.w3.org/2000/svg", "desc");
    desc.textContent = naming.description;
    element.prepend(desc);
  }
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
  hovers?: {
    selector: string;
    text: string[];
    cursor: Cursor;
    /**
     * Where a keyboard or a tap puts the tip, as a y value in data units, one per mark.
     *
     * For a hit target that is not where its value is drawn. A fan or series column is the full
     * height of the frame, and the tip anchored to its foot sat over the axis note and the key
     * below the chart (#607). Written onto each mark as `data-y`, in the drawing's own units, by
     * {@link draw}; `null` leaves a mark anchored to its box.
     */
    anchor?: (number | null)[];
  };
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
   * argument `draw` makes about its hover text: a pairing followed by index that does not
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
const BASE: Plot.PlotOptions = {
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
 * The three widths every chart is drawn at.
 *
 * # Why a chart is drawn more than once rather than scaled
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
 * So the layout is recomputed at the width it will be shown at. `narrow` is sized so that a phone
 * scales it by at least 0.9 rather than 0.46; `wide` is the width these forms were designed and
 * tuned at, unchanged. `renderToString` draws all three and the stylesheet shows one — see
 * `.chart-pair` in `app.css` for the container query that picks.
 *
 * # Why three
 *
 * Two left a band neither drawing filled (#609). A drawing grows to {@link MAX_SCALE} of its width
 * and no further, so the narrow one stopped at 400px while its box ran on to the 576px swap: a
 * 400px fan in a 552px card, then a jump to the wide drawing 46% wider with its text a third
 * smaller. Moving the swap cannot close that, because the wide drawing at 0.8 paints its 10px axis
 * text at 8. The band needs a drawing of its own.
 *
 * Each drawing takes over where it is shown at exactly its own width, from the one below it at
 * its cap. That makes the step in painted type at a swap 1 − 1/1.25, a fifth, and never more; it
 * means `middle` and `wide` are never shown smaller than they were laid out; and it fixes
 * `middle` as the width that leaves the two bands it cannot fill equally short of full — the
 * narrow drawing's 375 in a box nearly 440 wide, and its own 550 in a box nearly 640. √(375 × 512)
 * is 438. Both bands fill at least 85% of their box.
 *
 * The cost is real and was weighed: each drawing after the first is about 31% more built HTML,
 * and a scatter's coordinates do not compress against another width's (#518). It buys legible
 * axis text on the width most of this site's readers are on, and a chart that fills its card at
 * every width between.
 */
export const WIDTHS = {
  /**
   * For a phone. 280px is the content box at 360px, the narrowest common Android viewport, so 300
   * is scaled by 0.93 there rather than shrunk. 320 was, and painted 10px axis text at 8.8 (#609).
   */
  narrow: 300,
  /** For the band between a phone and a desktop column. See "Why three" above. */
  middle: 440,
  /** What every chart in this file was drawn at before there was more than one, and still is. */
  wide: 640,
} as const;

/**
 * The most a drawing is scaled **up** by, as a multiple of the width it was laid out at.
 *
 * {@link WIDTHS} put a floor under chart text and nothing over it. The SVG is `width: 100%` over
 * a `viewBox`, so a drawing grows with its box: the 640 drawing in `/counties`' 1180px column
 * painted its dumbbell labels at 16.9–18.6px, larger than the 15px prose around them, and the 320
 * drawing in `/scenario/reach`'s 548px box painted at 18.8px. The same form came out at different
 * sizes on pages under one menu, because some pages are `.wrap.wide` and some are not.
 *
 * 1.25 because the largest type any form here sets is 12 units — `BASE`'s `fontSize` — and 12 ×
 * 1.25 is the 15px of body text. So no chart label is painted larger than the sentence above it.
 * `draw` writes the cap onto every SVG as a `max-width`, which is the one place both renderers and
 * every panel width pass through; a box wider than the cap leaves the drawing at its left edge.
 */
export const MAX_SCALE = 1.25;

/** The interval between two panels of a small multiple: `--space-6`, 1.1rem, at the 16px root. */
const PANEL_GAP = 18;

/**
 * The narrowest a panel is laid out at before the row wraps: the `minmax()` floor in `.panels`'s
 * grid, in `app.css`.
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
 * rather than at each of them — which is the reason this is a function and not a call to
 * `renderToString`. The cloud these draw is 609 dots; another copy for a layout it is never shown
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

/** Where a renderer's document comes from, and what it does with a hover layer that misaligns. */
export interface Renderer {
  /**
   * The DOM Plot draws into. `ssr.ts` passes `linkedom`'s; the browser omits it and Plot uses the
   * global one. That is the only reason there are two renderers.
   */
  document?: Document;
  /**
   * Called with a sentence when the hover text or the cursor's paired layer does not line up with
   * what Plot drew. The build throws, so a misalignment cannot reach a reader; the browser drops
   * the channel, because a chart that cannot pair its marks should lose its second channel rather
   * than take the page down with it. Either way the misaligned pairing is never written.
   */
  onMisaligned: (message: string) => void;
}

/**
 * Write each hit mark's anchor onto it, through the y scale Plot actually drew with.
 *
 * Plot's scale rather than arithmetic on the spec's margins and domain, because the scale is the
 * one thing that agrees with the paths by construction. Rounded to a tenth of a unit: the tip is
 * placed in whole pixels, and full precision would be bytes on every year of every district fan.
 */
function anchorTips(node: Element, marks: NodeListOf<Element>, anchor?: (number | null)[]): void {
  if (!anchor) return;
  const y = (node as unknown as { scale(name: "y"): { apply(v: number): number } | undefined })
    .scale("y");
  if (!y) return;
  marks.forEach((mark, index) => {
    const value = anchor[index];
    if (value == null) return;
    mark.setAttribute("data-y", `${Math.round(y.apply(value) * 10) / 10}`);
  });
}

/**
 * One SVG element, at one width, or `null` where the builder declines the data.
 *
 * Both renderers are this function and a line either side of it, so the chart a slider redraws is
 * put together by the same code as the one baked into the page.
 *
 * The hover text is put on each mark in data order, and the order is checked rather than trusted.
 * Plot emits marks in the order it is given them, but that is an assumption about someone else's
 * renderer: a mismatch means Plot dropped, filtered or reordered something, and tooltips attached
 * by index would label the wrong quantities — worse than having none, because they would look
 * right. Applied here rather than through Plot's `title` channel, which produces the browser's
 * native tooltip: slow to appear, unstyleable, and unable to hold the two-clause sentences these
 * charts need.
 */
export function draw(
  build: Drawing,
  naming: Naming,
  width: number,
  renderer: Renderer,
): Element | null {
  const spec = build(width);
  if (!spec) return null;
  const options = renderer.document
    ? { ...BASE, ...spec.options, document: renderer.document }
    : { ...BASE, ...spec.options };
  const node = Plot.plot(options) as unknown as Element;
  if (spec.hovers) {
    const { selector, text } = spec.hovers;
    const marks = node.querySelectorAll(selector);
    if (marks.length !== text.length) {
      renderer.onMisaligned(
        `The hover layer does not line up with what Plot drew: ${text.length} values against ` +
          `${marks.length} marks matching "${selector}". Attaching them by index would label ` +
          `the wrong quantities.`,
      );
    } else {
      marks.forEach((mark, index) => mark.setAttribute("data-hover", text[index]!));
      anchorTips(node, marks, spec.hovers.anchor);
      const misaligned = declareCursor(node, spec.hovers);
      if (misaligned) renderer.onMisaligned(misaligned);
    }
  }
  applyNaming(node, naming);
  node.setAttribute("style", `${node.getAttribute("style") ?? ""};max-width:${width * MAX_SCALE}px`);
  return node;
}

/**
 * A chart laid out at every one of {@link WIDTHS}, for the stylesheet to choose between.
 *
 * # Why more than one
 *
 * A static SVG has one layout and the stylesheet used to make it fit by scaling it. At 375px that
 * scale is 0.46, which took the axis text to about 4.6px — see {@link WIDTHS} for why enlarging
 * the type instead does not work. So the drawing is laid out at each width it is actually shown
 * at, and `app.css` picks with a container query. All of them are in the document; the ones the
 * reader is not looking at are `display: none`, which also takes them out of the accessibility
 * tree, so a screen reader is offered one chart rather than the same chart three times.
 *
 * `null` renders to nothing. A spec builder returns null when the data cannot support the form it
 * was asked for — a fan chart of one observation, say — and an empty string is the honest output:
 * better than an axis with a single mark on it, which would read as a finding. That decision is
 * about the data and never about the width, so it is taken once, on the wide drawing, and the
 * other two are not asked.
 */
export function pair(at: (width: number) => string): string {
  const wide = at(WIDTHS.wide);
  if (!wide) return "";
  const narrow = at(WIDTHS.narrow);
  const middle = at(WIDTHS.middle);
  return (
    `<div class="chart-pair">` +
    `<div class="chart-at" data-at="narrow">${narrow}</div>` +
    `<div class="chart-at" data-at="middle">${middle}</div>` +
    `<div class="chart-at" data-at="wide">${wide}</div>` +
    `</div>`
  );
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

/**
 * How much wider a name is drawn at weight 600 than at normal weight.
 *
 * The budget above is measured on normal-weight text, and a bar chart draws its subject's name
 * bold. On a phone the name gutter is capped, so the bold name is the one that has to wrap — and
 * `Cleveland Municipal`, marked on its county's chart at `WIDTHS.narrow`, fitted the normal-weight
 * budget, stayed on one line and painted 10px past the left of the frame on CI's fonts (#639).
 * Ten in 125px is 8%, so the factor is that rather than a figure chosen first.
 */
const BOLD_WIDENS = 1.08;

/** A `lineWidth` in ems that holds a line to `px` pixels of drawn text. */
function lineWidth(px: number, fontSize = 10): number {
  return (px / fontSize) * (0.5 / EM_PER_CHAR);
}

/** How wide a string is drawn, in pixels, at the annotation type size. */
function textPx(text: string, fontSize = 11): number {
  return text.length * EM_PER_CHAR * fontSize;
}

/**
 * A string broken into lines that each hold to `px` pixels of drawn text, at word boundaries.
 *
 * Not Plot's `lineWidth`, for the one place that option cannot serve: the caller has to know how
 * many lines came out, because each one costs margin. Plot wraps against its own width table and
 * does not say what it decided, so a margin sized beside it is a second estimate of the same
 * thing, and the two disagree on exactly the labels long enough to matter. The lines are joined
 * with `\n` by the caller, which Plot draws as one `tspan` each, so the count here is the count
 * drawn. A single word wider than the budget is left whole: cutting it would print a fragment.
 */
function wrapText(text: string, px: number, fontSize = 11): string[] {
  const lines: string[] = [];
  let line = "";
  // Not `\s`, which matches U+00A0: a non-breaking space holds its two words together (#662).
  for (const word of text.split(/[^\S\u00a0]+/).filter(Boolean)) {
    const next = line ? `${line} ${word}` : word;
    if (line && textPx(next, fontSize) > px) {
      lines.push(line);
      line = word;
    } else {
      line = next;
    }
  }
  if (line) lines.push(line);
  return lines.length > 0 ? lines : [""];
}

/** The extra drop per wrapped line of a y-axis title: 11px type and the clear space under it. */
const TITLE_LINE = 14;

/**
 * The y-axis title a cloud or a plane prints above its frame, wrapped to the drawing.
 *
 * It sits at the left edge and runs right, over the frame, and was one line: on the two-panel TTAG
 * plane the left panel's title ran 139 units past its own SVG and across its neighbour's. Wrapped
 * to the whole width less 4 units each side, and the frame moves down by a line for every line
 * past the first, so the top of the title stays where a one-line title has always been.
 */
function yTitle(width: number, marginLeft: number, label: string, marginTop: number) {
  const lines = wrapText(sentence(label), width - 8);
  const extra = (lines.length - 1) * TITLE_LINE;
  /*
   * Plot sets a top-anchored text's first baseline at `0.71 × lineHeight` em, so a spacing given
   * to the lines moves the first one down with them — 2.1 units here, which put every title
   * through the y-axis maximum printed under it. A one-line title is given no spacing and is
   * drawn exactly where it was; a wrapped one is lifted by what the spacing moved it.
   */
  const lift = lines.length > 1 ? 0.71 * (TITLE_LINE - 11) : 0;
  return {
    marginTop: marginTop + extra,
    mark: Plot.text([0], {
      frameAnchor: "top-left",
      dx: -marginLeft + 4,
      dy: -12 - extra - lift,
      text: () => lines.join("\n"),
      textAnchor: "start",
      ...(lines.length > 1 ? { lineHeight: TITLE_LINE / 11 } : {}),
      fill: INK.muted,
      fontSize: 11,
      className: "y-title",
    }),
  };
}

/**
 * A numeric scale whose ends a foot labels where they fall, rather than at the frame's corners.
 *
 * `format` is the chart's own formatter for a label. `floor`, where given, is what the low end of
 * the domain says instead of a number: `rankSpec`'s domain starts half a step below its smallest
 * drawable value, so the row at zero has somewhere to sit, and a log scale reaches no zero to
 * label. It printed "0" there, a value the axis cannot hold.
 */
export interface FootScale {
  domain: [number, number];
  log?: boolean | undefined;
  format: (v: number) => string;
  floor?: string;
  /**
   * A label the scale's own labels keep clear of, centred on its value: the strips' "no change" at
   * zero. Drawn by the foot so that it is measured with the rest rather than drawn over them.
   */
  beside?: { value: number; label: string };
}

/**
 * One axis's labels, in one style. A money axis is the case that needs the whole set at once —
 * see {@link compactMoneyTicks} — and every other format is per value.
 */
function tickLabels(values: number[], format: (v: number) => string): string[] {
  return format === compactMoney ? compactMoneyTicks(values) : values.map(format);
}

/**
 * What a log axis appends to its title. The space inside the parenthesis is U+00A0: `/bounds`
 * wrapped "(log" and "scale)" onto two lines at 1280 (#662).
 */
const LOG_SCALE = " (log\u00a0scale)";

/** `rangeSpec`'s x domain: the data on a log scale, and 4% either side of it on a linear one. */
function rangeDomain(min: number, max: number, log = false): [number, number] {
  return log ? [min, max] : [min - (max - min) * 0.04, max + (max - min) * 0.04];
}

/**
 * The two round ends of a y scale, in the left margin, each at its own height.
 *
 * The highest hangs down from its value and the lowest hangs up from its value, so the pair stays
 * inside the frame whichever end of the domain a round value lands at.
 */
function yEnds(
  domain: [number, number],
  format: (v: number) => string,
  marginLeft: number,
  log = false,
): Plot.Markish[] {
  const ticks = axisTicks(domain, log);
  const pair = [firstOf(ticks), lastOf(ticks)];
  const labels = tickLabels(pair, format);
  return pair.map((value, i) =>
    Plot.text([value], {
      y: () => value,
      frameAnchor: "left",
      dx: -marginLeft + 4,
      text: () => labels[i]!,
      // The top label 3 units clear of the axis name above the frame, which a round value at the
      // very top would otherwise touch where the name wraps to two lines.
      ...(i === 1 ? { dy: 3 } : {}),
      textAnchor: "start",
      lineAnchor: i === 0 ? "bottom" : "top",
      fill: INK.muted,
      fontSize: 11,
      className: "axis-end",
    }),
  );
}

/**
 * The values to label along a numeric axis: round ones, inside the domain.
 *
 * Every foot printed the data's two extremes, `$78,992 | $1,350,078`, as written. The strips' and
 * the scatters' domains are padded past the extremes, so the extremes were not even where they
 * were printed. "$1,350,078" sat 25px of 800 to the right of the dot it named, over nothing. A
 * round value at its own position tells the reader where the scale is, which is the one job the
 * label has.
 *
 * Linear: the first and last multiples of a 1-2-5 step inside the domain. Log: the first and last
 * 1-2-5 values inside it and every power of ten between them. Two numbers on a log axis say
 * nothing about its spacing, and the decades are what a reader reads it by.
 */
export function axisTicks(domain: [number, number], log = false): number[] {
  const [lo, hi] = domain;
  if (!(hi > lo)) return [lo];
  if (log && lo > 0) {
    const first = niceLog(lo, "up");
    const last = niceLog(hi, "down");
    if (first < last) {
      const decades: number[] = [];
      for (let k = Math.ceil(Math.log10(first)); 10 ** k < last; k++) {
        const decade = clean(10 ** k);
        if (decade > first) decades.push(decade);
      }
      return [first, ...decades, last];
    }
  }
  // Coarse first, finer until there are two to label: a narrow domain can hold no multiple of the
  // step its span suggests, or only one.
  for (let step = niceStep((hi - lo) / 4); step > (hi - lo) / 1000; step = niceStep(step / 2)) {
    const first = clean(Math.ceil(lo / step) * step);
    const last = clean(Math.floor(hi / step) * step);
    if (last > first) return [first, last];
  }
  return [lo, hi];
}

/** The largest 1, 2 or 5 times a power of ten that is at most `v`. */
function niceStep(v: number): number {
  const power = 10 ** Math.floor(Math.log10(v));
  const m = v / power;
  return clean((m >= 5 ? 5 : m >= 2 ? 2 : 1) * power);
}

/** The nearest 1-2-5 value at or beyond `v` in the direction given. */
function niceLog(v: number, toward: "up" | "down"): number {
  const k = Math.floor(Math.log10(v));
  const rungs = [-1, 0, 1].flatMap((d) => [1, 2, 5].map((m) => clean(m * 10 ** (k + d))));
  return toward === "up"
    ? Math.min(...rungs.filter((r) => r >= v * (1 - 1e-9)))
    : Math.max(...rungs.filter((r) => r <= v * (1 + 1e-9)));
}

/** A product of a step and an integer, without the float residue `0.1 * 3` leaves. */
function clean(v: number): number {
  return Number(v.toPrecision(12));
}

/**
 * The annotation along the foot of a chart: the scale's labels, and between them the one sentence
 * the chart has to say about itself.
 *
 * Four forms drew this row and all four drew it the same way — three text marks at one `dy`,
 * anchored bottom-left, bottom and bottom-right. On a 640 frame the three fit. On a 320 one they
 * do not: `/history` drew `FY2009`, `axis starts at 32%, not zero` and `FY2022` across 186px of
 * frame and the centre ran through both years, so the chart's statement that its axis is truncated
 * — the whole reason that statement is on the chart rather than in the caption — arrived as a
 * smear of overlapping type.
 *
 * So the row measures itself. Where the centre fits between the labels, it stays there. Where it
 * does not, the centre drops to a line of its own under them — and stops being a centre, for the
 * reason given where it is drawn — and the caller is told how much more bottom margin that costs.
 *
 * # Two kinds of end
 *
 * `low` and `high` as strings are written at the frame's corners. That is right for a time axis
 * whose domain is exactly its first and last year, which is what the fan and the series pass. A
 * {@link FootScale} is labelled at round values where they fall instead (see {@link axisTicks}),
 * and an interior label that would collide with its neighbours is left out rather than drawn
 * through them. The ends are kept first, because they are what says how far the scale runs.
 */
function axisFoot(options: {
  width: number;
  marginLeft: number;
  marginRight: number;
  dy: number;
  /** What the chart says about itself, between the labels. Empty for a strip, which says nothing. */
  says: string;
  /**
   * The type size, 11 by default. A strip is drawn at the narrow width and painted across a whole
   * column, so on a phone its 11 units came out at 12.3px, past the band every other chart's foot
   * sits in; it passes 10.
   */
  fontSize?: number;
  /**
   * Above the frame rather than under it: the scale repeated at the top of a chart too tall to
   * read against a foot alone (#611). Drawn without `says`, which the foot already carries, and
   * classed `axis-head` so the two can be told apart.
   */
  top?: boolean;
} & ({ low: string; high: string } | { scale: FootScale })): {
  marks: Plot.Markish[];
  extraBottom: number;
} {
  const { width, marginLeft, marginRight, dy, fontSize = 11, top = false } = options;
  const says = sentence(options.says);
  const frame = width - marginLeft - marginRight;
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
  const text = (
    value: number,
    position: { frameAnchor: "bottom-left" | "bottom" | "bottom-right" } | { x: number },
    y: number,
    label: string,
    textAnchor: "start" | "middle" | "end",
  ) =>
    Plot.text([value], {
      ...("x" in position
        ? { x: () => position.x, frameAnchor: top ? ("top" as const) : ("bottom" as const) }
        : top
          ? { frameAnchor: position.frameAnchor.replace("bottom", "top") as "top-left" | "top" | "top-right" }
          : position),
      dy: top ? -y : y,
      // A tick is a word as often as a number — "one year", "under 1" — and a word starts a line.
      text: () => sentence(label),
      // A bottom anchor stacks a wrapped string upward from its last line, which is what the
      // dropped line's `dy` is measured to. Spaced at `line` so each one keeps its clear space.
      lineHeight: line / fontSize,
      ...(textAnchor === "middle" ? {} : { textAnchor }),
      fill: INK.muted,
      fontSize,
      className: top ? "axis-head" : "axis-foot",
    });

  /** The labels, and the spans of the frame they occupy, measured from its left edge. */
  let ends: Plot.Markish[];
  let taken: [number, number][];
  if ("scale" in options) {
    const { domain, log = false, format, floor } = options.scale;
    const beside = options.scale.beside && {
      ...options.scale.beside,
      label: sentence(options.scale.beside.label),
    };
    const [lo, hi] = domain;
    const px = (v: number) =>
      frame * (log ? Math.log(v / lo) / Math.log(hi / lo) : (v - lo) / (hi - lo));
    const ticks = axisTicks(domain, log).filter((v) => v > lo || floor == null);
    const labels = tickLabels(ticks, format);
    const candidates = [
      ...(floor != null ? [{ value: lo, label: floor }] : []),
      ...ticks.map((value, i) => ({ value, label: labels[i]! })),
    ].map((c, i, all) => {
      const anchor: "start" | "middle" | "end" =
        i === 0 ? "start" : i === all.length - 1 ? "end" : "middle";
      const x = px(c.value);
      const w = textPx(c.label, fontSize);
      const span: [number, number] =
        anchor === "start" ? [x, x + w] : anchor === "end" ? [x - w, x] : [x - w / 2, x + w / 2];
      return { ...c, anchor, span, end: i === 0 || i === all.length - 1 };
    });
    const fixed = beside
      ? [beside].map((b) => {
          const x = px(b.value);
          const w = textPx(b.label, fontSize);
          return { ...b, anchor: "middle" as const, span: [x - w / 2, x + w / 2] as [number, number], end: false };
        })
      : [];
    // The fixed label first, then the ends, then the interior in order, each only where it clears
    // what is already kept.
    const kept: typeof candidates = [...fixed];
    for (const c of [...candidates.filter((c) => c.end), ...candidates.filter((c) => !c.end)]) {
      if (kept.every((k) => c.span[0] >= k.span[1] + 8 || c.span[1] <= k.span[0] - 8)) kept.push(c);
    }
    kept.sort((a, b) => a.value - b.value);
    ends = kept.map((c) => text(c.value, { x: c.value }, dy, c.label, c.anchor));
    taken = kept.map((c) => c.span);
  } else {
    const { low, high } = options;
    ends = [
      text(0, { frameAnchor: "bottom-left" }, dy, low, "start"),
      text(0, { frameAnchor: "bottom-right" }, dy, high, "end"),
    ];
    taken = [
      [0, textPx(low)],
      [frame - textPx(high), frame],
    ];
  }
  // A gap either side of the centre, so "fits" means legibly rather than exactly.
  const half = textPx(says) / 2;
  const fits = taken.every(([a, b]) => frame / 2 - half >= b + 12 || frame / 2 + half <= a - 12);
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
   * painted width and the estimate is the thing that was wrong. A line of its own has no reason to
   * be centred in anything: anchored to the frame's left edge it sits under the low end of the
   * scale it is talking about, and where it starts stops depending on which font the reader has.
   *
   * It does not have the whole width, though, and this said it did. It starts at `marginLeft`, so
   * what it has is the width less that, and on a 311px panel an 85-character axis name ran 51
   * units past the SVG. So it wraps to what it has, and every line past the first costs another
   * `line` of bottom margin.
   */
  if (says === "") return { marks: ends, extraBottom: 0 };
  if (fits) {
    return { marks: [...ends, text(0, { frameAnchor: "bottom" }, dy, says, "middle")], extraBottom: 0 };
  }
  const lines = wrapText(says, width - marginLeft - 4);
  return {
    marks: [
      ...ends,
      text(0, { frameAnchor: "bottom-left" }, dy + line * lines.length, lines.join("\n"), "start"),
    ],
    extraBottom: line * lines.length,
  };
}

/**
 * The dash each of a spread's rules is drawn in, and the legend swatch that names it (#611).
 *
 * Every rule was dashed `4 3`, so the TTAG spread's two — "the floor equals the formula" and "the
 * formula pays what the FY2020 regime did" — had one key swatch between them and a reader could not
 * tell which line was which. Distinct by dash rather than by hue: a rule is arithmetic, and the
 * hues are the classes. The swatch half is `.sw[data-series="rule-dotted"]` in `app.css`.
 */
const RULE_DASHES = [
  { dash: RULE.boundary[0], swatch: "rule" },
  { dash: RULE.boundary[1], swatch: "rule-dotted" },
] as const;

/** The dash of a spread's `at`th rule. Throws past the last, which wants a third pattern. */
function ruleDash(at: number): string {
  const entry = RULE_DASHES[at];
  if (!entry) throw new Error(`rule ${at + 1} has no dash of its own; add one to RULE_DASHES`);
  return entry.dash;
}

/** The legend swatch of a spread's `at`th rule, drawn in the dash {@link ruleDash} gives it. */
export function ruleSwatch(at: number): string {
  ruleDash(at);
  return RULE_DASHES[at]!.swatch;
}

/** The height past which a chart states its scale at the top as well as the foot (#611). */
const TALL = 400;

/**
 * The scale again, above a chart too tall to be read against its foot alone.
 *
 * `/counties` draws 84 rows in 1470px, so a reader at the top of it was a screen and a half from
 * the only numbers that said what a position meant. Above {@link TALL} the scale is repeated over
 * the frame — the labels the foot keeps, without the axis name, which the foot says.
 */
function tallHead(
  body: number,
  options: { width: number; marginLeft: number; marginRight: number; scale: FootScale },
): { marks: Plot.Markish[]; marginTop: number } {
  if (body <= TALL) return { marks: [], marginTop: 0 };
  return { marks: axisFoot({ ...options, dy: 12, says: "", top: true }).marks, marginTop: 22 };
}

/** What a bar chart's fill says — see `hue` on {@link barSpec}. */
export type BarHue = "formula" | "guarantee" | "plain" | "polarity";

/** The unsigned fill of each {@link BarHue}. A polarity chart is always signed, so its entry is never read. */
const BAR_HUE: Record<BarHue, string> = {
  formula: SERIES.formula,
  guarantee: SERIES.guarantee,
  plain: INK.muted,
  polarity: SERIES.formula,
};

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
  options: {
    width: number;
    max?: number;
    min?: number;
    labelChars?: number;
    /**
     * The scale, for a chart whose bars carry no direct label (#611). Without one the bars have
     * a length and no value: a corpus column drew seven panels of five bars and not one number,
     * so the only way to learn what a bar was worth was to hover every one of them. Drawn only
     * where no bar is labelled, because a labelled bar already says its value at its end.
     */
    scale?: { format: (v: number) => string; says: string };
    /**
     * What the bars' colour means, which only the caller knows (#652).
     *
     * Every bar chart was filled formula blue by default, so the share of districts *held by the
     * guarantee* was drawn in the formula's hue, and so were cash, casino receipts and the tax
     * base, which are neither. Required so a new chart has to say.
     *
     *   - `"formula"` — the bars are formula aid.
     *   - `"guarantee"` — the bars are the guarantee.
     *   - `"plain"` — the bars are neither: `INK.muted`, 5.7:1 on the card and 3.0:1 or better
     *     against the cursor's ink, so it clears both floors without a new token.
     *   - `"polarity"` — a chart of change, read as gain against loss whatever its data's sign
     *     (#651). Signed mode was entered only when a value fell below zero, so a county where every
     *     district rose dropped back to an unsigned chart and filled its subject's *gain* in the
     *     loss hue. This keeps the chart signed always.
     *
     * Any chart with a value below zero is signed and filled by polarity whatever this says: a
     * deficit drawn in the same colour as a surplus is the defect signed mode was written for.
     */
    hue: BarHue;
  },
): Spec {
  const { width } = options;
  /*
   * The `1` is a floor for counts and dollars, where a domain under one is a chart of nothing. A
   * chart of change is a chart of ratios, and the floor drew every county's change on 0 to +100%
   * when the median county's largest move was 7% (#651), so a polarity chart fits its own bars.
   */
  const max =
    options.max ??
    Math.max(...bars.map((b) => Math.abs(b.value)), options.hue === "polarity" ? Number.MIN_VALUE : 1);
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
  const signed = lowest < 0 || options.hue === "polarity";
  const negativeLabelled = labelled.filter((b) => b.value < 0);
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
  // A marked name is drawn at 600 (below), which is wider than the same name at normal weight, so
  // it is counted at the width it is drawn at — see {@link BOLD_WIDENS}.
  const longestLabel = Math.max(
    0,
    ...bars.map((b) => b.label.length * (b.current ? BOLD_WIDENS : 1)),
  );
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
   * Its fill is the chart's, whatever that is (#652). It was the guarantee's hue, which made
   * Ohio's bar say "guarantee" in a chart of local shares, and in a chart of change made a
   * district's gain the colour of a loss (#651). A fill already means something — the hue the
   * caller chose, or polarity — and cannot mean "this one" as well. So the subject is marked by a
   * {@link SUBJECT} ring and by its name in primary ink at 600, and weight is what survives a
   * monochrome print and a forced-colours mode.
   *
   * The ring is a mark of its own, drawn behind the bar and 2px larger. It was a `stroke` on the
   * bar, which is a hover target, and `svg.plot [data-hover] { stroke: transparent }` widens every
   * target with an invisible stroke: CSS beats a presentation attribute, so on the built page the
   * ring #651 shipped was never painted. Behind, it intercepts no pointer, and it is not in
   * `.bar-fill`, so the tooltips' index is untouched.
   */
  const marked = bars.filter((b) => b.current);
  const plain = bars.filter((b) => !b.current);
  /*
   * A reference row is hollow: an outline in its own mark under a transparent hit bar.
   *
   * Not a third fill. The palette's neutral is 2.35:1 on the card, and a hover target's fill is
   * held to what the cursor's ink clears (`tests/e2e/cursor.spec.ts`); transparent is the hit
   * layers' own fill and clears by construction. And not the bar's own stroke, which the hover
   * layer's CSS paints transparent — the reason the subject's ring is its own mark too.
   */
  const references = bars.filter((b) => b.reference);
  const base = signed
    ? (b: Bar) => (b.value < 0 ? SERIES.guarantee : SERIES.formula)
    : BAR_HUE[options.hue];

  const marginRight = gutter(width, longest > 0 ? 16 + longest * 7.2 : 20);
  /*
   * A direct label on a negative bar is written leftwards from the bar's end, so the domain gets
   * room for it rather than letting it collide with the category names outside the frame.
   *
   * Eight per cent of the span is enough where the negative side is most of it, and was what every
   * signed chart had. A chart of change is the opposite shape: Franklin runs from −2% to +25%, and
   * eight per cent of that is 20px for a 35px "−2.2%", so the county chart drew its subject's value
   * through its own name at every width (#651). There the room is sized in pixels, for
   * each labelled bar, from the frame the bar is drawn in: `frame × (v − floor) / (max − floor)`
   * is where the bar ends, and it has to clear the label, the `dx` and a margin for a wider font.
   */
  const frame = width - nameGutter - marginRight;
  const clear = (b: Bar) => {
    const r = Math.min(0.9, (textPx(b.direct!) + 12) / frame);
    return (b.value - r * max) / (1 - r);
  };
  const floor = !signed
    ? 0
    : options.hue === "polarity"
      ? Math.min(lowest, ...negativeLabelled.map(clear))
      : lowest - (negativeLabelled.length > 0 ? (max - lowest) * 0.08 : 0);
  const foot =
    options.scale && labelled.length === 0
      ? axisFoot({
          width,
          marginLeft: nameGutter,
          marginRight,
          dy: 16,
          says: options.scale.says,
          scale: { domain: [floor, max], format: options.scale.format },
        })
      : null;
  const marginBottom = foot ? 22 + foot.extraBottom : 0;

  return {
    options: {
      width,
      height: bars.length * rowHeight + marginBottom,
      // Bounded below so short-label charts keep their existing proportions, and above so a very
      // long name costs the bars width rather than running off the plot.
      marginLeft: nameGutter,
      // Room at the right for the longest direct label actually present. Without it the largest
      // bar's value runs off the viewBox and is clipped — and it is the one most worth reading.
      marginRight,
      marginTop: 0,
      marginBottom,
      x: { axis: null, domain: [floor, max] },
      y: { axis: null, domain: bars.map((b) => b.label), padding: 0.47 },
      marks: [
        // The subject's ring, under its bar. See "The bar the chart was built to locate" above.
        ...(marked.length > 0
          ? [
              Plot.barX(marked, {
                y: "label",
                ...(signed
                  ? { x1: 0, x2: (b: Bar) => b.value }
                  : { x: (b: Bar) => Math.abs(b.value) }),
                inset: -2,
                fill: SUBJECT,
                className: "bar-subject",
                ...(signed ? {} : { rx2: 5 }),
              }),
            ]
          : []),
        ...(references.length > 0
          ? [
              Plot.barX(references, {
                y: "label",
                ...(signed
                  ? { x1: 0, x2: (b: Bar) => b.value }
                  : { x: (b: Bar) => Math.abs(b.value) }),
                inset: 0.75,
                fill: "none",
                stroke: INK.muted,
                strokeWidth: 1.5,
                className: "bar-reference",
                ...(signed ? {} : { rx2: 4 }),
              }),
            ]
          : []),
        Plot.barX(bars, {
          y: "label",
          // One mark, not two: `draw` maps tooltips onto `.bar-fill > *` by index and
          // throws if the counts disagree, so splitting positives from negatives here would
          // reorder the marks out from under the hover layer.
          ...(signed
            ? { x1: 0, x2: (b: Bar) => b.value }
            : { x: (b: Bar) => Math.abs(b.value) }),
          // A constant where the chart is unsigned, and not merely as an optimisation: Plot
          // hoists a constant fill onto the group and pushes a channel down onto each rect, so a
          // function here would move where the colour lives on every one of the nine charts built
          // on this spec — including the two the theme tests read `.bar-fill`'s computed fill from.
          // A channel where a reference row needs its own fill, and only there.
          fill:
            references.length === 0
              ? base
              : (b: Bar) => (b.reference ? "transparent" : typeof base === "string" ? base : base(b)),
          className: "bar-fill",
          // Rounded at the data end, square at the baseline: the bar grows from the axis and
          // rounding that end would detach it from the thing it is measured against. Not in
          // signed mode — see above.
          ...(signed ? {} : { rx2: 4 }),
        }),
        ...(signed ? [Plot.ruleX([0], RULE.zero)] : []),
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
                // The budget is what is left of the gutter after the `dx`, at the bold width.
                lineWidth: lineWidth((nameGutter - 10) / BOLD_WIDENS),
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
        ...(foot ? foot.marks : []),
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
 * Exported because callers reason about it. `/reach` subsets its cloud to a selection only on
 * request and only above this — 79 of Ohio's 88 counties hold fewer districts than it — and both
 * the page's default and its refusal are argued from the figure in prose, so the number those
 * sentences say has to be this number rather than a second copy of it.
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
 *
 * `banded` is held to 3:1 and was not. This comment said its end step sat "near 2.2:1" at 0.62;
 * composited over the card it measured 1.60 light and 1.56 dark (#615), because 2.2 is the ramp's
 * figure at full opacity and the alpha comes off it. This is the light figure and the one written
 * onto the SVG; dark paints at 0.65 through `--banded-opacity`, because above 0.66 the keyboard
 * ring stops clearing the dark ramp's pale top step (the token says more). Either way the middle
 * and outer steps clear 3:1. The light step cannot at any alpha — `--ordinal-1` is 2.20:1 fully
 * opaque — and is exempt by name in `palette.spec.ts`, relieved by the legend the ramp obliges.
 */
export const DOT = {
  radius: { plain: 2.4, muted: 1.6 },
  opacity: { banded: 0.8, plain: 0.45, muted: 0.22 },
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

/**
 * Left to right, then upwards: the order the arrow keys walk a cloud in.
 *
 * The cursor walks the hit layer in document order, which is the order a caller passed the data
 * in. For a cloud that was alphabetical, and ArrowRight on `/outcomes` jumped 137px on average
 * across an 800px plot — Manchester, then Akron, then Alliance, then back to Ashland (#614). The
 * bar, rank and range forms never had it, because their data order is their axis order.
 *
 * Sorted once, before any layer is drawn, so the hit layer and the marks it is paired with by
 * index are drawn from the same array still.
 */
function byPosition<T extends { x: number; y: number }>(marks: readonly T[]): T[] {
  return [...marks].sort((a, b) => a.x - b.x || a.y - b.y);
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
    /**
     * Draw the lines where a classification stops being defined, under the cloud.
     *
     * The third and last reference geometry, and the note on `identity` is still the argument for
     * why this is not a general draw-a-line API: an arbitrary line through a cloud is a claim, and
     * the claims on this site are computed in `crates/`. A rule here is not a claim — it is a
     * **definition**, and one the axes make arithmetically. On two terms that sum to a log
     * multiple, "the multiple is one" is `y = -x` and "the formula pays per pupil what the prior
     * regime did" is `y = 0`. Nothing about either goes stale when a district moves, which is why
     * neither names a figure where a fit names two.
     *
     * Segments in the data's own units, for {@link Fit}'s reason: evaluating a slope and an
     * intercept here would be a second arithmetic in a second language for one crate computation.
     * `crates/figures` constrains them to whole-frame lines on the way in, so a caller cannot
     * smuggle a claim through as a definition.
     *
     * **The labels go in the legend, not on the picture.** A spread carries more than one rule and
     * draws its panels as small multiples 280px wide; an end label on each would cross the cloud
     * at every width, and a spread is classed so it carries a legend already. They are set as each
     * segment's `ariaLabel` so the line is named where a legend cannot be read.
     */
    rules?: { label: string; from: { x: number; y: number }; to: { x: number; y: number } }[];
  },
): Spec | null {
  // Two points are not a cloud. Same rule as the line forms, for the same reason: a scatter of
  // three districts would read as a finding about a population that has not been measured.
  if (points.length < MIN_CLOUD) return null;
  points = byPosition(points);

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

  /*
   * `"neutral"` and no series at all land in the same place and mean different things: one is an
   * unsplit population drawn in one colour, the other is a point a split does not classify. See
   * {@link ScatterPoint.series}. There is no branch for it because there is nothing to branch to —
   * the palette's neutral is what both are drawn in.
   */
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
  const title = yTitle(width, marginLeft, axes.y.label + (axes.y.log ? LOG_SCALE : ""), 28);
  const marginTop = title.marginTop;
  const foot = axisFoot({
    width,
    marginLeft,
    marginRight,
    dy: 20,
    says: axes.x.label + (axes.x.log ? LOG_SCALE : ""),
    scale: {
      domain: axes.x.log ? [xMin, xMax] : [xMin - xPad, xMax + xPad],
      log: axes.x.log,
      format: axes.x.format,
    },
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

        // The boundaries, drawn where the identity and the fit are drawn and read the same way:
        // under the cloud, because they are what it is read against. Dashed, which is the one
        // thing separating them from those two — a definition is not a measurement, and a spread
        // may carry several where a cloud carries one fitted line.
        ...(options.rules ?? []).map((rule, at) =>
          Plot.line([rule.from, rule.to], {
            x: "x",
            y: "y",
            stroke: INK.rule,
            strokeWidth: 1.5,
            strokeDasharray: ruleDash(at),
            // A function, not the string: Plot reads a string `ariaLabel` as a field name, finds
            // nothing on either end point, and drops the whole segment as undefined. The line
            // then renders as an empty `<g>` — a boundary that is silently not there, which is
            // the one failure this mark must not have.
            ariaLabel: () => rule.label,
            className: "scatter-rule",
            clip: true,
          }),
        ),

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
          /* A ring is a stroke channel only where some point wears one, so a chart without
             rings carries no per-dot stroke attribute — six hundred of them on every scatter. */
          ...(points.some((p) => p.ring && !p.muted)
            ? { stroke: (p: ScatterPoint) => (p.ring && !p.muted ? INK.primary : "none"), strokeWidth: 1 }
            : { stroke: "none" }),
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
                Plot.text([lastOf(trace.points)], {
                  x: "x",
                  y: "y",
                  dx: 8,
                  text: () => trace.label,
                  textAnchor: "start",
                  // A banded trace is not labelled here, so the label is always one of the pair.
                  fill: SERIES_TEXT[trace.series],
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
        /*
         * The y scale's round ends, each at its own height, as the foot's are at their own x.
         *
         * These were the data's extremes printed at the frame's corners, over a domain padded past
         * them. Each label hangs inward from its value, the top one down and the bottom one up, so
         * neither reaches the axis name above the frame. Those two sat on the frame's top edge
         * once and "Performance Index" and the 113 it labels were drawn through each other.
         */
        ...yEnds(
          axes.y.log ? [yMin, yMax] : [yMin - yPad, yMax + yPad],
          axes.y.format,
          marginLeft,
          axes.y.log,
        ),
        title.mark,

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
export interface Box {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** A direct label's offset and the lines it is drawn in: one, or the name broken in two. */
export interface Placement {
  at: LabelAt;
  lines: readonly string[];
  /** Where it lands, in the frame's pixels: what {@link placeLabels} costed it at. */
  box: Box;
}

/**
 * How far a label's centre moves off its offset to keep a second line clear of its own mark.
 *
 * A wrapped label is drawn centred on its anchor, so above and below a dot it would grow back
 * across the dot by half a line. Beside one it grows up and down evenly, which the dot's keep-out
 * square already allows for.
 */
function labelShift(at: LabelAt, lines: number): number {
  const half = ((lines - 1) * LABEL_LINE) / 2;
  return at === "above" ? -half : at === "below" ? half : 0;
}

/** Where a label drawn at `at` would land, given its mark's position and its lines. */
function labelBox(at: LabelAt, px: number, py: number, lines: readonly string[]): Box {
  const { dx, dy, textAnchor } = LABEL_AT[at];
  const w = Math.max(...lines.map((line) => textPx(line)));
  const h = LABEL_LINE * lines.length;
  const left = px + dx + (textAnchor === "start" ? 0 : textAnchor === "end" ? -w : -w / 2);
  const top = py + dy + labelShift(at, lines.length) - h / 2;
  return { left, top, right: left + w, bottom: top + h };
}

/** How much of each other two boxes cover, in square pixels. Zero where they merely touch. */
function overlap(a: Box, b: Box): number {
  const w = Math.min(a.right, b.right) - Math.max(a.left, b.left);
  const h = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
  return w > 0 && h > 0 ? w * h : 0;
}

/** How much of a box lies outside the frame, in square pixels. */
function outside(box: Box, frame: Box): number {
  return (box.right - box.left) * (box.bottom - box.top) - overlap(box, frame);
}

/**
 * A name broken at the space nearest its middle, so the two lines are as near one length as the
 * words allow. Null for a name of one word, which has nowhere to break.
 */
function halves(label: string): [string, string] | null {
  const words = label.split(" ");
  if (words.length < 2) return null;
  let best: [string, string] | null = null;
  for (let at = 1; at < words.length; at += 1) {
    const pair: [string, string] = [words.slice(0, at).join(" "), words.slice(at).join(" ")];
    if (!best || Math.max(...pair.map((l) => l.length)) < Math.max(...best.map((l) => l.length))) {
      best = pair;
    }
  }
  return best;
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
 * # Where no offset fits at all
 *
 * A name wider than the room beside its mark fits at none of the four, and the placer used to
 * fall through to the first of them anyway — so on the narrow TTAG plane `[M] mirrored beside
 * [L], [M], [O]` was drawn rightward from a mark near the right edge and cut at the SVG's, and
 * `[M] mirrored inside [H]` landed on another dot's mark because the default was never costed.
 * So a name with no clear place on one line is tried again broken in two, at all four offsets,
 * and only where neither fits inside the frame does the placer settle for the candidate with the
 * least of it outside. A cut name is the last resort rather than the default.
 *
 * The frame a label has is the drawing's width, not the plot's. The side margins are empty but for
 * the y-axis's two ends, which the caller passes as `occupied`, and holding labels to the plot
 * left `[M] mirrored inside [H]` with no clear place on the narrow TTAG plane: its one clear place
 * is broken in two to the left of its mark, two units into a margin with nothing in it there.
 *
 * # Why the order is searched
 *
 * Greedy is order-dependent, and the order is the manifest's, which knows nothing about the
 * picture. On the narrow TTAG plane (#609) the rolling count came first and took the one place
 * above its mark that was clear *then* — the place `[M] mirrored inside [H]` needed — and the
 * mirror was left to overlap it by half a pixel in this model, which the runner's taller font made
 * a collision. The other way round, both are clear. So where the manifest's order leaves any
 * overlap, each mark is tried first in turn, and the least costly of those orders is kept: more
 * than one pass, still deterministic, and a tie keeps the earlier order.
 *
 * Deterministic in the order the marks arrive, which is the manifest's order, so the same document
 * draws the same picture on every build.
 */
export function placeLabels(
  marks: readonly { label: string; px: number; py: number }[],
  frame: Box,
  occupied: readonly Box[] = [],
): Placement[] {
  const manifest = marks.map((_, i) => i);
  let best = placeInOrder(marks, manifest, frame, occupied);
  if (best.outside === 0 && best.overlap === 0) return best.placements;
  for (const first of manifest.slice(1)) {
    const tried = placeInOrder(marks, [first, ...manifest.filter((i) => i !== first)], frame, occupied);
    if (
      tried.outside < best.outside ||
      (tried.outside === best.outside && tried.overlap < best.overlap)
    ) {
      best = tried;
    }
  }
  return best.placements;
}

/**
 * One greedy pass of {@link placeLabels}, visiting the marks in `order`. The placements come back
 * in the marks' own order, with how much of them lies outside the frame and how much they cover
 * of each other and of the marks, both in square pixels.
 */
function placeInOrder(
  marks: readonly { label: string; px: number; py: number }[],
  order: readonly number[],
  frame: Box,
  occupied: readonly Box[],
): { placements: Placement[]; outside: number; overlap: number } {
  const taken: Box[] = [
    ...occupied,
    ...marks.map((m) => ({
      left: m.px - DOT_KEEP_OUT,
      top: m.py - DOT_KEEP_OUT,
      right: m.px + DOT_KEEP_OUT,
      bottom: m.py + DOT_KEEP_OUT,
    })),
  ];
  const out: Placement[] = new Array(marks.length);
  let spilled = 0;
  let covered = 0;
  for (const index of order) {
    const mark = marks[index]!;
    const split = halves(mark.label);
    const forms: (readonly string[])[] = [[mark.label], ...(split ? [split] : [])];
    let best: Placement | null = null;
    // The least-overlapping candidate inside the frame. One line is tried before two at every
    // offset, so a name is broken only where whole it would collide or leave the frame.
    let least = Infinity;
    search: for (const lines of forms) {
      for (const at of LABEL_ORDER) {
        const box = labelBox(at, mark.px, mark.py, lines);
        if (outside(box, frame) > 0) continue;
        const cost = taken.reduce((sum, b) => sum + overlap(box, b), 0);
        if (cost < least) {
          least = cost;
          best = { at, lines, box };
        }
        if (cost === 0) break search;
      }
    }
    // Nothing fits: whichever candidate leaves the least of itself outside.
    if (!best) {
      let least = Infinity;
      for (const lines of forms) {
        for (const at of LABEL_ORDER) {
          const box = labelBox(at, mark.px, mark.py, lines);
          const cost = outside(box, frame);
          if (cost < least) {
            least = cost;
            best = { at, lines, box };
          }
        }
      }
      spilled += least;
    } else {
      covered += least;
    }
    out[index] = best!;
    taken.push(best!.box);
  }
  return { placements: out, outside: spilled, overlap: covered };
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
  const title = yTitle(width, marginLeft, axes.y.label, 28);
  const marginTop = title.marginTop;
  const marginRight = gutter(width, 40);
  const foot = axisFoot({
    width,
    marginLeft,
    marginRight,
    dy: 20,
    says: axes.x.label,
    scale: { domain: [xLo, xHi], format: axes.x.format },
  });
  const marginBottom = 40 + foot.extraBottom;
  const height = options.height ?? 420;

  const frame: Box = {
    left: marginLeft,
    top: marginTop,
    right: width - marginRight,
    bottom: height - marginBottom,
  };
  // The dots, their hit layer and its readings, in the order the arrow keys walk them. Not the
  // labels: the placer is order-dependent, and the order it was tuned in is the caller's.
  const walk = byPosition(places);
  const at = places.map((place) => ({
    label: place.label,
    px: frame.left + ((place.x - xLo) / (xHi - xLo)) * (frame.right - frame.left),
    py: frame.bottom - ((place.y - yLo) / (yHi - yLo)) * (frame.bottom - frame.top),
  }));
  // Labels have the drawing's width less the 4 units every edge label here keeps, between the
  // title and the foot. The y-axis's two ends are the only text in the side margins.
  const [yHiLabel, yLoLabel] = tickLabels([yHi, yLo], axes.y.format);
  const ends = [yHiLabel!, yLoLabel!].map((text) => 4 + textPx(text));
  const placed = placeLabels(
    at,
    { left: 4, top: frame.top, right: width - 4, bottom: frame.bottom },
    [
      { left: 4, top: frame.top + 3, right: ends[0]!, bottom: frame.top + 3 + LABEL_LINE },
      { left: 4, top: frame.bottom - LABEL_LINE, right: ends[1]!, bottom: frame.bottom },
    ],
  );

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
        Plot.ruleX([0], { ...RULE.zero, className: "plane-zero" }),
        Plot.ruleY([0], { ...RULE.zero, className: "plane-zero" }),

        Plot.dot(walk, {
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

        // One text mark per placement and line count, because `dx`, `dy` and `textAnchor` are
        // constants in Plot and a wrapped label's `dy` is shifted by its second line. Text is not
        // in the hover selector, so splitting these reorders nothing that is indexed.
        ...LABEL_ORDER.flatMap((which) =>
          [1, 2].flatMap((count) => {
            const mine = places.filter(
              (_, i) => placed[i]!.at === which && placed[i]!.lines.length === count,
            );
            if (mine.length === 0) return [];
            const lines = new Map(mine.map((place) => [place, placed[places.indexOf(place)]!.lines]));
            return [
              Plot.text(mine, {
                x: "x",
                y: "y",
                ...LABEL_AT[which],
                dy: LABEL_AT[which].dy + labelShift(which, count),
                text: (place: Place) => lines.get(place)!.join("\n"),
                // Spacing for the broken names only: Plot scales a centred line's offset by it,
                // so a one-line name given it would move off the mark it was placed against.
                ...(count > 1 ? { lineHeight: LABEL_LINE / 11 } : {}),
                fill: INK.primary,
                fontSize: 11,
                className: "plane-label",
              }),
            ];
          }),
        ),

        // Both ends of both scales, as a cloud carries them: a frame with no numbers on it is a
        // texture, and here the numbers are what say how far from the rule in force these get.
        ...foot.marks,
        Plot.text([0], {
          frameAnchor: "top-left",
          dx: -marginLeft + 4,
          dy: 3,
          text: () => yHiLabel!,
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([0], {
          frameAnchor: "bottom-left",
          dx: -marginLeft + 4,
          text: () => yLoLabel!,
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        title.mark,

        // The hit layer, above everything, as every pointed-at form here draws one.
        Plot.dot(walk, {
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
      text: walk.map((p) => escapeHtml(p.hover)),
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
 *
 * # Markers, and why they are placed in pixels
 *
 * `options.markers` draws a dashed rule **across** the rows at a fractional row position, for a
 * threshold in the plan that the ordering is supposed to vary with. The y-scale here is
 * categorical, so there is no scale to ask for the position of "3.16 rows down" — and reaching
 * into d3's band scale for its step and padding would be reading Plot's internals to re-derive a
 * number the layout already knows. `Plot.frame({ anchor: "top", insetTop })` is the first-class
 * form: the frame mark draws its top edge at `marginTop + insetTop`, so the offset is computed
 * against the plot area's own height and the rule lands where it should at any width.
 */
export function rangeSpec(
  rows: Range[],
  axis: { label: string; format: (v: number) => string; log?: boolean },
  options: {
    width: number;
    markers?: RangeMarker[];
    /**
     * Text drawn beside each row's high end, for the figure the rows are sorted by (#656).
     *
     * `/counties` sorts its 84 rows by the high-to-low ratio and printed it nowhere, so on a log
     * axis the reader had to compare segment lengths to recover an order they had been given.
     */
    rowLabel?: (row: Range) => string;
  },
): Spec | null {
  if (rows.length < 2) return null;

  const values = rows.flatMap((r) => [r.low, r.high]);
  const min = Math.min(...values);
  const max = Math.max(...values);

  // 16 for 11px names (#662). At 14 the row held 10px type and drew 11px through its neighbours.
  const rowHeight = 16;
  const longest = Math.max(...rows.map((r) => r.label.length));
  // A 16px row cannot hold a second line, so this gutter is capped rather than wrapped: a name
  // too long for a phone's frame is drawn shorter, not folded into the row below it.
  const { width, markers = [], rowLabel } = options;
  const marginLeft = gutter(width, Math.max(70, Math.min(150, Math.round(longest * 6.8) + 10)));
  /* Room for the widest row label past the frame's right edge, where the row holding the maximum
     draws it; any other row's sits inside the frame, past its own high end. */
  const ROW_LABEL_DX = DOT_RADIUS + 5;
  const marginRight = rowLabel
    ? 16 + Math.ceil(ROW_LABEL_DX + Math.max(...rows.map((r) => textPx(rowLabel(r)))))
    : 16;
  const foot = axisFoot({
    width,
    marginLeft,
    marginRight,
    dy: 16,
    says: axis.label + (axis.log ? LOG_SCALE : ""),
    scale: { domain: rangeDomain(min, max, axis.log), log: axis.log, format: axis.format },
  });

  const marginBottom = 22 + foot.extraBottom;
  const head = tallHead(marginBottom + rows.length * rowHeight, {
    width,
    marginLeft,
    marginRight,
    scale: { domain: rangeDomain(min, max, axis.log), log: axis.log, format: axis.format },
  });
  const marginTop = head.marginTop;
  /*
   * The rows' band plus the margins, not the rows' band with the margins taken out of it.
   *
   * This was `rows.length * rowHeight`, and the foot's margin came out of it — which on 84 rows is
   * a quarter of a row each and on six is nearly four units of every row's fourteen. The six-row
   * sextile chart on the base-cost node drew 10-unit names in 7.7–10.3-unit rows and ran each into
   * the next. A row is `rowHeight` at any count, so the foot is added to it.
   */
  const height = marginTop + marginBottom + rows.length * rowHeight;
  // The rows' own band, which is what a marker's row position is a fraction of. `at` is clamped
  // rather than dropped: a threshold past the end of the ordering is a real thing to say, and
  // saying it at the last edge is truer than saying nothing.
  const area = height - marginTop - marginBottom;
  const across = markers.map((marker) => ({
    label: marker.label,
    at: Math.min(Math.max(marker.at, 0), rows.length),
  }));

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
        type: axis.log ? "log" : "linear",
        domain: rangeDomain(min, max, axis.log),
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
          /* The middle step and not the light one. `--ordinal-1` is 2.20:1 against the card at full
             opacity in both themes, and this dot is a data mark a reader has to find on a 16px row
             (#615). `--ordinal-2` is 4.30 light and 5.47 dark, and is still the lower of the two
             shades, so a low end and a high end stay one measure at two points. */
          fill: ORDINAL[1],
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
          // The axis foot's size (#662): at 10 the names painted at 12.5px under a 13.8px foot.
          fontSize: 11,
          className: "range-label",
        }),
        ...(rowLabel
          ? [
              Plot.text(rows, {
                y: "label",
                x: "high",
                dx: ROW_LABEL_DX,
                text: (r: Range) => rowLabel(r),
                textAnchor: "start",
                fill: INK.muted,
                fontSize: 11,
                className: "range-row-label",
              }),
            ]
          : []),
        // Across the rows, under the foot and over the spans: it is a position in the plan
        // rather than a value of any row, so it is dashed, and it carries no text of its own —
        // a 16px row has nowhere to put one, and the legend says what it is.
        ...across.map((marker) =>
          Plot.frame({
            anchor: "top",
            insetTop: (marker.at / rows.length) * area,
            ...RULE.reference,
            className: "range-marker",
          }),
        ),
        ...foot.marks,
        ...head.marks,
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
  // grows to hold two lines rather than the name being cut — the 16px row `rangeSpec` draws
  // cannot, which is why that form truncates and this one wraps.
  const wanted = Math.max(90, Math.min(260, Math.round(longest * 6.8) + 12));
  const marginLeft = gutter(width, wanted);
  /*
   * Wrapped here rather than by Plot's `lineWidth`, so the row knows how many lines it holds. At
   * 11px (#662) a name that took two lines at 10 took three, and a row sized for two drew it
   * through its neighbours on `/bounds` at 375. The marked row is bold and wraps on a narrower
   * budget — see {@link BOLD_WIDENS}.
   */
  const wrapped = new Map(
    rows.map((r) => [
      r.label,
      wrapText(r.label, (marginLeft - 10) / (r.marked != null ? BOLD_WIDENS : 1)).join("\n"),
    ]),
  );
  const lines = Math.max(...[...wrapped.values()].map((t) => t.split("\n").length));
  const rowHeight = lines > 1 ? Math.max(28, 14 * lines) : 16;
  const name = (r: Rank) => wrapped.get(r.label) ?? r.label;

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
  const phrase = Math.max(0, ...marked.map((r) => textPx(r.marked ?? "", 11)));
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
    says: `${axis.label}${LOG_SCALE}`,
    scale: {
      domain: [floor, max],
      log: true,
      format: axis.format,
      // The zero row sits at the floor, which is below every value the scale can hold. "0" there
      // named a point a log axis has no position for.
      ...(smallest > 0 ? {} : { floor: `under ${axis.format(Math.min(...positive))}` }),
    },
  });

  const head = tallHead(22 + foot.extraBottom + rows.length * rowHeight, {
    width,
    marginLeft,
    marginRight,
    scale: {
      domain: [floor, max],
      log: true,
      format: axis.format,
      ...(smallest > 0 ? {} : { floor: `under ${axis.format(Math.min(...positive))}` }),
    },
  });

  return {
    options: {
      width,
      // `rangeSpec`'s height, for the defect recorded there: the foot is added to the rows.
      height: head.marginTop + 22 + foot.extraBottom + rows.length * rowHeight,
      marginLeft,
      marginRight,
      marginTop: head.marginTop,
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
         * Split by subject the way the labels below are and `draw` would index the wrong
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
              ? (r: Rank) => (r.marked == null ? SERIES.formula : SUBJECT)
              : SERIES.formula,
          stroke: "none",
          className: "rank-dot",
        }),
        Plot.text(plain, {
          y: "label",
          frameAnchor: "left",
          dx: -8,
          text: name,
          textAnchor: "end",
          fill: INK.secondary,
          fontSize: 11,
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
                text: name,
                textAnchor: "end",
                fill: INK.primary,
                fontSize: 11,
                fontWeight: 600,
                className: "rank-label current",
              }),
              Plot.text(marked, {
                y: "label",
                x: at,
                dx: 8,
                text: "marked",
                textAnchor: "start",
                fill: INK.primary,
                fontSize: 11,
                className: "rank-mark",
              }),
            ]
          : []),
        ...foot.marks,
        ...head.marks,
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
 * How a strip's members are painted, and why not in the cloud's neutral.
 *
 * They were `--neutral-mark` at 0.5 and 0.7: 1.48 and 1.77 against the card in light (#615). A
 * county strip draws sixteen of them, so they are not a density the eye integrates, as a scatter's
 * six hundred are — each one is a district a reader is meant to find. `--neutral-mark` cannot get
 * there at any alpha: it is 2.35:1 light fully opaque. So the fill is the muted ink, the grey the
 * chart's own annotation is set in — still grey, still no hue — at 4.08 light and 4.56 dark where
 * every member is drawn, and 3.03 and 3.51 where only the outliers are.
 *
 * What is **not** held to 3:1, by name in `palette.spec.ts`: the box behind the dots and the
 * whisker and axis rules. They are the frame a reader locates the dots in, not data a reader reads
 * off, and the median rule drawn across the box is the ink.
 */
export const STRIP = {
  fill: INK.muted,
  opacity: { dots: 0.85, outliers: 0.7 },
  box: 0.28,
} as const;

/**
 * The `q`th quantile of an ascending series by **nearest rank**, or zero for an empty one.
 *
 * Deliberately not `stats.percentile`, which interpolates and is the definition every median this
 * site *publishes* now uses. A box plot is different in kind: its rule and its hinges are drawn at
 * observations, and a caller describing the box in prose has to name the same three values the
 * marks sit on. Interpolating here would place the rule between two dots and then describe it as
 * a figure the population does not contain.
 *
 * Exported for exactly that reason. `Strip.astro` writes the `description` for the strip charts —
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
     * How the strip's scale labels are written. Required: a strip is drawn with no axis, and
     * without these a dot four fifths along means nothing at all.
     *
     * The labels used to be an HTML row under the SVG, its two ends pinned to the corners of a
     * domain padded past the data, so "$1,350,078" sat about 25px of 800 from the dot it named.
     * They are drawn here now, at round values, where those values fall (see {@link axisFoot}).
     */
    format: (v: number) => string;
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
  const whiskerLow = sorted.find((v) => v.value >= lowFence)?.value ?? firstOf(sorted).value;
  const whiskerHigh = [...sorted].reverse().find((v) => v.value <= highFence)?.value ?? lastOf(sorted).value;

  const min = firstOf(sorted).value;
  const max = lastOf(sorted).value;
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
   * the strip carried nothing to say which. `histogramSpec` draws a rule at zero and labels it
   * "no change" for exactly that reason, and gives a bin straddling it a neutral fill.
   *
   * Detected from the domain rather than passed in, so it appears wherever the condition holds and
   * cannot be forgotten at a call site. The rule is `RULE.zero`, solid as every baseline is (#662):
   * it was dashed muted ink here and solid rule grey on signed bars. It is not confused with the
   * marker, which is 2.5px of `SUBJECT` ink against this 1px of recessive rule grey.
   */
  const min0 = firstOf(sorted).value;
  const max0 = lastOf(sorted).value;
  const crossesZero = min0 < 0 && max0 > 0;
  // Room under the strip for one line of labels. Every strip has the line now, so the six on
  // `/districts` stay one row apart whichever of them crosses zero.
  const height = 64;
  const foot = axisFoot({
    width: options.width,
    marginLeft: 2,
    marginRight: 2,
    dy: 15,
    says: "",
    fontSize: 10,
    scale: {
      domain: [min - pad, max + pad],
      format: options.format,
      ...(crossesZero ? { beside: { value: 0, label: "no change" } } : {}),
    },
  });
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
      marginBottom: 22,
      x: { axis: null, domain: [min - pad, max + pad] },
      y: { axis: null, domain: [-19, 19] },
      marks: [
        // Under everything, because it is what the dots are read against rather than a mark
        // among them.
        ...(crossesZero ? [Plot.ruleX([0], RULE.zero)] : []),
        ...foot.marks,

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
                fillOpacity: STRIP.box,
                rx: 3,
              }),
              Plot.ruleX([med], { y1: -11, y2: 11, stroke: INK.secondary, strokeWidth: 2 }),
            ]
          : []),

        Plot.dot(drawn, {
          x: "value",
          y: (_d: DistributionValue, i: number) => (dots ? lane(i) : 0),
          r: dots ? 2.8 : 2.4,
          fill: STRIP.fill,
          fillOpacity: dots ? STRIP.opacity.dots : STRIP.opacity.outliers,
          stroke: "none",
          className: "dist-dot",
        }),

        // The member the page is about: full height, in `SUBJECT` ink (#652), above everything, and
        // labelled. Taller than the median rule beside it, which is how the two are told apart.
        ...(options.marker
          ? [
              Plot.ruleX([options.marker.value], {
                y1: -17,
                y2: 17,
                stroke: SUBJECT,
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
  const first = firstOf(bins);
  const last = lastOf(bins);
  const histEnds = tickLabels([first.from, last.to], format);
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
        Plot.ruleY([0], RULE.zero),
        ...(crossesZero
          ? [
              Plot.ruleX([0], RULE.zero),
              Plot.text([0], {
                x: 0,
                frameAnchor: "bottom",
                dy: 18,
                text: () => "No change",
                fill: INK.muted,
                fontSize: 11,
              }),
            ]
          : []),
        Plot.text([first.from], {
          frameAnchor: "bottom-left",
          dy: 18,
          text: () => histEnds[0]!,
          textAnchor: "start",
          fill: INK.muted,
          fontSize: 11,
        }),
        Plot.text([last.to], {
          frameAnchor: "bottom-right",
          dy: 18,
          text: () => histEnds[1]!,
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
  options: {
    width: number;
    /**
     * What each line is called, written into its end label: "Received $114M", "Formula $87.2M".
     *
     * For a fan drawn beside its reference, where the two ends are two quantities and the finding
     * is the distance between them (#655). Without names the end labels were two bare numbers and
     * which was which lived in a key under the chart. `point` names the band's upper bound — its
     * only bound when the band has collapsed to a line — and `reference` the reference's end.
     */
    labels?: { point: string; reference: string };
  },
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
  const last = lastOf(points);
  // A band that never opens: for a district the guarantee pays, aid does not move with enrollment
  // at all. Not a degenerate chart to hide — the flat line is the finding — but two identical
  // bound labels stacked on each other would be noise.
  const degenerate = last.high - last.low < Math.max(1e-9, Math.abs(last.point) * 1e-6);

  const bounds = degenerate ? [last.high] : [last.high, last.low];
  const named = (name: string | undefined, value: number) =>
    name ? `${sentence(name)} ${format(value)}` : format(value);
  const boundText = (v: number, i: number) => named(i === 0 ? options.labels?.point : undefined, v);
  const referenceText = hasReference ? named(options.labels?.reference, last.reference ?? 0) : "";

  /*
   * Which half of the pair the banded series wears.
   *
   * Alone, the band is the formula run at projected enrollment, and it takes the formula's hue.
   * Beside a reference it is not: the reference is the formula's own answer, so the band is what
   * the district *receives* — for a guaranteed district, the guarantee's fixed amount. It was drawn
   * in the formula's hue regardless and the reference in the guarantee's, which reversed the pair
   * against "Where the aid comes from" a few hundred pixels up the same page (#607).
   */
  const banded = hasReference ? SERIES.guarantee : SERIES.formula;

  /*
   * The end labels live here, sized to the longest string drawn as `seriesSpec` sizes its own.
   * It was a fixed 104px, which held two bare amounts and would have cut "Received $114M" short.
   * Capped like every other gutter, because a named label is a sixth of the wide frame and a
   * third of the narrow one.
   */
  const longestEnd = Math.max(referenceText.length, ...bounds.map((v, i) => boundText(v, i).length));
  const marginRight = gutter(options.width, 32 + longestEnd * 7.2);
  const foot = axisFoot({
    width: options.width,
    marginLeft: 0,
    marginRight,
    dy: 18,
    low: fiscalYear(firstOf(points).year),
    // The truncated axis, stated on the chart rather than in the caption underneath it. A reader
    // who takes the shape at face value has been misled by the time they reach prose.
    says: `axis starts at ${format(min)}, not zero`,
    high: fiscalYear(last.year),
  });

  return {
    options: {
      width: options.width,
      height: 220,
      marginTop: 14,
      marginBottom: 26 + foot.extraBottom,
      marginLeft: 0,
      marginRight,
      x: { axis: null, domain: [firstOf(points).year, last.year] },
      y: { axis: null, domain: [min, max] },
      marks: [
        ...(projected.length > 1
          ? [
              Plot.areaY(projected, {
                x: "year",
                y1: "low",
                y2: "high",
                fill: banded,
                fillOpacity: 0.16,
                className: "fan-band",
              }),
              Plot.line(projected, {
                x: "year",
                y: "high",
                stroke: banded,
                strokeWidth: 2,
                className: "fan-edge",
              }),
              Plot.line(projected, {
                x: "year",
                y: "low",
                stroke: banded,
                strokeWidth: 2,
                className: "fan-edge",
              }),
              Plot.line(projected, {
                x: "year",
                y: "point",
                stroke: banded,
                strokeWidth: 2,
                ...RULE.projection,
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
                stroke: banded,
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
                stroke: SERIES.formula,
                strokeWidth: 2,
                className: "fan-reference",
              }),
              Plot.text([last], {
                x: last.year,
                // Guarded by `hasReference` above, which the type system cannot see through.
                y: last.reference ?? 0,
                dx: 8,
                text: () => referenceText,
                textAnchor: "start",
                fill: SERIES_TEXT.formula,
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
                stroke: banded,
                strokeWidth: 2,
                className: "fan-anchor",
              }),
            ]
          : []),
        Plot.text(bounds, {
          x: last.year,
          y: (v: number) => v,
          dx: 8,
          text: (v: number, i: number) => boundText(v, i),
          textAnchor: "start",
          // In the band's text hue when there is a second line to tell it from. In primary ink the
          // received line's label read as belonging to neither, beside a formula label in blue.
          fill: hasReference ? SERIES_TEXT.guarantee : INK.primary,
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
      // The topmost line drawn that year, so the tip opens over the plot and not under it.
      anchor: points.map((p) => Math.max(p.high, p.point, ...(hasReference ? [p.reference!] : []))),
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

  const first = firstOf(points);
  const last = lastOf(points);
  // The direct label goes on the last year that has a value, which is not necessarily the last
  // year: a series ending in a gap would otherwise be labelled at a point it does not occupy.
  const endOf = (key: "a" | "b") => [...points].reverse().find((p) => p[key] != null);
  const endA = endOf("a");
  const endB = endOf("b");
  /** What the end labels say, which is what the right gutter has to hold. */
  const endText = (point: SeriesPoint | undefined, key: "a" | "b"): string =>
    point ? `${sentence(key === "a" ? labels.a : labels.b)} ${format(point[key] ?? 0)}` : "";
  const longestEnd = Math.max(endText(endA, "a").length, endText(endB, "b").length);
  /*
   * And the first value of each, at the left (#655). Every dek over these charts is a
   * start-to-end comparison — "fell from 45.9% to 34.5%" — and the end alone left the start to be
   * estimated off a line with no axis. Bare numbers, since the end label already names the line
   * and the hue joins the two.
   */
  const startOf = (key: "a" | "b") => points.find((p) => p[key] != null);
  const startA = startOf("a");
  const startB = startOf("b");
  const startText = (point: SeriesPoint | undefined, key: "a" | "b"): string =>
    point ? format(point[key] ?? 0) : "";
  const widestStart = Math.max(textPx(startText(startA, "a"), 12), textPx(startText(startB, "b"), 12));

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

  const endLabel = (point: SeriesPoint | undefined, key: "a" | "b", ink: string) =>
    point
      ? [
          Plot.text([point], {
            x: point.at,
            y: point[key] ?? 0,
            dx: 8,
            text: () => endText(point, key),
            textAnchor: "start",
            fill: ink,
            className: "series-end",
          }),
        ]
      : [];

  /*
   * Above the upper line's start and below the lower one's, so two starts a few units apart do not
   * print over each other. In the gutter rather than over the plot: an audit render that drew them
   * inside the frame put a falling line through the upper label at both 1280 and 390.
   */
  const startLabel = (point: SeriesPoint | undefined, key: "a" | "b", ink: string) => {
    if (!point) return [];
    const other = key === "a" ? startB?.b : startA?.a;
    const value = point[key] ?? 0;
    const dy = other == null ? 4 : value >= other ? -5 : 7;
    return [
      Plot.text([point], {
        x: point.at,
        y: value,
        dx: -6,
        dy,
        text: () => startText(point, key),
        textAnchor: "end",
        fill: ink,
        className: "series-start",
      }),
    ];
  };

  /*
   * Under everything, because it is what the two lines are read against rather than a third line
   * among them. Muted and dashed for the same reason: the categorical pair is the quantities, and
   * a reference in a third hue would read as a quantity that has no per-position value.
   */
  const ref = options.reference;
  const reference = ref
    ? [
        Plot.ruleY([ref.value], {
          ...RULE.reference,
          className: "series-reference",
        }),
        /*
         * At the right end, under the rule, on a halo (#662). It sat at the left end above the
         * rule, where both lines start — the blue one ran through "What ±1σ claims to hold" at
         * every width on `/method`, and through "No bias" on both projection-bias panels. A halo
         * in the card's own colour is painted first, so a line that still crosses it is cut
         * rather than read through.
         */
        Plot.text([ref.value], {
          x: last.at,
          y: ref.value,
          dx: -2,
          dy: 8,
          text: () => sentence(ref.label),
          textAnchor: "end",
          fill: INK.muted,
          stroke: INK.surface,
          strokeWidth: 3,
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
  /*
   * The start labels' gutter: the string drawn at `BASE`'s 12px, the 6px offset from the line, and
   * 6px clear. Not the 7.2px-a-character the right gutter uses, which is slack on a long name and
   * none on three glyphs — `29%` and `64%` were drawn 2-3px off the left edge at 375 and 600.
   */
  const marginLeft = gutter(options.width, 12 + widestStart);
  const foot = axisFoot({
    width: options.width,
    marginLeft,
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
      marginLeft,
      marginRight,
      x: { axis: null, domain: [first.at, last.at] },
      y: { axis: null, domain: [min, max] },
      marks: [
        ...reference,
        line("a", SERIES.formula, "series-a"),
        line("b", SERIES.guarantee, "series-b"),
        ...endLabel(endA, "a", SERIES_TEXT.formula),
        ...endLabel(endB, "b", SERIES_TEXT.guarantee),
        ...startLabel(startA, "a", SERIES_TEXT.formula),
        ...startLabel(startB, "b", SERIES_TEXT.guarantee),
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
      // The upper of the two lines, as the fan anchors at its topmost; a year with neither has none.
      anchor: points.map((p) => {
        const here = [p.a, p.b].filter((v): v is number => v != null);
        return here.length ? Math.max(...here) : null;
      }),
      cursor: {
        second: "none",
        because:
          "A full-height column over two continuous lines, as the fan chart is. Nothing per-year exists to brighten.",
      },
    },
  };
}
