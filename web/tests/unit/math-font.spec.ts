/**
 * The shipped maths font, read back from the bytes that get uploaded.
 *
 * This is the only binary asset on the site and it is the only one nothing else can check. A
 * stylesheet that has lost a rule shows up in a screenshot; a font that has lost its `MATH` table
 * does not. It loads, it reports the right family, it draws every letter correctly, and the single
 * thing it can no longer do is make a delimiter tall — which shows up as a `{` one line high beside
 * a five-line table, on one page, on the machines least likely to be the one you are testing on.
 *
 * So: `src/lib/math-font.ts` parses the file with nothing but the WOFF2 and OpenType specs, and the
 * assertions below are against what it finds there. Deliberately not `subset-font`, which produced
 * it, and deliberately not `fontkit` or `fontTools`, which would need adding. Verification that
 * routes through the producer answers a different question from the one being asked.
 *
 * The browser half is in `tests/e2e/app.spec.ts`, which makes chromium actually stretch a brace
 * with this file. Both halves are needed: this one says the table is there, that one says it works.
 */

import { readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";

import { describe, expect, test } from "vitest";

import {
  CODEPOINTS,
  FAMILY,
  FONT_PATH,
  readCmap,
  readGlyphCount,
  readNames,
  readVerticalConstructions,
  readWoff2Tables,
} from "../../src/lib/math-font.ts";

const path = fileURLToPath(new URL(`../../${FONT_PATH}`, import.meta.url));
const FONT = new Uint8Array(readFileSync(path));
const TABLES = readWoff2Tables(FONT);
const CMAP = readCmap(TABLES.get("cmap")!);
const CONSTRUCTIONS = readVerticalConstructions(TABLES.get("MATH") ?? new Uint8Array());
const GLYPHS = readGlyphCount(TABLES.get("maxp")!);

describe("the file", () => {
  /*
   * A guard on the guard. Every assertion below reads a table through a parser this repository
   * wrote, and a parser that silently returns nothing makes all of them pass over an empty font.
   * `palette.spec.ts` learned this when a round-trip validated an inverse against its own forward
   * transform and agreed with a coefficient that was wrong in the fourth decimal.
   */
  test("parses into something, so nothing below passes vacuously", () => {
    expect(TABLES.size).toBeGreaterThan(10);
    expect(GLYPHS).toBe(361);
    expect(CMAP.size).toBeGreaterThan(170);
    expect(CONSTRUCTIONS.size).toBeGreaterThan(0);
  });

  /*
   * The figure #202 exists to produce, recorded rather than estimated.
   *
   * The source is `@fontsource/stix-two-math`'s only file, 403,344 bytes, and shipping it whole was
   * one of the three options the issue listed. The subset is 6.8% of it. The ceiling is not a
   * target — it is there so a repertoire widened without a thought costs somebody a conversation.
   */
  test("is 27,236 bytes, against 403,344 for the font it came from", () => {
    expect(statSync(path).size).toBe(27_236);
    expect(statSync(path).size).toBeLessThan(60_000);
  });

  test("is a WOFF2 file, which is the only format declared for it", () => {
    expect(String.fromCharCode(...FONT.subarray(0, 4))).toBe("wOF2");
  });
});

describe("the MATH table", () => {
  test("survived the subsetter", () => {
    // The whole reason this file exists. fontTools' subsetter keeps `MATH` and closes over the
    // variant glyphs; harfbuzz's does too, measured. Neither promises to, and a subsetter that
    // stopped would leave a font that looks entirely fine.
    expect(TABLES.has("MATH")).toBe(true);
  });

  /*
   * Every delimiter the corpus can reach, and what it needs to grow.
   *
   * Two mechanisms, both required. `variants` are whole pre-drawn glyphs at fixed larger sizes,
   * used up to about four lines. Beyond that a renderer stops finding one and starts stacking
   * `parts` — the brace's five pieces are top, upper arm, middle pinch, lower arm, foot. A
   * construction with variants but no assembly stops growing at the largest variant, which for the
   * `cases` block in `fsfp-local-capacity-measure` is exactly where it would have to keep going.
   */
  test.each([
    ["{", 13, 5],
    ["}", 13, 5],
    ["(", 13, 3],
    [")", 13, 3],
    ["[", 13, 3],
    ["]", 13, 3],
  ])("%s can grow: %i variants and %i assembly parts", (character, variants, parts) => {
    const glyph = CMAP.get(character.codePointAt(0)!);
    expect(glyph, `${character} is not in the font at all`).toBeDefined();
    const construction = CONSTRUCTIONS.get(glyph!);
    expect(construction, `${character} has no vertical construction`).toBeDefined();
    expect(construction!.variants).toHaveLength(variants);
    expect(construction!.parts).toHaveLength(parts);
  });

  /*
   * The failure mode a count cannot see.
   *
   * A construction naming thirteen variants whose glyphs were dropped is worse than no construction
   * at all: the table reads as intact, the renderer asks for glyph 4,812 in a font with 357, and
   * what it draws is undefined. This is the same shape as the `<mspace>` bug from #200 — the
   * structure was present and balanced, and the contents were gone.
   */
  test("every glyph a construction points at is actually in the font", () => {
    const dangling = (constructions: typeof CONSTRUCTIONS, count: number): string[] =>
      [...constructions].flatMap(([glyph, { variants, parts }]) =>
        [...variants, ...parts].filter((id) => id >= count).map((id) => `${glyph} -> ${id}`),
      );

    expect(dangling(CONSTRUCTIONS, GLYPHS)).toEqual([]);

    /*
     * Verified to bite, permanently rather than once.
     *
     * The other checks in this file were each broken by hand to prove they fire — the MATH table
     * stripped, the italic block dropped — and this one could not be, because forging a font whose
     * records outlive their glyphs means writing a WOFF2 encoder to do it. So the predicate is
     * asserted against a construction that dangles by construction. Without this the assertion
     * above would pass just as happily over a bug in the comparison as over a sound font.
     */
    expect(dangling(new Map([[7, { variants: [GLYPHS], parts: [] }]]), GLYPHS)).toHaveLength(1);
  });
});

describe("the repertoire", () => {
  test("every codepoint the subset was built for is in the cmap", () => {
    const missing = CODEPOINTS.filter((codepoint) => !CMAP.has(codepoint)).map(
      (codepoint) => `U+${codepoint.toString(16).toUpperCase().padStart(4, "0")}`,
    );
    expect(missing).toEqual([]);
  });

  /*
   * Mathematical Italic Latin, which is not a nicety.
   *
   * MathML Core gives a single-character `<mi>` `text-transform: math-auto`, and chromium
   * implements that by mapping the character into this block. `<mi>C</mi>` renders U+1D436, never
   * U+0043 — so a subset with the whole of ASCII and none of this block has no glyph for any
   * variable in any formula on the site.
   *
   * What that looks like, measured through CDP's `CSS.getPlatformFontsForNode` against a subset
   * built without the block:
   *
   *     mi     STIX Two Math          <- left the font entirely
   *     mtext  Probe NoItal Math
   *     mn     Probe NoItal Math
   *
   * On the machine that built it, that is invisible: the fallback IS the same typeface, installed.
   * On the reader this font exists for — no maths face at all — the variables render in a generic
   * serif and everything around them in STIX. 6,336 bytes, and it only fails where nobody looks.
   */
  test("carries the italic block a variable actually renders as", () => {
    expect(CMAP.has(0x1d436), "italic C, which is what <mi>C</mi> is").toBe(true);
    expect(CMAP.has(0x1d44e), "italic a").toBe(true);
    // U+1D455 is a permanent hole in Unicode; italic h lives at U+210E and is easy to forget.
    expect(CMAP.has(0x1d455), "the reserved hole in the block").toBe(false);
    expect(CMAP.has(0x210e), "italic h, which is not where the block would put it").toBe(true);
  });
});

describe("what the font says it is", () => {
  const NAMES = readNames(TABLES.get("name")!);

  /*
   * The rename is functional before it is legal.
   *
   * `--font-math` names "STIX Two Math" for the readers who have it. An @font-face declaring that
   * family shadows the local font for all of them, so every macOS reader would fetch this 27 KB
   * subset and lose the other 4,800 glyphs — a fallback that displaces the thing it is a fallback
   * for. The trademark is the second reason and it points the same way.
   */
  test("is not named after the font it was cut from", () => {
    expect(NAMES.get(1)).toBe(FAMILY);
    expect(NAMES.get(1)).not.toContain("STIX");
    expect(NAMES.get(6), "the PostScript name may not carry a space").toBe("OhioMathFallback-Regular");
  });

  /*
   * The OFL requires the copyright notice and the licence to travel with the font, and harfbuzz
   * drops every name id above 6. `src/assets/fonts/OFL.txt` covers the repository; these records
   * cover the file, which is the copy that leaves it.
   */
  test("carries its copyright and its licence, which the subsetter dropped", () => {
    expect(NAMES.get(0)).toContain("STIX Fonts Project Authors");
    expect(NAMES.get(13)).toContain("SIL Open Font License");
    expect(NAMES.get(14)).toContain("scripts.sil.org/OFL");
  });

  test("says in its own description what it is and where it came from", () => {
    expect(NAMES.get(10)).toContain("subset of STIX Two Math");
    expect(NAMES.get(3)).toContain("STIX Two Math");
  });
});
