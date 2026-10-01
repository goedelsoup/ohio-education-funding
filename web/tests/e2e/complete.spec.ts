/**
 * What is true before any script runs, and what is still true when none ever does.
 *
 * Every page on this site is complete as served — the tables, the charts and the figures are in the
 * HTML, and the scripts add interaction rather than content. That claim is only worth making if
 * something checks it, so the first block checks it with JavaScript on and the second with it off.
 *
 * The third is about the cost of the scripts that do load: a page declares the chunks it will
 * import, and the declaration has to match the closure. The artefact half of that — every page's
 * declarations against every page's imports, across the build — is in
 * `tests/dist/payload.spec.ts`.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

import { REQUIRED_CONTRACT } from "../../src/lib/types.ts";
import { CLEVELAND } from "./helpers.ts";

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
     *
     * # The scenario route has to be waited for, not raced
     *
     * Four of the five are complete before any script runs and can be scanned the moment they
     * load. `/district/X/scenario` is the one view that is not: `#scenario-out` ships empty and
     * `scripts/scenario.ts` fills it after fetching the panel. A version of this test that scanned
     * on arrival found nothing there on a fast local machine and found the "Current law" card in
     * CI, which is a test that reports on the machine it ran on rather than on the page — and it
     * did exactly that, passing here and failing there.
     *
     * So the scan waits for the client render. That also brings the script-rendered cards inside
     * the rule, which is where they belong: a card injected by `innerHTML` is as unaddressable as
     * one written at build time, and rather more likely to be missed.
     */
    const unaddressed: string[] = [];
    // The fifth tab is the runner opened on the district, not a route of the district's own (#548).
    const views: [string, string][] = [
      ["/dashboard", `/district/${CLEVELAND}`],
      ["/finances", `/district/${CLEVELAND}/finances`],
      ["/outcome", `/district/${CLEVELAND}/outcome`],
      ["/taxes", `/district/${CLEVELAND}/taxes`],
      ["/scenario", `/scenario?d=${CLEVELAND}`],
    ];
    for (const [suffix, url] of views) {
      await page.goto(url);
      if (suffix === "/scenario") {
        await expect(page.locator("#scenario-out .card")).not.toHaveCount(0);
      }
      const bare = await page
        .locator(".card:not([data-part])")
        .evaluateAll((nodes) =>
          nodes.map((n) => n.querySelector("h2")?.textContent?.trim() ?? "(no heading)"),
        );
      for (const heading of bare) unaddressed.push(`${suffix}: ${heading}`);
    }
    expect(unaddressed, "a card with no data-part is a card only its heading can reach").toEqual(
      [],
    );
  });

/**
 * Every route family that renders a figure, for the year-chip rule.
 *
 * **Not** the five district routes the `data-part` rule uses. That scoping is deliberate for
 * addressability — the statewide pages name their cards in prose — but it was wrong here, and
 * silently: the first version of this test reused the district list, so it passed while four cards
 * on the front page, two on `/outcomes`, two on `/history` and one each on `/counties` and `/data`
 * showed figures under no year at all. A rule that only checks where it was already applied is not
 * a rule.
 *
 * One district and one county stand for their families; the rest are singletons.
 */
const ROUTES_WITH_FIGURES = [
  "/",
  /* `/statewide` and not only `/`: the figures moved here when the root became a front door, and
     this list kept pointing at the root — so the sweeps below went on passing against a page that
     no longer carries what they scan for. */
  "/statewide",
  "/legislation",
  "/outcomes",
  "/history",
  "/counties",
  "/data",
  "/districts",
  "/house",
  "/senate",
  `/district/${CLEVELAND}`,
  `/district/${CLEVELAND}/finances`,
  `/district/${CLEVELAND}/outcome`,
  `/district/${CLEVELAND}/taxes`,
  `/scenario?d=${CLEVELAND}`,
  /*
   * `/scenario` was missing, and the omission is the same shape the docstring above describes.
   * Scanned by hand it had two unchipped cards carrying figures — the projection, present in the
   * default state, and "Most affected".
   */
  "/scenario",
  /*
   * And the runner's other view. It contributes nothing to the `.tnum`/`.v` filter today — its one
   * card states its figures in prose, inside a chart's own description — so it is listed for the
   * same reason `/scenario` was added rather than because it currently catches anything: the
   * moment a tile lands on it, it is covered.
   */
  "/scenario/reach",
];

  test("a card with figures says what year they are on", async ({ page }) => {
    /*
     * The failure this closes. A district page shows the FY2027 formula, a 2024 tax year, a
     * 2024-25 report card, an FY2022 Census survey and a forecast reaching back to FY2020 — and
     * every one of those used to sit under a single header reading `FY2027`.
     *
     * The rule is per card because the year is a property of the source, and the sources differ
     * card by card. A card with no figures is exempt: a chip over prose dates nothing.
     *
     * Scanned on the built page rather than asserted per renderer, because the renderers are not
     * where cards can go missing — a card added in an `.astro` file is as unchipped as one added
     * in `src/lib`, and only the rendered page sees both.
     */
    const missing: string[] = [];
    for (const route of ROUTES_WITH_FIGURES) {
      await page.goto(route);
      // Both of the runner's views, with and without a district chosen.
      if (route.startsWith("/scenario")) {
        /*
         * Past a lever, not merely past the first render.
         *
         * Waiting for `#scenario-out .card` was enough to make this route *appear* covered and not
         * enough to cover it. In the default state `renderDistrictScenario` returns the
         * `current-law` card alone, which carries no `.tnum` and no `.v` — so the filter dropped it
         * and the route contributed nothing to the sweep for as long as it had been in the list.
         *
         * Moving a lever is what brings out the cards that hold figures. The projection is waited
         * for separately: it is a second gate and renders into its own container.
         */
        await page.locator("#lv-base").fill("1.05");
        await page.locator("#lv-base").dispatchEvent("input");
        // `/reach` renders one card and it is not the scenario's headline. It is the only one of
        // the three views whose card is there before a lever moves — the guarantee wall does not
        // depend on the levers — but it is waited for past the lever anyway, so the sweep sees the
        // card the reader sees rather than the one the page opened with.
        await expect(
          page.locator(route === "/scenario/reach" ? '[data-part="positions"]' : '[data-part="outcome"]'),
        ).toBeVisible();
        // The forecast is statewide and stays under a chosen district's cards (#548).
        if (route !== "/scenario/reach") {
          await expect(page.locator('#projection-out [data-part="projection"]')).toBeVisible();
        }
      }
      // A card with no figures takes no chip: dating an absence says nothing. `.tnum` is the
      // numeric-cell class and `.v` the stat-tile value, which between them are every figure this
      // site renders.
      const unchipped = await page.locator(".card").evaluateAll((nodes) =>
        nodes
          .filter((n) => n.querySelector(".tnum, .v"))
          .filter((n) => !n.querySelector(".year-chip"))
          .map((n) => n.querySelector("h2")?.textContent?.trim() ?? "(no heading)"),
      );
      for (const heading of unchipped) missing.push(`${route}: ${heading}`);
    }
    expect(missing, "a card showing figures has to say what year they are on").toEqual([]);
  });

  test("no word is fused to the tag beside it", async ({ page }) => {
    /*
     * Astro trims a newline between an element and adjacent text, the same as JSX. So a paragraph
     * reflowed across lines silently loses a space:
     *
     *     ... for only <strong>{count(n)}</strong>
     *     of the 606 ...          ->   "219of the 606"
     *
     * Two of these shipped in the de-literalisation of this very card, and the scan that found
     * them turned up nine more that had been live for much longer — "computed by<code>", "in
     * proportion to<strong>", "cost input refresh</a>question". Every one is invisible in the
     * source, which reads correctly, and visible only in the output.
     *
     * The check is deliberately narrow: a letter immediately against an inline tag boundary. A
     * tag against punctuation is normal — `<strong>219</strong>,` — and so is a tag against
     * another tag. The one punctuation mark that is *not* normal against a tag is the middot
     * separator, and that is checked in `scripts/check-dist-links.ts` rather than here, because
     * the page it was live on is not in `ROUTES_WITH_FIGURES` and that sweep reads every page.
     *
     * # This is half the defect, and the other half is in the unit suite
     *
     * The pattern above needs a tag to match on. The same trimming with no tag involved — prose on
     * one line and `{count(…)}` opening the next — renders `model.294` as a single text node, and
     * nothing in the output distinguishes it from a sentence that meant to say that: scanning
     * rendered text for a letter beside a digit finds 13,441 matches across 400 pages, essentially
     * all of them `textContent` running across a table cell.
     *
     * So that half is checked at the source instead, in `tests/unit/fusion.spec.ts`, which had six
     * of them live when it was written. The two are complements and neither subsumes the other:
     * this one sees defects the source scan cannot infer, in components and interpolated strings;
     * that one sees the shape no rendered output records.
     */
    const fused: string[] = [];
    for (const route of ROUTES_WITH_FIGURES) {
      await page.goto(route);
      const html = await page.content();
      for (const match of html.matchAll(
        /(?:<\/(?:strong|em|code|a)>[A-Za-z]|[A-Za-z]<(?:strong|em|code)\b)/g,
      )) {
        const at = match.index ?? 0;
        fused.push(`${route}: …${html.slice(Math.max(0, at - 40), at + 40).replace(/\s+/g, " ")}…`);
      }
    }
    expect(fused, "a newline between text and an inline tag is trimmed, not rendered").toEqual([]);
  });

  test("the chips on one page name more than one year, and say which reckoning each is", async ({
    page,
  }) => {
    // The proof the feature is doing something. If every chip on a district page read `FY2027`
    // the page would be exactly as misleading as it was before, and passing the test above.
    await page.goto(`/district/${CLEVELAND}/taxes`);
    const chips = page.locator(".year-chip");
    await expect(chips).not.toHaveCount(0);

    const kinds = await chips.evaluateAll((nodes) => [
      ...new Set(nodes.map((n) => n.getAttribute("data-kind"))),
    ]);
    expect(kinds).toContain("tax");
    expect(kinds).toContain("fiscal");

    /*
     * And the long form is on the element, because `2024` alone does not say it is a tax year.
     *
     * It is the button's own `aria-label` rather than a `title`. A `title` reached a mouse and
     * nothing else — no keyboard, no touch screen, and not announced by most screen readers —
     * which for 12,449 chips meant the distinction this whole feature exists to draw was
     * available only to a pointer. See `yearChip` in `src/lib/year.ts`.
     */
    await expect(chips.filter({ hasText: /^2024$/ }).first()).toHaveAttribute(
      "aria-label",
      /calendar year of valuation and levy/,
    );
  });

  test("a defined term is reachable by keyboard and readable with no script", async ({
    browser,
  }) => {
    /*
     * Three constraints, and the shape has to satisfy all three: no script — this site's filter
     * and basis toggle both work without it — no hover, because touch has none, and announced
     * once, so no `title` beside the `aria-describedby`.
     *
     * Run in a context with JavaScript disabled, which is the only way to assert the first of
     * those rather than assume it.
     */
    const context = await browser.newContext({ javaScriptEnabled: false });
    const noScript = await context.newPage();
    await noScript.goto(`/district/${CLEVELAND}/taxes`);

    const trigger = noScript.locator("button.term").first();
    await expect(trigger).toBeVisible();

    // The definition is in the document, not fetched and not injected.
    const id = await trigger.getAttribute("aria-describedby");
    expect(id).toBeTruthy();
    const definition = noScript.locator(`#${id}`);
    await expect(definition).toHaveCount(1);
    await expect(definition).toContainText(/H\.B\. 920|need|weighted/);

    // And described while the definition is closed, which is `display: none` (#601): a hidden
    // target is still read for `aria-describedby`, so a closed panel costs the term nothing.
    await expect(definition).toBeHidden();
    await expect(trigger).toHaveAccessibleDescription(/H\.B\. 920|need|weighted/);

    // Not announced twice: a `title` would be read alongside the description.
    await expect(trigger).not.toHaveAttribute("title", /./);

    // Focus reveals it, so a keyboard reaches what a pointer does.
    await trigger.focus();
    await expect(definition).toBeVisible();
    await context.close();
  });

  test("states the model year in the footer, and the contract on /method", async ({ page }) => {
    /*
     * The model year, which is what the footer is for. The contract version moved to /method in
     * #571: the footer is read by people arriving from a shared link, and a version number is for
     * the technical reader who looks for it there.
     *
     * It used to assert on `FY27` inside the provenance paragraph, and that paragraph no longer
     * names any year — it said "millage is TY2023" while the data said 2024, so the years moved
     * into `series_years` where they are derived. The model year is still here, from
     * `bundle.fiscal_year`, and it is the one the footer is entitled to state because the footer
     * is about the feed rather than about any one card.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator("footer")).toContainText("FY2027 model");
    await expect(page.locator("footer")).not.toContainText("Bundle contract");
    // And it does not quietly grow the years back, which is how the paragraph went stale.
    await expect(page.locator("footer")).not.toContainText("TY20");
    await page.goto("/method");
    await expect(page.locator("#provenance tr", { hasText: "Bundle contract" })).toContainText(
      REQUIRED_CONTRACT,
    );
  });

  test("the footer reports the build-time formula check", async ({ page }) => {
    // The central invariant, as the footer states it. Both halves are counted: the simulation
    // checkpoints and the forecasts, which are gated separately.
    await page.goto("/");
    await expect(page.locator("footer")).toContainText(
      "Every figure is checked against 13 reference scenarios and 4 reference forecasts before the site is built.",
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
    // And the casino block, for the strongest version of the same reason: no lever in the
    // scenario builder can move money that never passes through an appropriation, and a browser
    // holding it would be one property access from a per-pupil figure whose denominator is not in
    // the feed at all.
    expect(slim.districts[0]).not.toHaveProperty("casino");
    expect(slim).not.toHaveProperty("casino");
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
    // And on the reach view, for the same reason and the same one extra script: its cloud is
    // redrawn on every lever tick.
    expect(await weigh("/scenario/reach")).toBe(2);
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

  test("the district index still shows a distribution", async ({ page }) => {
    /*
     * The reason the six strips are rendered at build rather than drawn on sort. A reader with no
     * script cannot change which one is shown and does not get an empty frame either: the default
     * is state aid per pupil, this site's central measure and the honest one for a page nobody has
     * sorted.
     */
    await page.goto("/districts");
    await expect(page.locator("#district-measures .measure:visible")).toHaveCount(1);
    await expect(page.locator('.measure[data-measure="aid"] svg.plot:visible')).toBeVisible();
    await expect(page.locator('.measure[data-measure="aid"]')).toContainText("State aid per pupil");
    // And the table it summarises is complete, as it always was.
    await expect(page.locator("#district-table tbody tr")).toHaveCount(609);
  });

  test("the contents list is navigation, not a script", async ({ page }) => {
    // Derived at build from the rendered body, emitted as plain links. A reader with no script
    // gets the same list as everyone else, which is the whole reason it is not built on load.
    await page.goto("/method");
    const entries = page.locator("main nav.contents a");
    expect(await entries.count()).toBeGreaterThan(3);
    await entries.first().click();
    await expect(page).toHaveURL(/#computed-twice$/);
    await expect(page.locator("#computed-twice")).toBeInViewport();
  });

  test("a section anchor is a link and needs nothing running", async ({ page }) => {
    /*
     * `section.ts` chose a bare `<a href="#…">` over a copy-to-clipboard button for the reason
     * `BasisToggle.astro` chose two radios over a script: a third of this suite runs here, and a
     * control that dies without JavaScript is worse than no control. Fragment navigation is what a
     * browser does natively, so this asserts the whole feature works in the half of the site that
     * never loads a script.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const anchor = page.locator(".card#base-cost > h2 a.section-anchor");
    await expect(anchor).toBeVisible();
    await expect(anchor).toHaveAttribute("href", "#base-cost");
    await anchor.click();
    await expect(page).toHaveURL(/#base-cost$/);
    await expect(page.locator("#base-cost")).toBeInViewport();
  });

  test("a district's figures are all present", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator("h1")).toHaveText("Cleveland Municipal");
    await expect(page.getByRole("heading", { name: /Where the state aid comes from/ })).toBeVisible();
    await expect(page.locator(".tile .v").first()).not.toBeEmpty();
    // Charts are build-time SVG, not a canvas drawn on load.
    await expect(page.locator("svg.plot:visible").first()).toBeVisible();
  });

  test("the district index lists every district", async ({ page }) => {
    await page.goto("/districts");
    await expect(page.locator("#district-table tbody tr")).toHaveCount(609);
  });

  test("the homepage's field still lands on the index, and the index is the whole table", async ({
    page,
  }) => {
    /*
     * The other half of `?q=`. With no script the form is still a form: Enter makes the same
     * address, and the index it reaches is what a GET to it has always been — every row, shown,
     * for the browser's own find. A filter that hid rows without script would be a page a reader
     * could not undo.
     */
    await page.goto("/");
    await page.locator("#home-q").fill(CLEVELAND);
    await page.locator("#home-q").press("Enter");
    await expect(page).toHaveURL(new RegExp(`/districts\\?q=${CLEVELAND}$`));
    await expect(page.locator("#district-table tbody tr:visible")).toHaveCount(609);
  });

  test("the section menus still open, and their links still go somewhere", async ({ page }) => {
    // The reason they are `<details>` rather than a scripted menu. Four of the six entries in the
    // bar are disclosures, so a menu that needed script to open would put most of the site behind
    // JavaScript, on a site whose whole point is that nothing is.
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

  test("two entries are links and four are disclosures, and every one of them opens", async ({
    page,
  }) => {
    /*
     * #548 put two flat links back in the bar — `Find a district` and `Change the formula` — so a failure
     * in the disclosure machinery leaves those two reachable and nothing else. The four menus are
     * everything else the site holds, and each has to open with nothing running.
     */
    await page.goto("/");
    const flat = page.locator("header.site nav a.menu-place");
    await expect(flat).toHaveText(["Find a district", "Change the formula"]);
    await expect(flat.nth(0)).toBeVisible();
    await expect(flat.nth(0)).toHaveAttribute("href", "/districts");
    await expect(flat.nth(1)).toHaveAttribute("href", "/scenario");
    const menus = page.locator("header.site nav details.menu");
    await expect(menus).toHaveCount(4);

    /*
     * By position and not by `filter({ hasText })`: the panels carry prose, `hasText` is a
     * case-insensitive substring, and `Library` holds a run headed "Formula" and a class called
     * "Scenario" — a filter on either word matches a menu it does not name.
     */
    for (const [index, label] of ["Places", "Analysis", "Library", "About"].entries()) {
      const menu = menus.nth(index);
      await expect(menu.locator("summary"), `entry ${index} is not ${label}`).toHaveText(label);
      const first = menu.locator(".menu-panel a").first();
      await expect(first, `${label} has no links`).toBeHidden();
      await menu.locator("summary").click();
      await expect(first, `${label} did not open`).toBeVisible();
      await menu.locator("summary").click();
    }
  });

  test("at phone width the bar is one disclosure, and it still opens without script", async ({
    page,
  }) => {
    /*
     * The other half of the no-JS contract, at the width #189 was about.
     *
     * Above 820px the five menus sit loose in the bar and the outer `<details>` is neutralised by
     * CSS — its summary hidden, its content forced visible. Below 820px that CSS does not apply
     * and the outer one is a real disclosure, so reaching `/counties` takes two opens rather than
     * one. Both have to work with JavaScript off, and only the wide path was covered.
     *
     * This is also the check that the neutralisation is doing what it claims. If
     * `::details-content { content-visibility: visible }` ever stopped applying, the desktop tests
     * above would fail and this one would keep passing — they are the two sides of the same
     * mechanism and neither on its own says it works.
     */
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/");

    const bar = page.locator("header.site nav details.menu-all");
    const places = bar.locator("details.menu").filter({ hasText: "Places" });
    await expect(bar.locator("> summary")).toBeVisible();
    await expect(places.locator("summary")).toBeHidden();

    await bar.locator("> summary").click();
    await expect(places.locator("summary")).toBeVisible();
    await places.locator("summary").click();
    await places.locator('a[href="/counties"]').click();
    await expect(page).toHaveURL(/\/counties$/);
    await expect(page.locator("h1")).toBeVisible();
  });

  test("Library names every class with what is behind it, under a heading saying what it is for", async ({
    page,
  }) => {
    /*
     * The bar reads `Library` out of `.yidam/corpus/`, and the unit suite checks it selects every
     * class. This asserts the selection survives to the page with its two readings: the heading
     * over each run, and the count inside each class's label — a panel of twenty ontology names
     * with neither is an index nobody sorted.
     */
    await page.goto("/");
    const library = page.locator("header.site nav details.menu").nth(2);
    await library.locator("summary").click();
    await expect(library.locator(".menu-heading")).toHaveText([
      "Law",
      "Formula",
      "Institutions",
      "Proposals",
      "The record",
    ]);
    // Every class index, each carrying its count. Eighteen classes today, read off the page
    // rather than typed, so the claim is that each one it holds is counted.
    const classes = library.locator('a[href^="/wiki/"]:not([href="/wiki/source"]):not([href="/wiki/decision"])');
    expect(await classes.count()).toBeGreaterThan(15);
    for (const label of await classes.locator(".menu-label").allTextContents()) {
      expect(label, "a class link without its count").toMatch(/ \(\d+\)$/);
    }
  });

  test("the group holding the current page is marked, and the page itself is marked inside it", async ({
    page,
  }) => {
    // Two different claims: `page` on the link a reader is on, `true` on the group containing it.
    // Marking the summary as the current *page* would tell a screen reader the reader is on a
    // thing that is not a destination.
    await page.goto("/history");
    const analysis = page.locator("header.site nav details.menu").nth(1);
    await expect(analysis.locator("summary")).toHaveAttribute("aria-current", "true");
    await analysis.locator("summary").click();
    await expect(analysis.locator('a[href="/history"]')).toHaveAttribute("aria-current", "page");
  });

  test("a flat entry is the page on its own address, and holds the pages under it", async ({
    page,
  }) => {
    // The same two claims for the two links, which have no summary to carry the second: a district
    // page sits under `Find a district` without being `/districts`.
    const find = page.locator('header.site nav a.menu-place[href="/districts"]');
    await page.goto("/districts");
    await expect(find).toHaveAttribute("aria-current", "page");
    await page.goto(`/district/${CLEVELAND}`);
    await expect(find).toHaveAttribute("aria-current", "true");
    await page.goto("/history");
    await expect(find).not.toHaveAttribute("aria-current", /.*/);
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
    // `main` rather than `.prose-body`: a node now renders its description, its findings and each
    // of its revisions in separate prose cards, so the singular locator is ambiguous. The claim
    // being made is about the page, not about which card the phrase landed in.
    await expect(page.locator("main")).toContainText("20 mills");
  });

  test("the scenario route says it needs JavaScript, and why", async ({ page }) => {
    await page.goto("/scenario");
    // Located by role rather than by text: Playwright's text engine does not descend into
    // `<noscript>`, even in a context where the parser has turned its contents into real DOM.
    await expect(
      page.getByRole("heading", { name: "Re-running the formula needs JavaScript" }),
    ).toBeVisible();
  });
});

/*
 * What a page tells the browser to fetch before it has finished reading the first file.
 *
 * Every page loads one module and that module's first line imports another, so the chain was:
 * parse the HTML, fetch the stub, parse it, discover the import, fetch that — two serial round
 * trips before anything ran, for about 2 KB, on all 3,487 pages. The scenario routes went a level
 * deeper. `preloadModules` in `astro.config.mjs` reads the import graph off the emitted chunks and
 * declares it.
 *
 * That every page's declarations match its chunk closure, across the whole build, is checked in
 * `tests/dist/payload.spec.ts` — it is a question about the emitted files and needs no browser.
 * What is here is the other half: that the declarations are what the browser actually fetches.
 */
test.describe("module preloading", () => {
  test("the preloads are what the browser actually fetches", async ({ page }) => {
    // The artefact check above says the links are right; this says the browser agrees they are
    // modules and asks for them, rather than treating them as an unknown `rel` and ignoring them.
    const asked = new Set<string>();
    page.on("request", (request) => {
      if (request.url().includes("/_astro/")) asked.add(new URL(request.url()).pathname);
    });
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator("svg.plot:visible").first()).toBeVisible();

    const preloads = await page.locator('link[rel="modulepreload"]').evaluateAll((links) =>
      links.map((l) => new URL((l as HTMLLinkElement).href).pathname),
    );
    expect(preloads.length, "the page declares at least one").toBeGreaterThan(0);
    for (const href of preloads) expect([...asked], `${href} was fetched`).toContain(href);
  });
});
