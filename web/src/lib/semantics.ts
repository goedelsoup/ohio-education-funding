/**
 * What the markup means, applied to the page after it has rendered.
 *
 * # Why several passes and one parse
 *
 * Three things here are properties of a *document* rather than of the renderer that wrote a piece
 * of it — which table cell heads which row, what a scrolling box should be called, how deep a
 * heading sits — and none of them can be decided where the markup is written. A card renderer does
 * not know what heading it will end up under; a corpus author writing `###` does not know what
 * page their prose will be placed into. The layout is the one place that holds the whole body at
 * once, which is the same argument `contents.ts` makes for reading the table of contents off the
 * rendered page instead of declaring it.
 *
 * They share a parse because `linkedom` over 3,487 page bodies is the expensive part and doing it
 * three times would be three times the cost for no gain.
 *
 * Build-time only, for the reason `plot/ssr.ts` states about itself: `linkedom` has no business in
 * a browser. Imported by `Base.astro`'s frontmatter, which runs during the build.
 */

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
 * (Module docstring above covers where this runs.)
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
function nameScrollers(document: Document, order: Map<Element, number>): number {
  const boxes = [...document.querySelectorAll("div.scroll")].filter((box) =>
    box.querySelector("table"),
  );
  const headings = [...document.querySelectorAll("h1, h2, h3")];

  let unnamed = 0;
  for (const box of boxes) {
    box.setAttribute("tabindex", "0");
    box.setAttribute("role", "group");
    const label = nameOf(box, headings, order);
    if (label === "") unnamed += 1;
    else box.setAttribute("aria-label", label);
  }

  /*
   * The aligned property blocks, which are the site's second kind of scrolling box.
   *
   * `pre.aligned` is `overflow-x: auto` and the corpus wraps its YAML at column 95, so these
   * scroll on nearly every viewport — the widest needs 803px and the widest row offers 788. That
   * is the situation the whole of this function exists for, arriving in a new element: a box a
   * mouse can drag and a keyboard could not put focus in, holding the right-hand half of a
   * piecewise rule.
   *
   * Named from the property name rather than from the heading above it, which is the one place
   * this departs from `nameOf`. Every one of these sits under "Properties", so the heading names
   * eight boxes the same thing; the spanning `<th>` immediately before each one says which
   * property it is, and it is on the page for the same reason — a sighted reader uses exactly
   * that word to know what they are looking at.
   */
  for (const block of document.querySelectorAll("pre.aligned")) {
    block.setAttribute("tabindex", "0");
    block.setAttribute("role", "group");
    const row = block.closest("tr");
    const name = row?.previousElementSibling?.textContent?.trim() ?? "";
    if (name === "") unnamed += 1;
    else block.setAttribute("aria-label", name);
  }
  return unnamed;
}

/**
 * Say which way each header cell heads.
 *
 * # What was wrong
 *
 * 13,164 two-axis tables across 2,074 pages carried no `scope`, and `/districts` was the only
 * table on the site that did. Without it a screen reader cannot associate a cell with its row and
 * its column, which is most of what makes a data table readable without sight — and this site is
 * substantially data tables. A district's programme breakdown is eight columns of figures whose
 * meaning is entirely positional.
 *
 * # Why it is derived
 *
 * Because it is derivable, exactly and everywhere, from where the cell already is. A `th` in the
 * head heads its column; a `th` that opens a row in the body heads that row. Seventy call sites
 * writing `scope="col"` by hand would be seventy chances to write `row`, and a table added later
 * would simply not have it — which is how the one table that got it right came to be alone.
 *
 * # Why "in the body" and not "first in its row"
 *
 * Because it is not always first. The acts table on `/legislation` opens each row with the year as
 * a `td` and makes the act's name the `th` — which is right, the act is what the row is about —
 * and a rule keyed on position left those eighteen cells unscoped while claiming to have covered
 * the table. No row in this build carries more than one `th`, so "a `th` in the body heads its
 * row" is unambiguous everywhere it applies, and it does not depend on where the row's author put
 * it.
 */
function scopeTables(document: Document): number {
  let scoped = 0;
  for (const cell of document.querySelectorAll("table th")) {
    if (cell.hasAttribute("scope")) continue;
    const inHead = cell.closest("thead") != null;
    cell.setAttribute("scope", inHead ? "col" : "row");
    scoped += 1;
  }
  return scoped;
}

/**
 * A cell that reads as a null value in a column of figures.
 *
 * `no change` sits among `+0.57%`, `not published` among dollar amounts, `—` everywhere. They are
 * the *absence* of a figure rather than prose about one, and counting them as text is what turned
 * four otherwise-uniform columns into mixed ones when this rule was measured.
 */
const NOTHING = /^(—|–|-|n\/a|none|no change|not published|not stated|not reported|unknown)$/i;

/**
 * A cell whose whole content is a quantity: what it is, optionally what it is measured in, and
 * optionally a range. `<0.1%` is here because the site writes suppressed small shares that way.
 */
const FIGURE =
  /^[\s(\[]*[<>]?\s*[-+−]?\$?[\d,]+(\.\d+)?\s*(%|pp|x|×|mills?|per pupil|FTE|ADM|of\s+[\d,]+|districts?(\s*\([\d.]+%\))?)?[\s)\]]*((–|—|to|-)\s*[-+−]?\$?[\d,]+(\.\d+)?\s*(%|pp)?)?[\s)\]]*$/i;

const isFigure = (text: string): boolean => text === "" || NOTHING.test(text) || FIGURE.test(text);

/**
 * Move a section's address to the end of its heading, where it stops being the first thing read.
 *
 * # What was wrong
 *
 * `section.ts` puts a muted `#` at the head of every section heading and argues, correctly, that
 * it must be permanently visible rather than revealed on hover — a hover-only affordance does not
 * exist on a touch screen. What it also did was make `#` the first glyph on all **22,334** cards
 * in the build, so every card opened with something that reads as leaked markup.
 *
 * The fix is its position, not its existence. Nothing about the decision to keep it visible
 * changes; it hangs after the title instead of before it.
 *
 * # Why here and not at the 141 call sites
 *
 * The same argument `nameScrollers` makes above, with a larger number. `<SectionAnchor id="…" />`
 * is written 141 times across `src/pages` and `src/lib`, always immediately after the opening
 * `<h2>`, and moving it in each one is 141 chances to leave one behind — with no check that would
 * notice, because a heading with its anchor in the wrong place is still a valid heading.
 *
 * It also could not simply go last. Most of these headings carry a year chip pinned to the far end
 * of the same flex row, and appending the anchor after that would put the address beyond the chip
 * rather than beside the title. So it goes *before the chip*, which is a relationship only the
 * rendered heading knows.
 *
 * # Why the DOM and not `order`
 *
 * CSS `order` would move it visually in one line and leave it first for anything reading the
 * document — a keyboard, a screen reader, a copy-paste. A heading that says one thing to the eye
 * and another to everything else is the shape of defect this module exists to remove.
 */
function moveAnchors(document: Document): number {
  let moved = 0;
  for (const anchor of document.querySelectorAll("a.section-anchor")) {
    const heading = anchor.parentElement;
    if (!heading) continue;

    /*
     * The separator moves with it, and this is not cosmetic.
     *
     * `section.ts` emits a trailing space after `</a>` on purpose, so the anchor never sits
     * against the first letter of the title — it records eleven shipped defects of exactly that
     * shape. Moving the element and leaving the space behind reproduces the defect at the other
     * end: `from<a` is a letter immediately against an inline tag boundary, which is what
     * `app.spec.ts` scans every route for.
     */
    const after = anchor.nextSibling;
    if (after && after.nodeType === 3 && /^\s/.test(after.textContent ?? "")) {
      after.textContent = (after.textContent ?? "").replace(/^\s+/, "");
    }

    const chip = heading.querySelector(".year-chip-wrap");
    if (chip) chip.before(anchor);
    else heading.append(anchor);
    anchor.before(document.createTextNode(" "));
    moved += 1;
  }
  return moved;
}

/**
 * Right-align the columns that hold figures, and leave the rest alone.
 *
 * # What was wrong
 *
 * `app.css` sets `td { text-align: right }` for every table on the site and corrects it only for
 * tables tagged `.prose`. Measured against a build: **21,333 cells carrying more than six words
 * are set right-aligned, on 1,433 of 3,492 pages** — including 31 on the district dashboard, whose
 * build-up tables are headed `STEP / VALUE / WHAT IT IS` and whose third column is a sentence.
 * Ragged-left multi-line prose is hard to read for everyone and hardest for the readers who need
 * the explanation.
 *
 * # Why the column and not the cell, and why not the table
 *
 * Tagging tables one at a time is what `.prose` already is, and it has been applied to none of the
 * fourteen tables on the flagship route in the year it has existed. A per-CELL rule is worse than
 * either: a column of figures with one long footnote in it would have that one cell jump to the
 * left while its neighbours stay right, and alignment is what makes a column a column.
 *
 * So the unit is the column, and the question asked of it is what it holds.
 *
 * # The rule, and why this test rather than the obvious one
 *
 * The obvious test is word count, and it does not work. Measured over every table in the build,
 * classifying columns by "how many of these cells run past six words" leaves **83.6%** of them
 * genuinely mixed — because a column of dollar amounts contains plenty of short cells and a column
 * of sentences contains plenty of terse ones, and the boundary is nowhere.
 *
 * Asking whether each cell *is a figure* instead splits almost perfectly: of 38,029 body columns,
 * **31,101 are entirely figures, 6,923 are entirely not, and 5 are mixed** — one part in ten
 * thousand. Two of those five are `GRF` and an act's name in columns that are 16% and 3% figures,
 * which the majority test lands correctly; the other three were `<0.1%`, now part of the pattern.
 *
 * A middle test also failed and is worth recording so nobody re-derives it: "starts with a digit"
 * scores `609 districts in the FY2027 model` as a figure and puts 3,565 columns back in the mixed
 * band.
 *
 * The threshold is half, and with a split this clean it is not doing any work — it is there so the
 * five have an answer rather than so the boundary can be tuned.
 */
function alignColumns(document: Document): number {
  let marked = 0;

  for (const table of document.querySelectorAll("table")) {
    // `.prose` already left-aligns everything it holds and says why in `app.css`. Leave it.
    if (table.classList.contains("prose")) continue;

    /*
     * Column index has to count `colspan`, not children. `scopeTables` records what happened the
     * last time a rule here was keyed on position without doing that: eighteen cells went unscoped
     * while the code claimed to have covered the table.
     */
    const columns = new Map<number, Element[]>();
    for (const row of table.querySelectorAll("tr")) {
      let index = 0;
      for (const cell of row.children) {
        const span = Number(cell.getAttribute("colspan") ?? 1);
        if (cell.tagName === "TD") {
          if (!columns.has(index)) columns.set(index, []);
          columns.get(index)!.push(cell);
        }
        index += span;
      }
    }

    for (const cells of columns.values()) {
      const filled = cells.filter((cell) => (cell.textContent ?? "").trim() !== "");
      if (filled.length === 0) continue;
      const figures = filled.filter((cell) =>
        isFigure((cell.textContent ?? "").replace(/\s+/g, " ").trim()),
      ).length;
      if (figures * 2 >= filled.length) continue;

      /*
       * A second question, because "not a figure" covers two different things.
       *
       * A column of district names and a column of explanations both read left, and only one of
       * them wants a minimum width. Measured across the build's 6,925 text columns, by the length
       * of their longest cell: 1,054 stay under 20 characters, 56 land between 20 and 39, and
       * 5,815 run past 40. The valley is at 20-39, so that is where the line goes — and it is a
       * real valley rather than a chosen number, which is why 40 and not 50.
       *
       * Without this the district dashboard's `WHAT IT IS` column is squeezed to about fifteen
       * characters a line at 390px, which is a ribbon rather than a sentence.
       */
      const longest = Math.max(
        ...filled.map((cell) => (cell.textContent ?? "").replace(/\s+/g, " ").trim().length),
      );
      for (const cell of cells) {
        cell.classList.add("says");
        if (longest > 40) cell.classList.add("reads");
        marked += 1;
      }
    }
  }

  return marked;
}

/**
 * Put the corpus's headings at the depth of the page they were placed into.
 *
 * # What was wrong
 *
 * 32 wiki node pages went from `<h1>` straight to `<h3>`, because a corpus field is written with
 * `##` and `###` and the markdown processor renders those at the depth they were authored at. The
 * corpus is right to author that way — a node's description is a document, and its author is
 * dividing an argument, not choosing a position in a page they cannot see. The renderer is what
 * should be placing it.
 *
 * # The rule
 *
 * Within each `.prose-body`, the headings the processor wrote are shifted so the shallowest of
 * them sits exactly one level below the nearest heading before the first of them. Relative depth
 * is preserved, which is the point: a `###` under a `##` stays one level under it.
 *
 * "The nearest heading before" is sometimes outside the block and sometimes inside it, and both
 * matter. A node's description follows the page `h1`, so its `###`s become `h2`s. The findings
 * card writes its own `<h2>` *inside* the block before the prose, so the prose under it becomes
 * `h3` — taking the block's own heading as the reference is what makes that come out right, and
 * taking the last heading before the block instead is what made an early version of this push the
 * findings prose to `h5`.
 *
 * # Which headings are the corpus's
 *
 * The ones carrying an `id`. Every heading the markdown processor emits is given one derived from
 * its text — that is what `anchorHeadings` in `prose.ts` hangs the visible address off — and the
 * headings a template writes into a card carry none, because the address is on the card. So the
 * discriminator is not a convention invented here; it is the one the two producers already differ
 * by, and a template heading that grew an id would be a heading with two addresses.
 */
function levelHeadings(document: Document): number {
  const SELECTOR = "h1, h2, h3, h4, h5, h6";
  const depth = (heading: Element) => Number(heading.tagName.slice(1));
  let moved = 0;

  for (const block of document.querySelectorAll(".prose-body")) {
    // Re-read every time. Re-levelling replaces heading elements rather than renaming them — a tag
    // name cannot be changed in place — so a list taken once goes stale the moment the first block
    // is shifted, and every later block is then placed against a detached element still reporting
    // its old depth.
    const all = [...document.querySelectorAll(SELECTOR)];
    const prose = all.filter((heading) => block.contains(heading) && heading.hasAttribute("id"));
    if (prose.length === 0) continue;

    const above = all[all.indexOf(prose[0]!) - 1];
    // With nothing above it the block is the page, and a page's own prose opens at `h2`.
    const target = Math.min(6, (above ? depth(above) : 1) + 1);
    const shift = target - Math.min(...prose.map(depth));
    if (shift === 0) continue;

    for (const heading of prose) {
      const to = Math.min(6, Math.max(2, depth(heading) + shift));
      if (to === depth(heading)) continue;
      const replacement = document.createElement(`h${to}`);
      for (const name of heading.getAttributeNames()) {
        replacement.setAttribute(name, heading.getAttribute(name) ?? "");
      }
      replacement.innerHTML = heading.innerHTML;
      heading.replaceWith(replacement);
      moved += 1;
    }
  }
  return moved;
}

/**
 * Every pass, over one parse of the rendered body.
 *
 * The counts are returned rather than swallowed so the tests can hold them: a scrolling box with
 * no name and a heading that skips a level are both silent failures, visible only to a reader who
 * is not the one writing the markup.
 */
export function applySemantics(body: string): {
  html: string;
  unnamed: number;
  scoped: number;
  relevelled: number;
  /** Cells in a column that holds words rather than quantities, and so reads left. */
  aligned: number;
  /** Section addresses moved out of the first position in their heading. */
  anchored: number;
} {
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

  // Document order as a number, so "the nearest heading above this" is a comparison rather than a
  // tree walk that has to know how the cards are nested. Taken once, before anything moves.
  const order = new Map<Element, number>();
  let n = 0;
  for (const node of document.querySelectorAll("*")) order.set(node, (n += 1));

  const relevelled = levelHeadings(document);
  const unnamed = nameScrollers(document, order);
  const scoped = scopeTables(document);
  const aligned = alignColumns(document);
  const anchored = moveAnchors(document);

  return { html: document.body.innerHTML, unnamed, scoped, relevelled, aligned, anchored };
}
