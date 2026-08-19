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
 * # Why it is always visible
 *
 * The convention elsewhere is to reveal the anchor on hover. That repeats exactly the criticism
 * the `.year-chip` rule in `app.css` makes of `title`: it is hover-only, so it does not exist on a
 * touch screen and it does not exist for a reader who navigates by keyboard until focus lands on
 * it by accident. This stylesheet already accepted a permanently visible muted annotation on every
 * heading — the year chip — and a second one costs the design nothing it has not already paid.
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
 * chart legend swatch — `<i class="sw anchor">`, the marker for the last observed year — and a
 * bare `.anchor` rule would style both.
 */

import { escapeHtml } from "./format.ts";

/**
 * The link a section heading wears.
 *
 * The `#` is `aria-hidden` and the accessible name comes from the label, so a screen reader
 * announces "Link to this section" rather than "number sign" — and the heading it sits in keeps a
 * name a reader can recognise. It is first in the heading rather than last so that it never lands
 * between a title and the year chip pinned to the other end of the same flex row.
 *
 * # The trailing space is not cosmetic
 *
 * `</a>` immediately against a letter is the fused-word defect `app.spec.ts` scans every route for
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
