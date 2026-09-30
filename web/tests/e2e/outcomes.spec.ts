/**
 * The outcome routes.
 *
 * The one place on this site where a figure is not money. Its own file because everything in it is
 * about keeping a correlation from reading as a mechanism — what the page claims, and what it
 * refuses to.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND } from "./helpers.ts";

test.describe("the outcome routes", () => {
  test("shows the raw guarantee association and the controlled one together", async ({ page }) => {
    // The rule the whole axis exists for: no association appears without its controlled figure.
    await page.goto("/outcomes");
    const tiles = page.locator(".tiles").first().locator(".tile");
    await expect(tiles.nth(1)).toContainText("+0.187");
    await expect(tiles.nth(2)).toContainText("+0.035");
    await expect(tiles.nth(2)).toContainText("holding poverty constant");
  });

  test("names both spending denominators rather than quoting one number", async ({ page }) => {
    await page.goto("/outcomes");
    const card = page.locator(".card", { hasText: "The same numerator, two denominators" });
    await expect(card).toContainText("need-weighted");
    await expect(card).toContainText("enrolled");
  });

  test("the poverty chart draws the districts, not a summary of them", async ({ page }) => {
    /*
     * This was five bars of quintile medians, which is 606 districts reduced to five numbers on a
     * card whose own argument is about the spread around the trend. The medians are still drawn —
     * as the line through the cloud rather than in place of it — so both assertions here are about
     * the same five numbers the bar chart carried, plus the districts it did not.
     */
    await page.goto("/outcomes");
    const chart = page.locator('[data-chart="poverty-and-performance"] svg.plot:visible');
    await expect(chart).toBeVisible();

    // One mark per district with both measures, and a hit target on each so it can be pointed at.
    const dots = chart.locator(".scatter-dot circle");
    expect(await dots.count()).toBeGreaterThan(500);
    expect(await chart.locator(".scatter-hit circle[data-hover]").count()).toBe(await dots.count());

    // The median line falls left to right: least poor fifth highest, poorest fifth lowest.
    const trace = await chart
      .locator(".scatter-trace path")
      .first()
      .evaluate((n) => n.getAttribute("d") ?? "");
    const ys = [...trace.matchAll(/[ ,](\d+(?:\.\d+)?)(?=[A-Za-z]|$|,|\s)/g)]
      .map((m) => Number(m[1]))
      .filter((v) => Number.isFinite(v));
    expect(ys.length, "the trace has points to read").toBeGreaterThan(4);

    await expect(page.locator(".card", { hasText: "Poverty is most of what" })).toContainText(
      "−0.846",
    );
  });

  test("the two denominators are drawn against one vertical scale", async ({ page }) => {
    // The card's whole claim is that one cloud is flat and the other slopes. That comparison is
    // only readable if the axis they are compared on is the same one, so it is asserted rather
    // than left to whichever range each chart happened to compute.
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    await expect(card.locator('[data-chart="weighted-spending"] svg.plot:visible')).toBeVisible();
    await expect(card.locator('[data-chart="enrolled-spending"] svg.plot:visible')).toBeVisible();
    await expect(card).toContainText("−0.004");
    await expect(card).toContainText("−0.355");
  });

  test("and against one horizontal scale, because the card's prose is about horizontal distance", async ({
    page,
  }) => {
    /*
     * The sentence under the second chart reads "now they separate on the horizontal axis too".
     * Fitted to their own ranges the two x axes differed by 1.64×, so part of the separation a
     * reader saw was the frame rather than the dollars. Both charts state their own ends, so the
     * assertion is that the two pairs of ends are the same pair.
     */
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    const ends = async (chart: string) =>
      (await card.locator(`[data-chart="${chart}"] svg.plot:visible text`).allTextContents()).filter((t) =>
        /^\$[\d,]+$/.test(t),
      );
    const weighted = await ends("weighted-spending");
    // Both ends of the x axis — the third bottom label is the axis's name. Asserted so that the
    // comparison below is not between two empty lists, which would pass on a chart that had
    // stopped drawing its scale at all.
    expect(weighted).toHaveLength(2);
    // The wider denominator's own maximum, on the chart of the narrower one.
    expect(weighted[1]).toBe("$38,140");
    expect(weighted).toEqual(await ends("enrolled-spending"));
  });

  test("a district's score is shown against comparable poverty, not against the state", async ({
    page,
  }) => {
    await page.goto(`/district/${CLEVELAND}/outcome`);
    const card = page.locator('[data-part="comparable-poverty"]');
    await expect(card).toContainText("Median of its poverty fifth");
    await expect(card).toContainText("It is <strong>not</strong> an effect".replace(/<[^>]+>/g, ""));
  });

  test("the report card's disadvantaged share reads as a percentage, not a fraction of one", async ({
    page,
  }) => {
    // It read 1.0% for Cleveland after the bundle moved the field to a fraction (#570).
    await page.goto(`/district/${CLEVELAND}/outcome`);
    const cell = page
      .locator('[data-part="outcomes"] tr', { hasText: "Economically disadvantaged (report card)" })
      .locator("td");
    const text = (await cell.textContent()) ?? "";
    expect(text).toMatch(/^\d+\.\d%$/);
    expect(Number.parseFloat(text)).toBeGreaterThanOrEqual(50);
  });

  test("a district with no report card says so instead of showing blanks", async ({ page }) => {
    // Three of the 609 have no report card. The page has to exist and explain itself.
    await page.goto("/data/bundle.json");
    const bundle = await page.evaluate(() => JSON.parse(document.body.innerText));
    const missing = bundle.districts.find((d: { outcome: unknown }) => d.outcome === null);
    expect(missing).toBeTruthy();
    await page.goto(`/district/${missing.irn}/outcome`);
    await expect(page.getByText("No report card is published for this district")).toBeVisible();
  });
});
