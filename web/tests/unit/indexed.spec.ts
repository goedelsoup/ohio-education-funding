/**
 * The preview cards, re-encoded — and decoded again to prove it is the same picture.
 *
 * `indexedPng` writes a PNG by hand. That is a small amount of code in a well-specified format,
 * and it is also the kind of code where a wrong filter byte or an off-by-one in a scanline
 * produces a file that *opens* and is subtly wrong — a sheared image, a colour channel rotated —
 * which nothing downstream would report. Every test here therefore decodes the output rather than
 * inspecting the encoder's intentions.
 *
 * The decoder below is deliberately independent of the encoder: it implements the five PNG
 * reconstruction filters from the specification rather than inverting anything `indexed.ts` does.
 * A shared helper would agree with itself.
 */

import { inflateSync } from "node:zlib";

import { describe, expect, test } from "vitest";

import { indexedPng } from "../../src/lib/og/indexed.ts";

/** Decode an 8-bit indexed PNG to RGBA, by the specification. */
function decode(png: Buffer): { width: number; height: number; rgba: Buffer } {
  expect([...png.subarray(0, 8)]).toEqual([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  let at = 8;
  let width = 0;
  let height = 0;
  const palette: number[][] = [];
  const idat: Buffer[] = [];
  let sawEnd = false;

  while (at < png.length) {
    const length = png.readUInt32BE(at);
    const type = png.toString("latin1", at + 4, at + 8);
    const data = png.subarray(at + 8, at + 8 + length);
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      expect(data[8], "bit depth").toBe(8);
      expect(data[9], "colour type: indexed").toBe(3);
    }
    if (type === "PLTE") for (let i = 0; i < length; i += 3) palette.push([data[i]!, data[i + 1]!, data[i + 2]!]);
    if (type === "IDAT") idat.push(Buffer.from(data));
    if (type === "IEND") sawEnd = true;
    at += 12 + length;
  }
  expect(sawEnd, "the file is terminated").toBe(true);

  const raw = inflateSync(Buffer.concat(idat));
  const rgba = Buffer.alloc(width * height * 4);
  let previous = new Uint8Array(width);
  let read = 0;
  for (let y = 0; y < height; y += 1) {
    const filter = raw[read];
    read += 1;
    const row = new Uint8Array(raw.subarray(read, read + width));
    read += width;
    for (let x = 0; x < width; x += 1) {
      const a = x >= 1 ? row[x - 1]! : 0;
      const b = previous[x]!;
      const c = x >= 1 ? previous[x - 1]! : 0;
      if (filter === 1) row[x] = (row[x]! + a) & 255;
      else if (filter === 2) row[x] = (row[x]! + b) & 255;
      else if (filter === 3) row[x] = (row[x]! + ((a + b) >> 1)) & 255;
      else if (filter === 4) {
        const p = a + b - c;
        const pa = Math.abs(p - a);
        const pb = Math.abs(p - b);
        const pc = Math.abs(p - c);
        row[x] = (row[x]! + (pa <= pb && pa <= pc ? a : pb <= pc ? b : c)) & 255;
      }
      const colour = palette[row[x]!]!;
      const out = (y * width + x) * 4;
      rgba[out] = colour[0]!;
      rgba[out + 1] = colour[1]!;
      rgba[out + 2] = colour[2]!;
      rgba[out + 3] = 255;
    }
    previous = row;
  }
  return { width, height, rgba };
}

/** A pixmap in resvg's own layout: four bytes a pixel, row-major, no padding. */
function pixmap(width: number, height: number, colour: (x: number, y: number) => number[]): Buffer {
  const out = Buffer.alloc(width * height * 4);
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const [r, g, b, a = 255] = colour(x, y);
      const at = (y * width + x) * 4;
      out[at] = r!;
      out[at + 1] = g!;
      out[at + 2] = b!;
      out[at + 3] = a;
    }
  }
  return out;
}

describe("a card re-encoded as a palette", () => {
  test("decodes to exactly the pixels it was given", () => {
    /*
     * Deliberately awkward content rather than flat fills: diagonals and a hard vertical edge are
     * what exercise all five reconstruction filters, and a shear or a channel rotation shows up in
     * a diagonal where it would hide in a rectangle.
     */
    const width = 97;
    const height = 61;
    const source = pixmap(width, height, (x, y) => {
      if (x === 40) return [11, 22, 33];
      if ((x + y) % 17 === 0) return [250, 5, 120];
      // Bounded at 13 × 11 = 143 combinations, so the fixture stays a palette. An earlier version
      // reached 385 and was refused — correctly, which is what the next test is about.
      return [(x * 3) % 13, (y * 5) % 11, 0];
    });

    const png = indexedPng(source, width, height);
    expect(png, "the fixture is inside both limits").not.toBeNull();

    const back = decode(png!);
    expect(back.width).toBe(width);
    expect(back.height).toBe(height);
    expect(back.rgba.equals(source), "every pixel survives the round trip").toBe(true);
  });

  test("is smaller than the truecolour it replaces", () => {
    // The whole point, stated as a property rather than as the 45% one build happened to measure.
    const width = 200;
    const height = 200;
    const source = pixmap(width, height, (x) => (x < 100 ? [250, 250, 249] : [26, 26, 25]));
    const png = indexedPng(source, width, height)!;
    expect(png.length).toBeLessThan(source.length / 4);
  });

  test("refuses a pixmap with more than 256 colours", () => {
    /*
     * A palette cannot hold them, and the alternative is quantising — which is a different
     * picture, silently, on the thing a reader sees before deciding whether to open the page. The
     * caller keeps resvg's truecolour output instead.
     *
     * 257 distinct colours and not 300-that-collide-to-256. The obvious fixture —
     * `[x % 256, (x * 7) % 256, (x * 13) % 256]` over 300 pixels — wraps at x = 256 back onto the
     * colour x = 0 already used, so it is exactly 256 and encodes fine. It looked like a failing
     * guard and was a fixture landing on the boundary from the wrong side.
     */
    const source = pixmap(257, 1, (x) => [x % 256, x < 256 ? 0 : 1, 0]);
    expect(indexedPng(source, 257, 1)).toBeNull();

    // And exactly 256 is a palette, which is the other side of the same boundary.
    const full = pixmap(256, 1, (x) => [x, 0, 0]);
    expect(indexedPng(full, 256, 1)).not.toBeNull();
  });

  test("refuses a pixmap that is not fully opaque", () => {
    // An indexed PNG carries alpha only through a `tRNS` chunk this does not write, so a
    // translucent pixel would come back opaque. None of the cards has one; if one ever does, it
    // keeps its truecolour encoding rather than quietly losing its transparency.
    const source = pixmap(4, 4, (x, y) => (x === 2 && y === 2 ? [1, 2, 3, 128] : [9, 9, 9]));
    expect(indexedPng(source, 4, 4)).toBeNull();
  });

  test("a single-colour card is one palette entry", () => {
    // The degenerate case, and the one where an off-by-one in the palette would be invisible in
    // the round trip because every index is zero.
    const source = pixmap(16, 16, () => [7, 8, 9]);
    const png = indexedPng(source, 16, 16)!;
    const plte = png.indexOf(Buffer.from("PLTE", "latin1"));
    expect(png.readUInt32BE(plte - 4), "three bytes, one colour").toBe(3);
    expect(decode(png).rgba.equals(source)).toBe(true);
  });
});
