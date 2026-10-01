/**
 * What the per-route e2e files share: six districts, a heading reader, and a lever.
 *
 * # Why these files are per-route
 *
 * Until #502 this directory held one `app.spec.ts` of 7,448 lines carrying 341 of the suite's 350
 * tests. Playwright shards by *file*, so `fullyParallel: true` bought almost nothing: one worker
 * ran that file start to finish while the others took nine tests between them. The split is along
 * the seams the file already had — its `test.describe` blocks — grouped by the route or the concern
 * each block is about. No test body changed in the move.
 *
 * Twenty-three tests left the directory altogether in the same change. They read `dist/` with
 * `readFileSync` and never opened a browser, and they now live in `tests/dist/` as a Vitest
 * project. See the header of `tests/dist/artefact.ts` for why an assertion about the artefact is
 * not an assertion about the browser, and for the ordering that keeps it honest.
 *
 * # Why chart selectors say `svg.plot:visible`
 *
 * Every chart is in the document twice — one drawing per width, of which the stylesheet shows one.
 * See `WIDTHS` in `src/lib/plot/spec.ts` for why. So `[data-chart="fan"] svg` matches three elements
 * and every assertion written against it is a strict-mode violation rather than a failure that
 * says anything.
 *
 * `:visible` resolves it to the drawing this viewport is actually being shown, which is also the
 * one the assertion means: what a reader sees. Playwright's default viewport is 1280 wide, so that
 * is the wide drawing everywhere except the phone-width tests, which set their own viewport and
 * get the narrow one from the same selector without saying so.
 */

import { expect, type Locator, type Page } from "@playwright/test";

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
export const headingText = (heading: Locator): Promise<string> =>
  heading.evaluate((node) => {
    const clone = node.cloneNode(true) as HTMLElement;
    clone.querySelector("a.section-anchor")?.remove();
    // The chip carries its reckoning in a panel beside it now, so a heading's `textContent`
    // includes that sentence unless the whole wrapper goes. See `yearChip` in `src/lib/year.ts`.
    clone.querySelector(".year-chip-wrap")?.remove();
    return (clone.textContent ?? "").replace(/\s+/g, " ").trim();
  });

/** Cleveland Municipal. On the guarantee, so the guarantee copy has something to render. */
export const CLEVELAND = "043786";
/** Northern Local (Perry County). The corpus's property-poor exemplar. */
export const NORTHERN = "049056";
/** Manchester Local — funded by the formula, so its band opens rather than collapsing. */
export const ON_FORMULA = "000442";
/** Fremont City — 20.0000 mills in both tax years, so the floor holds still under it. */
export const AT_FLOOR = "044016";
/** Vinton County Local — 18.70 mills voted and charged. The floor never reached it. */
export const BELOW_FLOOR = "050393";
/** Alexander Local — 29.0% federal, the most exposed district in the state. */
export const MOST_FEDERAL = "045906";

/**
 * Move a lever and wait for the scenario to re-render behind it.
 *
 * The wait is the point. `src/scripts/scenario-load.ts` boots the runner once
 * `fetch(data/panel.json)` lands, and until then `#scenario-out` is empty and no `input` listener
 * exists — so a control touched before then is an `<input>` and not a lever. Selecting and then asserting the output is non-empty is
 * what makes the next assertion in a test about the scenario rather than about the fetch. See the
 * note on `live` in `drafts.spec.ts` for the sharper form of the same hazard.
 */
export async function setGuarantee(page: Page, value: string): Promise<void> {
  await page.selectOption("#lv-guarantee", value);
  await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
}
