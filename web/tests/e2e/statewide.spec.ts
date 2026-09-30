/**
 * The statewide view, and whether Ohio is unusual.
 *
 * Presentation first — the page a reader lands on has to say what it is measuring before it says
 * how much — then the comparison that gives the level a meaning. The national fixture behind the
 * second block is Ohio plus every state's comparable set rather than every state's whole return —
 * see `renderNational` in `src/lib/statewide.ts`.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND } from "./helpers.ts";

test.describe("presentation", () => {
  test("renders in dark mode without losing the series colours", async ({ page }) => {
    // Charts are rendered at build time and cannot re-render on a theme change, so every colour in
    // them is a custom property. This is the test that the indirection actually resolves.
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/statewide");
    const fill = await page
      .locator(".bar-fill")
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    // #3987e5 — the dark-mode step, not the light one.
    expect(fill).toBe("rgb(57, 135, 229)");
  });

  test("the theme toggle beats the OS setting in both directions", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/statewide");
    await page.locator("#theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    const fill = await page
      .locator(".bar-fill")
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    expect(fill).toBe("rgb(42, 120, 214)");
  });

  /**
   * Paper has no theme, and this is the test that the cascade agrees.
   *
   * `data-theme="dark"` is an explicit stamp that no medium unsets, and `print` matches at the
   * same time as `prefers-color-scheme: dark` when a dark-mode reader prints — so both dark
   * palettes are live when the print block has to win. Whether it does is a question about
   * specificity and source order in a real engine, which is the one thing the stylesheet unit
   * tests cannot answer. Before the print block existed this printed #ffffff ink on white paper.
   */
  test("a dark-theme reader prints in the light palette", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/statewide");
    // Twice: the first click leaves the dark OS setting for light, the second stamps dark back on
    // explicitly. That is the case the print block has to beat — a `[data-theme="dark"]` attribute
    // AND the dark media query, both live at once.
    await page.locator("#theme").click();
    await page.locator("#theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

    await page.emulateMedia({ media: "print" });
    const paper = await page.evaluate(() => {
      const style = getComputedStyle(document.body);
      return { ink: style.color, ground: style.backgroundColor };
    });
    // #0b0b0b on #f4f4f2 — the light block, not dark's #ffffff on #111110.
    expect(paper.ink).toBe("rgb(11, 11, 11)");
    expect(paper.ground).toBe("rgb(244, 244, 242)");
  });

  /**
   * Browsers omit backgrounds from print, so a mark drawn with one prints blank. The stacked bar,
   * the legend swatches and the regime spans were all background-only; each carries a border on
   * paper now, and the border's STYLE is what tells the two series apart once the colour is gone.
   */
  test("marks drawn with a ground carry a second channel on paper", async ({ page }) => {
    // The stacked bar and its legend live on a district's aid card.
    await page.goto(`/district/${CLEVELAND}`);
    await page.emulateMedia({ media: "print" });
    const styles = await page.evaluate(() =>
      [
        ".seg[data-series=formula]",
        ".seg[data-series=guarantee]",
        ".sw[data-series=formula]",
        ".sw[data-series=guarantee]",
      ].map((selector) => {
        const element = document.querySelector(selector);
        if (!element) return `${selector} is not on the page`;
        const { borderStyle, borderWidth } = getComputedStyle(element);
        return `${selector} ${borderStyle} ${borderWidth}`;
      }),
    );
    expect(styles).toEqual([
      ".seg[data-series=formula] solid 1px",
      ".seg[data-series=guarantee] dashed 1px",
      ".sw[data-series=formula] solid 1px",
      ".sw[data-series=guarantee] dashed 1px",
    ]);
  });

  test("the page does not scroll sideways on a phone", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    for (const path of [
      "/",
      "/statewide",
      `/district/${CLEVELAND}`,
      "/districts",
      "/wiki/metric/performance-index",
    ]) {
      await page.goto(path);
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow, `${path} scrolls sideways`).toBeLessThanOrEqual(1);
    }
  });

  test("the hover layer follows the marks", async ({ page }) => {
    await page.goto("/statewide");
    const tip = page.locator("#tip");
    await expect(tip).toBeHidden();
    await page.locator("svg.plot:visible .bar-fill > *").first().hover();
    await expect(tip).toBeVisible();
    await expect(tip).toContainText("districts on the guarantee");
  });

  test("the diverging histogram marks where zero is", async ({ page }) => {
    // A reader has to be able to see where zero is, not infer it from the hues.
    // Base cost up 10% with the guarantee removed: some districts gain and some lose, so the
    // distribution straddles zero and the neutral midpoint has to be findable.
    await page.goto("/scenario?g=removed&arg=0.5&base=1.1&min=0.1&pb=1&pc=1&h=2026");
    const chart = page.locator('[data-chart="deltas"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart).toContainText("no change");
    // Two hues and a neutral midpoint, never a hue at zero.
    await expect(chart.locator(".hist > *")).not.toHaveCount(0);
  });
});

test.describe("whether Ohio is unusual", () => {
  test("the front page ranks Ohio against the other states", async ({ page }) => {
    /*
     * The only comparative claim on the site, and the only federal source behind it. Everything
     * else here is Ohio describing itself, which cannot answer a question of the form "too
     * heavily" — the form the DeRolph holding takes.
     */
    await page.goto("/statewide");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    await expect(card).toContainText("Local share of school revenue");
    await expect(card).toContainText("7 of 51");
    await expect(card).toContainText("45 of 51");
    await expect(card).toContainText("DeRolph");
  });

  test("the property tax rank names the states it excludes and why", async ({ page }) => {
    /*
     * Nine states report zero school property tax and levy plenty of it — their districts are
     * agencies of a city or county, so the survey attributes the tax to the parent. Ranking all
     * fifty-one would put Massachusetts and Virginia at the bottom of a measure they are near the
     * top of, and the first version of this extractor did exactly that.
     */
    await page.goto("/statewide");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    await expect(card).toContainText("39 states whose districts levy their own tax");
    await expect(card).toContainText("Massachusetts and Virginia report");
    await expect(card).toContainText("excludes twelve states on purpose");
  });

  test("the relief year is named as cutting against the finding", async ({ page }) => {
    await page.goto("/statewide");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    await expect(card).toContainText("peak year of federal pandemic relief");
    await expect(card).toContainText("against this finding rather than for it");
  });

  test("the Census comparison stays out of the scenario panel", async ({ request }) => {
    // 51 states is small, but the formula never reads it and the panel is a formula input.
    const panel = await (await request.get("/data/panel.json")).json();
    expect(panel).not.toHaveProperty("national");
    const feed = await (await request.get("/data/bundle.json")).json();
    expect(feed.national.states).toHaveLength(51);
  });
});
