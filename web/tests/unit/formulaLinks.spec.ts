/**
 * A named quantity in a formula links to the parameter node that defines it.
 *
 * # The rule, and the measurement that changed how it is derived
 *
 * #204's requirement is that a formula carries **no hand-written link**: the corpus states the
 * mathematics and the site derives the navigation, so a renamed parameter node breaks the build
 * like every other internal link rather than leaving a stale `href` inside a `function_tex`.
 *
 * The issue proposed deriving it from a parameter's `name` property. Measured across the corpus's
 * fourteen `function_tex` fields: **152 distinct `\text{}` phrases, of which exactly one equals a
 * parameter's `name`**. A formula does not write "Special education category multiples"; it writes
 * `w_k`, and where it names a quantity in words it uses the section's words — "statewide average
 * base cost per pupil", "the school-age weights", "general proration".
 *
 * So the vocabulary is declared on the parameter, as `written_as`, and the formula stays clean.
 * That preserves the property the issue was protecting and moves the authoring to the one place
 * where a phrase is a fact about the parameter rather than about the formula quoting it.
 *
 * # Two directions, because a vocabulary rots in both
 *
 * A phrase no formula writes is a link that will never render, and it reads as coverage. A formula
 * phrase that ought to link and does not is invisible — there is nothing on the page to notice.
 * The first is checkable and is checked here. The second is not checkable in general, which is
 * what `formulaConstants.spec.ts` covers from the other side: every *constant* a formula states
 * must have a parameter node behind it.
 */

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, test } from "vitest";
import { DOMParser } from "linkedom";

import { linkTerms } from "../../src/lib/math.ts";

const CORPUS = join(import.meta.dirname, "../../../.yidam/corpus");

function read(dir: string): Map<string, string> {
  return new Map(
    readdirSync(join(CORPUS, dir))
      .filter((name) => name.endsWith(".yml"))
      .map((name) => [name.slice(0, -".yml".length), readFileSync(join(CORPUS, dir, name), "utf8")]),
  );
}

function block(source: string, field: string): string | null {
  return new RegExp(`^  ${field}: \\|\\n((?:    .*\\n|\\n)+)`, "m").exec(source)?.[1] ?? null;
}

const components = read("formula-component");
const parameters = read("parameter");

test("no formula carries a hand-written link", () => {
  /*
   * The property #204 asked for, and the reason it asked: a `function_tex` is corpus prose, and a
   * link typed into one is an `href` no build check reaches. `check-dist-links.ts` walks every
   * anchor in `dist/`, so a *derived* link is covered by the sweep that already exists — which is
   * only true while nothing is authored by hand.
   */
  const authored: string[] = [];
  for (const [name, source] of components) {
    const tex = block(source, "function_tex");
    if (tex === null) continue;
    if (/\\href|\\url|href=/.test(tex)) authored.push(`formula-component/${name}`);
  }
  expect(authored, "a link typed into a formula: declare the phrase in the parameter's `written_as` instead").toEqual([]);
});

test("every phrase a parameter is written as is written by a formula that declares it", () => {
  const unused: string[] = [];
  const linked: string[] = [];

  for (const [name, source] of parameters) {
    const declared = block(source, "written_as");
    if (declared === null) continue;
    for (const line of declared.split("\n")) {
      const phrase = line.trim();
      if (phrase === "") continue;

      const writers = [...components].filter(([, component]) => {
        const tex = block(component, "function_tex");
        if (tex === null) return false;
        const governs = new RegExp(
          `- target: \\.\\./parameter/${name}\\.yml\\n\\s+relationship: governed-by`,
        ).test(component);
        if (!governs) return false;
        return [...tex.matchAll(/\\text\{([^}]*)\}/g)].some(
          ([, written]) => written!.trim().toLowerCase() === phrase.toLowerCase(),
        );
      });

      if (writers.length === 0) unused.push(`${name}: "${phrase}"`);
      for (const [component] of writers) linked.push(`${component} → ${name}: ${phrase}`);
    }
  }

  expect(
    unused,
    "a phrase no formula writes under an edge to this parameter — the formula was reworded, or the edge is missing",
  ).toEqual([]);

  /* The floor is a floor and not a pin. It says the mechanism reaches most of the plan rather
     than one formula, and it does not have to move every time a component is reworded. */
  expect(linked.length, "the phrases reach this many formula terms").toBeGreaterThan(20);
});

test("a term is linked whole, or not at all", () => {
  /*
   * The substring case, which would be the natural implementation and is wrong. `\text{state
   * share}` must not pick up a link because some parameter is written as "share": a rule drawn
   * through half a named quantity tells a reader the wrong thing about where the quantity ends.
   *
   * And the spaces are the other half. temml writes the gaps inside `\text{}` as U+00A0, so a
   * lookup on the raw text matched only single-word phrases — five of twenty-eight, every one of
   * them a word with no gap in it. Normalised for the lookup, preserved in the output.
   */
  const parse = (markup: string) => {
    const document = new DOMParser().parseFromString(`<math>${markup}</math>`, "text/xml");
    return { document, root: document.querySelector("math")! };
  };
  const terms = new Map([["base cost per pupil", "/wiki/parameter/base-cost-per-pupil"]]);

  const exact = parse("<mtext>base cost per pupil</mtext>");
  expect(linkTerms(exact.root as never, terms, exact.document as never)).toBe(1);
  expect(exact.root.querySelector("a")?.getAttribute("href")).toBe(
    "/wiki/parameter/base-cost-per-pupil",
  );
  // The non-breaking spaces survive: a named quantity that wraps mid-phrase is one the reader has
  // to reassemble.
  expect(exact.root.querySelector("a")?.textContent).toBe("base cost per pupil");

  const partial = parse("<mtext>statewide base cost per pupil index</mtext>");
  expect(linkTerms(partial.root as never, terms, partial.document as never)).toBe(0);
  expect(partial.root.querySelector("a")).toBeNull();

  // Nothing to link is not an error, and an empty map short-circuits.
  const none = parse("<mtext>state share</mtext>");
  expect(linkTerms(none.root as never, terms, none.document as never)).toBe(0);
  expect(linkTerms(none.root as never, new Map(), none.document as never)).toBe(0);
});
