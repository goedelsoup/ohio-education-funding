/**
 * The stylesheet is on the scale, and stays on it.
 *
 * `tokens/` has held type and space values since the design system landed and `app.css` carried
 * its own literals anyway. #112 counted that ledger and closed without spending it, which is the
 * failure this file exists to make impossible to repeat: an adoption nothing asserts is an
 * adoption that comes undone one convenient literal at a time.
 *
 * See `src/lib/tokens.ts` for why a value match alone is not a licence to adopt.
 */

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import {
  NEAR_MISS_CEILING,
  audit,
  clusters,
  parseTokens,
  withoutComments,
} from "../../src/lib/tokens.ts";

const read = (relative: string): string =>
  readFileSync(fileURLToPath(new URL(`../../src/styles/${relative}`, import.meta.url)), "utf8");

const APP = read("app.css");
const TOKENS = [
  ...parseTokens(read("tokens/typography.css")),
  ...parseTokens(read("tokens/space.css")),
];

describe("the type and space scales", () => {
  test("are declared, and there are enough of them to be a scale", () => {
    // A guard on the guard: if the token files move or the parser stops matching, every assertion
    // below passes vacuously against an empty list. `palette.spec.ts` learned this the hard way.
    expect(TOKENS.length).toBeGreaterThan(30);
    expect(TOKENS.map((t) => t.name)).toContain("--space-6");
    expect(TOKENS.map((t) => t.name)).toContain("--text-body");
  });

  test("every token name is unique across the two files", () => {
    // Two files declaring one name is the `--text-body` collision that shipped once already — a
    // size and a colour on `:root`, decided by import order. Cheap to assert, expensive to find.
    const names = TOKENS.map((t) => t.name);
    expect([...new Set(names)].sort()).toEqual([...names].sort());
  });
});

describe("app.css", () => {
  const { exact, near } = audit(APP, TOKENS);

  test("uses the token wherever a token of the right category already holds that value", () => {
    // The hard zero. 114 declarations adopted a token when this landed; the point of the rule is
    // the 115th, written by hand next to a value the scale already had a name for.
    const offenders = exact.map(
      (f) => `app.css:${f.line}  ${f.property}: ${f.value}  ->  var(${f.token})`,
    );
    expect(offenders).toEqual([]);
  });

  test("does not grow the pile of literals that sit near a token without using it", () => {
    /*
     * A ratchet, not a target.
     *
     * These are NOT adoptable here: each one moves a pixel, and the phase that put `app.css` on
     * the scale is contractually a no-op. #186 resolves them, and until it does the count may not
     * rise — a new literal near a token is a new size chosen by eye, which is how a seven-size
     * scale became the thirteen the page renders.
     */
    expect(near.length).toBeLessThanOrEqual(NEAR_MISS_CEILING);
  });

  test("holds no type left, because #186 took the type", () => {
    /*
     * This test used to assert the opposite — that eight-plus distinct literal font sizes were
     * reaching for a handful of `--text-*` tokens — and said that if it stopped being true the
     * finding behind #186 had changed shape. It has: #186 snapped every `font-size` and
     * `line-height` near miss onto the reading and apparatus ramps.
     *
     * A hard zero now, because the register is the thing that was wrong and a new literal size is
     * how it comes back. What remains on the ratchet is spacing, which is rhythm rather than type.
     */
    const grouped = clusters(near);
    expect(grouped.filter((c) => /^(font-size|line-height|letter-spacing)$/.test(c.property))).toEqual([]);
    // What is left is geometry — spacing and one radius — which is the ratchet's remaining job and
    // a different phase's decision. `border-radius: 8px` against `--radius-md: 7px`, five times,
    // is the largest of them and is not a type question at all.
    const kinds = [...new Set(grouped.map((c) => c.property))].sort();
    const GEOMETRY = /^(margin|padding|gap|row-gap|column-gap|inset|top|right|bottom|left|border-radius)/;
    expect(kinds.every((k) => GEOMETRY.test(k)), kinds.join(", ")).toBe(true);
  });
});

describe("--sticky-chrome", () => {
  const FILES = ["app.css", "tokens/space.css", "tokens/colors.css", "tokens/typography.css"];

  test("is declared in exactly one file", () => {
    /*
     * It was declared in two, with identical values, and `app.css` imports the tokens at the top —
     * so both matched at the same specificity and the copy here won on order. The token file's
     * copy was inert, and editing the value where the tokens live would have changed nothing with
     * no way to discover that. #112 counted it and it stayed.
     */
    const declaring = FILES.filter((file) =>
      /--sticky-chrome\s*:/.test(withoutComments(read(file))),
    );
    expect(declaring).toEqual(["tokens/space.css"]);
  });

  test("carries both measured values in that one file, and no longer four", () => {
    /*
     * It was four — 168 / 135 / 96 / 87 — and #189 re-derived them, because the header they
     * measure stopped wrapping. The four were right at approximately the four widths they were
     * taken at and wrong between them: at 680-999px the header is 52px against a token of 96, and
     * at 1000px and up the covered region is 105px against a token of 87, so every fragment link
     * at desktop width landed 19px beneath the chrome.
     *
     * Two now, and the step is the one at 1000px where `.subnav` becomes sticky and adds itself.
     * Asserted as a count as well as by value so that a third breakpoint reappearing is a decision
     * somebody made rather than a value somebody guessed.
     */
    const space = withoutComments(read("tokens/space.css"));
    expect([...space.matchAll(/--sticky-chrome\s*:/g)]).toHaveLength(2);
    expect(space).toContain("52px");
    expect(space).toContain("106px");
    for (const stale of ["168px", "135px", "96px", "87px"]) expect(space).not.toContain(stale);
  });
});

describe("the font stacks", () => {
  test("name IBM Plex first, and body still does not use them", () => {
    /*
     * `--font-sans` has no consumer: `body` restates the platform stack *minus* IBM Plex, so the
     * documented "drop the .woff2 in and every stack picks them up" plan does not work as written.
     * #112 found this and it is still true.
     *
     * It is deliberately NOT fixed here, and this test records why rather than leaving the gap
     * looking unnoticed. Pointing `body` at `var(--font-sans)` *adds* IBM Plex to the front of the
     * stack, so a reader who happens to have it installed gets a different site — which is a
     * visual change, in a phase whose entire contract is that there is none. Worse, it is invisible
     * to this repository's own verification: IBM Plex is not installed on the machines that build
     * or test it, so the baseline would report nothing moved and the change would ship unseen.
     *
     * It belongs to #186, which is allowed to move type and is where the serif decision lands.
     */
    const typography = withoutComments(read("tokens/typography.css"));
    expect(typography).toMatch(/--font-sans:\s*\n?\s*"IBM Plex Sans"/);

    const body = withoutComments(APP).match(/\bbody\s*\{[^}]*\}/)?.[0] ?? "";
    expect(body).toContain("font: 15px/1.55");
    expect(body).not.toContain("var(--font-sans)");
  });

  /*
   * `--font-math` is the one stack where naming the families IS the mechanism.
   *
   * Measured in chromium, same markup, same machine: with the generic `math` keyword alone a
   * `cases` brace rendered 13.7px against a 74.6px table; with the families named it rendered
   * 81.6px against 81.6px. The keyword did not reach a maths font even on a machine that had one
   * installed. So a stack that decayed back to `math` would not fail — it would draw every formula
   * on the site with flat delimiters and nothing would say so.
   */
  test("--font-math names families, and never decays to the bare `math` keyword", () => {
    const typography = withoutComments(read("tokens/typography.css"));
    const stack = typography.match(/--font-math:\s*([^;]+);/)?.[1]?.replace(/\s+/g, " ").trim();
    expect(stack).toBeDefined();
    expect(stack).not.toBe("math");
    for (const family of ["Cambria Math", "STIX Two Math", "NotoSansMath-Regular"]) {
      expect(stack).toContain(family);
    }
  });

  /*
   * The shipped fallback is last, and that position is the whole cost control.
   *
   * CSS matches a font stack per character, so a browser fetches a family only when it needs a
   * glyph from it. Last means a reader whose platform already has Cambria Math or STIX Two Math
   * never touches the file. Anywhere earlier and every reader downloads 27 KB to be handed a face
   * their machine already had — and if it were declared under the name "STIX Two Math", the
   * @font-face would shadow the local font and they would get 357 glyphs instead of 5,169.
   */
  test("the one shipped face is last in the stack, and declared with font-display: block", () => {
    const typography = withoutComments(read("tokens/typography.css"));
    const stack = typography.match(/--font-math:\s*([^;]+);/)![1]!.replace(/\s+/g, " ").trim();
    const families = stack.split(",").map((family) => family.trim().replace(/^"|"$/g, ""));
    expect(families.at(-1), "the generic keyword still closes the stack").toBe("math");
    expect(families.at(-2)).toBe("Ohio Math Fallback");

    const face = typography.match(/@font-face\s*\{[^}]*\}/)?.[0]?.replace(/\s+/g, " ") ?? "";
    expect(face).toContain('font-family: "Ohio Math Fallback"');
    expect(face).toContain("ohio-math-fallback.woff2");
    /*
     * `block`, not `swap`, and the difference is not stylistic. A swap on a text face shows the
     * same words in another face. A swap here shows a `{` one line tall beside a five-line table,
     * which is a formula that means something else, and then moves the page under the reader when
     * it arrives. `block`'s swap period is unbounded, so a slow font is late rather than lost.
     */
    expect(face).toContain("font-display: block");
  });

  test("exactly one @font-face is declared, and it is not for a text face", () => {
    // The standing rule this phase reversed says text ships no binary. One face crossing that line
    // was argued for in writing; a second one arriving quietly is what this counts.
    const typography = withoutComments(read("tokens/typography.css"));
    expect(typography.match(/@font-face/g) ?? []).toHaveLength(1);
    for (const stack of ["--font-sans", "--font-mono", "--font-serif"]) {
      expect(typography.match(new RegExp(`${stack}:[^;]+;`))![0]).not.toContain("Ohio Math");
    }
  });
});
