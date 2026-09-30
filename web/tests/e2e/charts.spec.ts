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

import { CLEVELAND } from "./helpers.ts";

/*
 * Charts, at the width they are actually read at.
 *
 * Every chart is drawn twice — see `WIDTHS` in `src/lib/plot/spec.ts` — because a static SVG
 * scaled to a phone takes its axis text down with it. The defect these replace: at 375px the
 * 640-unit drawing was scaled by 0.46 and the tick labels rendered near 4.6px, which is half the
 * size at which text is legible, on the viewport most first visits arrive at. Nothing caught it,
 * because a scaled SVG is correct markup and its `font-size` attribute still says 10.
 *
 * So these measure what the browser paints rather than what the file says. A unit test cannot:
 * the scale factor only exists once the SVG is in a box of a known width.
 */
test.describe("charts on a phone", () => {
  const CHARTED = [
    "/",
    "/statewide",
    "/outcomes",
    "/counties",
    "/history",
    "/method",
    "/districts",
    `/district/${CLEVELAND}`,
    `/district/${CLEVELAND}/finances`,
    `/district/${CLEVELAND}/outcome`,
    // The one wiki node that draws a series, and the only wiki route with a chart on it.
    "/wiki/education-agency/toledo-city",
  ];

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

  test("no chart draws its text below 9px", async ({ page }) => {
    // 390px is the common modern phone and 375 the narrowest worth drawing for. The floor is 9px
    // rather than a round 10 because `WIDTHS.narrow` is sized to scale by at least 0.9 there, and
    // the 10px axis marks are the smallest type any of these forms uses.
    await page.setViewportSize({ width: 375, height: 900 });
    const small: string[] = [];
    for (const route of CHARTED) {
      await page.goto(route);
      for (const mark of await paintedText(page)) {
        if (mark.px < 9) small.push(`${route} [${mark.chart}] "${mark.text}" at ${mark.px.toFixed(1)}px`);
      }
    }
    expect(small.slice(0, 10), "chart text a reader cannot read").toEqual([]);
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
    await page.setViewportSize({ width: 375, height: 900 });
    const clipped: string[] = [];
    for (const route of CHARTED) {
      await page.goto(route);
      for (const mark of await paintedText(page)) {
        if (mark.outside > 0.5) {
          clipped.push(`${route} [${mark.chart}] "${mark.text}" over by ${Math.round(mark.outside)}px`);
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
    await page.setViewportSize({ width: 375, height: 900 });
    const through: string[] = [];
    for (const route of CHARTED) {
      await page.goto(route);
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
      for (const hit of hits) through.push(`${route}: ${hit}`);
    }
    expect(through.slice(0, 10), "overlapping chart type").toEqual([]);
  });

  test("the reader is shown one of the two drawings, never both and never neither", async ({
    page,
  }) => {
    // The container query is the only thing choosing between them, so a stylesheet that stopped
    // loading, or a breakpoint edited to leave a gap, would show two charts stacked or none at
    // all — both of which read as a broken page rather than as a missing rule.
    for (const width of [375, 500, 640, 900, 1280]) {
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
  test("the spending charts are banded by poverty, with the legend the ramp obliges", async ({
    page,
  }) => {
    /*
     * A median line says what the middle of a cloud does. Banding says what the cloud is made of,
     * and here that is the card's whole argument: the three poverty thirds sit at the same
     * spending per need-weighted pupil and at different attainment.
     *
     * The legend is not decoration. The ramp's end steps sit near 2.2:1 against their surface,
     * which is a contrast warning that obligates relief rather than one that can be waved off.
     */
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    await expect(card.locator(".legend .sw[data-series=ordinal-1]")).toHaveCount(1);
    await expect(card.locator(".legend .sw[data-series=ordinal-2]")).toHaveCount(1);
    await expect(card.locator(".legend .sw[data-series=ordinal-3]")).toHaveCount(1);

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

  test("a banded chart does not label its traces as well as its legend", async ({ page }) => {
    // Three labelled lines over the densest part of the cloud said the same thing the legend
    // already said, on top of the data it was describing.
    await page.goto("/outcomes");
    const chart = page.locator('[data-chart="weighted-spending"] svg.plot:visible');
    await expect(chart.locator(".scatter-trace")).toHaveCount(3);
    await expect(chart.locator(".scatter-trace-end")).toHaveCount(0);
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
    // Not a cap this repository applies — the source publishes exactly 100% for them and the
    // values below approach it continuously. It is still a ceiling, and the page says so.
    await page.goto("/outcomes");
    const limits = page.locator("#limits");
    await expect(limits).toContainText("has a ceiling");
    await expect(limits).toContainText("not a value this repository caps");
  });
});
