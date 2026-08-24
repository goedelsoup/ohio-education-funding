/**
 * A preview card as an indexed PNG, which is what it always was.
 *
 * # Why the cards are the wrong kind of PNG
 *
 * `resvg` renders to an RGBA pixmap and `asPng()` writes it out as 8-bit truecolour with alpha —
 * four bytes a pixel, the right default for an arbitrary drawing. These are not arbitrary
 * drawings. A card is flat fills, one hairline and glyph outlines in two weights, over a solid
 * ground: **128 distinct colours** on a district card, and **every pixel fully opaque**. Measured
 * across the built set, not one of the 1,050 exceeds 256 colours.
 *
 * So each pixel was spending 32 bits to say one of 128 things. Written as a palette the same
 * pixels come to **41%** of the size — 51.8 MB of deploy down to 21.5 MB — and it is not a
 * quantisation: every colour present gets its own entry, so the decoded image is identical to the
 * one that would have shipped, pixel for pixel. Checked that way rather than asserted: all 1,050
 * cards of a build were decoded and compared against the truecolour build they replace.
 *
 * # Why this is written here rather than pulled in
 *
 * The alternative is `sharp`, which is thirty megabytes and a per-platform prebuilt binary, for
 * one call. What is needed is not image processing — there is nothing to resample, quantise or
 * convert — it is the same pixels in the encoding they should have had. That is a palette, a
 * scanline and `zlib`, and Node has all three.
 *
 * It is also very nearly free. `asPng()` no longer runs where this succeeds, so the whole change
 * costs the build about a second and a half.
 *
 * # It refuses rather than approximates
 *
 * Two conditions, and either one returns `null` so the caller keeps the truecolour original:
 * more than 256 distinct colours, or any pixel that is not fully opaque. Both are impossible for
 * the cards this site draws today and both would be silent corruption if they were guessed at —
 * a quantised palette is a *different picture*, and a card is the thing a reader sees before they
 * decide whether to open the page.
 */

import { deflateSync } from "node:zlib";

/** PNG's own CRC-32, over the chunk type and its data. */
const CRC_TABLE = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  return table;
})();

function crc32(buffer: Buffer): number {
  let c = -1;
  for (const byte of buffer) c = CRC_TABLE[(c ^ byte) & 255]! ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}

/** Length, type, data, CRC — the shape every PNG chunk has. */
function chunk(type: string, data: Buffer): Buffer {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "latin1"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([length, body, crc]);
}

/**
 * Why every row is written unfiltered.
 *
 * A PNG row may be prefixed with one of five filters, each predicting a byte from its neighbours
 * so that what `zlib` sees is mostly zeroes. That is the right thing for a photograph, where a
 * byte *is* a magnitude and its neighbour is a good guess at it. It is the wrong thing here, and
 * measurably so: a palette index is an arbitrary label, so subtracting one index from the next
 * turns a flat region — a long run of one repeated byte, which `deflate` encodes almost for free —
 * into a run of differences between unrelated numbers.
 *
 * Measured over twelve real cards, as a share of the truecolour originals and the time to encode:
 *
 * | filters offered | level | size | encode |
 * |---|---|---|---|
 * | all five, chosen per row | 9 | 47% | 53ms |
 * | all five, chosen per row | 6 | 49% | 18ms |
 * | `Sub` only | 6 | 49% | 9ms |
 * | **none** | **6** | **42%** | **10ms** |
 * | none | 9 | 39% | 37ms |
 *
 * So the standard per-row heuristic makes the file *bigger* here while costing five passes over
 * every scanline to decide. Level 9 buys a further three points for 27ms a card — 28 seconds
 * across the build — which is not a trade this repository wants made on its behalf; see #111.
 */
const NO_FILTER = 0;

/**
 * Encode an RGBA pixmap as an 8-bit indexed PNG, or `null` if it is not one.
 *
 * `pixels` is `resvg`'s own buffer — four bytes per pixel, row-major, no padding — so nothing has
 * to be decoded first. See the module docstring for the two conditions this refuses on.
 */
export function indexedPng(pixels: Buffer, width: number, height: number): Buffer | null {
  const indexes = Buffer.alloc(width * height);
  const seen = new Map<number, number>();
  const palette: number[] = [];

  for (let at = 0, pixel = 0; at < pixels.length; at += 4, pixel += 1) {
    if (pixels[at + 3] !== 255) return null;
    const colour = (pixels[at]! << 16) | (pixels[at + 1]! << 8) | pixels[at + 2]!;
    let index = seen.get(colour);
    if (index === undefined) {
      if (palette.length === 256) return null;
      index = palette.length;
      palette.push(colour);
      seen.set(colour, index);
    }
    indexes[pixel] = index;
  }

  // One filter byte per row, then the row. See {@link NO_FILTER} for why the byte is always zero.
  const scanlines = Buffer.alloc(height * (width + 1));
  for (let y = 0; y < height; y += 1) {
    scanlines[y * (width + 1)] = NO_FILTER;
    indexes.copy(scanlines, y * (width + 1) + 1, y * width, (y + 1) * width);
  }

  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0);
  header.writeUInt32BE(height, 4);
  header[8] = 8; // bit depth
  header[9] = 3; // colour type: indexed
  // Compression 0, filter 0, interlace 0 — the only values PNG defines, already zeroed.

  const plte = Buffer.alloc(palette.length * 3);
  palette.forEach((colour, n) => {
    plte[n * 3] = (colour >> 16) & 255;
    plte[n * 3 + 1] = (colour >> 8) & 255;
    plte[n * 3 + 2] = colour & 255;
  });

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", header),
    chunk("PLTE", plte),
    chunk("IDAT", deflateSync(scanlines, { level: 6 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}
