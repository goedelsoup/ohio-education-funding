/**
 * Every keyboard tab stop shows a focus ring, and every ring clears 3:1 against what is behind it.
 *
 * # Why this exists as its own file
 *
 * It is item 8 of #182's exit gate — *"every restyled interactive element needs its
 * `:focus-visible` state re-checked"* — and it was the last of the nine with no runner. The
 * redesign created the need for it twice over: phase 2 (#185) took the border off controls, and
 * phase 5 (#187) moved every ring off the data hue it had been drawn in.
 *
 * Auditing it found the behaviour already correct on both counts. What was missing was the check,
 * which is the difference between a property the site has and a property the site keeps.
 *
 * # The thing that makes a naive version of this useless
 *
 * `element.focus()` does **not** match `:focus-visible` in Chromium for a link or a button —
 * that is the whole point of the pseudo-class, which exists to tell a keyboard reader apart from a
 * mouse one. The first version of this swept every focusable element by script and reported that
 * 642 anchors on `/districts` had no focus ring. They all do. It was measuring the unfocused state
 * under another name.
 *
 * So this tabs. Real `Tab` presses, `document.activeElement` after each, and `:focus-visible`
 * asserted rather than assumed.
 *
 * # The routes and the depth, both computed
 *
 * A greedy minimal cover over thirteen candidate routes. Tabbing them reaches **14 control
 * families**, and each of the four earns its place: `/statewide` +9, `/scenario` +3, `/` +1,
 * `/districts` +1. The count is asserted below, so a route that stops carrying its share fails
 * here rather than quietly shrinking the sweep.
 *
 * 45 tab stops because 90 reaches nothing further — measured, across six routes, identical sets.
 *
 * The population is what **tabbing** reaches, which is smaller than what the focusable-element
 * selector matches: a `[tabindex]` inside a closed `<details>` is one of those, and a keyboard
 * reader cannot get to it either. Counting by selector rather than by tab is what made the first
 * version of this assert 15 and fail.
 *
 * What sampling cannot catch is a family that appears only on a route nobody visits. That is the
 * limit; the alternative is tabbing every one of 3,506 pages.
 *
 * # Two populations, and only one of them is measurable
 *
 * The site authors a ring for some controls and leaves the rest to the browser, which is the
 * recommended arrangement — enhance the default rather than remove it. Measured over the four
 * routes, those are two distinct computed shapes and the split is clean:
 *
 *     solid 2px, var(--link)   a.section-anchor, button.year-chip, summary, svg
 *     auto 1px, the UA ring    a, a.brand, a.flag, button, button.flag, button.ghost,
 *                              div.scroll, input, input.vh, select
 *
 * **An `auto` ring cannot be judged from `outlineColor`.** Chromium paints it as a two-tone
 * indicator — a dark stroke and a light one — precisely so it stays visible on any background, and
 * `getComputedStyle` reports a single colour that is neither the whole thing nor stable across
 * platforms: `rgb(0, 95, 204)` on macOS against `rgb(16, 16, 16)` on the Linux runner, same
 * commit, same theme.
 *
 * The first version of this checked both populations alike. It passed locally and failed on CI
 * with ten families at 1.01:1 — and all ten were `auto`, while the four `solid` ones passed on
 * both. A green that was a property of the reviewer's machine, which is the hazard `axisFoot` and
 * the display-face sweep are both already about.
 *
 * So contrast is asserted on the rings the site *authors*, which is exactly what item 8 names:
 * "every **restyled** interactive element". Presence is asserted on all of them, and the authored
 * count is asserted too — otherwise dropping every `outline` rule would fall through to the UA
 * ring and read as fine.
 *
 * # Both themes
 *
 * `--focus-ring` resolves to `--link`, which is a different colour in each, and the surfaces it is
 * drawn against differ too. A ring solved in one theme is not solved in the other — the same
 * reason `palette.spec.ts` runs every contrast twice.
 */

import { expect, test } from "@playwright/test";

/** The minimal cover, with the number of families tabbing them reaches. */
const ROUTES = ["/statewide", "/scenario", "/", "/districts"];
const FAMILIES = 14;

/** Deep enough that doubling it reaches nothing new. */
const TABS = 45;

test.describe("the keyboard focus ring", () => {
  for (const scheme of ["light", "dark"] as const) {
    test(`is present and clears 3:1 on every tab stop, in ${scheme}`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: scheme });
      await page.setViewportSize({ width: 1280, height: 900 });

      const missing = new Map<string, string>();
      const faint = new Map<string, string>();
      const families = new Set<string>();
      /** Families whose ring the site draws itself, rather than leaving to the browser. */
      const authored = new Set<string>();

      for (const route of ROUTES) {
        await page.goto(route);
        // Start from the document, not from wherever the previous page left the cursor.
        await page.locator("body").click({ position: { x: 2, y: 2 } });

        for (let step = 0; step < TABS; step += 1) {
          await page.keyboard.press("Tab");
          const stop = await page.evaluate(() => {
            const el = document.activeElement as HTMLElement | null;
            if (!el || el === document.body) return null;

            const name = `${el.tagName.toLowerCase()}${
              el.className && typeof el.className === "string"
                ? `.${el.className.split(/\s+/)[0]}`
                : ""
            }`;
            // A mouse-focused control legitimately shows nothing; only the keyboard state is asked.
            if (!el.matches(":focus-visible")) return { name, visible: false };

            const style = getComputedStyle(el);
            const width = parseFloat(style.outlineWidth);
            if (style.outlineStyle === "none" || width === 0) {
              return { name, visible: true, ring: false };
            }
            /*
             * The browser's own ring. Present, which is the half that can be checked here; its
             * contrast cannot be, because it is two strokes and this is one colour. Left to the UA
             * on purpose — removing it to gain a measurable ring would trade a working indicator
             * for a checkable one.
             */
            if (style.outlineStyle === "auto") return { name, visible: true, ring: true, ua: true };

            const parse = (value: string) => {
              const parts = value.match(/[\d.]+/g)!.map(Number);
              return { r: parts[0]!, g: parts[1]!, b: parts[2]!, a: parts[3] ?? 1 };
            };
            const luminance = (c: { r: number; g: number; b: number }) => {
              const channel = (v: number) => {
                const s = v / 255;
                return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
              };
              return 0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b);
            };
            const contrast = (a: typeof parse extends never ? never : ReturnType<typeof parse>, b: ReturnType<typeof parse>) => {
              const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
              return (hi! + 0.05) / (lo! + 0.05);
            };

            /*
             * The first OPAQUE background up the tree. A control's own background is usually
             * transparent, and the colour a ring is actually drawn against is the card or the page
             * behind it — which is what 1.4.11 means by adjacent.
             */
            let node: HTMLElement | null = el;
            let behind = { r: 255, g: 255, b: 255, a: 1 };
            while (node) {
              const candidate = parse(getComputedStyle(node).backgroundColor);
              if (candidate.a > 0.5) { behind = candidate; break; }
              node = node.parentElement;
            }
            return {
              name,
              visible: true,
              ring: true,
              ratio: contrast(parse(style.outlineColor), behind),
              colours: `${style.outlineColor} on rgb(${behind.r}, ${behind.g}, ${behind.b})`,
            };
          });

          if (stop === null) break;
          families.add(stop.name);
          if (!stop.visible) continue;
          if (!stop.ring) {
            missing.set(stop.name, `${route}: no outline on keyboard focus`);
          } else if (!stop.ua) {
            authored.add(stop.name);
            if (stop.ratio! < 3) {
              faint.set(stop.name, `${route}: ${stop.ratio!.toFixed(2)}:1 — ${stop.colours}`);
            }
          }
        }
      }

      expect(
        families.size,
        "the four routes reach 14 control families between them; one has stopped carrying its share",
      ).toBeGreaterThanOrEqual(FAMILIES);
      expect(
        [...missing].map(([name, where]) => `${name} — ${where}`),
        "a keyboard tab stop with no focus ring",
      ).toEqual([]);
      expect(
        [...faint].map(([name, where]) => `${name} — ${where}`),
        "a site-authored focus ring under WCAG 1.4.11's 3:1 against the surface behind it",
      ).toEqual([]);
      // Otherwise a stylesheet that dropped every `outline` rule would satisfy everything above by
      // falling through to the UA ring, and this would call that fine.
      expect(
        authored.size,
        "no control on these routes draws its own focus ring any more",
      ).toBeGreaterThanOrEqual(4);
    });
  }
});
