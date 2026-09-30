/**
 * A table too wide for a phone says so (#575).
 *
 * At 375px, 15 tables on 8 routes scrolled sideways inside their `.scroll` box with nothing to show
 * it — `/statewide`'s funding units table was 540px in 295px with its Amount column wholly off
 * screen. The cue is `.scroll`'s background in `app.css`: shades pinned to the box's edges, and
 * covers the colour of the ground riding with the content to hide a shade once its edge is reached.
 *
 * What only a browser can check is what the covers depend on. They are painted in
 * `--scroll-ground`, so on any other ground they show as a band; and they are behind the cells, so
 * a cell that paints its own background hides the shade. Both are asked of every box that actually
 * overflows at this width, on the routes #575 measured.
 */

import { expect, test } from "@playwright/test";

const ROUTES = ["/statewide", "/county/ottawa", "/house/090", "/history", "/district/043786"];

for (const route of ROUTES) {
  test(`every table that overflows ${route} at 375px shows the cue`, async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 900 });
    await page.goto(route);

    const report = await page.evaluate(() => {
      const opaque = (colour: string) => colour !== "rgba(0, 0, 0, 0)" && colour !== "transparent";
      const resolve = (value: string) => {
        const probe = document.createElement("div");
        probe.style.backgroundColor = value;
        document.body.append(probe);
        const out = getComputedStyle(probe).backgroundColor;
        probe.remove();
        return out;
      };

      const boxes = [...document.querySelectorAll<HTMLElement>(".scroll")].filter(
        (box) => box.scrollWidth > box.clientWidth + 1,
      );
      const faults: string[] = [];
      for (const box of boxes) {
        const style = getComputedStyle(box);
        const name = (box.querySelector("th")?.textContent ?? "").trim().slice(0, 30);
        if (!style.backgroundAttachment.includes("local")) faults.push(`${name}: no cue`);

        let ground: Element | null = box.parentElement;
        while (ground && !opaque(getComputedStyle(ground).backgroundColor)) ground = ground.parentElement;
        const want = resolve(style.getPropertyValue("--scroll-ground").trim());
        const got = ground ? getComputedStyle(ground).backgroundColor : "none";
        if (want !== got) faults.push(`${name}: covers ${want} on a ${got} ground`);

        for (const cell of box.querySelectorAll("th, td, tr")) {
          if (opaque(getComputedStyle(cell).backgroundColor)) {
            faults.push(`${name}: a cell paints over the shade`);
            break;
          }
        }
      }
      return {
        overflowing: boxes.length,
        faults,
        pageWidth: document.documentElement.scrollWidth,
      };
    });

    // Each of these routes was measured overflowing in #575; a pass over no box asks nothing.
    expect(report.overflowing, "a table here overflows at 375px").toBeGreaterThan(0);
    expect(report.faults).toEqual([]);
    // The minimum on a row's name widens a table inside its box, never the page.
    expect(report.pageWidth).toBeLessThanOrEqual(375);
  });
}

test("the funding units table overflows at 375px, so the cue has something to say", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 900 });
  await page.goto("/statewide");
  const box = page.locator("#funding-units .scroll");
  const { scrollWidth, clientWidth } = await box.evaluate((el) => ({
    scrollWidth: el.scrollWidth,
    clientWidth: el.clientWidth,
  }));
  expect(scrollWidth).toBeGreaterThan(clientWidth);
});
