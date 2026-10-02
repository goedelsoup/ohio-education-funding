/**
 * A chart at phone width, without a mouse, and the two annotations drawn beside it.
 *
 * Grouped because all four blocks are about the drawing rather than about the numbers in it: that a
 * narrow viewport gets the narrow drawing and it still fits, that every value a hover would reveal
 * is reachable from a keyboard and from a table, that a chip says which year the marks are from,
 * and that a colour legend says what the third variable is. The arithmetic behind the marks is the
 * unit suite's; `cursor.spec.ts` holds the focus ring.
 *
 * The one assertion in `a year chip` that read the build rather than a page — that no chip ships a
 * `title` — is in `tests/dist/semantics.spec.ts`.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test, type Page } from "@playwright/test";

import { chartedRoutes } from "./charted.ts";
import { CLEVELAND } from "./helpers.ts";

/*
 * Charts, at the width they are actually read at.
 *
 * Every chart is drawn at three widths — see `WIDTHS` in `src/lib/plot/spec.ts` — because a
 * static SVG scaled to a phone takes its axis text down with it. The defect these replace: at
 * 375px the 640-unit drawing was scaled by 0.46 and the tick labels rendered near 4.6px, which is
 * half the size at which text is legible, on the viewport most first visits arrive at. Nothing
 * caught it, because a scaled SVG is correct markup and its `font-size` attribute still says 10.
 *
 * So these measure what the browser paints rather than what the file says. A unit test cannot:
 * the scale factor only exists once the SVG is in a box of a known width.
 */
test.describe("charts on a phone", () => {
  /*
   * Every route that draws a chart, read off the build — see `charted.ts`. This was a hand list
   * that left out `/bounds` and four of the five wiki routes with plots, so none of the four
   * tests below had ever run on the charts #608 found leaving their frames.
   *
   * Read in `beforeAll` rather than at import: locally the build is the web server's, and it is
   * not there yet when this file is loaded.
   */
  let CHARTED: string[] = [];
  test.beforeAll(() => {
    CHARTED = chartedRoutes();
    expect(CHARTED, "the build has charts to visit").toContain(`/district/${CLEVELAND}`);
  });

  /**
   * Open a route with its charts drawn. The scenario routes draw theirs in the browser after a
   * fetch, so a page that has only loaded has no chart yet, and every assertion below would pass
   * on nothing. The baked routes are drawn when they load, and not all of them show a chart at
   * every width, so they are not waited on.
   */
  const visit = async (page: Page, route: string) => {
    await page.goto(route);
    if (route.startsWith("/scenario/")) await page.locator("svg.plot:visible").first().waitFor();
  };

  /** Every text mark in a chart the reader can actually see, with the size it is painted at. */
  const paintedText = (page: Page) =>
    page.evaluate(() => {
      const out: { chart: string; text: string; px: number; outside: number }[] = [];
      for (const svg of document.querySelectorAll("svg.plot")) {
        // The drawing for the other width is `display: none`, so it paints nothing and is not
        // what any of this is about.
        if (!svg.getClientRects().length) continue;
        const frame = svg.getBoundingClientRect();
        const box = (svg as SVGSVGElement).viewBox.baseVal;
        const scale = box.width ? frame.width / box.width : 1;
        const chart = svg.closest("[data-chart]")?.getAttribute("data-chart") ?? "unnamed";
        for (const mark of svg.querySelectorAll("text")) {
          const text = (mark.textContent ?? "").trim();
          if (!text) continue;
          const at = mark.getBoundingClientRect();
          out.push({
            chart,
            text,
            px: (parseFloat(getComputedStyle(mark).fontSize) || 10) * scale,
            outside: Math.max(frame.left - at.left, at.right - frame.right),
          });
        }
      }
      return out;
    });

  test("on a phone, chart text is painted between 9px and 12px", async ({ page }) => {
    // 375 is the common iPhone and 360 the common Android, the narrowest worth drawing for. The
    // floor is 9px rather than a round 10 because `WIDTHS.narrow` is sized to scale by at least 0.9
    // at 360, and the 10px axis marks are the smallest type any of these forms uses. Only 375 was
    // tested until #609, and at 360 the county labels painted at 8.8px. The ceiling is the 12.8px
    // HTML legend beside these charts: a chart should not be both the smallest and the largest
    // type on a phone page.
    const outside: string[] = [];
    for (const width of [360, 375]) {
      await page.setViewportSize({ width, height: 900 });
      for (const route of CHARTED) {
        await visit(page, route);
        for (const mark of await paintedText(page)) {
          if (mark.px < 9 || mark.px > 12) {
            outside.push(`${route} at ${width} [${mark.chart}] "${mark.text}" at ${mark.px.toFixed(1)}px`);
          }
        }
      }
    }
    expect(outside.slice(0, 10), "chart text outside the phone band").toEqual([]);
  });

  test("between a phone and a desktop, a chart fills its box and its type moves in small steps", async ({
    page,
  }) => {
    /*
     * #609. With two drawings, the narrow one stopped growing at 400px and its box ran on to the
     * 576px swap — a 400px fan in a 552px card — and then the wide drawing took over 46% wider,
     * its type a third smaller. Every phone and desktop test passed throughout, because the band
     * is neither.
     *
     * So this walks the band in 16px steps, on every charted route, and holds two things.
     *
     * - A drawing fills at least 85% of its box, up to the 800px a drawing is capped at. 85% is
     *   what the swaps leave by construction — see "Why three" on `WIDTHS` — and not a tolerance.
     * - The largest type in a chart never falls by more than a fifth from one step to the next.
     *   A fifth is the step a swap makes by construction: the drawing before it is at its cap and
     *   the one after at its own width, 1/1.25. A layout change that shrinks a chart's box — a
     *   rail opening beside it at 960 — is held to the same step.
     *
     * One load per route and the viewport walked under it: the container queries answer a resize
     * without a reload, which is the behaviour a reader dragging a window gets.
     */
    const thin: string[] = [];
    const jumps: string[] = [];
    for (const route of CHARTED) {
      await page.setViewportSize({ width: 360, height: 900 });
      await visit(page, route);
      let before = new Map<string, number>();
      for (let width = 360; width <= 1104; width += 16) {
        await page.setViewportSize({ width, height: 900 });
        const charts = await page.evaluate(() => {
          const seen = new Map<string, number>();
          const out: { key: string; box: number; painted: number; type: number }[] = [];
          for (const svg of document.querySelectorAll<SVGSVGElement>(".chart-pair svg.plot")) {
            if (!svg.getClientRects().length) continue;
            if (svg.closest(".chart-at")?.getAttribute("data-at") === "panel") continue;
            const name = svg.closest("[data-chart]")?.getAttribute("data-chart") ?? "unnamed";
            const n = seen.get(name) ?? 0;
            seen.set(name, n + 1);
            const painted = svg.getBoundingClientRect().width;
            const scale = painted / svg.viewBox.baseVal.width;
            const sizes = [...svg.querySelectorAll("text")]
              .filter((t) => (t.textContent ?? "").trim())
              .map((t) => (parseFloat(getComputedStyle(t).fontSize) || 10) * scale);
            out.push({
              key: `${name}#${n}`,
              box: svg.closest(".chart-pair")!.getBoundingClientRect().width,
              painted,
              type: sizes.length ? Math.max(...sizes) : 0,
            });
          }
          return out;
        });
        const now = new Map<string, number>();
        for (const chart of charts) {
          const want = 0.85 * Math.min(chart.box, 800);
          if (chart.painted < want - 0.5) {
            thin.push(
              `${route} at ${width} [${chart.key}] ${Math.round(chart.painted)}px in a ${Math.round(chart.box)}px box`,
            );
          }
          if (!chart.type) continue;
          now.set(chart.key, chart.type);
          const last = before.get(chart.key);
          if (last && chart.type < 0.8 * last - 0.05) {
            jumps.push(
              `${route} at ${width} [${chart.key}] ${last.toFixed(1)}px to ${chart.type.toFixed(1)}px`,
            );
          }
        }
        before = now;
      }
    }
    expect(thin.slice(0, 10), "drawings that leave their box blank").toEqual([]);
    expect(jumps.slice(0, 10), "chart type that jumps between neighbouring widths").toEqual([]);
  });

  test("the reach plot is the wide drawing on a desktop", async ({ page }) => {
    // #609. The rail opens beside the stage at 960, and on a reading-width page that left the
    // stage 548px: under the swap, so from 968 up the plot was drawn at its phone size and halved
    // as the window widened. The page is the wide `.wrap` now.
    for (const width of [1024, 1440]) {
      await page.setViewportSize({ width, height: 900 });
      await visit(page, "/scenario/reach");
      const shown = page.locator('[data-chart="positions"] svg.plot:visible');
      await expect(shown).toHaveCount(1);
      expect(
        await shown.evaluate((svg) => svg.closest(".chart-at")?.getAttribute("data-at")),
        `${width}px`,
      ).toBe("wide");
      expect((await shown.boundingBox())!.width, `${width}px`).toBeGreaterThanOrEqual(600);
    }
  });

  test("a note under a chart does not run past the drawing it qualifies", async ({ page }) => {
    // A figure reads as one width. At 1440 every wide drawing is at its 800px cap inside a wider
    // column, and a note under it is held to `--measure-note`, which ends inside the drawing; this
    // keeps it there. 8px of slack is a glyph's overhang, not a layout.
    await page.setViewportSize({ width: 1440, height: 900 });
    const over: string[] = [];
    let seen = 0;
    for (const route of CHARTED) {
      await visit(page, route);
      const runs = await page.locator(".chartwrap + p.note").evaluateAll((notes) =>
        notes.flatMap((note) => {
          const svg = [...note.previousElementSibling!.querySelectorAll("svg.plot")].find(
            (s) => s.getClientRects().length,
          );
          if (!svg || !note.getClientRects().length) return [];
          return [note.getBoundingClientRect().right - svg.getBoundingClientRect().right];
        }),
      );
      seen += runs.length;
      for (const run of runs) if (run > 8) over.push(`${route}: ${Math.round(run)}px past`);
    }
    expect(seen, "the routes carry notes under charts").toBeGreaterThan(0);
    expect(over.slice(0, 10), "notes wider than their chart").toEqual([]);
  });

  test("on a wide page, no chart text is painted larger than the body text", async ({ page }) => {
    /*
     * The other end of the same scale. A drawing is `width: 100%` over its `viewBox`, so before
     * `MAX_SCALE` it grew with its column: `/counties`' dumbbell labels painted at 16.9–18.6px at
     * 1280, over 15px prose, and the same form was a different size on a 900px page and an 1180px
     * one. 15px is `12 × MAX_SCALE` — the largest type any form sets, at the most it is scaled —
     * and the hundredth is float slack on an exact product, not tolerance.
     */
    await page.setViewportSize({ width: 1280, height: 900 });
    const large: string[] = [];
    for (const route of CHARTED) {
      await visit(page, route);
      for (const mark of await paintedText(page)) {
        if (mark.px > 15.01) large.push(`${route} [${mark.chart}] "${mark.text}" at ${mark.px.toFixed(1)}px`);
      }
    }
    expect(large.slice(0, 10), "chart text painted over body size").toEqual([]);
  });

  test("no chart draws a label outside its own frame", async ({ page }) => {
    /*
     * The other half of drawing narrow. Every gutter in `spec.ts` is sized to the text that goes
     * in it, and text does not shrink with the frame — so a name that fits a 640 drawing asks for
     * three quarters of a 320 one. Capped, it has to wrap; uncapped, `Career-technical education`
     * is drawn off the left edge of the viewBox and arrives as `er-technical education`, which
     * reads as a rendering fault rather than as a truncation and is exactly the kind of thing
     * nobody reports.
     */
    // 375 for the narrow drawing and 600 for the middle one, which every route shows there (#609).
    const clipped: string[] = [];
    for (const width of [375, 600]) {
      await page.setViewportSize({ width, height: 900 });
      for (const route of CHARTED) {
        await visit(page, route);
        for (const mark of await paintedText(page)) {
          if (mark.outside > 0.5) {
            clipped.push(`${route} at ${width} [${mark.chart}] "${mark.text}" over by ${Math.round(mark.outside)}px`);
          }
        }
      }
    }
    expect(clipped.slice(0, 10), "chart labels drawn past the edge of the drawing").toEqual([]);
  });

  test("no two chart labels are drawn through each other", async ({ page }) => {
    /*
     * A narrow frame is where the annotation along the foot stops fitting on one line: `/history`
     * drew `FY2009`, `axis starts at 32%, not zero` and `FY2022` across 186px and the centre ran
     * through both years. `axisFoot` measures the row and drops the centre to its own line — and
     * measuring it is only possible here, because a string's drawn width is a fact about the font.
     *
     * Any overlap at all, rather than a pixel of slack. The slack is what let this through once:
     * the two foot rows were laid exactly touching, which came out as a 0.1px overlap under the
     * font this was written against and a 1.1px one under DejaVu Sans, so a tolerance of a pixel
     * made the defect a property of the reviewer's machine. Zero pairs on this build clear even a
     * 1px *gap* requirement, so there is no slack being spent here.
     */
    const through: string[] = [];
    for (const [route, width] of CHARTED.flatMap((r) => [375, 600].map((w) => [r, w] as const))) {
      await page.setViewportSize({ width, height: 900 });
      await visit(page, route);
      const hits = await page.evaluate(() => {
        const out: string[] = [];
        for (const svg of document.querySelectorAll("svg.plot")) {
          if (!svg.getClientRects().length) continue;
          const marks = [...svg.querySelectorAll("text")]
            .filter((m) => (m.textContent ?? "").trim())
            .map((m) => ({ t: (m.textContent ?? "").trim(), r: m.getBoundingClientRect() }))
            .filter((m) => m.r.width > 0);
          for (let a = 0; a < marks.length; a += 1) {
            for (let b = a + 1; b < marks.length; b += 1) {
              const A = marks[a]!.r;
              const B = marks[b]!.r;
              const over =
                Math.min(A.right, B.right) - Math.max(A.left, B.left) > 0 &&
                Math.min(A.bottom, B.bottom) - Math.max(A.top, B.top) > 0;
              if (over) out.push(`"${marks[a]!.t}" through "${marks[b]!.t}"`);
            }
          }
        }
        return out;
      });
      for (const hit of hits) through.push(`${route} at ${width}: ${hit}`);
    }
    expect(through.slice(0, 10), "overlapping chart type").toEqual([]);
  });

  test("the reader is shown one of the three drawings, never two and never none", async ({
    page,
  }) => {
    // The container queries are the only thing choosing between them, so a stylesheet that
    // stopped loading, or a breakpoint edited to leave a gap, would show charts stacked or none at
    // all — both of which read as a broken page rather than as a missing rule. A width in each
    // band, and one either side of each swap.
    for (const width of [375, 500, 519, 520, 640, 719, 720, 900, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      await page.goto(`/district/${CLEVELAND}`);
      const shown = await page.locator(".chart-pair").evaluateAll((pairs) =>
        pairs.map((pair) =>
          [...pair.querySelectorAll(".chart-at")].filter((at) => at.getClientRects().length).length,
        ),
      );
      expect(shown.length, `${width}px`).toBeGreaterThan(0);
      expect([...new Set(shown)], `${width}px shows one drawing per chart`).toEqual([1]);
    }
  });

  test("a key sits directly above its chart, and every dek is one size", async ({ page }) => {
    /*
     * #657. Keys sat above three charts and below five, and on `/outcomes` one key served two
     * scatters from a paragraph above the first. The rule is the one `.legend` states in
     * `app.css`: a key's next sibling is the drawing it keys — a `.chartwrap`, the `.panels` of a
     * small multiple, or the CSS `.bar` on a district's aid card.
     *
     * And a dek, the note directly above a chart (or above its key), computed 16.96px as a card's
     * own child and 14.08px a level down, so the Change tab carried both in one card. One size,
     * wherever it is nested.
     */
    await page.setViewportSize({ width: 1280, height: 900 });
    const astray: string[] = [];
    const sizes = new Map<string, string>();
    let keys = 0;
    for (const route of CHARTED) {
      await visit(page, route);
      const found = await page.evaluate(() => ({
        keys: [...document.querySelectorAll(".legend")].map((key) => ({
          ok: key.nextElementSibling?.matches(".chartwrap, .panels, .bar") ?? false,
          text: (key.textContent ?? "").replace(/\s+/g, " ").trim().slice(0, 40),
        })),
        deks: [...document.querySelectorAll("p.note")]
          .filter((note) => {
            const next = note.nextElementSibling;
            return (
              next?.matches(".chartwrap") ||
              (next?.matches(".legend") && next.nextElementSibling?.matches(".chartwrap"))
            );
          })
          .map((note) => ({
            px: getComputedStyle(note).fontSize,
            text: (note.textContent ?? "").replace(/\s+/g, " ").trim().slice(0, 40),
          })),
      }));
      keys += found.keys.length;
      for (const key of found.keys) if (!key.ok) astray.push(`${route}: "${key.text}"`);
      for (const dek of found.deks) if (!sizes.has(dek.px)) sizes.set(dek.px, `${route}: "${dek.text}"`);
    }
    expect(keys, "the routes carry keys").toBeGreaterThan(0);
    expect(astray, "a key with something other than its chart below it").toEqual([]);
    expect([...sizes.entries()], "deks at more than one size").toHaveLength(1);
  });
});

/*
 * The values behind the marks, and the reckoning behind a chip.
 *
 * Both were mouse-only. 159,530 marks carried their value in `data-hover` and the layer that read
 * it bound `mousemove` and nothing else; 12,449 chips carried their reckoning in a `title`, which
 * is not announced by most screen readers, cannot be reached from a keyboard, and does not exist
 * on a touch screen at all.
 */
test.describe("a chart's values, beyond the mouse", () => {
  /** What the page is currently saying a mark is worth, by all three channels at once. */
  const reading = (page: Page) =>
    page.evaluate(() => ({
      marked: document.querySelectorAll("svg.plot [data-hover].at").length,
      on: document.querySelector("svg.plot [data-hover].at")?.getAttribute("data-hover") ?? null,
      tip:
        document.querySelector<HTMLElement>("#tip")?.hidden === false
          ? (document.querySelector("#tip")?.textContent ?? "")
          : null,
      said: document.querySelector("#said")?.textContent ?? "",
    }));

  test("the arrow keys walk the marks, and say what each is worth", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const chart = page.locator("svg.plot[tabindex='0']:visible").first();
    await chart.scrollIntoViewIfNeeded();
    await chart.focus();

    // Focusing a chart reads nothing on its own: the reader is on the graphic, not in it.
    expect(await reading(page)).toMatchObject({ marked: 0, tip: null, said: "" });

    await page.keyboard.press("ArrowRight");
    const first = await reading(page);
    expect(first.marked, "exactly one mark carries the cursor").toBe(1);
    expect(first.on).toBeTruthy();
    // The same value by both routes: the tooltip a reader sees, the live region a reader hears.
    expect(first.tip).toBe(first.on);
    expect(first.said).toBe(first.on);

    await page.keyboard.press("ArrowRight");
    const second = await reading(page);
    expect(second.on, "the cursor moved").not.toBe(first.on);
    expect(second.marked, "and did not leave the last one behind").toBe(1);

    await page.keyboard.press("ArrowLeft");
    expect((await reading(page)).on, "and moves back").toBe(first.on);

    await page.keyboard.press("End");
    const end = await reading(page);
    await page.keyboard.press("Home");
    expect((await reading(page)).on, "Home and End are the two ends").not.toBe(end.on);

    await page.keyboard.press("Escape");
    expect(await reading(page), "Escape steps out of the chart").toMatchObject({
      marked: 0,
      tip: null,
      said: "",
    });
  });

  test("a chart is one tab stop, not one per mark", async ({ page }) => {
    /*
     * The reason there is a cursor at all. This district page draws 159 marks a reader can point
     * at; making each of them focusable would put 159 tab stops between the top of the page and
     * the next link, which is a worse defect than the one being fixed.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const counts = await page.evaluate(() => {
      const drawn = [...document.querySelectorAll("svg.plot")].filter(
        (svg) => svg.getClientRects().length,
      );
      return {
        charts: drawn.length,
        stops: drawn.filter((svg) => svg.getAttribute("tabindex") === "0").length,
        marks: drawn.reduce((n, svg) => n + svg.querySelectorAll("[data-hover]").length, 0),
        focusableMarks: document.querySelectorAll("svg.plot [data-hover][tabindex]").length,
      };
    });
    expect(counts.marks, "the page has marks worth reading").toBeGreaterThan(50);
    expect(counts.focusableMarks, "and not one of them is its own tab stop").toBe(0);
    expect(counts.stops, "every chart with marks is reachable").toBeGreaterThan(0);
    expect(counts.stops).toBeLessThanOrEqual(counts.charts);
  });

  test("the tab stop and its instruction arrive together, or not at all", async ({ page }) => {
    // The cursor is script, so the affordance advertising it is added by script too. A `tabindex`
    // baked into the build would be a stop that goes nowhere for a reader running none.
    await page.goto(`/district/${CLEVELAND}`);
    const label = await page.locator("svg.plot[tabindex='0']:visible").first().getAttribute("aria-label");
    expect(label, "the chart still says what it plots").toBeTruthy();
    expect(label, "and that it can be read").toContain("arrow keys");
  });

  test("a tap reaches a value, where there is no pointer to hover", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const shown = await page.evaluate(() => {
      const mark = [...document.querySelectorAll("svg.plot [data-hover]")].find(
        (m) => m.getClientRects().length,
      );
      mark?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      const tip = document.querySelector<HTMLElement>("#tip");
      return { hidden: tip?.hidden, text: tip?.textContent ?? "", want: mark?.getAttribute("data-hover") };
    });
    expect(shown.hidden, "a tap on a mark shows its value").toBe(false);
    expect(shown.text).toBe(shown.want);
  });
});

test.describe("a year chip", () => {
  test("says its reckoning to a keyboard, and to a screen reader", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const wrap = page.locator(".year-chip-wrap").first();
    const chip = wrap.locator("button.year-chip");

    // The whole of what the chip means is the button's own name, so nothing has to be pointed at
    // to reach it. `yearTitle` opens with the label the chip shows, so the visible text is a
    // prefix of the accessible name.
    const label = (await chip.getAttribute("aria-label")) ?? "";
    expect(label).toContain(((await chip.textContent()) ?? "").trim());
    expect(label, "the reckoning, not just the digits").toMatch(/fiscal year|tax year|school year/);
    expect(label, "and where the figure came from").toContain("Source:");

    // And the panel a sighted reader gets opens on focus rather than only on hover.
    const panel = wrap.locator(".year-chip-def");
    await expect(panel).toBeHidden();
    await chip.focus();
    await expect(panel).toBeVisible();
    await expect(panel).toHaveText(label);
  });
});

test.describe("colour that carries a third variable", () => {
  test("the spending charts are banded by poverty, each line named where it ends", async ({
    page,
  }) => {
    /*
     * A median line says what the middle of a cloud does. Banding says what the cloud is made of,
     * and here that is the card's whole argument: the three poverty thirds sit at the same
     * spending per need-weighted pupil and at different attainment.
     *
     * The ramp's end steps sit near 2.2:1 against their surface, which obliges a key in text ink.
     * One legend keyed both charts from above the first until #657; each chart now names its own
     * three lines in its gutter, behind a swatch of each band.
     */
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    await expect(card.locator(".legend")).toHaveCount(0);
    for (const key of ["weighted-spending", "enrolled-spending"]) {
      const chart = card.locator(`[data-chart="${key}"] svg.plot:visible`);
      await expect(chart.locator(".scatter-trace")).toHaveCount(3);
      await expect(chart.locator(".scatter-band-key > *")).toHaveCount(3);
      expect((await chart.locator(".scatter-trace-end").allTextContents()).sort()).toEqual([
        "Least poor",
        "Middle",
        "Poorest",
      ]);
    }

    // Three distinct fills across the dots, and they are the ramp rather than the series pair.
    const fills = await card
      .locator('[data-chart="weighted-spending"] .scatter-dot circle')
      .evaluateAll((nodes) => [...new Set(nodes.map((n) => n.getAttribute("fill")))]);
    expect(fills.sort()).toEqual([
      "var(--ordinal-1)",
      "var(--ordinal-2)",
      "var(--ordinal-3)",
    ]);
  });

  test("the two denominators are one screen, each under its own lead-in", async ({ page }) => {
    /*
     * The card's argument is a contrast between its two charts, and a note between them put the
     * pair at 949px at 1280, so they were never on one screen together (#657). The readings follow
     * the pair; what stays between them is the second chart's one-line lead-in. And the contrast
     * is printed, flat against falling, where it was only in the prose.
     */
    await page.setViewportSize({ width: 1280, height: 800 });
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    const first = card.locator('[data-chart="weighted-spending"]');
    const second = card.locator('[data-chart="enrolled-spending"]');
    const lead = (chart: typeof first) => chart.locator("xpath=preceding-sibling::*[1]");
    await expect(lead(first)).toContainText("Flat:");
    await expect(lead(second)).toContainText("Falls:");
    const top = await first.evaluate((el) => el.getBoundingClientRect().top);
    const bottom = await second.evaluate((el) => el.getBoundingClientRect().bottom);
    expect(bottom - top, "the first scatter's top to the second's foot").toBeLessThanOrEqual(800);
  });

  test("the millage chart splits on the variable that explains it, not a proxy", async ({
    page,
  }) => {
    /*
     * Banding that chart by valuation was tried and is floor status in disguise: the terciles'
     * exact-reproduction counts track their at-floor counts almost exactly. Floor status is two
     * states, so it takes the categorical pair rather than three steps of one hue.
     */
    await page.goto("/method");
    const card = page.locator("#reduction-factors");
    const fills = await card
      .locator(".scatter-dot circle")
      .evaluateAll((nodes) => [...new Set(nodes.map((n) => n.getAttribute("fill")))]);
    expect(fills.sort()).toEqual(["var(--series-formula)", "var(--series-guarantee)"]);
    await expect(card.locator(".legend .sw[data-series=formula]")).toHaveCount(1);
    await expect(card.locator(".legend .sw[data-series=guarantee]")).toHaveCount(1);
  });

  test("the card says how much the picture is hiding", async ({ page }) => {
    // 152 districts are one dot at (20.00, 20.00). A scatter draws the exceptions and hides the
    // rule whenever the rule is a single value, and the counts are what a reader should take.
    await page.goto("/method");
    await expect(page.locator("#reduction-factors")).toContainText("single dot at (20.00, 20.00)");
  });

  test("the poverty measure's ceiling is stated where the limits are", async ({ page }) => {
    // Not a cap this site applies — the source publishes exactly 100% for them and the
    // values below approach it continuously. It is still a ceiling, and the page says so.
    await page.goto("/outcomes");
    const limits = page.locator("#limits");
    await expect(limits).toContainText("has a ceiling");
    await expect(limits).toContainText("not a value this site caps");
  });
});
