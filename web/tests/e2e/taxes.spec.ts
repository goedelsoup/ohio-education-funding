/**
 * The property tax routes: what a district can raise, and what it actually charged.
 *
 * Three blocks about local capacity. The floor in the middle of it is a `max` rather than a cliff,
 * and the third block is why that matters — voted millage and charged millage are two numbers, and
 * the reduction factors are the wedge between them.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND, NORTHERN, AT_FLOOR, BELOW_FLOOR } from "./helpers.ts";

test.describe("the property tax route", () => {
  test("a district at the floor is told the reduction factors have stopped", async ({ page }) => {
    // Fremont City charges 20.0000 mills in both tax years, so H.B. 920 has nothing left to roll
    // back and a reappraisal reaches its revenue directly. That is the point of carrying two.
    await page.goto(`/district/${AT_FLOOR}/taxes`);
    await expect(page.locator("h1")).toHaveText("Fremont City");
    await expect(page.locator(`.subnav a[aria-current="page"]`)).toHaveText("Property tax");
    const change = page.locator('[data-part="valuation-change"]');
    await expect(change).toContainText("at the");
    await expect(change).toContainText("reduction factors have stopped operating");
    await expect(change).toContainText("A reappraisal is a revenue event here");
  });

  test("a district a hundredth of a mill above the floor is not told the factors are operative", async ({
    page,
  }) => {
    /*
     * Northern Local is 20.0154 mills — one of the 82 districts that crossed 20.0000 between the
     * two tax years. Both of the copy's original branches were wrong for it: "reduction factors
     * have stopped" overstates, and "fully operative" is absurd for a rate a hundredth of a mill
     * clear of the floor. The third branch exists because this district does.
     */
    await page.goto(`/district/${NORTHERN}/taxes`);
    const change = page.locator('[data-part="valuation-change"]');
    await expect(change).toContainText("above the");
    await expect(change).toContainText("close enough that the distinction carries little meaning");
    await expect(change).not.toContainText("fully operative");
    await expect(change).not.toContainText("have stopped operating");
  });

  test("a district under twenty mills is not told it is above the floor", async ({ page }) => {
    /*
     * The bug contract 9.0.0 exists for. Vinton County charges 18.70 mills, and comparing that to
     * a literal 20.0 for equality put it on the wrong side of the floor entirely — reported as
     * having reduction factors operative when it has never been subject to one.
     */
    await page.goto(`/district/${BELOW_FLOOR}/taxes`);
    const change = page.locator('[data-part="valuation-change"]');
    await expect(change).toContainText("charges less than twenty mills");
    await expect(change).toContainText("never have");
    await expect(change).not.toContainText("reduction factors are fully operative");

    // And the millage card says the same thing from the other direction: nothing was taken.
    const millage = page.locator('[data-part="millage"]');
    await expect(millage).toContainText("18.70");
    await expect(millage).toContainText("Reduction factors have taken nothing");
  });

  test("the millage card shows the calculator's prediction and names the residual", async ({
    page,
  }) => {
    /*
     * The distinguishing feature of this card is that its numbers are computed rather than
     * copied. Columbus voted 79.68 mills and charges 25.97 — a figure no published column states,
     * because it takes both departments' tables to produce it.
     */
    await page.goto(`/district/${CLEVELAND}/taxes`);
    const millage = page.locator('[data-part="millage"]');
    await expect(millage).toContainText("Voted current operating millage");
    await expect(millage).toContainText("Taken by reduction factors");
    await expect(millage).toContainText("What the factors alone predict");
    await expect(millage).toContainText("Residual");
    await expect(millage).toContainText("What one mill raises here");

    // The three-row prediction has to close: observed minus predicted is the residual shown.
    const rows = millage.locator("tbody tr");
    const cell = async (i: number) =>
      Number((await rows.nth(i).locator("td").first().innerText()).replace(/[^0-9.-]/g, ""));
    const [predicted, observed, residual] = [await cell(1), await cell(2), await cell(3)];
    expect(Math.abs(observed - predicted - residual)).toBeLessThan(0.001);
  });

  test("the charge-off counterfactual is labelled as one and names what it cannot reach", async ({
    page,
  }) => {
    /*
     * The only counterfactual on the site, so the framing carries more weight than the numbers.
     * Columbus is the useful case: 23 mills against its valuation exceeds its whole base cost,
     * so the charge-off would leave it nothing — the plan's minimum state share is what does not
     * exist in the earlier mechanism.
     */
    await page.goto("/district/043802/taxes");
    const card = page.locator('[data-part="charge-off"]');
    await expect(card).toContainText("23 mills");
    await expect(card).toContainText("no minimum state share to stop at");
    await expect(card).toContainText("counterfactual at FY2027 inputs");
    await expect(card).toContainText("not a reconstruction of any year the charge-off governed");
    await expect(card).toContainText("one row of the calculation");
    // The correction: the page used to say recognised valuation was an H.B. 920 adjustment this
    // project did not hold, and that every figure therefore overstated the charge-off. It is
    // neither — it is a reappraisal phase-in, and it is now computed. Asserting the corrected
    // wording rather than deleting the check, so the page cannot regress to the old claim.
    await expect(card).toContainText("because this page said something wrong");
    await expect(card).toContainText("is not an H.B. 920 adjustment");
    await expect(card).toContainText("staggered county calendar");
    await expect(card).toContainText("Franklin County revalued in 2023");
  });

  test("a district below the charge-off rate is told it would be charged for phantom revenue", async ({
    page,
  }) => {
    // Vinton County charges 18.70 mills against a mechanism that assumes 23 — the failure the
    // charge-off was replaced for, and half the state is in the same position.
    await page.goto(`/district/${BELOW_FLOOR}/taxes`);
    const card = page.locator('[data-part="charge-off"]');
    await expect(card).toContainText("would be charged for revenue it could not raise");
    await expect(card).toContainText("4.30 mills short");
    await expect(card).toContainText("gap aid");
  });

  test("a district whose two pupil counts diverge is shown both", async ({ page }) => {
    /*
     * The defect this catches is the one the site shipped: printing Table SD-1's valuation per
     * pupil against a statewide median computed on the profile report's denominator. The card
     * only appears where the two are more than 5% apart, and Columbus is 1.7x.
     */
    await page.goto("/district/043802/taxes");
    const card = page.locator('[data-part="denominators"]');
    await expect(card).toContainText("The valuations are the same to the dollar");
    await expect(card).toContainText("73,746");
    await expect(card).toContainText("43,019");
    await expect(card).toContainText("Education, base cost ADM");

    // And the tile above compares against a median on the same basis as the figure it shows.
    await expect(page.locator(".tile", { hasText: "Taxable value per pupil" })).toContainText(
      "on this table's pupil count",
    );
  });

  test("a district whose pupil counts agree is not shown the caution", async ({ page }) => {
    // Under 5% apart, so the card would be noise rather than a caution. Only 78 of 609 districts
    // are within 2%; Bexley City is one, and most of the state is not — which is the finding.
    await page.goto("/district/043620/taxes");
    await expect(page.locator("h1")).toHaveText("Bexley City");
    await expect(page.locator('[data-part="denominators"]')).toHaveCount(0);
  });

  test("a district above the floor is told they are operative", async ({ page }) => {
    await page.goto("/district/044933/taxes");
    const change = page.locator('[data-part="valuation-change"]');
    await expect(change).toContainText("above the");
    await expect(change).toContainText("reduction factors are fully operative");
  });

  test("the base is broken into the classes that are reduced separately", async ({ page }) => {
    await page.goto(`/district/${NORTHERN}/taxes`);
    const base = page.locator('[data-part="tax-base"]');
    for (const label of ["Residential", "Agricultural", "Commercial", "Public utility"]) {
      await expect(base.locator('[data-chart="tax-base"]')).toContainText(label);
    }
  });

  test("names the department it came from, which is not the usual one", async ({ page }) => {
    // Every other page reads the Department of Education. This one reads Taxation, and says so —
    // along with the fact that where the two overlap they agree.
    await page.goto(`/district/${NORTHERN}/taxes`);
    const source = page.locator('[data-part="not"]');
    await expect(source).toContainText("Ohio Department of Taxation");
    await expect(source).toContainText("to 0.01 mills");
  });
});

test.describe("the local capacity measure", () => {
  test("the method page credits it as computed and verified exactly", async ({ page }) => {
    /*
     * The strongest verification claim this project can make — a statutory formula reproduced
     * against the department's own answer for every district — and it sat in a test file, invisible
     * to any reader, for three commits.
     */
    await page.goto("/method");
    const row = page.locator("tr", { hasText: "The local capacity measure" }).first();
    await expect(row).toContainText("all 609 districts");
    await expect(row).toContainText("hundredth of a percent");
    // And the route it was got wrong by, which is the part worth keeping.
    await expect(row).toContainText("4.4% light");
  });

  test("a district at the minimum state share still shows a capacity figure", async ({ page }) => {
    // These 138 were censored until the published figure was found: recovering capacity by
    // subtracting aid from base cost cannot reach a district whose share is set by the floor.
    // Bay Village is at the minimum state share, so the subtraction genuinely could not reach it.
    await page.goto("/district/043547/taxes");
    const card = page.locator('[data-part="charge-off"]');
    await expect(card).toContainText("Local capacity, the plan");
    await expect(card).not.toContainText("Not recoverable");
    await expect(card).toContainText("reproduces it");
  });
});

test.describe("the reduction factors, against what was charged", () => {
  test("the model is drawn against the record it is a model of", async ({ page }) => {
    /*
     * `/method` had four tables and no chart, on a page whose subject is which figures are models
     * and which are records. This one draws both: mills `crates/millage` predicts against mills a
     * county auditor charged, with the line where they agree.
     */
    await page.goto("/method");
    const chart = page.locator('#reduction-factors [data-chart="reduction-factors"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart.locator(".scatter-identity")).toHaveCount(1);

    // One mark per district that has a millage record, each pointable.
    const dots = chart.locator(".scatter-dot circle");
    expect(await dots.count()).toBeGreaterThan(500);
    expect(await chart.locator(".scatter-hit circle[data-hover]").count()).toBe(await dots.count());
  });

  test("the identity line is drawn at 45 degrees, not at whatever the frame allows", async ({
    page,
  }) => {
    /*
     * The reading this card offers is "distance from the line is the residual, in mills". That is
     * only true if the plot area is square: on the default 640×420 frame a shared domain still
     * draws y = x at about 33°, which reads as a trend the cloud is beating. Measured off the
     * rendered geometry rather than trusted from the spec.
     */
    await page.goto("/method");
    const box = await page
      .locator('#reduction-factors [data-chart="reduction-factors"] svg.plot:visible .scatter-identity path')
      .evaluate((n) => {
        const r = (n as SVGGraphicsElement).getBBox();
        return { w: r.width, h: r.height };
      });
    expect(box.w).toBeGreaterThan(100);
    expect(Math.abs(box.h / box.w - 1), "the identity line is at 45°").toBeLessThan(0.02);
  });

  test("the card counts the districts rather than asserting a number", async ({ page }) => {
    // "182 of 273" is the kind of sentence that is true when written and wrong two bundles later
    // with nothing to notice, so the counts are read off the feed. This checks they agree with it.
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const withMillage = feed.districts.filter((d: any) => d.millage != null);
    const atFloor = withMillage.filter((d: any) => d.millage.at_floor);
    const exact = atFloor.filter((d: any) => Math.abs(d.millage.residual) < 0.01).length;

    await page.goto("/method");
    const card = page.locator("#reduction-factors");
    await expect(card).toContainText(`${exact} of the ${atFloor.length} districts`);
  });
});
