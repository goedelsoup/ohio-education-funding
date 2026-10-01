/**
 * A long page leads back to its contents, and its views say they scroll (#594).
 *
 * `/district/043802` is about 33 screens at 375px with its contents list closed at the top, and
 * nothing past the list led back to it. Every heading with an address now carries "↑ Contents"
 * beside its `#` — `linkContents` in `semantics.ts` — so a reader is never more than a screen of
 * scrolling below one.
 *
 * Touch emulation, because that is the device this is for: under a fine pointer the link hides
 * until its heading is hovered, as the `#` does, and a desktop Chrome profile would be measuring
 * that rather than a phone.
 */

import { expect, test } from "@playwright/test";

const DISTRICT = "/district/043802";

test.use({ viewport: { width: 375, height: 800 }, hasTouch: true, isMobile: true });

test("deep in a district page, a link back to the contents is within a screen, and lands on it", async ({
  page,
}) => {
  await page.goto(DISTRICT);
  await page.evaluate(() => window.scrollTo(0, 10_000));

  // The nearest link at or above the bottom of the window — the one a reader reaches first by
  // scrolling up, or already sees.
  const nearest = await page.evaluate(() => {
    const above = [...document.querySelectorAll<HTMLElement>("a.to-contents")]
      .map((a) => ({ a, top: a.getBoundingClientRect().top }))
      .filter(({ top }) => top <= window.innerHeight)
      .sort((x, y) => y.top - x.top)[0];
    above?.a.setAttribute("data-nearest", "");
    return above ? { top: above.top, opacity: getComputedStyle(above.a).opacity } : null;
  });
  expect(nearest, "a link back above the window").not.toBeNull();
  expect(nearest!.top, "at most a screen above it").toBeGreaterThan(-800);
  expect(nearest!.opacity, "shown on a touch screen").toBe("1");

  const link = page.locator("a.to-contents[data-nearest]");
  await link.scrollIntoViewIfNeeded();
  await link.click();
  await expect(page).toHaveURL(/#contents$/);
  await expect(page.locator("nav#contents .contents-summary")).toBeInViewport();
});

test("the district's view strip shows a cue while its last tab is cut off, and none once it is reached", async ({
  page,
}) => {
  await page.goto(DISTRICT);
  const strip = page.locator(".subnav");

  const state = () =>
    strip.evaluate((el) => {
      const box = el.getBoundingClientRect();
      const last = [...el.querySelectorAll("a")].at(-1)!.getBoundingClientRect();
      const cue = getComputedStyle(el, "::after");
      return {
        overflows: el.scrollWidth > el.clientWidth + 1,
        lastShown: last.right <= box.right + 0.5,
        cue: cue.content !== "none" && cue.position === "sticky" && cue.backgroundImage.includes("gradient"),
      };
    });

  // At 375 the fifth tab does not fit — #594 measured it as a 6px sliver — so this is asked of a
  // strip that has something to say.
  const before = await state();
  expect(before.overflows).toBe(true);
  expect(before.lastShown).toBe(false);
  expect(before.cue).toBe(true);

  await strip.evaluate((el) => (el.scrollLeft = el.scrollWidth));
  const after = await state();
  expect(after.lastShown, "scrolled to the end, the last tab is whole and the cue covers none of it").toBe(true);
});

test("a row far down the district directory names each of its figures", async ({ page }) => {
  await page.goto("/districts");
  // The head cannot stick — the table scrolls sideways in its own box — so each cell names itself.
  const row = page.locator("#district-table tbody tr").nth(400);
  await row.scrollIntoViewIfNeeded();
  const { labels, heads } = await page.evaluate(() => {
    const cells = [...document.querySelectorAll("#district-table tbody tr")[400]!.querySelectorAll("td")];
    return {
      labels: cells.map((td) => getComputedStyle(td, "::before").content),
      heads: [...document.querySelectorAll("#district-table thead th")].slice(1).map((th) => th.textContent ?? ""),
    };
  });
  expect(labels).toHaveLength(6);
  labels.forEach((label, i) => {
    // `content` serialises with its quotes, and the alternative text after a solidus.
    const name = /^"([^"]+)"/.exec(label)?.[1];
    expect(name, `column ${i + 1}`).toBeTruthy();
    expect(heads[i], `column ${i + 1} is headed by its label`).toContain(name!);
  });
});
