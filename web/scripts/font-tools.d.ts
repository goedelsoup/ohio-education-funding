/**
 * Types for the two font libraries `subset-math-font.ts` uses, which ship none of their own.
 *
 * Written out rather than declared `any`, because `tsconfig.json` is Astro's strict base and an
 * untyped import is the one hole in it that would let a wrong argument through silently. Both
 * signatures are narrowed to the calls this repository makes: `subset-font` takes more target
 * formats than `sfnt`, and `fontverter` converts in both directions.
 *
 * `fontverter` sniffs the format off the first four bytes and does it by stringifying, so it needs
 * a Buffer and not a bare Uint8Array — a Uint8Array stringifies to a comma-joined list of every
 * byte in the font, which it reports as an unrecognised signature. The type says Buffer for that
 * reason.
 */

declare module "subset-font" {
  export default function subsetFont(
    font: Buffer,
    characters: string,
    options?: { targetFormat?: "sfnt" | "woff" | "woff2"; preserveNameIds?: number[] },
  ): Promise<Buffer>;
}

declare module "fontverter" {
  export function convert(font: Buffer, to: "sfnt" | "woff" | "woff2"): Promise<Buffer>;
}
