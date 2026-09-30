/**
 * Districts the formula does not reach, and the machinery that holds them.
 *
 * The guarantee is the one mechanism on this site whose population is not its reach — 138 districts
 * sit on the minimum share and only 31 of them move — so its page has to distinguish being held
 * from being paid. These two blocks are that distinction, on the page and in the arithmetic shown
 * beside it.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

test.describe("outside the formula", () => {
  test("the card says plainly that the guarantee does not hold these", async ({ page }) => {
    /*
     * The structural point. Everything above this card is `[H] Foundation Funding`, which the
     * guarantee protects; these sit in `[R] Total State Support` and nothing cushions a fall in
     * them. A district that drops a star loses the money outright.
     */
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("Outside the formula");
    await expect(card).toContainText("the guarantee does not hold a district at them");
    await expect(card).toContainText("Base funding supplement");
    await expect(card).toContainText("$40 a pupil");
  });

  test("the performance supplement names the rating it was paid on", async ({ page }) => {
    // Cleveland is rated 2.5 stars and has a 4.0 progress rating, and is paid on the greater of
    // the two — the progress route working exactly as designed for a high-poverty district.
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("the greater of its");
    await expect(card).toContainText("progress rating");
  });

  test("the page states the gradient rather than only the district's own figure", async ({
    page,
  }) => {
    /*
     * The finding has to appear on every district's page, not only where it flatters. A component
     * distributed inversely to need is a fact about the program, and a reader looking at a
     * well-funded district should meet it there too.
     */
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("$54.74");
    await expect(card).toContainText("$23.31");
    await expect(card).toContainText("track intake");
  });

  test("transportation names which of the two bases the district is paid on", async ({ page }) => {
    /*
     * The `MAX` flips for more than half the state and is invisible in the amount. A district paid
     * on miles gains nothing from carrying more children on the same routes; one paid on riders
     * gains nothing from covering more ground. The card says which.
     */
    await page.goto("/district/000442");
    const table = page.locator('[data-part="supplements"] table[data-program="transportation"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("Per-rider base");
    await expect(table).toContainText("Per-mile base");
    await expect(table).toContainText("The greater of the two");
  });

  test("the proration is shown as a shortfall, not as a rate", async ({ page }) => {
    /*
     * A factor below one means the appropriation did not cover the computed entitlement. The
     * published amount is therefore not what the district was owed, and the only way to show that
     * is to print both figures and the difference.
     */
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("That last factor is not a rate");
    await expect(card).toContainText("short");
    await expect(card).toContainText("what there was to divide");
  });

  test("the 50% transportation floor is stated against the formula's 10%", async ({ page }) => {
    // The largest single difference between how Ohio equalises instruction and how it equalises
    // getting to it, and it exists only in a spreadsheet.
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("50%");
    await expect(card).toContainText("against the formula's 10%");
  });

  test("preschool special education names the flat grant the state share does not touch", async ({
    page,
  }) => {
    /*
     * The one payment in Ohio's school funding a district's wealth does not reduce: a flat $4,000
     * a pupil whatever the category, 69% of the program. It is also what flattens the same six
     * weights that make school-age special education steeply top-heavy.
     */
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card.locator('table[data-program="preschool"]')).toBeVisible();
    await expect(card).toContainText("flat $4,000");
    await expect(card).toContainText("the state share does not reduce");
    await expect(card).toContainText("Weight, halved");
  });

  test("the page says the preschool program is over its own appropriation", async ({ page }) => {
    /*
     * The sheet is the only place in the calculator that shows what a proration is, because the
     * appropriation limit sits in a cell beside the factor — and at the stated factor the program
     * exceeds it by $908,184. Reproducing that silently and calling it verified would be the wrong
     * answer; the page states the inconsistency and what it most likely is.
     */
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("over its appropriation");
    await expect(card).toContainText("$147,500,000");
    await expect(card).toContainText("do not agree");
  });

  test("a district just below the growth cliff is told what it forwent", async ({ page }) => {
    /*
     * The only honest way to show a cliff. New Lexington grew 2.9502% against the 3% required and
     * the supplement pays on the whole roll, so missing by three hundredths of a percentage point
     * cost it $430,477.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const near = feed.districts.find(
      (d: { supplements: { growth_eligible: boolean; enrollment_change: number } }) =>
        !d.supplements.growth_eligible && d.supplements.enrollment_change > 0.029,
    );
    expect(near, "a district just below the 3% threshold").toBeTruthy();

    await page.goto(`/district/${near.irn}`);
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("just under the growth cliff");
    await expect(card).toContainText("pays on the whole roll rather than on the pupils gained");
  });
});

test.describe("the hold-harmless machinery", () => {
  test("the guarantee's open-enrolment clawback is shown as a deduction", async ({ page }) => {
    /*
     * The guarantee is not "hold the district at its old amount". A guaranteed district losing
     * open-enrolment FTE beyond a threshold has its guarantee cut at the full statewide average
     * base cost per pupil — more than the state was paying for those pupils. Columbus lost 106.2
     * FTE and had $674,561 taken off.
     */
    await page.goto("/district/043802");
    const table = page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("FY2021 funding base");
    await expect(table).toContainText("Open-enrollment clawback");
    await expect(page.locator("main")).toContainText("not at this district's share of it");
  });

  test("the second hold-harmless is named as a second one", async ({ page }) => {
    // The formula transition supplement holds a district at a larger FY2021 base that includes
    // transportation, and reaches 17 districts the guarantee does not.
    await page.goto("/district/043802");
    const main = page.locator("main");
    await expect(main).toContainText("Formula transition supplement");
    await expect(main).toContainText("three");
    await expect(main).toContainText("anchored to");
  });

  test("every component on a district page reaches the node describing it", async ({ page }) => {
    /*
     * The whole point of the wiki: a glossary nobody arrives at from the number they were reading
     * is a document museum. Every line of Ohio's formula now has a node, and every figure on the
     * dashboard links to its own.
     */
    await page.goto("/district/043802");
    const links = page.locator('main a[href^="/wiki/formula-component/fsfp-"]');
    const hrefs = await links.evaluateAll((els) =>
      [...new Set(els.map((e) => (e as HTMLAnchorElement).getAttribute("href")))].sort(),
    );
    for (const node of [
      "fsfp-targeted-assistance",
      "fsfp-special-education-weights",
      "fsfp-disadvantaged-pupil-impact-aid",
      "fsfp-career-technical-weights",
      "fsfp-english-learner-weights",
      "fsfp-gifted-units",
      "fsfp-transportation",
      "fsfp-preschool-special-education",
      "fsfp-performance-supplement",
      "fsfp-enrolment-supplements",
      "fsfp-formula-transition-supplement",
    ]) {
      expect(hrefs, node).toContain(`/wiki/formula-component/${node}`);
    }
  });

  test("the guarantee node records the clawback it used to omit", async ({ page }) => {
    /*
     * The correction that matters most: the node described a hold-harmless with no clawback in it,
     * which reproduces correctly for 566 districts and wrongly for 43.
     *
     * It is a `revisions:` entry now rather than a shouted paragraph in the body, so this asserts
     * on the disclosure — that the withdrawal is present, that it is filed apart from the current
     * account, and that it still carries the number that makes it legible.
     */
    await page.goto("/wiki/formula-component/temporary-transitional-aid-guarantee");
    const body = page.locator("main");
    await expect(body).toContainText("Open Enrolment Adjustment");

    const withdrawals = page.locator(".card.apparatus", { hasText: "What this node used to say" });
    await expect(withdrawals).toBeVisible();
    await expect(withdrawals.locator("details.revision")).toHaveCount(3);
    await expect(withdrawals).toContainText("566 districts and wrongly for 43");

    // Struck through, so a reader landing on an open disclosure can tell the withdrawn claim from
    // the live one without relying on colour.
    await expect(withdrawals.locator(".withdrawn").first()).toHaveCSS(
      "text-decoration-line",
      "line-through",
    );
  });

  test("the proration parameter names all three and which one publishes its limit", async ({
    page,
  }) => {
    await page.goto("/wiki/parameter/appropriation-proration-factor");
    const body = page.locator("main");
    await expect(body).toContainText("not a rate, a weight, a price or a threshold");
    await expect(body).toContainText("147,500,000");
    await expect(body).toContainText("A proration of 1.0 is not an absence");
  });

  test("a district touched by none of the three shows no hold-harmless table", async ({ page }) => {
    /*
     * The guard is a union of three conditions, and this test used to select on two of them. It
     * asked for the first district with no clawback and no supplement, which under the widened
     * guard is not the same set — and it would have kept passing anyway, because the first such
     * district in feed order happens to carry no guarantee either. A test that stays green while
     * asserting something it no longer means is worse than one that goes red, so the selector
     * names all three conditions explicitly.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const clean = feed.districts.find(
      (d: {
        guarantee: number;
        transition: { open_enrollment_adjustment: number; transition_supplement: number };
      }) =>
        d.guarantee <= 0 &&
        d.transition.open_enrollment_adjustment === 0 &&
        d.transition.transition_supplement === 0,
    );
    expect(clean, "a district touched by none of the three mechanisms").toBeTruthy();
    await page.goto(`/district/${clean.irn}`);
    await expect(
      page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]'),
    ).toHaveCount(0);
  });

  test("a guaranteed district with neither mechanism still gets its guarantee total", async ({
    page,
  }) => {
    /*
     * The 158 this phase is for. The guard was `clawback || supplement`, so a district carrying a
     * guarantee and neither mechanism had no table here at all, and its guarantee dollar total
     * appeared on the dashboard only in the Detail card — which is the card the next phase
     * deletes. This has to land before that one or those 158 lose the figure silently.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const guaranteed = feed.districts.find(
      (d: {
        guarantee: number;
        transition: { open_enrollment_adjustment: number; transition_supplement: number };
      }) =>
        d.guarantee > 0 &&
        d.transition.open_enrollment_adjustment === 0 &&
        d.transition.transition_supplement === 0,
    );
    expect(guaranteed, "a guaranteed district with neither mechanism").toBeTruthy();
    await page.goto(`/district/${guaranteed.irn}`);
    const table = page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("FY2021 funding base");
    await expect(table).toContainText("Guarantee");
    // The two conditional rows stay conditional. This district has neither.
    await expect(table).not.toContainText("Open-enrollment clawback");
    await expect(table).not.toContainText("Formula transition supplement");
  });

  test("a district with a mechanism and no guarantee keeps its table", async ({ page }) => {
    /*
     * The 37 a literal reading of "widen the guard to `guarantee > 0`" would have deleted the
     * table from — 22 with a clawback and 17 drawing the supplement while drawing nothing from the
     * guarantee. The supplement paragraph names those 17 by count, so a replacement rather than a
     * union would have removed the table from precisely the districts that sentence is about.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const touched = feed.districts.find(
      (d: {
        guarantee: number;
        transition: { open_enrollment_adjustment: number; transition_supplement: number };
      }) =>
        d.guarantee <= 0 &&
        (d.transition.open_enrollment_adjustment > 0 || d.transition.transition_supplement > 0),
    );
    expect(touched, "a district with a mechanism and no guarantee").toBeTruthy();
    await page.goto(`/district/${touched.irn}`);
    await expect(
      page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]'),
    ).toBeVisible();
  });
});
