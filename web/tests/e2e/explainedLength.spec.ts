/**
 * An Explained topic answers before the fold and ends within its length (#712, #715).
 *
 * Loads each sampled topic at 375×800. The short answer — the `.lead` under the `h1` — must end
 * inside the first screen, and the whole page must be no longer than `PAGE_HEIGHT_CEILINGS`
 * allows. The length is `collect`'s own `pageHeight`, so this reads the number the measure report
 * prints rather than a second definition of it.
 */

import { expect, test } from "@playwright/test";

import {
  BOX_MIN_RADIUS,
  PAGE_HEIGHT_CEILINGS,
  PROSE_CELL_MIN_WORDS,
  PROSE_MIN_CHARS,
  collect,
  pageHeightCeiling,
} from "../../src/lib/measure.ts";
import { SECTIONS } from "../../src/lib/routes.ts";

const FOLD = 800;

for (const kind of PAGE_HEIGHT_CEILINGS) {
  test.describe(`the ${kind.name} class at 375px`, () => {
    for (const route of kind.sample) {
      test(`${route} answers above the fold and ends within ${kind.ceiling}px`, async ({ page }) => {
        expect(pageHeightCeiling(route)).toBe(kind.ceiling);

        await page.setViewportSize({ width: 375, height: FOLD });
        const response = await page.goto(route);
        expect(response?.ok(), route).toBe(true);
        await page.evaluate(() => document.fonts.ready.then(() => undefined));

        const answer = page.locator(`#${SECTIONS.explained.shortAnswer}`);
        const box = await answer.boundingBox();
        expect(box, `${route} has no short answer`).not.toBeNull();
        expect(
          box!.y + box!.height,
          `${route}'s short answer ends at y=${Math.round(box!.y + box!.height)}, below the ${FOLD}px fold`,
        ).toBeLessThanOrEqual(FOLD);

        const { pageHeight, bodyFont } = await page.evaluate(collect, {
          proseMinChars: PROSE_MIN_CHARS,
          proseCellMinWords: PROSE_CELL_MIN_WORDS,
          boxMinRadius: BOX_MIN_RADIUS,
        });
        expect(
          pageHeight,
          `${route} is ${pageHeight}px long (body font ${bodyFont}), over the ${kind.ceiling}px ` +
            `the ${kind.name} class allows — cut the topic rather than raise the class`,
        ).toBeLessThanOrEqual(kind.ceiling);
      });
    }
  });
}
