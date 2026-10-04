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

/** A bar of the one chart on `/statewide` drawn in a series hue: the guarantee quintiles. */
const GUARANTEE_BAR = '.bar-fill[fill="var(--series-guarantee)"] > *';

test.describe("presentation", () => {
  test("renders in dark mode without losing the series colours", async ({ page }) => {
    // Charts are rendered at build time and cannot re-render on a theme change, so every colour in
    // them is a custom property. This is the test that the indirection actually resolves.
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/statewide");
    // The guarantee quintiles: most of this page's bars mean neither series and are drawn plain
    // (#652), so the test reads the one chart whose bars are a series.
    const fill = await page
      .locator(GUARANTEE_BAR)
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    // #d95926 — the dark-mode step, not the light one.
    expect(fill).toBe("rgb(217, 89, 38)");
  });

  test("the theme toggle beats the OS setting in both directions", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/statewide");
    await page.locator("#theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    const fill = await page
      .locator(GUARANTEE_BAR)
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    expect(fill).toBe("rgb(235, 104, 52)");
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

  /**
   * A district page is what a legislator prints and takes to a meeting, and it printed 19 sheets:
   * the first held the menu, the theme button, five tabs and the contents list and was 60% blank,
   * because every card was kept whole and the first one went to the next sheet; a later sheet held
   * two stranded lines (#602).
   */
  test("paper leaves out the screen's chrome and lets a card break", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    await page.emulateMedia({ media: "print" });
    const shown = await page.evaluate(() =>
      [
        "header.site",
        "nav.subnav",
        "nav.contents",
        "#theme",
        "footer.site .screen-links",
        "footer.site .print-url",
        "input[type=range]",
        "button",
      ].map((selector) => {
        const element = document.querySelector(selector);
        if (!element) return `${selector} is not on the page`;
        return `${selector} ${getComputedStyle(element).display}`;
      }),
    );
    expect(shown).toEqual([
      "header.site none",
      "nav.subnav none",
      "nav.contents none",
      "#theme none",
      "footer.site .screen-links none",
      "footer.site .print-url block",
      "input[type=range] is not on the page",
      "button none",
    ]);
    await expect(page.locator("footer.site .print-url")).toHaveText(
      new RegExp(`/district/${CLEVELAND}$`),
    );
    const breaks = await page.evaluate(() => ({
      card: getComputedStyle(document.querySelector(".card")!).breakInside,
      chart: getComputedStyle(document.querySelector(".chartwrap")!).breakInside,
    }));
    expect(breaks).toEqual({ card: "auto", chart: "avoid" });
  });

  /**
   * The bound is the runner's count, not this machine's. The page is the same and the fonts are
   * not: the fixed stylesheet prints 16 sheets on a Mac and 17 on CI, and the one before #602
   * printed 20 on the same Mac. Print layout is font-sensitive in the way chart layout is, so a
   * lower bound has to be read off a CI log rather than a local run.
   */
  test("a district page prints to at most seventeen sheets", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const pdf = await page.pdf({ format: "Letter" });
    // Each sheet is one `/Type /Page` object; `/Type /Pages` is the tree that holds them.
    const sheets = pdf.toString("latin1").match(/\/Type\s*\/Page(?!s)/g)?.length ?? 0;
    expect(sheets).toBeGreaterThan(0);
    expect(sheets).toBeLessThanOrEqual(17);
  });

  /**
   * A control prints as its position. A switch prints the half that is chosen, as a word, so the
   * chart under it still says which basis it is drawn on; a slider's position is in its label's
   * `<output>`, so the slider itself goes; a select prints the option it holds.
   */
  test("a control prints as the position it is in", async ({ page }) => {
    await page.goto("/statewide");
    await page.emulateMedia({ media: "print" });
    const basis = page.locator(".basis-scope").first();
    await expect(basis.locator('.basis label[data-basis="nominal"]')).toBeVisible();
    await expect(basis.locator('.basis label[data-basis="real"]')).toBeHidden();
    await expect(basis.locator(".basis .n")).toBeHidden();

    await page.goto("/scenario");
    await page.emulateMedia({ media: "print" });
    await expect(page.locator("#lv-base")).toBeHidden();
    await expect(page.locator("#lv-base-out")).toBeVisible();
    await expect(page.locator("#scenario-reset")).toBeHidden();
    const select = await page
      .locator("#lv-guarantee")
      .evaluate((element) => {
        const style = getComputedStyle(element);
        return `${style.display} ${style.appearance} ${style.borderTopWidth}`;
      });
    expect(select).not.toMatch(/^none /);
    expect(select).toMatch(/ none 0px$/);
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
    await expect(chart).toContainText("No change");
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

  test("the local-share chart shows its selection rule and the national figure", async ({ page }) => {
    /*
     * #656. Drawn as the six highest local shares and Ohio, Ohio was the shortest bar and read as
     * the lowest; the rule that explained it was only in the SVG's `<desc>`. The national share is
     * read off the table above and has to be on the chart, and the rule has to be visible text.
     */
    await page.goto("/statewide");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    const national = (await card.locator("tbody tr").first().locator("td").nth(2).innerText()).trim();
    const chart = card.locator('[data-chart="local-share"] svg.plot:visible');
    await expect(chart).toContainText("All 51, combined");
    await expect(chart).toContainText(`${Math.round(parseFloat(national))}%`);
    await expect(chart).toContainText("Ohio (7th of 51)");
    const rule = card.locator('p.note:has(+ [data-chart="local-share"])');
    await expect(rule).toBeVisible();
    await expect(rule).toContainText("the national figure");
    // The ranking is the card's finding, so it leads the first note in bold (#653).
    await expect(card.locator("p.note").first().locator("strong")).toContainText("than 44 states");
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
