/**
 * Both faces, under every fallback the stack can resolve to.
 *
 * A layout measured against one font is a layout measured against one machine. These two blocks
 * walk the display and body stacks entry by entry, because `system-ui` is not a fixed metric and a
 * runner that resolves it differently from this laptop is the case that breaks.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, test } from "@playwright/test";

/*
 * The display face is a platform stack, so it is six different faces in the wild.
 *
 * `--font-serif` names Iowan Old Style, Palatino, Charter and Georgia and falls through to
 * whatever `serif` resolves to. That variance was chosen deliberately over committing a font
 * binary — see `tokens/typography.css` — and the way a chosen risk stays affordable is that
 * something checks it.
 *
 * This repository has paid for the alternative once already: `axisFoot` laid two rows touching
 * under DejaVu Sans and a pixel apart under SF Pro, and the defect was invisible on the machine it
 * was written on. The lesson there was to re-measure under other fonts rather than to reason about
 * one. A heading is far more forgiving than a chart axis — it reflows, and it is not laid out
 * against pixel constants — but "more forgiving" is not "cannot break", and the failure mode here
 * is a title running off the side of a phone or sitting on the line beneath it.
 *
 * Times New Roman and DejaVu Serif are in the list precisely because they are NOT named by the
 * token: they are what a Linux or Android reader with none of the four gets, and they are the
 * widest and the narrowest of the set.
 */
test.describe("the display face, under every fallback it can resolve to", () => {
  const FACES = ["Iowan Old Style", "Palatino", "Charter", "Georgia", "Times New Roman", "DejaVu Serif"];
  const ROUTES = [
    "/district/043786",
    "/wiki/funding-regime/fair-school-funding-plan",
    "/wiki/decision/the-four-kinds-of-parameter",
  ];

  for (const face of FACES) {
    test(`neither overflows nor collides in ${face}`, async ({ page }) => {
      await page.addInitScript((family: string) => {
        addEventListener("DOMContentLoaded", () => {
          const style = document.createElement("style");
          /* Only the two elements that take the serif. Forcing it wider would test a site that
             does not exist — body and every figure stay in the sans stack by design. */
          style.textContent = `h1, .lead { font-family: "${family}", serif !important }`;
          document.head.append(style);
        });
      }, face);

      const complaints: string[] = [];
      for (const width of [375, 768, 1280]) {
        await page.setViewportSize({ width, height: 1000 });
        for (const route of ROUTES) {
          await page.goto(route);
          const found = await page.evaluate(() => {
            const out: string[] = [];
            const doc = document.documentElement;
            if (doc.scrollWidth > doc.clientWidth + 1) {
              out.push(`the page scrolls sideways by ${doc.scrollWidth - doc.clientWidth}px`);
            }
            for (const selector of ["h1", ".lead"]) {
              const el = document.querySelector(selector);
              if (el == null) continue;
              const box = el.getBoundingClientRect();
              if (box.right > doc.clientWidth + 1) {
                out.push(`${selector} runs ${Math.round(box.right - doc.clientWidth)}px off the right`);
              }
              const next = el.nextElementSibling;
              if (next != null && next.getBoundingClientRect().top < box.bottom - 1) {
                out.push(`${selector} sits on what follows it`);
              }
            }
            return out;
          });
          for (const complaint of found) complaints.push(`${width}px ${route}: ${complaint}`);
        }
      }
      expect(complaints, complaints.join("\n")).toEqual([]);
    });
  }
});

/*
 * The body face, under every fallback the sans stack can resolve to.
 *
 * The sibling sweep above covers `--font-serif`, which is a platform stack by choice. `--font-sans`
 * is one too — "IBM Plex Sans", ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto — and
 * it was not swept, which is a gap #182's exit gate names in as many words: *zero text overlap
 * under `system-ui`, DejaVu Sans, Liberation Sans, Arial and Verdana*. Those are the five here.
 *
 * # Why the charts and not the prose
 *
 * A paragraph reflows. A chart does not: `plot/spec.ts` lays its labels out against pixel
 * constants — margins, tick counts, the `axisFoot` row — computed from one font's metrics, and a
 * wider face pushes two labels through each other with nothing to give. This repository has
 * already paid for that once, in the defect the sweep above cites: `axisFoot` laid two rows
 * touching under DejaVu Sans and a pixel apart under SF Pro, and it was invisible on the machine
 * it was written on.
 *
 * 375px because that is where a chart has least room, which is where a wider face lands first.
 *
 * # What a missing font does here, and why it is still worth running
 *
 * Forcing a family the machine does not carry falls back rather than failing, so a runner without
 * DejaVu measures its fallback under DejaVu's name. That makes this a sweep over *whatever faces
 * the runner has*, not a proof about five specific ones — and it is still the check that would
 * have caught the `axisFoot` defect, because the failure needs only a face with different metrics
 * from the author's, not a particular one.
 */
test.describe("the body face, under every fallback it can resolve to", () => {
  const FACES = ["system-ui", "DejaVu Sans", "Liberation Sans", "Arial", "Verdana"];
  /* The three densest charted routes: a bar row, a scatter, and the range strip. */
  const ROUTES = ["/statewide", "/outcomes", "/counties"];

  for (const face of FACES) {
    test(`draws no chart label through another in ${face}`, async ({ page }) => {
      await page.addInitScript((family: string) => {
        addEventListener("DOMContentLoaded", () => {
          const style = document.createElement("style");
          /*
           * `system-ui` is a generic keyword and the other four are family names. Quoting the
           * keyword turns it into a lookup for a family called "system-ui", which no machine has,
           * so the run silently fell through to `sans-serif` and measured the same face twice.
           */
          const named = family === "system-ui" ? family : `"${family}"`;
          // The sans stack and the charts that inherit from it. Headings keep the serif, which the
          // sibling sweep owns, and figures keep the mono.
          style.textContent =
            `:root { --font-sans: ${named}, sans-serif !important }` +
            `svg.plot, svg.plot text, svg.plot tspan { font-family: ${named}, sans-serif !important }`;
          document.head.append(style);
        });
      }, face);

      await page.setViewportSize({ width: 375, height: 900 });
      const through: string[] = [];
      for (const route of ROUTES) {
        await page.goto(route);
        const hits = await page.evaluate(() => {
          const out: string[] = [];
          const doc = document.documentElement;
          if (doc.scrollWidth > doc.clientWidth + 1) {
            out.push(`the page scrolls sideways by ${doc.scrollWidth - doc.clientWidth}px`);
          }
          for (const svg of document.querySelectorAll("svg.plot")) {
            if (!svg.getClientRects().length) continue;
            const labels = [...svg.querySelectorAll("text")]
              .filter((label) => (label.textContent ?? "").trim())
              .map((label) => ({ text: (label.textContent ?? "").trim(), box: label.getBoundingClientRect() }))
              .filter((label) => label.box.width > 0);
            for (let a = 0; a < labels.length; a += 1) {
              for (let b = a + 1; b < labels.length; b += 1) {
                const [A, B] = [labels[a]!.box, labels[b]!.box];
                const overlaps =
                  Math.min(A.right, B.right) - Math.max(A.left, B.left) > 0 &&
                  Math.min(A.bottom, B.bottom) - Math.max(A.top, B.top) > 0;
                if (overlaps) out.push(`"${labels[a]!.text}" through "${labels[b]!.text}"`);
              }
            }
          }
          return out;
        });
        for (const hit of hits) through.push(`${route}: ${hit}`);
      }
      // Zero tolerance, for the reason the sibling sweep records: a pixel of slack made the last
      // defect of this kind a property of the reviewer's machine.
      expect(through.slice(0, 10), through.join("\n")).toEqual([]);
    });
  }
});
