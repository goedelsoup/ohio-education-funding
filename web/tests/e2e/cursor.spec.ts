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

import { expect, type Page, test } from "@playwright/test";

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
    "plane-hit",
    "range-hit",
    "rank-hit",
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
/** One route per paired chart form, and per caller where two callers stamp the same form. */
const PAIRED = [
  { route: "/outcomes", hit: ".scatter-hit", mark: ".scatter-dot" },
  { route: "/counties", hit: ".range-hit", mark: ".range-high" },
  { route: "/district/043786", hit: ".dist-hit", mark: ".dist-dot" },
  { route: "/bounds", hit: ".rank-hit", mark: ".rank-dot" },
  {
    route: "/wiki/formula-component/temporary-transitional-aid-guarantee",
    hit: ".plane-hit",
    mark: ".plane-dot",
  },
  // The same route twice, because the same page carries two chart forms and the second is a
  // spread: `scatterSpec` marks in a panel the node builds, rather than the `/outcomes` cloud
  // this file already drives. A layer class is shared; the code path that stamps the pairing
  // onto it is not.
  {
    route: "/wiki/formula-component/temporary-transitional-aid-guarantee",
    hit: ".scatter-hit",
    mark: ".scatter-dot",
  },
  // And `range-hit` twice, for the same reason: `/counties` above is a page that calls
  // `rangeSpec` itself, and this is the band chart a corpus node binds and the wiki route
  // draws. Same form, different caller, and the caller is what stamps the pairing.
  {
    route: "/wiki/formula-component/fsfp-base-cost-calculation",
    hit: ".range-hit",
    mark: ".range-high",
  },
] as const;

test.describe("the cursor's second channel", () => {
  for (const { route, hit, mark } of PAIRED) {
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

/**
 * When the tip goes away (#613).
 *
 * It was easy to raise and hard to lower: a scroll left it fixed over whatever scrolled in, Escape
 * reached it only from inside a chart, and the gaps between marks blinked it off and on. These are
 * the three readings the issue took, as checks. Where the tip is placed is #600's.
 */
/**
 * Two frames, for a scroll to finish being a scroll. A scroll event is dispatched on the next
 * frame rather than when the scroll happens, so one raised by `scrollIntoViewIfNeeded` arrives
 * after a hover that follows it and takes the tip down — which is the behaviour under test.
 */
const settled = (page: Page) =>
  page.evaluate(
    () => new Promise((done) => requestAnimationFrame(() => requestAnimationFrame(done))),
  );

test.describe("the tip's dismissal", () => {
  test.describe("on a phone", () => {
    test.use({ hasTouch: true, viewport: { width: 375, height: 812 } });

    test("a tapped tip goes with its mark when the page scrolls", async ({ page }) => {
      await page.goto("/statewide");
      const mark = page.locator("svg.plot .bar-fill > *").first();
      await mark.scrollIntoViewIfNeeded();
      await settled(page);
      await mark.tap();
      const tip = page.locator("#tip");
      await expect(tip, "the tap raised the tip").toBeVisible();

      await page.evaluate(() => window.scrollBy(0, 250));
      await expect
        .poll(async () => {
          if (await tip.isHidden()) return 0;
          const [t, m] = await Promise.all([tip.boundingBox(), mark.boundingBox()]);
          return t && m ? Math.abs(t.y - (m.y + m.height)) : Infinity;
        }, { message: "hidden, or still beside the mark it reads" })
        .toBeLessThanOrEqual(30);
    });
  });

  test("Escape dismisses a hover tip with focus outside the chart", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto("/outcomes");
    const mark = page.locator("svg.plot:visible .scatter-hit > *").first();
    await mark.scrollIntoViewIfNeeded();
    await settled(page);
    await mark.hover({ force: true });
    const tip = page.locator("#tip");
    await expect(tip, "the hover raised the tip").toBeVisible();

    await page.keyboard.press("Escape");
    await expect(tip).toBeHidden();
  });

  test("the tip holds across the gap between two bars", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto("/statewide");
    const bars = page.locator("svg.plot:visible").filter({ has: page.locator(".bar-fill") }).first();
    await bars.scrollIntoViewIfNeeded();
    await settled(page);
    const [a, b] = await bars.evaluate((svg) =>
      [...svg.querySelectorAll(".bar-fill > *")].slice(0, 2).map((m) => {
        const r = m.getBoundingClientRect();
        return { left: r.left, right: r.right, top: r.top, bottom: r.bottom };
      }),
    );
    expect(a && b, "a chart with two bars").toBeTruthy();
    // Bars run down the chart or across it; the gap is on whichever side they do not share.
    const down = b!.top >= a!.bottom;
    const between = down
      ? { x: (Math.max(a!.left, b!.left) + Math.min(a!.right, b!.right)) / 2, y: (a!.bottom + b!.top) / 2 }
      : { x: (a!.right + b!.left) / 2, y: (Math.max(a!.top, b!.top) + Math.min(a!.bottom, b!.bottom)) / 2 };
    expect(down ? b!.top - a!.bottom : b!.left - a!.right, "the bars have a gap").toBeGreaterThan(0);
    /*
     * Every mark reaches 12px past its edge since #614, so the gap between two bars is now the
     * nearer bar's. The chart still has ground no mark reaches, out past the ends of the bars along
     * that same gap, and the tip has to hold there for the reason it had to hold in the gap.
     */
    const gap = await bars.evaluate(
      (svg, { start, down }) => {
        const own = svg.getBoundingClientRect();
        for (let step = 0; step < 2000; step += 2) {
          const x = down ? start.x + step : start.x;
          const y = down ? start.y : start.y - step;
          if (x > own.right || y < own.top) break;
          const under = document.elementFromPoint(x, y);
          if (under && svg.contains(under) && !under.closest("[data-hover]")) return { x, y };
        }
        return null;
      },
      { start: between, down },
    );
    expect(gap, "somewhere on the chart, along the gap, that no mark reaches").not.toBeNull();

    await page.mouse.move((a!.left + a!.right) / 2, (a!.top + a!.bottom) / 2);
    const tip = page.locator("#tip");
    await expect(tip, "the hover raised the tip").toBeVisible();
    await page.mouse.move(gap!.x, gap!.y, { steps: 4 });
    await expect(tip, "and the gap did not take it down").toBeVisible();
  });
});

/**
 * Where the tip is placed (#600). It went below and right of the pointer and was clamped on the
 * right only, so a chart scrolled to the foot of the window put the tip under the fold: 5 of 8
 * hovers on `/statewide` at 1280x800, and 6 of 10 on a district page.
 */
test.describe("the tip's placement", () => {
  /** Scroll the chart's foot to the window's, and return a point on its lowest mark. */
  const lowest = async (page: Page, route: string) => {
    await page.goto(route);
    const chart = page.locator("svg.plot:visible").filter({ has: page.locator("[data-hover]") }).last();
    await chart.scrollIntoViewIfNeeded();
    await chart.evaluate((svg) => window.scrollBy(0, svg.getBoundingClientRect().bottom - window.innerHeight));
    await settled(page);
    const at = await chart.evaluate((svg) => {
      const boxes = [...svg.querySelectorAll("[data-hover]")]
        .map((mark) => mark.getBoundingClientRect())
        .filter((box) => box.width > 0 && box.height > 0 && box.bottom <= window.innerHeight);
      const box = boxes.reduce((low, box) => (box.bottom > low.bottom ? box : low));
      // Its lower edge, where a pointer still reads it: the lowest place a hover can be.
      return { x: box.left + box.width / 2, y: box.bottom - 2 };
    });
    return at;
  };
  const inside = async (page: Page, at: { y: number }) => {
    const tip = page.locator("#tip");
    await expect(tip, "the tip is raised").toBeVisible();
    const box = (await tip.boundingBox())!;
    const { width, height } = page.viewportSize()!;
    // Or the check passes on a mark with room below it, and measures nothing.
    expect(at.y + 12 + box.height, "below the mark, the tip would have run off").toBeGreaterThan(height);
    expect(box.y, "the tip's top is on screen").toBeGreaterThanOrEqual(0);
    expect(box.y + box.height, "the tip's foot is on screen").toBeLessThanOrEqual(height);
    expect(box.x, "the tip's left is on screen").toBeGreaterThanOrEqual(0);
    expect(box.x + box.width, "the tip's right is on screen").toBeLessThanOrEqual(width);
  };

  for (const route of ["/statewide", "/counties", "/district/043802"]) {
    test(`a hover at the foot of the window keeps the tip on screen on ${route}`, async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 800 });
      const at = await lowest(page, route);
      await page.mouse.move(at.x, at.y);
      await inside(page, at);
    });
  }

  /*
   * #607. A fan's or a series' hit column is the full height of the frame, and the tip hung from
   * its foot sat over the axis note and the key under the chart — at y=626 against a frame ending
   * at 614 on Columbus. Every year is walked, because the anchor is the drawn value and the value
   * moves.
   */
  for (const { route, hit } of [
    { route: "/district/043802", hit: ".fan-hit" },
    { route: "/history", hit: ".series-hit" },
  ]) {
    test(`the keyboard tip on a ${hit.slice(1)} column clears the axis note and the key on ${route}`, async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 800 });
      await page.goto(route);
      const svg = page.locator(`svg.plot:visible:has(${hit})`).first();
      await svg.scrollIntoViewIfNeeded();
      await svg.focus();
      const years = await svg.evaluate((node, selector) => node.querySelectorAll(`${selector} > *`).length, hit);
      expect(years, "the chart has years to walk").toBeGreaterThan(1);
      const covered: string[] = [];
      for (let year = 0; year < years; year++) {
        await page.keyboard.press("ArrowRight");
        const tip = (await page.locator("#tip").boundingBox())!;
        const under = await svg.evaluate((node) => {
          const wrap = node.closest(".chartwrap");
          const key = wrap?.nextElementSibling?.matches(".legend") ? wrap.nextElementSibling : null;
          return [...node.querySelectorAll(".axis-foot"), ...(key ? [key] : [])].map((el) => {
            const b = el.getBoundingClientRect();
            return { name: el.textContent?.trim().slice(0, 30) ?? "", top: b.top, bottom: b.bottom, left: b.left, right: b.right };
          });
        });
        for (const b of under) {
          const apart =
            tip.x + tip.width <= b.left || b.right <= tip.x || tip.y + tip.height <= b.top || b.bottom <= tip.y;
          if (!apart) covered.push(`year ${year}: ${b.name}`);
        }
      }
      expect(covered).toEqual([]);
    });
  }

  test.describe("on a phone", () => {
    test.use({ hasTouch: true, viewport: { width: 375, height: 812 } });

    test("a tap at the foot of the window keeps the tip on screen", async ({ page }) => {
      const at = await lowest(page, "/statewide");
      await page.touchscreen.tap(at.x, at.y);
      await inside(page, at);
    });
  });
});

/**
 * The pointer, the keys and a tap, made to agree (#614).
 *
 * The keyboard lit the pair; a hover on six of nine forms raised a tip and changed nothing on the
 * chart, and a tap lit nothing anywhere. The arrows walked a cloud alphabetically. A click drew the
 * browser's ring round the whole chart. And on a phone every form but the fan was a target under
 * 24px.
 */
test.describe("the pointer's cursor", () => {
  /** The `index`th mark of the first visible chart's `hit` layer, scrolled to, with its box. */
  const aim = (page: Page, hit: string, index: number) =>
    page
      .locator(`svg.plot:visible:has(${hit})`)
      .first()
      .evaluate(
        (svg, [selector, i]) => {
          const marks = [...svg.querySelectorAll(`${selector} > *`)];
          const mark = marks[Math.min(Number(i), marks.length - 1)]!;
          mark.scrollIntoView({ block: "center" });
          const box = mark.getBoundingClientRect();
          return {
            index: marks.indexOf(mark),
            x: box.left + box.width / 2,
            y: box.top + box.height / 2,
            bottom: box.bottom,
          };
        },
        [hit, String(index)] as const,
      );

  /** Which marks of that chart's `layer` carry the class `cls`, by index. */
  const lit = (page: Page, hit: string, layer: string, cls: string) =>
    page
      .locator(`svg.plot:visible:has(${hit})`)
      .first()
      .evaluate(
        (svg, [selector, name]) =>
          [...svg.querySelectorAll(`${selector} > *`)]
            .map((m, i) => (m.classList.contains(name) ? i : -1))
            .filter((i) => i >= 0),
        [layer, cls] as const,
      );

  for (const { route, hit, mark } of PAIRED) {
    test(`a hover lights the ${mark.slice(1)} on ${route}, without the ring`, async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 900 });
      await page.goto(route);
      const at = await aim(page, hit, 3);
      await settled(page);
      await page.mouse.move(at.x, at.y);

      const hovered = await lit(page, hit, hit, "hover");
      expect(hovered, "one hit mark is the pointer's").toHaveLength(1);
      expect(await lit(page, hit, mark, "hover-mark"), "and the mark it names is lit").toEqual(hovered);
      const style = await page
        .locator(`svg.plot:visible:has(${hit})`)
        .first()
        .evaluate((svg, m) => ({
          fillOpacity: getComputedStyle(svg.querySelector(`${m} > .hover-mark`)!).fillOpacity,
          outline: getComputedStyle(svg.querySelector("[data-hover].hover")!).outlineStyle,
        }), mark);
      expect(style.fillOpacity, "at full opacity, the keyboard's second channel").toBe("1");
      expect(style.outline, "and no ring: the pointer is already standing on the mark").toBe("none");

      await page.mouse.move(1, 1);
      expect(await lit(page, hit, mark, "hover-mark"), "leaving the chart puts it out").toEqual([]);
    });
  }

  test("the arrow keys walk a cloud left to right", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto("/outcomes");
    const svg = page.locator("svg.plot:visible:has(.scatter-hit)").first();
    await svg.focus();
    const xs: number[] = [];
    for (let i = 0; i < 10; i++) {
      await page.keyboard.press("ArrowRight");
      xs.push(
        await svg.evaluate((node) => {
          const b = node.querySelector("[data-hover].at")!.getBoundingClientRect();
          return b.left + b.width / 2;
        }),
      );
    }
    expect(xs, "each step is at or right of the last").toEqual([...xs].sort((a, b) => a - b));
  });

  test("a click focuses a chart without the browser's ring round it", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto("/history");
    const svg = page.locator("svg.plot[tabindex]:visible").first();
    await svg.scrollIntoViewIfNeeded();
    await settled(page);
    await svg.click();
    const state = await svg.evaluate((node) => ({
      focused: document.activeElement === node,
      outline: getComputedStyle(node).outlineStyle,
    }));
    expect(state.focused, "the click did focus it, or this measures nothing").toBe(true);
    expect(state.outline).toBe("none");
  });

  test.describe("on a phone", () => {
    test.use({ hasTouch: true, viewport: { width: 375, height: 812 } });

    test("a tap lights the mark it reads", async ({ page }) => {
      await page.goto("/outcomes");
      const at = await aim(page, ".scatter-hit", 10);
      await settled(page);
      await page.touchscreen.tap(at.x, at.y);
      await expect(page.locator("#tip")).toBeVisible();
      const hovered = await lit(page, ".scatter-hit", ".scatter-hit", "hover");
      expect(hovered).toHaveLength(1);
      expect(await lit(page, ".scatter-hit", ".scatter-dot", "hover-mark")).toEqual(hovered);
    });

    /*
     * Not each target's box, which is what the issue measured: 84 range rows 11px apart cannot each
     * have a 24px box without overlapping, and the browser gives an overlap to the row drawn last.
     * The reach is a transparent stroke instead (see `app.css`), so this asks the browser's own hit
     * test whether the points 11.5px either side of a mark's centre, across a dimension under 24px,
     * find the mark.
     */
    for (const { route, hit } of [
      { route: "/outcomes", hit: ".scatter-hit" },
      { route: "/counties", hit: ".range-hit" },
      { route: "/bounds", hit: ".rank-hit" },
      { route: "/district/043786", hit: ".dist-hit" },
      { route: "/district/043786", hit: ".fan-hit" },
      { route: "/district/043786", hit: ".bar-fill" },
      { route: "/history", hit: ".series-hit" },
      { route: "/statewide", hit: ".bar-fill" },
      { route: "/wiki/formula-component/temporary-transitional-aid-guarantee", hit: ".plane-hit" },
    ]) {
      test(`every ${hit.slice(1)} target on ${route} reaches 24px`, async ({ page }) => {
        await page.goto(route);
        const reading = await page
          .locator(`svg.plot:visible:has(${hit})`)
          .first()
          .evaluate((svg, selector) => {
            const short: string[] = [];
            const marks = [...svg.querySelectorAll(`${selector} > [data-hover]`)];
            for (const mark of marks) {
              mark.scrollIntoView({ block: "center", inline: "center" });
              const b = mark.getBoundingClientRect();
              const cx = b.left + b.width / 2;
              const cy = b.top + b.height / 2;
              const probes: [number, number][] = [];
              if (b.height < 24) probes.push([cx, cy - 11.5], [cx, cy + 11.5]);
              if (b.width < 24) probes.push([cx - 11.5, cy], [cx + 11.5, cy]);
              // Only inside the chart. A series' first year sits on the drawing's left edge, and
              // the card's `.scroll` clips its reach there: a tap beside the chart is not a tap
              // on it, and the reach inward is the one a reader has.
              const own = svg.getBoundingClientRect();
              const inside = ([x, y]: [number, number]) =>
                x >= own.left && x <= own.right && y >= own.top && y <= own.bottom;
              const missed = ([x, y]: [number, number]) =>
                !document.elementsFromPoint(x, y).includes(mark);
              if (probes.filter(inside).some(missed)) {
                short.push(`#${marks.indexOf(mark)} ${b.width.toFixed(1)}×${b.height.toFixed(1)}`);
              }
            }
            return { short, of: marks.length };
          }, hit);
        expect(reading.of, "the chart has targets").toBeGreaterThan(1);
        expect(reading.short, `targets that reach under 24px, of ${reading.of}`).toEqual([]);
      });
    }

    test("where two reaches overlap, the nearer mark has it", async ({ page }) => {
      /*
       * A range row reaches 12px past its edge and the rows are 11px apart, so the bottom of one
       * row is inside the reach of the next. The browser answers with the row drawn last, which is
       * the next one down; the reader is pointing at this one.
       */
      await page.goto("/counties");
      const row = await aim(page, ".range-hit", 20);
      await settled(page);
      const topmost = await page.evaluate(
        ([x, y]) => {
          const t = document.elementFromPoint(x!, y!);
          return t ? [...t.parentElement!.children].indexOf(t) : -1;
        },
        [row.x, row.bottom - 1],
      );
      expect(topmost, "the browser alone would answer with the next row").toBe(row.index + 1);
      await page.touchscreen.tap(row.x, row.bottom - 1);
      expect(await lit(page, ".range-hit", ".range-hit", "hover")).toEqual([row.index]);
    });
  });
});
