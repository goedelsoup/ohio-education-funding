/**
 * A projected year past FY2027 says that "current law" is not a law there.
 *
 * # What was wrong
 *
 * The forecast card already stated what it holds fixed — *"the levers are held fixed and only
 * enrollment moves"* — and the band already carried the enrollment interval. Both are honest about
 * **enrollment**. Neither said anything about the **law**, and the runner's default horizon is
 * FY2032, five years past the last year the plan's own sections apply to.
 *
 * R.C. 3317.011 (base cost), 3317.017 (local capacity, and the minimum state share inside it) and
 * 3317.0217 (targeted assistance) each open *"This section shall apply only for fiscal years 2026
 * and 2027."* Five further sections hand values back clause by clause — forty divisions reading
 * *"For fiscal year 2028 and each fiscal year thereafter, an amount calculated in a manner
 * determined by the general assembly"*, seventeen of them in R.C. 3317.022 alone.
 *
 * Nothing in the band's construction was wrong. It was the label that was short of the truth.
 *
 * # Both directions, which is the whole point
 *
 * A caveat that renders unconditionally is not a caveat — it is a footer, and a reader learns to
 * skip it. The runner's horizon slider runs FY2026 to FY2036, so FY2027 is a *real projection*
 * that is still inside the statute: the band is drawn, the enrollment is forecast, and there is
 * nothing to warn about. That is the case the second test pins, and it is the one that would break
 * if somebody moved the note out of its conditional.
 *
 * # Where 2027 comes from
 *
 * `project::statute::LAST_STATUTORY_YEAR`, checked in that crate against the committed extract of
 * the Revised Code by counting the sections that carry the expiry clause. It reaches this page
 * through the feed as `projection.statute_ends`. Nobody types the year twice, and an amendment
 * that moves it fails in the crate rather than leaving a caveat here that has quietly become
 * false.
 */

import { readFileSync, readdirSync } from "node:fs";
import { join, relative } from "node:path";

import { expect, test } from "@playwright/test";

const DIST = join(import.meta.dirname, "../../dist");
const CAVEAT = /“Current law” stops at FY(\d{4})/;

function* walk(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* walk(path);
    else if (entry.name.endsWith(".html")) yield path;
  }
}

test("every built page that draws a year past the statute carries the caveat", () => {
  /*
   * The district pages are the static half: each carries a six-year carry-forward to FY2032, so
   * each has to say it. Read from `dist/` rather than a browser because the question is about
   * every page and the answer has to be too.
   */
  const drawn: string[] = [];
  const silent: string[] = [];

  for (const file of walk(DIST)) {
    const page = readFileSync(file, "utf8");
    /*
     * The fan chart's own accessible label names the span it covers, and the two surfaces word it
     * differently — the statewide runner says "at projected enrollment, FY2026 to FY2032" and a
     * district page says "by enrollment year, FY2024 to FY2032". Both are matched, because the
     * question is which pages draw a projected year and not which template drew it.
     */
    const span = /(?:by enrollment year|at projected enrollment), FY(\d{4}) to FY(\d{4})/.exec(page);
    if (!span) continue;
    const end = Number(span[2]);
    if (end <= 2027) continue;
    drawn.push(relative(DIST, file));
    if (!CAVEAT.test(page)) silent.push(relative(DIST, file));
  }

  expect(drawn.length, "pages drawing a projected year past FY2027").toBeGreaterThan(600);
  expect(silent.slice(0, 5), "a projected year past the statute with nothing said about it").toEqual([]);
});

test("the caveat names the year the feed says, not one written here", () => {
  /*
   * Guards the seam rather than the sentence. If `project::statute` moves the year and the feed
   * carries it, this follows; if someone hard-codes 2027 into the page, the two stop agreeing the
   * first time the statute is amended and nothing else would notice.
   */
  const feed = JSON.parse(
    readFileSync(join(import.meta.dirname, "../../public/data/bundle.json"), "utf8"),
  ) as { projection: { statute_ends: number } };

  const page = readFileSync(join(DIST, "district", "043786.html"), "utf8");
  const said = CAVEAT.exec(page);
  expect(said, "the district page carries the caveat at all").not.toBeNull();
  expect(Number(said![1])).toBe(feed.projection.statute_ends);
});

test.describe("the runner's horizon", () => {
  test("says it past the statute, and says nothing inside it", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 1200 });
    await page.goto("/scenario");

    const note = page.locator("#projection").getByText(/“Current law” stops at/);
    const horizon = page.locator("#lv-horizon");

    // The default. Six years past the last observed year, which is FY2032.
    await expect(note).toBeVisible();

    /*
     * FY2027: still a forecast — enrollment is carried a year forward and the band is drawn — and
     * still inside the statute, so there is nothing to caveat. This is the assertion that stops
     * the note becoming unconditional.
     */
    await horizon.fill("2027");
    await horizon.dispatchEvent("input");
    await expect(page.locator("#projection")).toContainText("At projected enrollment");
    await expect(note).toHaveCount(0);

    // And back, so the conditional is shown to move in both directions rather than once.
    await horizon.fill("2032");
    await horizon.dispatchEvent("input");
    await expect(note).toBeVisible();
  });
});
