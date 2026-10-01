/**
 * Districts aggregated to a boundary: counties, senate districts, house districts.
 *
 * Four blocks, one apportionment. A school district crosses a legislative line, so every figure
 * here is a share of a district's value rather than the whole of it — rank on value times share, or
 * a 0.3% district tops the table. The last block is the same page read the other way: a county as a
 * peer group rather than as a jurisdiction.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

test.describe("counties", () => {
  test("the index ranks counties by the disparity inside them", async ({ page }) => {
    /*
     * The ordering is the argument. Alphabetical would make this a directory; ordered by how far
     * apart the richest and poorest district in each county are, it puts the finding first.
     */
    await page.goto("/counties");
    await expect(page.locator("h1")).toHaveText("Counties");

    const ratios = await page
      .locator("tbody tr td:nth-child(4)")
      .allTextContents();
    const numeric = ratios
      .map((t) => Number.parseFloat(t.replace("×", "")))
      .filter((n) => Number.isFinite(n));
    expect(numeric.length).toBeGreaterThan(80);
    for (let i = 1; i < numeric.length; i += 1) {
      expect(numeric[i]!).toBeLessThanOrEqual(numeric[i - 1]!);
    }
  });

  test("a county page names its widest pair and links both districts", async ({ page }) => {
    await page.goto("/county/cuyahoga");
    await expect(page.locator("h1")).toHaveText("Cuyahoga County");
    const spread = page.locator('.card[data-part="spread"]');
    await expect(spread).toContainText("Orange City");
    await expect(spread).toContainText("Maple Heights City");
    await expect(spread).toContainText("times");
    // Both extremes must be reachable, since the point of the page is to send a reader onward. The
    // two name links, plus — deliberately, from this phase — one into the tax base section of the
    // poorer district's property tax page, which is the figure this card is actually arguing about
    // and which it never linked.
    await expect(spread.locator('a[href^="/district/"]')).toHaveCount(3);
    await expect(spread.locator('a[href$="/taxes#tax-base"]')).toHaveCount(1);
  });

  test("a county page says its pupil total is a sum of districts, not the county's children", async ({
    page,
  }) => {
    /*
     * The honesty constraint the whole feature rests on. Ohio school district boundaries cross
     * county lines, so a county page that reports a total without saying what it is a total *of*
     * is making a geographic claim the data does not support.
     */
    await page.goto("/county/cuyahoga");
    await expect(page.locator('.card[data-part="roster"]')).toContainText(
      "a sum of these districts rather than a count of the county's children",
    );
  });

  test("a single-district county says there is nothing to compare", async ({ page }) => {
    // Rather than printing a ratio of 1.0, which would read as "no disparity here" — a different
    // claim from "this county has one district".
    await page.goto("/counties");
    const row = page.locator("tbody tr").filter({ hasText: "a single district" }).first();
    await expect(row).toBeVisible();
    const href = await row.locator("a").first().getAttribute("href");
    await page.goto(href!);
    await expect(page.locator('.card[data-part="spread"]')).toContainText(
      "no internal disparity to measure",
    );
  });

  test("every district page's county is reachable from the counties index", async ({ page }) => {
    // The link direction that matters: a reader arrives at a district and wants its neighbours.
    await page.goto("/counties");
    await expect(page.locator('a[href^="/county/"]')).toHaveCount(88);
  });
});

test.describe("senate districts", () => {
  test("all thirty-three seats are listed and the page says it is the tighter view", async ({
    page,
  }) => {
    /*
     * Senate seats are three times larger than House seats, so 392 of 609 school districts lie
     * wholly inside one against 270 for the House. Two pages built from one template are not
     * equally reliable, and the page says which is which rather than leaving a reader to infer it.
     */
    await page.goto("/senate");
    await expect(page.locator("h1")).toHaveText("Senate districts");
    await expect(page.locator('a[href^="/senate/"]')).toHaveCount(33);
    await expect(page.locator(".card").first()).toContainText("less approximate");
  });

  test("the two chambers apportion the same total", async ({ page }) => {
    // Both partition the state, so the rows on each index must add to the same figure. If they
    // did not, one of the two crosswalks would be losing districts.
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const sum = (xs: { realized_aid: number }[]) => xs.reduce((a, x) => a + x.realized_aid, 0);
    expect(Math.abs(sum(feed.house_districts) - sum(feed.senate_districts))).toBeLessThan(1);
    expect(feed.senate_districts).toHaveLength(33);
  });

  test("a district page links both chambers", async ({ page }) => {
    // Columbus spans eleven House seats and four Senate seats.
    await page.goto("/district/043802");
    // `[data-part]` rather than `p.sub`: #188 made this a labelled `<dl>`, and a test that names
    // the element is a test that breaks when the markup is improved. Addressed by part, like the
    // cards are, for the reason the card-locator note above gives.
    const sub = page.locator('[data-part="identity"]').first();
    await expect(sub.locator('a[href^="/house/"]')).toHaveCount(11);
    await expect(sub.locator('a[href^="/senate/"]')).toHaveCount(4);
  });
});

test.describe("house districts", () => {
  test("the index says plainly that its figures are estimates", async ({ page }) => {
    /*
     * The constraint the whole feature rests on. No House district is a unit of account in Ohio's
     * funding system, so every figure here is derived by splitting school districts across census
     * blocks. A page that shows "$43,707,982" without saying so is passing off a derivation as a
     * published fact, and the precision makes it worse rather than better.
     */
    await page.goto("/house");
    await expect(page.locator("h1")).toHaveText("House districts");
    await expect(page.locator(".card").first()).toContainText(
      "These figures are estimates, and nobody publishes them",
    );
    await expect(page.locator(".card").first()).toContainText("under-18 population");
    await expect(page.locator(".card").first()).toContainText("2020 census");
  });

  test("all ninety-nine seats are listed and reachable", async ({ page }) => {
    await page.goto("/house");
    await expect(page.locator('a[href^="/house/"]')).toHaveCount(99);
  });

  test("the seat's valuation strip can be read off, in the scale row and in the table", async ({
    page,
  }) => {
    /*
     * 129 seat pages drew a valuation strip and gave a reader no way to recover a value from it:
     * `distributionSpec` draws no axis by construction, and this was the one page carrying it with
     * neither the `.scale` row every other strip on the site has nor a valuation column in the
     * table underneath. A dot four fifths along meant nothing at all.
     *
     * Both, and not either: the scale row states the ends, and the column is what lets a reader go
     * from a dot to the district it belongs to.
     */
    await page.goto("/house/065");
    const card = page.locator('[data-part="members"]');
    await expect(card.locator('[data-chart="seat-spread"] svg.plot:visible')).toBeVisible();
    const scale = card.locator(".scale").first();
    await expect(scale.locator("span")).toHaveCount(2);
    await expect(scale.locator("span").first()).toContainText("$");
    await expect(card.locator("thead")).toContainText("Valuation per pupil");
  });

  test("a seat page distinguishes the two shares it prints side by side", async ({ page }) => {
    /*
     * "Of the district" and "of this seat" answer opposite questions, and a swap would be
     * invisible: 100% in the wrong column reads as "the member speaks for all of it" when it means
     * "this seat is entirely that district".
     */
    await page.goto("/house/054");
    const members = page.locator('.card[data-part="members"]');
    await expect(members).toContainText("Of the district");
    await expect(members).toContainText("Of this seat");
    await expect(members).toContainText("answer opposite questions");
  });

  test("a district page links every seat it lies in", async ({ page }) => {
    // Columbus spans eleven. The reverse direction is the one a reader arriving from a legislative
    // page is asking about, so it has to be complete rather than a count.
    await page.goto("/district/043802");
    const sub = page.locator('[data-part="identity"]').first();
    await expect(sub.locator('a[href^="/house/"]')).toHaveCount(11);
  });

  test("the seat totals a reader can add up reconcile to the statewide figure", async ({
    page,
  }) => {
    // The one accuracy claim the apportionment makes, checked as rendered rather than as data:
    // the index states the statewide total, and it must be the sum of the rows above it.
    await page.goto("/house");
    const cells = await page.locator("tbody tr td:nth-child(2)").allTextContents();
    const rows = cells.map((t) => Number(t.replace(/[$,]/g, ""))).filter(Number.isFinite);
    expect(rows).toHaveLength(99);
    const summed = rows.reduce((a, b) => a + b, 0);
    const stated = await page.locator(".card").first().textContent();
    const match = stated?.match(/these\s+99\s+rows\s+add\s+to\s+\$([\d,]+)/);
    expect(match, "the index states the statewide total").not.toBeNull();
    const declared = Number(match![1]!.replace(/,/g, ""));
    // Each row is rendered to the dollar, so ninety-nine roundings can drift by that much.
    expect(Math.abs(summed - declared)).toBeLessThan(100);
  });
});

test.describe("counties, which are a peer group and not a boundary", () => {
  test("the index draws what each county spans, not only the ratio", async ({ page }) => {
    /*
     * The table ranks 88 counties by richest ÷ poorest and prints the ratio. A ratio is one number
     * standing for two and the two are not recoverable from it — two counties at the same ratio
     * can have non-overlapping wealth.
     *
     * Deliberately not a map. The card above this one says a county here is a peer group and not a
     * boundary, because district lines cross county lines freely; a choropleth would assert the
     * geography the page spends its first card denying, and the department's one-county-per-
     * district attribution could not honestly draw the crossing anyway.
     */
    await page.goto("/counties");
    const chart = page.locator('#disparity [data-chart="county-disparity"] svg.plot:visible');
    await expect(chart).toBeVisible();

    // One row per county with two districts to compare; four of the 88 have only one.
    const spans = chart.locator(".range-span line, .range-span path");
    expect(await spans.count()).toBeGreaterThan(70);
    expect(await chart.locator(".range-low circle").count()).toBe(await spans.count());
    expect(await chart.locator(".range-high circle").count()).toBe(await spans.count());

    /*
     * The ends are two shades of one hue: a low end and a high end are one measure at two points,
     * not two series.
     *
     * Read off the mark *group* rather than the circles. Plot puts a constant channel on the `<g>`
     * and a computed one on each mark, so the banded scatters — whose fill is a function of the
     * point — carry it per circle and this one does not.
     */
    const fills = await chart
      .locator(".range-low, .range-high")
      .evaluateAll((n) => n.map((g) => g.getAttribute("fill")));
    expect(fills.sort()).toEqual(["var(--ordinal-2)", "var(--ordinal-3)"]);

    await expect(page.locator("#disparity .legend .sw[data-series=ordinal-2]")).toHaveCount(1);
    await expect(page.locator("#disparity .legend .sw[data-series=ordinal-3]")).toHaveCount(1);
  });

  test("the rows are ordered the way the table beside them is", async ({ page }) => {
    // The chart and the table are the same data at two resolutions, and a reader moving between
    // them should not have to re-find their county.
    await page.goto("/counties");
    // `textContent`, not `allInnerTexts`: `innerText` is empty for an SVG `<text>`, so the
    // convenient helper silently compares eighty-four empty strings against the table.
    const rows = await page
      .locator('#disparity [data-chart="county-disparity"] svg.plot:visible .range-label text')
      .evaluateAll((nodes) => nodes.map((n) => n.textContent ?? ""));
    const table = await page.locator("#roster tbody tr th a").allInnerTexts();
    expect(rows.length).toBeGreaterThan(70);
    // Every charted county is in the table, in the same relative order.
    const positions = rows.map((r) => table.indexOf(r));
    expect(positions.every((p) => p >= 0), "a charted county missing from the table").toBe(true);
    expect([...positions].sort((a, b) => a - b)).toEqual(positions);
  });

  test("a county page says whether its own disparity is a large one", async ({ page }) => {
    // The card stated the ratio and never said what it was a lot of. The median county is 2.1x
    // and Cuyahoga is 5.5x, which the page had no way of conveying.
    await page.goto("/county/cuyahoga");
    const chart = page.locator('#spread [data-chart="county-position"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart.locator(".dist-marker")).toHaveCount(1);
    expect(await chart.locator(".dist-dot circle").count()).toBeGreaterThan(70);
    // "The median" and not "The median county": the figure is `stats.median`, which interpolates,
    // so on an even count of counties it is a ratio none of them has. See `src/lib/stats.ts`.
    await expect(page.locator("#spread")).toContainText("The median is");
  });

  test("a single-district county gets neither chart, and says why", async ({ page }) => {
    // A ratio needs two districts. Four counties have one, and their absence from the comparison
    // is a real answer rather than a gap to fill.
    await page.goto("/county/vinton");
    await expect(page.locator('[data-chart="county-position"]')).toHaveCount(0);
    await expect(page.locator('[data-chart="county-spread"]')).toHaveCount(0);
    await expect(page.locator("#spread")).toContainText("no internal disparity to measure");
  });
});
