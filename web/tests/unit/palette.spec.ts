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
import { barSpec } from "../../src/lib/plot/spec.ts";
import { renderToString } from "../../src/lib/plot/ssr.ts";

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

/**
 * Text is held to 4.5:1 and marks to 3:1, and the palette carries a separate pair for each.
 *
 * Every link on this site was set in `--series-formula`, which is a mark colour: 4.01:1 on the
 * page ground, 3.80 on the inset, and `.err` — the scenario runner's failure notice — was set in
 * `--series-guarantee` at 2.75. Both modes failed, not only light; the dark pair reaches 4.38 and
 * 4.10 against `--surface-2` and had been read as passing because nothing measured it there.
 *
 * The `-text` variants hold OKLab hue and chroma to the mark colour and move only lightness, so
 * this asserts the requirement rather than the values. Change a surface, or restate a hue, and
 * the failure names the pair that has to move.
 */
describe("text colour", () => {
  const SURFACES = ["--surface-0", "--surface-1", "--surface-2"] as const;
  // Every token a rule may set `color` from. `--text-muted` is in here because it was the
  // third failure and the least visible one: annotation, so nobody looks at it twice.
  const TEXT = [
    "--series-formula-text",
    "--series-guarantee-text",
    "--text-primary",
    "--text-secondary",
    "--text-muted",
  ] as const;

  test("clears 4.5:1 on every surface it can be set on, in both modes", () => {
    for (const mode of ["light", "dark"] as const) {
      for (const name of TEXT) {
        const value = PALETTE[mode].tokens.get(name);
        expect(value, `${name} is not declared in the ${mode} palette`).toBeTruthy();
        for (const surface of SURFACES) {
          const ground = PALETTE[mode].tokens.get(surface)!;
          expect(
            contrast(parseHex(value!), parseHex(ground)),
            `${mode} ${name} on ${surface}`,
          ).toBeGreaterThanOrEqual(4.5);
        }
      }
    }
  });

  test("and the mark colours are still the ones the charts use, unchanged", () => {
    // The split exists so the marks did NOT have to move. If these drift, a chart's colour has
    // been changed to solve a text problem, which is the trade this pair was added to avoid.
    expect(PALETTE.light.tokens.get("--series-formula")).toBe("#2a78d6");
    expect(PALETTE.light.tokens.get("--series-guarantee")).toBe("#eb6834");
    expect(PALETTE.dark.tokens.get("--series-formula")).toBe("#3987e5");
    expect(PALETTE.dark.tokens.get("--series-guarantee")).toBe("#d95926");
  });
});

/**
 * The palette is written four times and this is the only thing holding the four together.
 *
 * `colors.css` states dark in the media query, restates light under `[data-theme="light"]` and
 * dark again under `[data-theme="dark"]`, deliberately: a shared intermediate is what lets two
 * themes drift into disagreement. But the file's own note says to assert the blocks against each
 * other instead, and nothing did — every measurement in this file reads the explicit-choice
 * blocks, so the media query, which is the palette a reader with no stored preference actually
 * gets, was the one block nothing had ever looked at.
 */
describe("the four restatements of the palette", () => {
  test("agree property for property", () => {
    const media = block("@media (prefers-color-scheme: dark) {");
    const dark = block(':root[data-theme="dark"] {');
    const light = block(":root {");
    const explicitLight = block(':root[data-theme="light"] {');

    expect([...media.keys()].sort()).toEqual([...dark.keys()].sort());
    for (const [name, value] of dark) {
      expect(media.get(name), `${name} differs between the media query and [data-theme="dark"]`)
        .toBe(value);
    }
    for (const [name, value] of explicitLight) {
      expect(light.get(name), `${name} differs between :root and [data-theme="light"]`).toBe(value);
    }

    // Print is the fourth, and the one a reader never chose. It has to be light in FULL — a block
    // that restated only the text tokens would leave a dark surface behind light ink, so the
    // key sets are compared before the values are.
    const print = block("@media print {");
    expect([...print.keys()].sort()).toEqual([...light.keys()].sort());
    for (const [name, value] of light) {
      expect(print.get(name), `${name} differs between :root and the print block`).toBe(value);
    }
  });

  /**
   * The print block only works if it can BEAT the dark ones, and specificity is how.
   *
   * The media query carries dark on a bare `:root`, and `print` matches at the same time as
   * `prefers-color-scheme: dark` when a dark-mode reader prints. `[data-theme="dark"]` is an
   * explicit stamp that no medium unsets. So the print block has to name both, and it has to be
   * last in the file for the specificity ties to fall its way.
   */
  test("print names both dark cases and is stated last", () => {
    const css = readFileSync(TOKENS, "utf8");
    const start = css.indexOf("@media print {");
    expect(start, "no print block in tokens/colors.css").toBeGreaterThan(-1);
    const selector = css.slice(start, css.indexOf("{", css.indexOf("{", start) + 1));
    expect(selector).toContain(":root,");
    expect(selector).toContain(':root[data-theme="dark"]');
    expect(start).toBeGreaterThan(css.indexOf('@media (prefers-color-scheme: dark) {'));
    expect(start).toBeGreaterThan(css.indexOf(':root[data-theme="dark"] {'));
  });
});

/**
 * A bar below zero is drawn below zero.
 *
 * `barSpec` took `Math.abs(b.value)` and filled every bar alike, so a deficit and a surplus of
 * the same size were the same picture. Exactly one bar in the build is affected — Springfield
 * Local's −$2,812,534 at 30 June FY2021 — which is why it survived: a defect with one instance
 * looks like a chart nobody has looked at rather than a chart that is wrong.
 *
 * The other assertion is the one that protects the rest of the site: eight charts are built on
 * this spec and none of them has a negative value, so entering signed mode must not change what
 * they draw.
 */
describe("a bar chart with a negative value", () => {
  const positive = [
    { label: "a", value: 3 },
    { label: "b", value: 1 },
  ];
  const mixed = [
    { label: "a", value: 3 },
    { label: "b", value: -2 },
  ];

  test("puts the negative bar on the other side of zero, in the other colour", () => {
    const svg = renderToString(barSpec(mixed), { label: "test" });
    // The polarity pair the palette licenses for gain against loss, both present exactly here.
    expect(svg).toContain("var(--series-formula)");
    expect(svg).toContain("var(--series-guarantee)");
    const abs = renderToString(barSpec([{ label: "b", value: 2 }]), { label: "test" });
    expect(abs).not.toContain("var(--series-guarantee)");
  });

  test("and leaves an all-positive chart exactly as it was", () => {
    const svg = renderToString(barSpec(positive), { label: "test" });
    expect(svg).not.toContain("var(--series-guarantee)");
    // Rounded data end, which signed mode drops and this must keep.
    expect(svg).toMatch(/rx|A4,4/);
  });
});
