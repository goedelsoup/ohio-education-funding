/**
 * The colour arithmetic behind every separation claim this repository makes.
 *
 * # Why this exists
 *
 * `tokens.ts` and `tokens/colors.css` state figures — "ΔE 21.4 light, 21.6 dark, 20.8 under the
 * worst CVD simulation", "five steps of this hue close to 10.9" — and until now nothing in the
 * repository could produce them. They were computed once, somewhere else, and written down. That
 * makes them provenance rather than measurement: a reader cannot check them, and a palette change
 * cannot be checked *against* them.
 *
 * So the arithmetic lives here and `palette.spec.ts` runs it. The numbers in those comments are
 * now assertions, and a ramp that drifts out of its band fails a build rather than a review.
 *
 * # What is implemented, and to which definition
 *
 * - sRGB → linear → CIEXYZ (D65) → CIELAB, the standard chain.
 * - **CIEDE2000** for perceptual difference, and CIE76 alongside it. Both, because the figures
 *   being reproduced were computed by a tool this repository does not have, and identifying which
 *   metric produced them is part of checking them rather than something to assume.
 * - **Machado, Oliveira & Fernandes (2009)** dichromacy matrices at severity 1.0 for protanopia,
 *   deuteranopia and tritanopia, applied in linear RGB, which is where they are defined.
 * - WCAG 2.x relative luminance and contrast ratio.
 *
 * # The threshold, and what it is not
 *
 * A ΔE floor is a *separation* claim: two marks a reader can tell apart. It is not a contrast
 * claim and does not substitute for one — the end steps of both ramps sit near 2.2:1 against their
 * own surface, which is why a chart drawn in them is required to carry a legend and a companion
 * table. Separation says "these are two classes"; contrast says "this mark is visible at all".
 */

/** A colour as three 0–255 channels. */
export interface Rgb {
  r: number;
  g: number;
  b: number;
}

/** CIELAB, D65. */
export interface Lab {
  l: number;
  a: number;
  b: number;
}

/** The three dichromacies this repository simulates, plus the trivial case. */
export type Vision = "normal" | "protan" | "deutan" | "tritan";

/** `#rrggbb` or `#rgb` to channels. Throws rather than guessing at anything else. */
export function parseHex(hex: string): Rgb {
  const clean = hex.trim().replace(/^#/, "");
  const full =
    clean.length === 3
      ? clean
          .split("")
          .map((c) => c + c)
          .join("")
      : clean;
  if (!/^[0-9a-f]{6}$/i.test(full)) throw new Error(`not a hex colour: ${hex}`);
  return {
    r: Number.parseInt(full.slice(0, 2), 16),
    g: Number.parseInt(full.slice(2, 4), 16),
    b: Number.parseInt(full.slice(4, 6), 16),
  };
}

/** sRGB companding, reversed: 0–255 to linear-light 0–1. */
const linearize = (channel: number): number => {
  const c = channel / 255;
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
};

/** And forward again, for a simulated colour that has to come back to a displayable one. */
const compand = (c: number): number => {
  const v = c <= 0.0031308 ? c * 12.92 : 1.055 * c ** (1 / 2.4) - 0.055;
  return Math.round(Math.min(1, Math.max(0, v)) * 255);
};

/** WCAG 2.x relative luminance. */
export function luminance(colour: Rgb): number {
  return (
    0.2126 * linearize(colour.r) + 0.7152 * linearize(colour.g) + 0.0722 * linearize(colour.b)
  );
}

/** WCAG 2.x contrast ratio, always ≥ 1 and order-independent. */
export function contrast(a: Rgb, b: Rgb): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi! + 0.05) / (lo! + 0.05);
}

/** D65 white point, the reference this whole chain is relative to. */
const WHITE = { x: 0.95047, y: 1.0, z: 1.08883 };

export function toLab(colour: Rgb): Lab {
  const r = linearize(colour.r);
  const g = linearize(colour.g);
  const b = linearize(colour.b);
  const x = (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / WHITE.x;
  const y = (0.2126729 * r + 0.7151522 * g + 0.072175 * b) / WHITE.y;
  const z = (0.0193339 * r + 0.119192 * g + 0.9503041 * b) / WHITE.z;
  const f = (t: number): number => (t > (6 / 29) ** 3 ? Math.cbrt(t) : t / (3 * (6 / 29) ** 2) + 4 / 29);
  return { l: 116 * f(y) - 16, a: 500 * (f(x) - f(y)), b: 200 * (f(y) - f(z)) };
}

/** Chroma in the a*b* plane. The "chroma floor" check reads this. */
export const chroma = (lab: Lab): number => Math.hypot(lab.a, lab.b);

/**
 * OKLCH, because that is the space this palette's derivation rule is stated in.
 *
 * `tokens/colors.css` says the `-text` variants hold "OKLab hue and chroma to the mark colour's
 * exactly and only lightness moves". Nothing in this repository could check that — the file has a
 * documented history of figures computed once, elsewhere, by a tool nobody committed, and this was
 * one more of them. It is here so the rule can be run rather than trusted.
 *
 * It reproduces, which is worth saying plainly given that history: the claimed hue drifts of 0.01
 * and 2.75 degrees on light, and "under 0.1" on dark, come back as 0.00, 2.75, 0.06 and 0.05.
 *
 * Ottosson's matrices, straight through. Lightness here is OKLab's `L`, on 0–1, and is a different
 * quantity from CIELAB's `l` on 0–100 — the two are not interchangeable and this file carries both.
 */
export interface Oklch {
  /** 0–1. Perceptual lightness, NOT CIELAB `l`. */
  l: number;
  c: number;
  /** Degrees, 0–360. */
  h: number;
}

const CBRT = (t: number): number => Math.cbrt(t);

export function toOklch(colour: Rgb): Oklch {
  const r = linearize(colour.r);
  const g = linearize(colour.g);
  const b = linearize(colour.b);
  const l = CBRT(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = CBRT(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = CBRT(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  const okL = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const okA = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const okB = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  return {
    l: okL,
    c: Math.hypot(okA, okB),
    h: ((Math.atan2(okB, okA) * 180) / Math.PI + 360) % 360,
  };
}

/**
 * OKLCH back to channels, or `null` when the coordinate is outside sRGB.
 *
 * `null` rather than a clamp on purpose. A clamped out-of-gamut colour is a DIFFERENT colour that
 * still answers to the coordinate that produced it, so a search over the space would quietly
 * optimise over values it cannot ship. The tolerance below absorbs float error at the boundary and
 * nothing wider.
 */
export function fromOklch(colour: Oklch): Rgb | null {
  const a = colour.c * Math.cos((colour.h * Math.PI) / 180);
  const b = colour.c * Math.sin((colour.h * Math.PI) / 180);
  const l = (colour.l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (colour.l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (colour.l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  // Gamut is decided on the LINEAR channels, before companding, because `compand` clamps — an
  // out-of-gamut coordinate would otherwise come back as a valid-looking hex that is a different
  // colour from the one asked for. 1e-4 of linear light is a third of a channel step at the dark
  // end: float slop at the boundary, and nothing a search could hide inside.
  const linear = [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ];
  if (linear.some((v) => v < -1e-4 || v > 1 + 1e-4)) return null;
  const out = linear.map(compand);
  return { r: out[0]!, g: out[1]!, b: out[2]! };
}

/** Channels back to `#rrggbb`, the form the token files are written in. */
export const toHex = (colour: Rgb): string =>
  `#${[colour.r, colour.g, colour.b].map((c) => c.toString(16).padStart(2, "0")).join("")}`;

/** The smallest angle between two hues, which is not `a - b` when the pair straddles 0. */
export const hueGap = (a: number, b: number): number => {
  const d = Math.abs(a - b) % 360;
  return d > 180 ? 360 - d : d;
};

/** Straight-line distance in Lab. The 1976 definition, kept for identifying older figures. */
export function deltaE76(a: Lab, b: Lab): number {
  return Math.hypot(a.l - b.l, a.a - b.a, a.b - b.b);
}

/**
 * CIEDE2000.
 *
 * The full formulation — lightness, chroma and hue weightings, the neutral-region chroma
 * correction, and the rotation term that handles the blue region where a straight Lab distance
 * most overstates difference. That region is exactly where both of these ramps live, which is why
 * using the 1976 number for a blue ramp flatters it.
 */
export function deltaE2000(one: Lab, two: Lab): number {
  const kL = 1;
  const kC = 1;
  const kH = 1;
  const rad = Math.PI / 180;

  const c1 = Math.hypot(one.a, one.b);
  const c2 = Math.hypot(two.a, two.b);
  const cBar = (c1 + c2) / 2;
  const g = 0.5 * (1 - Math.sqrt(cBar ** 7 / (cBar ** 7 + 25 ** 7)));

  const a1p = (1 + g) * one.a;
  const a2p = (1 + g) * two.a;
  const c1p = Math.hypot(a1p, one.b);
  const c2p = Math.hypot(a2p, two.b);

  const hp = (b: number, ap: number): number => {
    if (b === 0 && ap === 0) return 0;
    const angle = Math.atan2(b, ap) / rad;
    return angle >= 0 ? angle : angle + 360;
  };
  const h1p = hp(one.b, a1p);
  const h2p = hp(two.b, a2p);

  const dLp = two.l - one.l;
  const dCp = c2p - c1p;

  let dhp: number;
  if (c1p * c2p === 0) dhp = 0;
  else if (Math.abs(h2p - h1p) <= 180) dhp = h2p - h1p;
  else if (h2p - h1p > 180) dhp = h2p - h1p - 360;
  else dhp = h2p - h1p + 360;
  const dHp = 2 * Math.sqrt(c1p * c2p) * Math.sin((dhp / 2) * rad);

  const lBar = (one.l + two.l) / 2;
  const cBarP = (c1p + c2p) / 2;

  let hBarP: number;
  if (c1p * c2p === 0) hBarP = h1p + h2p;
  else if (Math.abs(h1p - h2p) <= 180) hBarP = (h1p + h2p) / 2;
  else if (h1p + h2p < 360) hBarP = (h1p + h2p + 360) / 2;
  else hBarP = (h1p + h2p - 360) / 2;

  const t =
    1 -
    0.17 * Math.cos((hBarP - 30) * rad) +
    0.24 * Math.cos(2 * hBarP * rad) +
    0.32 * Math.cos((3 * hBarP + 6) * rad) -
    0.2 * Math.cos((4 * hBarP - 63) * rad);

  const sL = 1 + (0.015 * (lBar - 50) ** 2) / Math.sqrt(20 + (lBar - 50) ** 2);
  const sC = 1 + 0.045 * cBarP;
  const sH = 1 + 0.015 * cBarP * t;

  const dTheta = 30 * Math.exp(-(((hBarP - 275) / 25) ** 2));
  const rC = 2 * Math.sqrt(cBarP ** 7 / (cBarP ** 7 + 25 ** 7));
  const rT = -rC * Math.sin(2 * dTheta * rad);

  return Math.sqrt(
    (dLp / (kL * sL)) ** 2 +
      (dCp / (kC * sC)) ** 2 +
      (dHp / (kH * sH)) ** 2 +
      rT * (dCp / (kC * sC)) * (dHp / (kH * sH)),
  );
}

/**
 * Machado, Oliveira & Fernandes (2009), severity 1.0.
 *
 * Applied in linear RGB, which is where they are defined — running them on companded values is a
 * common error that makes every simulated colour too light and every separation figure too
 * generous.
 */
const CVD: Record<Exclude<Vision, "normal">, readonly number[]> = {
  protan: [0.152286, 1.052583, -0.204868, 0.114503, 0.786281, 0.099216, -0.003882, -0.048116, 1.051998],
  deutan: [0.367322, 0.860646, -0.227968, 0.280085, 0.672501, 0.047413, -0.01182, 0.04294, 0.968881],
  tritan: [1.255528, -0.076749, -0.178779, -0.078411, 0.930809, 0.147602, 0.004733, 0.691367, 0.3039],
};

/** A colour as a reader with the named vision sees it. `normal` is the identity. */
export function simulate(colour: Rgb, vision: Vision): Rgb {
  if (vision === "normal") return colour;
  const m = CVD[vision];
  const r = linearize(colour.r);
  const g = linearize(colour.g);
  const b = linearize(colour.b);
  return {
    r: compand(m[0]! * r + m[1]! * g + m[2]! * b),
    g: compand(m[3]! * r + m[4]! * g + m[5]! * b),
    b: compand(m[6]! * r + m[7]! * g + m[8]! * b),
  };
}

/** Every unordered pair of a list, as index pairs. An all-pairs form needs all of them. */
export function pairs<T>(items: readonly T[]): [T, T][] {
  const out: [T, T][] = [];
  for (let i = 0; i < items.length; i += 1) {
    for (let j = i + 1; j < items.length; j += 1) out.push([items[i]!, items[j]!]);
  }
  return out;
}

/** The worst (smallest) pairwise separation in a ramp, under one vision. */
export function minSeparation(
  ramp: readonly string[],
  vision: Vision,
  metric: (a: Lab, b: Lab) => number = deltaE2000,
): number {
  const seen = ramp.map((hex) => toLab(simulate(parseHex(hex), vision)));
  return Math.min(...pairs(seen).map(([a, b]) => metric(a, b)));
}

/** The worst separation across normal vision and all three dichromacies. */
export function worstSeparation(
  ramp: readonly string[],
  metric: (a: Lab, b: Lab) => number = deltaE2000,
): { vision: Vision; deltaE: number } {
  const visions: Vision[] = ["normal", "protan", "deutan", "tritan"];
  return visions
    .map((vision) => ({ vision, deltaE: minSeparation(ramp, vision, metric) }))
    .reduce((worst, one) => (one.deltaE < worst.deltaE ? one : worst));
}
