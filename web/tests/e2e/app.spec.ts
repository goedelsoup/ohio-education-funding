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
 * on a missing route. The 404 page is now static — it names four places to go and guesses at
 * nothing — so there is no behaviour left behind it for a unit test to hold either.
 */

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

import { expect, test, type Locator, type Page } from "@playwright/test";

import { REQUIRED_CONTRACT } from "../../src/lib/types.ts";

/**
 * A heading's own words, without the anchor that heads it.
 *
 * Every section heading on the site now opens with a link to itself — see `src/lib/section.ts` —
 * so `innerText` on one reads "# Context". Assertions about what a heading *says* want the words,
 * and stripping them here keeps those assertions exact rather than loosening each one to a
 * substring match, which is what would quietly stop them catching a heading that changed.
 *
 * The other half of the same hazard has no helper because it needs none: a page that lists its
 * sections carries every heading's text twice, once in the list and once on the section, so
 * `getByText` on a heading is a strict-mode violation rather than a match. Ask for the heading —
 * `getByRole("heading", { name: /…/ })`, with a pattern because the accessible name of one now
 * opens with "Link to this section".
 */
const headingText = (heading: Locator): Promise<string> =>
  heading.evaluate((node) => {
    const clone = node.cloneNode(true) as HTMLElement;
    clone.querySelector("a.section-anchor")?.remove();
    // The chip carries its reckoning in a panel beside it now, so a heading's `textContent`
    // includes that sentence unless the whole wrapper goes. See `yearChip` in `src/lib/year.ts`.
    clone.querySelector(".year-chip-wrap")?.remove();
    return (clone.textContent ?? "").replace(/\s+/g, " ").trim();
  });

/*
 * Why chart selectors here say `svg.plot:visible`.
 *
 * Every chart is in the document twice — one drawing per width, of which the stylesheet shows one.
 * See `WIDTHS` in `src/lib/plot/spec.ts` for why. So `[data-chart="fan"] svg` matches two elements
 * and every assertion written against it is a strict-mode violation rather than a failure that
 * says anything.
 *
 * `:visible` resolves it to the drawing this viewport is actually being shown, which is also the
 * one the assertion means: what a reader sees. Playwright's default viewport is 1280 wide, so that
 * is the wide drawing everywhere except the phone-width tests, which set their own viewport and
 * get the narrow one from the same selector without saying so.
 */

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
    //
    // `application/ld+json` is the one exception, and it is an exception to this test rather than
    // to the directive. `script-src` governs elements the browser would *execute*; an element
    // whose type is not a JavaScript MIME type is a data block, never parsed as script and never
    // blocked. `/data` carries one — a `schema.org/Dataset` naming the three downloads, which is
    // the one page on this site where the vocabulary does work prose cannot. It has to be inline
    // because a data block has no `src`.
    //
    // Every `<` in it is written as `<`, so the JSON cannot close its own element early
    // whatever a description string turns out to contain.
    const offenders: string[] = [];
    for (const file of html(DIST)) {
      const body = readFileSync(file, "utf8");
      for (const match of body.matchAll(/<script(?![^>]*\bsrc=)([^>]*)>([\s\S]*?)<\/script>/gi)) {
        if (/type\s*=\s*["']application\/ld\+json["']/i.test(match[1] ?? "")) continue;
        if ((match[2] ?? "").trim() !== "") offenders.push(file.slice(DIST.length + 1));
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
        /(?:src|href)\s*=\s*["'](?:https?:)?\/\/(?!ohio-education-funding\.pages\.dev)[^"']+/gi,
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

/**
 * The three files a machine reads that no reader ever opens.
 *
 * `robots.txt`, the JSON-LD on `/data`, and the response headers in `_headers`. Each is invisible
 * from every page of the site and from `vite preview`, and each was wrong: there was no
 * `robots.txt` at all, so Cloudflare served an auto-injected one that is entirely comments and
 * names no sitemap; there was no structured data anywhere, on a site whose `/data` route is
 * literally a dataset with three distributions; and the CSV route's `Content-Disposition` was
 * discarded by the static build, so the fiscal year the figures are on reached nobody.
 *
 * They are checked against the artefact rather than through a browser for the reason the block
 * above gives about `og:` tags: the consumers are crawlers and unfurlers, none of which is
 * available to a test, and the failure they produce is silence.
 */
test.describe("what a machine reads", () => {
  const DIST = join(import.meta.dirname, "../../dist");
  const feed = JSON.parse(readFileSync(join(DIST, "data", "bundle.json"), "utf8"));

  test("robots.txt names the sitemap, at the host the canonical links use", () => {
    const robots = readFileSync(join(DIST, "robots.txt"), "utf8");
    const sitemap = robots.match(/^Sitemap:\s*(\S+)$/m)?.[1];
    expect(sitemap, "no Sitemap: line — the whole reason to ship this file").toBeTruthy();

    // The host is written out in `robots.txt` and configured in `astro.config.mjs`, and a crawler
    // that follows a sitemap URL on the wrong host gets nothing. Take the site's own canonical
    // link as the authority rather than restating the hostname here for a third time.
    const canonical = readFileSync(join(DIST, "index.html"), "utf8").match(
      /<link rel="canonical" href="([^"]*)"/i,
    )?.[1];
    expect(new URL(sitemap!).origin).toBe(new URL(canonical!).origin);

    // And it has to point at something that was built.
    expect(existsSync(join(DIST, new URL(sitemap!).pathname.slice(1)))).toBe(true);
  });

  test("/data describes itself as a Dataset, and every distribution it names exists", () => {
    const body = readFileSync(join(DIST, "data.html"), "utf8");
    const block = body.match(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/i)?.[1];
    expect(block, "/data carries no structured data").toBeTruthy();

    const data = JSON.parse(block!);
    expect(data["@type"]).toBe("Dataset");
    // Read from the feed, not typed — so a contract bump moves this without anyone editing it,
    // which is the property that makes the block worth emitting rather than a liability.
    expect(data.version).toBe(feed.contract_version);
    expect(data.version).toBe(REQUIRED_CONTRACT);

    const named = data.distribution.map((d: { contentUrl: string }) => new URL(d.contentUrl).pathname);
    expect(named).toEqual(["/data/districts.csv", "/data/bundle.json", "/data/panel.json"]);
    for (const path of named) {
      expect(existsSync(join(DIST, path.slice(1))), `${path} is named but was not built`).toBe(true);
    }

    // The three the page links are the three it declares. A fourth download added to the table
    // and not to the block would leave the structured data quietly describing an older site.
    const linked = [...body.matchAll(/href="(\/data\/[^"]+)"/g)].map((m) => m[1]!);
    expect([...new Set(linked)].sort()).toEqual([...named].sort());
  });

  test("the CSV download's headers survive the static build", () => {
    /*
     * `output: "static"` discards the headers an endpoint's `Response` carries — Astro keeps the
     * body and writes a file. So the `Content-Disposition` in `src/pages/data/districts.csv.ts`
     * reached `astro dev` and `vite preview` and never the deploy, and the comment in that route
     * about provenance travelling in the filename described a local session.
     *
     * The `csv-download-headers` integration in `astro.config.mjs` appends them to `dist/_headers`,
     * reading the fiscal year out of the same feed. This asserts against the built `_headers`
     * because that is the file Cloudflare Pages reads; nothing else in this repository looks at it.
     */
    const headers = readFileSync(join(DIST, "_headers"), "utf8");
    expect(headers).toContain("/data/districts.csv");
    expect(headers).toContain(
      `Content-Disposition: attachment; filename="ohio-school-funding-fy${feed.fiscal_year}.csv"`,
    );
    // The block is appended, and appending to a file that had gone missing would produce a
    // `_headers` holding only this and silently dropping the CSP. The integration throws in that
    // case; this is the assertion that the throw is about something real.
    expect(headers).toContain("Content-Security-Policy:");
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
    for (const suffix of ["", "/finances", "/outcome", "/taxes", "/scenario"]) {
      await page.goto(`/district/${CLEVELAND}${suffix}`);
      if (suffix === "/scenario") {
        await expect(page.locator("#scenario-out .card")).not.toHaveCount(0);
      }
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
  `/district/${CLEVELAND}/scenario`,
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
      if (route.endsWith("/scenario")) {
        await expect(page.locator("#scenario-out .card")).not.toHaveCount(0);
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

    // Not announced twice: a `title` would be read alongside the description.
    await expect(trigger).not.toHaveAttribute("title", /./);

    // Focus reveals it, so a keyboard reaches what a pointer does.
    await trigger.focus();
    await expect(definition).toBeVisible();
    await context.close();
  });

  test("states the provenance of the figures in the footer", async ({ page }) => {
    /*
     * The model year and the contract, which is what the footer is for.
     *
     * It used to assert on `FY27` inside the provenance paragraph, and that paragraph no longer
     * names any year — it said "millage is TY2023" while the data said 2024, so the years moved
     * into `series_years` where they are derived. The model year is still here, from
     * `bundle.fiscal_year`, and it is the one the footer is entitled to state because the footer
     * is about the feed rather than about any one card.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator("footer")).toContainText("FY2027 model");
    await expect(page.locator("footer")).toContainText("Bundle contract");
    // And it does not quietly grow the years back, which is how the paragraph went stale.
    await expect(page.locator("footer")).not.toContainText("TY20");
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

  test("the section menus still open, and their links still go somewhere", async ({ page }) => {
    // The reason they are `<details>` rather than a scripted menu, and the reason it matters more
    // than it did. Every entry in the bar is a disclosure now — the flat `Statewide` and
    // `Scenario` links went into `Places` and `Research` — so a menu that needed script to open
    // would put the entire site behind JavaScript, on a site whose whole point is that nothing is.
    await page.goto("/");
    const places = page.locator("header.site nav details.menu").filter({ hasText: "Places" });
    await expect(places.locator("a")).toHaveCount(6);
    await expect(places.locator('a[href="/counties"]')).toBeHidden();

    await places.locator("summary").click();
    await expect(places.locator('a[href="/counties"]')).toBeVisible();
    await places.locator('a[href="/counties"]').click();
    await expect(page).toHaveURL(/\/counties$/);
    await expect(page.locator("h1")).toBeVisible();
  });

  test("all five entries are disclosures, and every one of them opens", async ({ page }) => {
    /*
     * Places was the only group this suite had ever opened without script, back when two of the
     * five entries were plain links and a failure in the disclosure machinery still left
     * `Statewide` and `Scenario` reachable. Both of those went into menus. There is no longer any
     * flat link in the bar at all, so a `<details>` that needed JavaScript would now put every
     * section of the site behind it rather than three fifths of one.
     */
    await page.goto("/");
    const menus = page.locator("header.site nav details.menu");
    await expect(menus).toHaveCount(5);
    await expect(page.locator("header.site nav > a")).toHaveCount(0);

    /*
     * By position and not by `filter({ hasText })`, which is how this was first written and which
     * does not work any more: the panels carry generated prose, `hasText` is a case-insensitive
     * substring, and "Formula" matches three menus — its own, `Law` (whose note under H.B. 153
     * reads "Bridge Formula") and `Research` ("re-run the formula").
     *
     * Positions are worth asserting in their own right. "After Places" is where the two lifted
     * corpus axes were asked to go, and nothing else says so.
     */
    for (const [index, label] of ["Places", "Law", "Formula", "Research", "Reference"].entries()) {
      const menu = menus.nth(index);
      await expect(menu.locator("summary"), `entry ${index} is not ${label}`).toHaveText(label);
      const first = menu.locator(".menu-panel a").first();
      await expect(first, `${label} has no links`).toBeHidden();
      await menu.locator("summary").click();
      await expect(first, `${label} did not open`).toBeVisible();
      await menu.locator("summary").click();
    }
  });

  test("a menu note says why a link is in the menu, not what the link already says", async ({
    page,
  }) => {
    /*
     * Two thirds of the bar is generated from `.yidam/corpus/`, and the second line under each
     * generated link is the derivation showing its work: which rule admitted an act, which year a
     * regime began. This asserts the notes survive to the page — the unit suite checks the rules
     * select correctly, and a correct selection rendered without its reason is a menu of seven
     * bill numbers a reader has no way to tell apart.
     */
    await page.goto("/");
    const law = page.locator("header.site nav details.menu").filter({ hasText: "Law" });
    await law.locator("summary").click();
    const fsfp = law.locator('a[href="/wiki/legislation/hb-110-2021"]');
    await expect(fsfp.locator(".menu-label")).toHaveText("Am. Sub. H.B. 110 (2021)");
    await expect(fsfp.locator(".menu-note")).toHaveText("Fair School Funding Plan");
    // The rule that maintains itself: whichever act is the current budget names the biennium.
    await expect(law.locator(".menu-note", { hasText: /^appropriates for FY/ })).toHaveCount(1);
  });

  test("the group holding the current page is marked, and the page itself is marked inside it", async ({
    page,
  }) => {
    // Two different claims: `page` on the link a reader is on, `true` on the group containing it.
    // Marking the summary as the current *page* would tell a screen reader the reader is on a
    // thing that is not a destination.
    await page.goto("/history");
    const research = page.locator("header.site nav details.menu").filter({ hasText: "Research" });
    await expect(research.locator("summary")).toHaveAttribute("aria-current", "true");
    await research.locator("summary").click();
    await expect(research.locator('a[href="/history"]')).toHaveAttribute("aria-current", "page");
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

  test("the root is a front door, and the statewide panel is a place", async ({ page }) => {
    /*
     * The move, asserted from both ends.
     *
     * `/` was the statewide panel for this site's whole life. It stopped being it when the bar
     * grew from three axes to five: a reader landing on a dense financial view has been handed an
     * answer before they asked anything, and the two axes lifted out of `.yidam/corpus/` had
     * nowhere to be introduced.
     *
     * Checking only the destination is what makes a move indistinguishable from a copy — a page
     * left behind at the old address passes every assertion about the new one. So both halves:
     * what `/statewide` carries, and what `/` no longer does.
     */
    await page.goto("/statewide");
    await expect(page.locator("h1")).toHaveText("Statewide");
    await expect(page.getByRole("heading", { name: /Who is on the guarantee/ })).toBeVisible();

    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("Ohio school funding");
    await expect(page.getByRole("heading", { name: /Who is on the guarantee/ })).toHaveCount(0);
    await expect(page.locator(".basis-panel")).toHaveCount(0);

    // No tab for it, by design: the brand mark is the way back and the bar does not say it twice.
    await expect(page.locator('header.site nav a[href="/"]')).toHaveCount(0);
    await expect(page.locator("a.brand")).toHaveAttribute("href", "/");
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

  test("a fragment that names nothing leaves the reader where they are", async ({ page }) => {
    /*
     * The redirect table above was an object literal, and an object literal answers for every name
     * on `Object.prototype`. `/#toString` looked up `moved["toString"]`, got
     * `Function.prototype.toString` — truthy, and unequal to `location.pathname`, so both guards
     * passed — and handed `location.replace` a function. It stringifies to
     * `function toString() { [native code] }`, which resolves as a relative URL, so the reader was
     * bounced off the front page onto a 404 whose path was source code.
     *
     * These are not hypothetical fragments. `#constructor` and `#toString` are what a fuzzer, a
     * link checker and an autocompleting URL bar all produce, and the failure is on the homepage.
     *
     * The table is a `Map` now. `#nowhere` is here beside them as the control: an unknown name
     * that is *not* a prototype key has always been handled correctly, and if it ever stops being
     * handled correctly this test says which of the two things broke.
     */
    for (const fragment of ["#toString", "#constructor", "#valueOf", "#nowhere"]) {
      await page.goto(`/${fragment}`);
      // Still on `/`, still carrying the fragment it arrived with — a fragment naming no route is
      // an anchor, and an anchor that matches no element is a no-op rather than a navigation.
      await expect(page, `${fragment} redirected off the homepage`).toHaveURL(
        new RegExp(`/${fragment}$`),
      );
      await expect(page.locator("h1")).toHaveText("Ohio school funding");
    }
  });

  test("/compare is reachable by following links, not only from the global menu", async ({
    page,
  }) => {
    /*
     * It was in the bar and nowhere else: `routes.compare()` was called by no template in the
     * repository, so a reader on a district page — holding exactly the figures the comparison is
     * built from — had no way to get to it except by going back to the menu and starting over.
     *
     * The two entry points are the two places where the question "compared to whom" is already
     * being asked. A district's position card places it against all 609 at once; a county's
     * spread card names that county's richest and poorest district and prints four figures for
     * each. Both now link on, and the link carries the district it came from.
     */
    await page.goto(`/district/${NORTHERN}`);
    const fromDistrict = page.locator(`#position a[href^="/compare"]`);
    await expect(fromDistrict).toHaveAttribute("href", `/compare?a=${NORTHERN}`);
    await fromDistrict.click();

    // And the runner honours it: the district that was linked from is on one side, and the other
    // side is not the same district — a half-specified pair used to be thrown away entirely.
    await expect(page.locator("#cmp-a")).toHaveValue(NORTHERN);
    await expect(page.locator("#cmp-b")).not.toHaveValue(NORTHERN);
    await expect(page.locator("#comparison")).toBeVisible();

    await page.goto("/county/cuyahoga");
    const fromCounty = page.locator(`#spread a[href^="/compare"]`);
    await expect(fromCounty).toHaveAttribute("href", /^\/compare\?a=\d{6}&b=\d{6}$/);
  });

  test("the comparison table says which year its tax figures are on", async ({ page }) => {
    /*
     * Two rows here are on a tax year, and the footnote under the table names three *other* years
     * — the fiscal year of the model, the year of the valuations, the year of the expenditure. So
     * the millage rows read as though they belonged to one of those, and a tax year is eleven
     * months out of step with the fiscal year it funds.
     *
     * "2024 tax year" in words rather than a bare "2024", which is the rule `src/lib/year.ts`
     * sets and the reason it gives: `FY2024` and `2024` differ by a prefix, and a prefix reads as
     * typography rather than as a claim about which period a figure covers.
     */
    await page.goto("/compare");
    const label = page.locator("#comparison tbody th").filter({ hasText: "Voted operating" });
    await expect(label).toContainText(/\d{4} tax year/);
    await expect(
      page.locator("#comparison tbody th").filter({ hasText: "Effective Class 1" }),
    ).toContainText(/\d{4} tax year/);
  });

  test("the sitemap lists the district pages", async ({ request }) => {
    const index = await request.get("/sitemap-index.xml");
    expect(index.status()).toBe(200);
    const first = await request.get("/sitemap-0.xml");
    expect(await first.text()).toContain(`/district/${CLEVELAND}`);
  });

  test("every section in the navigation resolves to a page", async ({ page }) => {
    // A menu item pointing at a route the build does not emit looks exactly like one that works,
    // right up until someone opens it. That risk grew: two thirds of the bar is now generated
    // from `.yidam/corpus/` rather than typed, so a node renamed in the corpus writes a dead
    // link into the header of all 3,486 pages and nothing else would notice.
    await page.goto("/");
    const hrefs = await page
      .locator("header.site nav a")
      .evaluateAll((nodes) => nodes.map((n) => (n as HTMLAnchorElement).getAttribute("href")!));
    // Thirty-one across five groups. An exact count rather than a floor, so that dropping an
    // entry fails here and adding one is an acknowledged change — and so that a derivation which
    // quietly stops selecting anything cannot pass by returning an empty menu. It went from
    // thirty to thirty-one when `/legislation` joined the `Law` panel, which is the mechanism
    // working: the count had to be changed on purpose by somebody who knew why.
    expect(hrefs).toHaveLength(31);
    for (const href of hrefs) {
      await page.goto(href);
      await expect(page.locator("h1"), `${href} has no heading`).toBeVisible();
    }
  });
});

/*
 * Charts, at the width they are actually read at.
 *
 * Every chart is drawn twice — see `WIDTHS` in `src/lib/plot/spec.ts` — because a static SVG
 * scaled to a phone takes its axis text down with it. The defect these replace: at 375px the
 * 640-unit drawing was scaled by 0.46 and the tick labels rendered near 4.6px, which is half the
 * size at which text is legible, on the viewport most first visits arrive at. Nothing caught it,
 * because a scaled SVG is correct markup and its `font-size` attribute still says 10.
 *
 * So these measure what the browser paints rather than what the file says. A unit test cannot:
 * the scale factor only exists once the SVG is in a box of a known width.
 */
test.describe("charts on a phone", () => {
  const CHARTED = [
    "/",
    "/statewide",
    "/outcomes",
    "/counties",
    "/history",
    "/method",
    "/districts",
    `/district/${CLEVELAND}`,
    `/district/${CLEVELAND}/finances`,
    `/district/${CLEVELAND}/outcome`,
  ];

  /** Every text mark in a chart the reader can actually see, with the size it is painted at. */
  const paintedText = (page: Page) =>
    page.evaluate(() => {
      const out: { chart: string; text: string; px: number; outside: number }[] = [];
      for (const svg of document.querySelectorAll("svg.plot")) {
        // The drawing for the other width is `display: none`, so it paints nothing and is not
        // what any of this is about.
        if (!svg.getClientRects().length) continue;
        const frame = svg.getBoundingClientRect();
        const box = (svg as SVGSVGElement).viewBox.baseVal;
        const scale = box.width ? frame.width / box.width : 1;
        const chart = svg.closest("[data-chart]")?.getAttribute("data-chart") ?? "unnamed";
        for (const mark of svg.querySelectorAll("text")) {
          const text = (mark.textContent ?? "").trim();
          if (!text) continue;
          const at = mark.getBoundingClientRect();
          out.push({
            chart,
            text,
            px: (parseFloat(getComputedStyle(mark).fontSize) || 10) * scale,
            outside: Math.max(frame.left - at.left, at.right - frame.right),
          });
        }
      }
      return out;
    });

  test("no chart draws its text below 9px", async ({ page }) => {
    // 390px is the common modern phone and 375 the narrowest worth drawing for. The floor is 9px
    // rather than a round 10 because `WIDTHS.narrow` is sized to scale by at least 0.9 there, and
    // the 10px axis marks are the smallest type any of these forms uses.
    await page.setViewportSize({ width: 375, height: 900 });
    const small: string[] = [];
    for (const route of CHARTED) {
      await page.goto(route);
      for (const mark of await paintedText(page)) {
        if (mark.px < 9) small.push(`${route} [${mark.chart}] "${mark.text}" at ${mark.px.toFixed(1)}px`);
      }
    }
    expect(small.slice(0, 10), "chart text a reader cannot read").toEqual([]);
  });

  test("no chart draws a label outside its own frame", async ({ page }) => {
    /*
     * The other half of drawing narrow. Every gutter in `spec.ts` is sized to the text that goes
     * in it, and text does not shrink with the frame — so a name that fits a 640 drawing asks for
     * three quarters of a 320 one. Capped, it has to wrap; uncapped, `Career-technical education`
     * is drawn off the left edge of the viewBox and arrives as `er-technical education`, which
     * reads as a rendering fault rather than as a truncation and is exactly the kind of thing
     * nobody reports.
     */
    await page.setViewportSize({ width: 375, height: 900 });
    const clipped: string[] = [];
    for (const route of CHARTED) {
      await page.goto(route);
      for (const mark of await paintedText(page)) {
        if (mark.outside > 0.5) {
          clipped.push(`${route} [${mark.chart}] "${mark.text}" over by ${Math.round(mark.outside)}px`);
        }
      }
    }
    expect(clipped.slice(0, 10), "chart labels drawn past the edge of the drawing").toEqual([]);
  });

  test("no two chart labels are drawn through each other", async ({ page }) => {
    /*
     * A narrow frame is where the annotation along the foot stops fitting on one line: `/history`
     * drew `FY2009`, `axis starts at 32%, not zero` and `FY2022` across 186px and the centre ran
     * through both years. `axisFoot` measures the row and drops the centre to its own line — and
     * measuring it is only possible here, because a string's drawn width is a fact about the font.
     */
    await page.setViewportSize({ width: 375, height: 900 });
    const through: string[] = [];
    for (const route of CHARTED) {
      await page.goto(route);
      const hits = await page.evaluate(() => {
        const out: string[] = [];
        for (const svg of document.querySelectorAll("svg.plot")) {
          if (!svg.getClientRects().length) continue;
          const marks = [...svg.querySelectorAll("text")]
            .filter((m) => (m.textContent ?? "").trim())
            .map((m) => ({ t: (m.textContent ?? "").trim(), r: m.getBoundingClientRect() }))
            .filter((m) => m.r.width > 0);
          for (let a = 0; a < marks.length; a += 1) {
            for (let b = a + 1; b < marks.length; b += 1) {
              const A = marks[a]!.r;
              const B = marks[b]!.r;
              const over =
                Math.min(A.right, B.right) - Math.max(A.left, B.left) > 1 &&
                Math.min(A.bottom, B.bottom) - Math.max(A.top, B.top) > 1;
              if (over) out.push(`"${marks[a]!.t}" through "${marks[b]!.t}"`);
            }
          }
        }
        return out;
      });
      for (const hit of hits) through.push(`${route}: ${hit}`);
    }
    expect(through.slice(0, 10), "overlapping chart type").toEqual([]);
  });

  test("the reader is shown one of the two drawings, never both and never neither", async ({
    page,
  }) => {
    // The container query is the only thing choosing between them, so a stylesheet that stopped
    // loading, or a breakpoint edited to leave a gap, would show two charts stacked or none at
    // all — both of which read as a broken page rather than as a missing rule.
    for (const width of [375, 500, 640, 900, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      await page.goto(`/district/${CLEVELAND}`);
      const shown = await page.locator(".chart-pair").evaluateAll((pairs) =>
        pairs.map((pair) =>
          [...pair.querySelectorAll(".chart-at")].filter((at) => at.getClientRects().length).length,
        ),
      );
      expect(shown.length, `${width}px`).toBeGreaterThan(0);
      expect([...new Set(shown)], `${width}px shows one drawing per chart`).toEqual([1]);
    }
  });
});

/*
 * The tables that run off the side of a phone.
 *
 * `.scroll` is `overflow-x: auto` and was nothing else: 14,767 boxes in the build, almost all
 * holding a data table wider than the screen, none of them focusable and none of them named. A
 * mouse drags them; a keyboard could not reach the columns past the right edge at all.
 */
test.describe("a table that scrolls sideways", () => {
  test("can be reached and moved by the keyboard", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 900 });
    await page.goto(`/district/${CLEVELAND}/finances`);

    // The first box on the page that genuinely has something behind its edge. Asserting on a box
    // that fits would pass whatever this did.
    const index = await page.locator("div.scroll").evaluateAll((boxes) =>
      boxes.findIndex((box) => box.scrollWidth > box.clientWidth + 1),
    );
    expect(index, "a table wide enough to need scrolling at 375px").toBeGreaterThanOrEqual(0);

    const box = page.locator("div.scroll").nth(index);
    await expect(box).toHaveAttribute("tabindex", "0");
    await expect(box).toHaveAttribute("role", "group");
    // Named from the heading it sits under — see `src/lib/scrollers.ts` for why that is read off
    // the page rather than written beside each of the seventy call sites.
    await expect(box).toHaveAttribute("aria-label", /\S/);

    await box.focus();
    expect(await box.evaluate((el) => document.activeElement === el)).toBe(true);
    for (let i = 0; i < 4; i += 1) await page.keyboard.press("ArrowRight");
    await expect
      .poll(() => box.evaluate((el) => el.scrollLeft), { message: "the keyboard moved it" })
      .toBeGreaterThan(0);
  });

  test("every one of them in the build says what it is", () => {
    /*
     * Scanned over the artefact rather than a page at a time, for the reason the CSP block above
     * gives: the treatment is applied in the layout, so it reaches every route including the ones
     * no test visits, and a sweep of thirteen pages would not notice a route that slipped it.
     *
     * The six boxes with no table are deliberate and excluded at the source — a chart cannot
     * overflow, so a tab stop on one leads nowhere. See `nameScrollers`.
     */
    const DIST = join(import.meta.dirname, "../../dist");
    const walk = (dir: string): string[] =>
      readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
        entry.isDirectory()
          ? walk(join(dir, entry.name))
          : entry.name.endsWith(".html")
            ? [join(dir, entry.name)]
            : [],
      );

    let boxes = 0;
    const bare: string[] = [];
    for (const file of walk(DIST)) {
      const page = readFileSync(file, "utf8");
      for (const match of page.matchAll(/<div\b([^>]*\bclass="scroll"[^>]*)>/g)) {
        const attrs = match[1] ?? "";
        if (!attrs.includes('tabindex="0"')) continue; // a chart wrapper, excluded at the source
        boxes += 1;
        if (!/aria-label="[^"]+"/.test(attrs)) bare.push(`${file.slice(DIST.length + 1)}: ${attrs}`);
      }
    }
    expect(boxes, "the build carries scrolling tables at all").toBeGreaterThan(10_000);
    expect(bare.slice(0, 5), "a focusable box a screen reader cannot name").toEqual([]);
  });
});

/*
 * The values behind the marks, and the reckoning behind a chip.
 *
 * Both were mouse-only. 159,530 marks carried their value in `data-hover` and the layer that read
 * it bound `mousemove` and nothing else; 12,449 chips carried their reckoning in a `title`, which
 * is not announced by most screen readers, cannot be reached from a keyboard, and does not exist
 * on a touch screen at all.
 */
test.describe("a chart's values, beyond the mouse", () => {
  /** What the page is currently saying a mark is worth, by all three channels at once. */
  const reading = (page: Page) =>
    page.evaluate(() => ({
      marked: document.querySelectorAll("svg.plot [data-hover].at").length,
      on: document.querySelector("svg.plot [data-hover].at")?.getAttribute("data-hover") ?? null,
      tip:
        document.querySelector<HTMLElement>("#tip")?.hidden === false
          ? (document.querySelector("#tip")?.textContent ?? "")
          : null,
      said: document.querySelector("#said")?.textContent ?? "",
    }));

  test("the arrow keys walk the marks, and say what each is worth", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const chart = page.locator("svg.plot[tabindex='0']:visible").first();
    await chart.scrollIntoViewIfNeeded();
    await chart.focus();

    // Focusing a chart reads nothing on its own: the reader is on the graphic, not in it.
    expect(await reading(page)).toMatchObject({ marked: 0, tip: null, said: "" });

    await page.keyboard.press("ArrowRight");
    const first = await reading(page);
    expect(first.marked, "exactly one mark carries the cursor").toBe(1);
    expect(first.on).toBeTruthy();
    // The same value by both routes: the tooltip a reader sees, the live region a reader hears.
    expect(first.tip).toBe(first.on);
    expect(first.said).toBe(first.on);

    await page.keyboard.press("ArrowRight");
    const second = await reading(page);
    expect(second.on, "the cursor moved").not.toBe(first.on);
    expect(second.marked, "and did not leave the last one behind").toBe(1);

    await page.keyboard.press("ArrowLeft");
    expect((await reading(page)).on, "and moves back").toBe(first.on);

    await page.keyboard.press("End");
    const end = await reading(page);
    await page.keyboard.press("Home");
    expect((await reading(page)).on, "Home and End are the two ends").not.toBe(end.on);

    await page.keyboard.press("Escape");
    expect(await reading(page), "Escape steps out of the chart").toMatchObject({
      marked: 0,
      tip: null,
      said: "",
    });
  });

  test("a chart is one tab stop, not one per mark", async ({ page }) => {
    /*
     * The reason there is a cursor at all. This district page draws 159 marks a reader can point
     * at; making each of them focusable would put 159 tab stops between the top of the page and
     * the next link, which is a worse defect than the one being fixed.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const counts = await page.evaluate(() => {
      const drawn = [...document.querySelectorAll("svg.plot")].filter(
        (svg) => svg.getClientRects().length,
      );
      return {
        charts: drawn.length,
        stops: drawn.filter((svg) => svg.getAttribute("tabindex") === "0").length,
        marks: drawn.reduce((n, svg) => n + svg.querySelectorAll("[data-hover]").length, 0),
        focusableMarks: document.querySelectorAll("svg.plot [data-hover][tabindex]").length,
      };
    });
    expect(counts.marks, "the page has marks worth reading").toBeGreaterThan(50);
    expect(counts.focusableMarks, "and not one of them is its own tab stop").toBe(0);
    expect(counts.stops, "every chart with marks is reachable").toBeGreaterThan(0);
    expect(counts.stops).toBeLessThanOrEqual(counts.charts);
  });

  test("the tab stop and its instruction arrive together, or not at all", async ({ page }) => {
    // The cursor is script, so the affordance advertising it is added by script too. A `tabindex`
    // baked into the build would be a stop that goes nowhere for a reader running none.
    await page.goto(`/district/${CLEVELAND}`);
    const label = await page.locator("svg.plot[tabindex='0']:visible").first().getAttribute("aria-label");
    expect(label, "the chart still says what it plots").toBeTruthy();
    expect(label, "and that it can be read").toContain("arrow keys");
  });

  test("a tap reaches a value, where there is no pointer to hover", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const shown = await page.evaluate(() => {
      const mark = [...document.querySelectorAll("svg.plot [data-hover]")].find(
        (m) => m.getClientRects().length,
      );
      mark?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      const tip = document.querySelector<HTMLElement>("#tip");
      return { hidden: tip?.hidden, text: tip?.textContent ?? "", want: mark?.getAttribute("data-hover") };
    });
    expect(shown.hidden, "a tap on a mark shows its value").toBe(false);
    expect(shown.text).toBe(shown.want);
  });
});

test.describe("a year chip", () => {
  test("says its reckoning to a keyboard, and to a screen reader", async ({ page }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const wrap = page.locator(".year-chip-wrap").first();
    const chip = wrap.locator("button.year-chip");

    // The whole of what the chip means is the button's own name, so nothing has to be pointed at
    // to reach it. `yearTitle` opens with the label the chip shows, so the visible text is a
    // prefix of the accessible name.
    const label = (await chip.getAttribute("aria-label")) ?? "";
    expect(label).toContain(((await chip.textContent()) ?? "").trim());
    expect(label, "the reckoning, not just the digits").toMatch(/fiscal year|tax year|school year/);
    expect(label, "and where the figure came from").toContain("Source:");

    // And the panel a sighted reader gets opens on focus rather than only on hover.
    const panel = wrap.locator(".year-chip-def");
    await expect(panel).toBeHidden();
    await chip.focus();
    await expect(panel).toBeVisible();
    await expect(panel).toHaveText(label);
  });

  test("no chip in the build hides behind a title", () => {
    /*
     * `title` is what this replaced, and it is the shape the defect would grow back in: it looks
     * right to whoever writes it, because a mouse is what they are testing with.
     *
     * Scanned over the artefact rather than a page at a time, since a chip is written by two
     * functions and rendered on 3,487 pages.
     */
    const DIST = join(import.meta.dirname, "../../dist");
    const walk = (dir: string): string[] =>
      readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
        entry.isDirectory()
          ? walk(join(dir, entry.name))
          : entry.name.endsWith(".html")
            ? [join(dir, entry.name)]
            : [],
      );

    let chips = 0;
    const bad: string[] = [];
    for (const file of walk(DIST)) {
      for (const match of readFileSync(file, "utf8").matchAll(
        /<button\b([^>]*\bclass="year-chip"[^>]*)>/g,
      )) {
        const attrs = match[1] ?? "";
        chips += 1;
        if (attrs.includes("title=")) bad.push(`${file.slice(DIST.length + 1)}: title on a chip`);
        if (!/aria-label="[^"]+"/.test(attrs)) {
          bad.push(`${file.slice(DIST.length + 1)}: a chip with no name`);
        }
      }
    }
    expect(chips, "the build carries chips at all").toBeGreaterThan(10_000);
    expect(bad.slice(0, 5)).toEqual([]);
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

  test("on a phone the longest menu stays inside the viewport, and scrolls if it cannot", async ({
    page,
  }) => {
    /*
     * Below 640px the panel is `position: static`, so an open menu adds its own height to the
     * header's rather than hanging below it — see the note in `app.css` for why absolute
     * positioning is wrong at that width.
     *
     * The header is also `position: sticky; top: 0`. Those two together are the defect: `Law` is
     * seven acts over a class index, each a two-line run, and on a 375px screen the open menu made
     * the header taller than the viewport. A sticky box pinned to y=0 does not move when the
     * document scrolls, and the overflowing entries were *inside* the box rather than under it —
     * so the bottom of the menu could not be reached by any gesture. On the one width the grouped
     * bar exists for.
     *
     * 375×667 is an iPhone SE, the smallest screen worth holding the site to. The assertion is in
     * two halves because either alone can pass while the defect is live: the header must fit, and
     * the last link in the longest menu must be somewhere a reader can actually get to.
     */
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/");

    const law = page.locator("header.site nav details.menu").filter({ hasText: "Law" });
    await law.locator("summary").click();
    await expect(law).toHaveAttribute("open", "");

    const header = page.locator("header.site");
    expect((await header.boundingBox())!.height).toBeLessThanOrEqual(667);

    const last = law.locator(".menu-panel a").last();
    await last.scrollIntoViewIfNeeded();
    await expect(last).toBeInViewport();
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

test.describe("one district's own band", () => {
  test("a guaranteed district shows what the formula computes beside what it receives", async ({
    page,
  }) => {
    await page.goto(`/district/${CLEVELAND}`);
    const chart = page.locator('[data-chart="district-fan"] svg.plot:visible');
    // Its aid does not respond to its enrollment at all, so the band collapses — and the second
    // line, the formula's own falling answer, is what makes the chart say something.
    await expect(chart.locator(".fan-reference")).toHaveCount(1);
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "flat by construction",
    );
  });

  test("a formula-funded district gets a band and no second line", async ({ page }) => {
    await page.goto(`/district/${ON_FORMULA}`);
    const chart = page.locator('[data-chart="district-fan"] svg.plot:visible');
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

  test("the poverty chart draws the districts, not a summary of them", async ({ page }) => {
    /*
     * This was five bars of quintile medians, which is 606 districts reduced to five numbers on a
     * card whose own argument is about the spread around the trend. The medians are still drawn —
     * as the line through the cloud rather than in place of it — so both assertions here are about
     * the same five numbers the bar chart carried, plus the districts it did not.
     */
    await page.goto("/outcomes");
    const chart = page.locator('[data-chart="poverty-and-performance"] svg.plot:visible');
    await expect(chart).toBeVisible();

    // One mark per district with both measures, and a hit target on each so it can be pointed at.
    const dots = chart.locator(".scatter-dot circle");
    expect(await dots.count()).toBeGreaterThan(500);
    expect(await chart.locator(".scatter-hit circle[data-hover]").count()).toBe(await dots.count());

    // The median line falls left to right: least poor fifth highest, poorest fifth lowest.
    const trace = await chart
      .locator(".scatter-trace path")
      .first()
      .evaluate((n) => n.getAttribute("d") ?? "");
    const ys = [...trace.matchAll(/[ ,](\d+(?:\.\d+)?)(?=[A-Za-z]|$|,|\s)/g)]
      .map((m) => Number(m[1]))
      .filter((v) => Number.isFinite(v));
    expect(ys.length, "the trace has points to read").toBeGreaterThan(4);

    await expect(page.locator(".card", { hasText: "Poverty is most of what" })).toContainText(
      "−0.846",
    );
  });

  test("the two denominators are drawn against one vertical scale", async ({ page }) => {
    // The card's whole claim is that one cloud is flat and the other slopes. That comparison is
    // only readable if the axis they are compared on is the same one, so it is asserted rather
    // than left to whichever range each chart happened to compute.
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    await expect(card.locator('[data-chart="weighted-spending"] svg.plot:visible')).toBeVisible();
    await expect(card.locator('[data-chart="enrolled-spending"] svg.plot:visible')).toBeVisible();
    await expect(card).toContainText("−0.004");
    await expect(card).toContainText("−0.355");
  });

  test("and against one horizontal scale, because the card's prose is about horizontal distance", async ({
    page,
  }) => {
    /*
     * The sentence under the second chart reads "now they separate on the horizontal axis too".
     * Fitted to their own ranges the two x axes differed by 1.64×, so part of the separation a
     * reader saw was the frame rather than the dollars. Both charts state their own ends, so the
     * assertion is that the two pairs of ends are the same pair.
     */
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    const ends = async (chart: string) =>
      (await card.locator(`[data-chart="${chart}"] svg.plot:visible text`).allTextContents()).filter((t) =>
        /^\$[\d,]+$/.test(t),
      );
    const weighted = await ends("weighted-spending");
    // Both ends of the x axis — the third bottom label is the axis's name. Asserted so that the
    // comparison below is not between two empty lists, which would pass on a chart that had
    // stopped drawing its scale at all.
    expect(weighted).toHaveLength(2);
    // The wider denominator's own maximum, on the chart of the narrower one.
    expect(weighted[1]).toBe("$38,140");
    expect(weighted).toEqual(await ends("enrolled-spending"));
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
    /*
     * The tags must not survive as literal text anywhere in the prose — and "anywhere" now means
     * several cards, not one. A node renders its description, its findings and each revision in
     * its own `.prose-body`, and a badge that failed only inside `findings:` would have gone
     * unnoticed while this read the first card and passed.
     */
    const cards = await page.locator(".prose-body").allInnerTexts();
    expect(cards.length).toBeGreaterThan(1);
    for (const prose of cards) {
      expect(prose).not.toContain("[verified]");
      expect(prose).not.toContain("[inference]");
      expect(prose).not.toContain("[open]");
      expect(prose).not.toContain("[unentered]");
    }
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
    const headings = await page.locator(".card h2").evaluateAll((nodes) =>
      nodes.map((node) => {
        const clone = node.cloneNode(true) as HTMLElement;
        clone.querySelector("a.section-anchor")?.remove();
    // The chip carries its reckoning in a panel beside it now, so a heading's `textContent`
    // includes that sentence unless the whole wrapper goes. See `yearChip` in `src/lib/year.ts`.
    clone.querySelector(".year-chip-wrap")?.remove();
        return (clone.textContent ?? "").replace(/\s+/g, " ").trim();
      }),
    );
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
    //
    // The leader is a tripwire rather than a fact about the corpus: it changes when a record gains
    // or loses a withdrawal, and it should be looked at when it does. It was
    // `the-directory-cannot-say-why` until `drafts-are-not-legislation` overtook it with three —
    // one resolving an [open] item, one narrowing another, and one withdrawing a claim that two
    // levers partition the state, which the record's own figures disproved.
    await expect(rows.first()).toContainText("claims");
    await expect(rows.first().locator("code")).toHaveText("drafts-are-not-legislation");
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
    await page.goto("/statewide");
    const fill = await page
      .locator(".bar-fill")
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    // #3987e5 — the dark-mode step, not the light one.
    expect(fill).toBe("rgb(57, 135, 229)");
  });

  test("the theme toggle beats the OS setting in both directions", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/statewide");
    await page.locator("#theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    const fill = await page
      .locator(".bar-fill")
      .first()
      .evaluate((element) => getComputedStyle(element).fill);
    expect(fill).toBe("rgb(42, 120, 214)");
  });

  /**
   * Paper has no theme, and this is the test that the cascade agrees.
   *
   * `data-theme="dark"` is an explicit stamp that no medium unsets, and `print` matches at the
   * same time as `prefers-color-scheme: dark` when a dark-mode reader prints — so both dark
   * palettes are live when the print block has to win. Whether it does is a question about
   * specificity and source order in a real engine, which is the one thing the stylesheet unit
   * tests cannot answer. Before the print block existed this printed #ffffff ink on white paper.
   */
  test("a dark-theme reader prints in the light palette", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto("/statewide");
    // Twice: the first click leaves the dark OS setting for light, the second stamps dark back on
    // explicitly. That is the case the print block has to beat — a `[data-theme="dark"]` attribute
    // AND the dark media query, both live at once.
    await page.locator("#theme").click();
    await page.locator("#theme").click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

    await page.emulateMedia({ media: "print" });
    const paper = await page.evaluate(() => {
      const style = getComputedStyle(document.body);
      return { ink: style.color, ground: style.backgroundColor };
    });
    // #0b0b0b on #f4f4f2 — the light block, not dark's #ffffff on #111110.
    expect(paper.ink).toBe("rgb(11, 11, 11)");
    expect(paper.ground).toBe("rgb(244, 244, 242)");
  });

  /**
   * Browsers omit backgrounds from print, so a mark drawn with one prints blank. The stacked bar,
   * the legend swatches and the regime spans were all background-only; each carries a border on
   * paper now, and the border's STYLE is what tells the two series apart once the colour is gone.
   */
  test("marks drawn with a ground carry a second channel on paper", async ({ page }) => {
    // The stacked bar and its legend live on a district's aid card.
    await page.goto(`/district/${CLEVELAND}`);
    await page.emulateMedia({ media: "print" });
    const styles = await page.evaluate(() =>
      [".seg.formula", ".seg.guarantee", ".sw.formula", ".sw.guarantee"].map((selector) => {
        const element = document.querySelector(selector);
        if (!element) return `${selector} is not on the page`;
        const { borderStyle, borderWidth } = getComputedStyle(element);
        return `${selector} ${borderStyle} ${borderWidth}`;
      }),
    );
    expect(styles).toEqual([
      ".seg.formula solid 1px",
      ".seg.guarantee dashed 1px",
      ".sw.formula solid 1px",
      ".sw.guarantee dashed 1px",
    ]);
  });

  test("the page does not scroll sideways on a phone", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    for (const path of [
      "/",
      "/statewide",
      `/district/${CLEVELAND}`,
      "/districts",
      "/wiki/metric/performance-index",
    ]) {
      await page.goto(path);
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow, `${path} scrolls sideways`).toBeLessThanOrEqual(1);
    }
  });

  test("the hover layer follows the marks", async ({ page }) => {
    await page.goto("/statewide");
    const tip = page.locator("#tip");
    await expect(tip).toBeHidden();
    await page.locator("svg.plot:visible .bar-fill > *").first().hover();
    await expect(tip).toBeVisible();
    await expect(tip).toContainText("districts on the guarantee");
  });

  test("the diverging histogram marks where zero is", async ({ page }) => {
    // A reader has to be able to see where zero is, not infer it from the hues.
    // Base cost up 10% with the guarantee removed: some districts gain and some lose, so the
    // distribution straddles zero and the neutral midpoint has to be findable.
    await page.goto("/scenario?g=removed&arg=0.5&base=1.1&min=0.1&pb=1&pc=1&h=2026");
    const chart = page.locator('[data-chart="deltas"] svg.plot:visible');
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
    const labels = page.locator('[data-chart="base-cost"] svg.plot:visible .bar-label text');
    await expect(labels).toHaveCount(5);
    const chart = page.locator('[data-chart="base-cost"] svg.plot:visible');
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
    await page.goto("/statewide");
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
    await page.goto("/statewide");
    const card = page.locator(".card", { hasText: "Whether Ohio is unusual" });
    await expect(card).toContainText("39 states whose districts levy their own tax");
    await expect(card).toContainText("Massachusetts and Virginia report");
    await expect(card).toContainText("excludes twelve states on purpose");
  });

  test("the relief year is named as cutting against the finding", async ({ page }) => {
    await page.goto("/statewide");
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
    // Section headings now, not shouted capitals. The claim is what is being pinned; the
    // capitals were how it competed for attention inside a wall of body copy.
    await expect(body).toContainText("The capacity tier has a size cliff");
    await expect(body).toContainText("qualifies districts and pays them nothing");
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
    // Both extremes must be reachable, since the point of the page is to send a reader onward. The
    // two name links, plus — deliberately, from this phase — one into the tax base section of the
    // poorer district's property tax page, which is the figure this card is actually arguing about
    // and which it never linked.
    await expect(spread.locator('a[href^="/district/"]')).toHaveCount(3);
    await expect(spread.locator('a[href$="/taxes#tax-base"]')).toHaveCount(1);
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

  test("the seat's valuation strip can be read off, in the scale row and in the table", async ({
    page,
  }) => {
    /*
     * 129 seat pages drew a valuation strip and gave a reader no way to recover a value from it:
     * `distributionSpec` draws no axis by construction, and this was the one page carrying it with
     * neither the `.scale` row every other strip on the site has nor a valuation column in the
     * table underneath. A dot four fifths along meant nothing at all.
     *
     * Both, and not either: the scale row states the ends, and the column is what lets a reader go
     * from a dot to the district it belongs to.
     */
    await page.goto("/house/065");
    const card = page.locator('[data-part="members"]');
    await expect(card.locator('[data-chart="seat-spread"] svg.plot:visible')).toBeVisible();
    const scale = card.locator(".scale").first();
    await expect(scale.locator("span")).toHaveCount(2);
    await expect(scale.locator("span").first()).toContainText("$");
    await expect(card.locator("thead")).toContainText("Valuation per pupil");
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

test.describe("the district index shows the distribution it is filtering", () => {
  test("one strip is shown and all six are in the document", async ({ page }) => {
    /*
     * Six rendered at build and one revealed by an attribute selector, which is `BasisToggle`'s
     * trick applied to charts. The alternative was drawing one in the browser when the sort
     * changes, which would put Observable Plot on the district index — roughly 100 KB gzipped, on
     * a route likely to be someone's first — for the sake of a 46px strip. All six cost 4.6 KB.
     */
    await page.goto("/districts");
    const measures = page.locator("#district-measures");
    await expect(measures.locator(".measure")).toHaveCount(6);
    await expect(measures.locator(".measure:visible")).toHaveCount(1);
    await expect(measures).toHaveAttribute("data-measure", "aid");
  });

  test("the strip follows the column being sorted", async ({ page }) => {
    // The sort keys and the strip keys are the same strings, so the script that reorders the table
    // and the markup that reveals a strip cannot name different columns.
    await page.goto("/districts");
    for (const key of ["valuation", "poverty", "adm", "enrollment", "guarantee"]) {
      await page.locator(`thead button[data-sort="${key}"]`).click();
      await expect(page.locator("#district-measures")).toHaveAttribute("data-measure", key);
      await expect(page.locator(`.measure[data-measure="${key}"]`)).toBeVisible();
      await expect(page.locator("#district-measures .measure:visible")).toHaveCount(1);
    }
  });

  test("the one signed strip draws its zero, and the other five are unchanged", async ({ page }) => {
    /*
     * Enrollment change is the site's one signed distribution. It drew no zero reference, so a dot
     * two thirds along could have been a district that grew or one that shrank and the strip
     * carried nothing to say which — while `histogramSpec` draws a dashed rule and labels it
     * "no change" for exactly that reason.
     *
     * The other five measure quantities that cannot be negative, and drawing a zero on them would
     * be a reference to a value outside their range. `distributionSpec` decides from the domain,
     * so what is asserted here is that the decision came out one way six times.
     */
    await page.goto("/districts");
    const measures = page.locator("#district-measures");
    await expect(measures.locator('.measure[data-measure="enrollment"]')).toContainText(
      "no change",
    );
    for (const key of ["aid", "valuation", "poverty", "adm", "guarantee"]) {
      await expect(measures.locator(`.measure[data-measure="${key}"]`)).not.toContainText(
        "no change",
      );
    }
  });

  test("sorting by name leaves the strip alone rather than blanking it", async ({ page }) => {
    // `name` is the default sort and is not a quantity. Hiding all six would be the obvious bug.
    await page.goto("/districts");
    await page.locator('thead button[data-sort="poverty"]').click();
    await page.locator('thead button[data-sort="name"]').click();
    await expect(page.locator("#district-measures")).toHaveAttribute("data-measure", "poverty");
    await expect(page.locator("#district-measures .measure:visible")).toHaveCount(1);
  });
});

test.describe("counties, which are a peer group and not a boundary", () => {
  test("the index draws what each county spans, not only the ratio", async ({ page }) => {
    /*
     * The table ranks 88 counties by richest ÷ poorest and prints the ratio. A ratio is one number
     * standing for two and the two are not recoverable from it — two counties at the same ratio
     * can have non-overlapping wealth.
     *
     * Deliberately not a map. The card above this one says a county here is a peer group and not a
     * boundary, because district lines cross county lines freely; a choropleth would assert the
     * geography the page spends its first card denying, and the department's one-county-per-
     * district attribution could not honestly draw the crossing anyway.
     */
    await page.goto("/counties");
    const chart = page.locator('#disparity [data-chart="county-disparity"] svg.plot:visible');
    await expect(chart).toBeVisible();

    // One row per county with two districts to compare; four of the 88 have only one.
    const spans = chart.locator(".range-span line, .range-span path");
    expect(await spans.count()).toBeGreaterThan(70);
    expect(await chart.locator(".range-low circle").count()).toBe(await spans.count());
    expect(await chart.locator(".range-high circle").count()).toBe(await spans.count());

    /*
     * The ends are two shades of one hue: a low end and a high end are one measure at two points,
     * not two series.
     *
     * Read off the mark *group* rather than the circles. Plot puts a constant channel on the `<g>`
     * and a computed one on each mark, so the banded scatters — whose fill is a function of the
     * point — carry it per circle and this one does not.
     */
    const fills = await chart
      .locator(".range-low, .range-high")
      .evaluateAll((n) => n.map((g) => g.getAttribute("fill")));
    expect(fills.sort()).toEqual(["var(--ordinal-1)", "var(--ordinal-3)"]);

    await expect(page.locator("#disparity .legend .sw.ordinal-1")).toHaveCount(1);
    await expect(page.locator("#disparity .legend .sw.ordinal-3")).toHaveCount(1);
  });

  test("the rows are ordered the way the table beside them is", async ({ page }) => {
    // The chart and the table are the same data at two resolutions, and a reader moving between
    // them should not have to re-find their county.
    await page.goto("/counties");
    // `textContent`, not `allInnerTexts`: `innerText` is empty for an SVG `<text>`, so the
    // convenient helper silently compares eighty-four empty strings against the table.
    const rows = await page
      .locator('#disparity [data-chart="county-disparity"] svg.plot:visible .range-label text')
      .evaluateAll((nodes) => nodes.map((n) => n.textContent ?? ""));
    const table = await page.locator("#roster tbody tr th a").allInnerTexts();
    expect(rows.length).toBeGreaterThan(70);
    // Every charted county is in the table, in the same relative order.
    const positions = rows.map((r) => table.indexOf(r));
    expect(positions.every((p) => p >= 0), "a charted county missing from the table").toBe(true);
    expect([...positions].sort((a, b) => a - b)).toEqual(positions);
  });

  test("a county page says whether its own disparity is a large one", async ({ page }) => {
    // The card stated the ratio and never said what it was a lot of. The median county is 2.1x
    // and Cuyahoga is 5.5x, which the page had no way of conveying.
    await page.goto("/county/cuyahoga");
    const chart = page.locator('#spread [data-chart="county-position"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart.locator(".dist-marker")).toHaveCount(1);
    expect(await chart.locator(".dist-dot circle").count()).toBeGreaterThan(70);
    // "The median" and not "The median county": the figure is `stats.median`, which interpolates,
    // so on an even count of counties it is a ratio none of them has. See `src/lib/stats.ts`.
    await expect(page.locator("#spread")).toContainText("The median is");
  });

  test("a single-district county gets neither chart, and says why", async ({ page }) => {
    // A ratio needs two districts. Four counties have one, and their absence from the comparison
    // is a real answer rather than a gap to fill.
    await page.goto("/county/vinton");
    await expect(page.locator('[data-chart="county-position"]')).toHaveCount(0);
    await expect(page.locator('[data-chart="county-spread"]')).toHaveCount(0);
    await expect(page.locator("#spread")).toContainText("no internal disparity to measure");
  });
});

test.describe("colour that carries a third variable", () => {
  test("the spending charts are banded by poverty, with the legend the ramp obliges", async ({
    page,
  }) => {
    /*
     * A median line says what the middle of a cloud does. Banding says what the cloud is made of,
     * and here that is the card's whole argument: the three poverty thirds sit at the same
     * spending per need-weighted pupil and at different attainment.
     *
     * The legend is not decoration. The ramp's end steps sit near 2.2:1 against their surface,
     * which is a contrast warning that obligates relief rather than one that can be waved off.
     */
    await page.goto("/outcomes");
    const card = page.locator('[data-part="two-denominators"]');
    await expect(card.locator(".legend .sw.ordinal-1")).toHaveCount(1);
    await expect(card.locator(".legend .sw.ordinal-2")).toHaveCount(1);
    await expect(card.locator(".legend .sw.ordinal-3")).toHaveCount(1);

    // Three distinct fills across the dots, and they are the ramp rather than the series pair.
    const fills = await card
      .locator('[data-chart="weighted-spending"] .scatter-dot circle')
      .evaluateAll((nodes) => [...new Set(nodes.map((n) => n.getAttribute("fill")))]);
    expect(fills.sort()).toEqual([
      "var(--ordinal-1)",
      "var(--ordinal-2)",
      "var(--ordinal-3)",
    ]);
  });

  test("a banded chart does not label its traces as well as its legend", async ({ page }) => {
    // Three labelled lines over the densest part of the cloud said the same thing the legend
    // already said, on top of the data it was describing.
    await page.goto("/outcomes");
    const chart = page.locator('[data-chart="weighted-spending"] svg.plot:visible');
    await expect(chart.locator(".scatter-trace")).toHaveCount(3);
    await expect(chart.locator(".scatter-trace-end")).toHaveCount(0);
  });

  test("the millage chart splits on the variable that explains it, not a proxy", async ({
    page,
  }) => {
    /*
     * Banding that chart by valuation was tried and is floor status in disguise: the terciles'
     * exact-reproduction counts track their at-floor counts almost exactly. Floor status is two
     * states, so it takes the categorical pair rather than three steps of one hue.
     */
    await page.goto("/method");
    const card = page.locator("#reduction-factors");
    const fills = await card
      .locator(".scatter-dot circle")
      .evaluateAll((nodes) => [...new Set(nodes.map((n) => n.getAttribute("fill")))]);
    expect(fills.sort()).toEqual(["var(--series-formula)", "var(--series-guarantee)"]);
    await expect(card.locator(".legend .sw.formula")).toHaveCount(1);
    await expect(card.locator(".legend .sw.guarantee")).toHaveCount(1);
  });

  test("the card says how much the picture is hiding", async ({ page }) => {
    // 152 districts are one dot at (20.00, 20.00). A scatter draws the exceptions and hides the
    // rule whenever the rule is a single value, and the counts are what a reader should take.
    await page.goto("/method");
    await expect(page.locator("#reduction-factors")).toContainText("single dot at (20.00, 20.00)");
  });

  test("the poverty measure's ceiling is stated where the limits are", async ({ page }) => {
    // Not a cap this repository applies — the source publishes exactly 100% for them and the
    // values below approach it continuously. It is still a ceiling, and the page says so.
    await page.goto("/outcomes");
    const limits = page.locator("#limits");
    await expect(limits).toContainText("has a ceiling");
    await expect(limits).toContainText("not a value this repository caps");
  });
});

test.describe("the reduction factors, against what was charged", () => {
  test("the model is drawn against the record it is a model of", async ({ page }) => {
    /*
     * `/method` had four tables and no chart, on a page whose subject is which figures are models
     * and which are records. This one draws both: mills `crates/millage` predicts against mills a
     * county auditor charged, with the line where they agree.
     */
    await page.goto("/method");
    const chart = page.locator('#reduction-factors [data-chart="reduction-factors"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart.locator(".scatter-identity")).toHaveCount(1);

    // One mark per district that has a millage record, each pointable.
    const dots = chart.locator(".scatter-dot circle");
    expect(await dots.count()).toBeGreaterThan(500);
    expect(await chart.locator(".scatter-hit circle[data-hover]").count()).toBe(await dots.count());
  });

  test("the identity line is drawn at 45 degrees, not at whatever the frame allows", async ({
    page,
  }) => {
    /*
     * The reading this card offers is "distance from the line is the residual, in mills". That is
     * only true if the plot area is square: on the default 640×420 frame a shared domain still
     * draws y = x at about 33°, which reads as a trend the cloud is beating. Measured off the
     * rendered geometry rather than trusted from the spec.
     */
    await page.goto("/method");
    const box = await page
      .locator('#reduction-factors [data-chart="reduction-factors"] svg.plot:visible .scatter-identity path')
      .evaluate((n) => {
        const r = (n as SVGGraphicsElement).getBBox();
        return { w: r.width, h: r.height };
      });
    expect(box.w).toBeGreaterThan(100);
    expect(Math.abs(box.h / box.w - 1), "the identity line is at 45°").toBeLessThan(0.02);
  });

  test("the card counts the districts rather than asserting a number", async ({ page }) => {
    // "182 of 273" is the kind of sentence that is true when written and wrong two bundles later
    // with nothing to notice, so the counts are read off the feed. This checks they agree with it.
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const withMillage = feed.districts.filter((d: any) => d.millage != null);
    const atFloor = withMillage.filter((d: any) => d.millage.at_floor);
    const exact = atFloor.filter((d: any) => Math.abs(d.millage.residual) < 0.01).length;

    await page.goto("/method");
    const card = page.locator("#reduction-factors");
    await expect(card).toContainText(`${exact} of the ${atFloor.length} districts`);
  });
});

test.describe("where a district sits among the others", () => {
  test("the position card draws the distribution, not a bar with a pin in it", async ({ page }) => {
    /*
     * The strip this replaced had the minimum at one end, the maximum at the other, and nothing
     * between — so the 60th percentile and the 95th were drawn identically, when the first is a
     * dense middle and the second is nearly alone. Assessed valuation per pupil reaches five and a
     * half times its median, which is the case the flat bar could not show.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator("#position");
    const charts = card.locator(".chartwrap svg.plot:visible");
    await expect(charts).toHaveCount(2);

    // The box, the median inside it, and this district's own rule on top of both.
    await expect(charts.first().locator("rect")).not.toHaveCount(0);
    await expect(charts.first().locator(".dist-marker")).toHaveCount(1);

    // Ohio's districts past the fences are drawn as themselves and can be pointed at.
    const outliers = charts.first().locator(".dist-hit circle[data-hover]");
    expect(await outliers.count()).toBeGreaterThan(0);
  });

  test("a county draws every district in it, not only its two extremes", async ({ page }) => {
    // The card's premise is that a county is a peer group. It named the richest and the poorest
    // and drew neither the fifteen between them nor how they were spread.
    await page.goto("/county/cuyahoga");
    const dots = page.locator('#spread [data-chart="county-spread"] .dist-dot circle');
    expect(await dots.count()).toBeGreaterThan(20);
    await expect(page.locator('#spread [data-chart="county-spread"] rect')).not.toHaveCount(0);
  });

  test("a population too small for quartiles gets its members and no box", async ({ page }) => {
    /*
     * 39 of Ohio's 132 legislative seats and 60 of its 88 counties hold fewer districts than a box
     * can summarise. The first version drew one anyway: a seat with three districts got a box
     * spanning almost the whole width, because the quartiles of three numbers are the numbers.
     */
    await page.goto("/house/001");
    const chart = page.locator('#members [data-chart="seat-spread"] svg.plot:visible');
    await expect(chart).toBeVisible();
    expect(await chart.locator(".dist-dot circle").count()).toBeLessThan(8);
    await expect(chart.locator("rect"), "no box below the floor").toHaveCount(0);
    await expect(page.locator("#members")).not.toContainText("shaded box");
  });

  test("a district's outcome gap is shown against the spread it is a gap in", async ({ page }) => {
    // Two tiles said this district's score and the median of its poverty fifth. Whether a
    // fifteen-point gap is remarkable depends on how wide that fifth is, and the tiles cannot say.
    await page.goto(`/district/${CLEVELAND}/outcome`);
    const chart = page.locator('#comparable-poverty [data-chart="peer-group"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart.locator(".dist-marker")).toHaveCount(1);
    expect(await chart.locator(".dist-dot circle").count()).toBeGreaterThan(50);
  });
});

test.describe("what is on this page", () => {
  test("the list names every section of the page, in the order they come", async ({ page }) => {
    /*
     * The unit suite holds the extraction — it is a function from an HTML string to a list, and
     * `contents.spec.ts` covers the shapes exhaustively. What only a browser can say is that the
     * list agrees with the page it was read off: same sections, same order, nothing listed that a
     * reader scrolling past would not meet.
     */
    await page.goto("/method");
    const listed = await page
      .locator("main nav.contents a")
      .evaluateAll((nodes) => nodes.map((n) => n.getAttribute("href")));
    const present = await page.locator("main .card[id]").evaluateAll((nodes) =>
      nodes.filter((n) => n.querySelector(":scope > h2")).map((n) => `#${n.id}`),
    );
    expect(listed, "the contents list and the page disagree").toEqual(present);
  });

  test("a page that lists its sections lists them above the first one", async ({ page }) => {
    // Above the sections and below the page introducing itself. On a district route that means
    // below the sub-navigation, which is the thing most likely to end up on the wrong side.
    await page.goto(`/district/${CLEVELAND}`);
    const order = await page.evaluate(() => {
      const nodes = [...document.querySelectorAll("main h1, main .subnav, main nav.contents, main .card")];
      return nodes.map((n) => n.tagName === "H1" ? "h1" : n.className.split(" ")[0]);
    });
    expect(order[0]).toBe("h1");
    expect(order.indexOf("contents")).toBeGreaterThan(order.indexOf("subnav"));
    expect(order.indexOf("contents")).toBeLessThan(order.indexOf("card"));
  });

  test("a section rendered in both dollar bases is listed once", async ({ page }) => {
    /*
     * The duplicate that the `id` check found: both panels of a basis switch are the same card,
     * and before the address moved up to the scope holding them they were the same `id` twice. A
     * contents list built from the headings would list the section twice for the same reason.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const actuals = page.locator('main nav.contents a[href="#actuals"]');
    await expect(actuals).toHaveCount(1);
    // And the entry is named for the section rather than for whichever panel came first.
    await expect(actuals).not.toContainText("nominal");
  });

  test("a page with three sections or fewer has no list", async ({ page }) => {
    // `/counties` is one card and a note; `/house` is two. A list of them is longer than they are.
    for (const route of ["/counties", "/house", "/data"]) {
      await page.goto(route);
      await expect(page.locator("main nav.contents"), `${route} should have no list`).toHaveCount(0);
    }
  });

  test("the corpus prose is what a node's list is made of", async ({ page }) => {
    /*
     * On a corpus node the description *is* the page, and the headings its author wrote inside it
     * are the sections. The cards around it — properties, links, the corrections — are the
     * apparatus. A list that named only the cards would name everything except the argument.
     */
    await page.goto("/wiki/doctrine/equity");
    const entries = page.locator("main nav.contents a");
    expect(await entries.count()).toBeGreaterThan(8);
    await expect(entries.filter({ hasText: "It trades against adequacy" })).toHaveCount(1);
  });

  test("every entry lands on the section it names", async ({ page }) => {
    // The behaviour. Each entry is a plain fragment link, so this is also the no-script path.
    await page.goto("/method");
    const entries = page.locator("main nav.contents a");
    const count = await entries.count();
    expect(count).toBeGreaterThan(3);

    for (let i = 0; i < count; i += 1) {
      const href = await entries.nth(i).getAttribute("href");
      await entries.nth(i).click();
      await expect(page).toHaveURL(new RegExp(`${href!.replace("#", "#")}$`));
      await expect(page.locator(`main ${href}`)).toBeInViewport();
    }
  });
});

test.describe("every card has an address", () => {
  test("every card and sub-section on the dashboard is reachable by fragment", async ({ page }) => {
    /*
     * Before this there was exactly one `id` in the whole rendering layer — `prose.ts`'s correction
     * blockquote — so nine entry surfaces deposited every reader at byte zero of the same 50,000
     * byte document and there was nothing to send them anywhere else.
     *
     * The names are not invented here. Each is the `data-part` the card already carried or the
     * `data-program` its sub-table did, emitted from the same string, so the hook a test locates by
     * and the address a reader links to cannot drift apart.
     */
    await page.goto("/district/043802");
    const parts = await page
      .locator("main .card[data-part]")
      .evaluateAll((nodes) => nodes.map((n) => [n.getAttribute("data-part"), n.id]));
    expect(parts.length).toBeGreaterThan(6);
    expect(
      parts.filter(([part, id]) => part !== id),
      "a card whose id and data-part disagree",
    ).toEqual([]);

    // And every one of them says its own address, in a link a reader can see and copy. The
    // addresses existed for some time before this did, and were used by two links in the whole
    // repository — see `src/lib/section.ts`.
    const misaddressed = await page.locator("main .card[id]").evaluateAll((nodes) =>
      nodes
        .filter((n) => n.querySelector(":scope > h2"))
        .map((n) => [n.id, n.querySelector(":scope > h2 a.section-anchor")?.getAttribute("href")])
        .filter(([id, href]) => href !== `#${id}`),
    );
    expect(misaddressed, "a card whose heading anchor names another section").toEqual([]);

    // The six categoricals, transportation and preschool head themselves with an `<h3>` inside a
    // card, so they are addressable one level below the card that contains them.
    for (const id of [
      "special-education",
      "targeted-assistance",
      "dpia",
      "gifted",
      "career-technical",
      "english-learners",
      "transportation",
      "preschool",
    ]) {
      await expect(page.locator(`h3#${id}`), `#${id} is in the vocabulary`).toHaveCount(1);
    }
  });

  test("every route family heads its sections with a link to them", async ({ page }) => {
    /*
     * One page from each family. `check-dist-links.ts` asserts this over all 3,466 built pages and
     * is the exhaustive check; what this adds is the browser — the anchor has to survive into a
     * rendered document and be something a reader can actually see, which a string check of the
     * HTML cannot tell you. So this asserts it is on screen and not merely present.
     *
     * The count is `> 0` rather than a number per route on purpose. Several of these sections are
     * conditional on the feed, and a test that pinned the total would fail on a bundle regenerated
     * with a district that has no filing rather than on anything being wrong.
     */
    for (const route of [
      "/",
      "/history",
      "/outcomes",
      "/method",
      "/data",
      "/counties",
      "/county/franklin",
      "/house",
      "/wiki",
      "/wiki/doctrine",
      "/wiki/doctrine/equity",
      "/wiki/source/bls-cpi-u",
      "/wiki/decision/ontology",
      `/district/${CLEVELAND}`,
      `/district/${CLEVELAND}/finances`,
      `/district/${CLEVELAND}/taxes`,
    ]) {
      await page.goto(route);
      const anchors = page.locator("main a.section-anchor");
      const count = await anchors.count();
      expect(count, `${route} heads no section with an anchor`).toBeGreaterThan(0);
      await expect(anchors.first(), `${route}: the anchor is not visible`).toBeVisible();

      // Every one of them names something the page carries. A fragment that resolves to nothing
      // does not 404 — it serves the right document and leaves the reader at the top of it.
      const dangling = await anchors.evaluateAll((nodes) =>
        nodes
          .map((n) => n.getAttribute("href") ?? "")
          .filter((href) => !document.querySelector(`main ${href.replace("#", "#")}`)),
      );
      expect(dangling, `${route}: anchors naming an id the page does not carry`).toEqual([]);
    }
  });

  test("clicking a section anchor addresses that section", async ({ page }) => {
    /*
     * The behaviour, rather than the markup. A reader clicks the `#` beside a heading to get a URL
     * they can send someone, so the click has to put the fragment in the address bar and the
     * section under it — and it has to do that without landing behind the sticky header, which is
     * the failure `--sticky-chrome` exists for and which no amount of correct markup prevents.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await page.locator('.card#categoricals > h2 a.section-anchor').click();
    await expect(page).toHaveURL(/#categoricals$/);

    const top = await page.locator("#categoricals").evaluate((n) => n.getBoundingClientRect().top);
    const chrome = await page
      .locator("header.site")
      .evaluate((n) => n.getBoundingClientRect().bottom);
    expect(top, "the section landed under the sticky header rather than below it").toBeGreaterThan(
      chrome - 1,
    );
  });

  test("the corpus prose heads its own sections too", async ({ page }) => {
    /*
     * These are the headings a corpus author wrote in a `findings` or `description` field. The
     * markdown processor has always given them an id derived from the text, which made them the
     * oldest addresses on the site and the least reachable: nothing rendered them as a link, so
     * the only way to learn one existed was to read the page source.
     *
     * `prose.ts` adds the anchor after the processor has run and after `rehype-sanitize`, which
     * strips `class` and `aria-label` — so this also pins that ordering. Emitted inside the
     * pipeline, the anchor arrives as a bare unstyled `<a>` and this assertion goes red.
     */
    await page.goto("/wiki/doctrine/equity");
    const headings = page.locator(".prose-body h2[id], .prose-body h3[id]");
    expect(await headings.count(), "the equity node writes headings in its prose").toBeGreaterThan(2);

    const broken = await headings.evaluateAll((nodes) =>
      nodes
        .map((n) => [n.id, n.querySelector("a.section-anchor")?.getAttribute("href")])
        .filter(([id, href]) => href !== `#${id}`),
    );
    expect(broken, "a prose heading with no anchor, or one naming another section").toEqual([]);
  });

  test("a fragment link lands its section clear of the sticky chrome, at every breakpoint", async ({
    page,
  }) => {
    /*
     * `--sticky-chrome` is a pixel constant measured from a build, which is brittle by nature: the
     * header wraps to three rows below 480px, two to 700, and is one row from 1000px up, and a
     * change to its contents moves all four numbers with nothing to notice. This is what makes that
     * a red test rather than a reader who clicked `#categoricals` and is looking at the base cost
     * card because their target scrolled under the header.
     *
     * 1000px is also where the sub-navigation becomes sticky and the two bars stack, so it is the
     * width where getting the offset wrong costs the most.
     */
    for (const width of [390, 700, 1000, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      await page.goto("/district/043802#categoricals");
      const chrome = await page
        .locator("header.site")
        .evaluate((n) => n.getBoundingClientRect().bottom);
      const target = await page
        .locator("#categoricals")
        .evaluate((n) => n.getBoundingClientRect().top);
      expect(
        target,
        `at ${width}px the section landed under the sticky header rather than below it`,
      ).toBeGreaterThanOrEqual(chrome);
    }
  });

  test("the sub-navigation is sticky where that is affordable and nowhere else", async ({ page }) => {
    /*
     * The site header's own rule justifies itself by reference to this bar — "a long finances table
     * should not strand them" — which is a claim the stylesheet did not implement, because
     * `.subnav` renders inside `<main>` and scrolls away with the document.
     *
     * It is sticky from 1000px and deliberately not below. Measured on a build: the header is 51px
     * at 1000px and up, 96px at 700, 135px at 480 and 168px at 390. A second sticky bar under it
     * would take 132px of an 800px viewport at 700px and 246px — 31% — at 390px, on a page whose
     * complaint was density. The tab row is one screen from the top there and the trade is not
     * worth it.
     */
    for (const [width, sticky] of [
      [390, false],
      [700, false],
      [1000, true],
      [1280, true],
    ] as [number, boolean][]) {
      await page.setViewportSize({ width, height: 900 });
      await page.goto("/district/043802");
      const position = await page
        .locator(".subnav")
        .evaluate((n) => getComputedStyle(n).position);
      expect(position === "sticky", `at ${width}px .subnav sticky should be ${sticky}`).toBe(sticky);
    }
  });

  test("the pupil-count note points at the reconciliation only where it exists", async ({ page }) => {
    /*
     * The dangling-fragment case, which is the one the plan's own critique caught. `#denominators`
     * is absent for 177 of 609 districts — `renderDenominators` returns "" where the two valuations
     * agree within 5% — and a missing fragment does not 404: it serves the right document and
     * leaves the reader at the top of it. Both branches are asserted, because only checking the
     * one that renders would pass on the version of this that shipped the bug.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const ratio = (d: any) =>
      Math.abs(
        d.property_tax[d.property_tax.length - 1].value_per_pupil / d.valuation_per_pupil - 1,
      );
    const wide = feed.districts.find((d: any) => d.valuation_per_pupil && ratio(d) >= 0.05);
    const close = feed.districts.find((d: any) => d.valuation_per_pupil && ratio(d) < 0.05);
    expect(wide && close, "one district on each side of the 5% test").toBeTruthy();

    await page.goto(`/district/${wide.irn}`);
    await expect(
      page.locator(`[data-part="not"] a[href="/district/${wide.irn}/taxes#denominators"]`),
    ).toHaveCount(1);

    await page.goto(`/district/${close.irn}`);
    await expect(
      page.locator(`[data-part="not"] a[href="/district/${close.irn}/taxes"]`),
    ).toHaveCount(1);
    await expect(page.locator('[data-part="not"] a[href*="#denominators"]')).toHaveCount(0);
  });

  test("the county card sends a reader to the tax base it is arguing about", async ({ page }) => {
    // The card's whole subject is the valuation ratio between two districts in one county, and the
    // page that decomposes that valuation is four cards down a route it never linked.
    await page.goto("/county/cuyahoga");
    const link = page.locator('[data-part="spread"] a[href*="/taxes#tax-base"]');
    await expect(link).toHaveCount(1);
    await expect(link).toContainText("the tax base each pupil stands on");
  });
});

test.describe("the answer first", () => {
  test("the page opens with state aid, not with an input to the formula", async ({ page }) => {
    /*
     * The first figure was base cost per pupil, which is what the plan says a district's education
     * costs — not money it receives, and not what the page says it is about. The `<meta
     * description>`, the share image's alt text and this card's own heading all say the question is
     * state aid and where it comes from, and the page answered a different one for eight phases.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const tiles = page.locator('[data-part="headline"] .tile');
    await expect(tiles).toHaveCount(3);
    await expect(tiles.nth(0)).toContainText("State aid, FY2027");
    await expect(tiles.nth(1)).toContainText("State aid / pupil");
    await expect(tiles.nth(2)).toContainText("Base-cost ADM");
    // And the tiles are inside the card whose heading asks the question, not floating above it.
    await expect(page.locator('[data-part="aid-source"] [data-part="headline"]')).toHaveCount(1);
  });

  test("the aid total is summed from its components, not multiplied by a pupil count", async ({
    page,
  }) => {
    /*
     * `realized_aid_per_pupil` divides by base cost ADM, and the pupil count printed one tile to
     * its right is the current-year one. Multiplying the per-pupil figure by the count beside it
     * is wrong by a median $112,601 and by $8.2m for Cleveland — a figure large enough to be a
     * finding, arrived at by an arithmetic a reader is being invited to perform.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const d = feed.districts.find((x: { irn: string }) => x.irn === CLEVELAND);
    const summed = d.base_cost_state_share + d.categorical_funding + d.guarantee;
    const multiplied = d.realized_aid_per_pupil * d.current_year_adm;
    expect(Math.abs(summed - multiplied), "the two constructions still differ").toBeGreaterThan(
      1_000_000,
    );

    await page.goto(`/district/${CLEVELAND}`);
    const printed = await page
      .locator('[data-part="headline"] .tile')
      .first()
      .locator(".v")
      .innerText();
    const dollars = Number(printed.replace(/[^0-9.]/g, ""));
    // Printed to the nearest dollar, so the tolerance is rounding rather than construction.
    expect(Math.abs(dollars - summed)).toBeLessThan(1);
  });

  test("what a year of enrollment is worth comes before what the plan says it costs", async ({
    page,
  }) => {
    /*
     * It was in position six of nine. For the 294 guaranteed districts this card answers the third
     * clause of the page's own question directly — a year of enrollment is worth nothing to them,
     * because the guarantee holds a fixed dollar amount enrollment does not enter — and a reader
     * had to pass base cost, the categoricals and the supplements to reach it.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const order = await page
      .locator("main .card[data-part]")
      .evaluateAll((nodes) => nodes.map((n) => n.getAttribute("data-part")));
    expect(order.slice(0, 4)).toEqual(["aid-source", "enrollment", "base-cost", "categoricals"]);
    expect(order[order.length - 1]).toBe("not");
  });

  test("the two views of one district finally point at each other", async ({ page }) => {
    // One district's counterfactual lever lived on one route and one district's projection on
    // another, with no cross-reference in either direction. They hold opposite things fixed, which
    // is the whole reason a reader on one wants the other.
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "holds enrollment at published FY2026 and moves the formula",
    );
    await expect(
      page.locator(`[data-part="enrollment"] a[href="/district/${CLEVELAND}/scenario"]`),
    ).toHaveCount(1);

    await page.goto(`/district/${CLEVELAND}/scenario`);
    await expect(page.locator('[data-part="not"]')).toContainText("holds enrollment fixed");
    await expect(
      page.locator(`[data-part="not"] a[href="/district/${CLEVELAND}"]`),
    ).toHaveCount(1);
  });

  test("the bar is drawn only where there are two proportions to draw", async ({ page }) => {
    /*
     * 315 districts receive exactly what the formula computes, so the split bar was a full-width
     * single segment under a legend that had collapsed to one item — a proportion chart with one
     * proportion in it. Nothing is lost by omitting it: where the guarantee pays zero, aid per
     * pupil *is* formula aid per pupil, for all 315.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator('[data-part="aid-source"] .bar .seg')).toHaveCount(2);

    await page.goto(`/district/${ON_FORMULA}`);
    await expect(page.locator('[data-part="aid-source"] .bar')).toHaveCount(0);
    // The figure the legend used to carry is still on the page, in the tile above where it was.
    await expect(page.locator('[data-part="headline"] .tile').nth(1)).toContainText(
      "all from the formula",
    );
  });

  test("each surviving condition says what follows from it, and links what it names", async ({
    page,
  }) => {
    /*
     * The flag row printed five pills and two of them restated the sub-line one element above.
     * "At or below the 20-mill floor" was strictly *less* precise than what it duplicated — the
     * sub-line separates the 21 districts genuinely below 20 mills from the 155 sitting on it, the
     * pill collapsed all 176. What is left carries a consequence, because "At the minimum state
     * share" is a noun phrase and a reader who does not already know the plan cannot get from it
     * to what it means for their district.
     */
    // Wilmington City trips both of the two that carry a corpus link: it is within a twentieth of
    // a mill of the floor and it is held at the minimum state share.
    await page.goto("/district/045112");
    const card = page.locator('[data-part="aid-source"]');
    await expect(card).not.toContainText("At or below the 20-mill floor");
    await expect(card).not.toContainText("Funded by the guarantee, not the formula");
    const conditions = page.locator('[data-part="aid-source"] .conditions li');
    await expect(conditions).toContainText([
      "Within a twentieth of a mill of the floor",
      "At the minimum state share",
      "of voted millage reduced away",
    ]);
    await expect(card.locator('.conditions a[href="/wiki/parameter/twenty-mill-floor"]')).toHaveCount(1);
    await expect(
      card.locator('.conditions a[href="/wiki/metric/state-share-percentage"]'),
    ).toHaveCount(1);
    // Every row carries a clause, not just a label.
    for (const text of await conditions.allInnerTexts()) {
      expect(text, "a condition with no consequence clause").toContain(" — ");
    }
  });

  test("the one district that tripped no condition is the one the symmetric flag was for", async ({
    page,
  }) => {
    /*
     * This asserted `toHaveCount(0)` when the conditions landed: Marion Local tripped none of the
     * four and rendered an empty `<ul>`, which reads as a gap in the page rather than as the
     * absence of a fact. The guard for that is still in `renderAidSource` and is still correct for
     * a feed that carries a null enrollment change.
     *
     * It is unreachable with this feed, and deliberately so. Making the enrollment condition
     * symmetric — it fired only on a fall, leaving 109 districts with a Detail row as the sole
     * carrier — gave every district at least one condition, and Marion Local's is the rise that
     * kept it silent before. The assertion is inverted rather than deleted, because the empty case
     * is the thing worth remembering.
     */
    await page.goto("/district/048553");
    await expect(page.locator("h1")).toHaveText("Marion Local");
    const conditions = page.locator('[data-part="aid-source"] .conditions li');
    await expect(conditions).toHaveCount(1);
    await expect(conditions).toContainText("Enrollment up");
  });

  test("the dashboard finally says what it is not", async ({ page }) => {
    /*
     * Three of the four siblings close this way and the dashboard, with the most collisions to
     * declare, did not — while `finances.astro`'s own closing card points here and received no
     * answer back.
     */
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator('[data-part="not"]');
    await expect(card).toContainText("None of the figures above is money this district received");
    await expect(card).toContainText("divides by more than one pupil count");
    await expect(card).toContainText("two other years on a fourth count");
    // The two vintages `crates/bundle` names and the page did not: valuation is FY2023 and
    // spending is FY2024, inside one card, and the closing note is the only place that says so.
    await expect(card).toContainText("FY2023");
    await expect(card.locator(`a[href="/district/${CLEVELAND}/finances"]`)).toHaveCount(1);
    await expect(card.locator('a[href="/wiki/metric/enrolled-adm"]')).toHaveCount(1);
  });

  test("a district whose two counts round alike is told so rather than shown two equal numbers", async ({
    page,
  }) => {
    // 110 of 609. Printing the same integer twice under "more than one pupil count" would read as
    // a rendering fault rather than as the fact that this district is one of the ones they agree
    // for — so the sentence branches on it.
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const same = feed.districts.find(
      (d: { adm: number; current_year_adm: number }) =>
        Math.round(d.adm) === Math.round(d.current_year_adm),
    );
    expect(same, "a district whose two counts round alike").toBeTruthy();
    await page.goto(`/district/${same.irn}`);
    await expect(page.locator('[data-part="not"]')).toContainText(
      "round to the same figure here, which they do not for 499",
    );
  });
});

test.describe("the card that restated the page", () => {
  test("the Detail card is gone, and nothing it uniquely carried went with it", async ({ page }) => {
    /*
     * Thirteen rows, ten of them a figure already on the page — base cost per pupil printed three
     * times, aggregate base cost four, state share of base cost three, categorical funding three,
     * guarantee per pupil three, enrolled ADM three. Its docstring was false twice: "every figure
     * the feed carries" was 13 of 319 numeric leaves, and "unrounded" was 9 of 13 rows calling
     * `money()` at zero decimals. `git show 9a04427:web/src/district.ts` is byte-identical to the
     * card as it stood, from a build where the view had three `<h2>`s and every row was the page's
     * only carrier of its figure. Zero tests referenced it, which is its own indictment.
     *
     * Three things it carried alone had to move first, and each is asserted here rather than in a
     * commit message.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await expect(page.locator('[data-part="detail"]')).toHaveCount(0);
    await expect(page.locator("main")).not.toContainText("Effective Class 1 millage");

    // 1. The departmental-estimate caveat, which was the only statement of that fact on any of the
    //    five district routes, now sits on the card whose terminal year is the estimated one.
    await expect(page.locator('[data-part="enrollment"]')).toContainText(
      "partly a departmental estimate",
    );

    // 3. The reader who was using it as a reference index is pointed at where it went.
    const closing = page.locator('[data-part="not"]');
    await expect(closing).toContainText("thirteen-row table");
    await expect(closing.locator('a[href="/data/districts.csv"]')).toHaveCount(1);
  });

  test("every guaranteed district still carries its guarantee dollar total", async ({ page }) => {
    /*
     * 2. The ordering the plan calls its own worst failure mode. Under the old guard 158 of the 294
     *    guaranteed districts had no hold-harmless table, so their guarantee total was on the
     *    dashboard only in Detail — and nothing in the suite would have caught the loss, because
     *    nothing tested Detail. The union guard landed a phase earlier; this asserts it held.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const guaranteed = feed.districts.filter((d: { guarantee: number }) => d.guarantee > 0);
    expect(guaranteed.length).toBe(294);
    // Three sampled across the branch space: one with a clawback, one with the supplement, one
    // with neither — the last being the class that would have lost the figure.
    const pick = (f: (d: Record<string, never>) => boolean) => guaranteed.find(f);
    const cases = [
      pick((d: any) => d.transition.open_enrollment_adjustment > 0),
      pick((d: any) => d.transition.transition_supplement > 0),
      pick(
        (d: any) =>
          d.transition.open_enrollment_adjustment === 0 && d.transition.transition_supplement === 0,
      ),
    ];
    for (const d of cases) {
      expect(d, "each branch of the guard has a district").toBeTruthy();
      await page.goto(`/district/${(d as any).irn}`);
      const table = page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]');
      await expect(table).toContainText("Guarantee");
      await expect(table.locator("tbody")).not.toContainText("The formula reaches its FY2021 base");
    }
  });

  test("a district whose enrollment rose is told so, not left silent", async ({ page }) => {
    /*
     * The condition fired only on a negative, so for the 109 districts whose enrollment rose
     * between FY2024 and FY2026 the dashboard's sole carrier of the fact was a Detail row. That is
     * why the flag had to become symmetric before the card was deleted rather than after.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const risen = feed.districts.filter(
      (d: { enrollment_change: number | null }) =>
        d.enrollment_change != null && d.enrollment_change > 0,
    );
    expect(risen.length).toBe(109);
    await page.goto(`/district/${risen[0].irn}`);
    const conditions = page.locator('[data-part="aid-source"] .conditions');
    await expect(conditions).toContainText("Enrollment up");
    await expect(conditions).not.toContainText("Enrollment down");
  });

  test("the taxes page names the year its two publications agree in", async ({ page }) => {
    /*
     * The other half of the collision the millage row was deleted over. `/taxes` closed by saying
     * its effective Class I rate matches the profile report "to 0.01 mills, as it does for all 606
     * districts carrying both" — true of TY2023, three cards below a tile printing TY2024, which
     * the two agree on for 219 of the 606. Deleting the dashboard row and leaving this standing
     * would have removed the only figure on the site that contradicted it.
     */
    await page.goto(`/district/${CLEVELAND}/taxes`);
    const card = page.locator('[data-part="not"]');
    await expect(card).toContainText("for TY2023");
    await expect(card).toContainText("That is not the rate in the tile above");
    await expect(card).toContainText("219");
    await expect(card).toContainText("effective_class1_millage_ty23");
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
    /*
     * The correction that matters most: the node described a hold-harmless with no clawback in it,
     * which reproduces correctly for 566 districts and wrongly for 43.
     *
     * It is a `revisions:` entry now rather than a shouted paragraph in the body, so this asserts
     * on the disclosure — that the withdrawal is present, that it is filed apart from the current
     * account, and that it still carries the number that makes it legible.
     */
    await page.goto("/wiki/formula-component/temporary-transitional-aid-guarantee");
    const body = page.locator("main");
    await expect(body).toContainText("Open Enrolment Adjustment");

    const withdrawals = page.locator(".card.apparatus", { hasText: "What this node used to say" });
    await expect(withdrawals).toBeVisible();
    await expect(withdrawals.locator("details.revision")).toHaveCount(3);
    await expect(withdrawals).toContainText("566 districts and wrongly for 43");

    // Struck through, so a reader landing on an open disclosure can tell the withdrawn claim from
    // the live one without relying on colour.
    await expect(withdrawals.locator(".withdrawn").first()).toHaveCSS(
      "text-decoration-line",
      "line-through",
    );
  });

  test("the proration parameter names all three and which one publishes its limit", async ({
    page,
  }) => {
    await page.goto("/wiki/parameter/appropriation-proration-factor");
    const body = page.locator("main");
    await expect(body).toContainText("not a rate, a weight, a price or a threshold");
    await expect(body).toContainText("147,500,000");
    await expect(body).toContainText("A proration of 1.0 is not an absence");
  });

  test("a district touched by none of the three shows no hold-harmless table", async ({ page }) => {
    /*
     * The guard is a union of three conditions, and this test used to select on two of them. It
     * asked for the first district with no clawback and no supplement, which under the widened
     * guard is not the same set — and it would have kept passing anyway, because the first such
     * district in feed order happens to carry no guarantee either. A test that stays green while
     * asserting something it no longer means is worse than one that goes red, so the selector
     * names all three conditions explicitly.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const clean = feed.districts.find(
      (d: {
        guarantee: number;
        transition: { open_enrollment_adjustment: number; transition_supplement: number };
      }) =>
        d.guarantee <= 0 &&
        d.transition.open_enrollment_adjustment === 0 &&
        d.transition.transition_supplement === 0,
    );
    expect(clean, "a district touched by none of the three mechanisms").toBeTruthy();
    await page.goto(`/district/${clean.irn}`);
    await expect(
      page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]'),
    ).toHaveCount(0);
  });

  test("a guaranteed district with neither mechanism still gets its guarantee total", async ({
    page,
  }) => {
    /*
     * The 158 this phase is for. The guard was `clawback || supplement`, so a district carrying a
     * guarantee and neither mechanism had no table here at all, and its guarantee dollar total
     * appeared on the dashboard only in the Detail card — which is the card the next phase
     * deletes. This has to land before that one or those 158 lose the figure silently.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const guaranteed = feed.districts.find(
      (d: {
        guarantee: number;
        transition: { open_enrollment_adjustment: number; transition_supplement: number };
      }) =>
        d.guarantee > 0 &&
        d.transition.open_enrollment_adjustment === 0 &&
        d.transition.transition_supplement === 0,
    );
    expect(guaranteed, "a guaranteed district with neither mechanism").toBeTruthy();
    await page.goto(`/district/${guaranteed.irn}`);
    const table = page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]');
    await expect(table).toBeVisible();
    await expect(table).toContainText("FY2021 funding base");
    await expect(table).toContainText("Guarantee");
    // The two conditional rows stay conditional. This district has neither.
    await expect(table).not.toContainText("Open-enrolment clawback");
    await expect(table).not.toContainText("Formula transition supplement");
  });

  test("a district with a mechanism and no guarantee keeps its table", async ({ page }) => {
    /*
     * The 37 a literal reading of "widen the guard to `guarantee > 0`" would have deleted the
     * table from — 22 with a clawback and 17 drawing the supplement while drawing nothing from the
     * guarantee. The supplement paragraph names those 17 by count, so a replacement rather than a
     * union would have removed the table from precisely the districts that sentence is about.
     */
    const feed = await (await page.request.get("/data/bundle.json")).json();
    const touched = feed.districts.find(
      (d: {
        guarantee: number;
        transition: { open_enrollment_adjustment: number; transition_supplement: number };
      }) =>
        d.guarantee <= 0 &&
        (d.transition.open_enrollment_adjustment > 0 || d.transition.transition_supplement > 0),
    );
    expect(touched, "a district with a mechanism and no guarantee").toBeTruthy();
    await page.goto(`/district/${touched.irn}`);
    await expect(
      page.locator('[data-part="aid-source"] table[data-program="hold-harmless"]'),
    ).toBeVisible();
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
    //
    // Data rows only: the table now closes one row group and opens another where the denominator
    // changes, and the labelled row between them is structure rather than a year. Counting every
    // `tr` made this fail at 18 when the break landed, which is the assertion catching a real
    // change in the table's shape — and the shape is the point of the change.
    await page.goto("/history");
    const card = page.locator(".card", { hasText: "What the poverty weight is counted on" });
    const rows = card.locator("tbody tr:not(.series-break)");
    await expect(rows).toHaveCount(17);
    await expect(rows.first()).toContainText("AdmCount");
    await expect(rows.last()).toContainText("CECount");
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

test.describe("a draft opened in the runner", () => {
  test("the unpriced provisions are on screen with the total, not below it", async ({ page }) => {
    /*
     * The rule this whole class rests on, at the layer where it is easiest to lose. `crates/project`
     * cannot print a bill's cost without the clauses it failed to price; the feed carries them; and
     * this is the check that the page actually shows them beside the number rather than somewhere a
     * reader scrolls past.
     */
    await page.goto("/scenario?draft=fund-the-plan-and-retire-the-guarantee");
    const card = page.locator('.card[data-part="draft"]');
    await expect(card).toContainText("Opened from a draft");
    await expect(card).toContainText("not in any figure on this page");
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(3);

    // First, not merely present: the placement the rule requires is a limit on what is about to be
    // read rather than a footnote on what was read.
    await expect(page.locator("#scenario-out > :first-child")).toHaveAttribute(
      "data-part",
      "draft",
    );
  });

  test("the figure is the draft's even where a slider cannot express it", async ({ page }) => {
    /*
     * `#lv-base` steps by 0.01 and the refresh provision is 1.0395, so the control can only get to
     * 1.04 — which is a −$139.9M scenario reaching 357 districts, against the draft's −$143.9M
     * reaching 356. The controls are set as near as they go and the first render is computed from
     * the draft, so the number under the banner is the bill's rather than the slider's.
     */
    await page.goto("/scenario?draft=fund-the-plan-and-retire-the-guarantee");
    await expect(page.locator("#lv-guarantee")).toHaveValue("phase-out");
    await expect(page.locator("#lv-arg")).toHaveValue("0.5");
    await expect(page.locator("#lv-base")).toHaveValue("1.04");

    // The combined figure, and not the one the rounded slider would give.
    await expect(page.locator("#scenario-out")).toContainText("\u2212$143.9M");
    await expect(page.locator("#scenario-out")).toContainText("356 up, 253 down");
    await expect(page.locator('[data-part="draft-departed"]')).toHaveCount(0);
  });

  test("moving a lever says the figures are no longer the bill's", async ({ page }) => {
    // The likely failure: a reader nudges a slider and the page goes on attributing the number to
    // the draft. Removing the banner instead would be worse — a figure they still believe is the
    // bill's, with nothing to correct them.
    await page.goto("/scenario?draft=fund-the-plan-and-retire-the-guarantee");
    await expect(page.locator('[data-part="draft-departed"]')).toHaveCount(0);

    await page.locator("#lv-base").fill("1.08");
    await page.locator("#lv-base").dispatchEvent("input");

    await expect(page.locator('[data-part="draft-departed"]')).toContainText(
      "no longer match the draft",
    );
    // And the missing clauses stay put: a departed scenario is short the same three provisions.
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(3);
  });

  test("a draft that prices completely says so rather than staying silent", async ({ page }) => {
    await page.goto("/scenario?draft=hb-96-with-refreshed-inputs");
    await expect(page.locator('[data-part="draft-complete"]')).toContainText(
      "a property of a one-clause draft",
    );
    await expect(page.locator('[data-part="draft-unpriced"]')).toHaveCount(0);
  });

  test("the draft's node page states the priced ratio beside the link, not just the link", async ({
    page,
  }) => {
    // An invitation to see a bill's cost carries, at the point of the invitation, the fact that the
    // cost is of some of its clauses. Third place the same refusal is made, and the first a reader
    // meets.
    await page.goto("/wiki/draft-legislation/fund-the-plan-and-retire-the-guarantee");
    const card = page.locator('.card[data-part="runner"]');
    await expect(card).toContainText("2 of this draft's 5 provisions");
    await expect(card).toContainText("not of the bill");
    await expect(card.locator("a.flag")).toHaveAttribute(
      "href",
      "/scenario?draft=fund-the-plan-and-retire-the-guarantee",
    );
  });

  test("the district route carries the draft card too, and it is first", async ({ page }) => {
    /*
     * The failure a review caught: `?draft=` was read on both routes but the card was written only
     * in the statewide branch, so a district page applied a bill's levers and reported a dollar
     * and per-pupil figure with nothing naming the three provisions it could not price. A district
     * page is the one a school board sends, which makes it the worst place for that and not the
     * mildest.
     */
    await page.goto(
      `/district/${NORTHERN}/scenario?draft=fund-the-plan-and-retire-the-guarantee`,
    );
    await expect(page.locator("#scenario-out > :first-child")).toHaveAttribute(
      "data-part",
      "draft",
    );
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(3);
    // And the district figures are still there below it.
    await expect(page.locator("#scenario-out")).toContainText("What moved for this district");
  });

  test("an unknown draft leaves the runner working rather than showing an empty card", async ({
    page,
  }) => {
    await page.goto("/scenario?draft=no-such-bill");
    await expect(page.locator('.card[data-part="draft"]')).toHaveCount(0);
    await expect(page.locator("#scenario-out")).toContainText("Current law");
  });
});

test.describe("a pending bill nothing here can price", () => {
  test("the page says it is not the bill rather than showing current law", async ({ page }) => {
    /*
     * H.B. 643 of the 136th caps EdChoice expansion eligibility, and every provision it has falls
     * in the scholarship channel this repository does not model. It sets no lever, so the runner
     * shows current law — true, and indistinguishable from a bill that costs nothing unless the
     * page says which it is.
     */
    await page.goto("/scenario?draft=hb-643-136-introduced");
    const card = page.locator('.card[data-part="draft"]');
    await expect(card).toContainText("Nothing on this page is this bill");
    await expect(card).toContainText("There is no cost of zero to report");
    await expect(page.locator('[data-part="draft-unpriced"] li')).toHaveCount(1);
  });

  test("its node page offers no invitation to a figure that does not exist", async ({ page }) => {
    await page.goto("/wiki/draft-legislation/hb-643-136-introduced");
    const card = page.locator('.card[data-part="runner"]');
    await expect(card).toContainText("0 of this draft's 1 provisions");
    await expect(card).toContainText("not of the bill");
  });
});

test.describe("the money no other figure on the site counts", () => {
  test("the card is on the finances route, and nothing above it carries the money", async ({
    page,
  }) => {
    /*
     * The failure this stands against is the one the feed made available for four phases: a
     * per-district series computed in Rust, tested in Rust, and reachable by nobody who was not
     * running cargo. It is also the failure the page made available for longer — a reader who
     * added up a district's pages was told by omission that they had seen all the state money.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator('.card[data-part="casino"]');
    await expect(card).toBeVisible();
    await expect(card).toContainText("nothing above this card counts it");
    await expect(card).toContainText("gross casino revenue county student fund");
  });

  test("the chip says the series, not the page's year", async ({ page }) => {
    // Nine fiscal years on a route whose other card spans six, and the chip has to describe the
    // card it sits on rather than the route it sits in.
    await page.goto(`/district/${CLEVELAND}/finances`);
    const chip = page.locator('.card[data-part="casino"] .year-chip');
    await expect(chip).toHaveText("FY2016-FY2024");
    await expect(chip).toHaveAttribute("data-kind", "fiscal");
  });

  test("the district figure and the statewide figure are both there, and they differ", async ({
    page,
  }) => {
    // A district's own money is the reason to look; the statewide column is what stops the reader
    // concluding that this district's fall in FY2021 was about this district.
    await page.goto(`/district/${CLEVELAND}/finances`);
    const rows = page.locator('.card[data-part="casino"] table[data-program="casino"] tbody tr');
    await expect(rows).toHaveCount(9);
    await expect(rows.first()).toContainText("FY2016");
    await expect(rows.last()).toContainText("FY2024");
    await expect(rows.last()).toContainText("$114,177,214");
  });

  test("the closure is named as a closure and put in the year the money moved", async ({
    page,
  }) => {
    /*
     * The casinos shut in March 2020 and the money arrives in FY2021, because the August payment
     * settles the half-year that ended in June. A page that put it in FY2020 would be describing
     * the revenue period, which is not what any other figure on the route is on.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator('.card[data-part="casino"]');
    await expect(card).toContainText("FY2021 is the closure");
    await expect(card).toContainText("mid-March 2020");
    await expect(card).toContainText("Nothing cushioned it");
  });

  test("no per-pupil figure appears, and the card says which count it would have had to use", async ({
    page,
  }) => {
    /*
     * Four per-pupil figures already sit on a district's pages. A fifth computed from this fund
     * would read as one of them and would divide by a denominator R.C. 5753.11 defines for this
     * fund alone — county-resident pupils, community and STEM and joint vocational enrolment
     * included, dual-enrolled pupils counted twice. The refusal is the finding.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const card = page.locator('.card[data-part="casino"]');
    await expect(card).toContainText("no per-pupil figure here");
    await expect(card).toContainText("R.C. 5753.11");
    await expect(card.locator(".tile", { hasText: "per pupil" })).toHaveCount(0);
  });

  test("the dashboard's caveat card sends a reader to it rather than staying silent", async ({
    page,
  }) => {
    // "What this page is not" already told a reader that none of the figures above it is money
    // received. It could not say that the finances route was also short of some — which made the
    // two routes together look exhaustive.
    await page.goto(`/district/${CLEVELAND}`);
    const card = page.locator('.card[data-part="not"]');
    await expect(card).toContainText("there is state money in neither");
    await expect(card.locator('a[href$="/finances#casino"]')).toHaveCount(1);
  });
});

test.describe("the statute timeline", () => {
  test("the Law menu opens onto it, and it is the first thing in the panel", async ({ page }) => {
    await page.goto("/");
    const law = page.locator("header.site nav details.menu").nth(1);
    await law.locator("summary").click();
    const first = law.locator(".menu-panel a").first();
    await expect(first).toHaveAttribute("href", "/legislation");
    await first.click();
    await expect(page).toHaveURL(/\/legislation$/);
    await expect(page.locator("h1")).toHaveText("Ohio school funding in statute");
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
    await expect(acts.filter({ hasText: "H.B. 920" })).toContainText("no formula edge");
    await expect(acts.filter({ hasText: "H.B. 920" })).toContainText("not a budget act");
    await expect(acts.filter({ hasText: "H.B. 583" })).toContainText("corrects");
    await expect(page.locator("#acts tbody td:empty")).toHaveCount(0);
  });
});
