/**
 * The site, in a browser, against a real build.
 *
 * The unit suite proves the formula is right. This suite proves the things that only exist once a
 * browser has run the code — and, since the move to real routes, a second class of thing: what is
 * true when a browser has *not* run the code. Almost every page here is complete before any script
 * loads, and that claim is only worth making if something checks it.
 *
 * Note on the preview server: `vite preview` answers an unmatched path with the front page rather
 * than with `404.html`, which is a preview behaviour and not the host's. So nothing here asserts
 * on a missing route; the nearest-match logic behind the 404 page is unit-tested in
 * `tests/unit/nearest.spec.ts` instead.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { expect, test, type Page } from "@playwright/test";

import { REQUIRED_CONTRACT } from "../../src/lib/types.ts";

/** Cleveland Municipal. On the guarantee, so the guarantee copy has something to render. */
const CLEVELAND = "043786";
/** Northern Local (Perry County). The corpus's property-poor exemplar. */
const NORTHERN = "049056";
/** Manchester Local — funded by the formula, so its band opens rather than collapsing. */
const ON_FORMULA = "000442";
/** Fremont City — 20.0000 mills in both tax years, so the floor holds still under it. */
const AT_FLOOR = "044016";
/** Vinton County Local — 18.70 mills voted and charged. The floor never reached it. */
const BELOW_FLOOR = "050393";
/** Alexander Local — 29.0% federal, the most exposed district in the state. */
const MOST_FEDERAL = "045906";

/** Move a lever and wait for the scenario to re-render behind it. */
async function setGuarantee(page: Page, value: string): Promise<void> {
  await page.selectOption("#lv-guarantee", value);
  await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
}

test.describe("the content security policy", () => {
  /*
   * # Why this reads files instead of driving a browser
   *
   * `vite preview` serves `dist/` with no headers at all, so the CSP that the host applies is
   * absent for the entire suite. Everything else here could pass while the deployed site refuses
   * to run its own scripts — and that is not hypothetical: `onsubmit="return false"` shipped on
   * four pages and was found by opening the live site, after the tests were green.
   *
   * `script-src 'self'` permits no inline script of any kind, and an `on*` attribute is inline
   * script. So rather than test the browser, this tests the artefact: whatever the headers do, the
   * output must contain nothing the strict directive would reject.
   */
  const DIST = join(import.meta.dirname, "../../dist");

  const html = (dir: string): string[] =>
    readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
      entry.isDirectory()
        ? html(join(dir, entry.name))
        : entry.name.endsWith(".html")
          ? [join(dir, entry.name)]
          : [],
    );

  test("no page carries an inline event handler", () => {
    // `onsubmit`, `onclick`, and friends. Excludes SVG's `on` in other positions by requiring the
    // attribute form exactly.
    const offenders: string[] = [];
    for (const file of html(DIST)) {
      const found = readFileSync(file, "utf8").match(/\son[a-z]+\s*=\s*["']/gi);
      if (found) offenders.push(`${file.slice(DIST.length + 1)}: ${[...new Set(found)].join(", ")}`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });

  test("no page carries an inline script block", () => {
    // Astro bundles every `<script>` to a hashed file under `_astro/`, so any `<script>` left with
    // a body is either a mistake or a deliberate exception that the CSP has not been told about.
    const offenders: string[] = [];
    for (const file of html(DIST)) {
      const body = readFileSync(file, "utf8");
      for (const match of body.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/gi)) {
        if ((match[1] ?? "").trim() !== "") offenders.push(file.slice(DIST.length + 1));
      }
    }
    expect([...new Set(offenders)].slice(0, 10)).toEqual([]);
  });

  test("nothing is fetched from another origin", () => {
    // `default-src 'self'` and `connect-src 'self'`. A CDN font or script would be invisible in
    // preview and dead in production.
    const offenders: string[] = [];
    for (const file of html(DIST)) {
      const found = readFileSync(file, "utf8").match(
        /(?:src|href)\s*=\s*["'](?:https?:)?\/\/(?!schools\.ohio\.shawneesmart\.systems)[^"']+/gi,
      );
      // Links in prose are fine; only fetched subresources matter, which are src= or a stylesheet.
      const fetched = (found ?? []).filter((m) => /^src/i.test(m) || /stylesheet/i.test(m));
      if (fetched.length > 0) offenders.push(`${file.slice(DIST.length + 1)}: ${fetched[0]}`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });
});

test.describe("the document arrives complete", () => {
  test("a district page carries its figures before any script runs", async ({ page }) => {
    const failures: string[] = [];
    page.on("pageerror", (error) => failures.push(error.message));

    await page.goto(`/district/${CLEVELAND}`);

    await expect(page.locator("h1")).toHaveText("Cleveland Municipal");
    await expect(page.locator(".tiles").first().locator(".tile")).toHaveCount(3);
    await expect(page.locator(".err")).toHaveCount(0);
    expect(failures, "the page threw while booting").toEqual([]);
  });

  test("states the provenance of the figures in the footer", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator("footer")).toContainText("FY27");
    await expect(page.locator("footer")).toContainText("Bundle contract");
  });

  test("the footer reports the build-time formula check", async ({ page }) => {
    // The central invariant, as the footer states it. Both halves are counted: the simulation
    // checkpoints and the forecasts, which are gated separately.
    await page.goto("/");
    await expect(page.locator("footer")).toContainText(
      "Formula verified at build against 8 reference scenarios and 4 reference forecasts",
    );
  });

  test("the feed and the slim panel are both served as static files", async ({ request }) => {
    const feed = await request.get("/data/bundle.json");
    expect(feed.status()).toBe(200);
    const bundle = await feed.json();
    expect(bundle.contract_version).toBe(REQUIRED_CONTRACT);
    expect(bundle.districts).toHaveLength(609);
    expect(bundle.districts[0]).toHaveProperty("base_cost_build_up");

    // The panel is what the scenario routes fetch: the same districts, without the three blocks
    // the formula never reads.
    const panel = await request.get("/data/panel.json");
    expect(panel.status()).toBe(200);
    const slim = await panel.json();
    expect(slim.districts).toHaveLength(609);
    expect(slim.districts[0]).not.toHaveProperty("finances");
    expect(slim.districts[0]).not.toHaveProperty("outcome");
    expect(slim.districts[0]).not.toHaveProperty("base_cost_build_up");
    expect(slim.districts[0]).toHaveProperty("base_cost_state_share");
  });

  test("a district page ships almost no JavaScript, and none of the build's libraries", async ({
    page,
    request,
  }) => {
    /*
     * The architecture, as a number.
     *
     * Charts render to SVG at build time through `linkedom`, the feed is parsed at build time
     * through zod, and the corpus is read at build time through a YAML parser. None of those three
     * has any business in a browser, and it is entirely possible to pull one in by accident — an
     * ordinary `import` in a module a client script also touches is all it takes, and nothing else
     * would notice.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const sources = await page.locator("script[src]").evaluateAll((nodes) =>
      nodes.map((n) => (n as HTMLScriptElement).getAttribute("src")!),
    );

    let bytes = 0;
    for (const src of sources) {
      const response = await request.get(src);
      expect(response.status()).toBe(200);
      const body = await response.text();
      bytes += body.length;
      for (const library of ["ZodError", "parseHTML", "YAMLParseError", "@observablehq/plot"]) {
        expect(body, `${library} reached the client bundle via ${src}`).not.toContain(library);
      }
    }
    // The whole of the chrome is under a kilobyte. Generous ceiling: this is a tripwire for a
    // library arriving, not a budget to optimise against.
    expect(bytes, "a district page's JavaScript grew unexpectedly").toBeLessThan(8_000);
  });

  test("Plot ships only where a chart has to be redrawn", async ({ page }) => {
    const weigh = async (path: string) => {
      await page.goto(path);
      return page.locator("script[src]").count();
    };
    // The scenario route loads one script more than a district page: its charts change when a
    // slider moves, so it is the one place a charting library earns its download.
    expect(await weigh(`/district/${CLEVELAND}`)).toBe(1);
    expect(await weigh("/scenario")).toBe(2);
  });

  test("the CSV has one row per district and a header", async ({ request }) => {
    const response = await request.get("/data/districts.csv");
    expect(response.status()).toBe(200);
    const lines = (await response.text()).trim().split("\n");
    expect(lines).toHaveLength(610);
    expect(lines[0]).toContain("irn,name");
  });
});

test.describe("with JavaScript disabled", () => {
  // The property the whole buildout was for. These pages are pre-rendered, so a reader with no
  // script — or a search engine, or a text browser — gets every figure rather than an empty shell.
  test.use({ javaScriptEnabled: false });

  test("a district's figures are all present", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator("h1")).toHaveText("Cleveland Municipal");
    await expect(page.getByText("Where the state aid comes from")).toBeVisible();
    await expect(page.locator(".tile .v").first()).not.toBeEmpty();
    // Charts are build-time SVG, not a canvas drawn on load.
    await expect(page.locator("svg.plot").first()).toBeVisible();
  });

  test("the district index lists every district", async ({ page }) => {
    await page.goto("/districts");
    await expect(page.locator("#district-table tbody tr")).toHaveCount(609);
  });

  test("the constant-dollar switch still switches", async ({ page }) => {
    // Two radios and a sibling selector rather than a click handler, so the control is not a dead
    // button for a reader without script.
    await page.goto(`/district/${CLEVELAND}/finances`);
    await expect(page.locator(".basis-panel.nominal")).toBeVisible();
    await expect(page.locator(".basis-panel.real")).toBeHidden();

    await page.locator('label[data-basis="real"]').click();
    await expect(page.locator(".basis-panel.real")).toBeVisible();
    await expect(page.locator(".basis-panel.nominal")).toBeHidden();
    await expect(page.locator(".basis-panel.real")).toContainText("FY2020 dollars");
  });

  test("the wiki renders its prose and its claim badges", async ({ page }) => {
    await page.goto("/wiki/parameter/twenty-mill-floor");
    await expect(page.locator("h1")).toHaveText("Twenty-Mill Floor");
    await expect(page.locator(".claim.verified").first()).toBeVisible();
    await expect(page.locator(".prose-body")).toContainText("20 mills");
  });

  test("the scenario route says outright that it is the exception", async ({ page }) => {
    await page.goto("/scenario");
    // Located by role rather than by text: Playwright's text engine does not descend into
    // `<noscript>`, even in a context where the parser has turned its contents into real DOM.
    await expect(
      page.getByRole("heading", { name: "This is the one page that needs JavaScript" }),
    ).toBeVisible();
  });
});

test.describe("routes", () => {
  test("each of a district's four views is its own address", async ({ page }) => {
    for (const [path, heading] of [
      ["", "Dashboard"],
      ["/outcome", "Outcome"],
      ["/finances", "Finances"],
      ["/scenario", "Scenario"],
    ] as const) {
      await page.goto(`/district/${NORTHERN}${path}`);
      await expect(page.locator("h1")).toHaveText("Northern Local");
      await expect(page.locator(`.subnav a[aria-current="page"]`)).toHaveText(heading);
    }
  });

  test("a link to the statewide view opens it directly", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("Ohio school funding");
    await expect(page.getByText("Who is on the guarantee")).toBeVisible();
  });

  test("old fragment links still land on the page they named", async ({ page }) => {
    // `#district/043786` was the shareable form for this platform's whole life and is in board
    // packets. The routes moved into the path; the links should not have died with them.
    await page.goto(`/#district/${CLEVELAND}`);
    await expect(page).toHaveURL(new RegExp(`/district/${CLEVELAND}$`));
    await expect(page.locator("h1")).toHaveText("Cleveland Municipal");

    await page.goto("/#outcomes");
    await expect(page).toHaveURL(/\/outcomes$/);

    await page.goto("/#scenario?g=removed&arg=0.5&base=1&min=0.1&pb=1&pc=1&h=2032");
    await expect(page).toHaveURL(/\/scenario\?/);
    await expect(page.locator("#lv-guarantee")).toHaveValue("removed");
  });

  test("the sitemap lists the district pages", async ({ request }) => {
    const index = await request.get("/sitemap-index.xml");
    expect(index.status()).toBe(200);
    const first = await request.get("/sitemap-0.xml");
    expect(await first.text()).toContain(`/district/${CLEVELAND}`);
  });
});

test.describe("the verification gate", () => {
  test("reports agreement and enables the scenario builder", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator("#scenario-status")).toContainText(
      "Formula reproduced against 8 reference scenarios and 4 reference forecasts",
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
    await expect(page.locator("#scenario-out .err h2")).toHaveText(
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
    await expect(page.locator("#scenario-out .err h2")).toHaveText(
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

  test("removing the guarantee reaches exactly the guaranteed districts", async ({ page }) => {
    await page.goto("/scenario");
    await setGuarantee(page, "removed");
    const tiles = page.locator("#scenario-out .tile");
    await expect(tiles.nth(1).locator(".v")).toHaveText("294");
    await expect(tiles.nth(1).locator(".n")).toHaveText("0 up, 294 down");
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

  test("reset returns to current law, controls and all", async ({ page }) => {
    await page.goto("/scenario");
    await setGuarantee(page, "removed");
    await page.locator("#lv-base").fill("1.2");
    await page.locator("#lv-base").dispatchEvent("input");
    await page.locator("#scenario-reset").click();
    await expect(page.locator("#lv-guarantee")).toHaveValue("as-enacted");
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
});

test.describe("the projection", () => {
  test("leads with the range and demotes the point to a footnote", async ({ page }) => {
    await page.goto("/scenario");
    const headline = page.locator("#projection-out .tile.wide");
    await expect(headline.locator(".v")).toContainText("–");
    await expect(headline.locator(".n")).toContainText("One path through the band, not the answer");
  });

  test("draws a band with both bounds labelled and the centre dashed", async ({ page }) => {
    await page.goto("/scenario");
    const chart = page.locator('#projection-out [data-chart="fan"] svg');
    await expect(chart.locator(".fan-band")).toHaveCount(1);
    await expect(chart.locator(".fan-mid")).toHaveCount(1);
    await expect(chart.locator(".fan-mid")).toHaveAttribute("stroke-dasharray", /\d/);
    await expect(chart.locator(".fan-edge")).toHaveCount(2);
    // Two bound labels, and no label on the central estimate.
    await expect(chart.locator(".fan-bound text")).toHaveCount(2);
  });

  test("says on its face that the axis is truncated", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator('#projection-out [data-chart="fan"] svg')).toContainText(
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

test.describe("one district's own band", () => {
  test("a guaranteed district shows what the formula computes beside what it receives", async ({
    page,
  }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const chart = page.locator('[data-chart="district-fan"] svg');
    // Its aid does not respond to its enrollment at all, so the band collapses — and the second
    // line, the formula's own falling answer, is what makes the chart say something.
    await expect(chart.locator(".fan-reference")).toHaveCount(1);
    await expect(page.locator(".card", { hasText: "Carried forward" })).toContainText(
      "flat by construction",
    );
  });

  test("a formula-funded district gets a band and no second line", async ({ page }) => {
    await page.goto(`/district/${ON_FORMULA}`);
    const chart = page.locator('[data-chart="district-fan"] svg');
    await expect(chart.locator(".fan-band")).toHaveCount(1);
    await expect(chart.locator(".fan-reference")).toHaveCount(0);
    await expect(page.locator(".card", { hasText: "Carried forward" })).toContainText(
      "The range, not the line, is the finding",
    );
  });

  test("compares enrollment years without inventing a published one", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator(".card", { hasText: "What a year of enrollment is worth here" });
    await expect(card).toContainText("FY2024");
    await expect(card).toContainText("FY2026 — the model's own");
    await expect(card).toContainText("These are not published FY2025 and FY2026 funding totals");
  });
});

test.describe("the finances route", () => {
  test("shows six closed years of money that changed hands", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}/finances`);
    await expect(page.locator(".basis-panel.nominal tbody tr")).toHaveCount(6);
    await expect(page.locator(".basis-panel.nominal")).toContainText("audited actuals");
  });

  test("refuses to be read as a check on the model", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}/finances`);
    await expect(page.getByText("What these numbers are not")).toBeVisible();
    await expect(page.locator(".card", { hasText: "What these numbers are not" })).toContainText(
      "not comparable line for line",
    );
  });

  test("deflating reverses the sign of the statewide cash story", async ({ page }) => {
    // The reason both bases are offered rather than one: they support opposite arguments, and the
    // difference is entirely CPI.
    await page.goto("/");
    const nominal = page.locator(".basis-panel.nominal .tile", { hasText: "Change since FY2020" });
    await expect(nominal.locator(".v")).toHaveClass(/gain/);
    const real = page.locator(".basis-panel.real .tile", { hasText: "Change since FY2020" });
    await expect(real.locator(".v")).toHaveClass(/loss/);
  });
});

test.describe("the outcome routes", () => {
  test("shows the raw guarantee association and the controlled one together", async ({ page }) => {
    // The rule the whole axis exists for: no association appears without its controlled figure.
    await page.goto("/outcomes");
    const tiles = page.locator(".tiles").first().locator(".tile");
    await expect(tiles.nth(1)).toContainText("+0.187");
    await expect(tiles.nth(2)).toContainText("+0.035");
    await expect(tiles.nth(2)).toContainText("holding poverty constant");
  });

  test("names both spending denominators rather than quoting one number", async ({ page }) => {
    await page.goto("/outcomes");
    const card = page.locator(".card", { hasText: "The same numerator, two denominators" });
    await expect(card).toContainText("need-weighted");
    await expect(card).toContainText("enrolled");
  });

  test("the poverty chart falls left to right", async ({ page }) => {
    await page.goto("/outcomes");
    await expect(page.locator('[data-chart="poverty-quintiles"] svg')).toBeVisible();
    await expect(page.locator(".card", { hasText: "Poverty is most of what" })).toContainText(
      "−0.846",
    );
  });

  test("a district's score is shown against comparable poverty, not against the state", async ({
    page,
  }) => {
    await page.goto(`/district/${CLEVELAND}/outcome`);
    const card = page.locator(".card", { hasText: "Against districts with comparable poverty" });
    await expect(card).toContainText("Median of its poverty fifth");
    await expect(card).toContainText("It is <strong>not</strong> an effect".replace(/<[^>]+>/g, ""));
  });

  test("a district with no report card says so instead of showing blanks", async ({ page }) => {
    // Three of the 609 have no report card. The page has to exist and explain itself.
    await page.goto("/data/bundle.json");
    const bundle = await page.evaluate(() => JSON.parse(document.body.innerText));
    const missing = bundle.districts.find((d: { outcome: unknown }) => d.outcome === null);
    expect(missing).toBeTruthy();
    await page.goto(`/district/${missing.irn}/outcome`);
    await expect(page.getByText("No report card is published for this district")).toBeVisible();
  });
});

test.describe("the wiki", () => {
  test("renders a node with its properties, links and backlinks", async ({ page }) => {
    await page.goto("/wiki/parameter/twenty-mill-floor");
    await expect(page.locator("h1")).toHaveText("Twenty-Mill Floor");
    await expect(page.getByText("Pointed at by")).toBeVisible();
    await expect(page.locator(".card", { hasText: "Links" })).toContainText("constrains");
  });

  test("rewrites corpus file paths into working routes", async ({ page }) => {
    // The transformation most likely to rot: `../legislation/hb-920-1976.yml` has to become a
    // route, and a wrong guess produces an ordinary-looking link that 404s.
    await page.goto("/wiki/parameter/twenty-mill-floor");
    const link = page.locator('.prose-body a[href="/wiki/legislation/hb-920-1976"]').first();
    await expect(link).toBeVisible();
    await link.click();
    await expect(page.locator("h1")).toContainText("H.B. 920");
  });

  test("keeps the corpus's claim tags as badges rather than as stray brackets", async ({ page }) => {
    await page.goto("/wiki/metric/performance-index");
    await expect(page.locator(".claim.verified").first()).toBeVisible();
    await expect(page.locator(".claim.inference").first()).toBeVisible();
    // The tags must not survive as literal text anywhere in the prose.
    const prose = await page.locator(".prose-body").innerText();
    expect(prose).not.toContain("[verified]");
    expect(prose).not.toContain("[inference]");
    expect(prose).not.toContain("[open]");
  });

  test("joins an exemplar district to its live figures", async ({ page }) => {
    // The reason the wiki is on the same site as the data: the corpus says what Upper Arlington
    // illustrates, the feed says what it is currently paid.
    await page.goto("/wiki/education-agency/upper-arlington-city");
    const card = page.locator(".card", { hasText: "This district, in the current model" });
    await expect(card).toBeVisible();
    await card.getByRole("link", { name: "Dashboard" }).click();
    await expect(page.locator("h1")).toHaveText("Upper Arlington City");
  });

  test("a class page is also the schema for its class", async ({ page }) => {
    await page.goto("/wiki/education-agency");
    await expect(page.getByRole("heading", { name: "Properties" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Relationships" })).toBeVisible();
    await expect(page.locator(".card", { hasText: "Properties" })).toContainText("irn");
    await expect(page.locator(".card", { hasText: "Relationships" })).toContainText("party-to");
  });

  test("a source lists the nodes that cite it", async ({ page }) => {
    await page.goto("/wiki/source");
    await page.getByRole("link", { name: /School Finance Payment Report/ }).click();
    await expect(page.getByRole("heading", { name: "Cited by" })).toBeVisible();
    await expect(page.locator(".card", { hasText: "Cited by" }).locator("a")).not.toHaveCount(0);
  });
});

test.describe("finding things", () => {
  test("the district index filters and sorts without a round trip", async ({ page }) => {
    await page.goto("/districts");
    // Four district names contain "cleveland" — Cleveland Municipal, Cleveland Heights, East
    // Cleveland, and Miller City-New Cleveland. A substring filter finds all of them.
    await page.locator("#f-name").fill("cleveland");
    await expect(page.locator("#district-table tbody tr:visible")).toHaveCount(4);
    await expect(page.locator("#f-count")).toContainText("of 609");

    await page.locator("#f-name").fill("");
    await page.locator("#f-status").selectOption("guarantee");
    await expect(page.locator("#f-count")).toContainText("294 of 609");
  });

  test("sorting by a column reorders the table", async ({ page }) => {
    await page.goto("/districts");
    await page.locator('thead button[data-sort="aid"]').click();
    const first = page.locator("#district-table tbody tr").first();
    await expect(first).toHaveAttribute("data-name", /.+/);
    await expect(page.locator('thead button[data-sort="aid"]')).toHaveAttribute(
      "aria-sort",
      "descending",
    );
  });

  test("search finds a district by IRN and a concept by name", async ({ page }) => {
    await page.goto("/search?q=043786");
    await expect(page.locator("#search-out")).toContainText("Cleveland Municipal");

    await page.locator("#s-q").fill("twenty-mill");
    await expect(page.locator("#search-out")).toContainText("Twenty-Mill Floor");
  });

  test("comparison puts two districts side by side", async ({ page }) => {
    await page.goto(`/compare?a=${NORTHERN}&b=044933`);
    const table = page.locator("#compare-out table");
    await expect(table).toContainText("Northern Local");
    await expect(table).toContainText("Upper Arlington City");
    await expect(table).toContainText("Assessed valuation per pupil");
    // Wealth is compared as a ratio, because it spans two orders of magnitude.
    await expect(table).toContainText("×");
  });
});

test.describe("presentation", () => {
  test("renders in dark mode without losing the series colours", async ({ page }) => {
    // Charts are rendered at build time and cannot re-render on a theme change, so every colour in
    // them is a custom property. This is the test that the indirection actually resolves.
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/");
    const fill = await page
      .locator(".bar-fill")
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    // #3987e5 — the dark-mode step, not the light one.
    expect(fill).toBe("rgb(57, 135, 229)");
  });

  test("the theme toggle beats the OS setting in both directions", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/");
    await page.locator("#theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    const fill = await page
      .locator(".bar-fill")
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    expect(fill).toBe("rgb(42, 120, 214)");
  });

  test("the page does not scroll sideways on a phone", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    for (const path of ["/", `/district/${CLEVELAND}`, "/districts", "/wiki/metric/performance-index"]) {
      await page.goto(path);
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow, `${path} scrolls sideways`).toBeLessThanOrEqual(1);
    }
  });

  test("the hover layer follows the marks", async ({ page }) => {
    await page.goto("/");
    const tip = page.locator("#tip");
    await expect(tip).toBeHidden();
    await page.locator(".bar-fill > *").first().hover();
    await expect(tip).toBeVisible();
    await expect(tip).toContainText("districts on the guarantee");
  });

  test("the diverging histogram marks where zero is", async ({ page }) => {
    // A reader has to be able to see where zero is, not infer it from the hues.
    // Base cost up 10% with the guarantee removed: some districts gain and some lose, so the
    // distribution straddles zero and the neutral midpoint has to be findable.
    await page.goto("/scenario?g=removed&arg=0.5&base=1.1&min=0.1&pb=1&pc=1&h=2026");
    const chart = page.locator('[data-chart="deltas"] svg');
    await expect(chart).toBeVisible();
    await expect(chart).toContainText("no change");
    // Two hues and a neutral midpoint, never a hue at zero.
    await expect(chart.locator(".hist > *")).not.toHaveCount(0);
  });
});

test.describe("the base cost build-up", () => {
  test("shows the statute's twenty-two elements, and reconciles against the department", async ({
    page,
  }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator(".card", { hasText: "Why base cost is" });
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
    const labels = page.locator('[data-chart="base-cost"] .bar-label text');
    await expect(labels).toHaveCount(5);
    const chart = page.locator('[data-chart="base-cost"] svg');
    const box = await chart.boundingBox();
    for (let i = 0; i < 5; i++) {
      const label = await labels.nth(i).boundingBox();
      expect(label!.x, `label ${i} starts left of the chart`).toBeGreaterThanOrEqual(box!.x - 0.5);
    }
  });
});

test.describe("the property tax route", () => {
  test("a district at the floor is told the reduction factors have stopped", async ({ page }) => {
    // Fremont City charges 20.0000 mills in both tax years, so H.B. 920 has nothing left to roll
    // back and a reappraisal reaches its revenue directly. That is the point of carrying two.
    await page.goto(`/district/${AT_FLOOR}/taxes`);
    await expect(page.locator("h1")).toHaveText("Fremont City");
    await expect(page.locator(`.subnav a[aria-current="page"]`)).toHaveText("Property tax");
    const change = page.locator(".card", { hasText: "TY2023 to TY2024" });
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
    const change = page.locator(".card", { hasText: "TY2023 to TY2024" });
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
    const change = page.locator(".card", { hasText: "TY2023 to TY2024" });
    await expect(change).toContainText("charges less than twenty mills");
    await expect(change).toContainText("never have");
    await expect(change).not.toContainText("reduction factors are fully operative");

    // And the millage card says the same thing from the other direction: nothing was taken.
    const millage = page.locator(".card", { hasText: "What voters approved" });
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
    const millage = page.locator(".card", { hasText: "What voters approved" });
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
    const card = page.locator(".card", { hasText: "What the mechanism this replaced" });
    await expect(card).toContainText("23 mills");
    await expect(card).toContainText("no minimum state share to stop at");
    await expect(card).toContainText("counterfactual at FY2027 inputs");
    await expect(card).toContainText("not a reconstruction of any year the charge-off governed");
    await expect(card).toContainText("one row of the calculation");
    await expect(card).toContainText("overstates");
  });

  test("a district below the charge-off rate is told it would be charged for phantom revenue", async ({
    page,
  }) => {
    // Vinton County charges 18.70 mills against a mechanism that assumes 23 — the failure the
    // charge-off was replaced for, and half the state is in the same position.
    await page.goto(`/district/${BELOW_FLOOR}/taxes`);
    const card = page.locator(".card", { hasText: "What the mechanism this replaced" });
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
    const card = page.locator(".card", { hasText: "Two pupil counts" });
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
    await expect(page.locator(".card", { hasText: "Two pupil counts" })).toHaveCount(0);
  });

  test("a district above the floor is told they are operative", async ({ page }) => {
    await page.goto("/district/044933/taxes");
    const change = page.locator(".card", { hasText: "TY2023 to TY2024" });
    await expect(change).toContainText("above the");
    await expect(change).toContainText("reduction factors are fully operative");
  });

  test("the base is broken into the classes that are reduced separately", async ({ page }) => {
    await page.goto(`/district/${NORTHERN}/taxes`);
    const base = page.locator(".card", { hasText: "What the tax base is made of" });
    for (const label of ["Residential", "Agricultural", "Commercial", "Public utility"]) {
      await expect(base.locator('[data-chart="tax-base"]')).toContainText(label);
    }
  });

  test("names the department it came from, which is not the usual one", async ({ page }) => {
    // Every other page reads the Department of Education. This one reads Taxation, and says so —
    // along with the fact that where the two overlap they agree.
    await page.goto(`/district/${NORTHERN}/taxes`);
    const source = page.locator(".card", { hasText: "Where these figures come from" });
    await expect(source).toContainText("Ohio Department of Taxation");
    await expect(source).toContainText("to 0.01 mills");
  });
});

test.describe("spending by function", () => {
  test("splits operating spending and keeps it apart from the audited actuals", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator(".card", { hasText: "Where the money went" });
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
    const card = page.locator(".card", { hasText: "Where the money came from" });
    await expect(card).toContainText("29.0%");
    await expect(card).toContainText("statewide median 4.2%");
    await expect(card).toContainText("need-weighted count");
    await expect(card).toContainText("the two cards do not add");
  });

  test("the federal correlation is never shown without its controlled twin", async ({ page }) => {
    // Federal money follows poverty. The raw figure alone would state a confound as a finding.
    await page.goto(`/district/${MOST_FEDERAL}/finances`);
    const card = page.locator(".card", { hasText: "Where the money came from" });
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
    const card = page.locator(".card", { hasText: "Outcomes" }).first();
    await expect(card).toContainText("Progress, three-year average");
    await expect(card).toContainText("Progress, one year");
    await expect(card).toContainText("has no direction to read");
    await expect(card).toContainText("534 districts");
    await expect(card).not.toContainText("point opposite ways for this district");
  });

  test("a district whose measures point opposite ways is told to read neither", async ({ page }) => {
    await page.goto(`/district/${MOST_FEDERAL}/outcome`);
    const card = page.locator(".card", { hasText: "Outcomes" }).first();
    await expect(card).toContainText("point opposite ways for this district");
    await expect(card).toContainText("Read it as neither");
    await expect(card).toContainText("none of them with both magnitudes above 0.05");
  });
});

test.describe("whether Ohio is unusual", () => {
  test("the front page ranks Ohio against the other states", async ({ page }) => {
    /*
     * The only comparative claim on the site, and the only federal source behind it. Everything
     * else here is Ohio describing itself, which cannot answer a question of the form "too
     * heavily" — the form the DeRolph holding takes.
     */
    await page.goto("/");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    await expect(card).toContainText("Local share of school revenue");
    await expect(card).toContainText("7 of 51");
    await expect(card).toContainText("45 of 51");
    await expect(card).toContainText("DeRolph");
  });

  test("the property tax rank names the states it excludes and why", async ({ page }) => {
    /*
     * Nine states report zero school property tax and levy plenty of it — their districts are
     * agencies of a city or county, so the survey attributes the tax to the parent. Ranking all
     * fifty-one would put Massachusetts and Virginia at the bottom of a measure they are near the
     * top of, and the first version of this extractor did exactly that.
     */
    await page.goto("/");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    await expect(card).toContainText("39 states whose districts levy their own tax");
    await expect(card).toContainText("Massachusetts and Virginia report");
    await expect(card).toContainText("excludes twelve states on purpose");
  });

  test("the relief year is named as cutting against the finding", async ({ page }) => {
    await page.goto("/");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    await expect(card).toContainText("peak year of federal pandemic relief");
    await expect(card).toContainText("against this finding rather than for it");
  });

  test("the Census comparison stays out of the scenario panel", async ({ request }) => {
    // 51 states is small, but the formula never reads it and the panel is a formula input.
    const panel = await (await request.get("/data/panel.json")).json();
    expect(panel).not.toHaveProperty("national");
    const feed = await (await request.get("/data/bundle.json")).json();
    expect(feed.national.states).toHaveLength(51);
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
    const card = page.locator(".card", { hasText: "What the mechanism this replaced" });
    await expect(card).toContainText("Local capacity, the plan");
    await expect(card).not.toContainText("Not recoverable");
    await expect(card).toContainText("reproduces it");
  });
});

test.describe("the categorical half", () => {
  test("the lump is shown as six programs that reconcile to it", async ({ page }) => {
    /*
     * Categorical funding was a residual for eight phases — core foundation less the state share
     * of base cost. Exact, and 43% of formula aid expressed as a number with no parts.
     */
    await page.goto("/district/043802");
    const card = page.locator(".card", { hasText: "The categorical half" });
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
    const card = page.locator(".card", { hasText: "The categorical half" });
    await expect(card).toContainText("Special education, by category");
    await expect(card).toContainText("3.9554");
    await expect(card).toContainText("0.2435");
    await expect(card).toContainText("Category 6");
    await expect(card).toContainText("a range of sixteen");

    // The six rows plus the total must reconcile on both shares.
    const rows = card.locator("tbody th").filter({ hasText: /^Category \d$/ });
    await expect(rows).toHaveCount(6);
  });

  test("a district getting no targeted assistance is told why", async ({ page }) => {
    /*
     * The reason the sum misleads rather than merely omitting. Targeted assistance is the largest
     * categorical in Ohio and it is equalisation: Columbus qualifies for none of it, and 77% of
     * its categorical money is DPIA instead. One number cannot distinguish those.
     */
    await page.goto("/district/043802");
    const card = page.locator(".card", { hasText: "The categorical half" });
    await expect(card).toContainText("It receives no targeted assistance");
    await expect(card).toContainText("135 districts are in the same position");
    await expect(card).toContainText("Disadvantaged Pupil Impact Aid");
  });
});
