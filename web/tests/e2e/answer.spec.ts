/**
 * The answer first, then the restatement, then where this district sits among the others.
 *
 * The shape every page on this site is supposed to have, in the order a reader meets it. Three
 * blocks: the lead figure and what it is measuring, the card that repeats the page's claim so it
 * can be linked to on its own, and the distribution that turns a number into a position.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND, ON_FORMULA } from "./helpers.ts";

test.describe("the answer first", () => {
  test("the page opens with state aid, not with an input to the formula", async ({ page }) => {
    /*
     * The first figure was base cost per pupil, which is what the plan says a district's education
     * costs — not money it receives, and not what the page says it is about. The `<meta
     * description>`, the share image's alt text and this card's own heading all say the question is
     * state aid and where it comes from, and the page answered a different one for eight phases.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const tiles = page.locator('[data-part="headline"] .tile');
    await expect(tiles).toHaveCount(3);
    await expect(tiles.nth(0)).toContainText("State aid, FY2027");
    await expect(tiles.nth(1)).toContainText("State aid / pupil");
    await expect(tiles.nth(2)).toContainText("Base-cost ADM");
    // And the tiles are inside the card whose heading asks the question, not floating above it.
    await expect(page.locator('[data-part="aid-source"] [data-part="headline"]')).toHaveCount(1);
  });

  test("the aid total is summed from its components, not multiplied by a pupil count", async ({
    page,
  }) => {
    /*
     * `realized_aid_per_pupil` divides by base cost ADM, and the pupil count printed one tile to
     * its right is the current-year one. Multiplying the per-pupil figure by the count beside it
     * is wrong by a median $112,601 and by $8.2m for Cleveland — a figure large enough to be a
     * finding, arrived at by an arithmetic a reader is being invited to perform.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const d = feed.districts.find((x: { irn: string }) => x.irn === CLEVELAND);
    const summed = d.base_cost_state_share + d.categorical_funding + d.guarantee;
    const multiplied = d.realized_aid_per_pupil * d.current_year_adm;
    expect(Math.abs(summed - multiplied), "the two constructions still differ").toBeGreaterThan(
      1_000_000,
    );

    await page.goto(`/district/${CLEVELAND}`);
    const printed = await page
      .locator('[data-part="headline"] .tile')
      .first()
      .locator(".v")
      .innerText();
    const dollars = Number(printed.replace(/[^0-9.]/g, ""));
    // Printed to the nearest dollar, so the tolerance is rounding rather than construction.
    expect(Math.abs(dollars - summed)).toBeLessThan(1);
  });

  test("what a year of enrollment is worth comes before what the plan says it costs", async ({
    page,
  }) => {
    /*
     * It was in position six of nine. For the 294 guaranteed districts this card answers the third
     * clause of the page's own question directly — a year of enrollment is worth nothing to them,
     * because the guarantee holds a fixed dollar amount enrollment does not enter — and a reader
     * had to pass base cost, the categoricals and the supplements to reach it.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const order = await page
      .locator("main .card[data-part]")
      .evaluateAll((nodes) => nodes.map((n) => n.getAttribute("data-part")));
    expect(order.slice(0, 4)).toEqual(["aid-source", "enrollment", "base-cost", "categoricals"]);
    expect(order[order.length - 1]).toBe("not");
  });

  test("the two views of one district finally point at each other", async ({ page }) => {
    // One district's counterfactual lever lived on one route and one district's projection on
    // another, with no cross-reference in either direction. They hold opposite things fixed, which
    // is the whole reason a reader on one wants the other.
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "holds enrollment at published FY2026 and moves the formula",
    );
    await expect(
      page.locator(`[data-part="enrollment"] a[href="/scenario?d=${CLEVELAND}"]`),
    ).toHaveCount(1);

    await page.goto(`/scenario?d=${CLEVELAND}`);
    await expect(page.locator('[data-part="not"]')).toContainText("holds enrollment fixed");
    await expect(
      page.locator(`[data-part="not"] a[href="/district/${CLEVELAND}"]`),
    ).toHaveCount(1);
  });

  test("the bar is drawn only where there are two proportions to draw", async ({ page }) => {
    /*
     * 315 districts receive exactly what the formula computes, so the split bar was a full-width
     * single segment under a legend that had collapsed to one item — a proportion chart with one
     * proportion in it. Nothing is lost by omitting it: where the guarantee pays zero, aid per
     * pupil *is* formula aid per pupil, for all 315.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator('[data-part="aid-source"] .bar .seg')).toHaveCount(2);

    await page.goto(`/district/${ON_FORMULA}`);
    await expect(page.locator('[data-part="aid-source"] .bar')).toHaveCount(0);
    // The figure the legend used to carry is still on the page, in the tile above where it was.
    await expect(page.locator('[data-part="headline"] .tile').nth(1)).toContainText(
      "all from the formula",
    );
  });

  test("each surviving condition says what follows from it, and links what it names", async ({
    page,
  }) => {
    /*
     * The flag row printed five pills and two of them restated the sub-line one element above.
     * "At or below the 20-mill floor" was strictly *less* precise than what it duplicated — the
     * sub-line separates the 21 districts genuinely below 20 mills from the 155 sitting on it, the
     * pill collapsed all 176. What is left carries a consequence, because "At the minimum state
     * share" is a noun phrase and a reader who does not already know the plan cannot get from it
     * to what it means for their district.
     */
    // Wilmington City trips both of the two that carry a corpus link: it is within a twentieth of
    // a mill of the floor and it is held at the minimum state share.
    await page.goto("/district/045112");
    const card = page.locator('[data-part="aid-source"]');
    await expect(card).not.toContainText("At or below the 20-mill floor");
    await expect(card).not.toContainText("Funded by the guarantee, not the formula");
    const conditions = page.locator('[data-part="aid-source"] .conditions li');
    await expect(conditions).toContainText([
      "Within a twentieth of a mill of the floor",
      "At the minimum state share",
      "of voted millage reduced away",
    ]);
    await expect(card.locator('.conditions a[href="/wiki/parameter/twenty-mill-floor"]')).toHaveCount(1);
    await expect(
      card.locator('.conditions a[href="/wiki/metric/state-share-percentage"]'),
    ).toHaveCount(1);
    // Every row carries a clause, not just a label.
    for (const text of await conditions.allInnerTexts()) {
      expect(text, "a condition with no consequence clause").toContain(" — ");
    }
  });

  test("the one district that tripped no condition is the one the symmetric flag was for", async ({
    page,
  }) => {
    /*
     * This asserted `toHaveCount(0)` when the conditions landed: Marion Local tripped none of the
     * four and rendered an empty `<ul>`, which reads as a gap in the page rather than as the
     * absence of a fact. The guard for that is still in `AidSource.astro` and is still correct for
     * a feed that carries a null enrollment change.
     *
     * It is unreachable with this feed, and deliberately so. Making the enrollment condition
     * symmetric — it fired only on a fall, leaving 109 districts with a Detail row as the sole
     * carrier — gave every district at least one condition, and Marion Local's is the rise that
     * kept it silent before. The assertion is inverted rather than deleted, because the empty case
     * is the thing worth remembering.
     */
    await page.goto("/district/048553");
    await expect(page.locator("h1")).toHaveText("Marion Local");
    const conditions = page.locator('[data-part="aid-source"] .conditions li');
    await expect(conditions).toHaveCount(1);
    await expect(conditions).toContainText("Enrollment up");
  });

  test("the dashboard finally says what it is not", async ({ page }) => {
    /*
     * Three of the four siblings close this way and the dashboard, with the most collisions to
     * declare, did not — while `finances.astro`'s own closing card points here and received no
     * answer back.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator('[data-part="not"]');
    await expect(card).toContainText("None of the figures above is money this district received");
    await expect(card).toContainText("divides by more than one pupil count");
    await expect(card).toContainText("two other years on a fourth count");
    // The two vintages `crates/bundle` names and the page did not: valuation is FY2023 and
    // spending is FY2024, inside one card, and the closing note is the only place that says so.
    await expect(card).toContainText("FY2023");
    await expect(card.locator(`a[href="/district/${CLEVELAND}/finances"]`)).toHaveCount(1);
    await expect(card.locator('a[href="/wiki/metric/enrolled-adm"]')).toHaveCount(1);
  });

  test("a district whose two counts round alike is told so rather than shown two equal numbers", async ({
    page,
  }) => {
    // 110 of 609. Printing the same integer twice under "more than one pupil count" would read as
    // a rendering fault rather than as the fact that this district is one of the ones they agree
    // for — so the sentence branches on it.
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const same = feed.districts.find(
      (d: { adm: number; current_year_adm: number }) =>
        Math.round(d.adm) === Math.round(d.current_year_adm),
    );
    expect(same, "a district whose two counts round alike").toBeTruthy();
    await page.goto(`/district/${same.irn}`);
    await expect(page.locator('[data-part="not"]')).toContainText(
      "round to the same figure here, which they do not for 499",
    );
  });
});

test.describe("the card that restated the page", () => {
  test("the Detail card is gone, and nothing it uniquely carried went with it", async ({ page }) => {
    /*
     * Thirteen rows, ten of them a figure already on the page — base cost per pupil printed three
     * times, aggregate base cost four, state share of base cost three, categorical funding three,
     * guarantee per pupil three, enrolled ADM three. Its docstring was false twice: "every figure
     * the feed carries" was 13 of 319 numeric leaves, and "unrounded" was 9 of 13 rows calling
     * `money()` at zero decimals. `git show 9a04427:web/src/district.ts` is byte-identical to the
     * card as it stood, from a build where the view had three `<h2>`s and every row was the page's
     * only carrier of its figure. Zero tests referenced it, which is its own indictment.
     *
     * Three things it carried alone had to move first, and each is asserted here rather than in a
     * commit message.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator('[data-part="detail"]')).toHaveCount(0);
    await expect(page.locator("main")).not.toContainText("Effective Class 1 millage");

    // 1. The departmental-estimate caveat, which was the only statement of that fact on any of the
    //    five district routes, now sits on the card whose terminal year is the estimated one.
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "partly a departmental estimate",
    );

    // 3. The reader who was using it as a reference index is pointed at where it went.
    const closing = page.locator('[data-part="not"]');
    await expect(closing).toContainText("thirteen-row table");
    await expect(closing.locator('a[href="/data/districts.csv"]')).toHaveCount(1);
  });

  test("every guaranteed district still carries its guarantee dollar total", async ({ page }) => {
    /*
     * 2. The ordering the plan calls its own worst failure mode. Under the old guard 158 of the 294
     *    guaranteed districts had no hold-harmless table, so their guarantee total was on the
     *    dashboard only in Detail — and nothing in the suite would have caught the loss, because
     *    nothing tested Detail. The union guard landed a phase earlier; this asserts it held.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const guaranteed = feed.districts.filter((d: { guarantee: number }) => d.guarantee > 0);
    expect(guaranteed.length).toBe(294);
    // Three sampled across the branch space: one with a clawback, one with the supplement, one
    // with neither — the last being the class that would have lost the figure.
    const pick = (f: (d: Record<string, never>) => boolean) => guaranteed.find(f);
    const cases = [
      pick((d: any) => d.transition.open_enrollment_adjustment > 0),
      pick((d: any) => d.transition.transition_supplement > 0),
      pick(
        (d: any) =>
          d.transition.open_enrollment_adjustment === 0 && d.transition.transition_supplement === 0,
      ),
    ];
    for (const d of cases) {
      expect(d, "each branch of the guard has a district").toBeTruthy();
      await page.goto(`/district/${(d as any).irn}`);
      const table = page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]');
      await expect(table).toContainText("Guarantee");
      await expect(table.locator("tbody")).not.toContainText("The formula reaches its FY2021 base");
    }
  });

  test("a district whose enrollment rose is told so, not left silent", async ({ page }) => {
    /*
     * The condition fired only on a negative, so for the 109 districts whose enrollment rose
     * between FY2024 and FY2026 the dashboard's sole carrier of the fact was a Detail row. That is
     * why the flag had to become symmetric before the card was deleted rather than after.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const risen = feed.districts.filter(
      (d: { enrollment_change: number | null }) =>
        d.enrollment_change != null && d.enrollment_change > 0,
    );
    expect(risen.length).toBe(109);
    await page.goto(`/district/${risen[0].irn}`);
    const conditions = page.locator('[data-part="aid-source"] .conditions');
    await expect(conditions).toContainText("Enrollment up");
    await expect(conditions).not.toContainText("Enrollment down");
  });

  test("the taxes page names the year its two publications agree in", async ({ page }) => {
    /*
     * The other half of the collision the millage row was deleted over. `/taxes` closed by saying
     * its effective Class I rate matches the profile report "to 0.01 mills, as it does for all 606
     * districts carrying both" — true of TY2023, three cards below a tile printing TY2024, which
     * the two agree on for 219 of the 606. Deleting the dashboard row and leaving this standing
     * would have removed the only figure on the site that contradicted it.
     */
    await page.goto(`/district/${CLEVELAND}/taxes`);
    const card = page.locator('[data-part="not"]');
    await expect(card).toContainText("for TY2023");
    await expect(card).toContainText("That is not the rate in the tile above");
    await expect(card).toContainText("219");
    await expect(card).toContainText("effective_class1_millage_ty23");
  });
});

test.describe("where a district sits among the others", () => {
  test("the position card draws the distribution, not a bar with a pin in it", async ({ page }) => {
    /*
     * The strip this replaced had the minimum at one end, the maximum at the other, and nothing
     * between — so the 60th percentile and the 95th were drawn identically, when the first is a
     * dense middle and the second is nearly alone. Assessed valuation per pupil reaches five and a
     * half times its median, which is the case the flat bar could not show.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator("#position");
    const charts = card.locator(".chartwrap svg.plot:visible");
    await expect(charts).toHaveCount(2);

    // The box, the median inside it, and this district's own rule on top of both.
    await expect(charts.first().locator("rect")).not.toHaveCount(0);
    await expect(charts.first().locator(".dist-marker")).toHaveCount(1);

    // Ohio's districts past the fences are drawn as themselves and can be pointed at.
    const outliers = charts.first().locator(".dist-hit circle[data-hover]");
    expect(await outliers.count()).toBeGreaterThan(0);
  });

  test("a county draws every district in it, not only its two extremes", async ({ page }) => {
    // The card's premise is that a county is a peer group. It named the richest and the poorest
    // and drew neither the fifteen between them nor how they were spread.
    await page.goto("/county/cuyahoga");
    const dots = page.locator('#spread [data-chart="county-spread"] .dist-dot circle');
    expect(await dots.count()).toBeGreaterThan(20);
    await expect(page.locator('#spread [data-chart="county-spread"] rect')).not.toHaveCount(0);
  });

  test("a population too small for quartiles gets its members and no box", async ({ page }) => {
    /*
     * 39 of Ohio's 132 legislative seats and 60 of its 88 counties hold fewer districts than a box
     * can summarise. The first version drew one anyway: a seat with three districts got a box
     * spanning almost the whole width, because the quartiles of three numbers are the numbers.
     */
    await page.goto("/house/001");
    const chart = page.locator('#members [data-chart="seat-spread"] svg.plot:visible');
    await expect(chart).toBeVisible();
    expect(await chart.locator(".dist-dot circle").count()).toBeLessThan(8);
    await expect(chart.locator("rect"), "no box below the floor").toHaveCount(0);
    await expect(page.locator("#members")).not.toContainText("shaded box");
  });

  test("a district's outcome gap is shown against the spread it is a gap in", async ({ page }) => {
    // Two tiles said this district's score and the median of its poverty fifth. Whether a
    // fifteen-point gap is remarkable depends on how wide that fifth is, and the tiles cannot say.
    await page.goto(`/district/${CLEVELAND}/outcome`);
    const chart = page.locator('#comparable-poverty [data-chart="peer-group"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart.locator(".dist-marker")).toHaveCount(1);
    expect(await chart.locator(".dist-dot circle").count()).toBeGreaterThan(50);
  });
});
