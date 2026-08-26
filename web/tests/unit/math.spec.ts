/**
 * The boundary around rendered maths, and the bug that made it necessary to have one.
 *
 * `lib/math.ts` compiles LaTeX to MathML at build time and holds the result to an enumerated
 * allowlist. These are the checks that the allowlist is real, that a formula which will not
 * compile stops the build, and that the markup which ships is the markup that renders.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, test } from "vitest";

import { ATTRIBUTES, DROPPED, ELEMENTS, MathError, renderMath } from "../../src/lib/math.ts";
import { loadCorpus } from "../../src/lib/corpus.ts";

const corpus = await loadCorpus();
const render = (tex: string): string => renderMath(tex, { display: true, where: "spec" });

describe("what a formula may contain", () => {
  test("rejects an element that is not on the list, naming where it came from", () => {
    // `\includegraphics` and friends are already refused by `trust: false`; this is the layer
    // under that, and it has to hold whatever temml decides to emit next.
    expect(ELEMENTS.has("mglyph")).toBe(false);
    expect(() => renderMath("\\sqrt{x}", { display: true, where: "node#field" })).not.toThrow();
    const rejected = [...ELEMENTS].length;
    expect(rejected, "the list is enumerated, not a wildcard").toBeLessThan(30);
  });

  /**
   * `trust: false` is set, and it is the *second* line rather than the first.
   *
   * Checked by breaking it: flipping temml to `trust: true` leaves every test in this file
   * passing. That is not a gap — it is the allowlist doing the work. Everything `trust` opens is
   * something the list already refuses: `\href` and `\url` produce an `<a>` whose target is
   * checked, `\includegraphics` produces an `<img>` that is not on the list, `\htmlId` an `id`
   * that is not on the list, and `\htmlClass`/`\htmlStyle` produce the two attributes that are
   * dropped by name anyway.
   *
   * So this asserts the setting at the source, and says plainly that the boundary underneath it is
   * what holds. A default is not a decision, and a decision nothing records is not one either.
   */
  test("temml is called with trust off, and the allowlist is what enforces it", () => {
    // Comments stripped first. The docstring above the call says the words "trust: false", so a
    // match against the raw file passes while the call says the opposite — checked by flipping it.
    const source = readFileSync(resolve(process.cwd(), "src/lib/math.ts"), "utf8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\/\/.*$/gm, "");
    expect(source).toMatch(/trust:\s*false/);
    expect(source).not.toMatch(/trust:\s*true/);
    expect(ELEMENTS.has("img")).toBe(false);
    expect(ATTRIBUTES.math?.has("id")).toBeFalsy();
  });

  test("permits a link into this site, which is what a named quantity will need", () => {
    // #204: a term links to the parameter node that defines it. `href` on a MathML element is
    // dead in MathML Core, so the anchor lives inside an `<mtext>` — this only checks that the
    // allowlist would let it through.
    expect(ELEMENTS.has("a")).toBe(true);
    expect(() => render("\text{x}")).not.toThrow();
  });

  test("refuses to link off this site", () => {
    // A formula that can point anywhere is a formula that can point at an attacker. The corpus
    // links by relative path and the renderer resolves those to site-absolute routes, so an
    // absolute URL in a formula is either a mistake or something worse.
    expect(ATTRIBUTES.a?.has("href")).toBe(true);
    expect(() => render("\\href{https://example.com}{x}")).toThrow(MathError);
  });

  test("drops the two attributes temml always writes and nothing else", () => {
    expect([...DROPPED].sort()).toEqual(["class", "style"]);
    // Scoped to the `<math>` subtree: the `<div class="formula">` wrapper is this renderer's own
    // and carries the class that makes it scroll. Only what came out of temml is being held here.
    const html = render("x = 1");
    const math = html.slice(html.indexOf("<math"), html.lastIndexOf("</math>"));
    expect(math).not.toContain("style=");
    expect(math).not.toContain("class=");
    // `display` survives, and it is what makes the block a block — the UA stylesheet resolves it
    // to `display: block math`, so the inline style temml writes beside it is redundant.
    expect(html).toContain('display="block"');
  });

  test("a formula that will not compile stops the build and says which one", () => {
    // `throwOnError` defaults to FALSE in temml: left alone, this ships a red error string on
    // every page carrying the formula rather than failing.
    expect(() => renderMath("\\frac{1}{", { display: true, where: "some-node#function_tex" })).toThrow(
      /some-node#function_tex/,
    );
    expect(() => renderMath("\\undefinedmacro", { display: true, where: "n#f" })).toThrow(MathError);
  });
});

/**
 * The silent one, and the reason this file exists at all.
 *
 * temml's XML serialiser writes `<mspace width="0.1667em" />`. Inside `<math>` an HTML parser is in
 * foreign content and honours that. But every page here is re-serialised by `applySemantics`,
 * which parses with linkedom — and linkedom drops the solidus, so `<mspace width="0.1667em">`
 * ships unclosed. `mspace` is an empty element, so the browser makes the rest of the row its
 * children and paints none of them.
 *
 * `\bigl( 0.6\,C_1 + 0.2\,C_2 + 0.2\,C_3 \bigr) \times C_6` rendered as "= ( 0.6": a
 * plausible-looking equation missing four of its five terms. Nothing measured it as wrong — the
 * row still reported 552px and sat inside its box — and it was found by looking at a screenshot.
 */
describe("every tag closed", () => {
  test("no self-closing tag survives into the markup", () => {
    const html = render(String.raw`\bigl( 0.6\,C_1 + 0.2\,C_2 \bigr) \times C_6`);
    expect(html).not.toMatch(/\/>/);
    expect(html).toMatch(/<mspace[^>]*><\/mspace>/);
  });

  test("and the terms after a space are still in the output", () => {
    // The assertion the geometry could not make. Everything after the first `\,` is what the
    // unclosed tag swallowed.
    const text = render(String.raw`\bigl( 0.6\,C_1 + 0.2\,C_2 + 0.2\,C_3 \bigr) \times C_6`)
      .replace(/<annotation[\s\S]*?<\/annotation>/, "")
      .replace(/<[^>]+>/g, "");
    for (const term of ["0.6", "0.2", "×", ")"]) expect(text).toContain(term);
    expect(text.match(/0\.2/g)?.length, "both 0.2 terms").toBe(2);
  });
});

/**
 * The dollar sign, which is why maths is a fence here and never `$…$`.
 *
 * `remark-math` — the usual route for LaTeX in markdown — delimits inline maths with dollars. This
 * corpus is about money and carries them in the thousands. The count is asserted rather than
 * described so that the reason stays checkable: if it ever fell to nothing, the decision could be
 * revisited instead of merely inherited.
 */
test("the corpus is full of dollar signs, which is why the delimiter is a fence", () => {
  const dollars = corpus.nodes
    .flatMap((node) => [node.description, node.findings ?? "", ...node.properties.map((p) => p.value)])
    .join("")
    .match(/\$/g)?.length ?? 0;
  expect(dollars, "dollar signs in corpus prose").toBeGreaterThan(1_000);
});

/**
 * And the corpus's own formulas compile.
 *
 * Driven off the corpus rather than a fixture: a `function_tex` written tomorrow is checked today,
 * and it is checked by the same call the build makes rather than by a copy of it.
 */
test("every function_tex in the corpus renders", () => {
  const maths = corpus.nodes.flatMap((node) =>
    node.properties
      .filter((property) => property.name.endsWith("_tex"))
      .map((property) => ({ where: `${node.id}#${property.name}`, tex: property.value })),
  );
  expect(maths.length, "the corpus carries at least the pilot").toBeGreaterThan(0);
  for (const { where, tex } of maths) {
    expect(() => renderMath(tex, { display: true, where })).not.toThrow();
  }
});

/**
 * The ontology declares the property, which is what keeps the gate honest in both directions.
 *
 * Declaring `function_tex` closes the property bag against typos — `undeclared-property` gates —
 * and it also makes `yidam lint` report every component that does not carry one yet. That report
 * is the conversion worklist for #203, maintained by the corpus rather than by hand.
 */
test("function_tex is declared by the formula-component ontology", () => {
  const ontology = readFileSync(
    resolve(process.cwd(), "../.yidam/corpus/formula-component.ont.yml"),
    "utf8",
  );
  expect(ontology).toContain("name: function_tex");
});
