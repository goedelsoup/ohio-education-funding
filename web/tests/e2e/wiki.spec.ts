/**
 * The corpus, as rendered: panels, wide tables, node pages, and the decisions behind them.
 *
 * Four blocks about the routes that render the knowledge graph rather than the formula. The two
 * layout blocks lead because they are the mechanism the node pages depend on — a panel grid and a
 * sideways-scrolling table are what a corpus node mostly is.
 *
 * Six assertions from `panels laid out against each other` read the build rather than a page and
 * are now in `tests/dist/` — the scroll-box labels and ragged-left prose in `semantics.spec.ts`,
 * the MathML, the duplicate property names and the two font checks in `formulas.spec.ts`.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test, type Page } from "@playwright/test";

import { CLEVELAND } from "./helpers.ts";

/*
 * A set of panels, and the one that is alone on its row.
 *
 * Every drawing on this site is an SVG at `width: 100%` over a fixed `viewBox`, so the layout does
 * not decide how large a chart is drawn — it decides what that drawing is *scaled by*, and the
 * type scales with it. `.panels` was wrapped flex with `flex: 1 1 280px`, which grows per row: the
 * seventh panel of #444's small multiples sat alone on the fourth row and took the whole card,
 * 804px of it against the 311 it was drawn at, while its six siblings sat at 393. It arrived as
 * one enormous panel with 26px type beside six normal ones, and the rule it was drawing does
 * nothing, so it had no bars either. It read exactly like a chart that had failed to render, and
 * that is how it was reported.
 *
 * Nothing in the unit tests can see this: `panelWidth` is asserted there and it was right. The
 * fault is entirely in how the page lays the drawing out, so it is measured on the page.
 */
test.describe("panels laid out against each other", () => {
  // The node that draws seven of them — an odd number, which is what it takes to leave one alone
  // on the last row. A set of two or six would have passed the whole time.
  const PANELLED = "/wiki/formula-component/temporary-transitional-aid-guarantee";

  /** Each set of panels on the page, as the widths its visible drawings are painted at. */
  const sets = (page: Page) =>
    page.evaluate(() =>
      [...document.querySelectorAll(".panels")].map((set) => ({
        key: set.closest("[data-series]")?.getAttribute("data-series") ?? "unnamed",
        panels: [...set.querySelectorAll("svg.plot")]
          // One of the two drawings is `display: none` at any width — see `renderToString`.
          .filter((svg) => svg.getClientRects().length > 0)
          .map((svg) => ({
            width: Math.round(svg.getBoundingClientRect().width),
            drawn: (svg as SVGSVGElement).viewBox.baseVal.width,
          })),
      })),
    );

  test("every panel of a set is painted at the width of every other", async ({ page }) => {
    for (const width of [1440, 1024, 768, 420]) {
      await page.setViewportSize({ width, height: 1200 });
      await page.goto(PANELLED);
      const drawn = await sets(page);
      // A set of one is a chart with a heading on it — the plane on this same node is drawn
      // through the same component — so what is asserted below has to have a set with siblings
      // in it to be asserting anything.
      expect(
        drawn.filter((set) => set.panels.length > 1).length,
        `${width}px draws no set of more than one panel`,
      ).toBeGreaterThan(0);
      for (const set of drawn) {
        // Exactly equal, not nearly: the panels of a set share a scale, and two panels of one set
        // scaled differently draw the same dollar at two lengths.
        expect(
          [...new Set(set.panels.map((panel) => panel.width))],
          `${set.key} at ${width}px paints its panels at different widths`,
        ).toHaveLength(1);
      }
    }
  });

  test("no panel is blown up to fill a row it is alone on", async ({ page }) => {
    for (const width of [1440, 1024, 768, 420]) {
      await page.setViewportSize({ width, height: 1200 });
      await page.goto(PANELLED);
      for (const set of await sets(page)) {
        for (const panel of set.panels) {
          // 1.4 leaves room for the ordinary case — a panel drawn at 311 in a 393px column is
          // 1.26 — and is nowhere near the 2.58 the stretched one arrived at. The ceiling is on
          // the scale factor rather than on the type size because it is the scale factor that the
          // layout chooses; the type only follows it.
          expect(
            panel.width / panel.drawn,
            `${set.key} at ${width}px: a panel drawn at ${panel.drawn} is painted at ${panel.width}`,
          ).toBeLessThan(1.4);
        }
      }
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

  /**
   * And it is the block that scrolls, not the table around it.
   *
   * `table-layout: fixed` is what decides this and does not look like it does. Without it the cell
   * sizes to its widest unbreakable content, the table outgrows the card, and the outer `.scroll`
   * takes the scroll — so a reader dragging a piecewise rule sideways to reach its `then` values
   * drags the property name off the screen with it.
   *
   * The node moved. This used to open the local capacity measure, whose `function` was the widest
   * aligned block in the corpus — and #203 gave that node a `function_tex`, so its ASCII statement
   * is no longer what the page shows. `fy2022_inputs` on the base cost calculation is now the
   * subject: a table of statewide salaries, wider than a phone, and a property nothing is going to
   * typeset because it is not a calculation.
   */
  test("an aligned property block scrolls itself, and the page never scrolls with it", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 900 });
    await page.goto("/wiki/formula-component/fsfp-base-cost-calculation");

    const block = page.locator("pre.aligned").first();
    await expect(block).toHaveAttribute("tabindex", "0");
    await expect(block).toHaveAttribute("aria-label", /\S/);

    const state = await block.evaluate((el) => ({
      blockScrolls: el.scrollWidth > el.clientWidth + 1,
      tableScrolls: (() => {
        const box = el.closest("div.scroll");
        return box ? box.scrollWidth > box.clientWidth + 1 : false;
      })(),
      pageScrolls: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
    }));
    expect(state.blockScrolls, "a wide salary table at 375px has something behind its edge").toBe(true);
    expect(state.tableScrolls, "the table must not be what scrolls").toBe(false);
    expect(state.pageScrolls, "the document must never scroll sideways").toBe(false);

    // Reachable and movable, which is the whole point of the tab stop.
    await block.focus();
    for (let i = 0; i < 6; i += 1) await page.keyboard.press("ArrowRight");
    await expect
      .poll(() => block.evaluate((el) => el.scrollLeft), { message: "the keyboard moved it" })
      .toBeGreaterThan(0);
  });

  /**
   * And a formula scrolls itself, like every other wide thing on this site.
   *
   * The scroll is on the `.formula` wrapper and never on the `<math>` element: measured, with
   * `overflow-x: auto` on a `display: block math` box Chromium reports `scrollWidth ===
   * clientWidth` while visibly cutting the last row in half.
   */
  test("a formula scrolls inside its own box and never the page", async ({ page }) => {
    for (const width of [375, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      await page.goto("/wiki/formula-component/fsfp-local-capacity-measure");
      const state = await page.locator(".formula").first().evaluate((box) => ({
        boxScrolls: box.scrollWidth > box.clientWidth + 1,
        mathOverflow: getComputedStyle(box.querySelector("math")!).overflowX,
        pageScrolls: document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
        emptyWithChildren: [...box.querySelectorAll("mspace")].filter((e) => e.childNodes.length > 0).length,
      }));
      expect(state.pageScrolls, `the document scrolls sideways at ${width}px`).toBe(false);
      expect(state.mathOverflow, "the scroll belongs to the wrapper, not the element").toBe("visible");
      expect(state.emptyWithChildren, "an <mspace> has eaten the rest of its row").toBe(0);
      if (width === 375) expect(state.boxScrolls, "a wide formula scrolls on a phone").toBe(true);
    }
  });

  test("a named quantity in a formula reaches the parameter that defines it", async ({ page }) => {
    /*
     * #204's last clause, and the one that needed measuring rather than assuming: `href` on a
     * MathML element is dead in MathML Core — the cursor stays `auto` and the element does not
     * activate — so the anchor goes inside an `<mtext>`, which is the one arrangement that
     * renders. `lib/math.ts` holds it to the allowlist, and this is the half only a browser sees.
     *
     * Three things, because the link failing in three different ways looks identical in the
     * markup: it has to be laid out (a zero-width anchor inside a maths box is not a target), it
     * has to be distinguishable **without colour** — the underline is the channel, the same one
     * `--link` carries everywhere else since #187 — and it has to navigate.
     */
    await page.goto("/wiki/formula-component/fsfp-special-education-weights");
    const term = page.locator(".formula mtext a").first();
    await expect(term).toBeVisible();

    const shown = await page.locator(".formula").first().evaluate((box) => {
      const a = box.querySelector("mtext a")!;
      const style = getComputedStyle(a);
      const rect = a.getBoundingClientRect();
      /* An `<mtext>` on the same page carrying no link: the ink to compare against. */
      const plain = [...box.querySelectorAll("mtext")].find((t) => !t.querySelector("a"))!;
      return {
        width: rect.width,
        height: rect.height,
        decoration: style.textDecorationLine,
        cursor: style.cursor,
        ink: style.color,
        underline: style.textDecorationColor,
        neighbour: getComputedStyle(plain).color,
      };
    });
    expect(shown.width, "a term with no width is not a target").toBeGreaterThan(20);
    expect(shown.height).toBeGreaterThan(5);
    expect(shown.decoration, "the underline is the channel that survives greyscale").toContain(
      "underline",
    );
    expect(shown.cursor).toBe("pointer");

    /*
     * And the word stays in the formula's own ink, which is the site's link doctrine since #187:
     * colour is a hover and focus state, never the identity. A `.formula a { color }` rule added
     * later would repaint one named quantity inside a line of maths and read as emphasis rather
     * than as a link — so the two inks matching is the assertion, and the underline being a
     * different colour from both is what says the anchor is styled at all.
     */
    expect(shown.ink, "a linked term is the same ink as the maths around it").toBe(shown.neighbour);
    expect(shown.underline, "the underline carries --link and the word does not").not.toBe(shown.ink);

    await term.click();
    await expect(page.locator("h1")).toHaveText("Base Cost Per Pupil");
  });

  /**
   * The shipped font stretches a brace — asked of the browser, not of the table.
   *
   * `tests/unit/math-font.spec.ts` reads the file and says the `MATH` table is there with thirteen
   * variants and five assembly parts behind the brace. That is a claim about bytes. This is the
   * claim that matters: that a browser handed only this font draws a delimiter the height of what
   * it delimits. A subsetter that dropped `MATH`, or kept it while dropping the glyphs it points
   * at, produces a font that passes every structural check and fails here.
   *
   * The family has to be forced. `--font-math` names Cambria Math and STIX Two Math ahead of the
   * shipped one on purpose, so on any machine with either — which is every machine this suite runs
   * on except a bare Linux runner — the fallback is never fetched and a test that just loaded the
   * page would be measuring the platform's font and reporting it as ours.
   *
   * The control is in the same test. `serif` has no `MATH` table, and it is what a reader with no
   * maths face would see if this font did not exist: the brace stays at one line while the table
   * it belongs to is five, which is the state #202 was opened to end.
   */
  test("a brace stretches with the shipped font, and does not without it", async ({ page }) => {
    await page.goto("/wiki/formula-component/fsfp-local-capacity-measure");
    await page.waitForFunction(() => document.fonts.status === "loaded");

    const measure = async (family: string) =>
      page.locator(".formula math").first().evaluate(async (math, declared) => {
        (math as HTMLElement).style.fontFamily = declared;
        await document.fonts.load(`17px ${declared}`);
        await document.fonts.ready;
        const brace = [...math.querySelectorAll("mo")].find((mo) => mo.textContent?.trim() === "{");
        const rows = brace?.parentElement?.querySelector("mtable");
        return {
          brace: brace?.getBoundingClientRect().height ?? 0,
          rows: rows?.getBoundingClientRect().height ?? 0,
          drawnWith: getComputedStyle(brace!).fontFamily,
        };
      }, family);

    const shipped = await measure('"Ohio Math Fallback"');
    expect(shipped.rows, "the cases block is several rows tall").toBeGreaterThan(60);
    expect(shipped.drawnWith).toContain("Ohio Math Fallback");
    // Not "taller than one line" — the brace has to reach the block. Anything less is a font that
    // stepped up through its variants and then ran out, which is a dropped GlyphAssembly.
    expect(shipped.brace, "the brace does not reach the block it opens").toBeGreaterThan(shipped.rows * 0.95);

    const none = await measure("serif");
    expect(none.brace, "a font with no MATH table must not stretch, or this proves nothing").toBeLessThan(
      none.rows * 0.5,
    );
  });

  /**
   * A property name never prints on top of its own value.
   *
   * This is here because it happened. `table-layout: fixed` — added so an aligned block scrolls
   * inside itself rather than dragging the table with it — also stops the name column growing for
   * a long name, and `statutory_basis` is fifteen monospace characters against a 99px column at
   * 390px. It ran across its own cell boundary and printed over the value: "statutory_ba*stat*e
   * share percentage". Nothing in the suite noticed; it was found by looking at a screenshot.
   *
   * The `#186` collision sweep exists and did not cover it, because that one walks three routes to
   * check a display face across six fallbacks and none of them is a corpus node. This walks the
   * property tables instead, at the width where a fixed column is tightest.
   */
  test("a property name never overlaps the value beside it", async ({ page }) => {
    const ROUTES = [
      // A long name against a block value, which is the case that broke.
      "/wiki/formula-component/fsfp-local-capacity-measure",
      // The longest property names in the corpus: `procedural_history`, `general_assembly`.
      "/wiki/litigation/derolph-ii-2000",
      "/wiki/legislation/hb-33-2023",
      "/wiki/parameter/fsfp-phase-in-percentage",
      "/wiki/metric/per-pupil-operating-expenditure",
    ];
    for (const width of [375, 768]) {
      await page.setViewportSize({ width, height: 900 });
      for (const route of ROUTES) {
        await page.goto(route);
        const overlaps = await page.evaluate(() =>
          [...document.querySelectorAll("table.prose tr")].flatMap((row) => {
            const name = row.querySelector("th");
            const value = row.querySelector("td");
            if (!name || !value) return [];
            // The name's own text box rather than the cell's: the cell stayed put and the word
            // inside it was what escaped.
            const box = (name.firstElementChild ?? name).getBoundingClientRect();
            const beside = value.getBoundingClientRect();
            return box.right > beside.left + 1 ? [`${name.textContent?.trim()}`] : [];
          }),
        );
        expect(overlaps, `${route} at ${width}px`).toEqual([]);
      }
    }
  });
});

test.describe("the wiki", () => {
  test("renders a node with its properties, links and backlinks", async ({ page }) => {
    await page.goto("/wiki/parameter/twenty-mill-floor");
    await expect(page.locator("h1")).toHaveText("Twenty-Mill Floor");
    await expect(page.getByText("Pointed at by")).toBeVisible();
    // By the relationship's label, not its slug: `constrains` is a key (#551).
    await expect(page.locator(".card", { hasText: "Links" })).toContainText("Constrains");
  });

  test("rewrites corpus file paths into working routes", async ({ page }) => {
    // The transformation most likely to rot: `../legislation/hb-920-1976.yml` has to become a
    // route, and a wrong guess produces an ordinary-looking link that 404s. The paragraph that
    // cites it is the page's lead, since the summary restated it (#551).
    await page.goto("/wiki/parameter/twenty-mill-floor");
    const link = page
      .locator('main :is(.lead, .prose-body) a[href="/wiki/legislation/hb-920-1976"]')
      .first();
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

  test("draws the series a node binds, at the end of the field it names", async ({ page }) => {
    /*
     * #441's seam: the Toledo node binds `dispersion/fy2016-step-by-business-class` under its
     * findings, so the chart is the last thing in the findings card and nowhere else. Four bars
     * because the column has four rows, and the endpoints of the column — +0.1175 on industrial,
     * −0.0321 on mineral — are numbers the same card states in prose, which is the rule that
     * makes a chart checkable at all.
     */
    await page.goto("/wiki/education-agency/toledo-city");
    const card = page.locator("#findings");
    const chart = card.locator('[data-chart="dispersion/fy2016-step-by-business-class"] svg.plot:visible');
    await expect(chart).toBeVisible();
    await expect(chart).toHaveAttribute("role", "img");
    await expect(chart.locator(".bar-fill > *")).toHaveCount(4);
    await expect(card.locator(".series h3")).toContainText("Correlation of the FY2016 move");
    await expect(card).toContainText("+0.1175");
    await expect(card).toContainText("−0.0321");
    expect(await page.locator("[data-chart]").count(), "one chart on the page").toBe(1);
  });

  test("a class page is also the schema for its class", async ({ page }) => {
    await page.goto("/wiki/education-agency");
    await expect(page.getByRole("heading", { name: "Properties" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Relationships" })).toBeVisible();
    await expect(page.locator(".card", { hasText: "Properties" })).toContainText("IRN");
    await expect(page.locator(".card", { hasText: "Relationships" })).toContainText("Party to");
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

test.describe("the wiki with scripts off", () => {
  test.use({ javaScriptEnabled: false });

  test("lists where a node appears on the site, and the link lands there", async ({ page }) => {
    // #551: the corpus links back to the data. The list is in the built page, not hydrated, so it
    // is the same list with scripts off — and its example is an exemplar district the corpus holds.
    await page.goto("/wiki/metric/performance-index");
    const card = page.locator("#on-the-site");
    await expect(card).toBeVisible();
    await expect(card).toContainText("pages like this one");
    const link = card.locator("th a").first();
    await expect(link).toHaveAttribute("href", "/district/043786/outcome");
    await link.click();
    await expect(page.locator("h1")).toContainText("Cleveland Municipal");
  });
});
