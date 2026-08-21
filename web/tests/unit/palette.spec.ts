/**
 * The separation claims this repository makes about its own palette, measured.
 *
 * # Why this test exists
 *
 * `plot/tokens.ts` and `tokens/colors.css` have carried figures for a long time — "ΔE 21.4 light,
 * 21.6 dark, 20.8 under the worst CVD simulation", "five steps of this hue close to 10.9", "the end
 * steps sit near 2.2:1" — and nothing in the repository could produce any of them. They were
 * computed once, elsewhere, by a tool that was never committed. A design system built on top of
 * this repository then inherited them, repeated them, and reasoned from them.
 *
 * That makes them provenance rather than measurement. This test turns the reproducible ones into
 * assertions and records, in the open, which ones do not reproduce.
 *
 * # What reproduces, and what does not
 *
 * The **contrast** figure reproduces exactly: the ramp's end steps sit at 2.20:1 against their own
 * surface in both modes, which is the number the comment claims. That agreement is worth stating
 * because it establishes that the colour values and the sRGB chain here are both right — so where
 * the ΔE figures disagree, the disagreement is about the metric, not about the colours.
 *
 * The **ΔE** figures do not reproduce under any standard metric:
 *
 *     claimed (3-step, normal vision)     21.4 light   21.6 dark
 *     CIE76                               31.1         28.1
 *     CIE94                               23.0         25.5
 *     CIEDE2000                           17.9         19.0
 *     OKLab x100                          18.1         21.6
 *
 * No metric produces both. OKLab happens to hit 21.6 in dark and misses by 3.3 in light. The
 * claimed value sits between CIE76 and CIEDE2000 in both modes, which is what a figure from an
 * unidentified tool looks like.
 *
 * **This does not overturn the decision those figures were used to justify.** Three steps really do
 * separate better than five — that ordering holds under every metric tested. What is not supported
 * is the precision: the argument was stated as a measurement and cannot be checked as one.
 *
 * CIEDE2000 is what this file uses, and says so rather than leaving it to be inferred. It is the
 * current standard and it is the least flattering of the four to a blue ramp, because its rotation
 * term corrects exactly the region where a straight Lab distance overstates difference — and both
 * of these ramps are blue.
 */

import { readdirSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, test } from "vitest";

import { contrast, minSeparation, parseHex } from "../../src/lib/plot/palette.ts";

const TOKENS = resolve(process.cwd(), "src/styles/tokens/colors.css");
const VISIONS = ["normal", "protan", "deutan", "tritan"] as const;

/** The declarations of one `:root` block, by custom property. */
function block(marker: string): Map<string, string> {
  const css = readFileSync(TOKENS, "utf8");
  const start = css.indexOf(marker);
  expect(start, `no ${marker} block in tokens/colors.css`).toBeGreaterThan(-1);
  const body = css.slice(start, css.indexOf("\n}", start));
  return new Map(
    [...body.matchAll(/(--[a-z0-9-]+):\s*(#[0-9a-f]{3,8})\s*;/gi)].map((m) => [
      m[1]!,
      m[2]!.toLowerCase(),
    ]),
  );
}

/** Light is the first `:root`; dark is read from the explicit choice, which restates the query. */
const PALETTE = {
  light: { tokens: block(":root {"), surface: "--surface-1" },
  dark: { tokens: block(':root[data-theme="dark"] {'), surface: "--surface-1" },
} as const;

const ramp = (mode: keyof typeof PALETTE, names: string[]): string[] =>
  names.map((n) => {
    const value = PALETTE[mode].tokens.get(n);
    expect(value, `${n} is not declared in the ${mode} palette`).toBeTruthy();
    return value!;
  });

const ORD3 = ["--ordinal-1", "--ordinal-2", "--ordinal-3"];
const PAIR = ["--series-formula", "--series-guarantee"];

/** The worst separation across normal vision and all three dichromacies. */
const worst = (colours: string[]): number =>
  Math.min(...VISIONS.map((vision) => minSeparation(colours, vision)));

describe("the categorical pair", () => {
  test("separates far beyond either ramp, under every vision, in both modes", () => {
    for (const mode of ["light", "dark"] as const) {
      expect(worst(ramp(mode, PAIR)), `${mode} pair`).toBeGreaterThan(45);
    }
  });
});

describe("the three-step ordinal ramp", () => {
  /**
   * The floor is the ramp's own measured worst, less a point of slack.
   *
   * Not an aspiration and not a standard threshold — this ramp is the one licensed for all-pairs
   * forms, so what it scores *is* the bar every other ordinal encoding here is judged against. A
   * change that drops it below this is a change that needs arguing for.
   */
  const FLOOR = { light: 14.0, dark: 16.0 };

  test("clears its floor under every vision, in both modes", () => {
    for (const mode of ["light", "dark"] as const) {
      expect(worst(ramp(mode, ORD3)), `${mode} ord3 worst-vision`).toBeGreaterThan(FLOOR[mode]);
    }
  });

  test("has end steps at 2.20:1 against their own surface, which is the figure that reproduced", () => {
    for (const mode of ["light", "dark"] as const) {
      const steps = ramp(mode, ORD3);
      const surface = PALETTE[mode].tokens.get(PALETTE[mode].surface)!;
      // Step 1 in both modes: dark is a selected ramp rather than an inversion, so its first step
      // is the one nearest the dark ground exactly as the light ramp's is nearest the light one.
      // This is the contrast warning the ramp's own comment says obligates a legend and a table.
      expect(contrast(parseHex(steps[0]!), parseHex(surface)), `${mode} step 1`).toBeCloseTo(2.2, 1);
    }
  });
});

describe("the ramp that was removed", () => {
  /**
   * A five-step ramp arrived with the design system, was measured, and is gone.
   *
   *     3-step, worst vision      15.0 light   17.1 dark
   *     5-step, worst vision      10.9 light   10.7 dark
   *
   * 10.9 was already the value this repository cited as its reason for refusing five steps of one
   * hue — "two bands a reader with full colour vision cannot tell apart". The second channel the
   * ramp added did not buy the separation it was introduced to buy, and no four-step subset of it
   * cleared either.
   *
   * `scripts/ramp-search.ts` then searched for a replacement instead of constructing one, and found
   * that five ordinal steps do not fit on the dark surface at all. The tokens were deleted rather
   * than left in place, because a failing ramp sitting in a palette file looks sanctioned.
   *
   * This test is what stops it coming back by hand. Reintroducing a multi-step ramp means running
   * the search and clearing the floor above — not choosing five values that look ordered.
   */
  test("is gone from the tokens, and from everything that could reference it", () => {
    const roots = ["src", "tests"];
    const offenders: string[] = [];
    const walk = (path: string): void => {
      for (const entry of readdirSync(path, { withFileTypes: true })) {
        const next = resolve(path, entry.name);
        if (entry.isDirectory()) {
          walk(next);
        } else if (/\.(ts|tsx|astro|css)$/.test(entry.name) && !next.endsWith("palette.spec.ts")) {
          // The search script names it in prose, describing what it replaced; a declaration or a
          // `var()` reference is the thing that would matter, and neither may exist.
          const text = readFileSync(next, "utf8");
          if (/--ord5-[1-9]\s*:/.test(text) || /var\(\s*--ord5-/.test(text)) offenders.push(next);
        }
      }
    };
    for (const dir of roots) walk(resolve(process.cwd(), dir));
    expect(offenders).toEqual([]);
  });

  test("and the ordinal vocabulary is exactly one ramp of three steps", () => {
    for (const mode of ["light", "dark"] as const) {
      const declared = [...PALETTE[mode].tokens.keys()].filter((k) => /^--ord/.test(k));
      expect(declared.sort()).toEqual(["--ordinal-1", "--ordinal-2", "--ordinal-3"]);
    }
  });
});
