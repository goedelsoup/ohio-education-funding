/**
 * A defined term's definition, closed and open, at every width the sheet and the panel meet.
 *
 * #601 found three ways a definition went wrong that no test saw, because the sweeps that look for
 * a page panning sideways visited the dashboard and not the three routes that carry terms:
 *
 * - closed, a 30rem panel still counted toward its scroll container's overflow, so it panned the
 *   finances page by up to 243px between 545 and 880px, and on the taxes page made the table
 *   wrapper scroll by 151px at every desktop width;
 * - open inside that wrapper, it was clipped by it;
 * - opened from a tile's key, it inherited the eyebrow's capitals and tracking.
 *
 * The widths straddle both lines the stylesheet draws: 544/545 where the sheet used to stop, and
 * 1023/1024 where it stops now.
 */

import { expect, test } from "@playwright/test";

import { CLEVELAND } from "./helpers.ts";

// Columbus: the largest real property tax charge of any district, so the widest tile figure.
const COLUMBUS = "043802";
const ROUTES = [CLEVELAND, COLUMBUS].flatMap((irn) =>
  ["finances", "taxes", "outcome", "change"].map((page) => `/district/${irn}/${page}`),
);

test.describe("a defined term", () => {
  test("a closed definition widens neither the page nor anything that scrolls", async ({ page }) => {
    test.setTimeout(120_000);
    const complaints: string[] = [];
    for (const width of [320, 375, 545, 600, 680, 768, 820, 900, 1023, 1024, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      for (const route of ROUTES) {
        await page.goto(route);
        const found = await page.evaluate(() => {
          const out: string[] = [];
          const doc = document.documentElement;
          if (doc.scrollWidth > doc.clientWidth + 1) {
            out.push(`the page scrolls sideways by ${doc.scrollWidth - doc.clientWidth}px`);
          }
          /*
           * Every scroll box that holds a definition, measured with the definitions and then
           * without them. A table wrapper may scroll on its own account — a wide table on a phone
           * is what it is for — so the assertion is that the definitions add nothing to it.
           */
          const boxes = [...document.querySelectorAll<HTMLElement>("*")].filter(
            (el) =>
              el.querySelector(".term-def, .year-chip-def") != null &&
              ["auto", "scroll"].includes(getComputedStyle(el).overflowX),
          );
          const before = boxes.map((box) => box.scrollWidth);
          const defs = [...document.querySelectorAll(".term-def, .year-chip-def")];
          const parked = defs.map((def) => [def, def.parentNode, def.nextSibling] as const);
          for (const def of defs) def.remove();
          boxes.forEach((box, i) => {
            if (box.scrollWidth !== before[i]) {
              out.push(`a closed definition widens .${box.className} by ${before[i]! - box.scrollWidth}px`);
            }
          });
          for (const [def, parent, next] of parked) parent!.insertBefore(def, next);
          if (defs.length === 0) out.push("carries no definition to measure");
          return out;
        });
        for (const complaint of found) complaints.push(`${width}px ${route}: ${complaint}`);
      }
    }
    expect(complaints, complaints.join("\n")).toEqual([]);
  });

  test("an open definition is a sentence, wholly on screen, and clipped by nothing", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const complaints: string[] = [];
    for (const width of [375, 680, 1023, 1024, 1280]) {
      await page.setViewportSize({ width, height: 900 });
      for (const route of ROUTES) {
        await page.goto(route);
        const terms = page.locator("button.term");
        const count = await terms.count();
        for (let i = 0; i < count; i += 1) {
          const term = terms.nth(i);
          await term.focus();
          const found = await term.evaluate((button) => {
            const out: string[] = [];
            const def = document.getElementById(button.getAttribute("aria-describedby") ?? "")!;
            const style = getComputedStyle(def);
            const box = def.getBoundingClientRect();
            const doc = document.documentElement;
            if (style.visibility !== "visible" || box.width === 0) out.push("is not open on focus");
            // The tile key is the eyebrow; what opens from it is a paragraph.
            if (style.textTransform !== "none") out.push(`is set in ${style.textTransform}`);
            if (style.letterSpacing !== "normal") out.push(`is tracked ${style.letterSpacing}`);
            if (box.left < -1 || box.right > doc.clientWidth + 1) {
              out.push(`runs from ${Math.round(box.left)} to ${Math.round(box.right)} on a ${doc.clientWidth}px screen`);
            }
            if (doc.scrollWidth > doc.clientWidth + 1) {
              out.push(`pans the page ${doc.scrollWidth - doc.clientWidth}px while open`);
            }
            // A fixed sheet's containing block is the viewport; an absolute panel's is not.
            if (style.position !== "fixed") {
              for (let el = def.parentElement; el != null && el !== doc; el = el.parentElement) {
                if (getComputedStyle(el).overflowX === "visible") continue;
                const clip = el.getBoundingClientRect();
                if (box.left < clip.left - 1 || box.right > clip.right + 1 || box.bottom > clip.bottom + 1) {
                  out.push(`is clipped by .${el.className}`);
                }
              }
            }
            return out.map((complaint) => `"${button.textContent}" ${complaint}`);
          });
          for (const complaint of found) complaints.push(`${width}px ${route}: ${complaint}`);
        }
      }
    }
    expect(complaints, complaints.join("\n")).toEqual([]);
  });
});
