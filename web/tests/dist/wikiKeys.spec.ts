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
