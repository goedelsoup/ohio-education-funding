/**
 * The projection: its fan, what the scenario holds fixed, what the band held, and the bias note.
 *
 * Four blocks that are one subject read at four depths — the drawing, the levers around it, the
 * realised coverage of the interval, and the sentence that says which way the method has been
 * wrong. A forecast page that draws a band without any of the last three is a page asserting more
 * than it knows.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND } from "./helpers.ts";

test.describe("the projection", () => {
  test("leads with the range and demotes the point to a footnote", async ({ page }) => {
    await page.goto("/scenario");
    const headline = page.locator("#projection-out .tile.wide");
    await expect(headline.locator(".v")).toContainText("–");
    await expect(headline.locator(".n")).toContainText("One path through the band, not the answer");
  });

  test("draws a band with both bounds labelled and the centre dashed", async ({ page }) => {
    await page.goto("/scenario");
    const chart = page.locator('#projection-out [data-chart="fan"] svg.plot:visible');
    await expect(chart.locator(".fan-band")).toHaveCount(1);
    await expect(chart.locator(".fan-mid")).toHaveCount(1);
    await expect(chart.locator(".fan-mid")).toHaveAttribute("stroke-dasharray", /\d/);
    await expect(chart.locator(".fan-edge")).toHaveCount(2);
    // Two bound labels, and no label on the central estimate.
    await expect(chart.locator(".fan-bound text")).toHaveCount(2);
  });

  test("says on its face that the axis is truncated", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator('#projection-out [data-chart="fan"] svg.plot:visible')).toContainText(
      "not zero",
    );
  });

  test("says it is a forecast and that the card above it is not", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator("#projection-out")).toContainText("This is a <strong>forecast".replace("<strong>", ""));
    await expect(page.locator("#projection-out")).toContainText("the card above it is not");
    // No figure anywhere adds the simulation and the forecast together.
    await expect(page.locator("#projection-out")).not.toContainText("combined");
  });

  test("the horizon control turns the band off at the base year", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator("#lv-horizon")).toBeVisible();
    await page.locator("#lv-horizon").fill("2026");
    await page.locator("#lv-horizon").dispatchEvent("input");
    await expect(page.locator("#projection-out")).toContainText("Not projected");
    await expect(page).toHaveURL(/[?&]h=2026/);
  });
});

test.describe("what the scenario holds fixed", () => {
  /*
   * Both routes, and the district one is the route that most needed it.
   *
   * It renders the identical five levers and carried none of this: its own "What this does not
   * move" card names assessed valuation and enrollment — limits specific to that view — and said
   * nothing about the $812.5M of categoricals the base cost lever also scales. A test that checked
   * only `/scenario` is what let that stand, and it is the same scoping mistake the year-chip rule
   * records making.
   */
  for (const route of ["/scenario", `/scenario?d=${CLEVELAND}`]) {
    test(`the caveat is above the controls, not below the results — ${route}`, async ({ page }) => {
      /*
       * It is a limit on what the reader is about to do, not a footnote on what they got. The
       * department's own calculator warns that its statewide constants are not recalculated when a
       * district's data changes; that caveat is larger here, where every lever moves every district.
       */
      await page.goto(route);
      const caveat = page.locator('.card[data-part="held-fixed"]');
      await expect(caveat).toContainText("What these levers hold fixed");
      await expect(caveat).toContainText("not recalculated");
      // The four things it exists to name, none of which the district route said before.
      await expect(caveat).toContainText("DPIA");
      await expect(caveat).toContainText("targeted assistance");
      await expect(caveat).toContainText("Gifted");
      await expect(caveat).toContainText("preschool");

      // Above the controls in document order.
      const order = await page.evaluate(() => {
        const c = document.querySelector('[data-part="held-fixed"]');
        const controls = document.querySelector("#scenario-root");
        if (!c || !controls) return null;
        return c.compareDocumentPosition(controls) & Node.DOCUMENT_POSITION_FOLLOWING
          ? "before"
          : "after";
      });
      expect(order).toBe("before");
    });
  }

  test("the two routes state the same caveat rather than two drifting copies", async ({ page }) => {
    // One component, so this is a property of the arrangement rather than of the prose. It fails
    // the moment somebody copies the markup back into a page to reword it there.
    await page.goto("/scenario");
    const statewide = (await page.locator('[data-part="held-fixed"]').innerText()).replace(/\s+/g, " ");
    await page.goto(`/scenario?d=${CLEVELAND}`);
    const district = (await page.locator('[data-part="held-fixed"]').innerText()).replace(/\s+/g, " ");
    expect(district).toBe(statewide);
  });

  test("the divergence from the department's tool is stated, with its size", async ({ page }) => {
    /*
     * This used to assert that the page named an omission — $858.2m of categoricals denominated in
     * the average base cost per pupil, making a refresh understate itself by 24%. The omission was
     * closed: `base_cost_scale` now moves the $812.5m of it that sits inside foundation funding,
     * and the page's job changed from confessing a gap to declaring a deliberate divergence from
     * the department's own simulator. A caveat without a magnitude is unfalsifiable; so is a
     * divergence, and the magnitude is asserted here.
     *
     * The second figure used to be pinned as `$25.2M` and no longer is. That literal was one half
     * of a worked example the card typed rather than ran — `$113.0M of base cost aid and about
     * $25.2M more` — where the first number was `3.1% × base cost state share`, a proportional
     * product `policy.ts` explicitly rejects. This test was green over it the whole time, because
     * asserting a string the page also hard-codes checks that two literals match and nothing else.
     *
     * What replaces it is the check that could have failed: the card's own arithmetic against the
     * tile a reader sees when they put the slider where the card says. That is the disagreement
     * that was live — `$138.2M` in the card against `+$163.1M` in the tile.
     */
    await page.goto("/scenario");
    const caveat = page.locator('.card[data-part="held-fixed"]');
    await expect(caveat).toContainText("diverges from the department's on purpose");
    await expect(caveat).toContainText("$812.5M");
    await expect(caveat).toContainText("genuinely cancel");

    const worked = (await caveat.innerText()).replace(/\s+/g, " ");
    const stated =
      /The refresh (\S+) prices — [\d.]+% on base cost — delivers \$([\d.]+)M through base cost aid and \$([\d.]+)M/.exec(
        worked,
      );
    expect(stated, "the card states a worked example").not.toBeNull();
    const [, slug, throughBase, throughCategoricals] = stated!;

    /*
     * Opened through the draft rather than through `?base=`, and that is not a detail.
     *
     * The refresh is `1.0395` and the base-cost slider steps by `0.01`, so `?base=1.0395` is
     * sanitised to `1.04` by the control before `readLevers` ever sees it — a $3.2M difference on a
     * $220.5M figure, which is the quantization `boot()`'s `fromControls: false` path exists to
     * avoid. The draft path renders from the draft, so it is the one that can be compared against a
     * figure the card computed from the draft's own value.
     */
    await page.goto(`/scenario?draft=${slug}`);
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    await expect(page.locator('[data-part="draft-departed"]')).toHaveCount(0);
    const tile = (await page.locator("#scenario-out .tile .v").first().innerText()).trim();

    /*
     * The card's two channels are both foundation aid; the tile is total state support, which
     * since `[K]` entered the model is smaller by the formula transition supplement the refresh
     * reduces. Both are right and they are not the same quantity — so the assertion is the
     * relationship rather than equality, and the gap is named.
     */
    const sum = Number(throughBase) + Number(throughCategoricals);
    const shown = Number(tile.replace(/[^0-9.]/g, ""));
    expect(tile, "the tile is a gain").toMatch(/^\+\$/);
    expect(
      sum - shown,
      `card says $${sum.toFixed(1)}M of foundation aid, tile says ${tile} of total state support`,
    ).toBeCloseTo(13.0, 0);
  });

  test("the three things still held fixed are given three different reasons", async ({ page }) => {
    /*
     * The point of the rewrite. "Held fixed" was one bucket and is now three, and they are not
     * interchangeable: an index that really does cancel, a price that would move by its own amount
     * rather than this one, and a program that sits outside the page entirely. Collapsing them
     * back into a single hedge would lose the only part a reader can act on.
     */
    await page.goto("/scenario");
    const caveat = page.locator('.card[data-part="held-fixed"]');
    await expect(caveat).toContainText("three different reasons");
    await expect(caveat).toContainText("$1.89B");
    await expect(caveat).toContainText("guess wearing the shape of an identity");
    await expect(caveat).toContainText("$45.7M");
    await expect(caveat).toContainText("outside foundation funding");
  });
});

test.describe("what the forecast band actually held", () => {
  test("the band's coverage is drawn at every horizon, not asserted for the reader", async ({
    page,
  }) => {
    /*
     * `#forecast-range` used to argue this in prose: the band is backtested, and it holds. Two
     * lines are the check — every error with the origins pooled, and the same errors with each
     * origin's own mean removed — over every horizon the F-33 panel reaches.
     */
    await page.goto("/method");
    const chart = page.locator('#forecast-range [data-chart="band-coverage"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart.locator(".series-a path")).toHaveCount(1);
    await expect(chart.locator(".series-b path")).toHaveCount(1);
    /*
     * Thirteen hoverable columns, one per horizon, as an exact count rather than a floor: the
     * depth of the backtest is a claim the corpus states in words — "every horizon from one to
     * thirteen" — so a panel that grew or lost a year should fail here and be changed on purpose.
     */
    await expect(chart.locator(".series-hit [data-hover]")).toHaveCount(13);
  });

  test("the sag is drawn: one line holds the target and the other falls away from it", async ({
    page,
  }) => {
    /*
     * The finding is a shape, and the shape is two distances from one dashed rule. Both lines
     * cross the 68.3% a ±1σ band claims — neither sits to one side of it — and what separates
     * them is how far they fall under it: the cross-district line by two points at its deepest,
     * the pooled line by nearly nine, in the middle distance where neither end shows it.
     *
     * Measured off the painted geometry rather than the numbers in the paragraph, because a
     * cross-district line that drifted steadily away from the rule would satisfy every number
     * that paragraph states and would not be the finding.
     */
    await page.goto("/method");
    const box = await page
      .locator('#forecast-range [data-chart="band-coverage"] svg.plot:visible')
      .evaluate((svg) => {
        const span = (selector: string) => {
          const r = (svg.querySelector(selector) as SVGGraphicsElement).getBBox();
          return { top: r.y, bottom: r.y + r.height };
        };
        const rule = (svg.querySelector(".series-reference line") as SVGGraphicsElement).getBBox();
        return { pooled: span(".series-a path"), cross: span(".series-b path"), target: rule.y };
      });
    // SVG y grows downward, so a horizon that held more than the target is drawn above the rule.
    for (const [what, line] of [
      ["the pooled line", box.pooled],
      ["the cross-district line", box.cross],
    ] as const) {
      expect(line.top, `${what} rises past the target somewhere`).toBeLessThan(box.target);
      expect(line.bottom, `${what} falls under it somewhere, so it crosses`).toBeGreaterThan(
        box.target,
      );
    }
    // The one asymmetry, which is the whole of the claim: 8.9 points under against 2.2.
    expect(
      box.pooled.bottom - box.target,
      "the pooled line sags far further under the target than the flat one",
    ).toBeGreaterThan(3 * (box.cross.bottom - box.target));
  });
});

test.describe("the bias beside the band", () => {
  const PANELS = '#forecast-range [data-chart="projection-bias"] svg.plot:visible';

  test("the level is drawn beside the width, as two populations and four lines", async ({
    page,
  }) => {
    /*
     * The card above draws the band's *width* and holds. This draws its *level*, which is a
     * different question with a different answer, and it is two panels rather than one chart
     * because `seriesSpec` draws exactly two lines and there are four series: the mean district
     * and the state total, over the forecasts that stay clear of the school closures and over all
     * of them.
     *
     * The horizon counts are exact rather than floors for the reason the coverage test gives, and
     * because the *difference* between them is a claim: the pre-closure population stops four
     * years shallower, and the card says so in words.
     */
    await page.goto("/method");
    const panels = page.locator(PANELS);
    await expect(panels).toHaveCount(2);
    for (const at of [0, 1]) {
      await expect(panels.nth(at).locator(".series-a path")).toHaveCount(1);
      await expect(panels.nth(at).locator(".series-b path")).toHaveCount(1);
    }
    await expect(panels.nth(0).locator(".series-hit [data-hover]")).toHaveCount(9);
    await expect(panels.nth(1).locator(".series-hit [data-hover]")).toHaveCount(13);
    // And the two panels are labelled, because "which population" is the axis between them.
    const labels = await page.locator("#forecast-range .panel .panel-label").allInnerTexts();
    expect(labels).toHaveLength(2);
    expect(new Set(labels).size, "the two panels are told apart by their captions").toBe(2);
  });

  test("the two quantities do not change sign together, which is why there are two", async ({
    page,
  }) => {
    /*
     * The finding #445 asks for, measured off the paint rather than off the sentence.
     *
     * Both quantities start under the truth and end over it, so both cross zero — but not at the
     * same horizon. On the forecasts that stay clear of the closures the mean district turns
     * first and the state total does not turn until five years, and in the gap between them the
     * average district is already being over-forecast while the statewide figure is still low.
     * That is the whole reason a single correction cannot be right: it would have the wrong
     * *sign*, not merely the wrong size, for one of the two published quantities.
     *
     * Read as the x of each path's first point above the "No bias" rule, walking the drawn
     * geometry. A test that read the paragraph instead would pass on a chart drawing one line
     * twice.
     */
    await page.goto("/method");
    const crossings = await page
      .locator(PANELS)
      .first()
      .evaluate((svg) => {
        const rule = (svg.querySelector(".series-reference line") as SVGGraphicsElement).getBBox().y;
        /** The x at which a path first rises above the rule, or null if it never does. */
        const crosses = (selector: string): number | null => {
          const path = svg.querySelector(selector) as SVGPathElement;
          const total = path.getTotalLength();
          for (let at = 0; at <= total; at += total / 2000) {
            const point = path.getPointAtLength(at);
            // SVG y grows downward, so "above the rule" is a smaller y, and a smaller y is a
            // bias above zero: the quantity has turned from under the truth to over it.
            if (point.y < rule) return point.x;
          }
          return null;
        };
        return { meanDistrict: crosses(".series-a path"), total: crosses(".series-b path") };
      });

    expect(crossings.meanDistrict, "the mean district crosses zero somewhere").not.toBeNull();
    expect(crossings.total, "the state total crosses zero somewhere").not.toBeNull();
    expect(
      crossings.meanDistrict!,
      "the mean district turns positive before the state total does",
    ).toBeLessThan(crossings.total!);
  });

  test("the drift does not wash out: every line ends at its own furthest point", async ({
    page,
  }) => {
    /*
     * The other half of the shape. A bias that peaked in the middle distance and came back would
     * be a different finding — an artefact of one set of origins rather than a horizon effect —
     * and the card states the opposite in as many words. Measured as: the deepest drawn point of
     * each line is also the one furthest from the rule.
     */
    await page.goto("/method");
    for (const at of [0, 1]) {
      const ends = await page
        .locator(PANELS)
        .nth(at)
        .evaluate((svg) => {
          const rule = (svg.querySelector(".series-reference line") as SVGGraphicsElement).getBBox()
            .y;
          return [".series-a path", ".series-b path"].map((selector) => {
            const path = svg.querySelector(selector) as SVGPathElement;
            const total = path.getTotalLength();
            let worst = 0;
            for (let along = 0; along <= total; along += total / 2000) {
              worst = Math.max(worst, Math.abs(path.getPointAtLength(along).y - rule));
            }
            return { last: Math.abs(path.getPointAtLength(total).y - rule), worst };
          });
        });
      for (const [line, end] of ends.entries()) {
        // A pixel of slack: `getPointAtLength` walks a polyline in even steps and need not land
        // exactly on the last vertex.
        expect(
          end.last,
          `panel ${at}, line ${line}: the last point is the furthest from zero`,
        ).toBeGreaterThan(end.worst - 1);
      }
    }
  });
});
