/**
 * The typeset half of the build, read as bytes.
 *
 * # Why a formula has to be checked in the file and not in the DOM
 *
 * An HTML parser repairs MathML on the way in, which is precisely what hid the worst defect this
 * file exists for: `<mspace width="0.1667em" />` reached `dist/` with its solidus dropped, and a
 * browser then made the rest of the row the empty element's children and painted none of it.
 * `\bigl( 0.6\,C_1 + 0.2\,C_2 + 0.2\,C_3 \bigr) \times C_6` rendered as "= ( 0.6", and every
 * geometric check on the page passed: the row still reported 552px and still sat inside its box.
 * By the time Chromium has a DOM, the evidence is gone.
 *
 * The other three are about what a reader meets rather than about markup — a quantity shown twice
 * under two names, a character the shipped subset cannot draw, a font served off a path that
 * cannot say `immutable` — and each is a property of the whole corpus rather than of a page, so
 * each is a sweep.
 *
 * The browser half is in `wiki.spec.ts`: a brace that stretches, a formula that scrolls inside its
 * own box, a named quantity that reaches the parameter defining it.
 */

import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, sep } from "node:path";

import { describe, expect, test } from "vitest";

import { FONT_PATH, readCmap, readWoff2Tables } from "../../src/lib/math-font.ts";
import { DIST, pages } from "./artefact.ts";

describe("a formula in the build", () => {


  /**
   * A formula that loses half its terms and still looks like a formula.
   *
   * temml serialises `<mspace width="0.1667em" />`. Inside `<math>` an HTML parser is in foreign
   * content and honours that solidus — but every page here is re-serialised by `applySemantics`,
   * which parses with linkedom, and linkedom drops it. What shipped was `<mspace width="…">`
   * unclosed; `mspace` is an empty element, so the browser made the rest of the row its children
   * and painted none of them.
   *
   * `\bigl( 0.6\,C_1 + 0.2\,C_2 + 0.2\,C_3 \bigr) \times C_6` rendered as "= ( 0.6".
   *
   * Nothing measured it as wrong. The row still reported 552px and still sat inside its box, so
   * every geometric check on this page passed while four of five terms were missing. It was found
   * by looking at a screenshot, and the assertion that catches it is structural rather than
   * geometric: an empty MathML element may not have a child.
   */
  test("no empty MathML element in the build has swallowed its siblings", () => {

    /*
     * Checked by ADJACENCY and not by counting, and the difference is the whole test.
     *
     * Counting `<mspace` against `</mspace>` cannot see this bug, verified by trying it: linkedom
     * parses the unclosed tag, makes the rest of the row its children, and re-serialises with a
     * closing tag in the wrong place. The counts balance perfectly on markup that has swallowed
     * four terms. For the same reason the artefact never contains a `/>` to look for — linkedom
     * has already dropped the solidus by the time anything lands in `dist`.
     *
     * An empty element's open tag must be followed immediately by its close tag. That is a
     * statement about structure, and it is one a text scan can still make because the markup is
     * machine-generated and the two are always adjacent.
     */
    const EMPTY = ["mspace", "mprescripts", "none"];
    let formulas = 0;
    const unclosed: string[] = [];
    for (const file of pages()) {
      const page = readFileSync(file, "utf8");
      if (!page.includes("<math")) continue;
      formulas += (page.match(/<math\b/g) ?? []).length;
      for (const tag of EMPTY) {
        for (const match of page.matchAll(new RegExp(`<${tag}\\b[^>]*>`, "g"))) {
          const after = page.slice(match.index + match[0].length);
          if (!after.startsWith(`</${tag}>`)) {
            unclosed.push(
              `${file.slice(DIST.length + 1)}: <${tag}> is followed by ${after.slice(0, 40)}`,
            );
          }
        }
      }
    }
    expect(formulas, "the build carries rendered formulas at all").toBeGreaterThan(0);
    expect(unclosed.slice(0, 5), "an empty MathML element that will eat the rest of its row").toEqual([]);
  });



  /**
   * No page states one calculation twice.
   *
   * `function:` and `function_tex:` are the same arithmetic in two encodings, and for one day the
   * node page rendered both — the aligned ASCII, and the typeset version directly beneath it. That
   * is what settled #203: whatever the checks say about the two agreeing, a reader met the local
   * capacity measure twice on one screen and had to work out they were the same thing.
   *
   * `shownProperties` in `src/lib/prose.ts` now shows the typeset one under the plain name where it
   * exists. This asserts the outcome on the artefact: a property name never appears twice in one
   * table, and `_tex` — which names an encoding, not a quantity — never reaches a reader at all.
   */
  test("a node never shows one property under two names", () => {

    const offenders: string[] = [];
    let tables = 0;
    for (const file of pages(join(DIST, "wiki"))) {
      /*
       * Node pages only — `wiki/<class>/<node>.html`, two segments deep.
       *
       * `wiki/<class>.html` is the class index, and its properties card is the ontology's own
       * schema: "What a formula component may carry", one row per declared property with its type
       * and meaning. `function_tex` belongs there under exactly that name, because an author
       * needs the name to write one. The rule below is about a reader meeting a formula, not
       * about a reader meeting the schema, and the first version of this check conflated them.
       */
      if (file.slice(join(DIST, "wiki").length + 1).split(sep).length !== 2) continue;
      const html = readFileSync(file, "utf8");
      /*
       * The properties card, and only it. Three tables on a node page carry `table.prose`, and the
       * other two are the edge lists — where a repeated name is the point: `ohio-general-assembly`
       * states `enacts` five times and `sourced` twice, one row per act. Scoping by class alone
       * reported those as duplicates, which is the first thing this check did.
       */
      const card = html.match(/<div class="card" id="properties"[\s\S]*?<\/table>/)?.[0];
      for (const [, found] of (card ?? "").matchAll(/<table class="prose">([\s\S]*?)$/g)) {
        const body = found ?? "";
        tables += 1;
        /*
         * One pass over the row headers, not two. Matching `scope="row"` and `colspan="2"`
         * separately counted every spanning property twice — a block value carries both attributes
         * on the same `<th>` — and reported `series_path` as a duplicate of itself on five pages.
         */
        const all = [...body.matchAll(/<th[^>]*scope="row"[^>]*>(?:<code>)?([a-z0-9_]+)/g)].map(
          ([, name]) => name!,
        );
        const where = file.slice(DIST.length + 1);
        for (const name of all.filter((name) => name.endsWith("_tex"))) {
          offenders.push(`${where}: ${name} reached the page under its encoding's name`);
        }
        const seen = new Set<string>();
        for (const name of all) {
          if (seen.has(name)) offenders.push(`${where}: ${name} appears twice in one table`);
          seen.add(name);
        }
      }
    }

    expect(tables, "the wiki carries property tables to check").toBeGreaterThan(50);
    expect(offenders.slice(0, 5)).toEqual([]);
  });



  /**
   * The font is served from the one path that already says `immutable`.
   *
   * #202 asked for a cache rule and the answer was to need none: reaching the file with a relative
   * `url()` from `tokens/typography.css` puts it through Vite, which content-hashes it into
   * `/_astro/` — and `/_astro/*` has held `immutable` since the header block was fixed. Dropping
   * the same file in `public/` instead would have served it unhashed at a fixed path, where
   * `immutable` is not available and it would have revalidated on every visit.
   *
   * `_headers` is unreadable from a page, so this reads the built artefact, like the CSV block.
   */
  test("the maths font is content-hashed into the path marked immutable", () => {
    const stylesheets = readdirSync(join(DIST, "_astro")).filter((name) => name.endsWith(".css"));
    const references = stylesheets.flatMap((name) =>
      [...readFileSync(join(DIST, "_astro", name), "utf8").matchAll(/url\(([^)]*\.woff2)\)/g)].map(
        ([, url]) => url!,
      ),
    );
    expect(references, "the @font-face reaches exactly one font").toHaveLength(1);
    expect(references[0]).toMatch(/^\/_astro\/ohio-math-fallback\.[A-Za-z0-9_-]{8}\.woff2$/);
    expect(existsSync(join(DIST, references[0]!.slice(1))), "the URL points at nothing").toBe(true);

    const headers = readFileSync(join(DIST, "_headers"), "utf8");
    const block = headers.match(/^\/_astro\/\*$\n((?:^ {2}.*$\n?)+)/m)?.[1] ?? "";
    expect(block).toContain("immutable");
  });



  /**
   * And the corpus has not written a character the shipped font cannot draw.
   *
   * The repertoire in `src/lib/math-font.ts` is stated rather than derived from the corpus, which
   * is the safer direction but leaves one gap: a formula using a character outside it renders that
   * character in some other face, and nothing about that looks like a failure. So the corpus is
   * checked against the font instead of the font against the corpus, and the fix when this fails
   * is to widen `REPERTOIRE` and re-run `scripts/subset-math-font.ts --write`.
   *
   * `<annotation>` is skipped: it holds the LaTeX source, it is `display: none`, and its backslashes
   * and braces are not what anybody renders.
   */
  test("every character inside a formula is one the shipped font carries", async () => {

    const font = new Uint8Array(readFileSync(join(import.meta.dirname, "../..", FONT_PATH)));
    const cmap = readCmap(readWoff2Tables(font).get("cmap")!);

    const outside = new Map<number, string>();
    let formulas = 0;
    for (const file of pages()) {
      const html = readFileSync(file, "utf8");
      if (!html.includes("<math")) continue;
      for (const [element] of html.matchAll(/<math[\s\S]*?<\/math>/g)) {
        formulas += 1;
        const text = element
          .replace(/<annotation[\s\S]*?<\/annotation>/g, "")
          .replace(/<[^>]+>/g, "")
          .replace(/&#x([0-9a-f]+);/gi, (_, hex: string) => String.fromCodePoint(parseInt(hex, 16)))
          .replace(/&#(\d+);/g, (_, decimal: string) => String.fromCodePoint(Number(decimal)))
          .replace(/&(amp|lt|gt|quot|apos);/g, (_, name: string) =>
            ({ amp: "&", lt: "<", gt: ">", quot: '"', apos: "'" })[name]!,
          );
        const where = file.slice(DIST.length + 1);
        for (const character of text) {
          const codepoint = character.codePointAt(0)!;
          if (!cmap.has(codepoint)) outside.set(codepoint, `${where}: ${JSON.stringify(character)}`);
        }

        /*
         * And the twin. A single-character `<mi>` with no `mathvariant` gets `text-transform:
         * math-auto`, so the letter in the markup is NOT the codepoint that renders — `<mi>C</mi>`
         * draws U+1D436. Checking the markup alone would pass over a font with no italic block at
         * all, which is the exact subset this suite has to be able to reject.
         */
        for (const [, letter] of element.matchAll(/<mi>([A-Za-z])<\/mi>/g)) {
          const code = letter!.charCodeAt(0);
          const italic =
            letter === "h"
              ? 0x210e // Unicode leaves a hole at U+1D455 and puts italic h in Letterlike Symbols.
              : code <= 0x5a
                ? 0x1d434 + code - 0x41
                : 0x1d44e + code - 0x61;
          if (!cmap.has(italic)) outside.set(italic, `${where}: <mi>${letter}</mi> renders as it`);
        }
      }
    }

    expect(formulas, "the build carries formulas to check at all").toBeGreaterThan(0);
    expect(
      [...outside].map(([codepoint, where]) => `U+${codepoint.toString(16).toUpperCase().padStart(4, "0")} ${where}`),
    ).toEqual([]);
  });
});
