/**
 * What a screen reader is told when something on the page changes.
 *
 * A control that redraws a region silently is a control that works for everyone who can see it.
 * This is the live-region half of that: the announcements, their politeness, and that they are not
 * duplicated by the region redrawing underneath them.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND, NORTHERN } from "./helpers.ts";

/*
 * What a page says when it rewrites itself.
 *
 * `/scenario`, `/compare` and the 609 `/district/*\/scenario` routes replace their whole result
 * block when a lever moves. The site had two live regions before this and both were on
 * `/districts`; everywhere else, a reader who could not see the figures change was not told they
 * had.
 */
test.describe("announcing what changed", () => {
  test("moving a lever says what the result now is", async ({ page }) => {
    await page.goto("/scenario");
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);

    // Nothing is announced for merely arriving: the region carries changes, not the page.
    await expect(page.locator("#changed")).toHaveText("");

    await page.selectOption("#lv-guarantee", "removed");
    const said = page.locator("#changed");
    await expect(said).toContainText("Scenario updated");

    /*
     * And it says the figures, read back off the tiles that were written — so the sentence a
     * reader hears is the figure a reader sees rather than a second description free to drift.
     */
    const tiles = await page.locator("#scenario-out .tile").evaluateAll((nodes) =>
      nodes.map((n) => n.querySelector(".v")?.textContent?.replace(/\s+/g, " ").trim() ?? ""),
    );
    const text = (await said.textContent()) ?? "";
    for (const value of tiles.filter((v) => v !== "")) expect(text).toContain(value);
  });

  test("the announcement waits for the reader to stop moving the slider", async ({ page }) => {
    /*
     * A range input fires `input` on every step of a drag. A polite region written on each of them
     * is either fifty interruptions or a sentence about a position the reader has already left.
     */
    await page.goto("/scenario");
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    const slider = page.locator("#lv-min");
    await slider.focus();
    for (let i = 0; i < 6; i += 1) await page.keyboard.press("ArrowRight");

    // Mid-drag it has said nothing yet.
    expect(await page.locator("#changed").textContent()).toBe("");
    await expect(page.locator("#changed")).toContainText("Scenario updated");
  });

  test("the district scenario announces its own change too", async ({ page }) => {
    // 609 routes, and the one a school board sends. The statewide page having a live region and
    // this one not would be the same oversight in a smaller place.
    await page.goto(`/scenario?d=${CLEVELAND}`);
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    await page.selectOption("#lv-guarantee", "removed");
    await expect(page.locator("#changed")).toContainText("Scenario updated");
  });

  test("/compare says which two districts it is now about", async ({ page }) => {
    // No tiles on this route: the whole result is one table of paired figures, so the sentence
    // worth interrupting someone with is which pair.
    await page.goto(`/compare?a=${CLEVELAND}&b=${NORTHERN}`);
    await expect(page.locator("#compare-out table")).toBeVisible();
    await expect(page.locator("#changed")).toContainText("Comparison updated");
    await expect(page.locator("#changed")).toContainText("Cleveland Municipal");
    await expect(page.locator("#changed")).toContainText("Northern Local");
  });

  test("a lever announces the quantity the page shows, not the formula's argument", async ({
    page,
  }) => {
    /*
     * A range input announces `value`, and these carry multipliers: the base-cost slider runs 0.8
     * to 1.3 while the label beside it reads `+4%`, and the minimum-state-share slider reads
     * `0.05` against `5%`. `aria-valuetext` is what makes the spoken value the shown one.
     */
    await page.goto("/scenario");
    await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    for (const [control, output] of [
      ["#lv-base", "#lv-base-out"],
      ["#lv-min", "#lv-min-out"],
      ["#lv-phase", "#lv-phase-out"],
    ] as const) {
      const shown = (await page.locator(output).textContent())?.trim() ?? "";
      expect(shown, `${output} shows something`).not.toBe("");
      await expect(page.locator(control), `${control} announces what it shows`).toHaveAttribute(
        "aria-valuetext",
        shown,
      );
      // And what it shows is not the raw argument, which is the whole point.
      const raw = await page.locator(control).inputValue();
      expect(shown).not.toBe(raw);
    }
  });

  test("the theme toggle says which theme is on, and that it changed", async ({ page }) => {
    // It announced "Switch colour theme, button" whether the page was light or dark, and
    // activating it announced nothing at all.
    await page.goto("/");
    const toggle = page.locator("#theme");
    await expect(toggle).toHaveAttribute("aria-label", "Dark theme");
    await expect(toggle).toHaveAttribute("aria-pressed", "false");
    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-pressed", "true");
    expect(await page.evaluate(() => document.documentElement.dataset["theme"])).toBe("dark");
    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-pressed", "false");
  });

  test("the basis radios say what is being chosen between", async ({ page }) => {
    /*
     * The group's name was on a `div` holding the two labels and no radios — the inputs have to be
     * siblings of the panels for the no-script sibling selectors to switch them. So the radios
     * announced "Nominal, 1 of 2" with no word about what the choice was over.
     */
    await page.goto(`/district/${CLEVELAND}/finances`);
    const radios = page.locator('.basis-scope input[type="radio"]');
    await expect(radios.first()).toHaveAttribute("aria-label", /Dollar basis/);
    await expect(radios.nth(1)).toHaveAttribute("aria-label", /Dollar basis/);
    // The visible label is still part of the name, so a reader saying it is naming the control.
    await expect(radios.first()).toHaveAttribute("aria-label", /Nominal/);
    await expect(radios.nth(1)).toHaveAttribute("aria-label", /Constant dollars/);
  });
});
