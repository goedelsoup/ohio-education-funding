/**
 * What is on this page, and the address of every card on it.
 *
 * Two blocks about the section contract: a page says what it contains, and each thing it contains
 * can be linked to. They are one file because the second is what makes the first useful — a
 * contents list whose entries do not resolve is decoration.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND } from "./helpers.ts";

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

test.describe("the room a card is given", () => {
  test("the first card stands clear of the note or lede above it, on every route family", async ({
    page,
  }) => {
    /*
     * A lede `.note` had a top margin and no bottom one, so on `/county/*` and `/house/*` the first
     * card sat flush against it — 0px, against 18px on the routes that open with a `.lead` and 52px
     * between cards (#578). The floor is the `.lead`'s 1.1rem, 17.6px.
     */
    for (const route of ["/county/ottawa", "/house/090", "/senate/003", "/counties", "/house"]) {
      await page.goto(route);
      const gap = await page.evaluate(() => {
        const card = document.querySelector("main .card");
        if (!card) return null;
        const top = card.getBoundingClientRect().top;
        // The lowest edge of anything in `main` before the card that is not one of its ancestors.
        let bottom = -Infinity;
        for (const n of document.querySelectorAll("main *")) {
          if (n === card) break;
          if (n.contains(card)) continue;
          const r = n.getBoundingClientRect();
          if (r.height > 0) bottom = Math.max(bottom, r.bottom);
        }
        return top - bottom;
      });
      expect(gap, `${route} has no card`).not.toBeNull();
      expect(gap!, `the first card on ${route} sits against what is above it`).toBeGreaterThanOrEqual(17.5);
    }
  });

  test("on a phone, no question on the home page wraps past two lines", async ({ page }) => {
    // The name column of a two-column prose table is 32%, 94px at 375, and each question link
    // wrapped to four lines (#578). Below 500px the rows stack instead.
    await page.setViewportSize({ width: 375, height: 800 });
    await page.goto("/");
    const links = page.locator("#three-questions table.prose th a");
    await expect(links).toHaveCount(3);
    // An inline element has one client rect per line box it occupies.
    const lines = await links.evaluateAll((nodes) => nodes.map((n) => n.getClientRects().length));
    for (const count of lines) expect(count, `a question takes ${count} lines`).toBeLessThanOrEqual(2);
  });
});
