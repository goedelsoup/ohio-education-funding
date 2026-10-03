/**
 * The series behind the current year: counts, set-asides, the budget's composition, the statute.
 *
 * Four blocks that are all a reading backwards. A break in any of these series is a population
 * change before it is a policy change — `comparable` flips per district per year — so each block
 * checks that the page says which population it is counting before it draws the line.
 */

import { expect, test } from "@playwright/test";

test.describe("the count the poverty weight is paid on", () => {
  test("the seventeen-year series is on the page, which is the point of exporting it", async ({
    page,
  }) => {
    // It was computed in `crates/dispersion`, tested there, and reachable by nobody who was not
    // running cargo. This test is the one that would have failed for the four phases it sat there.
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    await expect(card).toBeVisible();
    await expect(card).toContainText("FY1999");
    await expect(card).toContainText("FY2015");
  });

  test("the break in the denominator is stated where the chart is, not in a footnote", async ({
    page,
  }) => {
    /*
     * The share steps up at FY2011 and part of that step is the divisor changing definition. A
     * reader who takes the eleven years as one trend gets a number nothing in the source supports,
     * so the page has to refuse the reading before the eye has finished drawing the line.
     */
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    await expect(card).toContainText("The two lines are not one line");
    await expect(card).toContainText("AdmCount");
    await expect(card).toContainText("CECount");
  });

  test("each row says which count it divides by, so no row travels alone", async ({ page }) => {
    // The table is the part that gets copied out. A row carrying a share without its basis is a
    // figure that means two different things depending on a cutover the row does not mention.
    //
    // Data rows only: the table now closes one row group and opens another where the denominator
    // changes, and the labelled row between them is structure rather than a year. Counting every
    // `tr` made this fail at 18 when the break landed, which is the assertion catching a real
    // change in the table's shape — and the shape is the point of the change.
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    const rows = card.locator("tbody tr:not(.series-break)");
    // Twenty-eight Octobers, which used to be seventeen: the open directory ends at October 2014
    // and the report did not — it moved to the department's own page. See #16.
    await expect(rows).toHaveCount(28);
    await expect(rows.first()).toContainText("AdmCount");
    await expect(rows.last()).toContainText("CECount");
    // And the two the waivers emptied say so where a reader meets them, rather than printing a
    // band twenty points above their neighbours as though it were a reading of Ohio.
    await expect(card.locator("tbody tr", { hasText: "not the state" })).toHaveCount(2);
  });

  test("a headline tile carries its own year, because no card chip reaches it", async ({ page }) => {
    /*
     * Rule 1, scoped to where it is not already satisfied.
     *
     * The design system's highest-value rule is that no numeric or currency literal appears outside
     * a `.fig`. Measured against the built site, 96% of figures already carry their year: 52% sit
     * in a card whose heading has a year chip, and 44% in a table cell, where the design itself
     * puts the annotation on the column head rather than repeating it down 609 rows.
     *
     * The 4% that did not were the tiles at the head of a route, above the first card, where no
     * chip reaches them. This page led with a taxable value, a millage and a tax charged, none of
     * which said which year they measured.
     *
     * A built-DOM assertion rather than a source one: "above the first card" is a fact about the
     * rendered page. In the source those tiles sit in one branch of a ternary whose other branch
     * opens with a card, so a source-order check reads the head as empty and passes vacuously.
     */
    await page.goto("/district/043653/taxes");
    const headline = page.locator(".tiles").first().locator(".tile");
    await expect(headline).not.toHaveCount(0);
    for (const tile of await headline.all()) {
      await expect(tile.locator(".v .fig-year")).toHaveCount(1);
    }
    // And the year is the tax year the row is measured in, not the page's own.
    await expect(headline.first().locator(".v .fig-year")).toHaveText(/^TY\d{4}$/);
  });

  test("and the change of denominator is a break in the table, not only in the chart", async ({
    page,
  }) => {
    /*
     * The chart has drawn this as two separate series since it was built, because the share steps
     * up across the cutover and a line through it is a lie. The table ran straight through the
     * same break with only a per-row "Counted on" cell to say so — a fact a reader meets after the
     * eye has already gone down the column.
     */
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    await expect(card.locator("tr.series-break")).toHaveCount(1);
    await expect(card.locator("tr.series-break")).toContainText("two series");
    // A row group a screen reader announces, not a border only a sighted reader sees.
    await expect(card.locator("tbody")).toHaveCount(2);
  });

  test("the years published as three files carry a range where a share would be", async ({
    page,
  }) => {
    /*
     * The finding this extension exists for. From FY2013 only one of the three files still
     * counts applications, so the table has to print a band rather than a figure — and the page
     * has to say that the figure a reader could compute instead would read as poverty
     * collapsing.
     */
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    const rows = card.locator("tbody tr");
    await expect(rows.last()).toContainText("three files");
    await expect(rows.last()).toContainText("–");
    await expect(card).toContainText("poverty collapsing");
    await expect(card).toContainText("collect no forms at all");
  });

  test("the population is named as sponsors rather than districts", async ({ page }) => {
    /*
     * Everything else on this site counts 609 traditional districts. This counts meal-program
     * sponsors — community schools and county boards of developmental disabilities included — and
     * the count grows across the window for reasons that have nothing to do with poverty.
     */
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    await expect(card).toContainText("sponsors");
    await expect(card).toContainText("community schools");
    await expect(card).toContainText("formula side");
  });
});

test.describe("what the legislature set aside", () => {
  test("the appropriation card is on the page and reaches FY2002", async ({ page }) => {
    // The series exists in the crates since the Catalog extraction; this is the test that would
    // have failed while it was reachable only by running cargo.
    await page.goto("/history");
    const card = page.locator('.card[data-part="appropriations"]').first();
    await expect(card).toBeVisible();
    await expect(card).toContainText("FY2002");
    await expect(card).toContainText("What the legislature set aside");
  });

  test("both dollar bases are in the document, so the switch works without script", async ({
    page,
  }) => {
    /*
     * The whole argument for this card is that the nominal and real readings of one series support
     * opposite sentences. A reader who gets only the nominal panel has been shown the claim
     * without the check — so both are baked in at build, as `BasisToggle` requires.
     */
    await page.goto("/history");
    const cards = page.locator('.card[data-part="appropriations"]');
    await expect(cards).toHaveCount(2);
    await expect(cards.first()).toContainText("the dollars of each year");
    await expect(cards.last()).toContainText("constant FY");
  });

  test("the card says which publication each year came from", async ({ page }) => {
    // Four years rest on a different document from the rest. The two agree to the cent where both
    // speak, so the column records provenance rather than doubt — and the card says so.
    await page.goto("/history");
    const card = page.locator('.card[data-part="appropriations"]').first();
    await expect(card).toContainText("Catalog");
    await expect(card).toContainText("Greenbook");
    await expect(card).toContainText("agree to the cent");
  });

  test("the appropriation is never shown per pupil", async ({ page }) => {
    /*
     * The hazard `denominators.ts` records for this block. A statewide appropriation over a pupil
     * count would be a per-pupil figure on a denominator nothing else here uses, a scroll away
     * from the formula's own per-pupil numbers — and it would be a rate for money no pupil
     * necessarily received, since an appropriation is a ceiling.
     */
    await page.goto("/history");
    const card = page.locator('.card[data-part="appropriations"]').first();
    await expect(card).not.toContainText("per pupil");
    await expect(card).not.toContainText("per-pupil");
  });
});

test.describe("what the budget is made of", () => {
  test("the lines behind the totals are on the page", async ({ page }) => {
    // Extracted with the Catalog and reachable by no reader until now — the same gap this site
    // has closed for the Census panel, MR-81 and the appropriation series.
    await page.goto("/history");
    const card = page.locator('.card[data-part="line-origins"]');
    await expect(card).toBeVisible();
    await expect(card).toContainText("What the budget is made of");
    await expect(card.locator("tbody tr").first()).toBeVisible();
  });

  test("the oldest line is named with the act that created it", async ({ page }) => {
    /*
     * The finding. A budget line still being funded that predates DeRolph says something about
     * how budgets are made that no total does, and it is only sayable because the Catalog prints
     * an establishing act for each line.
     */
    await page.goto("/history");
    const card = page.locator('.card[data-part="line-origins"]');
    await expect(card).toContainText("oldest line still being funded");
    await expect(card).toContainText("G.A.");
    await expect(card).toContainText("DeRolph");
  });

  test("lines with no establishing act say so instead of rendering blank", async ({ page }) => {
    // A blank cell reads as an extraction that failed. "Not stated" reads as the document
    // declining to say, which is what happened — and the card explains why it is not filled in.
    await page.goto("/history");
    const card = page.locator('.card[data-part="line-origins"]');
    await expect(card).toContainText("not stated");
    await expect(card).toContainText("name no establishing act");
  });

  test("the discontinued label is not presented as abolition", async ({ page }) => {
    /*
     * `state-foundation-aid` holds the open question of whether the department's disappearing
     * lines were abolished or folded into others. This card carries the publisher's flag and must
     * not let a reader mistake it for the answer.
     */
    await page.goto("/history");
    const card = page.locator('.card[data-part="line-origins"]');
    await expect(card).toContainText("not a finding about");
    await expect(card).toContainText("open question this cannot settle");
  });
});

test.describe("the statute timeline", () => {
  test("the Library menu opens onto it, and it is the first thing in the panel's columns", async ({ page }) => {
    // First in the `Law` run, which is first in the panel's columns: the one link there that is
    // not an index, and the one a reader asking "how did this get here" wants before any single
    // act. Only the record's three tiles sit above it, across both columns.
    await page.goto("/");
    const law = page.locator("header.site nav details.menu").nth(2);
    await law.locator("summary").click();
    const first = law.locator(".menu-runs a").first();
    await expect(first).toHaveAttribute("href", "/legislation");
    await first.click();
    await expect(page).toHaveURL(/\/legislation$/);
    await expect(page.locator("h1")).toHaveText("The statute timeline");
  });

  test("a formula that ran one biennium is drawn narrower than one that ran five", async ({
    page,
  }) => {
    /*
     * The claim the chart exists to make, measured in rendered pixels rather than in the
     * percentages the renderer wrote — which is the half the unit suite cannot reach. The first
     * version of this chart put each label inside its own band, and the Evidence-Based Model's
     * band was too narrow to hold one; the bar is beside the name now precisely so that the
     * narrowest element can stay narrow.
     */
    await page.goto("/legislation");
    const row = (name: string) => page.locator("#regimes tbody tr").filter({ hasText: name });
    const width = async (name: string) =>
      (await row(name).locator(".span-bar > i").boundingBox())!.width;
    const left = async (name: string) =>
      (await row(name).locator(".span-bar > i").boundingBox())!.x;

    const brief = await width("Evidence-Based Model");
    const long = await width("Bridge Formula");
    expect(brief).toBeGreaterThan(0);
    expect(long).toBeGreaterThan(brief * 3);
    // And later formulas start further along, because the axis is time.
    expect(await left("Bridge Formula")).toBeGreaterThan(await left("Evidence-Based Model"));
    expect(await left("Equal Yield Formula")).toBeLessThan(await left("Bridge Formula"));
  });

  test("an act with no formula edge says so rather than rendering an empty cell", async ({
    page,
  }) => {
    // H.B. 920 sets the tax reduction factors and touches no formula; H.B. 583 corrects one and
    // appropriates nothing. Both would be blank cells, and a blank cell in a generated table
    // reads as a load that failed.
    await page.goto("/legislation");
    const acts = page.locator("#acts tbody tr");
    await expect(acts.filter({ hasText: "H.B. 920" })).toContainText("no formula change recorded");
    await expect(acts.filter({ hasText: "H.B. 920" })).toContainText("not a budget act");
    await expect(acts.filter({ hasText: "H.B. 583" })).toContainText("corrects");
    await expect(page.locator("#acts tbody td:empty")).toHaveCount(0);
  });
});

test.describe("the page's charts are near each other", () => {
  test("no two consecutive charts are more than 2,500px apart at 1280", async ({ page }) => {
    /*
     * #708. A 69-row table and a 30-row one stood between charts, and at 1280 the reader scrolled
     * 5,400px from one drawing to the next. Measured on the charts a reader is shown — the hidden
     * basis's drawings have no box — so a basis toggle does not count its charts twice.
     */
    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto("/history");
    const tops = await page.evaluate(() =>
      [...document.querySelectorAll("svg.plot")]
        .map((svg) => svg.getBoundingClientRect())
        .filter((box) => box.height > 0)
        .map((box) => Math.round(box.top + window.scrollY)),
    );
    expect(tops.length, "the page draws its charts").toBeGreaterThanOrEqual(4);
    const far = tops.slice(1).flatMap((top, i) => (top - tops[i]! > 2_500 ? [`${tops[i]} → ${top}`] : []));
    expect(far, "consecutive charts farther apart than 2,500px").toEqual([]);
  });

  test("each chart that crosses a change of formula marks it, and says which", async ({ page }) => {
    // #708: thirty years across four regimes, and no time axis marked one of them. FY2012 because
    // every chart here spans it; the meal series stops before FY2022, so its key ends at Bridge.
    await page.goto("/history");
    for (const part of ["revenue-mix", "equity-gap", "meal-program", "appropriations"]) {
      const card = page.locator(`.card[data-part="${part}"]`).first();
      await expect(card.locator(".series-event").first(), part).toBeAttached();
      await expect(card.locator(".regime-key"), part).toContainText("FY2012, Bridge Formula");
    }
  });
});
