/**
 * Finding a district, and the two indexes that help you.
 *
 * Search, the district index with the distribution it is filtering drawn above it, and the
 * comparison page. Together they are every route whose job is to get a reader to a district rather
 * than to tell them something about one.
 *
 * The artefact half of the comparison page — that `compare.html` ships a whole table rather than an
 * empty div for a script to fill, and that the 609 district JSON files stay small — is in
 * `tests/dist/payload.spec.ts`. A dist scan cannot see client-rendered rows, which is the defect
 * that check was written for.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND, NORTHERN } from "./helpers.ts";

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

  test("the homepage's field is a GET to the index, and the index reads it", async ({ page }) => {
    /*
     * #548's find-a-district. The homepage form is plain HTML and the index reads `?q=` on load,
     * so this is two claims: typing an IRN on the front door and pressing Enter makes the address,
     * and the address shows the one district — with the name in the field, where it can be seen
     * and cleared, rather than a filter applied from nowhere.
     */
    await page.goto("/");
    await page.locator("#home-q").fill(CLEVELAND);
    await page.locator("#home-q").press("Enter");
    await expect(page).toHaveURL(new RegExp(`/districts\\?q=${CLEVELAND}$`));
    await expect(page.locator("#f-name")).toHaveValue(CLEVELAND);
    const shown = page.locator("#district-table tbody tr:visible");
    await expect(shown).toHaveCount(1);
    await expect(shown.locator("a")).toHaveAttribute("href", `/district/${CLEVELAND}`);
    await expect(page.locator("#f-count")).toHaveText("1 of 609 districts");

    // And the field is the reader's: clearing it is the whole table again.
    await page.locator("#f-name").fill("");
    await expect(page.locator("#district-table tbody tr:visible")).toHaveCount(609);
  });

  test("sorting by a column reorders the table, and says so where ARIA looks", async ({ page }) => {
    /*
     * The assertion this replaces read `aria-sort` off the *button*, which is where the markup put
     * it and where the property is inert: ARIA defines it on the `columnheader`. So the test was
     * green against the defect it was written for — the table announced no sort order however it
     * was sorted — and correcting the markup was what broke a passing test.
     */
    await page.goto("/districts");
    const cell = page.locator('#district-table thead th:has(button[data-sort="aid"])');
    const button = page.locator('#district-table thead button[data-sort="aid"]');

    // Sortable and not currently sorted is a state of its own, and it was the absence of one.
    await expect(cell).toHaveAttribute("aria-sort", "none");

    await button.click();
    await expect(page.locator("#district-table tbody tr").first()).toHaveAttribute(
      "data-name",
      /.+/,
    );
    await expect(cell).toHaveAttribute("aria-sort", "descending");
    // And nothing is left on the control, where it would be read by nobody.
    await expect(button).not.toHaveAttribute("aria-sort", /.*/);

    // One column is sorted at a time: the name column held `ascending` from the build.
    await expect(
      page.locator('#district-table thead th:has(button[data-sort="name"])'),
    ).toHaveAttribute("aria-sort", "none");

    await button.click();
    await expect(cell, "a second press reverses it").toHaveAttribute("aria-sort", "ascending");
  });

  test("/what-changed sends a reader to the directory sorted on a change column (#693)", async ({ page }) => {
    /*
     * The page's by-county table went because this one has the same columns and sorts them; the
     * link is what keeps "who fell furthest" one click away. Followed, not built, so a key the
     * header does not carry fails here rather than opening an alphabetical table.
     */
    await page.goto("/what-changed");
    await page.locator("main a", { hasText: "state share's change" }).click();
    await expect(page).toHaveURL(/\/districts\?sort=share-1-2&order=ascending$/);
    const cell = page.locator('#district-table thead th:has(button[data-sort="share-1-2"])');
    await expect(cell).toHaveAttribute("aria-sort", "ascending");
    await expect(
      page.locator('#district-table thead th:has(button[data-sort="name"])'),
    ).toHaveAttribute("aria-sort", "none");
    const first = await page
      .locator("#district-table tbody tr")
      .evaluateAll((rows) => rows.slice(0, 3).map((r) => Number((r.lastElementChild as HTMLElement).dataset.sortValue)));
    expect(first[0]).toBeLessThan(0);
    expect(first[0]).toBeLessThanOrEqual(first[1]!);
    expect(first[1]).toBeLessThanOrEqual(first[2]!);
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
     * "No change" for exactly that reason.
     *
     * The other five measure quantities that cannot be negative, and drawing a zero on them would
     * be a reference to a value outside their range. `distributionSpec` decides from the domain,
     * so what is asserted here is that the decision came out one way six times.
     */
    await page.goto("/districts");
    const measures = page.locator("#district-measures");
    await expect(measures.locator('.measure[data-measure="enrollment"]')).toContainText(
      "No change",
    );
    for (const key of ["aid", "valuation", "poverty", "adm", "guarantee"]) {
      await expect(measures.locator(`.measure[data-measure="${key}"]`)).not.toContainText(
        "No change",
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

  test("the strip's high-value label never crowds the filter label below it", async ({ page }) => {
    /*
     * `.measures` and the `.filters` form that follows it had no margin between them anywhere in
     * the chain, so the visible strip's high-value label (e.g. "$896") sat directly against "Name
     * or IRN" at every width. Not font-sensitive: the gap was zero regardless of how either label
     * is set. The label is now the strip's own foot, drawn inside the SVG.
     */
    for (const width of [375, 768, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      await page.goto("/districts");
      const scaleHigh = page.locator(".measure:visible svg.plot:visible g.axis-foot text").last();
      const nameLabel = page.locator('label[for="f-name"]');
      const [scaleBox, labelBox] = await Promise.all([
        scaleHigh.boundingBox(),
        nameLabel.boundingBox(),
      ]);
      expect(scaleBox, `${width}px: strip label box`).not.toBeNull();
      expect(labelBox, `${width}px: filter label box`).not.toBeNull();
      expect(
        labelBox!.y,
        `${width}px: "${await scaleHigh.textContent()}" (bottom ${scaleBox!.y + scaleBox!.height}) vs "Name or IRN" (top ${labelBox!.y})`,
      ).toBeGreaterThan(scaleBox!.y + scaleBox!.height);
    }
  });
});

/**
 * `/compare` ships its comparison, and fetches a pair rather than a panel.
 *
 * # The measurement this holds
 *
 * The route used to download the whole 609-district panel — 641,042 B, 127,961 gzipped — to render
 * seventeen rows about two districts, and shipped an empty `#compare-out` until it landed. At
 * 400 Kbps / 400 ms RTT: first paint 1,284 ms, panel finishing **4,131 ms**, table 4,340 ms. Three
 * seconds of an empty box. That was the last open half of #111.
 *
 * Measured again after: a bare `/compare` has its table in the HTML and fetches **nothing**, and
 * `?a=&b=` fetched two files totalling **1,181 B**, finishing at 1,678 ms against a first paint of
 * 1,264 ms.
 *
 * Those two files are **4,006 B** today, against a panel that is now 1,237,912 B. Both figures
 * move with the feed — a column added to a district is added 609 times — so both are dated here
 * rather than maintained. What the assertions below hold is the shape, which does not move.
 *
 * # Why none of these assertions is a stopwatch
 *
 * A wall-clock threshold in CI is a claim about the runner. What actually changed is *what the
 * page does*, and that is exact: the table is either in the document or it is not, and the request
 * list is either two district files or it is a panel. Both survive a slow machine, and neither can
 * be satisfied by the old shape.
 *
 * The first of those two is now asked of the artefact, in `tests/dist/payload.spec.ts`, along with
 * the size of the 609 district files the second one fetches from. What is left here is the request
 * list, which only a browser can report.
 */
test.describe("the comparison arrives with the page", () => {
  test("a bare /compare fetches no data at all", async ({ page }) => {
    const asked: string[] = [];
    page.on("request", (request) => {
      if (request.url().includes("/data/")) asked.push(request.url().split("/data/")[1]!);
    });
    await page.goto("/compare");
    await expect(page.locator("#comparison")).toBeVisible();
    // Settle: a fetch fired late would still be a fetch.
    await expect(page.locator("#comparison th[data-head='a']")).toContainText("Northern Local");
    expect(
      asked,
      "the document it was served is already the comparison it is about, so there is nothing to ask for",
    ).toEqual([]);
  });

  test("a pair in the query costs two districts, and a swap costs one more", async ({ page }) => {
    const asked: string[] = [];
    page.on("request", (request) => {
      if (request.url().includes("/data/")) asked.push(request.url().split("/data/")[1]!);
    });
    await page.goto(`/compare?a=${CLEVELAND}&b=043802`);
    await expect(page.locator("#comparison")).toHaveAttribute("data-a", CLEVELAND);
    expect(asked.sort(), "two districts, and never the panel").toEqual([
      `district/${CLEVELAND}.json`,
      "district/043802.json",
    ]);

    // The other half of the pair is already held, so only the newcomer is fetched.
    await page.selectOption("#cmp-a", NORTHERN);
    await expect(page.locator("#comparison")).toHaveAttribute("data-a", NORTHERN);
    expect(asked.length, "one more request, not two").toBe(3);
    expect(asked[2]).toBe(`district/${NORTHERN}.json`);

    // And back: nothing is fetched twice.
    await page.selectOption("#cmp-a", CLEVELAND);
    await expect(page.locator("#comparison")).toHaveAttribute("data-a", CLEVELAND);
    expect(asked.length, "a district looked at twice is fetched once").toBe(3);
  });
});
