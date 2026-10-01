/**
 * What a district spends, on what, out of what, against everyone else.
 *
 * Four blocks over the F-33 routes. They are one file because the four readings share a source and
 * a hazard: `state_revenue` there is not foundation aid, and it breaks at FY2016. See
 * `src/lib/spending.ts` and the `survey_basis` note in the crates.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND, MOST_FEDERAL } from "./helpers.ts";

test.describe("the finances route", () => {
  test("shows six closed years of money that changed hands", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}/finances`);
    await expect(page.locator(".basis-panel.nominal tbody tr")).toHaveCount(6);
    await expect(page.locator(".basis-panel.nominal")).toContainText("audited actuals");
  });

  test("refuses to be read as a check on the model", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}/finances`);
    await expect(page.getByRole("heading", { name: /What these numbers are not/ })).toBeVisible();
    await expect(page.locator('[data-part="not"]')).toContainText(
      "not comparable line for line",
    );
  });

  test("deflating reverses the sign of the statewide cash story", async ({ page }) => {
    // The reason both bases are offered rather than one: they support opposite arguments, and the
    // difference is entirely CPI.
    await page.goto("/statewide");
    const nominal = page.locator(".basis-panel.nominal .tile", { hasText: "Change since FY2020" });
    await expect(nominal.locator(".v")).toHaveClass(/gain/);
    const real = page.locator(".basis-panel.real .tile", { hasText: "Change since FY2020" });
    await expect(real.locator(".v")).toHaveClass(/loss/);
  });
});

test.describe("spending by function", () => {
  test("splits operating spending and keeps it apart from the audited actuals", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator('[data-part="spending-by-function"]');
    await expect(card).toBeVisible();
    for (const fn of ["Instruction", "Pupil transportation", "General administration"]) {
      await expect(card.locator("tbody")).toContainText(fn);
    }
    // The department's two roll-ups, which partition the total exactly.
    await expect(card.locator("tbody")).toContainText("Classroom instruction");
    await expect(card.locator("tbody")).toContainText("Everything else");
    // And the separation this page exists to preserve.
    await expect(card).toContainText("not the audited actuals above");
    await expect(card).toContainText("unweighted ADM");
  });

  test("spending by function is drawn largest first", async ({ page }) => {
    // #611: in the department's line order the bars stepped down and back up, which reads as a
    // sort that failed.
    await page.goto(`/district/043802/finances`);
    const svg = page.locator('[data-chart="functions"] svg.plot:visible');
    // `> *` because a rounded bar is a path rather than a rect.
    const bars = svg.locator("g.bar-fill > *");
    await expect(bars.first()).toBeAttached();
    const widths = await bars.evaluateAll((marks) =>
      marks.map((m) => {
        const box = (m as SVGGraphicsElement).getBBox();
        return { y: box.y, w: box.width };
      }),
    );
    expect(widths.length).toBeGreaterThan(2);
    const ordered = widths.sort((a, b) => a.y - b.y).map((r) => r.w);
    expect(ordered).toEqual([...ordered].sort((a, b) => b - a));
  });

  test("a district with no report-card row says so rather than showing zero", async ({ page }) => {
    // Two of the 609 have no spending row. Rendering them as $0 across every function would be a
    // finding about their spending rather than about the file.
    await page.goto("/district/046797/finances");
    await expect(page.getByText("No report-card spending row is published")).toBeVisible();
  });
});

test.describe("where the money came from", () => {
  test("the federal share leads and the dollars name their denominator", async ({ page }) => {
    /*
     * The share is the figure that survives being compared between districts, because both its
     * parts are per need-weighted pupil and the ratio cancels the denominator. The dollars do not,
     * and the card above this one divides by a headcount — so the basis has to be on the page.
     */
    await page.goto(`/district/${MOST_FEDERAL}/finances`);
    const card = page.locator('[data-part="federal-share"]');
    await expect(card).toContainText("29.0%");
    await expect(card).toContainText("statewide median 4.2%");
    await expect(card).toContainText("need-weighted count");
    await expect(card).toContainText("the two cards do not add");
  });

  test("the federal correlation is never shown without its controlled twin", async ({ page }) => {
    // Federal money follows poverty. The raw figure alone would state a confound as a finding.
    await page.goto(`/district/${MOST_FEDERAL}/finances`);
    const card = page.locator('[data-part="federal-share"]');
    await expect(card).toContainText("Holding poverty constant");
    await expect(card).toContainText("Neither figure identifies an effect");
  });

  test("both growth measures appear, and a district on zero is told so", async ({ page }) => {
    /*
     * Bellevue City prints 0.00 over one year. At two decimals that covers anything within half a
     * hundredth of zero, so it has no direction — and calling it a disagreement is the arithmetic
     * error that turned 44 into 76.
     */
    await page.goto("/district/043596/outcome");
    const card = page.locator('[data-part="outcomes"]');
    await expect(card).toContainText("Progress, three-year average");
    await expect(card).toContainText("Progress, one year");
    await expect(card).toContainText("has no direction to read");
    await expect(card).toContainText("534 districts");
    await expect(card).not.toContainText("point opposite ways for this district");
  });

  test("a district whose measures point opposite ways is told to read neither", async ({ page }) => {
    await page.goto(`/district/${MOST_FEDERAL}/outcome`);
    const card = page.locator('[data-part="outcomes"]');
    await expect(card).toContainText("point opposite ways for this district");
    await expect(card).toContainText("Read it as neither");
    await expect(card).toContainText("none of them with both magnitudes above 0.05");
  });
});

test.describe("against america", () => {
  test("a district is placed in the national distribution, not the Ohio one", async ({ page }) => {
    /*
     * The comparison every other page here cannot make. Ohio describing itself can say what Ohio
     * does and not whether it is unusual, and 10,382 districts on the Census Bureau's definitions
     * is the only thing in this feed that is not Ohio describing itself.
     */
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="national"]');
    await expect(card).toContainText("Against America");
    await expect(card).toContainText("10,382 school districts in every state");
    await expect(card).toContainText("Local share of revenue");
    await expect(card).toContainText("percentile");
  });

  test("the year and the denominator are stated, because neither matches the page", async ({
    page,
  }) => {
    // FY2022 against the model's FY2027, on federal fall membership rather than Ohio's ADM. This
    // card sits three below one showing operating expenditure per pupil on a different count in a
    // different year, which is the exact shape of the error `denominators.ts` exists for.
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="national"]');
    await expect(card).toContainText("These are not the figures above");
    await expect(card).toContainText("FY2022");
    await expect(card).toContainText("federal fall membership");
  });

  test("a high-local-share district is told it is in the national top fifth", async ({ page }) => {
    // Orange City raises 85% of its money locally, the 99th percentile nationally — against
    // Cleveland at 38% and the 51st, which is the contrast the card exists to make possible.
    await page.goto("/district/044933");
    const card = page.locator('.card[data-part="national"]');
    await expect(card).toContainText("national top fifth");
  });

  test("Ohio carries a mark in the chart built to show Ohio's position", async ({ page }) => {
    /*
     * The statewide card ranks the states by local share and draws Ohio among the six highest. It
     * set `current: true` on Ohio's bar and nothing read it — `Bar` did not declare the field and
     * `barSpec` ignored it — so a reader had to find Ohio by reading the category names, which is
     * the work the chart was drawn to save.
     *
     * Two channels, because one of them is colour. The weight is the one that survives a
     * monochrome print, which is the rule the print stylesheet applies to every other mark here.
     */
    await page.goto("/statewide");
    const chart = page.locator('[data-chart="local-share"] svg.plot:visible');
    // Plot hoists constants onto the mark's group, which is why the subject is a group of its own.
    const marked = chart.locator("g.bar-label.current");
    await expect(marked).toHaveAttribute("font-weight", "600");
    await expect(marked.locator("text")).toHaveText(["Ohio"]);
    // And the rest of the states are still in the plain group, at the plain weight.
    const plain = chart.locator("g.bar-label:not(.current)");
    expect(await plain.locator("text").count()).toBeGreaterThan(3);
    await expect(plain).not.toHaveAttribute("font-weight", "600");
  });

  test("the one district outside the comparable set says so rather than showing a rank", async ({
    page,
  }) => {
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const outside = feed.districts.find((d: { national: unknown }) => d.national === null);
    expect(outside, "one Ohio K-8 district is outside the comparable set").toBeTruthy();
    await page.goto(`/district/${outside.irn}`);
    await expect(page.locator('.card[data-part="national"]')).toContainText(
      "not in the national comparison",
    );
  });

  for (const route of ["/district/043802", "/district/043802/finances"]) {
    test(`a chart on ${route} names the unit its labels are written in`, async ({ page }) => {
      /*
       * #611: the base-cost chart said "dollars, $353,814,437 in total" over bars labelled in
       * percent, and the categoricals chart said "in dollars" over the same. A chart whose labels
       * are percentages says what they are a share of.
       */
      await page.goto(route);
      const charts = page.locator(".chartwrap svg.plot:visible");
      await expect(charts.locator("g.bar-value text").first()).toBeAttached();
      let shares = 0;
      for (const svg of await charts.all()) {
        const values = await svg.locator("g.bar-value text").allTextContents();
        if (!values.some((v) => v.includes("%"))) continue;
        shares++;
        const label = (await svg.getAttribute("aria-label")) ?? "";
        expect(label, `labelled in percent: ${values.join(" ")}`).toMatch(/\bshare\b/);
        expect(label).not.toMatch(/\bin dollars\b|, dollars,/);
      }
      expect(shares).toBeGreaterThan(0);
    });
  }
});
