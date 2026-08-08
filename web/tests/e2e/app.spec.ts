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

/** Cleveland Municipal. On the guarantee, so the guarantee copy has something to render. */
const CLEVELAND = "043786";
/** Northern Local (Perry County). The corpus's property-poor exemplar. */
const NORTHERN = "049056";
/** Manchester Local — funded by the formula, so its band opens rather than collapsing. */
const ON_FORMULA = "000442";

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
    expect(bundle.contract_version).toBe("6.0.0");
    expect(bundle.districts).toHaveLength(609);

    // The panel is what the scenario routes fetch: the same districts, without the two blocks the
    // formula never reads.
    const panel = await request.get("/data/panel.json");
    expect(panel.status()).toBe(200);
    const slim = await panel.json();
    expect(slim.districts).toHaveLength(609);
    expect(slim.districts[0]).not.toHaveProperty("finances");
    expect(slim.districts[0]).not.toHaveProperty("outcome");
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
