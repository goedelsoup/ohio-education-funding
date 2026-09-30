/**
 * The address of a section, made visible.
 *
 * # Why this exists
 *
 * Every card on the district routes has carried an `id` for some time — `routes.ts` explains at
 * length why the vocabulary is centralised and what the `routes.parameter("state-share-percentage")`
 * 404 taught about writing fragments inline. What none of that gave a reader was any way to *see*
 * an address. The evidence is in the call sites: 21 names in `SECTIONS`, and exactly two links in
 * the whole repository ever used one. The addresses were built and then went unread, because the
 * only way to discover one was to open the page source.
 *
 * So this is the other half. A muted `#` at the head of every section heading, which is a link to
 * that section, which a reader can click, copy, or right-click into a full URL.
 *
 * # When it is visible (#549 revised this)
 *
 * It was always visible, on the argument that a hover-only affordance does not exist on a touch
 * screen and does not exist for a keyboard reader until focus lands on it by accident. Both halves
 * of that still hold and are still honoured — but the cost it waved through turned out not to be
 * nothing: every card heading ending in a muted glyph, which on a phone wrapped onto a line of its
 * own and read as a stray mark.
 *
 * So the reveal is scoped to the one kind of device where hover exists. `app.css` hides it only
 * under `(hover: hover) and (pointer: fine)` — a mouse — and there it appears on hovering the
 * heading and whenever the link has keyboard focus. A touch screen matches neither query and
 * keeps the permanently visible `#`; a keyboard reader tabs onto a link that is in the tab order
 * the whole time and shows the moment it is focused. It is `opacity`, never `display` or
 * `visibility`, because either of those would take it out of the tab order too. None of it needs
 * a script.
 *
 * # Why it is a bare `<a href="#…">` and nothing else
 *
 * A third of the end-to-end suite runs with JavaScript disabled, and this repository has decided
 * once already that a control which dies without script is worse than no control — see
 * `BasisToggle.astro`. Fragment navigation is what a browser does natively. Copying the absolute
 * URL to the clipboard would be an enhancement on top of a link that already works; it is not
 * here, and if it is added it belongs in `site.ts` and must leave the plain click alone.
 *
 * # The id is on the card, not on the heading
 *
 * `.card[id]` is the selector `app.css` already gives `scroll-margin-top`, so a fragment lands the
 * section clear of the sticky chrome. It is also the honest placement: the id names the section,
 * not its title. The consequence is that the id is written twice — once on the card and once in
 * the `anchor()` call inside its heading — and the two could drift. They cannot drift silently:
 * `check-dist-links.ts` asserts that every card's heading anchor points at the card that contains
 * it, in the built page, which is the same trick the `data-part` / `id` agreement test uses.
 *
 * The class is `section-anchor` and not `anchor` because `app.css` already spends `.anchor` on a
 * chart legend swatch — `<i class="sw" data-series="anchor">`, the marker for the last observed year — and a
 * bare `.anchor` rule would style both.
 */

import { escapeHtml } from "./format.ts";

/**
 * The link a section heading wears.
 *
 * The `#` is `aria-hidden` and the accessible name comes from the label, so a screen reader
 * announces "Link to this section" rather than "number sign" — and the heading it sits in keeps a
 * name a reader can recognise.
 *
 * # Where it goes, which is not where this puts it
 *
 * This emits the anchor and the caller prepends it to a title. `semantics.ts`'s `moveAnchors` then
 * relocates it *after* the title and *before* the year chip, as a build-time pass over the rendered
 * HTML — the placement argued for at length in that function, and the one that ships.
 *
 * That pass sees server-rendered markup only. The scenario routes build their headings in the
 * browser from this same helper, so `/scenario` rendered two headings with the anchor last and four
 * with it first, on one screen. {@link heading} is the fix: it states the final order once, in a
 * form both sides can use, and `moveAnchors` is what still carries the markup that has not moved to
 * it.
 *
 * # The trailing space is not cosmetic
 *
 * `</a>` immediately against a letter is the fused-word defect `complete.spec.ts` scans every route
 * for
 * — "computed by<code>", "219of the 606" — and eleven of those shipped before that scan existed.
 * Inside the flex row of a card heading the gap does the separating and a whitespace-only run
 * between two flex items is not rendered at all, so the space costs nothing there; outside one, on
 * a heading the corpus prose grew, it is the separation.
 */
export function anchor(id: string): string {
  const safe = escapeHtml(id);
  return (
    `<a class="section-anchor" href="#${safe}" aria-label="Link to this section">` +
    `<span aria-hidden="true">#</span></a> `
  );
}

/**
 * A whole heading's content, in the order it ships in.
 *
 * # Why this exists rather than a rule about calling `anchor` last
 *
 * Because a rule stated in prose is what produced the defect. Fifteen call sites wrote
 * `${anchor(id)}Title`, `moveAnchors` corrected the server-rendered ones on the way out, and the
 * fourteen the browser renders were never corrected — so `/scenario` showed `Levers #` beside
 * `# Most affected` and `# What moved underneath`.
 *
 * A helper that takes both pieces cannot be called in the wrong order.
 *
 * # The separator, and the two spans
 *
 * A space before the anchor, which `moveAnchors` also inserts and for the reason `anchor` records:
 * `</a>` immediately against a letter is the fused-word defect `complete.spec.ts` scans every route
 * for, and the same defect at the other end is `from<a`.
 *
 * The space sits inside `.heading-tail`, which does not wrap, and the title and the tail sit inside
 * `.heading-text`, which is one flex item in a card heading's row. Without the first the `#` could
 * still break onto its own line at that space; without the second it was a flex item of its own
 * and did so on most long headings at phone width. `moveAnchors` records the measurement.
 *
 * @param id the `id` of the card this heading belongs to
 * @param title the heading's text, already escaped or trusted
 * @param chip a year chip to pin at the far end of the row, or `""`
 */
export function heading(id: string, title: string, chip = ""): string {
  return (
    `<span class="heading-text">${title}` +
    `<span class="heading-tail"> ${anchor(id).trimEnd()}</span></span>${chip}`
  );
}
