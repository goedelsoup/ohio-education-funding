/**
 * The page, in a browser, against the built site.
 *
 * The unit suite proves the formula is right. This suite proves the things that only exist once
 * a browser has run the code: that the feed actually loads over HTTP from `dist/`, that the
 * verification gate is wired to what it claims to gate, and that a link to a view opens that
 * view — which is the property the three tabs were built around.
 */

import { expect, test } from "@playwright/test";

/** Cleveland Municipal. On the guarantee, so the guarantee copy has something to render. */
const CLEVELAND = "043786";

test.describe("boot", () => {
  test("loads the feed and renders a district without an error card", async ({ page }) => {
    const failures: string[] = [];
    page.on("pageerror", (error) => failures.push(error.message));

    await page.goto("/");

    // The picker is written from the feed; "Loading…" is the inert shell.
    await expect(page.locator("#pick option")).toHaveCount(609);
    await expect(page.locator("#district-out .tile")).toHaveCount(3);
    await expect(page.locator("#district-out .err")).toHaveCount(0);
    expect(failures, "the page threw while booting").toEqual([]);
  });

  test("serves the feed as a static file next to the page", async ({ request }) => {
    // The feed is deliberately outside the JS bundle so regenerating it is not a rebuild.
    const response = await request.get("/data/bundle.json");
    expect(response.status()).toBe(200);
    const bundle = await response.json();
    expect(bundle.contract_version).toBe("4.0.0");
    expect(bundle.districts).toHaveLength(609);
  });

  test("states the provenance of the figures", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#prov")).toContainText("FY27");
  });
});

test.describe("the verification gate", () => {
  test("reports agreement with the Rust checkpoints and enables the scenario builder", async ({
    page,
  }) => {
    await page.goto("/");

    // The central invariant of this page, as the footer states it. Both halves are counted:
    // the simulation checkpoints and the forecasts, which are gated separately.
    await expect(page.locator("#verified")).toHaveText(
      "Formula verified against 8 reference scenarios and 4 reference forecasts",
    );
    await expect(page.locator("#verified")).not.toHaveClass(/err/);

    await page.getByRole("tab", { name: "Scenario" }).click();
    await expect(page.locator("#lv-guarantee")).toBeVisible();
    await expect(page.locator("#scenario-out .err")).toHaveCount(0);
  });

  test("disables the scenario builder when a checkpoint disagrees", async ({ page }) => {
    // Tamper with the feed in flight. If the page can be made to render a scenario off a feed
    // whose checkpoints do not reproduce, the gate is decorative — and the whole argument for
    // computing the formula twice rests on it not being.
    await page.route("**/data/bundle.json", async (route) => {
      const response = await route.fetch();
      const bundle = await response.json();
      bundle.checkpoints[1].cost += 1_000_000;
      await route.fulfill({ response, json: bundle });
    });

    await page.goto("/");

    await expect(page.locator("#verified")).toHaveText(/FAILED/);
    await expect(page.locator("#verified")).toHaveClass(/err/);

    await page.getByRole("tab", { name: "Scenario" }).click();
    await expect(page.locator("#scenario-out .err")).toContainText(
      "The scenario builder is disabled",
    );
    await expect(page.locator("#scenario-out")).toContainText("guarantee removed");
    // The controls are gone, not merely greyed: there is nothing to move.
    await expect(page.locator("#lv-guarantee")).toHaveCount(0);
    // The published-figure views read the feed rather than recomputing it, so they still work.
    await page.getByRole("tab", { name: "District" }).click();
    await expect(page.locator("#district-out .tile")).toHaveCount(3);
  });

  test("refuses to render a feed on a different contract", async ({ page }) => {
    await page.route("**/data/bundle.json", async (route) => {
      const response = await route.fetch();
      const bundle = await response.json();
      bundle.contract_version = "99.0.0";
      await route.fulfill({ response, json: bundle });
    });

    await page.goto("/");
    await expect(page.locator("#district-out .err")).toContainText("99.0.0");
    await expect(page.locator("#district-out .tile")).toHaveCount(0);
  });

  test("explains itself when the feed is missing", async ({ page }) => {
    await page.route("**/data/bundle.json", (route) => route.fulfill({ status: 404 }));
    await page.goto("/");
    await expect(page.locator("#district-out .err")).toContainText("Could not load");
  });
});

test.describe("tabs and links", () => {
  test("switching tabs moves the fragment and the selected panel together", async ({ page }) => {
    await page.goto("/");

    await page.getByRole("tab", { name: "Statewide" }).click();
    await expect(page).toHaveURL(/#statewide$/);
    await expect(page.locator("[data-panel=statewide]")).toBeVisible();
    await expect(page.locator("[data-panel=district]")).toBeHidden();
    await expect(page.getByRole("tab", { name: "Statewide" })).toHaveAttribute(
      "aria-selected",
      "true",
    );

    await page.getByRole("tab", { name: "District" }).click();
    await expect(page).toHaveURL(new RegExp(`#district/\\d{6}$`));
    await expect(page.locator("[data-panel=district]")).toBeVisible();
  });

  test("a link to one district opens that district", async ({ page }) => {
    await page.goto(`/#district/${CLEVELAND}`);
    await expect(page.locator("#pick")).toHaveValue(CLEVELAND);
    await expect(page.locator("#district-out")).toContainText("Guarantee");
  });

  test("a link to the statewide view opens it directly", async ({ page }) => {
    await page.goto("/#statewide");
    await expect(page.locator("[data-panel=statewide]")).toBeVisible();
    await expect(page.locator("#statewide-out")).toContainText("Who is on the guarantee");
    await expect(page.locator("#statewide-out svg.chart")).toBeVisible();
  });

  test("a shared scenario link arrives with its levers already set", async ({ page }) => {
    // The point of the tab: "here is what this proposal does" is something one person can send
    // another, and it has to arrive showing the proposal rather than current law.
    await page.goto("/#scenario?g=phase-out&arg=0.5&base=1.05&min=0.1&pb=1&pc=1");

    await expect(page.locator("#lv-guarantee")).toHaveValue("phase-out");
    await expect(page.locator("#lv-arg")).toHaveValue("0.5");
    await expect(page.locator("#lv-base")).toHaveValue("1.05");
    await expect(page.locator("#scenario-out")).not.toContainText("Current law");
    await expect(page.locator("#scenario-out")).toContainText("Districts reached");
  });

  test("selecting a district from the picker rewrites the fragment", async ({ page }) => {
    await page.goto("/");
    await page.locator("#pick").selectOption(CLEVELAND);
    await expect(page).toHaveURL(new RegExp(`#district/${CLEVELAND}$`));
  });
});

test.describe("the scenario builder", () => {
  test("current law moves nothing and says so", async ({ page }) => {
    await page.goto("/#scenario");
    await expect(page.locator("#scenario-out")).toContainText("Current law");
    await expect(page.locator("#scenario-out")).toContainText("nothing moves");
  });

  test("removing the guarantee reaches exactly the guaranteed districts", async ({ page }) => {
    await page.goto("/#scenario");
    await page.locator("#lv-guarantee").selectOption("removed");

    // 294 districts are on the guarantee; removing it can reach those and only those.
    const tiles = page.locator("#scenario-out .tile");
    await expect(tiles.filter({ hasText: "Districts reached" })).toContainText("294");
    await expect(tiles.filter({ hasText: "Districts reached" })).toContainText("0 up, 294 down");
    await expect(page.locator("#scenario-out svg.chart")).toBeVisible();
  });

  test("the retained-share slider hides for the rules that do not take one", async ({ page }) => {
    await page.goto("/#scenario");
    const retained = page.locator("#lv-arg").locator("xpath=ancestor::div[@class='lever']");

    await page.locator("#lv-guarantee").selectOption("as-enacted");
    await expect(retained).toBeHidden();
    await page.locator("#lv-guarantee").selectOption("phase-out");
    await expect(retained).toBeVisible();
    await page.locator("#lv-guarantee").selectOption("removed");
    await expect(retained).toBeHidden();
  });

  test("moving a lever writes it into the fragment", async ({ page }) => {
    await page.goto("/#scenario");
    await page.locator("#lv-guarantee").selectOption("rebase");
    await expect(page).toHaveURL(/g=rebase/);
    await expect(page).toHaveURL(/arg=/);
  });

  test("reset returns to current law, controls and all", async ({ page }) => {
    await page.goto("/#scenario?g=removed&base=1.2&min=0.25&pb=1&pc=1&arg=0.5");
    await expect(page.locator("#scenario-out")).toContainText("Districts reached");

    await page.getByRole("button", { name: "Reset to current law" }).click();

    await expect(page.locator("#lv-guarantee")).toHaveValue("as-enacted");
    await expect(page.locator("#lv-base")).toHaveValue("1");
    // The minimum resets to the model's own value, not to the slider's floor.
    await expect(page.locator("#lv-min")).toHaveValue("0.1");
    await expect(page.locator("#scenario-out")).toContainText("Current law");
  });
});

test.describe("the projection", () => {
  test("leads with the range and demotes the point to a footnote", async ({ page }) => {
    // The design rule this axis was blocked on, made checkable. Everywhere else on this page the
    // large number is a point estimate; here it must be an interval, or the interval becomes a
    // disclaimer the eye skips.
    await page.goto("/#scenario");
    const headline = page.locator("#projection-out .tile.wide");
    await expect(headline.locator(".v")).toContainText("–");
    await expect(headline.locator(".v")).toContainText("$");
    await expect(headline.locator(".n")).toContainText("Central estimate");
    await expect(headline.locator(".n")).toContainText("not the answer");
  });

  test("draws a band with both bounds labelled and the centre dashed", async ({ page }) => {
    await page.goto("/#scenario");
    const fan = page.locator('#projection-out [data-chart="fan"] svg');
    await expect(fan.locator("path.fan-band")).toHaveCount(1);
    await expect(fan.locator("polyline.fan-edge")).toHaveCount(2);
    await expect(fan.locator("polyline.fan-mid")).toHaveCount(1);
    await expect(fan.locator("text.fan-bound")).toHaveCount(2);
    // The truncated axis says so rather than leaving it to be noticed.
    await expect(fan.locator("text.axis-label", { hasText: "not zero" })).toHaveCount(1);
  });

  test("opens on the observed years rather than on the forecast", async ({ page }) => {
    // A band starting at a point with nothing to its left asks the reader to take that value on
    // faith and gives them no way to judge the trend it was fitted from.
    await page.goto("/#scenario");
    const fan = page.locator('#projection-out [data-chart="fan"] svg');
    await expect(fan.locator("polyline.fan-observed")).toHaveCount(1);
    await expect(fan.locator("circle.fan-anchor")).toHaveCount(1);
    // The axis opens on the first year the department published, not on the first forecast one.
    await expect(fan.locator("text.axis-label").first()).toHaveText("FY2024");
    await expect(page.locator("#projection-out .legend")).toContainText("Observed enrollment");

    // Hovering an observed year says it is exact; hovering a projected one gives a range.
    const tip = page.locator("#tip");
    await fan.locator("rect.fan-hit").first().hover();
    await expect(tip).toContainText("FY2024");
    await expect(tip).toContainText("exact");
    await fan.locator("rect.fan-hit").last().hover();
    await expect(tip).toContainText("–");
  });

  test("says it is a forecast and that the card above it is not", async ({ page }) => {
    // Simulation and projection are different epistemic acts and the page must not let them
    // blur. There is deliberately no figure anywhere that adds the two together.
    await page.goto("/#scenario");
    await expect(page.locator("#projection-out")).toContainText("This is a forecast");
    await expect(page.locator("#projection-out")).toContainText("the card above it is not");

    // Under current law the simulation card says only that nothing moves; move a lever and it
    // has to name itself as the deterministic half.
    await page.locator("#lv-guarantee").selectOption("removed");
    await expect(page.locator("#scenario-out")).toContainText("This is a simulation, not a forecast");

    // And the forecast card is always after the simulation, never merged into it.
    const order = await page.evaluate(() => {
      const scenario = document.querySelector("#scenario-out");
      const projection = document.querySelector("#projection-out");
      return scenario!.compareDocumentPosition(projection!) & Node.DOCUMENT_POSITION_FOLLOWING;
    });
    expect(order).toBeGreaterThan(0);
  });

  test("removing the guarantee visibly widens the state's exposure", async ({ page }) => {
    await page.goto("/#scenario");
    const width = page
      .locator("#projection-out .tile")
      .filter({ hasText: "Band half-width" })
      .locator(".v");
    const enacted = await width.textContent();

    await page.locator("#lv-guarantee").selectOption("removed");
    await expect(width).not.toHaveText(enacted ?? "");
    const removed = await width.textContent();

    const number = (s: string | null) => Number((s ?? "").replace(/[^0-9.]/g, ""));
    expect(number(removed)).toBeGreaterThan(number(enacted) * 1.5);
    // And the card names what it is being compared against, so the reader is not left to
    // remember the previous number.
    await expect(page.locator("#projection-out")).toContainText("current law at the same horizon");
  });

  test("the horizon is a control, carries into the link, and turns the band off", async ({ page }) => {
    await page.goto("/#scenario");
    await page.locator("#lv-horizon").fill("2034");
    await page.locator("#lv-horizon").dispatchEvent("input");
    await expect(page).toHaveURL(/h=2034/);
    await expect(page.locator("#projection-out")).toContainText("FY2034");

    // The base year is the off position: no forecast, said out loud rather than an empty card.
    await page.locator("#lv-horizon").fill("2026");
    await page.locator("#lv-horizon").dispatchEvent("input");
    await expect(page.locator("#projection-out")).toContainText("Not projected");
    await expect(page.locator('#projection-out [data-chart="fan"]')).toHaveCount(0);
  });

  test("a shared link opens on the horizon it was shared with", async ({ page }) => {
    await page.goto("/#scenario?g=as-enacted&arg=0.5&base=1&min=0.1&pb=1&pc=1&h=2036");
    await expect(page.locator("#lv-horizon")).toHaveValue("2036");
    await expect(page.locator("#projection-out .tile.wide .k")).toContainText("FY2036");
  });

  test("the district view compares enrollment years without inventing a published one", async ({
    page,
  }) => {
    // The department publishes one calculator at a time, so there is no FY2025 payment figure in
    // this repository. The card must say that rather than labelling a computed row as published.
    await page.goto(`/#district/${CLEVELAND}`);
    const card = page
      .locator("#district-out .card")
      .filter({ hasText: "What a year of enrollment is worth here" });
    await expect(card.locator("tbody tr")).toHaveCount(3);
    await expect(card).toContainText("FY2026 — the model's own");
    await expect(card).toContainText("are not published");
    await expect(card.locator('[data-chart="district-fan"] svg')).toHaveCount(1);
  });

  test("a guaranteed district's band is flat, and the page says why", async ({ page }) => {
    // Cleveland is on the guarantee, which pays a fixed dollar amount that enrollment does not
    // enter. A flat band is the finding, not a broken chart, so it is captioned as one.
    await page.goto(`/#district/${CLEVELAND}`);
    const card = page
      .locator("#district-out .card")
      .filter({ hasText: "What a year of enrollment is worth here" });
    await expect(card).toContainText("The band is flat");
    await expect(card).toContainText("does not respond to its enrollment");
    // One bound label, because both ends are the same number.
    await expect(card.locator("text.fan-bound")).toHaveCount(1);
  });

  test("the footer counts the forecasts it checked, not only the scenarios", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("#verified")).toContainText("reference forecasts");
    await expect(page.locator("#verified")).not.toHaveClass(/err/);
  });
});

test.describe("charts", () => {
  test("the hover layer follows the marks and survives a re-render", async ({ page }) => {
    await page.goto("/#statewide");
    const tip = page.locator("#tip");
    await expect(tip).toBeHidden();

    await page.locator("#statewide-out .bar-row").first().hover();
    await expect(tip).toBeVisible();
    await expect(tip).toContainText("on the guarantee");

    // Re-render under a different view without reloading; the listener is delegated from the
    // body once at boot, so the marks that replaced these ones have to work too.
    await page.getByRole("tab", { name: "Scenario" }).click();
    await page.locator("#lv-guarantee").selectOption("removed");
    await page.locator("#scenario-out .bar-row").first().hover();
    await expect(tip).toBeVisible();
    await expect(tip).toContainText("district");
  });

  test("the diverging histogram marks where zero is", async ({ page }) => {
    // A reader has to be able to see the midpoint rather than infer it from the hues.
    await page.goto("/#scenario");
    await page.locator("#lv-guarantee").selectOption("phase-out");
    await page.locator("#lv-base").fill("1.05");
    await page.locator("#lv-base").dispatchEvent("input");

    const histogram = page.locator("#scenario-out svg.chart");
    await expect(histogram.locator("line.zero")).toHaveCount(1);
    await expect(histogram.locator("text.axis-label", { hasText: "no change" })).toHaveCount(1);
  });
});

test.describe("presentation", () => {
  test("renders in dark mode without losing the series colours", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/#statewide");

    const fill = page.locator("#statewide-out .bar-fill").first();
    await expect(fill).toBeVisible();
    // Dark mode is a selected step from the same ramp, not an inversion — so the mark still has
    // to carry a colour rather than fall back to the page background.
    const colour = await fill.evaluate((el) => getComputedStyle(el).fill);
    expect(colour).not.toBe("rgba(0, 0, 0, 0)");
    expect(colour).not.toBe("none");
  });

  test("the page does not scroll sideways on a phone", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(`/#district/${CLEVELAND}`);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow, "the document scrolls horizontally").toBeLessThanOrEqual(0);
  });
});

test.describe("the outcome view", () => {
  test("shows the raw guarantee association and the controlled one on the same screen", async ({
    page,
  }) => {
    // The specific thing this view exists to prevent is a reader taking +0.187 as a finding.
    // If the controlled figure ever stops being rendered beside it, this fails.
    await page.goto("/#outcomes");
    const panel = page.locator('[data-panel="outcomes"]');
    await expect(panel).toBeVisible();
    await expect(panel).toContainText("+0.187");
    await expect(panel).toContainText("+0.035");
    await expect(panel).toContainText("holding poverty constant");
  });

  test("names both spending denominators rather than quoting one number", async ({ page }) => {
    await page.goto("/#outcomes");
    const panel = page.locator('[data-panel="outcomes"]');
    await expect(panel).toContainText("need-weighted");
    await expect(panel).toContainText("enrolled");
  });

  test("the poverty chart falls left to right", async ({ page }) => {
    await page.goto("/#outcomes");
    // toHaveCount auto-retries; allTextContents does not, and the panel is written by script
    // after the feed loads.
    const labels = page.locator('[data-chart="poverty-quintiles"] .bar-value');
    await expect(labels).toHaveCount(5);
    const numbers = (await labels.allTextContents()).map(Number);
    for (let i = 1; i < numbers.length; i++) {
      expect(numbers[i]!).toBeLessThan(numbers[i - 1]!);
    }
  });

  test("a district with no report card says so instead of showing blanks", async ({ page }) => {
    // Kelleys Island: in the funding model, absent from every outcome file.
    await page.goto("/#district/046797");
    await expect(page.locator("#district-out")).toContainText("No report card is published");
  });

  test("a district with a report card shows its three-year index", async ({ page }) => {
    await page.goto("/#district/049056");
    const out = page.locator("#district-out");
    await expect(out).toContainText("Performance Index, 2024-25");
    await expect(out).toContainText("Performance Index, 2022-23");
  });

  test("the outcome tab is linkable like the others", async ({ page }) => {
    await page.goto("/");
    await page.getByRole("tab", { name: "Outcomes" }).click();
    await expect(page).toHaveURL(/#outcomes$/);
  });
});
