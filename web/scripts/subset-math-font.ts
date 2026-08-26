/**
 * How `src/assets/fonts/ohio-math-fallback.woff2` is made.
 *
 * ```
 * node scripts/subset-math-font.ts            # report only
 * node scripts/subset-math-font.ts --write    # rebuild the committed font
 * ```
 *
 * Not in the gate. This is a derivation, committed so the artefact can be re-made and argued with
 * rather than trusted — the same role `link-hue-search.ts` plays for `--link`. What the gate checks
 * is the file this produced: `tests/unit/math-font.spec.ts` reads it back with a parser that shares
 * no code with the subsetter, and `tests/e2e/app.spec.ts` makes a browser stretch a brace with it.
 *
 * # Where the bytes come from
 *
 * `@fontsource/stix-two-math`, a devDependency, so the source is pinned by the lockfile and its OFL
 * copy travels with it. The alternative was the copy of `STIXTwoMath.otf` sitting in
 * `/System/Library/Fonts/Supplemental/` on the machine that wrote this, and a build input that
 * exists on one operating system is not a build input.
 *
 * That package ships one file, `stix-two-math-latin-400-normal.woff2`, and despite the name it is
 * very nearly the whole font: 5,169 glyphs, 4,605 cmap entries, `MATH` intact with every vertical
 * assembly. It is also 403,344 bytes, which is what #202 was called to avoid.
 *
 * One thing it does NOT carry, and this is worth knowing before anyone tries to restore it: its
 * `GSUB` has been reduced to `ccmp` and `locl`. `ssty` — the feature that swaps in glyphs cut for
 * script size, so a subscript is not merely a shrunken letter — was dropped upstream by fontsource's
 * own build. Subsetting from the original OTF instead would bring it back at a measured cost of
 * about 27 KB and 300 glyphs, roughly doubling the file. For a face that only renders for a reader
 * whose platform has no math font at all, and who therefore has nothing to compare it against, that
 * is not a trade worth making. It is recorded here so the next person does not have to measure it.
 *
 * # The repertoire, and why the italic block is not optional
 *
 * MathML Core gives a single-character `<mi>` `text-transform: math-auto`, and Chromium implements
 * that by mapping the character to the Mathematical Alphanumeric Symbols block. So `<mi>C</mi>` does
 * not render U+0043. It renders U+1D436, and a subset holding only ASCII has no glyph for it.
 *
 * The failure that causes is invisible on a developer's machine and only there. Measured through
 * CDP's `CSS.getPlatformFontsForNode`, with a subset built without the block:
 *
 *     mi     STIX Two Math          <- fell out of the subset entirely
 *     mtext  Probe NoItal Math
 *     mn     Probe NoItal Math
 *
 * Every variable in every formula left the font and landed on the system's full STIX Two Math —
 * which looks perfect, because it IS the same typeface. On the reader this font exists for, the one
 * with no math font installed, there is nothing to land on but a generic serif, and the variables
 * render in a different face from everything around them. 6,336 bytes buys that not happening.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

import { convert } from "fontverter";
import subsetFont from "subset-font";

import {
  CODEPOINTS,
  FAMILY,
  FONT_PATH,
  readCmap,
  readGlyphCount,
  readNames,
  readVerticalConstructions,
  readWoff2Tables,
} from "../src/lib/math-font.ts";

const SOURCE = resolve(
  process.cwd(),
  "node_modules/@fontsource/stix-two-math/files/stix-two-math-latin-400-normal.woff2",
);
const OUTPUT = resolve(process.cwd(), FONT_PATH);

/* ------------------------------------------------------------------ *
 * Renaming, which harfbuzz will not do and which the font needs.
 * ------------------------------------------------------------------ */

interface NameRecord {
  platformID: number;
  encodingID: number;
  languageID: number;
  nameID: number;
  text: string;
}

const utf16be = (text: string): Uint8Array => {
  const out = new Uint8Array(text.length * 2);
  for (let i = 0; i < text.length; i += 1) {
    out[i * 2] = text.charCodeAt(i) >> 8;
    out[i * 2 + 1] = text.charCodeAt(i) & 0xff;
  }
  return out;
};

const latin1 = (text: string): Uint8Array =>
  Uint8Array.from(text, (character) => character.charCodeAt(0) & 0xff);

function directory(sfnt: Uint8Array): { tag: string; offset: number; length: number }[] {
  const view = new DataView(sfnt.buffer, sfnt.byteOffset, sfnt.byteLength);
  return Array.from({ length: view.getUint16(4) }, (_, i) => ({
    tag: String.fromCharCode(...sfnt.subarray(12 + i * 16, 12 + i * 16 + 4)),
    offset: view.getUint32(12 + i * 16 + 8),
    length: view.getUint32(12 + i * 16 + 12),
  }));
}

function parseNames(table: Uint8Array): NameRecord[] {
  const view = new DataView(table.buffer, table.byteOffset, table.byteLength);
  const storage = view.getUint16(4);
  return Array.from({ length: view.getUint16(2) }, (_, i) => {
    const at = 6 + i * 12;
    const platformID = view.getUint16(at);
    const length = view.getUint16(at + 8);
    const offset = storage + view.getUint16(at + 10);
    const raw = table.subarray(offset, offset + length);
    let text = "";
    if (platformID === 3) {
      for (let byte = 0; byte + 1 < raw.length; byte += 2) {
        text += String.fromCharCode((raw[byte]! << 8) | raw[byte + 1]!);
      }
    } else text = String.fromCharCode(...raw);
    return {
      platformID,
      encodingID: view.getUint16(at + 2),
      languageID: view.getUint16(at + 4),
      nameID: view.getUint16(at + 6),
      text,
    };
  });
}

function buildNames(records: NameRecord[]): Uint8Array {
  const strings = records.map((record) =>
    record.platformID === 3 ? utf16be(record.text) : latin1(record.text),
  );
  const header = new Uint8Array(6 + records.length * 12);
  const view = new DataView(header.buffer);
  view.setUint16(2, records.length);
  view.setUint16(4, header.length);
  let offset = 0;
  records.forEach((record, i) => {
    const at = 6 + i * 12;
    view.setUint16(at, record.platformID);
    view.setUint16(at + 2, record.encodingID);
    view.setUint16(at + 4, record.languageID);
    view.setUint16(at + 6, record.nameID);
    view.setUint16(at + 8, strings[i]!.length);
    view.setUint16(at + 10, offset);
    offset += strings[i]!.length;
  });
  const out = new Uint8Array(header.length + offset);
  out.set(header);
  strings.reduce((at, string) => (out.set(string, at), at + string.length), header.length);
  return out;
}

/** The sum of a table's words, which is what OpenType calls a checksum. */
function checksum(table: Uint8Array): number {
  let total = 0;
  const view = new DataView(table.buffer, table.byteOffset, table.byteLength);
  for (let at = 0; at + 3 < table.length; at += 4) total = (total + view.getUint32(at)) >>> 0;
  return total;
}

/** Re-lay an sfnt with a new `name` table: same tables, same order, recomputed offsets and sums. */
function replaceNameTable(sfnt: Uint8Array, records: NameRecord[]): Uint8Array {
  const tables = directory(sfnt).map(({ tag, offset, length }) => ({
    tag,
    body: tag === "name" ? buildNames(records) : sfnt.subarray(offset, offset + length),
  }));

  const head = new Uint8Array(12 + tables.length * 16);
  head.set(sfnt.subarray(0, 12));
  const view = new DataView(head.buffer);
  const padded = tables.map(({ body }) => {
    const block = new Uint8Array((body.length + 3) & ~3);
    block.set(body);
    return block;
  });

  let offset = head.length;
  tables.forEach(({ tag, body }, i) => {
    const at = 12 + i * 16;
    head.set(latin1(tag), at);
    view.setUint32(at + 4, checksum(padded[i]!));
    view.setUint32(at + 8, offset);
    view.setUint32(at + 12, body.length);
    offset += padded[i]!.length;
  });

  const out = new Uint8Array(offset);
  out.set(head);
  padded.reduce((at, block) => (out.set(block, at), at + block.length), head.length);

  // `checkSumAdjustment` is a sum over the whole file with its own four bytes read as zero.
  const headTable = directory(out).find(({ tag }) => tag === "head")!;
  const outView = new DataView(out.buffer);
  outView.setUint32(headTable.offset + 8, 0);
  outView.setUint32(headTable.offset + 8, (0xb1b0afba - checksum(out)) >>> 0);
  return out;
}

/* ------------------------------------------------------------------ *
 * The build.
 * ------------------------------------------------------------------ */

/**
 * What the font says about itself when somebody opens it in a font tool.
 *
 * Written out rather than search-and-replaced over the upstream strings, because two of these
 * records are obligations and one is a courtesy. The obligations: the OFL requires the copyright
 * notice and the licence to travel with the software, and harfbuzz drops every name id above 6 —
 * so id 13 and id 14 have to be put back by hand or this file ships stripped of its licence. The
 * courtesy is id 10, which is where the next person finds out what they are holding.
 *
 * The family name is ours; the provenance is stated in prose. That is the right way round: a
 * trademark must not be the name of a modified font, and it must be credited inside it.
 */
function describe(upstream: Map<number, string>): NameRecord[] {
  const version = upstream.get(5) ?? "unknown version";
  const strings = new Map<number, string>([
    [0, `${upstream.get(0) ?? ""}\nSubset and renamed for ohio-education-funding; not the original font.`],
    [1, FAMILY],
    [2, "Regular"],
    [3, `${FAMILY} — subset of STIX Two Math ${version.replace(/^Version /, "")}`],
    [4, `${FAMILY} Regular`],
    [5, `${version}; subset`],
    [6, FAMILY.replace(/ /g, "") + "-Regular"],
    [
      10,
      `A ${CODEPOINTS.length}-codepoint subset of STIX Two Math, carrying its MATH table so that ` +
        "delimiters still stretch. Renamed because it is not the whole font, and because " +
        "STIX Two is a trademark of the IEEE.",
    ],
    [13, upstream.get(13) ?? "This Font Software is licensed under the SIL Open Font License, Version 1.1."],
    [14, upstream.get(14) ?? "https://scripts.sil.org/OFL"],
  ]);
  return [...strings].map(([nameID, text]) => ({
    platformID: 3,
    encodingID: 1,
    languageID: 0x409,
    nameID,
    text,
  }));
}

export async function build(): Promise<Uint8Array> {
  const source = readFileSync(SOURCE);
  const text = CODEPOINTS.map((codepoint) => String.fromCodePoint(codepoint)).join("");
  const subset = new Uint8Array(await subsetFont(source, text, { targetFormat: "sfnt" }));
  const table = directory(subset).find(({ tag }) => tag === "name");
  if (!table) throw new Error("the subset came back with no name table to rename");
  const upstream = new Map(
    parseNames(subset.subarray(table.offset, table.offset + table.length)).map((record) => [
      record.nameID,
      record.text,
    ]),
  );
  const renamed = replaceNameTable(subset, describe(upstream));
  // `fontverter` is the wasm woff2 encoder `subset-font` itself uses, named here as a direct
  // dependency rather than reached through it: harfbuzz emits an sfnt, and the rename has to
  // happen between the subsetting and the compression.
  // `fontverter` sniffs the signature off a Buffer, and a bare Uint8Array stringifies to
  // a comma-joined list of every byte in the font rather than to `\0\u0001\0\0`.
  return new Uint8Array(await convert(Buffer.from(renamed), "woff2"));
}

if (process.argv[1]?.endsWith("subset-math-font.ts")) {
  const font = await build();
  const tables = readWoff2Tables(font);
  const cmap = readCmap(tables.get("cmap")!);
  const constructions = readVerticalConstructions(tables.get("MATH")!);
  const names = readNames(tables.get("name")!);
  const glyphs = readGlyphCount(tables.get("maxp")!);

  console.log(`source     ${SOURCE.replace(process.cwd() + "/", "")}  ${readFileSync(SOURCE).length} B`);
  console.log(`family     ${names.get(1)}  (PostScript ${names.get(6)})`);
  console.log(`repertoire ${CODEPOINTS.length} codepoints requested, ${cmap.size} in cmap, ${glyphs} glyphs`);
  console.log(`output     ${font.length} B`);
  console.log("");
  for (const character of "{}()[]|") {
    const glyph = cmap.get(character.codePointAt(0)!);
    const construction = glyph === undefined ? undefined : constructions.get(glyph);
    console.log(
      `  ${character}  ${construction ? `${construction.variants.length} variants, ${construction.parts.length} assembly parts` : "no vertical construction"}`,
    );
  }

  if (process.argv.includes("--write")) {
    writeFileSync(OUTPUT, font);
    console.log(`\nwrote ${OUTPUT.replace(process.cwd() + "/", "")}`);
  } else {
    const existing = readFileSync(OUTPUT);
    console.log(
      `\ncommitted ${existing.length} B — ${Buffer.from(font).equals(existing) ? "identical" : "DIFFERS from this build"}`,
    );
  }
}
