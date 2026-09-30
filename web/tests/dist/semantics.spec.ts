/**
 * What a document *means*, swept over every page of the build.
 *
 * # Why every assertion here is a sweep and not a visit
 *
 * Each of these treatments is applied in a layout or a shared helper, so it reaches every route
 * including the ones no test visits — and each of them was wrong on a scale no visit would have
 * found. 13,164 two-axis tables carried no `scope`. 32 wiki pages went from `<h1>` straight to
 * `<h3>`. Prose read ragged-left on 1,433 of 3,492 pages. 14,767 scrolling boxes were unfocusable
 * and unnamed. `measure.ts` grades the same ragged-left property over eight routes, and fixing
 * those eight would have satisfied it.
 *
 * That is the argument for reading the artefact rather than driving a browser: a rule that only
 * checks where it was already applied is not a rule, and a browser can only be in one place at a
 * time.
 *
 * The structural baseline underneath them was already strong — across 3,487 pages, zero unlabelled
 * form controls, zero images without `alt`, zero duplicate ids, zero positive `tabindex`. These are
 * the layer above it, and the reason none of them were caught is that a 245-test suite had no
 * automated accessibility assertion in it at all.
 *
 * The browser half of each of these lives on: `chrome.spec.ts` tabs to the skip link and checks
 * what a fragment target lands clear of, `wiki.spec.ts` drives the scrolling boxes with a keyboard,
 * and `charts.spec.ts` opens a chip's panel. What is here is the census.
 */

import { readFileSync } from "node:fs";

import { describe, expect, test } from "vitest";

import { parseHTML } from "linkedom";

import { DIST, pages } from "./artefact.ts";

describe("document semantics", () => {


  test("every header cell in the build says what it heads", () => {
    /*
     * 13,164 two-axis tables across 2,074 pages carried no `scope`, and `/districts` was the only
     * table on the site that did. Without it a screen reader cannot associate a cell with its row
     * and its column, which is most of what makes a data table readable without sight — and this
     * site is substantially data tables.
     *
     * Over the artefact rather than a page at a time, because the treatment is applied in the
     * layout and so reaches routes no test visits.
     */
    let scoped = 0;
    const bare: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      for (const table of page.matchAll(/<table\b[\s\S]*?<\/table>/g)) {
        for (const cell of (table[0] ?? "").matchAll(/<th\b([^>]*)>/g)) {
          if (/\sscope="(col|row|colgroup|rowgroup)"/.test(cell[1] ?? "")) scoped += 1;
          else bare.push(`${file.slice(DIST.length + 1)}: ${cell[0]}`);
        }
      }
    }
    expect(scoped, "the build carries header cells at all").toBeGreaterThan(100_000);
    expect(bare.slice(0, 5), "a header cell that does not say what it heads").toEqual([]);
  });



  test("no page skips a heading level", () => {
    /*
     * 32 wiki node pages went from `<h1>` straight to `<h3>`: a corpus field is written with `##`
     * and `###` and the processor renders it at the depth it was authored at. The corpus is right
     * to author that way — its author is dividing an argument, not choosing a position in a page
     * they cannot see — so the renderer is what places it. See `levelHeadings`.
     */
    const skipped: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      const main = page.slice(page.indexOf("<main"), page.indexOf("</main>"));
      const levels = [...main.matchAll(/<h([1-6])\b/g)].map((m) => Number(m[1]));
      for (let i = 1; i < levels.length; i += 1) {
        if (levels[i]! > levels[i - 1]! + 1) {
          skipped.push(`${file.slice(DIST.length + 1)}: h${levels[i - 1]} to h${levels[i]}`);
          break;
        }
      }
    }
    expect(skipped.slice(0, 5)).toEqual([]);
  });



  /**
   * No sentence in any table on the site is set ragged-left, and no card opens with a `#`.
   *
   * Two sweeps in one walk, because both are about every page and the walk is the expensive part.
   *
   * `measure.ts` now carries `rightAlignedProse: 0`, and that is the deterministic gate — but the
   * report it grades walks eight routes and the defect was on **1,433 of 3,492 pages**. Fixing the
   * eight would have satisfied it. This is the half that reads the build.
   *
   * The anchor half is here rather than in `measure.ts` because it is not a number about layout:
   * `section.ts` puts a muted `#` at the head of every heading and `alignColumns`' neighbour
   * `moveAnchors` moves it to the end, and what has to hold afterwards is that no heading has one
   * in front again — including a heading written at one of the 141 call sites that still put it
   * there, which is exactly what this cannot be allowed to miss.
   */
  test("no sentence reads ragged-left, and no heading opens with its own address", () => {

    const ragged: string[] = [];
    const leading: string[] = [];
    const loose: string[] = [];
    let cells = 0;
    let anchors = 0;

    for (const file of pages()) {
      const where = file.slice(DIST.length + 1);
      const html = readFileSync(file, "utf8");
      const { document } = parseHTML(html);

      for (const cell of document.querySelectorAll("td")) {
        if (cell.closest("table")?.classList.contains("prose")) continue;
        if (cell.classList.contains("says")) continue;
        cells += 1;
        const words = (cell.textContent ?? "").trim().split(/\s+/).filter(Boolean).length;
        if (words > 6 && ragged.length < 5) {
          ragged.push(`${where}: ${(cell.textContent ?? "").trim().slice(0, 60)}`);
        }
      }

      /*
       * "Opens with" is about the rendered text, not about being the first child. The title is a
       * text node, so the anchor is the heading's first *element* either way — reading the
       * heading's own words is the only version of this question that means anything.
       */
      for (const anchor of document.querySelectorAll("h1 a.section-anchor, h2 a.section-anchor, h3 a.section-anchor")) {
        anchors += 1;
        // The heading, not the anchor's parent: #549 wraps the title and the anchor in
        // `.heading-text`, and the anchor in `.heading-tail`, whose own text is just ` #`.
        const heading = anchor.closest("h1, h2, h3")!;
        const text = (heading.textContent ?? "").trim();
        if (text.startsWith("#") && leading.length < 5) leading.push(`${where}: ${text.slice(0, 50)}`);
        if (!anchor.parentElement?.matches(".heading-text > .heading-tail") && loose.length < 5) {
          loose.push(`${where}: ${text.slice(0, 50)}`);
        }
      }
    }

    expect(cells, "the build carries right-aligned cells to check").toBeGreaterThan(1000);
    expect(anchors, "the build carries section anchors to check").toBeGreaterThan(1000);
    expect(ragged, "a sentence set ragged-left in a column of figures").toEqual([]);
    expect(leading, "a card heading still opens with its own address").toEqual([]);
    expect(
      loose,
      "an anchor outside `.heading-tail`, where it can wrap onto a line of its own (#549)",
    ).toEqual([]);
  });



  /**
   * Four things a data table must not do, swept over every table in the build (#575).
   *
   * Each was found on a handful of routes and is produced by a rule that reaches all of them: the
   * head alignment and the classing by `alignColumns`, the bold row and the name by the templates.
   * Only the table is parsed, not the page — there are a great many pages and the question is
   * entirely inside `<table>`.
   */
  test("a figure column's head sits over its figures, and a table marks and names its rows apart", () => {
    let heads = 0;
    let tables = 0;
    const leftHeads: string[] = [];
    const wordFigures: string[] = [];
    const overmarked: string[] = [];
    const repeated: string[] = [];

    for (const file of pages()) {
      const where = file.slice(DIST.length + 1);
      for (const match of readFileSync(file, "utf8").matchAll(/<table\b[\s\S]*?<\/table>/g)) {
        const { document } = parseHTML(`<!doctype html><html><body>${match[0]}</body></html>`);
        const table = document.querySelector("table")!;
        if (table.classList.contains("prose")) continue;
        tables += 1;

        // Body columns by index, counting `colspan` — the same walk `alignColumns` makes.
        const columns = new Map<number, Element[]>();
        for (const row of table.querySelectorAll("tbody tr")) {
          let index = 0;
          for (const cell of row.children) {
            if (cell.tagName === "TD") columns.set(index, [...(columns.get(index) ?? []), cell]);
            index += Number(cell.getAttribute("colspan") ?? 1);
          }
        }
        for (const row of table.querySelectorAll("thead tr")) {
          let index = 0;
          for (const cell of row.children) {
            const span = Number(cell.getAttribute("colspan") ?? 1);
            const cells = span === 1 ? columns.get(index) : undefined;
            index += span;
            if (!cells || cells.every((c) => (c.textContent ?? "").trim() === "")) continue;
            if (cells.some((c) => c.classList.contains("says"))) continue;
            heads += 1;
            if (!cell.classList.contains("heads-figures") && leftHeads.length < 5) {
              leftHeads.push(`${where}: ${(cell.textContent ?? "").trim().slice(0, 40)}`);
            }
          }
        }

        for (const cell of table.querySelectorAll("td.says")) {
          if (cell.querySelector(".fig") && wordFigures.length < 5) {
            wordFigures.push(`${where}: ${(cell.textContent ?? "").trim().slice(0, 40)}`);
          }
        }

        const rows = table.querySelectorAll("tbody tr");
        const current = table.querySelectorAll("tbody tr.current");
        if (rows.length > 1 && current.length * 2 > rows.length && overmarked.length < 5) {
          overmarked.push(`${where}: ${current.length} of ${rows.length} rows`);
        }

        const names = [...table.querySelectorAll('tbody th a[href^="/district/"]')].map((a) =>
          (a.textContent ?? "").replace(/\s+/g, " ").trim(),
        );
        const twice = names.find((name, i) => names.indexOf(name) !== i);
        if (twice && repeated.length < 5) repeated.push(`${where}: ${twice}`);
      }
    }

    expect(tables, "the build carries tables to check").toBeGreaterThan(10_000);
    expect(heads, "the build carries figure columns to check").toBeGreaterThan(10_000);
    expect(leftHeads, "a figure column whose head reads left").toEqual([]);
    expect(wordFigures, "a fig() amount in a column classed as words").toEqual([]);
    expect(overmarked, "a bold row on most of a table's rows, which marks nothing").toEqual([]);
    expect(repeated, "one district name twice in a table, with nothing to tell them apart").toEqual([]);
  });



  test("every one of them in the build says what it is", () => {
    /*
     * Scanned over the artefact rather than a page at a time, for the reason the CSP block above
     * gives: the treatment is applied in the layout, so it reaches every route including the ones
     * no test visits, and a sweep of thirteen pages would not notice a route that slipped it.
     *
     * The six boxes with no table are deliberate and excluded at the source — a chart cannot
     * overflow, so a tab stop on one leads nowhere. See `nameScrollers`.
     */

    let boxes = 0;
    const bare: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      for (const match of page.matchAll(/<div\b([^>]*\bclass="scroll"[^>]*)>/g)) {
        const attrs = match[1] ?? "";
        if (!attrs.includes('tabindex="0"')) continue; // a chart wrapper, excluded at the source
        boxes += 1;
        if (!/aria-label="[^"]+"/.test(attrs)) bare.push(`${file.slice(DIST.length + 1)}: ${attrs}`);
      }
    }
    expect(boxes, "the build carries scrolling tables at all").toBeGreaterThan(10_000);
    expect(bare.slice(0, 5), "a focusable box a screen reader cannot name").toEqual([]);

    /*
     * The second kind of scrolling box, swept the same way and for the same reason.
     *
     * `pre.aligned` holds a property value whose columns carry meaning. The corpus wraps its YAML
     * at column 95, the widest of those needs 803px, and the widest row on a card offers 788 — so
     * these scroll on nearly every viewport, and a box that scrolls and cannot be focused is
     * unreachable content. This is the defect the whole describe block above was written for,
     * arriving in a new element, and the sweep is what stops it arriving in the next one.
     */
    let blocks = 0;
    const nameless: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      for (const match of page.matchAll(/<pre\b([^>]*\bclass="aligned"[^>]*)>/g)) {
        const attrs = match[1] ?? "";
        blocks += 1;
        if (!attrs.includes('tabindex="0"') || !attrs.includes('role="group"'))
          nameless.push(`${file.slice(DIST.length + 1)}: not focusable — ${attrs}`);
        else if (!/aria-label="[^"]+"/.test(attrs))
          nameless.push(`${file.slice(DIST.length + 1)}: ${attrs}`);
      }
    }
    expect(blocks, "the build carries aligned property blocks at all").toBeGreaterThan(30);
    expect(nameless.slice(0, 5), "an aligned block a keyboard cannot reach").toEqual([]);
  });



  test("no chip in the build hides behind a title", () => {
    /*
     * `title` is what this replaced, and it is the shape the defect would grow back in: it looks
     * right to whoever writes it, because a mouse is what they are testing with.
     *
     * Scanned over the artefact rather than a page at a time, since a chip is written by two
     * functions and rendered on 3,487 pages.
     */

    let chips = 0;
    const bad: string[] = [];
    for (const file of pages()) {
      for (const match of readFileSync(file, "utf8").matchAll(
        /<button\b([^>]*\bclass="year-chip[ "][^>]*)>/g,
      )) {
        const attrs = match[1] ?? "";
        chips += 1;
        if (attrs.includes("title=")) bad.push(`${file.slice(DIST.length + 1)}: title on a chip`);
        if (!/aria-label="[^"]+"/.test(attrs)) {
          bad.push(`${file.slice(DIST.length + 1)}: a chip with no name`);
        }
      }
    }
    expect(chips, "the build carries chips at all").toBeGreaterThan(10_000);
    expect(bad.slice(0, 5)).toEqual([]);
  });
});
