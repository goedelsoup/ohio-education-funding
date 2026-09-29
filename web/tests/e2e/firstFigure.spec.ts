/**
 * The answer is within reach on a phone (#549).
 *
 * Loads each class's sample at 375px and holds the first chart, table or figure below the `h1` to
 * the class's ceiling in `FIRST_FIGURE_CEILINGS`. The measurement is `collect`'s own
 * `firstFigureY`, so this reads the same number the measure report prints rather than a second
 * definition of it that could drift.
 *
 * `pnpm measure` walks eight routes and fails nothing on CI; this is the half that bites. It runs
 * under whatever face the runner resolves, which is why the ceilings carry the headroom they
 * document rather than the figure measured on one machine.
 */

import { expect, test } from "@playwright/test";

import {
  BOX_MIN_RADIUS,
  FIRST_FIGURE_CEILINGS,
  PROSE_CELL_MIN_WORDS,
  PROSE_MIN_CHARS,
  collect,
  firstFigureCeiling,
} from "../../src/lib/measure.ts";

for (const kind of FIRST_FIGURE_CEILINGS) {
  test.describe(`the first figure, ${kind.name} class`, () => {
    for (const route of kind.sample) {
      test(`starts within ${kind.ceiling}px on ${route} at 375px`, async ({ page }) => {
        // The sample and the pattern are two lists that could disagree; a route that another class
        // claims first would be held to that class's ceiling everywhere else.
        expect(firstFigureCeiling(route)).toBe(kind.ceiling);

        await page.setViewportSize({ width: 375, height: 800 });
        const response = await page.goto(route);
        expect(response?.ok(), route).toBe(true);
        await page.evaluate(() => document.fonts.ready.then(() => undefined));

        const { firstFigureY, bodyFont } = await page.evaluate(collect, {
          proseMinChars: PROSE_MIN_CHARS,
          proseCellMinWords: PROSE_CELL_MIN_WORDS,
          boxMinRadius: BOX_MIN_RADIUS,
        });
        expect(firstFigureY, `${route} has no figure below its h1`).not.toBeNull();
        expect(
          firstFigureY,
          `${route} puts its first figure at y=${firstFigureY} (body font ${bodyFont}), over the ` +
            `${kind.ceiling}px the ${kind.name} class allows — something was added above the answer`,
        ).toBeLessThanOrEqual(kind.ceiling);
      });
    }
  });
}
