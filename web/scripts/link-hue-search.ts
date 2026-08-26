/**
 * Whether a third text hue fits on this palette. It does not, and this is the run that says so.
 *
 *   pnpm exec node --experimental-strip-types scripts/link-hue-search.ts
 *
 * # The question
 *
 * `a` was `--series-formula-text`, which is also `.gain` and every formula figure on the site;
 * `--series-guarantee-text` is at once the guarantee series, `.loss`, `.err` and a correction
 * heading. So a coloured word on a district page meant either "navigable" or "datum" and a reader
 * had to work out which. #187 asked for the obvious repair: a third text hue, of its own, solved
 * the way the `-text` pair was, and separating from both of them under CVD.
 *
 * # The answer, and it is no
 *
 * At this palette's own chroma the best hue anywhere on the circle separates from the two series
 * text colours by CIEDE2000 10.2, worst of four visions, in the worse of the two modes.
 *
 *     15.0 / 17.1     the ordinal ramp this palette ACCEPTED
 *     10.9 / 10.7     the five-step ramp this palette MEASURED AND DELETED
 *     10.2            the best third text hue available
 *     31.2 / 33.8     a link that is not coloured at all: the ink, against the series pair
 *
 * Two structural reasons, both visible in the sweep this prints:
 *
 * 1. **Both ends of the axis are spent.** Protanopia and deuteranopia collapse red-green and leave
 *    blue-yellow. Blue at 255.5 degrees and orange at 37.9 are the two ends of what remains, so a
 *    third hue has to fit BETWEEN them and separates from both by less than they do from each
 *    other. Every survivor lands in 337-348 degrees, on the purple line, which is the only region
 *    that is neither.
 *
 * 2. **Half the circle has no solution at all.** 40 through 240 degrees — yellow, green, cyan —
 *    cannot clear 5:1 on three light surfaces at a real chroma and still stay clear of the ink.
 *
 * The sensitivity table shows the only lever: separation is bought by desaturating, because a grey
 * separates from everything. 14.4 is available at chroma 0.10, against the series pair's 0.162 and
 * 0.173 — a link visibly duller than every other colour on the site, which buys its legibility as
 * a datum by spending its legibility as a link.
 *
 * # What shipped instead
 *
 * The link stops being a coloured word. It is set in the ink and carries an underline in `--link`,
 * so the distinction moves from a hue a protanope cannot resolve to a form nobody has to resolve:
 * 31.2 and 33.8, three times what any hue could buy, because the word is not competing on colour.
 * The district index had already reached this conclusion for its 609 rows and never generalised it.
 *
 * `--link` is then solved for the role it actually has — an underline, a focus ring, and the word
 * under the cursor on hover — by this palette's own rule, and the last section here prints it.
 *
 * # Two wrong answers this returned first, both from under-constraining it
 *
 * Left free, it returns `#6b5c60`: a warm grey, which separates from three chromatic colours by
 * having no chroma. Given a chroma FLOOR instead, it pins chroma at the floor for the same reason,
 * so the floor becomes the answer and the search only appears to have chosen. `ramp-search.ts`
 * records the identical failure — an optimiser answers the question asked, and "be visibly a
 * colour" was not asked. Chroma is a stated constant here, and that is why.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import {
  contrast,
  deltaE2000,
  fromOklch,
  parseHex,
  simulate,
  toHex,
  toLab,
  hueGap,
  toOklch,
  type Vision,
} from "../src/lib/plot/palette.ts";

const VISIONS: Vision[] = ["normal", "protan", "deutan", "tritan"];
const TOKENS = resolve(process.cwd(), "src/styles/tokens/colors.css");

/** The declarations of one `:root` block, by custom property. Same reader as `palette.spec.ts`. */
function block(marker: string): Map<string, string> {
  const css = readFileSync(TOKENS, "utf8");
  const start = css.indexOf(marker);
  if (start < 0) throw new Error(`no ${marker} block in tokens/colors.css`);
  const body = css.slice(start, css.indexOf("\n}", start));
  return new Map(
    [...body.matchAll(/(--[a-z0-9-]+):\s*(#[0-9a-f]{3,8})\s*;/gi)].map((m) => [
      m[1]!,
      m[2]!.toLowerCase(),
    ]),
  );
}

const MODES = {
  light: block(":root {"),
  dark: block(':root[data-theme="dark"] {'),
} as const;
type Mode = keyof typeof MODES;

const get = (mode: Mode, name: string): string => {
  const v = MODES[mode].get(name);
  if (!v) throw new Error(`${name} is not declared in the ${mode} palette`);
  return v;
};

const SURFACES = ["--surface-0", "--surface-1", "--surface-2"] as const;
/** What a link must not be mistaken for: the two series text colours. */
const RIVALS = ["--series-formula-text", "--series-guarantee-text"] as const;

/** Worst-vision CIEDE2000 between a candidate and each of a set of colours. */
const worstAgainst = (candidate: string, others: readonly string[]): number =>
  Math.min(
    ...VISIONS.map((vision) => {
      const a = toLab(simulate(parseHex(candidate), vision));
      return Math.min(...others.map((o) => deltaE2000(a, toLab(simulate(parseHex(o), vision)))));
    }),
  );

const worstContrast = (candidate: string, mode: Mode): number =>
  Math.min(...SURFACES.map((s) => contrast(parseHex(candidate), parseHex(get(mode, s)))));

/*
 * The constraints, and each is here because leaving it out returned a wrong answer.
 *
 * CHROMA — stated, not searched, and that took two goes to get right.
 *
 * Left free, the search returns `#6b5c60`: a warm grey. It maximises separation from three
 * chromatic colours by having no chroma at all, which is the failure `ramp-search.ts` records —
 * an optimiser answers the question asked, and "be visibly a colour" was not asked. Given a floor
 * instead, it pins chroma AT the floor for the same reason, so the floor becomes the answer and
 * the search only appears to have chosen something.
 *
 * So chroma is a design constant here, set to sit between the two colours the link has to live
 * beside: `--series-formula-text` is at OKLab chroma 0.162 and `--series-guarantee-text` at 0.173.
 * A link duller than both would read as a third rank of grey rather than a third colour, and the
 * separation it bought would be bought by looking less like a link.
 *
 * INK_FLOOR — without it the search puts the link near the body ink, which separates from the
 * series colours beautifully and is unreadable as a link. 24 is under what the site already
 * ships between a series text colour and `--text-secondary` (28.1), and it is lower than the
 * rival floor deliberately: a link carries an underline and a series figure does not, so colour
 * is the only channel against a datum and the second channel against prose.
 */
const CHROMA = 0.165;
const INK_FLOOR = 24;
const CONTRAST_FLOOR = 5;

interface Solution {
  hex: string;
  rival: number;
  ink: number;
  contrast: number;
  l: number;
  c: number;
}

/** The best lightness at one (hue, chroma) in one mode: most separation from the two rivals. */
function solve(hue: number, chromaValue: number, mode: Mode): Solution | null {
  const rivals = RIVALS.map((r) => get(mode, r));
  const ink = get(mode, "--text-primary");
  let best: Solution | null = null;
  for (let l = 0.15; l <= 0.95; l += 0.002) {
    const rgb = fromOklch({ l, c: chromaValue, h: hue });
    if (!rgb) continue;
    const hex = toHex(rgb);
    const cr = worstContrast(hex, mode);
    if (cr < CONTRAST_FLOOR) continue;
    const vsInk = worstAgainst(hex, [ink]);
    if (vsInk < INK_FLOOR) continue;
    const vsRivals = worstAgainst(hex, rivals);
    if (!best || vsRivals > best.rival) {
      best = { hex, rival: vsRivals, ink: vsInk, contrast: cr, l, c: chromaValue };
    }
  }
  return best;
}

/*
 * One hue AND one chroma serve both modes; only lightness moves between them.
 *
 * That is not an extra constraint invented here — it is what the palette already does. The formula
 * pair sits at OKLab chroma 0.1622 light and 0.1614 dark, the guarantee pair at 0.1725 and 0.1730,
 * and both hold their hue to a tenth of a degree across the two themes. Rule 1 of `colors.css` says
 * a hue keeps its identity across themes; this is that rule stated tightly enough to search under,
 * and `palette.spec.ts` asserts it of all three pairs afterwards.
 */
const both = (
  hue: number,
  chromaValue: number,
): { score: number; light: Solution; dark: Solution } | null => {
  const light = solve(hue, chromaValue, "light");
  const dark = solve(hue, chromaValue, "dark");
  if (!light || !dark) return null;
  return { score: Math.min(light.rival, dark.rival), light, dark };
};

const OCCUPIED = RIVALS.map((r) => ({
  name: r,
  hue: toOklch(parseHex(get("light", r))).h,
}));

console.log("The two hues already spoken for, in OKLCH:");
for (const o of OCCUPIED) console.log(`  ${o.name.padEnd(26)} ${o.hue.toFixed(1)} deg`);
console.log(
  `\nFixed: OKLab chroma ${CHROMA}. Floors: contrast >= ${CONTRAST_FLOOR}:1 on three surfaces, ` +
    `CIEDE2000 >= ${INK_FLOOR} from the ink.\nSeparation is the worst of normal vision and three dichromacies, in the worse of the two modes.\n`,
);

const results: { hue: number; score: number; light: Solution; dark: Solution }[] = [];
for (let hue = 0; hue < 360; hue += 1) {
  const r = both(hue, CHROMA);
  if (r) results.push({ hue, ...r });
}

console.log("hue    sep   |  light      sep   ink   cr   |  dark       sep   ink   cr");
for (const r of [...results].sort((a, b) => b.score - a.score).slice(0, 12)) {
  console.log(
    `${String(r.hue).padStart(3)}   ${r.score.toFixed(1).padStart(4)}   |  ` +
      `${r.light.hex}  ${r.light.rival.toFixed(1).padStart(4)}  ${r.light.ink.toFixed(1).padStart(4)}  ${r.light.contrast.toFixed(2)}  |  ` +
      `${r.dark.hex}  ${r.dark.rival.toFixed(1).padStart(4)}  ${r.dark.ink.toFixed(1).padStart(4)}  ${r.dark.contrast.toFixed(2)}`,
  );
}

console.log("\nThe whole circle, every 20 degrees — this is the shape of the budget:");
for (const r of results.filter((r) => r.hue % 20 === 0))
  console.log(`  ${String(r.hue).padStart(3)}  ${r.score.toFixed(1).padStart(4)}`);
const dead = [...Array(18).keys()]
  .map((i) => i * 20)
  .filter((h) => !results.some((r) => r.hue === h));
if (dead.length) console.log(`  no solution at all: ${dead.join(", ")}`);

/*
 * What the chroma constant costs, so the decision above is visible rather than merely stated.
 *
 * Separation and saturation trade directly here: every step away from the palette's own chroma
 * buys separation from two chromatic rivals by being less of a colour. The table is the exchange
 * rate. It is not an invitation to take the cheapest row.
 */
console.log("\nSensitivity — the best hue available at each chroma:");
for (const c of [0.1, 0.12, 0.14, CHROMA, 0.18, 0.2]) {
  let best: { hue: number; score: number; light: Solution; dark: Solution } | null = null;
  for (let hue = 0; hue < 360; hue += 1) {
    const r = both(hue, c);
    if (r && (!best || r.score > best.score)) best = { hue, ...r };
  }
  const tag = c === CHROMA ? "  <- the palette's own" : "";
  console.log(
    best
      ? `  C ${c.toFixed(3)}   hue ${String(best.hue).padStart(3)}   sep ${best.score.toFixed(1).padStart(4)}   ${best.light.hex} / ${best.dark.hex}${tag}`
      : `  C ${c.toFixed(3)}   no hue on the circle has a solution${tag}`,
  );
}

/*
 * The reference points, without which the number above is just a number.
 *
 * Two of these are this repository's own precedents, and they bracket the answer: 15.0 is the
 * separation of the ordinal ramp it accepted, 10.9 the separation of the five-step ramp it
 * measured and deleted. The third is what the site gets today from NOT colouring a link — the
 * district index already sets its 609 district links in the ink rather than the series blue.
 */
const ORD3 = ["--ordinal-1", "--ordinal-2", "--ordinal-3"] as const;
console.log("\nReference points:");
for (const mode of ["light", "dark"] as const) {
  const ramp = ORD3.map((n) => get(mode, n));
  const worstRamp = Math.min(
    ...VISIONS.map((v) => {
      const labs = ramp.map((h) => toLab(simulate(parseHex(h), v)));
      return Math.min(
        deltaE2000(labs[0]!, labs[1]!),
        deltaE2000(labs[1]!, labs[2]!),
        deltaE2000(labs[0]!, labs[2]!),
      );
    }),
  );
  const ink = get(mode, "--text-primary");
  console.log(
    `  ${mode.padEnd(6)} the ordinal ramp this palette accepted        ${worstRamp.toFixed(1)}`,
  );
  console.log(
    `  ${mode.padEnd(6)} an UNCOLOURED link, ink against the series    ` +
      `${worstAgainst(ink, RIVALS.map((r) => get(mode, r))).toFixed(1)}`,
  );
}
console.log("  both   the five-step ramp this palette deleted        10.9 / 10.7");

/*
 * And what did ship: `--link`, derived rather than chosen.
 *
 * The rule is this palette's own, and `palette.spec.ts` checks it reproduces on the tokens that
 * were already here: chroma held, only lightness moved, until the tightest of the three surfaces
 * reaches 5:1. The hue is the bisector of the arc the palette does not spend — as far from both
 * series hues as one hue can be, which is the most that is available once the sweep above has
 * shown that "far from both" is worth much less than it sounds.
 *
 * 5:1 and not the 3:1 a mark would need, because `--link` is also the word under the cursor.
 */
const bisector =
  (((toOklch(parseHex(get("light", "--series-formula-text"))).h +
    toOklch(parseHex(get("light", "--series-guarantee-text"))).h) /
    2) +
    180) %
  360;

/** The least move that reaches the bar: down in light, up in dark. */
function derive(hue: number, chromaValue: number, mode: Mode, bar: number): string | null {
  for (let step = 0; step <= 440; step += 1) {
    const l = mode === "light" ? 0.98 - step * 0.002 : 0.1 + step * 0.002;
    const rgb = fromOklch({ l, c: chromaValue, h: hue });
    if (!rgb) continue;
    const hex = toHex(rgb);
    if (worstContrast(hex, mode) >= bar) return hex;
  }
  return null;
}

console.log(
  `\nWhat shipped — --link on the unspent bisector, ${bisector.toFixed(1)} deg, chroma ${CHROMA}:`,
);
for (const mode of ["light", "dark"] as const) {
  const hex = derive(bisector, CHROMA, mode, CONTRAST_FLOOR)!;
  const ok = toOklch(parseHex(hex));
  console.log(
    `  ${mode.padEnd(5)} ${hex}   tightest surface ${worstContrast(hex, mode).toFixed(3)}:1   ` +
      `OKLCH ${ok.l.toFixed(4)} ${ok.c.toFixed(4)} ${ok.h.toFixed(2)}   ` +
      `${hueGap(ok.h, toOklch(parseHex(get(mode, "--series-formula"))).h).toFixed(1)} / ` +
      `${hueGap(ok.h, toOklch(parseHex(get(mode, "--series-guarantee"))).h).toFixed(1)} deg from the two series hues`,
  );
}
