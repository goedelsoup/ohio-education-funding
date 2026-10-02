/**
 * Every figure cell in the build, written the way `format.ts` writes a number.
 *
 * # Why this reads the artefact
 *
 * The HTML is built in template literals, so a number reaches the page through whatever the line
 * that interpolated it happened to call. `format.ts` normalises the minus to U+2212 and groups
 * thousands; `toFixed` does neither, and there were ninety calls to it in the renderers. Before
 * #503 the build carried `-0.03` in the growth rows of 326 outcome pages and `-0.662` on
 * /statewide, beside cells that read `−$1,318`, and Columbus's Category 2 pupils as `8376.39`.
 *
 * `astro check` cannot see inside a string, and a source rule can only name the spellings it
 * already knows about. The cells are the thing a reader sees, so the cells are what this holds.
 */

import { readFileSync } from "node:fs";
import { relative } from "node:path";

import { parseHTML } from "linkedom";
import { describe, expect, test } from "vitest";

import { DIST, drawingsAsText, pages } from "./artefact.ts";

/** The text a `.tnum` element opens with — up to its first child tag, which is enough for a figure. */
const TNUM = /<(?:td|th|span|div)\b[^>]*\bclass="[^"]*\btnum\b[^"]*"[^>]*>([^<]*)/g;

/** A cell that cites the Revised Code carries section numbers, which are not quantities. */
const CITATION = /R\.C\.|§/;

describe("figure cells", () => {
  const cells: { page: string; text: string }[] = [];
  for (const file of pages()) {
    const page = readFileSync(file, "utf8");
    for (const match of page.matchAll(TNUM)) {
      const text = (match[1] ?? "").trim();
      if (text) cells.push({ page: file.slice(DIST.length + 1), text });
    }
  }

  test("the sweep reads the figure cells at all", () => {
    /*
     * Without this the two rules below pass on a build with no `.tnum` in it, or on a pattern that
     * stopped matching the markup. 252,986 cells carried text when this was written.
     */
    expect(cells.length).toBeGreaterThan(200_000);
    expect(cells.filter((c) => c.text.includes("−")).length, "some cell is negative").toBeGreaterThan(1_000);
  });

  test("no figure cell writes its minus as a hyphen", () => {
    const hyphens = cells.filter((c) => /(^|[\s($])-\d/.test(c.text));
    expect(
      hyphens.slice(0, 5).map((c) => `${c.page}: ${c.text}`),
      "format the value through `fixed`, `signed` or `money` in src/lib/format.ts",
    ).toEqual([]);
  });

  test("no figure cell leaves a quantity over a thousand ungrouped", () => {
    /*
     * A decimal quantity only: a four-digit run with no point is a year, and a six-digit one is an
     * IRN or a line item's number, and neither takes a separator.
     */
    const ungrouped = cells.filter((c) => !CITATION.test(c.text) && /(?<![\d,.])\d{4,}\.\d/.test(c.text));
    expect(
      ungrouped.slice(0, 5).map((c) => `${c.page}: ${c.text}`),
      "format the value through `fixed` or `count` in src/lib/format.ts",
    ).toEqual([]);
  });
});

/**
 * Every figure in the visible text, not only the ones in a `.tnum` cell.
 *
 * # What the cell sweep missed
 *
 * A figure in a sentence is not in a cell, and #574 found three kinds of them. Correlations went
 * out through `toFixed(3)` into prose on `/`, `/outcomes` and the Outcome tab, so "-0.846" sat on
 * the page beside "−0.846" from `format.ts`. `/data`'s checkpoint table wrote "7.281B" with no
 * dollar sign where `/statewide` wrote "$7.28B". And "$525m" was typed into a district card, a
 * third money style beside `millions()`'s "$812.5M".
 *
 * # What is exempt, and why
 *
 * `/wiki/**`. It renders the corpus, and the corpus's figures are matched by value by the figure
 * bindings — "$812m" and a table cell reading "-0.024" are what a node says and what its binding
 * checks. Rewriting them would be editing a claim to suit a stylesheet. The same exemption
 * `plainCopy.spec.ts` makes for the same reason.
 *
 * Text a script writes after load is invisible here, as it is to every sweep of `dist/`.
 */
describe("figures in prose", () => {
  const EXEMPT = /^wiki(?:\.html|\/)/;

  const PATTERNS: [string, RegExp][] = [
    /*
     * A hyphen standing where a minus would: not after a word character, another dash, a point or
     * a slash, which is a range ("K-12", "2019-20"), a compound or a path.
     */
    ["a hyphen-minus", /(?<![\w\-–−./#&])-\d[\d,]*(?:\.\d+)?/g],
    ["billions without a currency sign", /(?<![$\d.,])\d+\.\d+B\b/g],
    ["a lowercase money suffix", /\$[\d,]+(?:\.\d+)?(?:m|b|bn|k)\b/g],
  ];

  /**
   * The page's visible text: the title and the body, without scripts, styles or templates.
   *
   * With a space at every cell and block boundary, which `textContent` does not put there: a row
   * reading `<th>H.B. 110</th><td>7.281B</td>` is "H.B. 1107.281B" without it, and the digit
   * before the figure hid the defect this sweep was written for.
   *
   * Drawings reduced to their text first (#646): a chart's axis ends and labels are visible text
   * and stay in scope; its marks are not, and are half the parse.
   */
  function visibleText(file: string): string {
    const html = drawingsAsText(readFileSync(file, "utf8")).replace(/<\/(?:td|th|p|li|dt|dd|div|h[1-6]|caption)>/g, " $&");
    const { document } = parseHTML(html);
    for (const node of document.querySelectorAll("script, style, template")) node.remove();
    const title = document.querySelector("title")?.textContent ?? "";
    return `${title}\n${document.body?.textContent ?? ""}`.replace(/\s+/g, " ");
  }

  const scanned = pages().filter((file) => !EXEMPT.test(relative(DIST, file)));

  test("the scan reaches the routes it is about", () => {
    expect(scanned.length).toBeGreaterThan(2000);
    const names = new Set(scanned.map((file) => relative(DIST, file)));
    for (const route of ["index.html", "outcomes.html", "data.html", "legislation.html", "statewide.html"]) {
      expect(names.has(route), route).toBe(true);
    }
  });

  test("the guard fires on the defect it exists for", () => {
    // Doctored lines from the pages #574 fixed: a scan that passes everywhere measures nothing
    // unless it can be shown to fail somewhere.
    const fires = (text: string) => PATTERNS.some(([, re]) => new RegExp(re.source).test(text));
    expect(fires("Poverty explains Ohio's attainment measure at -0.846. Every other")).toBe(true);
    expect(fires("H.B. 110 7.281B 312 297")).toBe(true);
    expect(fires("$525m distributed on a curve like that")).toBe(true);
    // And holds its fire on what is right, and on the ranges and compounds that look like it.
    expect(fires("at −0.846, $7.28B and $812.5M, grades K-12, FY2002–FY2003, 2024-25")).toBe(false);
  });

  test("no page outside /wiki writes a figure outside format.ts's shapes", () => {
    // Collected by match, so a string repeated on 609 district pages is reported once.
    const found = new Map<string, { page: string; pages: number }>();
    for (const file of scanned) {
      const text = visibleText(file);
      for (const [kind, re] of PATTERNS) {
        for (const match of text.matchAll(re)) {
          const at = match.index ?? 0;
          const key = `${kind}: "${match[0]}" in "…${text.slice(Math.max(0, at - 50), at + 40)}…"`;
          const seen = found.get(key);
          if (seen) seen.pages += 1;
          else found.set(key, { page: relative(DIST, file), pages: 1 });
        }
      }
    }
    const report = [...found].map(([key, { page, pages }]) => `${key} — ${page}, ${pages} page(s)`);
    expect(report, "format the value through `fixed`, `signed` or `millions` in src/lib/format.ts").toEqual([]);
  });
});

/**
 * Every number a chart draws, written at a reading precision (#610).
 *
 * # What this holds
 *
 * Chart text came at three precisions: "$1,234,567,890" at the end of a fan, "$812.5M" on a bar
 * beside it, "$1.2B" in the scale row underneath. A label on a chart stands for a position, not a
 * figure to quote — the tooltip and the table carry the full figure — so the axis ends, the direct
 * labels and the series ends go through `compactMoney`. And a bar a few pupils wide was labelled
 * "0%", which reads as a bar that should not be there; `pct` writes "<1%" for it.
 */
describe("chart text", () => {
  /** The classes whose text is a number the chart states. */
  const NUMERIC = /^(?:axis-foot|axis-head|bar-value|series-end)$/;
  const texts: { page: string; cls: string; text: string }[] = [];
  /** Each drawing carrying a "0%" direct label, for the width check below. */
  const zeroes: { page: string; svg: string }[] = [];
  for (const file of pages()) {
    const page = readFileSync(file, "utf8");
    if (!page.includes('class="plot')) continue;
    const at = file.slice(DIST.length + 1);
    for (const svg of page.match(/<svg\b[\s\S]*?<\/svg>/g) ?? []) {
      for (const group of svg.matchAll(/<g\b[^>]*\bclass="([^"]+)"[^>]*>([\s\S]*?)<\/g>/g)) {
        const cls = group[1]!.split(/\s+/)[0]!;
        if (!NUMERIC.test(cls)) continue;
        for (const t of group[2]!.matchAll(/<text\b[^>]*>([\s\S]*?)<\/text>/g)) {
          const text = t[1]!.replace(/<[^>]+>/g, "").trim();
          if (text) texts.push({ page: at, cls, text });
          if (cls === "bar-value" && text === "0%") zeroes.push({ page: at, svg });
        }
      }
    }
  }

  test("the sweep reads chart text of every kind it is about", () => {
    for (const cls of ["axis-foot", "bar-value", "series-end"]) {
      expect(texts.filter((t) => t.cls === cls).length, cls).toBeGreaterThan(20);
    }
  });

  test("no axis end, direct label or series end writes dollars to the dollar past a million", () => {
    const long = texts.filter((t) => /\$\d{1,3}(,\d{3}){2,}/.test(t.text));
    expect(
      long.slice(0, 10).map((t) => `${t.page} [${t.cls}] ${t.text}`),
      "write the chart's number through `compactMoney`",
    ).toEqual([]);
  });

  /** The "0%" direct labels in one drawing that sit on a bar wider than nothing. */
  const zeroOnWidth = (svg: string): string[] => {
    const doc = parseHTML(`<div>${svg}</div>`).document;
    const rects = [...doc.querySelectorAll("g.bar-fill rect")].map((r) => ({
      mid: Number(r.getAttribute("y")) + Number(r.getAttribute("height")) / 2,
      width: Number(r.getAttribute("width")),
    }));
    const wrong: string[] = [];
    for (const t of doc.querySelectorAll("g.bar-value text")) {
      if (t.textContent?.trim() !== "0%") continue;
      const at = /translate\([^,]+,([-\d.]+)/.exec(t.getAttribute("transform") ?? "")?.[1] ?? t.getAttribute("y");
      const bar = rects.find((r) => Math.abs(r.mid - Number(at)) <= 1);
      if (bar == null || bar.width > 0.5) wrong.push(`"0%" on a bar ${bar?.width ?? "?"}px wide`);
    }
    return wrong;
  };

  test("no bar of any width is labelled 0%", () => {
    /*
     * A direct label sits on its bar's row, so the bar it labels is the `.bar-fill` rect whose
     * middle is the text's y. A bar of zero width labelled "0%" is true; any other is not.
     */
    const wrong = zeroes.flatMap(({ page, svg }) => zeroOnWidth(svg).map((w) => `${page}: ${w}`));
    expect(wrong.slice(0, 10)).toEqual([]);
  });

  test("the 0% guard fires on a real drawing written the old way", () => {
    // The build carries no "0%" label to check, so the rule above passes on nothing unless it is
    // shown to fire: a real "<1%" bar, relabelled as it was before #610, must be caught.
    const sample = pages()
      .map((file) => readFileSync(file, "utf8"))
      .find((page) => page.includes(">&lt;1%</text>"));
    const svg = sample?.match(/<svg\b[\s\S]*?<\/svg>/g)?.find((s) => s.includes(">&lt;1%</text>"));
    expect(svg, "a drawing with a <1% label").toBeDefined();
    expect(zeroOnWidth(svg!.replace(">&lt;1%</text>", ">0%</text>")).length).toBeGreaterThan(0);
  });

  /** Whether a drawing states a number in one of the {@link NUMERIC} classes. */
  const statesANumber = (svg: string): boolean =>
    [...svg.matchAll(/<g\b[^>]*\bclass="([^"]+)"[^>]*>([\s\S]*?)<\/g>/g)].some(
      (group) =>
        NUMERIC.test(group[1]!.split(/\s+/)[0]!) &&
        [...group[2]!.matchAll(/<text\b[^>]*>([\s\S]*?)<\/text>/g)].some((t) => /\d/.test(t[1]!)),
    );

  /** Every drawing in the build, with the page it is on. */
  const drawings = pages().flatMap((file) =>
    (readFileSync(file, "utf8").match(/<svg\b[^>]*class="plot"[^>]*>[\s\S]*?<\/svg>/g) ?? []).map((svg) => ({
      page: file.slice(DIST.length + 1),
      svg,
    })),
  );

  test("every chart states at least one number", () => {
    /*
     * #611: the corpus bar charts drew a length on every bar and no number anywhere — not a value,
     * not a scale — so the TTAG node's seven panels could be compared with each other and read as
     * nothing. A category name or a year is not the number; the classes above are.
     */
    expect(drawings.length).toBeGreaterThan(1000);
    const mute = drawings.filter((d) => !statesANumber(d.svg));
    expect(
      mute.slice(0, 10).map((d) => `${d.page}: ${/aria-label="([^"]*)"/.exec(d.svg)?.[1] ?? "?"}`),
      "give the chart a direct label or an axis foot",
    ).toEqual([]);
  });

  test("the mute-chart guard fires on a corpus bar panel drawn without its scale", () => {
    const panel = drawings.find((d) => d.page.startsWith("wiki/") && d.svg.includes('class="bar-fill"') && !d.svg.includes('class="bar-value"'));
    expect(panel, "a corpus bar panel read against its foot").toBeDefined();
    expect(statesANumber(panel!.svg)).toBe(true);
    expect(statesANumber(panel!.svg.replace(/class="axis-foot"/g, 'class="unread"'))).toBe(false);
  });
});

/*
 * A legend whose keys are not each their own picture (#611). The TTAG spread drew two rules in one
 * dash and named them with two copies of one swatch, so the key told a reader there were two lines
 * and not which was which.
 */
test("no legend repeats a swatch", () => {
  const repeated: string[] = [];
  let legends = 0;
  for (const file of pages()) {
    const page = readFileSync(file, "utf8");
    for (const legend of page.match(/<div class="legend"[^>]*>[\s\S]*?<\/div>/g) ?? []) {
      const swatches = [...legend.matchAll(/<i class="sw"[^>]*data-series="([^"]+)"/g)].map((m) => m[1]!);
      if (swatches.length === 0) continue;
      legends++;
      if (new Set(swatches).size !== swatches.length) {
        repeated.push(`${file.slice(DIST.length + 1)}: ${swatches.join(", ")}`);
      }
    }
  }
  expect(legends).toBeGreaterThan(10);
  expect(repeated.slice(0, 10)).toEqual([]);
});
