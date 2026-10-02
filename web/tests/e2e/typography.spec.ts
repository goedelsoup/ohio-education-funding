/**
 * Both faces, under every fallback the stack can resolve to.
 *
 * A layout measured against one font is a layout measured against one machine. These two blocks
 * walk the display and body stacks entry by entry, because `system-ui` is not a fixed metric and a
 * runner that resolves it differently from this laptop is the case that breaks.
 *
 * Chart selectors here say `svg.plot:visible`; the reason is in `helpers.ts`.
 */

import { expect, type Page, test } from "@playwright/test";

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
    // The taxes page because its tiles hold a nine-digit charge, which ran off a phone (#601).
    "/district/043786/taxes",
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

/*
 * The type system, read off the rendered page (#573).
 *
 * The two sweeps above force a face and ask whether the layout survives it. This one forces
 * nothing and asks what the site actually sets: every visible text node's computed family and
 * size, on one page per route family.
 *
 * It exists because five slips each looked too small to matter: `code` in Courier because it
 * named no family, the `/scenario/reach` presets in Arial because a `<button>` does not inherit
 * its font, figures in mono beside sans neighbours, a claim mark in three faces and three sizes,
 * and eighteen distinct sizes against an eight-step ramp — every off-ramp one an `em` rule taking
 * a proportion of whatever it landed in. None fails a stylesheet read, because each is a
 * cascade outcome rather than a declaration.
 *
 * # What counts as a family
 *
 * The three text tokens, `--font-math` for MathML, and `body`'s own stack. That last is
 * `--font-sans` without IBM Plex Sans in front, which is a decision recorded at `body` in
 * `app.css`, and the first test pins that relationship rather than assuming it. Text inside an
 * `<svg>` is excluded: chart text is `system-ui` by design (`plot/spec.ts`).
 *
 * # What counts as a size
 *
 * The eight ramp tokens and `body`'s 15px apparatus size, which the same comment at `body`
 * records as a decision. Nothing else. At 1280px only: `h1.slug-title` clamps between 2rem and
 * 2.4rem, so a narrower viewport draws a title between two steps on purpose.
 */
test.describe("the type system, as the page sets it", () => {
  const ROUTES = [
    "/",
    "/statewide",
    "/districts",
    "/district/043786",
    "/district/043786/finances",
    "/district/043786/outcome",
    "/district/043786/taxes",
    "/district/043786/change",
    "/scenario",
    "/scenario/reach",
    "/compare?a=043786&b=049056",
    "/county/cuyahoga",
    "/outcomes",
    "/method",
    "/history",
    "/legislation",
    "/wiki",
    "/wiki/doctrine/equity",
    "/wiki/funding-regime/fair-school-funding-plan",
    "/wiki/decision",
    "/wiki/decision/the-order-was-never-the-states",
    "/data",
  ] as const;
  const RAMP = ["xs", "sm", "base", "body", "lead", "h2", "h1", "figure"] as const;
  /** `body`'s apparatus size: a stated decision, not a leftover. See `body` in `app.css`. */
  const ALLOWANCE = 1;

  interface Census {
    tokens: { sans: string; mono: string; serif: string; math: string; body: string };
    ramp: string[];
    bodySize: string;
    figValue: string;
    families: Record<string, string>;
    sizes: Record<string, string>;
    claims: string[];
    codes: string[];
    /** A figure in a row of facts or a table cell, set in a face its cell is not. */
    oddFigures: string[];
  }

  const read = async (page: Page, route: string): Promise<Census> => {
    await page.goto(route);
    // The runner and the comparison render in the browser; reading before they have is reading
    // an empty container.
    if (route.startsWith("/scenario")) {
      await expect(page.locator("#scenario-out .tile, #scenario-out .card")).not.toHaveCount(0);
    }
    if (route.startsWith("/compare")) await expect(page.locator("#compare-out table")).toBeVisible();
    return page.evaluate((ramp) => {
      const probe = (style: string, className = ""): { fontFamily: string; fontSize: string } => {
        const el = document.createElement("span");
        el.setAttribute("style", style);
        el.className = className;
        document.body.append(el);
        const { fontFamily, fontSize } = getComputedStyle(el);
        el.remove();
        return { fontFamily, fontSize };
      };
      const tokens = {
        sans: probe("font-family: var(--font-sans)").fontFamily,
        mono: probe("font-family: var(--font-mono)").fontFamily,
        serif: probe("font-family: var(--font-serif)").fontFamily,
        math: probe("font-family: var(--font-math)").fontFamily,
        body: getComputedStyle(document.body).fontFamily,
      };
      const where = (el: Element): string =>
        el.tagName.toLowerCase() + [...el.classList].map((c) => `.${c}`).join("");

      const families: Record<string, string> = {};
      const sizes: Record<string, string> = {};
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
      for (let node = walker.nextNode(); node != null; node = walker.nextNode()) {
        const el = node.parentElement;
        if (el == null || !(node.textContent ?? "").trim() || el.closest("svg")) continue;
        if (!el.getClientRects().length) continue;
        const style = getComputedStyle(el);
        if (style.visibility === "hidden") continue;
        families[style.fontFamily] ??= where(el);
        sizes[style.fontSize] ??= where(el);
      }
      const visible = (selector: string): CSSStyleDeclaration[] =>
        [...document.querySelectorAll(selector)]
          .filter((el) => el.getClientRects().length)
          .map((el) => getComputedStyle(el));
      return {
        tokens,
        ramp: ramp.map((step) => probe(`font-size: var(--text-${step})`).fontSize),
        bodySize: getComputedStyle(document.body).fontSize,
        figValue: probe("", "fig-value").fontFamily,
        families,
        sizes,
        claims: visible(".claim").map((s) => `${s.fontFamily} @ ${s.fontSize}`),
        codes: visible("code").map((s) => s.fontFamily),
        oddFigures: [...document.querySelectorAll(".facts .fig-value, td .fig-value")]
          .filter((el) => el.getClientRects().length)
          .filter((el) => {
            const cell = el.closest("dd, td");
            return cell != null && getComputedStyle(el).fontFamily !== getComputedStyle(cell).fontFamily;
          })
          .map((el) => `"${(el.textContent ?? "").trim()}" in ${where(el.closest("dd, td")!)}`),
      };
    }, [...RAMP]);
  };

  test("body's stack is the sans token without the face that does not ship", async ({ page }) => {
    const { tokens } = await read(page, "/");
    const [first, ...rest] = tokens.sans.split(",").map((family) => family.trim());
    expect(first).toBe('"IBM Plex Sans"');
    expect(tokens.body).toBe(rest.join(", "));
  });

  test("every family, size, claim mark and code span is one the system names", async ({ page }) => {
    const strays: string[] = [];
    const allSizes = new Set<string>();
    const claims = new Map<string, string>();
    for (const route of ROUTES) {
      const census = await read(page, route);
      const { tokens } = census;
      const named = new Set(Object.values(tokens));
      for (const [family, el] of Object.entries(census.families)) {
        if (!named.has(family)) strays.push(`${route}: ${el} is set in ${family}`);
      }
      const ramp = new Set([...census.ramp, census.bodySize]);
      for (const [size, el] of Object.entries(census.sizes)) {
        allSizes.add(size);
        if (!ramp.has(size)) strays.push(`${route}: ${el} is ${size}, which is not a step on the ramp`);
      }
      for (const figure of census.oddFigures) strays.push(`${route}: ${figure} is in another face than its cell`);
      for (const claim of census.claims) claims.set(claim, route);
      expect(census.figValue, "a figure in prose is still the mono token").toBe(tokens.mono);
      for (const family of new Set(census.codes)) {
        if (family !== census.figValue) strays.push(`${route}: code is set in ${family}, not in the figures' face`);
      }
    }
    expect(strays, strays.join("\n")).toEqual([]);
    expect([...claims.keys()], "every claim mark shares one family and one size").toHaveLength(1);
    expect(allSizes.size, [...allSizes].join(" ")).toBeLessThanOrEqual(RAMP.length + ALLOWANCE);
  });
});
