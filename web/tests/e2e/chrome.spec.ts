/**
 * The furniture every page carries: the bar above the fold, the routes it points at, the section
 * menus inside it, and what a document means once you stop looking at it.
 *
 * Four blocks that are all about the layout rather than about any one route, which is why they are
 * together: a defect in any of them is a defect on 3,500 pages. `document semantics` keeps only the
 * assertions that need a browser — focus order, where a fragment target lands, what a scroll does
 * to a focused control. Its two artefact sweeps, over `<th scope>` and skipped heading levels,
 * moved to `tests/dist/semantics.spec.ts`.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND, NORTHERN } from "./helpers.ts";

test.describe("the chrome above the fold", () => {
  /**
   * A fragment lands its target clear of whatever is stuck to the top of the screen.
   *
   * `--sticky-chrome` exists for exactly this and it was wrong in both directions: 96px against a
   * 52px header between 680 and 999, and 87px against a 105px covered region from 1000 up — so at
   * every desktop width a fragment link put its own heading 19px underneath the bar the reader was
   * looking at. #189 re-derived it to 52 and 106.
   *
   * The four widths are the ones the token used to step at. Three of them no longer have a step,
   * which is the point: the header is one row everywhere now, so a value measured at 700px is the
   * same value measured at 480px, and a test that only checked the step boundaries would not have
   * caught the 43px error at 520px that the old four had between them.
   */
  for (const width of [375, 480, 700, 1000]) {
    test(`a fragment lands clear of the sticky chrome at ${width}px`, async ({ page }) => {
      await page.setViewportSize({ width, height: 800 });
      await page.goto("/district/043786");

      const landed = await page.evaluate(async () => {
        location.hash = "#categoricals";
        await new Promise((settle) => setTimeout(settle, 80));
        const target = document.getElementById("categoricals")!.getBoundingClientRect();
        // Only what is stuck ABOVE the target covers it. The dashboard's contents rail (#549) is
        // sticky too, from 1000px, and sits in a column beside the cards: its bottom is most of
        // the window down, and it covers none of them.
        const covered = Math.max(
          0,
          ...[...document.querySelectorAll("body *")]
            .filter((node) => {
              const style = getComputedStyle(node);
              if (style.position !== "sticky" || style.top === "auto") return false;
              const box = node.getBoundingClientRect();
              return box.left < target.right && box.right > target.left;
            })
            .map((node) => node.getBoundingClientRect().bottom),
        );
        return {
          covered: Math.round(covered),
          top: Math.round(document.getElementById("categoricals")!.getBoundingClientRect().top),
        };
      });

      expect(landed.covered, "nothing is stuck to the top at all").toBeGreaterThan(0);
      expect(
        landed.top,
        `the target lands ${landed.covered - landed.top}px under the chrome`,
      ).toBeGreaterThanOrEqual(landed.covered);
    });
  }

  test("the header is one row at every width, and the page never scrolls sideways", async ({
    page,
  }) => {
    /*
     * The defect, as a number: 166px of header at 390px and the `h1` beginning at y=194, which is
     * a quarter of a phone screen spent before the page says which district it is about.
     *
     * Checked across the range rather than at the breakpoints, because the header height used to
     * be a continuous function of width — it wrapped — and the four `--sticky-chrome` values were
     * measured at four widths that were their own breakpoints rather than at the widths where the
     * layout actually changed. 520px was 43px out and nothing looked at 520px.
     */
    const heights: string[] = [];
    /*
     * What the row was asked to hold, at each width, so a wrap says by how much.
     *
     * This is not asserted and cannot be: the brand is set in `system-ui`, which is a different
     * typeface on the machine running this than on the machine reading it, and the runner's is
     * wider than a Mac's. A header that wrapped used to report only that it was 92px instead of
     * 52 — true, and no help at all in deciding how much room to find. These are the numbers that
     * decide it, so they travel with the failure.
     */
    const room: string[] = [];
    /*
     * 820 is the tightest the loose bar gets: the first width at which the six entries sit in the
     * row rather than inside `Menu`. #548 widened that row — two flat entries and four menus,
     * where there had been five menus — so it is measured where it is narrowest rather than at
     * 900, where it has room it does not have at 820.
     */
    for (const width of [360, 390, 480, 520, 700, 820, 900, 1000, 1280]) {
      await page.setViewportSize({ width, height: 800 });
      await page.goto("/district/043786");
      const state = await page.evaluate(() => {
        const row = document.querySelector(".site-inner")!;
        const style = getComputedStyle(row);
        const inner =
          row.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
        const children = [...row.children].map((child) => ({
          what: child.className.split(" ")[0] || child.tagName.toLowerCase(),
          w: child.getBoundingClientRect().width,
        }));
        const gaps = (children.length - 1) * parseFloat(style.columnGap);
        const wants = children.reduce((total, child) => total + child.w, 0) + gaps;
        return {
          header: Math.round(document.querySelector("header.site")!.getBoundingClientRect().height),
          h1: Math.round(document.querySelector("h1")!.getBoundingClientRect().top + window.scrollY),
          sideways:
            document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
          room: `${Math.round(inner - wants)} spare (${Math.round(inner)} for ${children
            .map((child) => `${child.what} ${Math.round(child.w)}`)
            .join(" + ")} + ${Math.round(gaps)} gap)`,
        };
      });
      expect(state.sideways, `the document scrolls sideways at ${width}px`).toBe(false);
      heights.push(`${width}: header ${state.header}, h1 at ${state.h1}`);
      room.push(`${width}: ${state.room}`);
    }
    // One row is 52px. Asserted as a set so a regression names the width it happened at.
    expect(heights, `what the row was holding —\n${room.join("\n")}\n`).toEqual([
      "360: header 52, h1 at 80",
      "390: header 52, h1 at 80",
      "480: header 52, h1 at 80",
      "520: header 52, h1 at 80",
      "700: header 52, h1 at 80",
      "820: header 52, h1 at 80",
      "900: header 52, h1 at 80",
      "1000: header 52, h1 at 80",
      "1280: header 52, h1 at 80",
    ]);
  });

  test("the header leaves the site nowhere, and the repository is in the footer at every width", async ({
    page,
  }) => {
    /*
     * Every figure on this site names the crate that computes it, and a reader following that
     * attribution needs somewhere to go. The repository link was that somewhere, in the header
     * beside the theme button and hidden below 480px for want of room. #548 moved it to the
     * footer: the bar is for moving around this site, and it needed the width for six entries.
     *
     * So two claims. Nothing in the header leaves the site — a link added back there is a choice
     * this test makes somebody state. And the footer carries the link at the narrowest width and
     * the widest, by its name in words, opening a new tab without handing it this one.
     */
    for (const width of [360, 1000]) {
      await page.setViewportSize({ width, height: 800 });
      await page.goto("/district/043786");
      const outbound = await page.evaluate(() =>
        [...document.querySelectorAll("header.site a")]
          .map((a) => (a as HTMLAnchorElement).href)
          .filter((href) => new URL(href).origin !== location.origin),
      );
      expect(outbound, `the header links off the site at ${width}px`).toEqual([]);

      const mark = page.locator("footer.site a.mark");
      await expect(mark, `the repository link at ${width}px`).toBeVisible();
      await expect(mark).toHaveAccessibleName("Source on GitHub");
      await expect(mark).toHaveAttribute("href", "https://github.com/goedelsoup/ohio-education-funding");
      await expect(mark).toHaveAttribute("rel", "noopener noreferrer");
      await expect(mark).toHaveAttribute("target", "_blank");
    }
  });

  test("keyboard focus never lands under the header", async ({ page }) => {
    /*
     * The same question as the fragment, asked the other way. A reader tabbing down a long page
     * has the browser scroll each focused control into view, and `scroll-margin-top` is what keeps
     * it out from under a sticky bar — the same token, spent on a different mechanism.
     *
     * # Asked by hit-testing, after two false starts
     *
     * The obvious version compares the control's bottom edge against the lowest sticky bottom, and
     * it is wrong twice over. It reported the brand and the section menus, which are *inside* the
     * sticky header and so are the chrome rather than hidden by it. It then reported the county
     * and seat links, because at scroll 0 the district sub-navigation has not been reached yet and
     * sits in flow at y≈234, covering nothing. Both were fixed and it still reported the skip link,
     * which has `z-index: 10` against the header's 5 and is painted *over* it — geometry cannot
     * see that at all.
     *
     * So the question is put to the renderer: at the middle of the control, what is on top? If it
     * is the control, a reader can see and click it. That answers z-order, transforms and the
     * chrome-is-not-hiding-itself case in one, without this test having to model any of them.
     *
     * The settle loop is for the skip link too: `.skip` animates `top` over 120ms on focus, so a
     * single frame's wait catches it in flight somewhere between -64px and +12px. Waiting for the
     * rect to stop moving is what makes this deterministic — it passed alone and failed once in a
     * full run before that was added, which is the signature of a race and not of a defect.
     */
    /*
     * As a reader who has asked for less motion, which is not a workaround.
     *
     * `.skip` animates `top` over 120ms when it takes and loses focus, so tabbing off it leaves it
     * mid-flight across the brand for two more frames — the hit test caught exactly that and
     * reported `a Ohio school funding <- a.skip`. Waiting for the *focused* element to settle does
     * not help, because the thing still moving is the one that just lost focus.
     *
     * `app.css` already turns that transition off under `prefers-reduced-motion`, so asking for it
     * removes the only animation in the path and exercises a configuration real readers use. The
     * geometry this test is about is identical either way.
     */
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.setViewportSize({ width: 1000, height: 800 });
    await page.goto("/district/043786");

    /*
     * BACKWARDS, from the bottom, and that is the whole test.
     *
     * Tabbing forwards cannot land a control under the top chrome: chromium scrolls the minimum
     * distance, so a control below the fold arrives at the *bottom* edge. Measured — of 40 forward
     * stops only two scroll the page at all, and with `scroll-margin-top` deleted entirely this
     * test still passed. It was close to vacuous.
     *
     * Scrolling up is the direction that aligns to the top, and it is the direction a reader goes
     * when they shift-tab back to something they passed. That is the case `scroll-margin-top`
     * exists for and the one where a wrong `--sticky-chrome` shows.
     */
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await page.locator("footer a").last().focus();

    const obscured: string[] = [];
    for (let step = 0; step < 40; step += 1) {
      await page.keyboard.press("Shift+Tab");
      const state = await page.evaluate(async () => {
        const frame = () => new Promise((settle) => requestAnimationFrame(settle));
        const active = document.activeElement as HTMLElement | null;
        if (!active || active === document.body) return null;

        let last = "";
        for (let wait = 0; wait < 12; wait += 1) {
          await frame();
          const now = JSON.stringify(active.getBoundingClientRect());
          if (now === last) break;
          last = now;
        }

        const box = active.getBoundingClientRect();
        if (box.width === 0 || box.height === 0) return null;
        const x = box.left + box.width / 2;
        const y = box.top + box.height / 2;
        if (x < 0 || y < 0 || x > innerWidth || y > innerHeight) return null;

        const onTop = document.elementFromPoint(x, y);
        return {
          covered: onTop != null && onTop !== active && !active.contains(onTop) && !onTop.contains(active),
          what: `${active.tagName.toLowerCase()} ${(active.textContent ?? "").trim().slice(0, 24)}`,
          behind: onTop ? `${onTop.tagName.toLowerCase()}.${String(onTop.className).split(" ")[0]}` : "",
        };
      });
      if (state?.covered) obscured.push(`${state.what} <- ${state.behind}`);
    }

    expect(obscured.slice(0, 3), "a focused control is painted over by something").toEqual([]);
  });
});

test.describe("routes", () => {
  test("each of a district's four views is its own address, and the fifth tab opens the runner on it", async ({
    page,
  }) => {
    // `/taxes` landed after the other four and was left out of this list, so the one nav state
    // nothing had ever asserted was the heaviest sibling's. The label is read off the rendered
    // nav rather than hard-coded twice, so a rename fails here rather than passing on a stale
    // expectation.
    for (const [path, heading] of [
      ["", "Dashboard"],
      ["/outcome", "Outcome"],
      ["/finances", "Finances"],
      ["/taxes", "Property tax"],
    ] as const) {
      await page.goto(`/district/${NORTHERN}${path}`);
      await expect(page.locator("h1")).toHaveText("Northern Local");
      await expect(page.locator(`.subnav a[aria-current="page"]`)).toHaveText(heading);
    }
    // The scenario tab was `/district/[irn]/scenario`, a copy of the runner per district. It is
    // the runner now, opened with the district chosen (#548).
    await page.locator(".subnav a", { hasText: "Change the formula" }).click();
    await expect(page).toHaveURL(new RegExp(`/scenario\\?d=${NORTHERN}`));
    await expect(page.locator("h1")).toHaveText("Change the formula");
    await expect(page.locator("#sc-district")).toHaveValue(NORTHERN);
    await expect(page.locator(`.subnav a[aria-current="page"]`)).toHaveText("What changes");
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

    // The table sent this to `/` after the statewide panel had moved to `/statewide`, and the
    // no-loop guard turned that into doing nothing.
    await page.goto("/#statewide");
    await expect(page).toHaveURL(/\/statewide$/);

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
    // Thirty-two across five groups, again. An exact count rather than a floor, so that dropping an
    // entry fails here and adding one is an acknowledged change — and so that a derivation which
    // quietly stops selecting anything cannot pass by returning an empty menu. It went from
    // thirty to thirty-one when `/legislation` joined the `Law` panel, to thirty-two when
    // `/reach` joined `Research` beside the scenario runner it splits the second question off,
    // and to thirty-three when `/bounds` joined the same panel — the census of the plan's own
    // edges, which is the only page here whose subject is the shape of the whole formula rather
    // than a quantity it produces. And back to thirty-two when `/reach` became the runner's second
    // view (`/scenario/reach`), reached from a tab on the runner rather than from the bar (#548).
    //
    // Then thirty-four, when #548 rebuilt the bar by task: the seven acts and five regimes the
    // corpus selected for `Law` and `Formula` went, and every one of the eighteen classes came in
    // under `Library`, beside Sources and Decisions — two flat links, 5 places, 3 analyses, 22 in
    // the library and 2 about. Which is the mechanism working: the count is changed on purpose by
    // somebody who knew why.
    expect(hrefs).toHaveLength(34);
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
    const about = page.locator("header.site nav details.menu").last();

    await places.locator("summary").click();
    await expect(places).toHaveAttribute("open", "");

    await about.locator("summary").click();
    await expect(about).toHaveAttribute("open", "");
    await expect(places).not.toHaveAttribute("open", "");

    await page.keyboard.press("Escape");
    await expect(about).not.toHaveAttribute("open", "");
    // Focus returns to the summary rather than to the top of the document.
    await expect(about.locator("summary")).toBeFocused();
  });

  test("a click outside closes an open menu", async ({ page }) => {
    await page.goto("/");
    const places = page.locator("header.site nav details.menu").filter({ hasText: "Places" });
    await places.locator("summary").click();
    await expect(places).toHaveAttribute("open", "");
    await page.locator("h1").click();
    await expect(places).not.toHaveAttribute("open", "");
  });

  test("the Library panel opens inside the window at every width it hangs from the bar", async ({
    page,
  }) => {
    /*
     * `Library` is every class of the corpus, and one column of it ran to about 800px — past the
     * fold of a laptop, in an absolutely positioned box a reader cannot scroll to. It is two
     * columns above 640px, and above 820px it hangs from its own right edge, because it is the
     * fifth entry of six and a 30rem box hung from its left would run off the right of the window.
     *
     * 700 is inside the `Menu` disclosure; 820 is the narrowest loose bar; 1280 is the widest the
     * header grows. At each, the whole panel is on screen: nothing clipped at either side, and the
     * last link above the bottom of an 800px window.
     */
    for (const width of [700, 820, 1280]) {
      await page.setViewportSize({ width, height: 800 });
      await page.goto("/");
      if (width < 820) await page.locator("header.site nav details.menu-all > summary").click();
      const library = page.locator("header.site nav details.menu").nth(2);
      await library.locator("summary").click();
      const panel = library.locator(".menu-panel");
      const box = (await panel.boundingBox())!;
      expect(box.x, `Library runs off the left at ${width}px`).toBeGreaterThanOrEqual(0);
      expect(box.x + box.width, `Library runs off the right at ${width}px`).toBeLessThanOrEqual(width);
      expect(box.y + box.height, `Library runs below the window at ${width}px`).toBeLessThanOrEqual(800);
      await expect(panel.locator("a").last()).toBeInViewport();
    }
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
     *
     * # Two opens now, and the risk this guards went up rather than down
     *
     * #189 folded the menus into one outer disclosure below the nav's breakpoint, so
     * reaching `Library` on a phone means opening `Menu` and then opening `Library` inside it. This test caught that change by
     * timing out on a summary that is no longer rendered — which is the right failure, because the
     * thing it is about is now worse in principle: an open menu is nested one level deeper inside
     * the same sticky box, and the box still may not outgrow the screen.
     */
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/");

    await page.locator("header.site nav details.menu-all > summary").click();
    // `Library` since #548: every class of the corpus, twenty-two links in five headed runs, and
    // one column at this width. `Law` was seven.
    const library = page.locator("header.site nav details.menu").nth(2);
    await library.locator("summary").click();
    await expect(library).toHaveAttribute("open", "");

    const header = page.locator("header.site");
    expect((await header.boundingBox())!.height).toBeLessThanOrEqual(667);

    const last = library.locator(".menu-panel a").last();
    await last.scrollIntoViewIfNeeded();
    await expect(last).toBeInViewport();
  });
});

/*
 * What a document means, rather than what it looks like.
 *
 * The structural baseline here was already strong — across 3,487 pages, zero unlabelled form
 * controls, zero images without `alt`, zero duplicate ids, zero positive `tabindex`. These are the
 * layer above it, and the reason none of them were caught is the reason `axe.spec.ts` exists: this
 * was a 245-test suite with no automated accessibility assertion in it.
 *
 * The two assertions that swept the build rather than a page — every `<th>` has a `scope`, no page
 * skips a heading level — are in `tests/dist/semantics.spec.ts`. What is left here needs a browser:
 * focus order, where a fragment target lands, and what a scroll does to a focused control.
 */
test.describe("document semantics", () => {
  test("the first thing a keyboard reaches is a way past the bar", async ({ page }) => {
    /*
     * There was no skip link anywhere on the site. Every page opens with a header carrying a brand
     * link and five section disclosures, so a keyboard reader passed six controls before the
     * content on every page, and a screen-reader reader heard the whole bar again on each.
     */
    await page.goto(`/district/${CLEVELAND}`);
    await page.keyboard.press("Tab");
    const first = page.locator(":focus");
    await expect(first).toHaveClass(/skip/);
    // Off-screen until focused, and on screen once it is — `display: none` would take it out of
    // the tab order and leave the page exactly as it was.
    await expect(first).toBeVisible();

    await first.press("Enter");
    expect(await page.evaluate(() => location.hash)).toBe("#main");
    // And it lands clear of the sticky header rather than under it.
    const top = await page.locator("#main").evaluate((el) => el.getBoundingClientRect().top);
    expect(top, "the content starts below the chrome, not beneath it").toBeGreaterThan(0);
  });

  test("a fragment target lands clear of the header, prose headings included", async ({ page }) => {
    /*
     * `--sticky-chrome` is measured per breakpoint and was spent on `.card[id]`, `.basis-scope[id]`
     * and `h3[id]`. The corpus's own prose headings render at `h2`, and 88 of them across 37 pages
     * were fragment targets that landed completely underneath the header.
     */
    await page.goto("/wiki/doctrine/equity");
    const ids = await page.locator("#description h2[id]").evaluateAll((nodes) =>
      nodes.slice(0, 3).map((n) => n.id),
    );
    expect(ids.length, "the page has prose headings to address").toBeGreaterThan(0);
    for (const id of ids) {
      await page.goto(`/wiki/doctrine/equity#${id}`);
      // An attribute selector, not `#id`: these ids come from heading text and carry characters a
      // bare fragment selector will not take.
      const top = await page
        .locator(`[id="${id}"]`)
        .evaluate((el) => el.getBoundingClientRect().top);
      expect(top, `#${id} lands under the header`).toBeGreaterThan(0);
    }
  });

  test("a focused control is not scrolled under the header", async ({ page }) => {
    // The same measured offset, which had been spent only on fragment links. A browser scrolls a
    // focused element into view with the same arithmetic and the same header in the way.
    await page.goto("/districts");
    const button = page.locator('#district-table thead button[data-sort="aid"]');
    await page.evaluate(() => window.scrollTo(0, 2000));
    await button.focus();
    const box = await button.evaluate((el) => el.getBoundingClientRect().top);
    const chrome = await page.evaluate(() =>
      Number.parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--sticky-chrome")),
    );
    expect(box, "focus landed under the sticky header").toBeGreaterThanOrEqual(chrome);
  });
});
