/**
 * The sideways-scrolling boxes, made reachable by something other than a mouse.
 *
 * # What was wrong
 *
 * `.scroll` is `overflow-x: auto` and nothing else. Across the built site there are 14,767 of
 * them, almost all holding a data table wider than a phone, and not one carried a `tabindex`, a
 * role or a name. A sighted mouse user drags them sideways; a keyboard user cannot put focus in
 * the box at all, so the columns past the right edge are simply not reachable. Measured against a
 * build, 24 of them genuinely overflow at 375px and none do at 1280px.
 *
 * # Why this is derived and not written at the call sites
 *
 * Seventy `<div class="scroll">` are written across `src/lib/` and `src/pages/`, and the argument
 * `contents.ts` makes about the table of contents applies unchanged: a name written beside the
 * markup is a claim about what the table is, and a claim goes stale where the heading above it
 * moves. Every one of these boxes already sits under a heading that says what the figures in it
 * are, and a sighted reader uses exactly that heading to know what they are looking at. So the
 * name is read off the page after it has rendered, and the two cannot disagree.
 *
 * It also means the treatment cannot be forgotten. A table added in a new card is named because it
 * is on the page, not because whoever added it remembered a convention.
 *
 * # Why `group` and not `region`
 *
 * `region` is a landmark, and a district page holds eleven of these. Eleven landmarks named after
 * eleven cards buries the four that are actually the page's structure. `group` carries the
 * accessible name without entering the landmark list, which is the whole of what is wanted here.
 *
 * # Why they are focusable even where they do not scroll
 *
 * Whether a box overflows depends on the reader's viewport, which a static build does not have.
 * The alternative is to measure in the browser and set `tabindex` from script — and this site's
 * controls work with no script by policy, with a third of the end-to-end suite run that way to
 * hold it. A tab stop on a table that happens to fit is a small cost paid by every keyboard
 * reader; a table that scrolls and cannot be focused is unreachable content. The trade is
 * deliberate and it is the one WCAG 2.1.1 asks for.
 *
 * Build-time only, for the reason `plot/ssr.ts` states about itself: `linkedom` has no business in
 * a browser. Imported by `Base.astro`'s frontmatter, which runs during the build.
 */

import { parseHTML } from "linkedom";

/**
 * A heading's words, without the two annotations sharing its line.
 *
 * The same rule and the same reason as `contents.ts`: the section anchor reads as punctuation and
 * the year chip is a property of the figures rather than part of what they are. Duplicated in
 * spirit rather than shared, because that module's copy is about entries in a list and this one is
 * about the name of a box — and folding them together would make one function that has to know
 * which of the two it is doing.
 */
function words(heading: Element): string {
  const clone = heading.cloneNode(true) as Element;
  for (const annotation of clone.querySelectorAll("a.section-anchor, .year-chip-wrap")) {
    annotation.remove();
  }
  return (clone.textContent ?? "").replace(/\s+/g, " ").trim();
}

/**
 * What to call the box: the heading it sits under.
 *
 * Nearest first. A card's own `h2` is the right answer for the sixty-odd tables inside cards; a
 * preceding `h3` is the right answer for the eight programme tables on a district page, which
 * share one card and are told apart only by their own subheadings. Walking the document backwards
 * finds whichever is nearer without either case having to be enumerated.
 */
function nameOf(box: Element, headings: Element[], order: Map<Element, number>): string {
  const at = order.get(box) ?? -1;
  for (let i = headings.length - 1; i >= 0; i -= 1) {
    const heading = headings[i]!;
    if ((order.get(heading) ?? Infinity) < at) {
      const label = words(heading);
      if (label !== "") return label;
    }
  }
  return "";
}

/**
 * Make every scrolling table focusable and named, and say how many were not nameable.
 *
 * A box with no heading anywhere above it still becomes focusable — being operable is the part
 * that decides whether the content is reachable at all, and it does not depend on having found a
 * name. The count is returned rather than swallowed so a test can hold it at zero.
 *
 * Only boxes holding a table are touched. `.scroll` is also wrapped around four charts, which
 * cannot overflow since a chart is drawn to the width it is shown at — see `plot/ssr.ts` — so
 * making those focusable would add a tab stop to a box that has nothing behind its edge.
 */
export function nameScrollers(body: string): { html: string; unnamed: number } {
  /*
   * A whole document and then an assignment, rather than `parseHTML(\`<body>…</body>\`)`.
   *
   * `contents.ts` parses that way and is right to: it only ever queries, and linkedom answers a
   * query against a fragment perfectly well. This module has to hand the markup back, and a
   * fragment-parsed document has no populated `body` to serialise — `document.body.innerHTML` came
   * out empty, so every page that reached this returned a blank `<main>` and the build said
   * nothing. The shape `plot/ssr.ts` uses is the one that round-trips.
   */
  const { document } = parseHTML("<!doctype html><html><body></body></html>");
  document.body.innerHTML = body;
  const boxes = [...document.querySelectorAll("div.scroll")].filter((box) =>
    box.querySelector("table"),
  );
  if (boxes.length === 0) return { html: body, unnamed: 0 };

  const headings = [...document.querySelectorAll("h1, h2, h3")];
  // Document order as a number, so "the nearest heading above this box" is a comparison rather
  // than a tree walk that has to know how the cards are nested.
  const order = new Map<Element, number>();
  let n = 0;
  for (const node of document.querySelectorAll("*")) order.set(node, (n += 1));

  let unnamed = 0;
  for (const box of boxes) {
    box.setAttribute("tabindex", "0");
    box.setAttribute("role", "group");
    const label = nameOf(box, headings, order);
    if (label === "") unnamed += 1;
    else box.setAttribute("aria-label", label);
  }

  return { html: document.body.innerHTML, unnamed };
}
