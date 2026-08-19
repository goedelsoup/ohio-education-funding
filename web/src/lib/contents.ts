/**
 * What is on this page, read off the page itself.
 *
 * # Why it is derived and not declared
 *
 * The obvious shape is a list of sections written into each template beside the cards. It is also
 * the shape this repository has already been burned by twice, in the same way both times: a
 * section on these routes is *conditional*, and a list written by hand is a claim about what
 * renders rather than a record of it. `#denominators` is absent for 177 of 609 districts because
 * the two valuations agree within 5%; `renderOutcomes` returns an empty string for a feed with no
 * outcome block; a district with no five-year filing has no `#actuals` at all. A declared table of
 * contents is wrong for those pages and wrong silently — the entry is there, it looks like every
 * other entry, and it lands the reader at the top of the document.
 *
 * So this reads the body after it has rendered. Whatever is in the page is in the list, and
 * nothing else can be, which makes the conditional cases correct without anybody enumerating them.
 *
 * # What counts as a section
 *
 * Every `h2`, and the `h3`s the corpus prose grew — and not the `h3`s a card grew. The distinction
 * is not about depth. A corpus node's description *is* the page, and the headings inside it are
 * what its author divided the argument into; the district dashboard's eight `h3`s are the six
 * categoricals plus transportation and preschool, which are the rows of one breakdown and not
 * eight sections of the page. Listing those would produce a contents list longer than the card it
 * summarises, which has stopped being a contents list. Both kinds keep their addresses and their
 * anchors either way — this is about which of them is a way *into* the page.
 *
 * # Why the ids are not checked here
 *
 * Because every entry is a fragment link into the page it sits on, and `check-dist-links.ts`
 * already fails the build on a fragment naming an id its page does not carry. An entry that points
 * at nothing cannot reach a reader, so this module does not restate the guarantee.
 *
 * Build-time only, and for the reason `plot/ssr.ts` states about itself: `linkedom` is 200 KB and
 * has no business in a browser. This is imported by `Base.astro`'s frontmatter, which runs during
 * the build and never ships.
 */

import { parseHTML } from "linkedom";

import { escapeHtml } from "./format.ts";

export interface Entry {
  /** The fragment this entry points at, without the `#`. */
  id: string;
  /** The heading's own words. */
  label: string;
}

/** Below this a list of links is longer than the thing it lists. */
const MINIMUM = 4;

/**
 * The sections of a rendered page body, in the order a reader meets them.
 *
 * An `h2` is a section if anything in its ancestry carries an id — its own, for the headings the
 * corpus prose grew, or the card's, for everything else. The one that resolves to nothing is a
 * heading in a card the ratchet has yet to be run against, and it is skipped rather than guessed
 * at.
 */
export function contentsOf(body: string): Entry[] {
  const { document } = parseHTML(`<body>${body}</body>`);
  const order: string[] = [];
  const headings = new Map<string, string[]>();

  for (const heading of document.querySelectorAll("h2, .prose-body h3[id]")) {
    const owner = heading.hasAttribute("id") ? heading : heading.closest("[id]");
    const id = owner?.getAttribute("id");
    if (!id) continue;

    const label = words(heading);
    if (label === "") continue;

    const found = headings.get(id);
    if (found) found.push(label);
    else {
      headings.set(id, [label]);
      order.push(id);
    }
  }

  return order.map((id) => ({ id, label: shared(headings.get(id)!) }));
}

/**
 * One name for a section that headed itself more than once.
 *
 * A section rendered in both dollar bases has two headings and one address. Three of the four say
 * the same thing twice and the fourth does not: the statewide finances card names its basis in its
 * own heading — "…and hold — nominal" against "…and hold — FY2020 dollars" — because both panels
 * are in the document and a reader with no script has to be able to tell which is which.
 *
 * Neither of those is the name of the section, and picking the first is picking "nominal" for a
 * card that is about both. What they have in common is, so the entry is their common prefix, cut
 * back to a word boundary and stripped of the punctuation that was joining it to the difference.
 * A section whose headings agree gets its heading back unchanged, which is the same rule.
 */
function shared(labels: string[]): string {
  let prefix = labels[0] ?? "";
  for (const label of labels.slice(1)) {
    let i = 0;
    while (i < prefix.length && i < label.length && prefix[i] === label[i]) i += 1;
    prefix = prefix.slice(0, i);
  }
  if (prefix !== labels[0]) {
    // Back to a word boundary: a prefix that stops mid-word is not a name.
    const space = prefix.lastIndexOf(" ");
    if (space > 0) prefix = prefix.slice(0, space);
  }
  return prefix.replace(/[\s—–\-,:;]+$/, "").trim();
}

/**
 * A heading's words, without the two annotations that sit in the same line.
 *
 * The section anchor is a link to the heading and reads as punctuation in a list of headings; the
 * year chip is a property of the card's figures and not part of what the card is about. Both would
 * otherwise arrive in the label — "# Why base cost is $8,120 per pupil FY2027" — and the second is
 * the more misleading of the two, because in a list of eight it reads as a distinction between
 * entries rather than as an annotation on each.
 *
 * Deduplication upstream is what handles the third case: a section rendered once per dollar basis
 * has two headings and one address, and only the first becomes an entry.
 */
function words(heading: Element): string {
  const clone = heading.cloneNode(true) as Element;
  for (const annotation of clone.querySelectorAll("a.section-anchor, .year-chip")) {
    annotation.remove();
  }
  return (clone.textContent ?? "").replace(/\s+/g, " ").trim();
}

/**
 * The list itself.
 *
 * # Why a list and not the pills the rest of the site uses
 *
 * Pills were the obvious choice — "here are the places you might go next" is what the district
 * shortcuts on the front page and a corpus node's backlinks already are — and they were wrong for
 * this at the size it actually reaches. `/wiki/doctrine/equity` has fifteen sections and several
 * of their headings are sentences: as a wrapped row of pills that is six ragged rows and most of a
 * screen before the page starts, and a reader scanning it has to find each entry's beginning
 * somewhere new every time. A list in columns is the same fifteen entries in a third of the height
 * with every entry starting in one of two places.
 *
 * The columns are `column-width` rather than a count, so the number of them follows the width the
 * page has and no breakpoint has to know about it. Reading order down one column and then the next
 * is document order, which is the order the sections are in.
 *
 * # Why it says what it is
 *
 * `aria-label` names it for a screen reader and names it for nobody else. On `/` the list sits
 * under three headline figures, where an unlabelled row of links reads as a filter on them — so
 * the heading is visible, and small enough not to compete with the `h1` above it.
 */
export function renderContents(entries: Entry[]): string {
  const items = entries
    .map((entry) => `<li><a href="#${entry.id}">${escapeHtml(entry.label)}</a></li>`)
    .join("");
  return (
    `<nav class="contents" aria-labelledby="contents-label">` +
    `<p class="contents-label" id="contents-label">On this page</p>` +
    `<ul>${items}</ul></nav>`
  );
}

/**
 * Put the list above the sections it lists.
 *
 * # Why it is inserted rather than placed
 *
 * The layout holds the slot, and the slot opens with the page's `<h1>` and the sentence under it —
 * on a district route, with the sub-navigation as well. A table of contents above those is above
 * the title of the page, which is not where a reader looks for one. The alternative is to wrap the
 * card region of all twenty-odd templates in a component, which puts the same rule in twenty
 * places and leaves each of them able to get it wrong.
 *
 * So the boundary is found rather than marked, and the boundary is well defined: the first section
 * element in the body. Everything before it is the page introducing itself and everything from it
 * on is the page. Nothing is parsed to find it — the string is machine-generated and the opening
 * tag of a section is one of two shapes.
 */
export function withContents(body: string, entries: Entry[]): string {
  if (entries.length < MINIMUM) return body;
  const first = body.search(/<div class="(?:card|basis-scope)[ "]/);
  if (first === -1) return body;
  return body.slice(0, first) + renderContents(entries) + body.slice(first);
}
