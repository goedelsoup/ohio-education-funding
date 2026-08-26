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

import {
  contrast,
  deltaE2000,
  fromOklch,
  hueGap,
  minSeparation,
  parseHex,
  simulate,
  toHex,
  toLab,
  toOklch,
} from "../../src/lib/plot/palette.ts";
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

describe("the boundary of a control", () => {
  /*
   * A different requirement from a hairline, and it was being met with one.
   *
   * `--border` draws card edges and table rules, where the eye is the only judge. It also drew
   * every select, tab, toggle and pill on the site — and the edge of a control is what says where
   * the control *is*, which WCAG 1.4.11 asks 3:1 of against its surround. `--border` measures
   * 1.34:1 against `--surface-1` and 1.18:1 against `--surface-2` in light, 1.40 and 1.28 in dark:
   * roughly half, in both themes, on every form control the site has.
   *
   * Both surfaces, because a control sits on the inset ground inside a card that sits on the page
   * ground, and its boundary is against both.
   */
  test("clears 3:1 against both surfaces a control sits on, in both modes", () => {
    for (const mode of ["light", "dark"] as const) {
      const tokens = PALETTE[mode].tokens;
      const edge = tokens.get("--border-control");
      expect(edge, `--border-control is not declared in the ${mode} palette`).toBeTruthy();
      for (const surface of ["--surface-1", "--surface-2"] as const) {
        const ground = tokens.get(surface)!;
        expect(
          contrast(parseHex(edge!), parseHex(ground)),
          `${mode} --border-control on ${surface}`,
        ).toBeGreaterThanOrEqual(3);
      }
    }
  });

  test("the hairline is left alone, and is still the thing that failed", () => {
    // Stated rather than assumed: this is why there are two tokens and not one darkened one. A
    // `--border` at 3:1 would put a control's boundary on every card edge and table rule on the
    // site, which is a different design and not the one being asked for.
    for (const mode of ["light", "dark"] as const) {
      const tokens = PALETTE[mode].tokens;
      expect(
        contrast(parseHex(tokens.get("--border")!), parseHex(tokens.get("--surface-1")!)),
        `${mode} --border is a hairline`,
      ).toBeLessThan(3);
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
    // A link is set in the ink, so this is never body-length text — but it IS the word under the
    // cursor on hover and on keyboard focus, so it is held to the text bar and not the mark's.
    "--link",
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
    const svg = renderToString((w) => barSpec(mixed, { width: w }), { label: "test" });
    // The polarity pair the palette licenses for gain against loss, both present exactly here.
    expect(svg).toContain("var(--series-formula)");
    expect(svg).toContain("var(--series-guarantee)");
    const abs = renderToString((w) => barSpec([{ label: "b", value: 2 }], { width: w }), { label: "test" });
    expect(abs).not.toContain("var(--series-guarantee)");
  });

  test("and leaves an all-positive chart exactly as it was", () => {
    const svg = renderToString((w) => barSpec(positive, { width: w }), { label: "test" });
    expect(svg).not.toContain("var(--series-guarantee)");
    // Rounded data end, which signed mode drops and this must keep.
    expect(svg).toMatch(/rx|A4,4/);
  });
});

/**
 * The rule this file says it derives by, run instead of trusted.
 *
 * `colors.css` claims the `-text` variants hold "OKLab hue and chroma to the mark colour's exactly
 * and only lightness moves, until the worst of the three surfaces reaches 5:1", and states the
 * drift to the hundredth of a degree. Nothing here could check any of it, in a file whose own
 * header records three separate figures that turned out to come from a tool nobody committed.
 *
 * It reproduces. Every claimed number below came back within the tolerance stated beside it, which
 * is worth saying plainly: this derivation is the one thing in the palette that was always what it
 * said it was. The test exists so it stays that way, and so `--link` — which is derived the same
 * way and could not be checked against a parent, having none — is held to the same rule.
 */
describe("the derivation rule", () => {
  const SURFACES = ["--surface-0", "--surface-1", "--surface-2"] as const;
  const tightest = (mode: keyof typeof PALETTE, hex: string): number =>
    Math.min(
      ...SURFACES.map((s) => contrast(parseHex(hex), parseHex(PALETTE[mode].tokens.get(s)!))),
    );

  /** Hue drift a `-text` variant is allowed from its mark, as the file states it, per mode. */
  const DERIVED = [
    { mark: "--series-formula", text: "--series-formula-text", drift: { light: 0.02, dark: 0.1 } },
    { mark: "--series-guarantee", text: "--series-guarantee-text", drift: { light: 2.8, dark: 0.1 } },
  ] as const;

  test("holds hue and chroma to the mark, and moves only lightness", () => {
    for (const mode of ["light", "dark"] as const) {
      for (const pair of DERIVED) {
        const mark = toOklch(parseHex(PALETTE[mode].tokens.get(pair.mark)!));
        const text = toOklch(parseHex(PALETTE[mode].tokens.get(pair.text)!));
        expect(hueGap(mark.h, text.h), `${mode} ${pair.text} hue drift`).toBeLessThanOrEqual(
          pair.drift[mode],
        );
        expect(Math.abs(mark.c - text.c), `${mode} ${pair.text} chroma`).toBeLessThan(0.004);
        // And lightness DID move, or the split would not exist. Direction is the mode's: darker
        // against a light surface, lighter against a dark one.
        const moved = mode === "light" ? mark.l - text.l : text.l - mark.l;
        expect(moved, `${mode} ${pair.text} lightness`).toBeGreaterThan(0.01);
      }
    }
  });

  /*
   * "Until the worst of the three surfaces reaches 5:1" is a stronger claim than "clears 5:1", and
   * it is the one worth holding: a value that merely clears could be anything darker, chosen by
   * eye. Landing ON the bar is what makes it the least move, which is what makes it derived.
   */
  test("lands on the 5:1 bar rather than merely clearing it", () => {
    for (const mode of ["light", "dark"] as const) {
      for (const name of ["--series-formula-text", "--series-guarantee-text", "--link"] as const) {
        const cr = tightest(mode, PALETTE[mode].tokens.get(name)!);
        expect(cr, `${mode} ${name} tightest surface`).toBeGreaterThanOrEqual(5);
        expect(cr, `${mode} ${name} is further from the bar than one step of lightness`).toBeLessThan(5.1);
      }
    }
  });

  test("except light muted, which moved, and dark muted, which the file says did not", () => {
    // Stated rather than skipped. `--text-muted` was solved the same way in light and came out at
    // 5.03; dark was already at 5.29 and was left alone, so it clears the bar without landing on
    // it. An assertion that ignored the difference would be asserting nothing about either.
    expect(tightest("light", PALETTE.light.tokens.get("--text-muted")!)).toBeLessThan(5.1);
    expect(tightest("dark", PALETTE.dark.tokens.get("--text-muted")!)).toBeGreaterThan(5.2);
  });
});

/**
 * One hue for "you can operate this", and the reason it is not a third data hue.
 *
 * #187 asked for `--link` as a third TEXT colour so a coloured word could not be a link and a
 * datum at once. `scripts/link-hue-search.ts` searched for one and there is none: at this
 * palette's own chroma the best hue anywhere on the circle separates from the two series text
 * colours by CIEDE2000 10.2 under the worst of four visions, against 15.0 for the ordinal ramp
 * this file accepted and 10.9 for the five-step ramp it deleted. Blue and orange are the two ends
 * of the axis a dichromat keeps and both are spent.
 *
 * So the link is set in the ink and `--link` carries the underline. These tests hold the parts of
 * that which can go wrong quietly.
 */
describe("the interaction hue", () => {
  const SERIES_TEXT = ["--series-formula-text", "--series-guarantee-text"] as const;

  test("is one hue and one chroma across both themes, as the palette's rule 1 requires", () => {
    const light = toOklch(parseHex(PALETTE.light.tokens.get("--link")!));
    const dark = toOklch(parseHex(PALETTE.dark.tokens.get("--link")!));
    expect(hueGap(light.h, dark.h), "--link hue across themes").toBeLessThan(0.2);
    expect(Math.abs(light.c - dark.c), "--link chroma across themes").toBeLessThan(0.001);
  });

  test("is as far from both series hues as one hue can be", () => {
    // 326.7 degrees is the bisector of the arc the palette does not spend: 71.2 from each. The
    // floor is well under that, because the claim being held is "not a series hue", not the exact
    // bisector — but a value that drifted toward either series would be choosing a side.
    for (const mode of ["light", "dark"] as const) {
      const link = toOklch(parseHex(PALETTE[mode].tokens.get("--link")!));
      for (const series of ["--series-formula", "--series-guarantee"] as const) {
        const hue = toOklch(parseHex(PALETTE[mode].tokens.get(series)!)).h;
        expect(hueGap(link.h, hue), `${mode} --link against ${series}`).toBeGreaterThan(60);
      }
    }
  });

  test("and it is as saturated as the colours it sits beside, not a duller third rank", () => {
    // The search will trade saturation for separation without being asked — a grey separates from
    // everything. The chroma floor is what stopped it, and this is that floor, kept.
    for (const mode of ["light", "dark"] as const) {
      const link = toOklch(parseHex(PALETTE[mode].tokens.get("--link")!));
      const series = SERIES_TEXT.map((n) => toOklch(parseHex(PALETTE[mode].tokens.get(n)!)).c);
      expect(link.c, `${mode} --link chroma`).toBeGreaterThan(Math.min(...series) - 0.01);
    }
  });

  /**
   * The measurement the whole decision rests on: what a link gains by not being coloured.
   *
   * A link set in `--series-formula-text` separated from a formula figure by nothing at all — it
   * WAS the formula figure's colour. Set in the ink it separates by more than three times what
   * the best available third hue could have bought, because it is not competing on colour.
   */
  test("an ink-set link separates from a series figure by far more than any hue could", () => {
    for (const mode of ["light", "dark"] as const) {
      const ink = parseHex(PALETTE[mode].tokens.get("--text-primary")!);
      const worstVision = Math.min(
        ...VISIONS.map((vision) =>
          Math.min(
            ...SERIES_TEXT.map((n) =>
              deltaE2000(
                toLab(simulate(ink, vision)),
                toLab(simulate(parseHex(PALETTE[mode].tokens.get(n)!), vision)),
              ),
            ),
          ),
        ),
      );
      // 31.2 light and 33.8 dark when this was written. The floor is the ordinal ramp's own bar
      // doubled, which the best third hue (10.2) missed by a factor of three.
      expect(worstVision, `${mode} ink against the series text pair`).toBeGreaterThan(30);
    }
  });
});

/**
 * The focus ring, which was a token nothing read.
 *
 * `--focus-ring` was declared as `var(--series-formula)` and every one of the seven
 * `:focus-visible` rules in `app.css` hard-coded the series token instead — so the indirection
 * existed, pointed at the data hue, and could not have been changed from the palette.
 *
 * `svg.plot [data-hover].at` is the case that made it more than tidiness. Its own comment says a
 * cursor "drawn in the mark's own stroke would be indistinguishable from data", and it then drew
 * the cursor in `--series-formula`, which is the formula mark's fill: 1.00:1 against every formula
 * bar on the site, on the affordance a keyboard reader navigates by.
 */
describe("the focus ring", () => {
  const APP = readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8");
  const outlineRules = [...APP.matchAll(/outline:\s*([^;]+);?/g)].map((m) => m[1]!.trim());

  test("is a token that something actually reads", () => {
    expect(readFileSync(TOKENS, "utf8")).toMatch(/--focus-ring:\s*var\(--link\)/);
    expect(outlineRules.filter((r) => r.includes("var(--focus-ring)")).length).toBeGreaterThan(4);
  });

  test("and no ring anywhere is drawn in a data colour", () => {
    // The hard zero. A ring is "you are here"; a series token is "this is what the number is".
    // There is no case for the two being the same value, and this is what stops one reappearing.
    const offenders = outlineRules.filter((r) => /var\(--(series|ordinal)-/.test(r));
    expect(offenders).toEqual([]);
  });

  /**
   * The chart cursor takes the ink, and `--link` would not have done.
   *
   * A ring around a MARK is a different problem from a ring on a surface: it has to contrast with
   * the thing it surrounds. No single hue on this palette clears 3:1 against every mark — the best
   * available is 2.02:1, and `--link` itself manages 1.19 light and 1.03 dark against the formula
   * blue. The ink clears it, because the ink is the one colour a chart never draws with.
   */
  test("around a chart mark is the ink, which clears 3:1 against both series marks", () => {
    expect(APP).toMatch(/\[data-hover\]\.at\s*\{[^}]*outline:\s*2px solid var\(--text-primary\)/);
    for (const mode of ["light", "dark"] as const) {
      const ink = parseHex(PALETTE[mode].tokens.get("--text-primary")!);
      for (const mark of ["--series-formula", "--series-guarantee"] as const) {
        expect(
          contrast(ink, parseHex(PALETTE[mode].tokens.get(mark)!)),
          `${mode} ink ring on ${mark}`,
        ).toBeGreaterThanOrEqual(3);
      }
    }
  });

  /**
   * And still does not clear it against the **raw** `--ordinal-3` token — which is true, and was
   * the wrong thing to conclude from.
   *
   * Asserted as a failure on purpose, the way `--border` is asserted to still be a hairline. #198
   * read this row as "a keyboard reader landing on the darkest class of a scatter plot gets a ring
   * they cannot see as a ring", and that does not happen. A cursor does not land on a mark; it
   * lands on whatever carries `data-hover`, and measured over the built site not one of 319,060
   * hover targets is filled with an ordinal token. `tests/e2e/cursor.spec.ts` holds that.
   *
   * The value below is a fact about two tokens. It is not a fact about anything a reader meets.
   */
  test("and still does not clear it against the raw dark end of the ordinal ramp", () => {
    for (const mode of ["light", "dark"] as const) {
      expect(
        contrast(
          parseHex(PALETTE[mode].tokens.get("--text-primary")!),
          parseHex(PALETTE[mode].tokens.get("--ordinal-3")!),
        ),
        `${mode} ink ring on the raw --ordinal-3 token`,
      ).toBeLessThan(3);
    }
  });

  /**
   * But it clears against the ramp **as the site paints it**, which is the figure that was missing.
   *
   * A banded scatter draws the ramp at `fill-opacity: 0.62` and the neutral cloud at `0.45`, so the
   * colour a ring is ever adjacent to is the token composited over the card and not the token. The
   * blend moves every step *away* from the ink, necessarily: the ink and the surface are 19.17
   * apart light and 17.42 dark, so anything pulled toward one is pulled away from the other.
   *
   * `--ordinal-3` at 0.62 comes out at 5.30 light and 3.44 dark. That is the number the cursor
   * comment should have carried, and the reason the scatter was never the problem.
   */
  test("and does clear it against the ramp as a chart actually paints it", () => {
    /** `fillOpacity` for a banded scatter in `plot/spec.ts` — the most opaque the ramp is drawn. */
    const BANDED = 0.62;
    const over = (mark: string, surface: string, alpha: number) => {
      const [f, b] = [parseHex(mark), parseHex(surface)];
      return {
        r: Math.round(alpha * f.r + (1 - alpha) * b.r),
        g: Math.round(alpha * f.g + (1 - alpha) * b.g),
        b: Math.round(alpha * f.b + (1 - alpha) * b.b),
      };
    };
    for (const mode of ["light", "dark"] as const) {
      const tokens = PALETTE[mode].tokens;
      const ink = parseHex(tokens.get("--text-primary")!);
      for (const step of ORD3) {
        expect(
          contrast(ink, over(tokens.get(step)!, tokens.get("--surface-1")!, BANDED)),
          `${mode} ink ring on ${step} as painted`,
        ).toBeGreaterThanOrEqual(3);
      }
    }
  });
});

/**
 * A link is the ink with a rule under it, asserted at the source rather than at the value.
 *
 * The token being right is half of it; the rule consuming it is the other half, and that is where
 * this would rot — a later change that sets `a { color }` back to a series token would leave every
 * measurement in this file passing.
 */
describe("what a link is made of", () => {
  const APP = readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8");
  const base = APP.slice(APP.indexOf("\na {"), APP.indexOf("}", APP.indexOf("\na {")));

  test("is set in the ink, and carries the interaction hue as its underline", () => {
    expect(base).toMatch(/color:\s*var\(--text-primary\)/);
    expect(base).toMatch(/text-decoration-color:\s*var\(--link\)/);
  });

  test("and no rule anywhere sets a link's colour from a series token", () => {
    const offenders = [...APP.matchAll(/([^{}]*\ba\b[^{}]*)\{([^}]*)\}/g)]
      .map((m) => ({ selector: m[1] ?? "", body: m[2] ?? "" }))
      .filter((r) => /(^|[\s,>+~])a(:|\[|\s|$|\.)/.test(r.selector))
      .filter((r) => /color:\s*var\(--series-/.test(r.body))
      .map((r) => r.selector.trim());
    expect(offenders).toEqual([]);
  });
});

/**
 * OKLCH, round-tripped, because every claim above is stated in it.
 *
 * The conversion is new to this repository and the palette's derivation rule is now checked
 * through it, so a silent error in the matrices would make the derivation tests agree with a
 * wrong answer.
 */
describe("the OKLCH conversion", () => {
  test("round-trips every colour in the palette exactly", () => {
    for (const mode of ["light", "dark"] as const) {
      for (const [name, hex] of PALETTE[mode].tokens) {
        const back = fromOklch(toOklch(parseHex(hex)));
        expect(back, `${mode} ${name} left the gamut`).not.toBeNull();
        expect(toHex(back!), `${mode} ${name}`).toBe(hex);
      }
    }
  });

  test("refuses a coordinate outside sRGB rather than clamping it into a different colour", () => {
    // The clamp is the dangerous failure: a search over the space would optimise over values it
    // cannot ship, and every one of them would come back as a plausible hex.
    expect(fromOklch({ l: 0.6, c: 0.4, h: 150 })).toBeNull();
    expect(fromOklch({ l: 0.5, c: 0.165, h: 326.7 })).not.toBeNull();
  });

  /**
   * An absolute anchor, because the round-trip above is not one.
   *
   * Checked rather than assumed: perturbing a forward matrix coefficient in the fourth decimal
   * leaves the round-trip passing. It has to — the inverse is validated against the forward and
   * nothing else, the two errors partly cancel, and 8-bit quantisation absorbs what is left. Only
   * a transposed digit, three orders of magnitude larger, gets through to a changed hex.
   *
   * So the conversion is also pinned to the sRGB primaries' published OKLab coordinates, which
   * come from outside this repository. That is what catches an error in the matrices themselves
   * rather than an error in one direction of them.
   */
  test("agrees with the published coordinates of the sRGB primaries", () => {
    // Five decimals, and the precision is the whole value of the test. At three it still passes
    // with a matrix coefficient wrong in the fourth decimal — no better than the round-trip. At
    // five it catches one wrong in the fifth, which is finer than an 8-bit palette can express.
    const anchors = [
      { hex: "#ff0000", l: 0.627955, c: 0.257683, h: 29.2339 },
      { hex: "#00ff00", l: 0.86644, c: 0.294827, h: 142.4953 },
      { hex: "#0000ff", l: 0.452014, c: 0.313214, h: 264.052 },
    ];
    for (const a of anchors) {
      const got = toOklch(parseHex(a.hex));
      expect(got.l, `${a.hex} L`).toBeCloseTo(a.l, 5);
      expect(got.c, `${a.hex} C`).toBeCloseTo(a.c, 5);
      expect(got.h, `${a.hex} hue`).toBeCloseTo(a.h, 3);
    }
    // White is L = 1 exactly, which is the one value the whole scale is normalised on.
    expect(toOklch(parseHex("#ffffff")).l).toBeCloseTo(1, 5);
    expect(toOklch(parseHex("#000000")).l).toBeCloseTo(0, 5);
  });

  test("and puts the two spent hues where the palette says they are", () => {
    expect(toOklch(parseHex("#2a78d6")).h).toBeCloseTo(255.5, 1);
    expect(toOklch(parseHex("#eb6834")).h).toBeCloseTo(40.6, 1);
  });
});
