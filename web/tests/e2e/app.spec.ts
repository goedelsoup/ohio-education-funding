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

import { existsSync, readFileSync, readdirSync } from "node:fs";
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

test.describe("what a link to this site looks like when it is pasted somewhere", () => {
  /*
   * # Why this reads files instead of driving a browser, too
   *
   * Nothing a browser does exercises an `og:` tag. The consumers are Slack, Discord, Facebook,
   * LinkedIn, Bluesky and X, none of which is available to a test, and the failure they produce is
   * a blank rectangle rather than an error anyone sees. The one thing that *is* checkable is the
   * artefact: 3,430 documents, each naming a card, against 995 cards that were actually emitted.
   *
   * That pairing is the whole risk. The tags are written once in `Base.astro` and cannot be wrong
   * on one page and right on another; what can be wrong is one route family pointing at a path the
   * generator never produced — a renamed slug, a `getStaticPaths` that stopped covering something —
   * and it is invisible on every page of the site because the image is not on the page.
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

  const meta = (body: string, key: string): string | null =>
    body.match(
      new RegExp(`<meta [^>]*(?:property|name)="${key}"[^>]*content="([^"]*)"`, "i"),
    )?.[1] ?? null;

  test("every page carries the tags an unfurler reads", () => {
    const offenders: string[] = [];
    for (const file of html(DIST)) {
      const body = readFileSync(file, "utf8");
      for (const key of ["og:title", "og:description", "og:image", "og:url", "twitter:card"]) {
        if (!meta(body, key)) offenders.push(`${file.slice(DIST.length + 1)}: no ${key}`);
      }
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });

  test("every card a page names was actually built", () => {
    const offenders: string[] = [];
    const cards = new Set<string>();
    for (const file of html(DIST)) {
      const image = meta(readFileSync(file, "utf8"), "og:image");
      if (!image) continue;
      const path = new URL(image).pathname;
      cards.add(path);
      if (!existsSync(join(DIST, path)))
        offenders.push(`${file.slice(DIST.length + 1)} names ${path}, which was not emitted`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
    // A sanity floor on the other side: if the endpoints silently stopped producing per-subject
    // cards, every page would fall back to the default and the check above would still pass.
    expect(cards.size).toBeGreaterThan(900);
  });

  test("the canonical link and og:url agree, and neither ends in .html", () => {
    // `build.format` is `"file"`, so `Astro.url.pathname` is the output file. Both of these are
    // that path put back — see `canonicalPath` — and the site shipped for a long time claiming
    // `/district/043786.html` while the sitemap beside it said `/district/043786`.
    const offenders: string[] = [];
    for (const file of html(DIST)) {
      const body = readFileSync(file, "utf8");
      const canonical = body.match(/<link rel="canonical" href="([^"]*)"/i)?.[1] ?? null;
      const url = meta(body, "og:url");
      const name = file.slice(DIST.length + 1);
      if (canonical !== url) offenders.push(`${name}: canonical ${canonical} but og:url ${url}`);
      else if (canonical?.endsWith(".html")) offenders.push(`${name}: ${canonical}`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });

  test("the canonical URLs are the ones the sitemap lists", () => {
    // Two files a crawler reads, which have to name the same set of addresses. They are produced
    // by different code — the layout and `@astrojs/sitemap` — so nothing but this makes them agree.
    const listed = new Set(
      [...readFileSync(join(DIST, "sitemap-0.xml"), "utf8").matchAll(/<loc>([^<]*)<\/loc>/g)].map(
        (m) => m[1]!,
      ),
    );
    const offenders: string[] = [];
    for (const file of html(DIST)) {
      // The 404 is the one page deliberately absent from the sitemap.
      if (file.endsWith("404.html")) continue;
      const canonical =
        readFileSync(file, "utf8").match(/<link rel="canonical" href="([^"]*)"/i)?.[1] ?? "";
      // The root is listed without its trailing slash; `new URL` keeps one. Compare both forms.
      if (!listed.has(canonical) && !listed.has(canonical.replace(/\/$/, "")))
        offenders.push(`${file.slice(DIST.length + 1)}: ${canonical} is not in the sitemap`);
    }
    expect(offenders.slice(0, 10)).toEqual([]);
  });

  test("the icons the layout links are all present", () => {
    for (const icon of ["favicon.svg", "icon-32.png", "apple-touch-icon.png"]) {
      expect(existsSync(join(DIST, icon)), `${icon} was not emitted`).toBe(true);
    }
  });
});

test.describe("the document arrives complete", () => {
  test("a district page carries its figures before any script runs", async ({ page }) => {
    const failures: string[] = [];
    page.on("pageerror", (error) => failures.push(error.message));

    await page.goto(`/district/${CLEVELAND}`);

    await expect(page.locator("h1")).toHaveText("Cleveland Municipal");
    await expect(page.locator('[data-part="headline"] .tile')).toHaveCount(3);
    await expect(page.locator(".err")).toHaveCount(0);
    expect(failures, "the page threw while booting").toEqual([]);
  });

  test("every card on the five district routes is addressed by attribute, not by heading", async ({
    page,
  }) => {
    /*
     * The headings are prose, and this project rewrites prose. Thirty-six e2e locators used to
     * find their card with `hasText`, which made a house-style `<h2>` part of the test API: a
     * reworded heading broke assertions that were about something else entirely, and a card whose
     * heading nobody had matched on could not be scoped to at all — which is why
     * `renderCategoricals` addressed its six sub-tables globally rather than within its own card.
     *
     * `data-part` is the hook now, and this asserts there is no card without one. Without it the
     * next card arrives with no address, the next assertion about it reaches for its heading
     * again, and the rule decays back to prose one card at a time.
     *
     * Scoped to the five district routes because that is the family the rule is for. `/outcomes`,
     * the front page and the wiki still name their cards in prose, deliberately.
     */
    const unaddressed: string[] = [];
    for (const suffix of ["", "/finances", "/outcome", "/taxes", "/scenario"]) {
      await page.goto(`/district/${CLEVELAND}${suffix}`);
      const bare = await page
        .locator(".card:not([data-part])")
        .evaluateAll((nodes) =>
          nodes.map((n) => n.querySelector("h2")?.textContent?.trim() ?? "(no heading)"),
        );
      for (const heading of bare) unaddressed.push(`${suffix || "/dashboard"}: ${heading}`);
    }
    expect(unaddressed, "a card with no data-part is a card only its heading can reach").toEqual(
      [],
    );
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

  test("the section menus still open, and their links still go somewhere", async ({ page }) => {
    // The reason they are `<details>` rather than a scripted menu. Four of the ten sections are
    // only reachable through one of these, so a menu that needs script to open would put a
    // quarter of the site behind JavaScript on a site whose whole point is that nothing is.
    await page.goto("/");
    const places = page.locator("header.site nav details.menu").filter({ hasText: "Places" });
    await expect(places.locator("a")).toHaveCount(5);
    await expect(places.locator('a[href="/counties"]')).toBeHidden();

    await places.locator("summary").click();
    await expect(places.locator('a[href="/counties"]')).toBeVisible();
    await places.locator('a[href="/counties"]').click();
    await expect(page).toHaveURL(/\/counties$/);
    await expect(page.locator("h1")).toBeVisible();
  });

  test("the group holding the current page is marked, and the page itself is marked inside it", async ({
    page,
  }) => {
    // Two different claims: `page` on the link a reader is on, `true` on the group containing it.
    // Marking the summary as the current *page* would tell a screen reader the reader is on a
    // thing that is not a destination.
    await page.goto("/history");
    const findings = page.locator("header.site nav details.menu").filter({ hasText: "Findings" });
    await expect(findings.locator("summary")).toHaveAttribute("aria-current", "true");
    await findings.locator("summary").click();
    await expect(findings.locator('a[href="/history"]')).toHaveAttribute("aria-current", "page");
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
  test("each of a district's five views is its own address", async ({ page }) => {
    // `/taxes` landed after the other four and was left out of this list, so the one nav state
    // nothing had ever asserted was the heaviest sibling's. The label is read off the rendered
    // nav rather than hard-coded twice, so a rename fails here rather than passing on a stale
    // expectation.
    for (const [path, heading] of [
      ["", "Dashboard"],
      ["/outcome", "Outcome"],
      ["/finances", "Finances"],
      ["/taxes", "Property tax"],
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

  test("every section in the navigation resolves to a page", async ({ page }) => {
    // Grouping moved four entries behind a disclosure and admitted two pages that had never been
    // in the bar at all. A menu item pointing at a route the build does not emit looks exactly
    // like one that works, right up until someone opens it.
    await page.goto("/");
    const hrefs = await page
      .locator("header.site nav a")
      .evaluateAll((nodes) => nodes.map((n) => (n as HTMLAnchorElement).getAttribute("href")!));
    // Two flat entries and ten inside the three groups. An exact count rather than a floor, so
    // that dropping an entry fails here and adding one is an acknowledged change.
    expect(hrefs).toHaveLength(12);
    for (const href of hrefs) {
      await page.goto(href);
      await expect(page.locator("h1"), `${href} has no heading`).toBeVisible();
    }
  });
});

test.describe("the section menus", () => {
  test("opening one closes the other, and Escape closes it", async ({ page }) => {
    // Purely the enhancement half — the menus open without any of this, which the JavaScript
    // disabled suite asserts. What is checked here is that the script does not make it worse.
    await page.goto("/");
    const places = page.locator("header.site nav details.menu").filter({ hasText: "Places" });
    const reference = page.locator("header.site nav details.menu").filter({ hasText: "Reference" });

    await places.locator("summary").click();
    await expect(places).toHaveAttribute("open", "");

    await reference.locator("summary").click();
    await expect(reference).toHaveAttribute("open", "");
    await expect(places).not.toHaveAttribute("open", "");

    await page.keyboard.press("Escape");
    await expect(reference).not.toHaveAttribute("open", "");
    // Focus returns to the summary rather than to the top of the document.
    await expect(reference.locator("summary")).toBeFocused();
  });

  test("a click outside closes an open menu", async ({ page }) => {
    await page.goto("/");
    const places = page.locator("header.site nav details.menu").filter({ hasText: "Places" });
    await places.locator("summary").click();
    await expect(places).toHaveAttribute("open", "");
    await page.locator("h1").click();
    await expect(places).not.toHaveAttribute("open", "");
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
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "flat by construction",
    );
  });

  test("a formula-funded district gets a band and no second line", async ({ page }) => {
    await page.goto(`/district/${ON_FORMULA}`);
    const chart = page.locator('[data-chart="district-fan"] svg');
    await expect(chart.locator(".fan-band")).toHaveCount(1);
    await expect(chart.locator(".fan-reference")).toHaveCount(0);
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "The range, not the line, is the finding",
    );
  });

  test("compares enrollment years without inventing a published one", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator('[data-part="enrollment"]');
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
    await expect(page.locator('[data-part="not"]')).toContainText(
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
    const card = page.locator('[data-part="comparable-poverty"]');
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

test.describe("the decisions behind the corpus", () => {
  test("a decision record renders its sections in the order they are meant to be read", async ({
    page,
  }) => {
    await page.goto("/wiki/decision/the-order-was-never-the-states");
    await expect(page.locator("h1")).toHaveText("the-order-was-never-the-states");
    const headings = await page.locator(".card h2").allInnerTexts();
    expect(headings.slice(0, 4)).toEqual([
      "Context",
      "The decision",
      "Consequences",
      "Alternatives considered",
    ]);
  });

  test("a withdrawn claim is announced at the top and marked where it stands", async ({ page }) => {
    // The whole reason these are published. `the-directory-cannot-say-why` says Newbury Local's
    // territory split between West Geauga and Chardon; it did not, and the record now says so in
    // four places. A reader who lands here from a search must not have to find that by reading.
    await page.goto("/wiki/decision/the-directory-cannot-say-why");
    const index = page.locator(".card.correction-index");
    await expect(index).toBeVisible();
    await expect(index.getByRole("heading")).toContainText("been withdrawn or superseded");

    // Four corrections, each with an anchor into the section that carries it.
    await expect(page.locator("blockquote.correction")).toHaveCount(4);
    const first = index.locator("a").first();
    const target = (await first.getAttribute("href"))!.slice(1);
    await first.click();
    await expect(page.locator(`#${target}`)).toBeVisible();
    await expect(page.locator(`#${target}`)).toHaveClass(/correction/);
  });

  test("a quotation is not dressed as a correction", async ({ page }) => {
    // Both kinds are blockquotes and the distinction is the feature. `reading-an-amending-act`
    // quotes the previous phase's blocker and withdraws nothing.
    await page.goto("/wiki/decision/reading-an-amending-act");
    await expect(page.locator("blockquote")).not.toHaveCount(0);
    await expect(page.locator("blockquote.correction")).toHaveCount(0);
    await expect(page.locator(".card.correction-index")).toHaveCount(0);
  });

  test("a catalog entry reaches the decision behind it without leaving the site", async ({
    page,
  }) => {
    // These used to resolve to GitHub, which was honest while nothing published them. Thirteen
    // links from published prose point into this subtree.
    await page.goto("/wiki/source/derolph-litigation-record");
    await page.locator('.prose-body a[href="/wiki/decision/what-a-citator-reaches"]').first().click();
    await expect(page.locator("h1")).toHaveText("what-a-citator-reaches");
    await expect(page.getByRole("heading", { name: "Cited by" })).toBeVisible();
  });

  test("the index leads with the records that turned out wrong", async ({ page }) => {
    await page.goto("/wiki/decision");
    const rows = page.locator("tbody tr");
    await expect(rows).not.toHaveCount(0);
    // Ordered by how much of each has since been withdrawn, so the first row carries corrections
    // and names them. A count of zero renders as an em dash rather than as "0 claims".
    await expect(rows.first()).toContainText("claims");
    await expect(rows.first().locator("code")).toHaveText("the-directory-cannot-say-why");
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
    const change = page.locator('[data-part="valuation-change"]');
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
    const change = page.locator('[data-part="valuation-change"]');
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
    const change = page.locator('[data-part="valuation-change"]');
    await expect(change).toContainText("charges less than twenty mills");
    await expect(change).toContainText("never have");
    await expect(change).not.toContainText("reduction factors are fully operative");

    // And the millage card says the same thing from the other direction: nothing was taken.
    const millage = page.locator('[data-part="millage"]');
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
    const millage = page.locator('[data-part="millage"]');
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
    const card = page.locator('[data-part="charge-off"]');
    await expect(card).toContainText("23 mills");
    await expect(card).toContainText("no minimum state share to stop at");
    await expect(card).toContainText("counterfactual at FY2027 inputs");
    await expect(card).toContainText("not a reconstruction of any year the charge-off governed");
    await expect(card).toContainText("one row of the calculation");
    // The correction: the page used to say recognised valuation was an H.B. 920 adjustment this
    // project did not hold, and that every figure therefore overstated the charge-off. It is
    // neither — it is a reappraisal phase-in, and it is now computed. Asserting the corrected
    // wording rather than deleting the check, so the page cannot regress to the old claim.
    await expect(card).toContainText("because this page said something wrong");
    await expect(card).toContainText("is not an H.B. 920 adjustment");
    await expect(card).toContainText("staggered county calendar");
    await expect(card).toContainText("Franklin County revalued in 2023");
  });

  test("a district below the charge-off rate is told it would be charged for phantom revenue", async ({
    page,
  }) => {
    // Vinton County charges 18.70 mills against a mechanism that assumes 23 — the failure the
    // charge-off was replaced for, and half the state is in the same position.
    await page.goto(`/district/${BELOW_FLOOR}/taxes`);
    const card = page.locator('[data-part="charge-off"]');
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
    const card = page.locator('[data-part="denominators"]');
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
    await expect(page.locator('[data-part="denominators"]')).toHaveCount(0);
  });

  test("a district above the floor is told they are operative", async ({ page }) => {
    await page.goto("/district/044933/taxes");
    const change = page.locator('[data-part="valuation-change"]');
    await expect(change).toContainText("above the");
    await expect(change).toContainText("reduction factors are fully operative");
  });

  test("the base is broken into the classes that are reduced separately", async ({ page }) => {
    await page.goto(`/district/${NORTHERN}/taxes`);
    const base = page.locator('[data-part="tax-base"]');
    for (const label of ["Residential", "Agricultural", "Commercial", "Public utility"]) {
      await expect(base.locator('[data-chart="tax-base"]')).toContainText(label);
    }
  });

  test("names the department it came from, which is not the usual one", async ({ page }) => {
    // Every other page reads the Department of Education. This one reads Taxation, and says so —
    // along with the fact that where the two overlap they agree.
    await page.goto(`/district/${NORTHERN}/taxes`);
    const source = page.locator('[data-part="not"]');
    await expect(source).toContainText("Ohio Department of Taxation");
    await expect(source).toContainText("to 0.01 mills");
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
    const card = page.locator('[data-part="charge-off"]');
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
    await expect(card).toContainText("along programme type rather than need");
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
    await expect(body).toContainText("SIZE CLIFF");
    await expect(body).toContainText("PAYS THEM NOTHING");
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

test.describe("counties", () => {
  test("the index ranks counties by the disparity inside them", async ({ page }) => {
    /*
     * The ordering is the argument. Alphabetical would make this a directory; ordered by how far
     * apart the richest and poorest district in each county are, it puts the finding first.
     */
    await page.goto("/counties");
    await expect(page.locator("h1")).toHaveText("Counties");

    const ratios = await page
      .locator("tbody tr td:nth-child(4)")
      .allTextContents();
    const numeric = ratios
      .map((t) => Number.parseFloat(t.replace("×", "")))
      .filter((n) => Number.isFinite(n));
    expect(numeric.length).toBeGreaterThan(80);
    for (let i = 1; i < numeric.length; i += 1) {
      expect(numeric[i]!).toBeLessThanOrEqual(numeric[i - 1]!);
    }
  });

  test("a county page names its widest pair and links both districts", async ({ page }) => {
    await page.goto("/county/cuyahoga");
    await expect(page.locator("h1")).toHaveText("Cuyahoga County");
    const spread = page.locator('.card[data-part="spread"]');
    await expect(spread).toContainText("Orange City");
    await expect(spread).toContainText("Maple Heights City");
    await expect(spread).toContainText("times");
    // Both extremes must be reachable, since the point of the page is to send a reader onward.
    await expect(spread.locator('a[href^="/district/"]')).toHaveCount(2);
  });

  test("a county page says its pupil total is a sum of districts, not the county's children", async ({
    page,
  }) => {
    /*
     * The honesty constraint the whole feature rests on. Ohio school district boundaries cross
     * county lines, so a county page that reports a total without saying what it is a total *of*
     * is making a geographic claim the data does not support.
     */
    await page.goto("/county/cuyahoga");
    await expect(page.locator('.card[data-part="roster"]')).toContainText(
      "a sum of these districts rather than a count of the county's children",
    );
  });

  test("a single-district county says there is nothing to compare", async ({ page }) => {
    // Rather than printing a ratio of 1.0, which would read as "no disparity here" — a different
    // claim from "this county has one district".
    await page.goto("/counties");
    const row = page.locator("tbody tr").filter({ hasText: "a single district" }).first();
    await expect(row).toBeVisible();
    const href = await row.locator("a").first().getAttribute("href");
    await page.goto(href!);
    await expect(page.locator('.card[data-part="spread"]')).toContainText(
      "no internal disparity to measure",
    );
  });

  test("every district page's county is reachable from the counties index", async ({ page }) => {
    // The link direction that matters: a reader arrives at a district and wants its neighbours.
    await page.goto("/counties");
    await expect(page.locator('a[href^="/county/"]')).toHaveCount(88);
  });
});

test.describe("senate districts", () => {
  test("all thirty-three seats are listed and the page says it is the tighter view", async ({
    page,
  }) => {
    /*
     * Senate seats are three times larger than House seats, so 392 of 609 school districts lie
     * wholly inside one against 270 for the House. Two pages built from one template are not
     * equally reliable, and the page says which is which rather than leaving a reader to infer it.
     */
    await page.goto("/senate");
    await expect(page.locator("h1")).toHaveText("Senate districts");
    await expect(page.locator('a[href^="/senate/"]')).toHaveCount(33);
    await expect(page.locator(".card").first()).toContainText("less approximate");
  });

  test("the two chambers apportion the same total", async ({ page }) => {
    // Both partition the state, so the rows on each index must add to the same figure. If they
    // did not, one of the two crosswalks would be losing districts.
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const sum = (xs: { realized_aid: number }[]) => xs.reduce((a, x) => a + x.realized_aid, 0);
    expect(Math.abs(sum(feed.house_districts) - sum(feed.senate_districts))).toBeLessThan(1);
    expect(feed.senate_districts).toHaveLength(33);
  });

  test("a district page links both chambers", async ({ page }) => {
    // Columbus spans eleven House seats and four Senate seats.
    await page.goto("/district/043802");
    const sub = page.locator("p.sub").first();
    await expect(sub.locator('a[href^="/house/"]')).toHaveCount(11);
    await expect(sub.locator('a[href^="/senate/"]')).toHaveCount(4);
  });
});

test.describe("house districts", () => {
  test("the index says plainly that its figures are estimates", async ({ page }) => {
    /*
     * The constraint the whole feature rests on. No House district is a unit of account in Ohio's
     * funding system, so every figure here is derived by splitting school districts across census
     * blocks. A page that shows "$43,707,982" without saying so is passing off a derivation as a
     * published fact, and the precision makes it worse rather than better.
     */
    await page.goto("/house");
    await expect(page.locator("h1")).toHaveText("House districts");
    await expect(page.locator(".card").first()).toContainText(
      "These figures are estimates, and nobody publishes them",
    );
    await expect(page.locator(".card").first()).toContainText("under-18 population");
    await expect(page.locator(".card").first()).toContainText("2020 census");
  });

  test("all ninety-nine seats are listed and reachable", async ({ page }) => {
    await page.goto("/house");
    await expect(page.locator('a[href^="/house/"]')).toHaveCount(99);
  });

  test("a seat page distinguishes the two shares it prints side by side", async ({ page }) => {
    /*
     * "Of the district" and "of this seat" answer opposite questions, and a swap would be
     * invisible: 100% in the wrong column reads as "the member speaks for all of it" when it means
     * "this seat is entirely that district".
     */
    await page.goto("/house/054");
    const members = page.locator('.card[data-part="members"]');
    await expect(members).toContainText("Of the district");
    await expect(members).toContainText("Of this seat");
    await expect(members).toContainText("answer opposite questions");
  });

  test("a district page links every seat it lies in", async ({ page }) => {
    // Columbus spans eleven. The reverse direction is the one a reader arriving from a legislative
    // page is asking about, so it has to be complete rather than a count.
    await page.goto("/district/043802");
    const sub = page.locator("p.sub").first();
    await expect(sub.locator('a[href^="/house/"]')).toHaveCount(11);
  });

  test("the seat totals a reader can add up reconcile to the statewide figure", async ({
    page,
  }) => {
    // The one accuracy claim the apportionment makes, checked as rendered rather than as data:
    // the index states the statewide total, and it must be the sum of the rows above it.
    await page.goto("/house");
    const cells = await page.locator("tbody tr td:nth-child(2)").allTextContents();
    const rows = cells.map((t) => Number(t.replace(/[$,]/g, ""))).filter(Number.isFinite);
    expect(rows).toHaveLength(99);
    const summed = rows.reduce((a, b) => a + b, 0);
    const stated = await page.locator(".card").first().textContent();
    const match = stated?.match(/these\s+99\s+rows\s+add\s+to\s+\$([\d,]+)/);
    expect(match, "the index states the statewide total").not.toBeNull();
    const declared = Number(match![1]!.replace(/,/g, ""));
    // Each row is rendered to the dollar, so ninety-nine roundings can drift by that much.
    expect(Math.abs(summed - declared)).toBeLessThan(100);
  });
});

test.describe("outside the formula", () => {
  test("the card says plainly that the guarantee does not hold these", async ({ page }) => {
    /*
     * The structural point. Everything above this card is `[H] Foundation Funding`, which the
     * guarantee protects; these sit in `[R] Total State Support` and nothing cushions a fall in
     * them. A district that drops a star loses the money outright.
     */
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("Outside the formula");
    await expect(card).toContainText("the guarantee does not hold a district at them");
    await expect(card).toContainText("Base funding supplement");
    await expect(card).toContainText("$40 a pupil");
  });

  test("the performance supplement names the rating it was paid on", async ({ page }) => {
    // Cleveland is rated 2.5 stars and has a 4.0 progress rating, and is paid on the greater of
    // the two — the progress route working exactly as designed for a high-poverty district.
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("the greater of its");
    await expect(card).toContainText("progress rating");
  });

  test("the page states the gradient rather than only the district's own figure", async ({
    page,
  }) => {
    /*
     * The finding has to appear on every district's page, not only where it flatters. A component
     * distributed inversely to need is a fact about the program, and a reader looking at a
     * well-funded district should meet it there too.
     */
    await page.goto("/district/043786");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("$54.74");
    await expect(card).toContainText("$23.31");
    await expect(card).toContainText("track intake");
  });

  test("transportation names which of the two bases the district is paid on", async ({ page }) => {
    /*
     * The `MAX` flips for more than half the state and is invisible in the amount. A district paid
     * on miles gains nothing from carrying more children on the same routes; one paid on riders
     * gains nothing from covering more ground. The card says which.
     */
    await page.goto("/district/000442");
    const table = page.locator('[data-part="supplements"] table[data-program="transportation"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("Per-rider base");
    await expect(table).toContainText("Per-mile base");
    await expect(table).toContainText("The greater of the two");
  });

  test("the proration is shown as a shortfall, not as a rate", async ({ page }) => {
    /*
     * A factor below one means the appropriation did not cover the computed entitlement. The
     * published amount is therefore not what the district was owed, and the only way to show that
     * is to print both figures and the difference.
     */
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("That last factor is not a rate");
    await expect(card).toContainText("short");
    await expect(card).toContainText("what there was to divide");
  });

  test("the 50% transportation floor is stated against the formula's 10%", async ({ page }) => {
    // The largest single difference between how Ohio equalises instruction and how it equalises
    // getting to it, and it exists only in a spreadsheet.
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("50%");
    await expect(card).toContainText("against the formula's 10%");
  });

  test("preschool special education names the flat grant the state share does not touch", async ({
    page,
  }) => {
    /*
     * The one payment in Ohio's school funding a district's wealth does not reduce: a flat $4,000
     * a pupil whatever the category, 69% of the program. It is also what flattens the same six
     * weights that make school-age special education steeply top-heavy.
     */
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card.locator('table[data-program="preschool"]')).toBeVisible();
    await expect(card).toContainText("flat $4,000");
    await expect(card).toContainText("the state share does not reduce");
    await expect(card).toContainText("Weight, halved");
  });

  test("the page says the preschool program is over its own appropriation", async ({ page }) => {
    /*
     * The sheet is the only place in the calculator that shows what a proration is, because the
     * appropriation limit sits in a cell beside the factor — and at the stated factor the program
     * exceeds it by $908,184. Reproducing that silently and calling it verified would be the wrong
     * answer; the page states the inconsistency and what it most likely is.
     */
    await page.goto("/district/000442");
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("over its appropriation");
    await expect(card).toContainText("$147,500,000");
    await expect(card).toContainText("do not agree");
  });

  test("a district just below the growth cliff is told what it forwent", async ({ page }) => {
    /*
     * The only honest way to show a cliff. New Lexington grew 2.9502% against the 3% required and
     * the supplement pays on the whole roll, so missing by three hundredths of a percentage point
     * cost it $430,477.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const near = feed.districts.find(
      (d: { supplements: { growth_eligible: boolean; enrollment_change: number } }) =>
        !d.supplements.growth_eligible && d.supplements.enrollment_change > 0.029,
    );
    expect(near, "a district just below the 3% threshold").toBeTruthy();

    await page.goto(`/district/${near.irn}`);
    const card = page.locator('.card[data-part="supplements"]');
    await expect(card).toContainText("just under the growth cliff");
    await expect(card).toContainText("pays on the whole roll rather than on the pupils gained");
  });
});

test.describe("the hold-harmless machinery", () => {
  test("the guarantee's open-enrolment clawback is shown as a deduction", async ({ page }) => {
    /*
     * The guarantee is not "hold the district at its old amount". A guaranteed district losing
     * open-enrolment FTE beyond a threshold has its guarantee cut at the full statewide average
     * base cost per pupil — more than the state was paying for those pupils. Columbus lost 106.2
     * FTE and had $674,561 taken off.
     */
    await page.goto("/district/043802");
    const table = page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("FY2021 funding base");
    await expect(table).toContainText("Open-enrolment clawback");
    await expect(page.locator("main")).toContainText("not at this district's share of it");
  });

  test("the second hold-harmless is named as a second one", async ({ page }) => {
    // The formula transition supplement holds a district at a larger FY2021 base that includes
    // transportation, and reaches 17 districts the guarantee does not.
    await page.goto("/district/043802");
    const main = page.locator("main");
    await expect(main).toContainText("Formula transition supplement");
    await expect(main).toContainText("three");
    await expect(main).toContainText("anchored to");
  });

  test("every component on a district page reaches the node describing it", async ({ page }) => {
    /*
     * The whole point of the wiki: a glossary nobody arrives at from the number they were reading
     * is a document museum. Every line of Ohio's formula now has a node, and every figure on the
     * dashboard links to its own.
     */
    await page.goto("/district/043802");
    const links = page.locator('main a[href^="/wiki/formula-component/fsfp-"]');
    const hrefs = await links.evaluateAll((els) =>
      [...new Set(els.map((e) => (e as HTMLAnchorElement).getAttribute("href")))].sort(),
    );
    for (const node of [
      "fsfp-targeted-assistance",
      "fsfp-special-education-weights",
      "fsfp-disadvantaged-pupil-impact-aid",
      "fsfp-career-technical-weights",
      "fsfp-english-learner-weights",
      "fsfp-gifted-units",
      "fsfp-transportation",
      "fsfp-preschool-special-education",
      "fsfp-performance-supplement",
      "fsfp-enrolment-supplements",
      "fsfp-formula-transition-supplement",
    ]) {
      expect(hrefs, node).toContain(`/wiki/formula-component/${node}`);
    }
  });

  test("the guarantee node records the clawback it used to omit", async ({ page }) => {
    // The correction that matters most: the node described a hold-harmless with no clawback in it,
    // which reproduces correctly for 566 districts and wrongly for 43.
    await page.goto("/wiki/formula-component/temporary-transitional-aid-guarantee");
    const body = page.locator("main");
    await expect(body).toContainText("NOT WHAT THIS NODE SAID");
    await expect(body).toContainText("Open Enrolment Adjustment");
    await expect(body).toContainText("566 districts and wrongly for 43");
  });

  test("the proration parameter names all three and which one publishes its limit", async ({
    page,
  }) => {
    await page.goto("/wiki/parameter/appropriation-proration-factor");
    const body = page.locator("main");
    await expect(body).toContainText("not a rate, a weight, a price or a threshold");
    await expect(body).toContainText("147,500,000");
    await expect(body).toContainText("A PRORATION OF 1.0 IS NOT AN ABSENCE");
  });

  test("a district touched by neither shows no hold-harmless table", async ({ page }) => {
    // The block renders only what applies, so a district on formula with no clawback and no
    // supplement gets nothing rather than a table of dashes.
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const clean = feed.districts.find(
      (d: { transition: { open_enrollment_adjustment: number; transition_supplement: number } }) =>
        d.transition.open_enrollment_adjustment === 0 && d.transition.transition_supplement === 0,
    );
    expect(clean, "a district touched by neither mechanism").toBeTruthy();
    await page.goto(`/district/${clean.irn}`);
    await expect(page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]')).toHaveCount(
      0,
    );
  });
});

test.describe("what the scenario holds fixed", () => {
  test("the caveat is above the controls, not below the results", async ({ page }) => {
    /*
     * It is a limit on what the reader is about to do, not a footnote on what they got. The
     * department's own calculator warns that its statewide constants are not recalculated when a
     * district's data changes; that caveat is larger here, where every lever moves every district.
     */
    await page.goto("/scenario");
    const caveat = page.locator('.card[data-part="held-fixed"]');
    await expect(caveat).toContainText("What these levers hold fixed");
    await expect(caveat).toContainText("not recalculated");

    // Above the controls in document order.
    const order = await page.evaluate(() => {
      const c = document.querySelector('[data-part="held-fixed"]');
      const controls = document.querySelector("#scenario-root");
      if (!c || !controls) return null;
      return c.compareDocumentPosition(controls) & Node.DOCUMENT_POSITION_FOLLOWING ? "before" : "after";
    });
    expect(order).toBe("before");
  });

  test("the divergence from the department's tool is stated, with its size", async ({ page }) => {
    /*
     * This used to assert that the page named an omission — $858.2m of categoricals denominated in
     * the average base cost per pupil, making a refresh understate itself by 24%. The omission was
     * closed: `base_cost_scale` now moves the $812.5m of it that sits inside foundation funding,
     * and the page's job changed from confessing a gap to declaring a deliberate divergence from
     * the department's own simulator. A caveat without a magnitude is unfalsifiable; so is a
     * divergence, and both numbers are asserted here.
     */
    await page.goto("/scenario");
    const caveat = page.locator('.card[data-part="held-fixed"]');
    await expect(caveat).toContainText("diverges from the department's on purpose");
    await expect(caveat).toContainText("$812.5M");
    await expect(caveat).toContainText("$25.2M");
    await expect(caveat).toContainText("genuinely cancel");
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
});


test.describe("the count the poverty weight is paid on", () => {
  test("the seventeen-year series is on the page, which is the point of exporting it", async ({
    page,
  }) => {
    // It was computed in `crates/dispersion`, tested there, and reachable by nobody who was not
    // running cargo. This test is the one that would have failed for the four phases it sat there.
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    await expect(card).toBeVisible();
    await expect(card).toContainText("FY1998");
    await expect(card).toContainText("FY2014");
  });

  test("the break in the denominator is stated where the chart is, not in a footnote", async ({
    page,
  }) => {
    /*
     * The share steps up at FY2010 and part of that step is the divisor changing definition. A
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
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    const rows = card.locator("tbody tr");
    await expect(rows).toHaveCount(17);
    await expect(rows.first()).toContainText("AdmCount");
    await expect(rows.last()).toContainText("CECount");
  });

  test("the years published as three files carry a range where a share would be", async ({
    page,
  }) => {
    /*
     * The finding this extension exists for. From FY2012 only one of the three files still
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
