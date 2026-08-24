/**
 * No page in the built site carries a relative `href`.
 *
 * # Why this runs over `dist/` and not over the route table
 *
 * `tests/unit/links.spec.ts` checks the links this repository *generates*, by asking
 * `resolveTarget` about every target the corpus writes. That is the right check for that layer and
 * it is fast enough to run on every save, but it is blind by construction to any anchor that
 * reaches a page some other way — a hand-written `<a>` in a component, a href built inline in a
 * template, a rewrite that silently stops firing. Forty broken relative hrefs shipped while that
 * suite was green, and the fix that closed the hole in it does not close this one.
 *
 * So this asks the opposite question, of the actual artefact: given the HTML that will be
 * uploaded, is there anything in it that a static host cannot serve? Every route on this site is
 * absolute from the root and `build.format` is `"file"`, so a relative href is never correct here
 * — `<a href="ocg-white-paper-013.md">` on `/wiki/source/x` resolves to
 * `/wiki/source/ocg-white-paper-013.md`, which is a 404 that looks in the markup like a link.
 *
 * Runs after `build` rather than in the unit suite, which runs before it: scanning a stale `dist/`
 * would report on the previous commit's output and is worse than not scanning at all.
 *
 * # And no fragment names an id that is not in the page it points at
 *
 * The second half is newer and closes the same hole one level down. A fragment written inline in a
 * template is enumerated by nothing, exactly like the `routes.parameter("state-share-percentage")`
 * call that shipped a 404 on 609 pages — and a fragment fails *worse* than that call did. A bad
 * path 404s and is visible; a bad fragment resolves, serves the right document, and silently
 * leaves the reader at the top of it having been promised a section four cards down.
 *
 * Several sections on these routes are conditional — `#denominators` is absent for 177 of 609
 * districts because the two valuations agree within 5% — so this is not a check that could be done
 * by reading the templates. It has to ask the built page.
 *
 * # And every section carries the address it advertises
 *
 * Three more, added with the section anchors. Each closes a hole the two checks above have by
 * construction:
 *
 * 1. **Same-page fragments were checked by nothing.** The pattern that collects fragment links
 *    required a leading `/`, so `href="#base-cost"` — which is every anchor a heading now wears —
 *    was invisible to the dangling check. The one shape most likely to be written by hand was the
 *    one shape not being verified.
 *
 * 2. **A duplicate `id` was invisible.** The ids of a page went into a `Set`, so a page carrying
 *    `#not` twice looked exactly like a page carrying it once. That was tolerable while 31 cards
 *    had ids; it is not now that 111 do and their names sit in the same namespace as the form
 *    controls (`#q`, `#levers`, `#theme`). A duplicate does not fail loudly — the browser scrolls
 *    to whichever came first, which is right half the time.
 *
 * 3. **Nothing stopped a new card shipping unaddressed.** Which is how the site got to 80 cards
 *    with no `id`: each one was locally reasonable and nothing counted them. So every card in
 *    `main` must now carry an `id`, that `id` must be a name `routes.ts` lists, and a card that
 *    heads itself must carry the anchor that points back at it — the check that keeps the `id` on
 *    the card and the `href` in its heading from drifting apart, since they are written twice.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { parseHTML } from "linkedom";
import { SECTION_NAMES } from "../src/lib/routes.ts";

const DIST = resolve(process.cwd(), "dist");

/** Absolute from the root, off the site, or a fragment. Anything else resolves against the page. */
const ACCEPTABLE = /^(\/|https?:|mailto:|tel:|#|data:)/;

function* htmlFiles(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* htmlFiles(path);
    else if (entry.name.endsWith(".html")) yield path;
  }
}

/**
 * Every card in `main` carries an address, and every heading advertises the one it is under.
 *
 * Parsed rather than pattern-matched. The two sweeps beside this one are regular expressions over
 * the source because they ask about single attributes, and that is the right tool for it; this one
 * asks which card an `h2` is inside, which is a question about structure and not about text.
 * `linkedom` is already a build dependency — it is the DOM Plot draws its charts into.
 */
function checkSections(here: string, html: string): void {
  const { document } = parseHTML(html);
  const main = document.querySelector("main");
  if (!main) return;

  for (const card of main.querySelectorAll(".card")) {
    cards += 1;
    /*
     * `closest` rather than `getAttribute`, because a card is not always the thing that carries
     * its own address. The two panels of a basis switch are one card rendered twice — see
     * `BasisToggle.astro` — so the address sits on the scope that holds both, and each panel is
     * addressed by it. Anything else with an id in the ancestry would do as well; nothing else has
     * one that a card sits under.
     */
    const owner = card.closest("[id]");
    const id = owner?.getAttribute("id");
    if (!id) {
      unaddressed.push(`  ${here}\n    a .card with no id: ${summarise(card)}`);
      continue;
    }
    if (!SECTION_NAMES.has(id)) {
      unaddressed.push(
        `  ${here}\n    .card#${id} names a section routes.ts does not list: ${summarise(card)}`,
      );
    }
    // A card that heads itself has to carry the link back to itself. The id is written twice —
    // once here and once in the `anchor()` call inside the heading — and this is what stops the
    // two drifting.
    // A heading carrying its own id is a section in its own right — a prose card's body starts
    // with one — and is checked against itself by the loop below rather than against its card.
    const heading = [...card.children].find(
      (child) => child.tagName === "H2" && !child.hasAttribute("id"),
    );
    if (heading) requireAnchor(here, id, heading, `.card#${id}`);
  }

  /*
   * A heading that owns its id directly: the district sub-sections, and every heading the corpus
   * prose grew — those come out of the markdown processor already carrying a slug of their own
   * text, which is why `h2` is in the list here and not only in the card loop above.
   */
  for (const heading of main.querySelectorAll("h2[id], h3[id], h4[id], h5[id], h6[id]")) {
    const id = heading.getAttribute("id")!;
    requireAnchor(here, id, heading, `${heading.tagName.toLowerCase()}#${id}`);
  }
}

/** The heading names its own section, and no other. */
function requireAnchor(here: string, id: string, heading: Element, what: string): void {
  const anchor = heading.querySelector("a.section-anchor");
  if (!anchor) {
    unaddressed.push(`  ${here}\n    ${what} heads itself with no section anchor`);
    return;
  }
  const href = anchor.getAttribute("href");
  if (href !== `#${id}`) {
    unaddressed.push(`  ${here}\n    ${what} carries an anchor pointing at ${href} instead`);
  }
}

/** Enough of a card to find it in the template that emitted it. */
function summarise(card: Element): string {
  const heading = card.querySelector("h2, h3, p");
  const text = (heading?.textContent ?? "").replace(/\s+/g, " ").trim();
  return text.length > 60 ? `${text.slice(0, 60)}…` : text || "(no text)";
}

const offenders: string[] = [];
let pages = 0;
let anchors = 0;

/** Every `id` in a built page, by the site path that serves it. Filled on the first pass. */
const identifiers = new Map<string, Set<string>>();
/** Every internal fragment link: where it was written, and what it points at. */
const fragments: { from: string; path: string; id: string }[] = [];
/** An `id` a page carries more than once. */
const duplicates: { page: string; id: string }[] = [];
/** A card in `main` with no `id`, an `id` outside the vocabulary, or a heading that misaddresses it. */
const unaddressed: string[] = [];
let cards = 0;
/** A word, then an inline element opening against it with no space between the two. */
const fused: string[] = [];
/** Every page's `<title>`, so two pages cannot claim to be the same thing. */
const titles = new Map<string, string[]>();

/** `dist/district/043786.html` is served as `/district/043786`; `build.format` is "file". */
const servedAs = (file: string): string => "/" + relative(DIST, file).replace(/\.html$/, "");

for (const file of htmlFiles(DIST)) {
  pages += 1;
  const html = readFileSync(file, "utf8");
  const here = servedAs(file);

  const ids = new Set<string>();
  for (const match of html.matchAll(/\sid="([^"]*)"/g)) {
    const id = match[1]!;
    // Reported once per page per id, however many times it repeats beyond the first.
    if (ids.has(id)) {
      if (!duplicates.some((d) => d.page === here && d.id === id)) duplicates.push({ page: here, id });
    }
    ids.add(id);
  }
  identifiers.set(here, ids);

  checkSections(here, html);

  for (const match of html.matchAll(/\s(?:href|src)="([^"]*)"/g)) {
    const value = match[1]!;
    anchors += 1;
    if (value === "" || ACCEPTABLE.test(value)) continue;
    offenders.push(`  ${relative(DIST, file)}\n    ${value}`);
  }

  for (const match of html.matchAll(/\shref="(\/[^"]*)#([^"]+)"/g)) {
    fragments.push({ from: here, path: match[1]!, id: match[2]! });
  }

  /*
   * A fragment with no path is a link into the page it is written on, which is what every section
   * anchor is. It was outside the pattern above — that one requires a leading `/` — so until this
   * line the most hand-written shape on the site was the one shape nothing checked.
   */
  for (const match of html.matchAll(/\shref="#([^"]+)"/g)) {
    fragments.push({ from: here, path: here, id: match[1]! });
  }

  /*
   * The `<title>`, which is also the `og:title` and the search result and the browser tab.
   *
   * Ohio names districts after townships and townships repeat: three Green Local, three Buckeye
   * Local, three Southern Local. 48 districts shared a name, every district route identified a
   * district by bare name, and 242 pages therefore carried a title belonging to two or three
   * different places. The corpus supplied one more on its own — a `scenario` class against the
   * scenario runner.
   *
   * A title is the one string a reader sees before they have the page, so this asks the artefact
   * whether any two pages are indistinguishable at that distance.
   */
  const title = /<title>([\s\S]*?)<\/title>/.exec(html)?.[1]?.trim();
  if (title) {
    const pages = titles.get(title);
    if (pages) pages.push(here);
    else titles.set(title, [here]);
  }

  /*
   * A word, then an inline element opening with no space between them.
   *
   * Astro drops the newline between a text node that ends a line and a tag that opens the next, so
   * `half of` on one line and `<a>preschool` on the one below render as `half ofpreschool`. The
   * idiom that prevents it is `{" "}` closing the text line, and it is used correctly within a few
   * lines of every place it was forgotten — this is a defect of omission, not of understanding.
   *
   * Two guards for this already exist and both are blind to that shape by construction.
   * `tests/unit/fusion.spec.ts` walks `{…}` expressions, and a bare `<a>` after prose is not one.
   * The sweep in `tests/e2e/app.spec.ts` iterates ROUTES_WITH_FIGURES, a list assembled for the
   * year-chip rule, which contains none of the three pages that were actually fusing — though the
   * suite loads /scenario nineteen times for other reasons. Asking the artefact closes both holes
   * at once, because it cannot miss a page: it reads every page there is.
   *
   * The year chip is the one deliberate adjacency — `<h2>Two floors<span class="year-chip">` is
   * how a chip attaches to its heading — and it is 8,806 of the 8,814 matches here. Excluded by
   * class, because excluding it by tolerating a count is how the eight got in.
   */
  for (const match of html.matchAll(
    /([A-Za-z][A-Za-z.,;:)]?)<(a|abbr|code|em|span|strong)\b([^>]*)>[A-Za-z]/g,
  )) {
    if (/year-chip/.test(match[3]!)) continue;
    const at = match.index!;
    const context = html.slice(Math.max(0, at - 45), at + match[0].length + 25).replace(/\s+/g, " ");
    fused.push(`  ${here}\n    …${context}…`);
  }

  /*
   * The same trimming, where the thing fused is a separator rather than a word.
   *
   * The rule above needs a letter on the outside of the tag, and a letter is what it looked for
   * because that is what the eight live cases had. This is the other outcome of the identical
   * mistake: a line that *begins* with `·`, below a line ending in an element or an expression.
   *
   *     …{district.county} County</a>
   *     · Senate {senateSeats.map(…)}      ->   "Cuyahoga County· Senate 18"
   *
   * It was live on every district dashboard, three times each — 1,827 in the build, more than
   * every other fusion this file has ever found put together, and invisible to both the rule
   * above and to `tests/unit/fusion.spec.ts`, whose scan is written around `{…}` expressions.
   *
   * Only the middot, not punctuation generally. A comma or a full stop closes the clause the tag
   * ended and belongs tight against it — `<strong>219</strong>,` is correct and common. A
   * separator does the opposite: it stands *between* two items and this site spaces it on both
   * sides everywhere it writes one, so `</a>·` is never what anybody meant. An em dash is spaced
   * here too, but it is also written closed-up as a range, and a rule with exceptions is one
   * somebody eventually suppresses.
   */
  for (const match of html.matchAll(
    /(<\/(?:a|abbr|code|em|span|strong)>·|·<(?:a|abbr|code|em|span|strong)\b)/g,
  )) {
    const at = match.index!;
    const context = html.slice(Math.max(0, at - 45), at + match[0].length + 25).replace(/\s+/g, " ");
    fused.push(`  ${here}\n    …${context}…`);
  }
}

if (offenders.length > 0) {
  console.error(
    `\n${offenders.length} relative link${offenders.length === 1 ? "" : "s"} in the built site.\n\n` +
      `Each one resolves against the page it is on rather than against the site root, which on a\n` +
      `static host is a 404 that looks exactly like a working link in the markup.\n\n` +
      offenders.join("\n") +
      `\n\nIf a corpus target is behind this, the shapes resolveTarget places are listed above it\n` +
      `in src/lib/corpus.ts. If a template is, the routes belong in src/lib/routes.ts.\n`,
  );
  process.exit(1);
}

const dangling = fragments.filter(({ path, id }) => {
  const ids = identifiers.get(path);
  // A path this site does not build is the other check's business, not this one's.
  return ids !== undefined && !ids.has(id);
});

if (dangling.length > 0) {
  console.error(
    `\n${dangling.length} fragment link${dangling.length === 1 ? "" : "s"} in the built site name\n` +
      `an id the target page does not carry.\n\n` +
      `Unlike a bad path this does not 404. The document is served, the browser finds nothing to\n` +
      `scroll to, and the reader lands at the top of the right page with no sign that the section\n` +
      `they were sent to is missing — so nothing but this reports it.\n\n` +
      `Sections on the district routes are conditional: #denominators is absent wherever the two\n` +
      `valuations agree within 5%. A link to one has to be gated on the same predicate its\n` +
      `renderer uses.\n\n` +
      dangling
        .slice(0, 20)
        .map(({ from, path, id }) => `  ${from}\n    → ${path}#${id}`)
        .join("\n") +
      `\n`,
  );
  process.exit(1);
}

if (duplicates.length > 0) {
  console.error(
    `\n${duplicates.length} id${duplicates.length === 1 ? " is" : "s are"} carried more than once by\n` +
      `the page that serves ${duplicates.length === 1 ? "it" : "them"}.\n\n` +
      `An id is an address, and two elements at one address is not a thing a browser reports. It\n` +
      `scrolls to whichever came first in the document, which is the right one about half the time,\n` +
      `and every check above this one is satisfied either way — the ids of a page were collected\n` +
      `into a Set, so a duplicate looked exactly like the single occurrence it was hiding behind.\n\n` +
      duplicates
        .slice(0, 20)
        .map(({ page, id }) => `  ${page}\n    #${id}`)
        .join("\n") +
      `\n`,
  );
  process.exit(1);
}

if (unaddressed.length > 0) {
  console.error(
    `\n${unaddressed.length} card${unaddressed.length === 1 ? "" : "s"} in the built site ` +
      `${unaddressed.length === 1 ? "does" : "do"} not carry the address ${unaddressed.length === 1 ? "it advertises" : "they advertise"}.\n\n` +
      `A card with no id is a section a reader cannot link to and a search result cannot land on.\n` +
      `Nothing used to count them, which is how the site reached 80 of them: each was locally\n` +
      `reasonable and the total was nobody's business.\n\n` +
      `The fixes, by what the line below says:\n` +
      `  no id                    give the card id="…" and the matching data-part, and add the\n` +
      `                           anchor to its heading — src/lib/section.ts\n` +
      `  not listed in routes.ts  add the name to SECTIONS under its route family, or use the\n` +
      `                           name already there\n` +
      `  no section anchor        \${anchor("…")} in a lib renderer, <SectionAnchor id="…" /> in a\n` +
      `                           template\n` +
      `  points at … instead      the id on the card and the id in its heading have drifted\n\n` +
      unaddressed.slice(0, 20).join("\n") +
      (unaddressed.length > 20 ? `\n  … and ${unaddressed.length - 20} more` : "") +
      `\n`,
  );
  process.exit(1);
}

const duplicated = [...titles].filter(([, pages]) => pages.length > 1);
if (duplicated.length > 0) {
  const pages = duplicated.reduce((n, [, list]) => n + list.length, 0);
  console.error(
    `\n${duplicated.length} title${duplicated.length === 1 ? "" : "s"} used by more than one page, ` +
      `across ${pages} pages.\n\n` +
      `A title is what a reader sees in a tab, a search result and a shared link — before they\n` +
      `have the page. Two pages with one title are indistinguishable at exactly the moment the\n` +
      `reader is choosing between them.\n\n` +
      `For districts, \`qualifiedName\` in src/lib/feed.ts adds the county to a name that repeats.\n\n` +
      duplicated
        .slice(0, 15)
        .map(([title, list]) => `  ${title}\n    ${list.join("\n    ")}`)
        .join("\n") +
      (duplicated.length > 15 ? `\n  … and ${duplicated.length - 15} more` : "") +
      `\n`,
  );
  process.exit(1);
}

if (fused.length > 0) {
  console.error(
    `\n${fused.length} fused word${fused.length === 1 ? "" : "s"} in the built site.\n\n` +
      `A text node ends and an inline element opens against it with no space, so the two run\n` +
      `together on the page: "the weighted half of" followed by <a>preschool…</a> is read by every\n` +
      `reader as "half ofpreschool", and a "·" opening a line below an element renders as\n` +
      `"Cuyahoga County· Senate 18".\n\n` +
      `The fix is \`{" "}\` at the end of the line the text sits on, which is what the prose on\n` +
      `either side of each of these already does.\n\n` +
      fused.slice(0, 20).join("\n") +
      (fused.length > 20 ? `\n  … and ${fused.length - 20} more` : "") +
      `\n`,
  );
  process.exit(1);
}

console.log(
  `${anchors} links across ${pages} built pages, all absolute or off-site; ` +
    `${fragments.length} fragment links all resolve to an id in the page they name; ` +
    `${cards} cards each addressed by a name routes.ts lists, with no id used twice in a page; ` +
    `nothing fused against an inline element, word or separator; ` +
    `${titles.size} distinct titles across ${pages} pages, none used twice`,
);
