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
    expect(bundle.contract_version).toBe("2.0.0");
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

    // The central invariant of this page, as the footer states it.
    await expect(page.locator("#verified")).toHaveText(
      "Formula verified against 8 reference scenarios",
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
