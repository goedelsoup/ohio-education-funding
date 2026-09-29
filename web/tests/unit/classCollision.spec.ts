/**
 * A bare class rule and a compound selector, both trying to own the same element.
 *
 * # The defect this catches
 *
 * `.formula` was the code-block rule — background, padding, a radius — and `.sw.formula` was the
 * legend swatch for the formula series, one of six ten-pixel boxes drawn beside a chart legend.
 * Nothing about those two facts is related: a swatch is not a code block. But CSS does not know
 * that. A **bare** single-class rule matches every element carrying that class, including one that
 * also carries a second class the rule's author never heard of, so `.formula`'s padding and radius
 * landed on `<i class="sw formula">` and the swatch computed to 22.375px against 10px for every
 * other one — on 556 swatches across 296 pages, since aee2e0e7. The fix (`app.css`, the code-block
 * rule) was to require the type this component actually renders as: `div.formula`, which a bare
 * `<i>` can never match. Reverting that one word reopens the hole — see `classCollision.spec.ts.md`
 * in the PR body for the proof.
 *
 * # What "bare" and "compound" mean here
 *
 * A **bare single-class selector** is a whole simple selector consisting of exactly one class and
 * nothing else — no type, no id, no attribute, no second class. `.formula` is one; `div.formula`
 * is not, because the type is part of what it takes to match.
 *
 * A **compound class selector** is two or more classes chained with no combinator between them —
 * `.sw.formula` — which an element matches only by carrying every class listed. That is exactly
 * the shape a component's variant modifier takes (`.card.apparatus`), and exactly the shape a
 * coincidence takes (`.sw.formula`). The difference between them is not visible in either
 * selector alone.
 *
 * # How a coincidence is told from a modifier, without being told which is which
 *
 * A modifier class — `.apparatus`, `.correction-index`, every one of the card variants — never
 * gets a bare rule of its own. It exists only chained after `.card`, because it was never meant to
 * style anything by itself. `.formula` and `.sw` are different: **both** halves of `.sw.formula`
 * carry an independent bare rule elsewhere in this file — `.formula` styled a code block on its
 * own, `.sw` styles a ten-pixel box on its own — which is the actual signature of a collision: two
 * classes that each already mean something on their own being chained together means whichever
 * bare rule loads styles the other's elements too, whether anyone intended it or not.
 *
 * So the check below does not ask "is this selector a modifier pattern" — it cannot know that. It
 * asks the narrower, checkable question: does a compound selector chain two classes that **both**
 * also have a bare rule elsewhere in this stylesheet. `.card.apparatus` never trips it, because
 * `.apparatus` has no bare rule to collide with. `.sw.formula` did, until the fix.
 *
 * # The allow-list
 *
 * Two pairs pass through it, `sw.gain` and `sw.loss`. `.gain` and `.loss` are bare rules that set
 * `color` on the signed delta figures used elsewhere on the site (a "+$500" printed in the
 * formula/guarantee hue); `.sw.gain` and `.sw.loss` set `background` on the same two legend
 * swatches `.sw.formula` and `.sw.guarantee` already draw, under different names for a signed
 * chart's legend. Both halves are bare, so the shape is identical to the defect above — but
 * `color` has nothing to paint on an empty `<i>` swatch with no text content, so the collision is
 * real and harmless. Anything added here needs the same sentence: which property leaks, and why it
 * draws nothing on the element it was never meant to reach.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, test } from "vitest";

const CSS = readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8").replace(
  /\/\*[\s\S]*?\*\//g,
  "",
);

/** A pair the check must not flag, and the one-sentence reason it is safe. */
const ALLOW_LIST: Record<string, string> = {
  "gain.sw": "`.gain` only sets `color`, which paints nothing on an empty swatch `<i>`.",
  "loss.sw": "`.loss` only sets `color`, which paints nothing on an empty swatch `<i>`.",
};

/** Strips a pseudo-class or pseudo-element so `.sw.formula:hover` still reads as two classes. */
const stripPseudo = (token: string): string => token.replace(/::?[a-zA-Z-]+(\([^)]*\))?/g, "");

/** The classes a compound token carries, in the order written. `div.formula` yields `["formula"]`. */
const classesOf = (token: string): string[] =>
  [...token.matchAll(/\.[a-zA-Z0-9_-]+/g)].map((m) => m[0]!.slice(1));

/** True for a token that is a class and only a class — no type, id, or attribute alongside it. */
const isBareClassToken = (token: string): boolean =>
  !/^[a-zA-Z]/.test(token) && !token.includes("#") && !token.includes("[") && classesOf(token).length === 1;

interface Collision {
  pair: string;
  selector: string;
}

/**
 * Every compound selector chaining two classes that each also carry an independent bare rule.
 *
 * Selector lists are split on top-level commas and each side handled as its own selector; each
 * selector is then split on combinators into the individual compound tokens a descendant chain is
 * built from. A one-token, one-class, unqualified selector marks that class as bare; a token
 * carrying two or more classes is a compound, and every unordered pair inside it is a candidate.
 */
function findCollisions(css: string): { bareClasses: Set<string>; collisions: Collision[] } {
  const rules = [...css.matchAll(/([^{}]+)\{[^{}]*\}/g)].map(([, selectorList]) => selectorList!.trim());

  const bareClasses = new Set<string>();
  const compounds = new Map<string, Set<string>>();

  for (const selectorList of rules) {
    for (const raw of selectorList.split(",")) {
      const selector = raw.trim();
      if (!selector) continue;
      const tokens = selector
        .split(/\s*[>+~]\s*|\s+/)
        .filter(Boolean)
        .map(stripPseudo);

      if (tokens.length === 1 && isBareClassToken(tokens[0]!)) {
        bareClasses.add(classesOf(tokens[0]!)[0]!);
      }

      for (const token of tokens) {
        const classes = classesOf(token);
        if (classes.length < 2) continue;
        for (let i = 0; i < classes.length; i += 1) {
          for (let j = i + 1; j < classes.length; j += 1) {
            const key = [classes[i], classes[j]].sort().join(".");
            if (!compounds.has(key)) compounds.set(key, new Set());
            compounds.get(key)!.add(selector);
          }
        }
      }
    }
  }

  const collisions: Collision[] = [];
  for (const [pair, selectors] of compounds) {
    const [a, b] = pair.split(".") as [string, string];
    if (!bareClasses.has(a) || !bareClasses.has(b)) continue;
    for (const selector of selectors) collisions.push({ pair, selector });
  }
  return { bareClasses, collisions };
}

describe("the class-collision gate", () => {
  test("finds real component classes to check against, or the parser has drifted", () => {
    // A stylesheet this size declares far more than a handful of standalone classes. If this
    // number collapses, the check below is passing by finding nothing to compare rather than by
    // finding nothing wrong.
    const { bareClasses } = findCollisions(CSS);
    expect(bareClasses.size).toBeGreaterThan(50);
  });

  test("no compound selector chains two classes that each also stand alone, unless allow-listed", () => {
    const { collisions } = findCollisions(CSS);
    const offenders = collisions
      .filter(({ pair }) => !(pair in ALLOW_LIST))
      .map(({ pair, selector }) => `${selector} — bare .${pair.split(".")[0]} and bare .${pair.split(".")[1]} both exist`);
    expect(offenders).toEqual([]);
  });

  test("the allow-list names only pairs the stylesheet actually contains", () => {
    // An allow-list entry for a pair that no longer collides is a stale exemption — the fix that
    // should have deleted it landed and nobody noticed the line was still there.
    const { collisions } = findCollisions(CSS);
    const present = new Set(collisions.map((c) => c.pair));
    for (const pair of Object.keys(ALLOW_LIST)) {
      expect(present.has(pair), `${pair} is allow-listed but does not collide`).toBe(true);
    }
  });

  test("reverting the .formula fix reopens the collision this gate exists to catch", () => {
    // Proof that the gate bites, run in-process rather than by hand: the code-block rule scoped to
    // `div.formula` is stood back down to a bare `.formula`, which is exactly the one-word revert
    // that shipped the original defect, and the gate must fail against it.
    const reverted = CSS.replace(/(^|\n)div\.formula(\s*\{)/, "$1.formula$2");
    expect(reverted, "the replacement above did not match — has div.formula moved or been reworded?").not.toBe(CSS);
    const { collisions } = findCollisions(reverted);
    const offenders = collisions.filter(({ pair }) => !(pair in ALLOW_LIST));
    expect(offenders.length, "reverting div.formula to .formula should reopen a collision").toBeGreaterThan(0);
    expect(offenders.some((o) => o.pair === "formula.sw")).toBe(true);
  });
});
