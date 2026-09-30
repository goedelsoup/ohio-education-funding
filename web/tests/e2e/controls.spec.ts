/**
 * A form control is one size wherever it appears (#576).
 *
 * The homepage's Find button was a 28px pill beside a 35px field, `/districts` put a 32px select
 * beside a 35px input, and the runners drew selects at 32, 37 and 40px — each select took its text
 * size from whatever held it. The box is now three tokens, `--control-*` in `tokens/space.css`, and
 * what only a browser can check is that every control resolves to them.
 *
 * Three questions, at a phone's width and a desktop's: do the controls in one form row share a
 * height, is every select the token's height, and does each select's chosen option fit inside it —
 * a lever option was clipped mid-glyph at 258px on `/scenario`.
 */

import { expect, test } from "@playwright/test";

const ROUTES = ["/", "/districts", "/compare", "/scenario", "/scenario/reach"];
// The flex rows a control shares with its neighbours.
const ROWS = ".filters, .reach-axes, .scope-picks";
const CONTROLS = 'select, input[type="text"], input[type="search"], button[type="submit"], summary';

for (const width of [375, 1280]) {
  for (const route of ROUTES) {
    test(`every control on ${route} at ${width}px is the one control size, and its option fits`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height: 900 });
      await page.goto(route);

      const report = await page.evaluate(
        ({ rows, controls }) => {
          const visible = (el: Element) => (el as HTMLElement).offsetParent !== null;
          const probe = document.createElement("div");
          probe.style.height = "var(--control-height)";
          document.body.append(probe);
          const token = probe.getBoundingClientRect().height;
          probe.remove();

          const faults: string[] = [];
          const name = (el: Element) => el.id || el.textContent?.trim().slice(0, 20) || el.tagName;

          let rowCount = 0;
          for (const row of document.querySelectorAll(`main :is(${rows})`)) {
            const members = [...row.querySelectorAll(`:scope > * :is(${controls}), :scope > :is(${controls})`)]
              .filter(visible)
              // A summary belongs to the row only as the head of a disclosure that sits in it.
              .filter((el) => el.tagName !== "SUMMARY" || el.parentElement?.parentElement === row);
            if (members.length === 0) continue;
            rowCount++;
            for (const el of members) {
              const h = el.getBoundingClientRect().height;
              if (Math.abs(h - token) > 1) faults.push(`${name(el)} in a row is ${h}px, not ${token}px`);
            }
          }

          const canvas = document.createElement("canvas").getContext("2d")!;
          const selects = [...document.querySelectorAll<HTMLSelectElement>("main select")].filter(visible);
          for (const select of selects) {
            const style = getComputedStyle(select);
            const h = select.getBoundingClientRect().height;
            if (Math.abs(h - token) > 1) faults.push(`select ${name(select)} is ${h}px, not ${token}px`);

            canvas.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
            const text = select.selectedOptions[0]?.textContent?.trim() ?? "";
            const room =
              select.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
            const drawn = canvas.measureText(text).width;
            if (drawn > room) faults.push(`select ${name(select)}: "${text}" is ${drawn.toFixed(0)}px in ${room}px`);
          }

          return { token, rows: rowCount, selects: selects.length, faults };
        },
        { rows: ROWS, controls: CONTROLS },
      );

      // 36px is `--control-height`; a probe that resolved nothing would pass every comparison at 0.
      expect(report.token).toBeGreaterThan(30);
      // Every one of these routes has a form row and a select or a field in it.
      expect(report.rows).toBeGreaterThan(0);
      expect(report.faults).toEqual([]);
    });
  }
}

test("every lever option fits its select, not only the one chosen at load", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/scenario");

  const faults = await page.evaluate(() => {
    const canvas = document.createElement("canvas").getContext("2d")!;
    const out: string[] = [];
    for (const select of document.querySelectorAll<HTMLSelectElement>("#levers select")) {
      const style = getComputedStyle(select);
      canvas.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
      const room = select.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
      for (const option of select.options) {
        const drawn = canvas.measureText(option.text.trim()).width;
        if (drawn > room) out.push(`${select.id}: "${option.text}" is ${drawn.toFixed(0)}px in ${room}px`);
      }
    }
    return out;
  });
  expect(faults).toEqual([]);
});

test("the homepage's Find button is a button, not a pill", async ({ page }) => {
  await page.goto("/");
  const button = page.locator('main form[action="/districts"] button[type="submit"]');
  await expect(button).not.toHaveClass(/pill/);
  const field = page.locator("#home-q");
  const [b, f] = await Promise.all([button.boundingBox(), field.boundingBox()]);
  // The same box, on the same line: top and bottom both, where the pill shared only a bottom.
  expect(Math.abs(b!.y - f!.y)).toBeLessThanOrEqual(1);
  expect(Math.abs(b!.height - f!.height)).toBeLessThanOrEqual(1);
});
