/**
 * axe-core, over a sample of the routes.
 *
 * Its own file because it is the only thing here that loads an accessibility engine into the page,
 * and because it is the slowest single block in the suite: every assertion is a full rule pass over
 * a rendered document. On one worker it delayed everything behind it.
 */

import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

import { CLEVELAND, NORTHERN } from "./helpers.ts";

/*
 * The check that would have caught the four issues this suite has just grown assertions for.
 *
 * A 245-test Playwright suite with no automated accessibility assertion in it is how 13,164
 * unscoped tables, 12,449 mouse-only chips and 7,605 unnamed charts all shipped green. `axe` is
 * not a substitute for the specific assertions elsewhere in this directory and in `tests/dist/` —
 * it cannot know that a chip's reckoning matters or that a chart is drawn twice — but it holds the whole floor those sit on, and it holds
 * it on routes nobody thought to write a test for.
 *
 * One representative page per route family rather than all 3,487: the families differ in what they
 * render, and pages within a family differ only in their figures.
 */
test.describe("axe", () => {
  const FAMILIES = [
    ["the front door", "/"],
    ["the statewide panel", "/statewide"],
    ["the district index, which is the one table with controls over it", "/districts"],
    ["a district dashboard", `/district/${CLEVELAND}`],
    ["a district's finances, which is where the basis switch is", `/district/${CLEVELAND}/finances`],
    ["a district's outcomes", `/district/${CLEVELAND}/outcome`],
    ["a district's taxes", `/district/${CLEVELAND}/taxes`],
    ["a district's change, which has the measure switch", `/district/${CLEVELAND}/change`],
    ["the scenario runner, which rewrites itself", "/scenario"],
    ["the reach view, which is the runner asking who rather than how much", "/scenario/reach"],
    /*
     * The mode control is `hidden` until there is a selection, and axe does not scan what is
     * hidden — so a bare `/reach` leaves the one fieldset on the site unexamined. This is the same
     * route with a selection large enough to draw on its own, which shows the control, the subset
     * card and the in-prose button that switches back.
     */
    ["the reach view subsetted to a county, where the mode control is", "/scenario/reach?co=cuyahoga&only=1"],
    /* And the refusal, which is a card with no chart in it and its own way out. */
    ["the reach view refusing a selection too small to draw", "/scenario/reach?co=athens&only=1"],
    ["the comparison", `/compare?a=${CLEVELAND}&b=${NORTHERN}`],
    ["a county", "/county/cuyahoga"],
    ["the counties index", "/counties"],
    ["outcomes", "/outcomes"],
    ["the method note", "/method"],
    ["history", "/history"],
    ["legislation", "/legislation"],
    ["a wiki node, which is where the corpus's own prose renders", "/wiki/doctrine/equity"],
    ["a decision record", "/wiki/decision/the-order-was-never-the-states"],
    ["a legislative seat", "/house/1"],
    ["the data page", "/data"],
    ["the 404", "/404"],
  ] as const;

  for (const [what, route] of FAMILIES) {
    test(`${what} has no violation axe can name`, async ({ page }) => {
      await page.goto(route);
      // The scenario and comparison routes compute in the browser; scanning before they have
      // rendered would be scanning an empty container and calling it clean.
      if (route.startsWith("/scenario") || route.startsWith("/scenario/reach")) {
        await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
      }
      if (route.startsWith("/compare")) await expect(page.locator("#compare-out table")).toBeVisible();

      const { violations } = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
        .analyze();

      expect(
        violations.map((v) => `${v.id} (${v.nodes.length}): ${v.help}`),
        `${route}`,
      ).toEqual([]);
    });
  }
});
