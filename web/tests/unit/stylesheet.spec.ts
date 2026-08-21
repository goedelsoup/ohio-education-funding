/**
 * The design system's rules, as checks against the stylesheet.
 *
 * Its README states fifteen and says of them: "these are stated so a check can be run against the
 * built HTML/CSS". Three are checkable against the source stylesheet alone and are checked here.
 * The rest need the built DOM and belong with the end-to-end suite, or need markup this site does
 * not emit yet.
 *
 * A rule with no check is a preference. That is the whole reason for this file.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { expect, test } from "vitest";

/**
 * The stylesheet with its comments removed.
 *
 * Stripped, because this file's checks scan for shapes — `.claim.unentered`, `box-shadow`,
 * `--series-3` — and this stylesheet explains itself at length. The first run of the
 * status-class check failed on a sentence in a comment saying that `.claim.unentered` used to
 * exist, which is a true sentence and not a rule. A check that reads prose as code will keep
 * finding what the prose is about.
 */
const CSS = readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8").replace(
  /\/\*[\s\S]*?\*\//g,
  "",
);

/** Every declaration block whose selector matches, as `[selector, body]`. */
function rules(pattern: RegExp): [string, string][] {
  return [...CSS.matchAll(/([^{}]+)\{([^{}]*)\}/g)]
    .map(([, selector, body]) => [selector!.trim(), body!.trim()] as [string, string])
    .filter(([selector]) => pattern.test(selector));
}

/**
 * Rule 7: a claim mark has no box, and there are exactly three of them.
 *
 * The box is the thing that made four marks in a sentence read as a field of badges. A border or a
 * background creeping back is how it returns — and a fourth status class is how the fourth
 * epistemic state gets back onto an axis it does not belong on.
 */
test("a claim mark carries no border, background or radius", () => {
  // Declaration by declaration, not block by block. The first version filtered out any rule whose
  // body mentioned `border-bottom` — which `.claim`'s does, because the rule under the text IS the
  // mark — and so exempted the whole block. Adding `background` beside it passed. A check that
  // cannot fail on the defect it names is not a check, and this one was caught only by
  // reintroducing the box deliberately and watching it stay green.
  const BOXY = /^(border|background|background-color|border-radius|border-top|border-left|border-right|box-shadow)$/;
  const offenders: string[] = [];
  for (const [selector, body] of rules(/(^|,|\s)\.claim(\.|,|\s|$)/)) {
    if (selector.includes("claim-detail")) continue;
    for (const declaration of body.split(";")) {
      const property = declaration.split(":")[0]?.trim() ?? "";
      if (BOXY.test(property)) offenders.push(`${selector} { ${property} }`);
    }
  }
  expect(offenders).toEqual([]);
});

test("there are exactly three claim status classes", () => {
  const statuses = new Set(
    [...CSS.matchAll(/\.claim\.([a-z]+)/g)].map(([, status]) => status!),
  );
  expect([...statuses].sort()).toEqual(["inference", "open", "verified"]);
});

/**
 * Rule 15: no shadow on a card, a tile, or a table.
 *
 * Elevation belongs to things that float over the page — a menu panel, a tooltip. A page of forty
 * raised rectangles reads as forty competing objects rather than as one document.
 */
test("nothing that sits on the page carries a shadow", () => {
  const offenders = rules(/(^|,|\s)\.(card|tile)(\.|,|\s|$)|(^|,|\s)table(\s|,|$)/)
    .filter(([, body]) => /box-shadow\s*:\s*(?!none)/.test(body))
    .map(([selector]) => selector);
  expect(offenders).toEqual([]);
});

/**
 * Rule 4: no third categorical series.
 *
 * Only the formula/guarantee pair and the neutral mark may encode identity, and nothing generates
 * a palette. A `--series-3` appearing is the shape this catches.
 */
test("identity is encoded by two series and a neutral, and nothing else", () => {
  const series = new Set(
    [...CSS.matchAll(/--series-([a-z0-9]+)/g)].map(([, name]) => name!),
  );
  expect([...series].sort()).toEqual(["formula", "guarantee"]);
});
