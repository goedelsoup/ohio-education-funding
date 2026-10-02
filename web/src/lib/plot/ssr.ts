/**
 * Observable Plot, rendered to SVG at build time.
 *
 * # Why server-side and not in the browser
 *
 * Plot needs a DOM. The usual answer is to ship it and draw on load, which would make it this
 * site's first runtime dependency — roughly 100 KB gzipped of Plot and d3 on every page — and
 * would mean the charts are the one part of a document that arrives empty. Both are avoidable:
 * `linkedom` supplies a DOM here, Plot draws into it during `astro build`, and what reaches the
 * reader is plain SVG inside the HTML. Plot stays a devDependency for every page that uses this.
 *
 * The scenario routes are the deliberate exception. Their charts change when a slider moves, so
 * those pages import Plot for real through `client.ts` — same specifications, different document.
 *
 * Nothing in this module may be imported by client code: `linkedom` is 200 KB and there is no
 * reason for a browser to have a second DOM implementation inside its first one.
 */

import { parseHTML } from "linkedom";

import { draw, type Drawing, type Naming, pair, panelWidth, valuesOf } from "./spec.ts";

/**
 * Anything that looks like a baked-in colour: `#abc`, `#aabbcc`, `rgb(…)`, `hsl(…)`.
 *
 * Not a numeric character reference: the serializer writes a no-break space as `&#160;`, and the
 * log-scale axis names carry one so their parenthetical does not wrap apart (#662).
 */
const LITERAL_COLOUR = /((?<!&)#[0-9a-f]{3,8}\b|\brgba?\(|\bhsla?\()/i;

/**
 * Refuse to emit a chart carrying a literal colour.
 *
 * A hex code in build-time SVG is a chart that ignores the theme, and the mistake is invisible to
 * whoever makes it: they are looking at light mode, where the literal is very nearly right. The
 * failure only shows up for a reader on a dark page, which is not a reader this build has. So it
 * stops the build instead.
 *
 * Plot's own `<style>` block used to be excluded here — it sets type, display and overflow, and
 * the one colour in it is a `--plot-background` custom property this platform never uses. There is
 * no block to exclude any more: `untangle` takes it out before this runs, and its declarations
 * live in `app.css`. The exclusion stays as the guard it always was, so a future Plot that inlines
 * a literal somewhere this does not strip is caught rather than waved through.
 */
function ensureThemeable(svg: string): string {
  const found = svg.replace(/<style>[\s\S]*?<\/style>/g, "").match(LITERAL_COLOUR);
  if (found) {
    throw new Error(
      `A chart was rendered with the literal colour ${found[0]}. Build-time SVG cannot re-render ` +
        `when the reader switches theme, so every colour must be a var(--…) reference into the ` +
        `stylesheet. Use the SERIES and INK tokens in src/lib/plot/tokens.ts.`,
    );
  }
  return svg;
}

/** One SVG, at one width. A misaligned hover layer stops the build — see `Renderer` in `spec.ts`. */
function drawn(build: Drawing, naming: Naming, width: number): string {
  const { document } = parseHTML("<!doctype html><html><body></body></html>");
  const node = draw(build, naming, width, {
    document: document as unknown as Document,
    onMisaligned: (message) => {
      throw new Error(message);
    },
  });
  return node ? ensureThemeable(node.outerHTML) : "";
}

/**
 * Render a chart to one SVG, at a width the caller has chosen.
 *
 * For a **panel of a small multiple**, which is the one drawing on this site whose width is not
 * the page's. Two panels share the wide frame and stack on a phone, so a panel is about 300px in
 * every layout — near enough `WIDTHS.narrow` that the other drawings {@link renderToString}
 * makes would be copies shown to nobody. That is worth saying out loud because the cloud these
 * draw is 609 dots and 609 hit targets: the pair mechanism exists to stop a chart being scaled to
 * illegibility, not to be applied where one layout already serves them all.
 *
 * The SVG is `width: 100%` over a `viewBox`, so a panel drawn at 312 fits a 293px phone column by
 * shrinking, exactly as every other chart here does.
 *
 * Returns the empty string where the builder returns null, on {@link renderToString}'s rule: a
 * form the data cannot support draws nothing rather than an axis with a mark on it.
 *
 * @throws for a width past two panels' share of the wide frame. That is the widest a panel of a
 * real small multiple is drawn at, and anything wider is one chart drawn once — `panelWidth(1)` is
 * 640, and the TTAG node's one-panel spread went through here at that width and painted 5px type
 * on a phone. A chart with no sibling goes through {@link renderToString}.
 */
export function renderPanelToString(build: Drawing, naming: Naming, width: number): string {
  if (width > panelWidth(2)) {
    throw new Error(
      `renderPanelToString was asked for a ${width}px panel, wider than a panel of two. A chart ` +
        `drawn once at that width is scaled to a phone with its type; draw it with renderToString, ` +
        `which makes the narrow drawing as well.`,
    );
  }
  const svg = drawn(build, naming, width);
  return svg
    ? `<div class="chart-pair"><div class="chart-at" data-at="panel">${svg}</div>${valuesOf(svg)}</div>`
    : "";
}

/** A chart at every one of `WIDTHS`, for putting straight into a document. See {@link pair}. */
export function renderToString(build: Drawing, naming: Naming): string {
  return pair((width) => drawn(build, naming, width));
}
