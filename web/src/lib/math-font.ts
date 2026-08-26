/**
 * Reading the shipped math font, without the library that produced it.
 *
 * `scripts/subset-math-font.ts` builds `src/assets/fonts/ohio-math-fallback.woff2` with harfbuzz,
 * through `subset-font`. This module reads that file back with nothing but the WOFF2 and OpenType
 * specifications, and `tests/unit/math-font.spec.ts` asserts against what it finds.
 *
 * The separation is the point. The defect this file exists to catch is a subsetter that drops the
 * `MATH` table or the vertical glyph assemblies inside it — and a font missing them still loads,
 * still reports the right family name, still draws every letter correctly, and only fails when a
 * delimiter is asked to stretch. Verifying that with the same wasm build that did the subsetting
 * asks the accused to testify: whatever harfbuzz thinks it wrote is what harfbuzz will read back.
 *
 * So: a table directory walker, a coverage-table reader, and enough of `MATH` to answer one
 * question — does the brace still know how to be tall.
 *
 * # WOFF2, briefly
 *
 * A WOFF2 file is a 48-byte header, a table directory of variable-width records, and one brotli
 * stream holding every table back to back in directory order with no padding between them. The
 * directory record is a flag byte — six bits of index into a fixed table of 63 known tags, two
 * bits of transform version — then the original length as UIntBase128, then the transformed length,
 * present only when the table is actually transformed.
 *
 * Only `glyf`, `loca` and `hmtx` have transforms defined, and the two this reader wants — `cmap`
 * and `MATH` — are stored verbatim. That is what makes a parser this small possible.
 */

import { brotliDecompressSync } from "node:zlib";

/** The 63 tags a WOFF2 directory can name by index rather than spelling out. Order is normative. */
const KNOWN_TABLES = [
  "cmap", "head", "hhea", "hmtx", "maxp", "name", "OS/2", "post", "cvt ", "fpgm",
  "glyf", "loca", "prep", "CFF ", "VORG", "EBDT", "EBLC", "gasp", "hdmx", "kern",
  "LTSH", "PCLT", "VDMX", "vhea", "vmtx", "BASE", "GDEF", "GPOS", "GSUB", "EBSC",
  "JSTF", "MATH", "CBDT", "CBLC", "COLR", "CPAL", "SVG ", "sbix", "acnt", "avar",
  "bdat", "bloc", "bsln", "cvar", "fdsc", "feat", "fmtx", "fvar", "gvar", "hsty",
  "just", "lcar", "mort", "morx", "opbd", "prop", "trak", "Zapf", "Silf", "Glat",
  "Gloc", "Feat", "Sill",
] as const;

/** A cursor over a buffer. Every reader below is a sequence of big-endian reads at a position. */
class Reader {
  private at = 0;
  private readonly data: Uint8Array;

  constructor(data: Uint8Array) {
    this.data = data;
  }

  get offset(): number {
    return this.at;
  }

  seek(to: number): void {
    this.at = to;
  }

  u8(): number {
    return this.data[this.at++]!;
  }

  u16(): number {
    const value = (this.data[this.at]! << 8) | this.data[this.at + 1]!;
    this.at += 2;
    return value;
  }

  u32(): number {
    const value =
      this.data[this.at]! * 0x1000000 +
      ((this.data[this.at + 1]! << 16) | (this.data[this.at + 2]! << 8) | this.data[this.at + 3]!);
    this.at += 4;
    return value;
  }

  tag(): string {
    const value = String.fromCharCode(...this.data.subarray(this.at, this.at + 4));
    this.at += 4;
    return value;
  }

  /** WOFF2's variable-length integer: seven bits a byte, high bit continues. */
  base128(): number {
    let value = 0;
    for (let i = 0; i < 5; i += 1) {
      const byte = this.u8();
      value = value * 128 + (byte & 0x7f);
      if ((byte & 0x80) === 0) return value;
    }
    throw new Error("UIntBase128 ran past five bytes");
  }
}

/**
 * Every table in a WOFF2 file, by tag, uncompressed.
 *
 * `glyf` and `loca` come back in their *transformed* form and are not usable as OpenType tables —
 * decoding that transform is a font library's job and nothing here needs the outlines. Every other
 * table, including the two this module reads, is byte-identical to what an `.otf` would carry.
 */
export function readWoff2Tables(file: Uint8Array): Map<string, Uint8Array> {
  const header = new Reader(file);
  if (header.tag() !== "wOF2") throw new Error("not a WOFF2 file: signature is not wOF2");
  header.seek(12);
  const count = header.u16();
  header.seek(48);

  const directory: { tag: string; length: number }[] = [];
  for (let i = 0; i < count; i += 1) {
    const flags = header.u8();
    const index = flags & 0x3f;
    const transform = flags >> 6;
    const tag = index === 0x3f ? header.tag() : KNOWN_TABLES[index]!;
    const original = header.base128();
    // `glyf`/`loca` invert the convention: 0 is the transform and 3 is the null transform.
    const transformed = tag === "glyf" || tag === "loca" ? transform !== 3 : transform !== 0;
    directory.push({ tag, length: transformed ? header.base128() : original });
  }

  // Node's brotli is the same decoder a browser uses, so a file this reads is a file that loads.
  const stream = new Uint8Array(brotliDecompressSync(file.subarray(header.offset)));

  const tables = new Map<string, Uint8Array>();
  let at = 0;
  for (const { tag, length } of directory) {
    tables.set(tag, stream.subarray(at, at + length));
    at += length;
  }
  return tables;
}

/** The glyph ids a coverage table covers, in coverage order — which is the order records use. */
export function readCoverage(data: Uint8Array, offset: number): number[] {
  const reader = new Reader(data);
  reader.seek(offset);
  const format = reader.u16();
  const count = reader.u16();
  if (format === 1) return Array.from({ length: count }, () => reader.u16());
  if (format !== 2) throw new Error(`coverage format ${format} is neither 1 nor 2`);
  const glyphs: number[] = [];
  for (let i = 0; i < count; i += 1) {
    const start = reader.u16();
    const end = reader.u16();
    const first = reader.u16();
    for (let glyph = start; glyph <= end; glyph += 1) glyphs.push(first + glyph - start);
  }
  return glyphs;
}

/** Codepoint to glyph id, from the Unicode subtables of `cmap`. */
export function readCmap(cmap: Uint8Array): Map<number, number> {
  const reader = new Reader(cmap);
  reader.seek(2);
  const count = reader.u16();
  const offsets: number[] = [];
  for (let i = 0; i < count; i += 1) {
    reader.u16();
    reader.u16();
    offsets.push(reader.u32());
  }

  const map = new Map<number, number>();
  for (const offset of offsets) {
    reader.seek(offset);
    const format = reader.u16();
    if (format === 4) {
      reader.seek(offset + 6);
      const segments = reader.u16() / 2;
      const ends = { at: offset + 14 };
      const starts = { at: ends.at + segments * 2 + 2 };
      const deltas = { at: starts.at + segments * 2 };
      const rangeOffsets = { at: deltas.at + segments * 2 };
      for (let segment = 0; segment < segments; segment += 1) {
        reader.seek(ends.at + segment * 2);
        const end = reader.u16();
        reader.seek(starts.at + segment * 2);
        const start = reader.u16();
        reader.seek(deltas.at + segment * 2);
        const delta = reader.u16();
        reader.seek(rangeOffsets.at + segment * 2);
        const rangeOffset = reader.u16();
        if (start === 0xffff) continue;
        for (let code = start; code <= end && code !== 0x10000; code += 1) {
          let glyph: number;
          if (rangeOffset === 0) glyph = (code + delta) & 0xffff;
          else {
            reader.seek(rangeOffsets.at + segment * 2 + rangeOffset + (code - start) * 2);
            glyph = reader.u16();
            if (glyph !== 0) glyph = (glyph + delta) & 0xffff;
          }
          if (glyph !== 0) map.set(code, glyph);
        }
      }
    } else if (format === 12) {
      reader.seek(offset + 12);
      const groups = reader.u32();
      for (let group = 0; group < groups; group += 1) {
        const start = reader.u32();
        const end = reader.u32();
        const glyph = reader.u32();
        for (let code = start; code <= end; code += 1) map.set(code, glyph + code - start);
      }
    }
  }
  return map;
}

/** How one glyph grows: the discrete sizes it has, and the parts it is built from beyond them. */
export interface Construction {
  /** Whole pre-drawn glyphs at increasing sizes. Empty means the glyph cannot step up at all. */
  variants: number[];
  /** The pieces a renderer stacks when no variant is tall enough. Empty means no assembly. */
  parts: number[];
}

/**
 * `MATH`'s vertical glyph constructions, keyed by the glyph they belong to.
 *
 * This is the table that makes a delimiter stretchy, and it is the one a naive subsetter throws
 * away. Two things have to survive it: the records themselves, and the glyphs they point at — a
 * construction naming thirteen variants that are no longer in the font is worse than no
 * construction, because it reads as intact.
 */
export function readVerticalConstructions(math: Uint8Array): Map<number, Construction> {
  const reader = new Reader(math);
  reader.seek(8);
  const variantsOffset = reader.u16();
  if (variantsOffset === 0) return new Map();

  reader.seek(variantsOffset + 2);
  const coverageOffset = reader.u16();
  reader.seek(variantsOffset + 6);
  const count = reader.u16();
  reader.seek(variantsOffset + 10);
  const constructions = Array.from({ length: count }, () => reader.u16());

  const glyphs = coverageOffset === 0 ? [] : readCoverage(math, variantsOffset + coverageOffset);
  const out = new Map<number, Construction>();
  glyphs.forEach((glyph, index) => {
    const at = variantsOffset + constructions[index]!;
    reader.seek(at);
    const assemblyOffset = reader.u16();
    const variantCount = reader.u16();
    const variants: number[] = [];
    for (let i = 0; i < variantCount; i += 1) {
      variants.push(reader.u16());
      reader.u16(); // advance measurement; the glyph id is the only part that can dangle
    }
    const parts: number[] = [];
    if (assemblyOffset !== 0) {
      reader.seek(at + assemblyOffset + 4); // past the italics correction MathValueRecord
      const partCount = reader.u16();
      for (let i = 0; i < partCount; i += 1) {
        parts.push(reader.u16());
        reader.seek(reader.offset + 8); // connector lengths, full advance, part flags
      }
    }
    out.set(glyph, { variants, parts });
  });
  return out;
}

/** The name records, by name id, from the Windows platform strings a browser actually matches on. */
export function readNames(name: Uint8Array): Map<number, string> {
  const reader = new Reader(name);
  reader.seek(2);
  const count = reader.u16();
  const storage = reader.u16();
  const names = new Map<number, string>();
  for (let i = 0; i < count; i += 1) {
    reader.seek(6 + i * 12);
    const platform = reader.u16();
    reader.seek(6 + i * 12 + 6);
    const id = reader.u16();
    const length = reader.u16();
    const offset = reader.u16();
    const raw = name.subarray(storage + offset, storage + offset + length);
    if (platform !== 3) continue;
    let text = "";
    for (let at = 0; at + 1 < raw.length; at += 2) text += String.fromCharCode((raw[at]! << 8) | raw[at + 1]!);
    names.set(id, text);
  }
  return names;
}

/** How many glyphs the font has, so a record pointing past the end can be caught. */
export function readGlyphCount(maxp: Uint8Array): number {
  return (maxp[4]! << 8) | maxp[5]!;
}

/**
 * The family the subset is renamed to, and it has to be renamed twice over.
 *
 * Functionally: `--font-math` already names "STIX Two Math" for the readers who have it installed.
 * An `@font-face` declaring that same family would shadow the local font for every one of them, so
 * a macOS reader would download this 27 KB subset and lose the other 5,000 glyphs — the exact
 * inverse of what a fallback is for. A distinct family, listed last, is never fetched by anyone
 * whose platform already answered.
 *
 * Legally: the OFL text in this font carries no Reserved Font Name, so a modified version is
 * permitted to keep the original name. Its `name` table also says "STIX Fonts and STIX Two are
 * trademarks of The Institute of Electrical and Electronics Engineers, Inc." A subset with four
 * fifths of the glyphs removed should not go out under a trademark, whatever the licence allows.
 */
export const FAMILY = "Ohio Math Fallback";

/** Where the built font lives, relative to `web/`. Named once so nothing spells it twice. */
export const FONT_PATH = "src/assets/fonts/ohio-math-fallback.woff2";

const range = (first: number, last = first): number[] =>
  Array.from({ length: last - first + 1 }, (_, i) => first + i);

/**
 * Every codepoint the font is built to carry, stated rather than derived.
 *
 * Deriving it from the corpus was the other option and is the wrong one: the set would then change
 * silently whenever a formula was written, and the day one used a character outside it the page
 * would render that character in a different face without anything failing. So the repertoire is
 * fixed here, and `tests/unit/math-font.spec.ts` walks every `<math>` element in `dist/` and fails
 * if the corpus has left it. Widening it is a deliberate act with a rebuilt font attached.
 */
const REPERTOIRE = {
  /** ASCII, whole. Digits, both cases, and every operator and delimiter that is one keystroke. */
  ascii: range(0x20, 0x7e),
  /** The four Latin-1 operators, and the no-break space Temml emits for `\,` and friends. */
  latin1: [0xa0, 0xb1, 0xb7, 0xd7, 0xf7],
  /**
   * Mathematical Italic Latin — what a one-letter `<mi>` actually renders as. U+1D455 is a
   * permanent hole in Unicode: italic h is U+210E, which is why it is listed beside the block.
   */
  italic: [...range(0x1d434, 0x1d454), ...range(0x1d456, 0x1d467), 0x210e],
  /** Relations, operators and the marks a fraction or a root needs. */
  operators: [
    0x2016, 0x2026, 0x2032, 0x2044, 0x2192, 0x2211, 0x2212, 0x2215, 0x220f, 0x221a, 0x2248, 0x2260,
    0x2261, 0x2264, 0x2265, 0x22c5,
  ],
  /**
   * Delimiters beyond ASCII. Floor and ceiling are here ahead of need: the department's worksheet
   * rounds down in four places, and `ROUNDDOWN(x, 1)` set in type is a floor bracket.
   */
  delimiters: [...range(0x2308, 0x2309), ...range(0x230a, 0x230b), ...range(0x27e8, 0x27e9)],
  /**
   * The invisible operators, which are in the markup whether or not anybody typed them.
   *
   * Temml emits U+2061 FUNCTION APPLICATION between a name and the bracket it applies to, so
   * `s(\text{ADM})` and `\text{ROUNDDOWN}(\ldots)` both carry one. They draw nothing, and a font
   * missing them still looks correct — which is exactly why they belong in a stated repertoire
   * rather than in an exemption. The corpus sweep found U+2061 on the open-enrolment clawback and
   * had no way to know it was harmless; widening the repertoire is cheaper than teaching a check
   * which characters not to care about, and these four are zero-width glyphs.
   */
  invisible: range(0x2061, 0x2064),
} as const;

export const CODEPOINTS: number[] = Object.values(REPERTOIRE).flat();
