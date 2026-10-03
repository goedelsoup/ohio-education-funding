/**
 * What a document *means*, swept over every page of the build.
 *
 * # Why every assertion here is a sweep and not a visit
 *
 * Each of these treatments is applied in a layout or a shared helper, so it reaches every route
 * including the ones no test visits — and each of them was wrong on a scale no visit would have
 * found. 13,164 two-axis tables carried no `scope`. 32 wiki pages went from `<h1>` straight to
 * `<h3>`. Prose read ragged-left on 1,433 of 3,492 pages. 14,767 scrolling boxes were unfocusable
 * and unnamed. `measure.ts` grades the same ragged-left property over nine routes, and fixing
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

import { DIST, drawingsAsText, pages } from "./artefact.ts";

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
   * report it grades walks nine routes and the defect was on **1,433 of 3,492 pages**. Fixing the
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
      // Drawings reduced to their text (#646): no `td` or heading is inside one, and a cell that
      // holds one counts the same words either way.
      const { document } = parseHTML(drawingsAsText(readFileSync(file, "utf8")));

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

  test("a chart's name is one sentence, and its words start a sentence", () => {
    /*
     * #612 counted 504 axis-foot texts and 18 series end labels starting lower-case beside the
     * capitalised legends, and a name that ran to 577 characters because the method was appended
     * to it. The method is a `<desc>` now, and `axisFoot`, `yTitle` and the series end labels
     * capitalise what they are handed — so the census is over what they drew, not who called them.
     *
     * 250 is a name a screen reader can be skipped past; the longest left is about 230. A ".." is
     * a sentence joined to one that already ended.
     */
    const length = (attr: string) => attr.replace(/&(#\d+|#x[0-9a-f]+|\w+);/gi, "x").length;
    let named = 0;
    const bad: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      const at = file.slice(DIST.length + 1);
      for (const svg of page.matchAll(/<svg\b[^>]*?\baria-label="([^"]*)"/g)) {
        const name = svg[1] ?? "";
        named += 1;
        if (length(name) > 250) bad.push(`${at}: a ${length(name)}-character name`);
        if (name.includes("..")) bad.push(`${at}: "${name}"`);
      }
      for (const group of page.matchAll(
        /<g\b[^>]*\bclass="(axis-foot|axis-head|y-title|series-end|series-reference|series-tick)"[^>]*>([\s\S]*?)<\/g>/g,
      )) {
        for (const text of (group[2] ?? "").matchAll(/<text\b[^>]*>(?:<tspan\b[^>]*>)?\s*([^<]*)/g)) {
          if (/^\p{Ll}/u.test(text[1] ?? "")) bad.push(`${at}: ${group[1]} "${text[1]}"`);
        }
      }
    }
    expect(named, "the build names its charts at all").toBeGreaterThan(1_000);
    expect(bad.slice(0, 5)).toEqual([]);
  });
});

describe("a chart's values and its address (#616)", () => {
  /** Each `.chart-pair` in a page, as the markup from its opening tag to the next one's. */
  const pairsOf = (page: string): string[] => page.split(/<div\b[^>]*\bclass="chart-pair"[^>]*>/).slice(1);
  /** The drawing a reader is shown at the widest layout, or the one drawing of a panel. */
  const shown = (pair: string): string =>
    /data-at="(?:wide|panel)"><svg\b[\s\S]*?<\/svg>/.exec(pair)?.[0] ?? "";
  const hovers = (svg: string): number => (svg.match(/\bdata-hover="/g) ?? []).length;

  test("a chart with more than eight marks to hover has its values as text beside it", () => {
    /*
     * A screen reader in browse mode does not drive the arrow-key cursor, so `role="img"` left it
     * the chart's name and nothing else: 4,456 charts with more than eight marks and no table or
     * list of their values. Eight is the issue's line — under it, a chart's values are its labels.
     *
     * And the list says what the marks say, count for count: it is built from their hovers, and a
     * list one short of the marks would be the alternative quietly dropping a district.
     */
    let checked = 0;
    const bare: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      if (!page.includes('class="chart-pair"')) continue;
      for (const pair of pairsOf(page)) {
        const svg = shown(pair);
        const marks = hovers(svg);
        if (marks <= 8 || /^<svg\b[^>]*\baria-hidden="true"/.test(svg.slice(svg.indexOf("<svg")))) continue;
        checked += 1;
        const list = /<details class="chart-values">[\s\S]*?<\/details>/.exec(pair)?.[0];
        const items = (list?.match(/<li>/g) ?? []).length;
        if (items !== marks) {
          bare.push(`${file.slice(DIST.length + 1)}: ${marks} marks, ${items} values listed`);
        }
      }
    }
    expect(checked, "the build carries charts with many marks").toBeGreaterThan(4_000);
    expect(bare.slice(0, 5), "a chart whose values only a pointer can reach").toEqual([]);
  });

  test("no chart hidden from assistive technology carries a hover", () => {
    /*
     * The county-position strip was `aria-hidden` and kept 84 `data-hover` dots: values a mouse
     * and a finger could reach and a screen reader could not. A chart hidden because the text
     * beside it says what it says has nothing to hover.
     */
    const hidden: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      for (const svg of page.matchAll(/<svg\b[^>]*\baria-hidden="true"[^>]*>[\s\S]*?<\/svg>/g)) {
        if (svg[0].includes("data-hover=")) hidden.push(file.slice(DIST.length + 1));
      }
    }
    expect(hidden.slice(0, 5)).toEqual([]);
  });

  test("every chart can be linked to on its own", () => {
    /*
     * A chart's address was its section's, so `/method`'s three forecast charts were all
     * `#forecast-range` and 21 wiki charts were all `#findings`. The rule here is stated over what
     * a link does rather than how the markup is shaped: some link on the page points at an element
     * that holds this chart and no other — or at a heading with no other chart under it before the
     * next. The panels of one small multiple are one chart, and so are the two drawings of a basis
     * toggle: `/finances` draws its cash chart once in nominal and once in constant dollars, shows
     * one, and `#actuals` lands on the pair.
     */
    let charts = 0;
    const unaddressed: string[] = [];
    for (const file of pages()) {
      const html = readFileSync(file, "utf8");
      if (!html.includes('class="chart-pair"')) continue;
      const { document } = parseHTML(html);
      const linked = new Set(
        [...document.querySelectorAll('a[href^="#"]')].map((a) => a.getAttribute("href")!.slice(1)),
      );
      const ownFigure = (pair: Element): Element => pair.closest(".panels") ?? pair;
      const figuresIn = (box: Element): Element[] => [
        ...new Set([...box.querySelectorAll(".chart-pair")].map(ownFigure)),
      ];
      // A drawing in a later basis panel is the same figure as its counterpart in the first.
      const figureOf = (pair: Element): Element => {
        const own = ownFigure(pair);
        const panel = own.closest(".basis-panel");
        const first = panel?.closest(".basis-scope")?.querySelector(".basis-panel");
        if (!panel || !first || first === panel) return own;
        return figuresIn(first)[figuresIn(panel).indexOf(own)] ?? own;
      };

      // Each heading and the figures under it, in document order.
      const under = new Map<Element, Set<Element>>();
      let heading: Element | null = null;
      for (const el of document.querySelectorAll("h1, h2, h3, h4, h5, h6, .chart-pair")) {
        if (!el.classList.contains("chart-pair")) heading = el;
        else if (heading && !el.closest("template")) {
          under.set(heading, (under.get(heading) ?? new Set()).add(figureOf(el)));
        }
      }

      const figures = new Set(
        [...document.querySelectorAll(".chart-pair")].filter((p) => !p.closest("template")).map(figureOf),
      );
      for (const figure of figures) {
        charts += 1;
        let ok = false;
        for (let box = figure as Element | null; box && !ok; box = box.parentElement) {
          if (!box.id || !linked.has(box.id)) continue;
          const held = new Set([...box.querySelectorAll(".chart-pair")].map(figureOf));
          if (held.size === 1) ok = true;
        }
        for (const [h, set] of under) {
          if (ok) break;
          if (h.id && linked.has(h.id) && set.size === 1 && set.has(figure)) ok = true;
        }
        if (!ok) {
          const name = figure.querySelector("svg")?.getAttribute("aria-label")?.slice(0, 60);
          unaddressed.push(`${file.slice(DIST.length + 1)}: ${name}`);
        }
      }
    }
    expect(charts, "the build carries charts at all").toBeGreaterThan(5_000);
    expect(unaddressed.slice(0, 10), "a chart with no address of its own").toEqual([]);
  });
});
