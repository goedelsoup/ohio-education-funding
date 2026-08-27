/**
 * The keyboard cursor on a chart mark, and the two things that decide whether it is visible.
 *
 * # What #198 said, and what was actually there
 *
 * The issue reported the cursor at **1.87:1 light and 1.56:1 dark** against `--ordinal-3`, and
 * described the failure as "a keyboard reader landing on the darkest class of a scatter plot gets
 * a ring they cannot see as a ring". The contrast numbers are right about the tokens. The sentence
 * about the scatter is not, and the reason is that a cursor does not land on a mark — it lands on
 * whatever carries `data-hover`.
 *
 * Measured over the built site: **319,060 hover targets across 2,652 pages**, every one of them
 * `transparent` or `--series-formula`, with six on `--series-guarantee`. Not one carries an
 * ordinal token. Most charts draw an invisible hit layer above the marks — `.scatter-hit`,
 * `.range-hit` — so a reader can point at a district rather than at a 2.4px dot, and it is that
 * layer the ring surrounds.
 *
 * So the issue's table was a table of the ink against marks that exist, silently assuming the
 * cursor could land on each of them. It cannot. The first test here is that assumption made into
 * a check.
 *
 * # The defect that was real, in one chart
 *
 * `rangeSpec`'s hit band ran from the data's `min` to its `max`, so the row holding the maximum
 * had its `range-high` dot **centred on the band's right edge**, overhanging by the dot's radius —
 * 5.08 device pixels on `/counties`, which is `DOT_RADIUS` times the 1.69 that SVG is scaled up
 * by. `range-high` is `--ordinal-3` at **full opacity**, so the ring drawn 1px outside the band
 * ran straight through the one mark on this palette the ink does not clear 3:1 against. It also
 * meant the dot's outer half was not hoverable, on the row a reader is most likely to point at.
 *
 * The band is inset by `DOT_RADIUS` now. The second test holds it there.
 *
 * # Why the scatter never had the problem the issue described
 *
 * Two independent reasons, either of which is sufficient. The ring surrounds a transparent r=7 hit
 * circle over a r=2.4 dot, so it is nowhere near the dot's edge. And the ramp is composited at
 * `fill-opacity: 0.62` there, against which the ink clears anyway — 5.30 light and 3.44 dark. That
 * second figure is asserted in `palette.spec.ts`, beside the raw-token one it corrects.
 *
 * # The second channel, which is #220 and is here now
 *
 * `filter: brightness(1.2)` on a `fill: transparent` element brightens nothing, so on 81.5% of
 * hover targets the cursor was the outline alone. Each chart declares a `Cursor` in `plot/spec.ts`
 * now, `declareCursor` writes the pairing onto the hit layer, and `attachValues` moves both marks
 * together. The two tests at the bottom of this file are the check: one over the whole build that
 * every invisible target either names its mark or is named as having none, and one in a browser
 * that the naming actually reaches the paint.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";

import { expect, test } from "@playwright/test";

const DIST = join(import.meta.dirname, "../../dist");

function* walk(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(path);
    else if (entry.name.endsWith(".html")) yield path;
  }
}

/**
 * The fills a hover target may carry.
 *
 * `transparent` is the hit layers. The two series tokens are the bar charts, where the hover
 * target *is* the bar — measured against the ink at 4.46/3.64 and 6.15/3.88, both clear.
 *
 * An ordinal token appearing here is the regression this exists for, and it is not hypothetical in
 * the other direction: #187 found this ring drawn in `--series-formula` itself, 1.00:1 against the
 * bar it surrounded.
 */
const PERMITTED = new Set(["transparent", "var(--series-formula)", "var(--series-guarantee)"]);

test("no chart puts the keyboard cursor on a mark the ink cannot clear", () => {
  /*
   * Read from the markup rather than a browser, because the question is about every page and the
   * answer has to be about every page. Plot sets `fill` on the layer's `<g>` and only sometimes on
   * the element, so both are read and the element wins.
   */
  const seen = new Map<string, string>();
  let targets = 0;
  let pages = 0;

  for (const file of walk(DIST)) {
    const page = readFileSync(file, "utf8");
    if (!page.includes("data-hover")) continue;
    pages += 1;
    for (const match of page.matchAll(/<(circle|rect|path|line)\b([^>]*\bdata-hover=[^>]*)>/g)) {
      targets += 1;
      const own = /\bfill="([^"]*)"/.exec(match[2]!)?.[1];
      let inherited: string | undefined;
      for (const group of page.slice(0, match.index).matchAll(/<g\b[^>]*>/g)) {
        const fill = /\bfill="([^"]*)"/.exec(group[0])?.[1];
        if (fill) inherited = fill;
      }
      const fill = own ?? inherited ?? "(none)";
      if (!PERMITTED.has(fill)) {
        seen.set(fill, `${relative(DIST, file)} <${match[1]}>`);
      }
    }
  }

  expect(pages, "the build carries hover targets at all").toBeGreaterThan(1000);
  expect(targets, "and this many of them").toBeGreaterThan(100_000);
  expect(
    [...seen].map(([fill, where]) => `${fill} — ${where}`),
    "a hover target in a colour the cursor ring may not clear 3:1. Either the mark should sit " +
      "under a transparent hit layer, or the ring needs a treatment that survives this fill.",
  ).toEqual([]);
});

test("a range row's dots are inside the band the cursor rings", async ({ page }) => {
  /*
   * The geometry #198 actually turned on, asserted in a browser because it is a rendering fact.
   * `getBoundingClientRect` is what says whether a dot overhangs, and the overhang is invisible in
   * the markup: the band and the dots are separate marks and the SVG is scaled to its container,
   * so the 3 user units came out as 5.08 device pixels.
   *
   * Zero is the bar rather than a tolerance. A dot flush with the band's edge is inside it, and
   * the ring's own `outline-offset: 1px` is what separates the two — which it can only do if the
   * dot is not already past the band.
   */
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/counties");

  const measured = await page.evaluate(() => {
    // The SSR copy is in the document too and is not laid out, so it reports zero-width boxes.
    const svg = [...document.querySelectorAll("svg.plot")].find(
      (candidate) => candidate.getBoundingClientRect().width > 0 && candidate.querySelector(".range-hit"),
    );
    if (!svg) return null;
    const bands = [...svg.querySelectorAll(".range-hit > *")].map((b) => b.getBoundingClientRect());
    const worst: { edge: string; overhang: number; mark: string }[] = [];
    for (const mark of ["range-low", "range-high"]) {
      let edge = "", overhang = Infinity;
      for (const dot of svg.querySelectorAll(`.${mark} > *`)) {
        const d = dot.getBoundingClientRect();
        const band = bands.find((b) => d.top >= b.top - 12 && d.bottom <= b.bottom + 12);
        if (!band) continue;
        for (const [name, gap] of [
          ["left", d.left - band.left],
          ["right", band.right - d.right],
          ["top", d.top - band.top],
          ["bottom", band.bottom - d.bottom],
        ] as const) {
          if (gap < overhang) { overhang = gap; edge = name; }
        }
      }
      worst.push({ mark, edge, overhang: Number(overhang.toFixed(2)) });
    }
    return { rows: bands.length, worst };
  });

  expect(measured, "no laid-out range chart on /counties").not.toBeNull();
  expect(measured!.rows, "the county range chart draws a row per county").toBeGreaterThan(80);
  for (const { mark, edge, overhang } of measured!.worst) {
    expect(
      overhang,
      `a ${mark} dot hangs ${-overhang}px past its own hit band at the ${edge}. The cursor ring ` +
        `is drawn 1px outside that band, so it runs through the dot — and on range-high the dot ` +
        `is --ordinal-3 at full opacity, which the ink clears at only 1.87:1. Inset the band.`,
    ).toBeGreaterThanOrEqual(0);
  }
});

/**
 * The two hit layers with no mark to brighten, and why.
 *
 * Named here rather than inferred from a class, so that a third one cannot join them by being
 * written. The reasons are the `because` strings the specs carry; if they diverge, one of the two
 * is wrong and the divergence is the finding.
 */
const EXEMPT = new Map([
  [
    "fan-hit",
    "A full-height column over a continuous line: the band and both series are paths, so there " +
      "is no per-year mark to brighten.",
  ],
  ["series-hit", "The same, over two lines."],
]);

test("every invisible hover target names the mark it brightens, or is named as having none", () => {
  /*
   * A single ordered pass rather than the quadratic re-scan the first test does. The question is
   * per *layer* and not per mark — `declareCursor` writes one attribute on the `<g>`, which is
   * 485,000 fewer attributes than stamping an index on every element and its twin would be — so
   * what this needs is the last `<g>` seen before each hoverable element, which one pass gives.
   */
  const layers = new Map<string, { transparent: boolean; paired: boolean; where: string }>();
  let scanned = 0;

  for (const file of walk(DIST)) {
    const page = readFileSync(file, "utf8");
    if (!page.includes("data-hover")) continue;
    scanned += 1;
    let open: { cls: string; fill: string | undefined; paired: boolean } | null = null;
    const token = /<g\b([^>]*)>|<(?:circle|rect|path|line)\b([^>]*\bdata-hover=[^>]*)>/g;
    for (const match of page.matchAll(token)) {
      if (match[1] !== undefined) {
        const attrs = match[1];
        open = {
          cls: /\bclass="([^"]*)"/.exec(attrs)?.[1] ?? "",
          fill: /\bfill="([^"]*)"/.exec(attrs)?.[1],
          paired: attrs.includes("data-paired="),
        };
        continue;
      }
      if (!open) continue;
      const own = /\bfill="([^"]*)"/.exec(match[2]!)?.[1];
      const fill = own ?? open.fill ?? "(none)";
      const seen = layers.get(open.cls);
      if (!seen) {
        layers.set(open.cls, {
          transparent: fill === "transparent",
          paired: open.paired,
          where: relative(DIST, file),
        });
      } else if (fill !== "transparent") {
        seen.transparent = false;
      }
    }
  }

  expect(scanned, "the build carries charts at all").toBeGreaterThan(1000);
  expect(layers.size, "and more than one kind of hover layer").toBeGreaterThan(3);

  const unexplained = [...layers]
    .filter(([cls, l]) => l.transparent && !l.paired && !EXEMPT.has(cls))
    .map(([cls, l]) => `${cls} — invisible, pairs with nothing, unexplained (${l.where})`);
  expect(
    unexplained,
    "a hover target a reader cannot see, whose cursor is therefore the outline alone. Give its " +
      "spec a `Cursor` of `paired marks` naming the layer it should brighten, or of `none` with " +
      "the reason — and add it to EXEMPT above, which is where the reason has to be argued.",
  ).toEqual([]);

  // And the exemption cannot outlive the chart it was written for.
  const stale = [...EXEMPT.keys()].filter((cls) => !layers.has(cls));
  expect(stale, "an exemption for a hover layer the build no longer draws").toEqual([]);

  // The declaration is not a comment: the layers that claim a pairing carry the attribute.
  const declared = [...layers].filter(([, l]) => l.paired).map(([cls]) => cls);
  expect(declared.sort(), "the layers that pair").toEqual([
    "dist-hit",
    "range-hit",
    "scatter-hit",
  ]);
});

/**
 * The declaration reaches the paint.
 *
 * Everything above is about the document. This is the part that can be true in the markup and
 * false on the screen — a `data-paired` selector that resolves to nothing, a class the stylesheet
 * does not match, an index that lands on the wrong element. One route per paired chart form,
 * driven with the arrow keys the cursor is actually for.
 */
test.describe("the cursor's second channel", () => {
  for (const { route, hit, mark } of [
    { route: "/outcomes", hit: ".scatter-hit", mark: ".scatter-dot" },
    { route: "/counties", hit: ".range-hit", mark: ".range-high" },
    { route: "/district/043786", hit: ".dist-hit", mark: ".dist-dot" },
  ]) {
    test(`brightens the ${mark.slice(1)} under the ring on ${route}`, async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 900 });
      await page.goto(route);

      const svg = page.locator(`svg.plot:visible:has(${hit})`).first();
      await svg.focus();
      await page.keyboard.press("ArrowRight");

      const reading = await svg.evaluate(
        (node, selectors) => {
          const [hitSelector, markSelector] = selectors;
          const at = node.querySelector("[data-hover].at");
          const twins = [...(node.querySelectorAll(`${markSelector} > *`) ?? [])];
          const lit = twins.filter((t) => t.classList.contains("at-mark"));
          const style = lit[0] ? getComputedStyle(lit[0]) : null;
          return {
            ringed: at !== null,
            index: at ? [...at.parentElement!.children].indexOf(at) : -1,
            litIndexes: lit.map((t) => twins.indexOf(t)),
            filter: style?.filter ?? "",
            fillOpacity: style?.fillOpacity ?? "",
            hits: node.querySelectorAll(`${hitSelector} > *`).length,
            marks: twins.length,
          };
        },
        [hit, mark] as const,
      );

      expect(reading.ringed, "the arrow key put the cursor somewhere").toBe(true);
      expect(reading.hits, "the hit layer and the mark layer hold the same marks").toBe(
        reading.marks,
      );
      expect(
        reading.litIndexes,
        "exactly one mark is lit, and it is the one the ring is on",
      ).toEqual([reading.index]);
      expect(reading.filter, "and the stylesheet is acting on it").not.toBe("none");
      expect(reading.fillOpacity, "at full opacity, which is the channel a reader sees").toBe("1");

      // And it moves rather than accumulating: two lit marks would be two answers to "which one".
      await page.keyboard.press("ArrowRight");
      const after = await svg.evaluate(
        (node, markSelector) =>
          [...node.querySelectorAll(`${markSelector} > *`)]
            .map((t, i) => (t.classList.contains("at-mark") ? i : -1))
            .filter((i) => i >= 0),
        mark,
      );
      expect(after, "the cursor moved on and took its brightening with it").toEqual([
        reading.index + 1,
      ]);
    });
  }
});
