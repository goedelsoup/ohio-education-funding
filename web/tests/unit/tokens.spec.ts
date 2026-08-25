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

  test("and the ledger is grouped, so #186 has decisions rather than an inventory", () => {
    const grouped = clusters(near);
    // The largest clusters are the type scale: many distinct literal sizes reaching for a few
    // `--text-*` tokens. If that stops being true the finding behind #186 has changed shape.
    const sizes = grouped.filter((c) => c.property === "font-size");
    expect(new Set(sizes.map((c) => c.part)).size).toBeGreaterThanOrEqual(8);
    expect(new Set(sizes.map((c) => c.token)).size).toBeLessThanOrEqual(6);
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

  test("carries all four measured breakpoints in that one file", () => {
    // Deleting a duplicate must not delete a breakpoint with it. The four are measured from a
    // build at each width and a missing one lands fragment links under the header.
    const space = withoutComments(read("tokens/space.css"));
    expect([...space.matchAll(/--sticky-chrome\s*:/g)]).toHaveLength(4);
    for (const px of ["168px", "135px", "96px", "87px"]) expect(space).toContain(px);
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
});
