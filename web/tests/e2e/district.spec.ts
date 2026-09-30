/**
 * One district's page, from the band at the top down to the money nothing else counts.
 *
 * `/district/[irn]` is the deepest route on the site and these four blocks are its four layers: the
 * band, the base cost build-up under it, the categorical weights beside it, and the federal and
 * other revenue that no formula figure includes. Together they are what a superintendent opening
 * their own district sees.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND, ON_FORMULA } from "./helpers.ts";

test.describe("one district's own band", () => {
  test("a guaranteed district shows what the formula computes beside what it receives", async ({
    page,
  }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const chart = page.locator('[data-chart="district-fan"] svg.plot:visible');
    // Its aid does not respond to its enrollment at all, so the band collapses — and the second
    // line, the formula's own falling answer, is what makes the chart say something.
    await expect(chart.locator(".fan-reference")).toHaveCount(1);
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "flat by construction",
    );
    // And no bias figure on this side. A guaranteed district's aid is a fixed dollar amount that
    // enrollment does not enter, so the bias of an enrollment forecast says nothing about what it
    // receives — and a card that printed one anyway would be attaching a measurement to a line the
    // measurement cannot reach.
    await expect(page.locator('[data-part="enrollment"]')).not.toContainText(
      "centered high",
    );
  });

  test("a formula-funded district gets a band and no second line", async ({ page }) => {
    await page.goto(`/district/${ON_FORMULA}`);
    const chart = page.locator('[data-chart="district-fan"] svg.plot:visible');
    await expect(chart.locator(".fan-band")).toHaveCount(1);
    await expect(chart.locator(".fan-reference")).toHaveCount(0);
    const card = page.locator('[data-part="enrollment"]');
    await expect(card).toContainText("The range, not the line, is the finding");
    // The level, beside the width. Read off the painted text rather than asserted as a figure: the
    // numbers come from the feed at the fan's own horizon, and what this checks is that both
    // populations reach the page and that neither is presented as this district's own error.
    await expect(card).toContainText("The band is centered high, on the average district");
    await expect(card).toContainText("forecast 6 years out");
    await expect(card).toContainText("before the pandemic school closures");
    await expect(card).toContainText("closures included");
    await expect(card).toContainText("not a figure for this one");
  });

  test("compares enrollment years without inventing a published one", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator('[data-part="enrollment"]');
    await expect(card).toContainText("FY2024");
    await expect(card).toContainText("FY2026 — the model's own");
    await expect(card).toContainText("These are not published FY2025 and FY2026 funding totals");
  });
});

test.describe("the base cost build-up", () => {
  test("shows the statute's twenty-two elements, and reconciles against the department", async ({
    page,
  }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator('[data-part="base-cost"]');
    await expect(card).toBeVisible();

    // Five sub-components and their elements, plus the aggregate row.
    for (const code of ["A1", "A4", "B7", "C1", "C7", "D3", "E"]) {
      await expect(card.locator("tbody")).toContainText(code);
    }
    await expect(card.locator("tbody tr").last()).toContainText("Aggregate base cost");

    // The claim, and the reconciliation that licenses it. A card asserting it reproduced the
    // department without printing the difference would be asking to be believed.
    await expect(card).toContainText("computed here, not quoted");
    await expect(card).toContainText("The department publishes its own aggregate");
  });

  test("the category labels are not clipped by the chart's margin", async ({ page }) => {
    // "Building leadership and operation" is the longest label on this site and rendered as
    // "g leadership and operation" against the fixed margin this replaced — which reads as a
    // rendering fault rather than as truncation, and so gets reported by nobody.
    await page.goto(`/district/${CLEVELAND}`);
    const labels = page.locator('[data-chart="base-cost"] svg.plot:visible .bar-label text');
    await expect(labels).toHaveCount(5);
    const chart = page.locator('[data-chart="base-cost"] svg.plot:visible');
    const box = await chart.boundingBox();
    for (let i = 0; i < 5; i++) {
      const label = await labels.nth(i).boundingBox();
      expect(label!.x, `label ${i} starts left of the chart`).toBeGreaterThanOrEqual(box!.x - 0.5);
    }
  });
});

test.describe("the categorical half", () => {
  test("the lump is shown as six programs that reconcile to it", async ({ page }) => {
    /*
     * Categorical funding was a residual for eight phases — core foundation less the state share
     * of base cost. Exact, and 43% of formula aid expressed as a number with no parts.
     */
    await page.goto("/district/043802");
    const card = page.locator('[data-part="categoricals"]');
    for (const program of [
      "Targeted assistance",
      "Special education",
      "Disadvantaged Pupil Impact Aid",
      "Career-technical education",
      "Gifted",
      "English learners",
    ]) {
      await expect(card).toContainText(program);
    }
    await expect(card).toContainText("Total categorical funding");
  });

  test("special education is broken into its six weighted categories", async ({ page }) => {
    /*
     * The second level. The weights span a factor of sixteen and the money runs against them —
     * Columbus's Category 6 is 20% of its special education pupils and 55% of the aid. A total
     * cannot say that, and the total is what this page showed until now.
     */
    await page.goto("/district/043802");
    const card = page.locator('[data-part="categoricals"]');
    await expect(card).toContainText("Special education, by category");
    await expect(card).toContainText("3.9554");
    await expect(card).toContainText("0.2435");
    await expect(card).toContainText("Category 6");
    await expect(card).toContainText("a range of sixteen");

    // Scoped to special education's own table. The card now carries three tables with rows
    // labelled `Category N` — special education's six, career-technical's five and English
    // learners' three — and an unscoped count picked up all thirteen.
    const rows = card
      .locator('table[data-program="special-education"] tbody th')
      .filter({ hasText: /^Category \d$/ });
    await expect(rows).toHaveCount(6);
  });

  test("the descending-weights claim is the one the page can support", async ({ page }) => {
    /*
     * This card asserted that English learners was the only weighted categorical whose weights
     * descend — "every other weighted categorical in the plan runs the other way" — five table
     * rows below career-technical's own weights, 0.623 → 0.157, printed in a column headed Weight
     * and strictly descending. The claim was false as written and was inherited from the corpus
     * node, which asserted it too.
     *
     * The supportable claim is narrower and more interesting: both descend, and only English
     * learners descends along a *need* gradient. Career-technical's ordering is programme type,
     * which its own corpus node says.
     *
     * The correction landed in three places and missed a fourth: the six-row summary table's own
     * description column said "Three weights that descend, unlike every other categorical", one
     * screenful above the drill-down that had just been corrected. Both wordings are asserted here
     * so the claim cannot come back in either of the two places it lived.
     */
    await page.goto("/district/043802");
    const card = page.locator('[data-part="categoricals"]');
    await expect(card).not.toContainText("Every other weighted categorical");
    await expect(card).not.toContainText("unlike every other categorical");
    await expect(card).toContainText("the only categorical that pays less as the thing it is for");
    await expect(card).toContainText("descend as need persists, alone among the six");
    // Both halves of the distinction are on the page, not just the correction.
    await expect(card).toContainText("Special education runs the other way");
    await expect(card).toContainText("along program type rather than need");
  });

  test("the frozen FY2021 counts travel with the columns they qualify", async ({ page }) => {
    // Two cards print an FTE column funded on enrolment as it stood six years before the year
    // being funded, beside a base cost built on a rolling average ending FY2026. The corpus said
    // so for both programs and neither page did.
    await page.goto("/district/043802");
    const card = page.locator('[data-part="categoricals"]');
    await expect(card).toContainText("Category 1 Career Tech FTE-FY21");
    await expect(card).toContainText("Category 1 EL ADM-FY21");
    await expect(card.getByText("The counts are frozen at FY2021", { exact: false })).toHaveCount(2);
  });

  test("the two base-cost figures say they do not divide into the state share", async ({ page }) => {
    // They are computed on different pupil counts, and a reader who divides the pair the sentence
    // puts side by side gets a percentage below the statutory floor for 125 districts.
    await page.goto("/district/046797");
    const card = page.locator('[data-part="base-cost"]');
    await expect(card).toContainText("do not divide into the state share percentage");
    await expect(card).toContainText("499 of Ohio's");
    // And it says only what has been established, rather than asserting how the floor applies.
    await expect(card).toContainText("is not something this site has established");
  });

  test("the smallest district reads as small rather than as broken", async ({ page }) => {
    // Kelleys Island is funded 0.22 classroom teachers. Rounded, that printed as "0 funded
    // classroom teachers" under a heading saying base cost is $371,449 per pupil.
    await page.goto("/district/046797");
    const card = page.locator('[data-part="base-cost"]');
    await expect(card).toContainText("0.22 funded classroom teachers");
    // And no share cell reads as a missing value.
    await expect(card.locator("td", { hasText: /^0\.0%$/ })).toHaveCount(0);
    await expect(card.locator("td", { hasText: "<0.1%" })).not.toHaveCount(0);
  });

  test("targeted assistance shows both tiers and names the two pupil counts", async ({ page }) => {
    /*
     * The largest categorical in Ohio, and the one whose total says least. `[G]` is `[C] + [F]`
     * and the addends measure different things — the size of the tax base against its size per
     * pupil — so a district can draw either, both or neither.
     *
     * The card must also say which pupil count each step uses. The wealth tier divides by resident
     * ADM and pays on enrolled ADM, one line apart in the department's own formula, and a page
     * that prints a per-pupil figure without saying which is the exact error this site shipped
     * twice before `denominators.ts` existed.
     */
    await page.goto("/district/043786");
    const card = page.locator('[data-part="categoricals"]');
    const table = card.locator('table[data-program="targeted-assistance"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("Weighted wealth");
    await expect(table).toContainText("Capacity index");
    await expect(table).toContainText("Wealth per resident pupil");
    await expect(table).toContainText("resident pupils");
    await expect(table).toContainText("enrolled");
  });

  test("DPIA says its index is squared and shows what that costs or earns", async ({ page }) => {
    /*
     * The squaring is the program. A district at twice the state's poverty rate scores four times
     * the index, and $525m distributed on a convex curve concentrates far more sharply than a
     * per-pupil rate would. Nothing in a DPIA total shows it, so the card states both the index
     * and what a linear one would have been.
     */
    await page.goto("/district/043786");
    const table = page.locator('[data-part="categoricals"] table[data-program="dpia"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("Blended count");
    await expect(table).toContainText("squared");
    await expect(page.locator('[data-part="categoricals"]')).toContainText(
      "DPIA is convex",
    );
  });

  test("gifted shows units against what the district earned, so a floor is visible", async ({
    page,
  }) => {
    /*
     * Gifted is the one categorical with a floor rather than a proportion: 0.5 coordinator units
     * and 0.3 of each specialist unit regardless of how few gifted pupils a district identifies.
     * 370 districts sit on the coordinator floor. The card prints units awarded beside units
     * earned, which is the only way a reader can see that the money is a minimum.
     */
    await page.goto("/district/043786");
    const table = page.locator('[data-part="categoricals"] table[data-program="gifted"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("Identification");
    await expect(table).toContainText("Coordinator");
    await expect(table).toContainText("Earned");
    // Cleveland is at the eight-coordinator cap, which binds from 26,400 pupils upward.
    await expect(table).toContainText("8.0000");
  });

  test("career-technical names the base cost its weights multiply", async ({ page }) => {
    /*
     * CTE weights multiply a career-technical base cost of $9,855.62, not the $8,241.61 the rest
     * of the plan uses. A table of Ohio's weights read as one scale understates this program by a
     * fifth, so the card says both figures rather than the weights alone.
     */
    await page.goto("/district/043786");
    const card = page.locator('[data-part="categoricals"]');
    await expect(card.locator('table[data-program="career-technical"]')).toBeVisible();
    await expect(card).toContainText("$9,856");
    await expect(card).toContainText("not the $8,242");
  });

  test("English learners says its weights descend", async ({ page }) => {
    /*
     * 0.2104, 0.1577, 0.1053 — Category 1 is the most recently arrived learner and is funded at
     * twice Category 3. Every other weighted categorical in the plan runs the other way, so a
     * reader assuming "more need, more money" has this one backwards.
     */
    await page.goto("/district/043786");
    const card = page.locator('[data-part="categoricals"]');
    await expect(card.locator('table[data-program="english-learners"]')).toBeVisible();
    await expect(card).toContainText("0.2104");
    await expect(card).toContainText("descend");
  });

  test("each of the six programs links to the corpus node describing its mechanism", async ({
    page,
  }) => {
    /*
     * The six were a single residual for eight phases and had nothing to link to. Now each is a
     * `formula-component` in its own right, and a glossary nobody arrives at from the number they
     * were reading is a document museum — so the link out from the figure is the point.
     */
    await page.goto("/district/043786");
    const card = page.locator('[data-part="categoricals"]');
    const links = card.locator('a[href^="/wiki/formula-component/fsfp-"]');
    await expect(links).toHaveCount(6);
    for (const node of [
      "fsfp-targeted-assistance",
      "fsfp-special-education-weights",
      "fsfp-disadvantaged-pupil-impact-aid",
      "fsfp-career-technical-weights",
      "fsfp-gifted-units",
      "fsfp-english-learner-weights",
    ]) {
      await expect(card.locator(`a[href="/wiki/formula-component/${node}"]`)).toHaveCount(1);
    }
  });

  test("the targeted assistance node says it is an equalisation, not a categorical", async ({
    page,
  }) => {
    // The modelling decision the six nodes exist to record. Targeted assistance pays for the
    // absence of a tax base, which is what local capacity measures; the other five pay for a kind
    // of pupil. A single "categoricals" node would have said none of that.
    await page.goto("/wiki/formula-component/fsfp-targeted-assistance");
    await expect(page.locator("h1")).toContainText("Targeted Assistance");
    const body = page.locator("main");
    await expect(body).toContainText("does not belong with them");
    // Section headings now, not shouted capitals. The claim is what is being pinned; the
    // capitals were how it competed for attention inside a wall of body copy.
    await expect(body).toContainText("The capacity tier has a size cliff");
    await expect(body).toContainText("qualifies districts and pays them nothing");
    // And it points at local capacity, which is the placement decision itself: both measure the
    // tax base, and the edge is what stops a reader filing this with the weighted programs.
    const capacity = page.locator(
      'main a[href="/wiki/formula-component/fsfp-local-capacity-measure"]',
    );
    expect(await capacity.count()).toBeGreaterThan(0);
  });

  test("a district getting no targeted assistance is told why", async ({ page }) => {
    /*
     * The reason the sum misleads rather than merely omitting. Targeted assistance is the largest
     * categorical in Ohio and it is equalisation: Columbus qualifies for none of it, and 77% of
     * its categorical money is DPIA instead. One number cannot distinguish those.
     */
    await page.goto("/district/043802");
    const card = page.locator('[data-part="categoricals"]');
    await expect(card).toContainText("It receives no targeted assistance");
    await expect(card).toContainText("135 districts are in the same position");
    await expect(card).toContainText("Disadvantaged Pupil Impact Aid");
  });
});

test.describe("the money no other figure on the site counts", () => {
  test("the card is on the finances route, and nothing above it carries the money", async ({
    page,
  }) => {
    /*
     * The failure this stands against is the one the feed made available for four phases: a
     * per-district series computed in Rust, tested in Rust, and reachable by nobody who was not
     * running cargo. It is also the failure the page made available for longer — a reader who
     * added up a district's pages was told by omission that they had seen all the state money.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator('.card[data-part="casino"]');
    await expect(card).toBeVisible();
    await expect(card).toContainText("nothing above this card counts it");
    await expect(card).toContainText("gross casino revenue county student fund");
  });

  test("the chip says the series, not the page's year", async ({ page }) => {
    // Nine fiscal years on a route whose other card spans six, and the chip has to describe the
    // card it sits on rather than the route it sits in.
    await page.goto(`/district/${CLEVELAND}/finances`);
    const chip = page.locator('.card[data-part="casino"] .year-chip');
    await expect(chip).toHaveText("FY2016-FY2024");
    await expect(chip).toHaveAttribute("data-kind", "fiscal");
  });

  test("the district figure and the statewide figure are both there, and they differ", async ({
    page,
  }) => {
    // A district's own money is the reason to look; the statewide column is what stops the reader
    // concluding that this district's fall in FY2021 was about this district.
    await page.goto(`/district/${CLEVELAND}/finances`);
    const rows = page.locator('.card[data-part="casino"] table[data-program="casino"] tbody tr');
    await expect(rows).toHaveCount(9);
    await expect(rows.first()).toContainText("FY2016");
    await expect(rows.last()).toContainText("FY2024");
    await expect(rows.last()).toContainText("$114,177,214");
  });

  test("the closure is named as a closure and put in the year the money moved", async ({
    page,
  }) => {
    /*
     * The casinos shut in March 2020 and the money arrives in FY2021, because the August payment
     * settles the half-year that ended in June. A page that put it in FY2020 would be describing
     * the revenue period, which is not what any other figure on the route is on.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator('.card[data-part="casino"]');
    await expect(card).toContainText("FY2021 is the closure");
    await expect(card).toContainText("mid-March 2020");
    await expect(card).toContainText("Nothing cushioned it");
  });

  test("no per-pupil figure appears, and the card says which count it would have had to use", async ({
    page,
  }) => {
    /*
     * Four per-pupil figures already sit on a district's pages. A fifth computed from this fund
     * would read as one of them and would divide by a denominator R.C. 5753.11 defines for this
     * fund alone — county-resident pupils, community and STEM and joint vocational enrolment
     * included, dual-enrolled pupils counted twice. The refusal is the finding.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator('.card[data-part="casino"]');
    await expect(card).toContainText("no per-pupil figure here");
    await expect(card).toContainText("R.C. 5753.11");
    await expect(card.locator(".tile", { hasText: "per pupil" })).toHaveCount(0);
  });

  test("the dashboard's caveat card sends a reader to it rather than staying silent", async ({
    page,
  }) => {
    // "What this page is not" already told a reader that none of the figures above it is money
    // received. It could not say that the finances route was also short of some — which made the
    // two routes together look exhaustive.
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator('.card[data-part="not"]');
    await expect(card).toContainText("there is state money in neither");
    await expect(card.locator('a[href$="/finances#casino"]')).toHaveCount(1);
  });
});
