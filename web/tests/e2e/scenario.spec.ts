/**
 * The two things that let a reader disagree with this site: the verification gate and the builder.
 *
 * Every number here is checkable against a published source, and the gate is where that is stated
 * per figure. The builder is where a reader moves a parameter and watches the consequence. They are
 * one file because the second is only trustworthy if the first is: a scenario that re-renders
 * against unverified arithmetic is a toy.
 *
 * On waiting for a lever, see `setGuarantee` in `helpers.ts` — a control touched before the runner
 * has loaded is an input, not a lever.
 */

import { expect, test } from "@playwright/test";

import { headingText, CLEVELAND, NORTHERN, setGuarantee } from "./helpers.ts";

test.describe("the verification gate", () => {
  test("reports agreement and enables the scenario builder", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator("#scenario-status")).toContainText(
      "Formula reproduced against 13 reference scenarios and 4 reference forecasts",
    );
    await expect(page.locator("#scenario-out .err")).toHaveCount(0);
    await expect(page.locator("#lv-guarantee")).toBeVisible();
  });

  test("disables the scenario builder when a checkpoint disagrees", async ({ page }) => {
    // Tamper with the panel in flight. If the page can be made to render a scenario off a panel
    // whose checkpoints do not reproduce, the gate is decorative — and the whole argument for
    // computing the formula twice rests on it not being.
    await page.route("**/data/panel.json", async (route) => {
      const response = await route.fetch();
      const panel = await response.json();
      panel.checkpoints[0].cost += 1_000_000;
      await route.fulfill({ json: panel });
    });

    await page.goto("/scenario");
    await expect(page.locator("#scenario-out .err h2")).toBeVisible();
    expect(await headingText(page.locator("#scenario-out .err h2"))).toBe(
      "The scenario builder is disabled",
    );
    await expect(page.locator("#scenario-out")).toContainText("current law");
    await expect(page.locator("#scenario-status")).toContainText("FAILED");
    // The controls are cleared too: a lever that cannot be trusted to compute should not invite
    // being moved.
    await expect(page.locator("#lv-guarantee")).toHaveCount(0);
  });

  test("the district scenario is gated by the same check", async ({ page }) => {
    await page.route("**/data/panel.json", async (route) => {
      const response = await route.fetch();
      const panel = await response.json();
      panel.checkpoints[0].gainers += 1;
      await route.fulfill({ json: panel });
    });

    await page.goto(`/district/${NORTHERN}/scenario`);
    await expect(page.locator("#scenario-out .err h2")).toBeVisible();
    expect(await headingText(page.locator("#scenario-out .err h2"))).toBe(
      "The scenario builder is disabled",
    );
  });

  test("refuses a panel on a different contract", async ({ page }) => {
    await page.route("**/data/panel.json", async (route) => {
      const response = await route.fetch();
      const panel = await response.json();
      panel.contract_version = "99.0.0";
      await route.fulfill({ json: panel });
    });
    await page.goto("/scenario");
    await expect(page.locator("#scenario-out")).toContainText("99.0.0");
  });

  test("the rest of the site is unaffected by a bad panel", async ({ page }) => {
    // The pages are baked, so a broken panel cannot reach them. This is the pay-off for moving
    // the gate to build time: one failure mode, contained to one route.
    await page.route("**/data/panel.json", (route) => route.fulfill({ status: 500, body: "no" }));
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator(".err")).toHaveCount(0);
    await expect(page.locator(".tile .v").first()).not.toBeEmpty();
  });
});

test.describe("the scenario builder", () => {
  test("current law moves nothing and says so", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator("#scenario-out")).toContainText("Current law");
    await expect(page.locator("#scenario-out")).toContainText("nothing moves");
  });

  test("removing the guarantee reaches only the districts its backstop does not catch", async ({
    page,
  }) => {
    /*
     * This asserted 294 — every guaranteed district — and that was true of a model with no `[K]`
     * in it. The formula transition supplement tops a district up to its FY2021 base from a total
     * that already contains the guarantee, so retiring the guarantee is a shortfall `[K]` makes
     * good for 127 of the 294. See `project::hold_harmless`.
     */
    await page.goto("/scenario");
    await setGuarantee(page, "removed");
    const tiles = page.locator("#scenario-out .tile");
    await expect(tiles.nth(1).locator(".v")).toHaveText("167");
    await expect(tiles.nth(1).locator(".n")).toHaveText("0 up, 167 down");
  });

  test("the tiles stay under the levers and the rest goes below the forecast", async ({ page }) => {
    // The layout this page is arranged around: three headline numbers where a reader who has just
    // moved a lever is looking, the forecast next, and the reading-at-rest material under it.
    await page.goto("/scenario");
    await setGuarantee(page, "removed");

    await expect(page.locator("#scenario-out .tile")).toHaveCount(3);
    await expect(page.locator("#scenario-detail")).toContainText("How the change is distributed");
    await expect(page.locator("#scenario-detail")).toContainText("Most affected");
    await expect(page.locator("#scenario-detail")).toContainText("What moved underneath");

    // And they are in that order in the document, not merely present. `compareDocumentPosition`
    // returns 4 — DOCUMENT_POSITION_FOLLOWING — when the argument comes after the receiver.
    const order = await page.evaluate(() => {
      const at = (id: string) => document.getElementById(id)!;
      return [
        at("scenario-out").compareDocumentPosition(at("projection-out")),
        at("projection-out").compareDocumentPosition(at("scenario-detail")),
      ];
    });
    expect(order).toEqual([4, 4]);
  });

  test("the material below the forecast does not outlive the levers that produced it", async ({
    page,
  }) => {
    // The failure the split introduces if the container is only ever written and never cleared:
    // a "Most affected" table sitting below the fan chart, describing a scenario the controls no
    // longer hold, where a reader has to scroll past a forecast to discover it.
    await page.goto("/scenario");
    await setGuarantee(page, "removed");
    await expect(page.locator("#scenario-detail")).toContainText("Most affected");
    await page.locator("#scenario-reset").click();
    await expect(page.locator("#scenario-out")).toContainText("Current law");
    await expect(page.locator("#scenario-detail")).toBeEmpty();
  });

  test("the retained-share slider hides for the rules that do not take one", async ({ page }) => {
    await page.goto("/scenario");
    const lever = page.locator("#lv-arg").locator("xpath=ancestor::div[@class='lever']");
    await expect(lever).toBeHidden();
    await setGuarantee(page, "phase-out");
    await expect(lever).toBeVisible();
    await setGuarantee(page, "removed");
    await expect(lever).toBeHidden();
  });

  test("moving a lever writes it into the query string", async ({ page }) => {
    await page.goto("/scenario");
    await setGuarantee(page, "phase-out");
    await expect(page).toHaveURL(/[?&]g=phase-out/);
  });

  test("a shared scenario link arrives with its levers already set", async ({ page }) => {
    await page.goto("/scenario?g=rebase&arg=0.9&base=1.05&min=0.15&pb=1&pc=1&h=2032");
    await expect(page.locator("#lv-guarantee")).toHaveValue("rebase");
    await expect(page.locator("#lv-arg")).toHaveValue("0.9");
    await expect(page.locator("#lv-base")).toHaveValue("1.05");
    await expect(page.locator("#lv-min")).toHaveValue("0.15");
  });

  /*
   * The three readings of Section 265.225, reachable from the control rather than only from the
   * crate.
   *
   * This is #490 as a reader meets it: the corpus node for the phase-out scenario reports all three
   * columns, and for as long as the select offered two of them a reader who moved the node's own
   * lever could not reproduce the middle one. The figures are asserted rather than merely their
   * order, because the order is the uninteresting half — the point is that a reading of four
   * sentences of uncodified law is worth $863M, and which reading a reader is looking at is a thing
   * the page has to let them choose and then say.
   */
  test("the backstop's three readings price the same removal three ways", async ({ page }) => {
    await page.goto("/scenario");
    await setGuarantee(page, "removed");
    const stateAid = page
      .locator("#outcome .tile")
      .filter({ hasText: "State aid" })
      .locator(".v");
    await expect(stateAid).toHaveText("−$79.8M");

    for (const [reading, figure] of [
      ["rebased", "−$253.6M"],
      ["repealed", "−$942.5M"],
      ["as-enacted", "−$79.8M"],
    ] as const) {
      await page.selectOption("#lv-backstop", reading);
      await expect(stateAid).toHaveText(figure);
      // And it is sendable, which is the other half of being reachable.
      await expect(page).toHaveURL(new RegExp(`[?&]bs=${reading}`));
    }
  });

  test("a shared link carries the reading of the section it was read under", async ({ page }) => {
    await page.goto("/scenario?g=removed&bs=rebased");
    await expect(page.locator("#lv-backstop")).toHaveValue("rebased");
    await expect(
      page.locator("#outcome .tile").filter({ hasText: "Unmoved" }).locator(".v"),
    ).toHaveText("348");
  });

  test("reset returns to current law, controls and all", async ({ page }) => {
    await page.goto("/scenario");
    await setGuarantee(page, "removed");
    await page.locator("#lv-base").fill("1.2");
    await page.locator("#lv-base").dispatchEvent("input");
    await page.selectOption("#lv-backstop", "repealed");
    await page.locator("#scenario-reset").click();
    await expect(page.locator("#lv-guarantee")).toHaveValue("as-enacted");
    await expect(page.locator("#lv-backstop")).toHaveValue("as-enacted");
    await expect(page.locator("#lv-base")).toHaveValue("1");
    await expect(page.locator("#scenario-out")).toContainText("Current law");
  });

  test("the district scenario names its own change and the statewide count", async ({ page }) => {
    // The design rule this page exists for: a change that helps this district is not thereby a
    // good change, and the number of districts it hurts is in the same view, not behind a link.
    await page.goto(`/district/${NORTHERN}/scenario?g=removed&arg=0.5&base=1&min=0.1&pb=1&pc=1&h=2026`);
    await expect(page.locator("#scenario-out")).toContainText("State aid under this scenario");
    await expect(page.locator("#scenario-out")).toContainText("And to everyone else");
    await expect(page.locator("#scenario-out")).toContainText("294");
    await expect(page.locator("#scenario-out")).toContainText("moves the district onto the formula");
  });

  /*
   * The `?draft=` path renders before the first read of a control, on purpose — a draft's lever
   * values are off the slider's step grid. That made it the one path a query string reached
   * unvalidated, because the range inputs were doing the clamping everywhere else.
   *
   * Both of these were live: the first locked the tab, the second put four readings of one scenario
   * on screen at once. They are pinned here rather than in the unit suite because what went wrong
   * is the *composition* of two entry points, and only a browser has both.
   */
  test("a draft composed with an absurd horizon still renders", async ({ page }) => {
    await page.goto("/scenario?draft=hb-96-with-refreshed-inputs&h=999999");
    // Before the clamp this never resolved: `forecastPath` walked a million fiscal years, each one
    // a projection plus three formula runs over 609 districts.
    await expect(page.locator("#projection-out .tile").first()).toBeVisible({ timeout: 15_000 });
    await expect(page.locator("#lv-horizon")).toHaveValue("2036");
    await expect(page).toHaveURL(/[?&]h=2036/);
  });

  test("a draft composed with an out-of-range lever agrees with its own control", async ({
    page,
  }) => {
    await page.goto("/scenario?draft=hb-96-with-refreshed-inputs&base=100");
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);

    // The three that used to disagree: the figure was computed at 100, the slider showed 1.3, and
    // the label read +9900%.
    await expect(page.locator("#lv-base")).toHaveValue("1.3");
    await expect(page.locator("#lv-base-out")).toHaveText("+30%");
    await expect(page).toHaveURL(/[?&]base=1\.3(&|$)/);

    /*
     * And the figure is the bound's, not the request's. Asserted against the same URL written with
     * the bound rather than against a literal: `+$2.31B` at the top of the slider is a real number
     * that will move when the feed does, and pinning it here would make this test fail for the
     * wrong reason. What has to hold is that 100 and 1.3 are the same scenario — before the clamp,
     * 100 rendered `+$928.15B`.
     */
    const clamped = await page.locator("#scenario-out .tile .v").first().textContent();
    await page.goto("/scenario?draft=hb-96-with-refreshed-inputs&base=1.3");
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    await expect(page.locator("#scenario-out .tile .v").first()).toHaveText(clamped!);
  });

  /*
   * The result is a card with a heading, and every heading on the page wears its anchor the same
   * way. Both were properties of the *client-rendered* half, which no build-time pass can see.
   */
  for (const route of ["/scenario", `/district/${CLEVELAND}/scenario`]) {
    test(`the result has a heading, and it is the answer — ${route}`, async ({ page }) => {
      /*
       * Under current law there was one, "Current law". Move a lever and the headline became a bare
       * `<div class="tiles">`: heading navigation went from the controls straight past "State aid
       * +$288.1M, 366 districts reached" to the forecast, and the one block a reader came for had no
       * place in the outline. axe reports nothing — no rule requires a content block to be headed.
       */
      await page.goto(route);
      await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
      await page.locator("#lv-base").fill("1.05");
      await page.locator("#lv-base").dispatchEvent("input");

      const result = page.locator('[data-part="outcome"]');
      await expect(result).toBeVisible();
      await expect(result.locator("h2")).toHaveCount(1);
      // The tiles are inside it rather than beside it, which is what makes the heading theirs.
      await expect(result.locator(".tile")).toHaveCount(3);

      // And it comes before everything the result is elaborated by.
      const outline = await page
        .locator("main h2")
        .evaluateAll((ns) => ns.map((n) => n.textContent!.replace(/\s+/g, " ").trim()));
      const result_at = outline.findIndex((h) => /What (these levers do|this would do)/.test(h));
      expect(result_at, `no result heading in ${JSON.stringify(outline)}`).toBeGreaterThan(-1);
      expect(outline.slice(result_at + 1).join(" ")).not.toContain("Levers");
    });

    test(`every heading wears its anchor the same way — ${route}`, async ({ page }) => {
      /*
       * `moveAnchors` relocates `a.section-anchor` after the title at build time, and it sees
       * server-rendered markup only. The scenario routes build headings in the browser from the
       * same `anchor()` helper, so `/scenario` shipped `Levers #` and `What these levers hold fixed
       * … #` beside `# At projected enrollment`, `# Most affected` and `# What moved underneath` —
       * the transform's input and its output on one screen.
       */
      await page.goto(route);
      await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
      await page.locator("#lv-base").fill("1.05");
      await page.locator("#lv-base").dispatchEvent("input");
      await expect(page.locator('[data-part="outcome"]')).toBeVisible();

      const positions = await page.locator("main h2").evaluateAll((ns) =>
        ns
          .map((n) => {
            const a = n.querySelector("a.section-anchor");
            if (!a) return null;
            // Read off the words rather than the child list: since #549 the anchor sits inside
            // `.heading-text > .heading-tail`, so it is never a direct child either way and a
            // child-index check would pass on every heading.
            const text = n.textContent!.replace(/\s+/g, " ").trim();
            return { heading: text.slice(0, 44), first: text.startsWith("#") };
          })
          .filter((x): x is { heading: string; first: boolean } => x != null),
      );
      expect(positions.length).toBeGreaterThan(3);
      expect(
        positions.filter((p) => p.first),
        "these headings open with their own address; the rest do not",
      ).toEqual([]);
    });
  }

  test("the district route mints no horizon it cannot set", async ({ page }) => {
    // It has no horizon control, draws no band, and reads nothing from `h` — so every URL it minted
    // carried `h=2032` for a reader to copy and send.
    await page.goto(`/district/${CLEVELAND}/scenario`);
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    await page.locator("#lv-base").fill("1.05");
    await page.locator("#lv-base").dispatchEvent("input");
    await expect(page).toHaveURL(/[?&]base=1\.05/);
    expect(new URL(page.url()).searchParams.has("h")).toBe(false);

    // And the statewide route, which does have one, still carries it.
    await page.goto("/scenario");
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    await page.locator("#lv-base").fill("1.05");
    await page.locator("#lv-base").dispatchEvent("input");
    await expect(page).toHaveURL(/[?&]h=\d{4}/);
  });

  test("a cost is not rendered as a gain", async ({ page }) => {
    // The gain/loss classes mean "more aid" and "less aid", which is what the tiles are about.
    // Under a row headed *cost* the same green rendered a billion dollars of spending as a win.
    await page.goto(`/district/${CLEVELAND}/scenario?g=as-enacted&arg=0.5&base=1.15&min=0.1&pb=1&pc=1`);
    const row = page.locator('[data-part="moved-elsewhere"] tr', { hasText: "Cost to the state" });
    await expect(row.locator("td")).not.toHaveClass(/gain|loss/);
    await expect(row.locator("td")).toContainText("+$");
  });

  test("the interruption holds its fire when nothing is lost", async ({ page }) => {
    /*
     * "A change that helps this district is not thereby a good change, and the count above is the
     * reason: 0 districts receive less" — the card exists to interrupt self-interest, and delivered
     * the interruption with a zero attached.
     */
    await page.goto(`/district/${CLEVELAND}/scenario?g=as-enacted&arg=0.5&base=1.15&min=0.1&pb=1&pc=1`);
    const card = page.locator('[data-part="moved-elsewhere"]');
    await expect(card.locator("tr", { hasText: "Down" }).locator("td")).toHaveText("0");
    await expect(card).toContainText("No district receives less");
    await expect(card).not.toContainText("0 districts receive less");
    // The statewide view is still offered either way.
    await expect(card).toContainText("The distribution across all");
  });

  test("a failed gate leaves no card describing controls that are gone", async ({ page }) => {
    // Emptying the form left `#levers` standing with nothing in it but its own heading.
    await page.route("**/data/panel.json", async (route) => {
      const response = await route.fetch();
      const panel = await response.json();
      panel.checkpoints[0].cost += 1e6;
      await route.fulfill({ json: panel });
    });
    await page.goto("/scenario");
    await expect(page.locator("#scenario-out .err")).toBeVisible();
    await expect(page.locator("#levers")).toHaveCount(0);
    await expect(page.locator("#scenario-status")).toContainText("FAILED");
  });

  test("a draft's own off-grid value survives the clamp", async ({ page }) => {
    // The clamp enforces the ends and not the step, and this is why: `1.0395` is what the bill
    // prices, the slider steps by 0.01, and snapping would put a number that is not the bill's
    // under a banner saying it is.
    await page.goto("/scenario?draft=hb-96-with-refreshed-inputs");
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    await expect(page.locator('[data-part="draft-departed"]')).toHaveCount(0);
    await expect(page.locator("#scenario-out .tile .v").first()).toHaveText("+$207.5M");
  });
});
