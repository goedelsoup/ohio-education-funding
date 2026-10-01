/**
 * No wiki page prints a key where a reader looks for a name (#551).
 *
 * The declaration end is `tests/unit/ontologyLabels.spec.ts`; this is the build end, because a
 * label every declaration carries still reaches no one if a template renders `edge.relationship`
 * instead of its label. The rule is `.yidam/decisions/a-key-is-not-a-label.yml`.
 *
 * The positions are the ones a key used to occupy: the heading cell of a property row, of a stated
 * link, of a class's relationship; and the relationship cell of a pointed-at-by row. A key in prose
 * — an author's `series_years` in a code span — is the author's to write and is not looked at.
 */

import { readFileSync } from "node:fs";
import { join, relative } from "node:path";

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import { loadCorpus } from "../../src/lib/corpus.ts";
import { DIST, pages } from "./artefact.ts";

/** A property key as the ontology spells it. */
const SNAKE = /^[a-z]+(_[a-z]+)+$/;

/** Every relationship slug the corpus writes, which is what a bare slug in a cell would be. */
const SLUGS = (() => {
  const slugs = new Set<string>();
  for (const node of loadCorpus().nodes) {
    for (const edge of [...node.out, ...node.in]) if (edge.relationship) slugs.add(edge.relationship);
  }
  return slugs;
})();

/** The text of every cell in a key position on one page, with the direction marker removed. */
function keyCells(html: string): string[] {
  const { document } = parseHTML(html);
  const cells = [
    ...document.querySelectorAll("#properties th[scope=row], #relationships th[scope=row], #links th[scope=row]"),
    // The pointed-at-by table puts the relationship in the data cell and the node in the heading.
    ...document.querySelectorAll("#links td"),
  ];
  return cells.map((cell) => {
    for (const marker of cell.querySelectorAll("span.n")) marker.remove();
    return (cell.textContent ?? "").replace(/\s+/g, " ").trim();
  });
}

/** The cells on one page that print a key. */
const keysIn = (html: string): string[] =>
  keyCells(html).filter((text) => SNAKE.test(text) || SLUGS.has(text));

/** Node pages and class pages: everything under `/wiki` but the catalog and the decisions. */
const WIKI = pages(join(DIST, "wiki")).filter((file) => !/\/wiki\/(source|decision)\//.test(file));

test("no wiki page prints a property key or a relationship slug in a key position", () => {
  expect(SLUGS.size).toBeGreaterThan(100);
  expect(WIKI.length).toBeGreaterThan(150);
  let cells = 0;
  const printed: string[] = [];
  for (const file of WIKI) {
    const html = readFileSync(file, "utf8");
    cells += keyCells(html).length;
    for (const key of keysIn(html)) printed.push(`${relative(DIST, file)}: ${key}`);
  }
  expect(cells).toBeGreaterThan(1000); // the selectors still find the cells they are about
  expect(printed).toEqual([]);
});

test("the check bites on a real page with its keys put back", () => {
  // H.B. 110's page as it rendered before #551: a property row and a stated link, each by its key.
  const html = readFileSync(join(DIST, "wiki/legislation/hb-110-2021.html"), "utf8");
  expect(keysIn(html)).toEqual([]);
  const doctored = html
    .replace('<th scope="row">When each part took effect</th>', '<th scope="row">effective_note</th>')
    .replace('<th scope="row">Instance of</th>', '<th scope="row">instance-of</th>');
  expect(doctored).not.toBe(html);
  expect(keysIn(doctored)).toEqual(["effective_note", "instance-of"]);
});

test("the check bites on a class page's relationship table and a pointed-at-by cell", () => {
  const classPage = readFileSync(join(DIST, "wiki/metric.html"), "utf8");
  const doctoredClass = classPage.replace('<th scope="row">Derived from <span class="n">out</span>', '<th scope="row">derived-from <span class="n">out</span>');
  expect(doctoredClass).not.toBe(classPage);
  expect(keysIn(doctoredClass)).toEqual(["derived-from"]);

  // A node something points at, with the relationship cell of its first inbound row as a slug.
  const node = loadCorpus().nodes.find((n) => n.in.some((edge) => edge.stated && edge.relationship === "sets"))!;
  const html = readFileSync(join(DIST, `wiki/${node.id}.html`), "utf8");
  const { document } = parseHTML(html);
  const cell = [...document.querySelectorAll("#links td")].find(
    (td) => td.textContent?.trim() === "Sets",
  );
  expect(cell).toBeDefined();
  cell!.textContent = "sets";
  expect(keysIn(document.toString())).toContain("sets");
});

/*
 * #579: the same rule for the names a page is headed and linked by. `SNAKE` above is a property
 * key's shape, and a decision record's slug is not one — `the-four-kinds-of-parameter` is
 * hyphenated — so every decision page was headed by its slug, every row of `/wiki/decision`
 * linked by one, and the check passed them all.
 *
 * A slug inside rendered corpus prose is the author's citation idiom — the corpus cites a node or
 * a record as ``[`bridge-formula`](…)`` throughout — and is left alone as #551 leaves a key in
 * prose alone. Prose renders into `.prose-body` and, for a property's value, into the row's data
 * cell; 61 such citations sit in property cells. What is held is what a template chose to print:
 * the title, the `h1`, a pill, a row heading, a crumb.
 */

/** A hyphenated slug: a node, catalog entry or decision record by its file stem. */
const KEBAB = /^[a-z0-9]+(-[a-z0-9]+)+$/;

/** Every wiki page, the catalog and the decisions included. */
const LIBRARY = pages(join(DIST, "wiki"));

/** The title, the `h1`, and every link outside rendered prose and property values, as text. */
function names(html: string): string[] {
  const { document } = parseHTML(html);
  const headings = [...document.querySelectorAll("title, h1")];
  const links = [...document.querySelectorAll("a")].filter((a) => !a.closest(".prose-body, td, code, pre"));
  return [...headings, ...links].map((element) => (element.textContent ?? "").replace(/\s+/g, " ").trim());
}

const slugsIn = (html: string): string[] => names(html).filter((text) => KEBAB.test(text));

test("no Library page is headed or linked by a slug", () => {
  expect(LIBRARY.length).toBeGreaterThan(250);
  expect(LIBRARY.filter((file) => file.includes("/wiki/decision/")).length).toBeGreaterThan(50);
  const printed: string[] = [];
  for (const file of LIBRARY) {
    for (const slug of slugsIn(readFileSync(file, "utf8"))) printed.push(`${relative(DIST, file)}: ${slug}`);
  }
  expect(printed).toEqual([]);
});

test("the check bites on a decision's heading, the decision index and a front-door pill", () => {
  const record = readFileSync(join(DIST, "wiki/decision/the-four-kinds-of-parameter.html"), "utf8");
  expect(slugsIn(record)).toEqual([]);
  expect(record).toContain("<h1>The four kinds of parameter</h1>");
  const doctoredRecord = record.replace(
    "<h1>The four kinds of parameter</h1>",
    "<h1>the-four-kinds-of-parameter</h1>",
  );
  expect(slugsIn(doctoredRecord)).toEqual(["the-four-kinds-of-parameter"]);

  const index = readFileSync(join(DIST, "wiki/decision.html"), "utf8");
  const doctoredIndex = index.replace(">The three streams of MR-81</a>", ">the-three-streams-of-mr81</a>");
  expect(doctoredIndex).not.toBe(index);
  expect(slugsIn(doctoredIndex)).toEqual(["the-three-streams-of-mr81"]);

  // A citation in prose is exempt, and stays exempt: the exemption is not what is letting the
  // doctored cases through.
  const prose = '<div class="prose-body"><a href="/wiki/decision/kuten"><code>kuten-x</code></a></div>';
  expect(slugsIn(prose)).toEqual([]);
  expect(slugsIn(prose.replace('class="prose-body"', 'class="flags"'))).toEqual(["kuten-x"]);
});

/** The foundational types a class description argues for once it has defined the class. */
const ONTOLOGY_TYPE = /\b(Kind|Phase|Event|Role|Quality|Relator|Situation)\b/;

/** The definition cell of every row in `/wiki`'s class table. */
function blurbs(html: string): string[] {
  const { document } = parseHTML(html);
  return [...document.querySelectorAll("#classes td")].map((td) => (td.textContent ?? "").trim());
}

const unclean = (html: string): string[] =>
  blurbs(html).filter((text) => !/[.!?]$/.test(text) || text.endsWith("…") || ONTOLOGY_TYPE.test(text));

test("every class on /wiki is defined in a whole sentence, without the ontology's vocabulary", () => {
  const html = readFileSync(join(DIST, "wiki.html"), "utf8");
  expect(blurbs(html).length).toBe(loadCorpus().classes.length);
  expect(unclean(html)).toEqual([]);

  // The cell as it rendered before #579.
  const before = "A named statewide method for distributing state aid to education agencies, holding for a bounded span of fiscal periods before being replaced. It is a Phase rather than a…";
  const doctored = html.replace(/(<td>)A named statewide method[^<]*(<\/td>)/, `$1${before}$2`);
  expect(doctored).not.toBe(html);
  expect(unclean(doctored)).toEqual([before]);
});
