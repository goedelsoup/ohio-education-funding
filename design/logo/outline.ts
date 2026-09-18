/**
 * The site logo, derived rather than drawn.
 *
 * The mark is the state of Ohio with the Indiana meridian drawn as a pencil. That is not a
 * decoration laid on top of the map: Ohio's western boundary is the one perfectly straight edge the
 * state has, a surveyed meridian running from the Michigan line to the Ohio River, and the pencil
 * occupies exactly it. The outer silhouette of the mark is therefore still the state — the pencil
 * is a colour change inside the boundary, not a shape added outside it — and the only thing that
 * leaves the outline is the point, which finishes below the corner where the meridian meets the
 * river at the Indiana, Kentucky and Ohio tripoint.
 *
 * # Why this file exists rather than an SVG somebody drew
 *
 * The outline is thirty-six coordinates, each one a named place, projected and fitted here. A
 * hand-drawn path is a shape nobody can check; this one can be re-derived, and a coordinate that is
 * wrong is wrong at a place with a name. `--check` re-emits every committed file and diffs it, so
 * an edit made to the SVG instead of to the source fails rather than drifting.
 *
 *   node design/logo/outline.ts            # write the SVGs
 *   node design/logo/outline.ts --check    # fail if the committed files disagree
 *
 * # The colours are the site's, written as literals
 *
 * Same reason `web/public/favicon.svg` and `web/src/lib/og/palette.ts` write them as literals: an
 * SVG served straight to a browser tab has no stylesheet, and neither does the resvg pass that
 * rasterizes it for the PNG icon routes. The values here are the two categorical series and the
 * primary ink, in both themes, and `palette.spec.ts` is what holds the stylesheet to them.
 *
 * (The custom property names those literals come from cannot be spelled out in an SVG comment. XML
 * forbids a double hyphen inside one — the same rule `favicon.svg` records having broken once.)
 */

import { writeFileSync, readFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

/**
 * The boundary, clockwise from the northwest corner, as (longitude, latitude).
 *
 * Four runs, and the mark depends on knowing which is which: the Harris line east to Maumee Bay,
 * the Lake Erie shore, the Pennsylvania meridian south, the Ohio River west and then north, and the
 * Indiana meridian closing it. Simplified for a mark rather than for a map — Marblehead and
 * Sandusky Bay are one shoulder here, because at 16 pixels the peninsula is a stray pixel and at
 * 180 it is a wobble that reads as an error in the drawing.
 */
const BOUNDARY: ReadonlyArray<readonly [number, number, string]> = [
  [-84.804, 41.700, "northwest corner: Indiana, Michigan, Ohio"],
  [-83.450, 41.733, "the Harris line meets Maumee Bay"],
  [-83.280, 41.690, "Maumee Bay"],
  [-83.100, 41.600, "Locust Point"],
  [-82.940, 41.512, "Port Clinton"],
  [-82.720, 41.470, "Marblehead and Sandusky Bay, taken as one shoulder"],
  [-82.450, 41.410, "Huron and Vermilion"],
  [-82.180, 41.460, "Lorain"],
  [-81.700, 41.500, "Cleveland"],
  [-81.280, 41.760, "the Mentor headlands"],
  [-80.950, 41.850, "Geneva"],
  [-80.790, 41.900, "Ashtabula"],
  [-80.560, 41.960, "Conneaut"],
  [-80.519, 41.978, "northeast corner, at the Pennsylvania line"],
  [-80.519, 40.638, "Pennsylvania, West Virginia, Ohio: East Liverpool"],
  [-80.630, 40.370, "Steubenville"],
  [-80.750, 40.190, "the Ohio River above Wheeling"],
  [-80.700, 40.070, "opposite Wheeling"],
  [-80.850, 39.920, "the river below Wheeling"],
  [-81.120, 39.740, "Newport"],
  [-81.300, 39.550, "above Marietta"],
  [-81.450, 39.420, "Marietta"],
  [-81.750, 39.220, "Belpre to Pomeroy"],
  [-82.030, 39.030, "Pomeroy"],
  [-82.150, 38.900, "Middleport"],
  [-82.200, 38.810, "Gallipolis"],
  [-82.370, 38.560, "the river above South Point"],
  [-82.510, 38.400, "South Point: the southernmost place in the state"],
  [-82.680, 38.520, "Ironton"],
  [-82.990, 38.730, "Portsmouth"],
  [-83.300, 38.630, "the river below Portsmouth"],
  [-83.770, 38.630, "opposite Maysville"],
  [-83.850, 38.740, "Ripley"],
  [-84.200, 38.790, "New Richmond"],
  [-84.500, 39.090, "Cincinnati"],
  [-84.820, 39.107, "Indiana, Kentucky, Ohio: the foot of the meridian"],
];

/**
 * Longitude, flattened at the state's middle latitude.
 *
 * A mark this size does not need a conic projection and would not show one. What it does need is
 * for Ohio not to come out fat: a degree of longitude at 40 degrees north covers cos(40) of what a
 * degree of latitude does, so leaving the factor out draws the state 30.5% too wide, which is
 * enough for the silhouette to read as some other state.
 */
const K = Math.cos((40.0 * Math.PI) / 180.0);

/** The icon grid. 64 with a 6-unit margin, which is what `favicon.svg` already uses. */
const BOX = 64;
const PAD = 6;

/** The pencil: as wide as the meridian strip it occupies, with a point below the tripoint. */
const PENCIL_W = 7.0;
const POINT = 0.9; // how far down towards the state's southern extreme the point reaches
const GRAPHITE = 0.72; // how much of the cone is graphite rather than painted barrel

/**
 * The two marks that make the pencil an object rather than a stripe.
 *
 * Flush against the state, an orange rectangle with a point on it is read as a band of colour in
 * the silhouette before it is read as a pencil. `KEYLINE` is a hairline of the ground between the
 * barrel and the state, which is what says the pencil is lying ON the map; `FACET` is the barrel's
 * highlight, at a third of the width, which is where a hexagonal pencil's facet edge falls.
 *
 * Both are narrower than a device pixel below about 90px, and that is fine HERE in a way it is not
 * for the calculator's keypad. A sub-pixel line still renders as partial coverage — a lighter
 * blend along the edge, which is all either of these has to be. The keypad's gaps have to separate
 * six shapes into six readable objects, and a blend cannot do that. When these two do finally
 * merge, what is left is the mark without them, which is a pencil; when the keypad's gaps merge,
 * what is left is a pale block.
 */
const KEYLINE = 0.9;
const FACET_AT = 0.33;
const FACET_W = 0.75;

type Pt = { x: number; y: number };

/** Project, then fit inside the padded box preserving aspect. */
function fitted(): Pt[] {
  const xs = BOUNDARY.map(([lon]) => lon * K);
  const ys = BOUNDARY.map(([, lat]) => -lat);
  const w = Math.max(...xs) - Math.min(...xs);
  const h = Math.max(...ys) - Math.min(...ys);
  const s = Math.min((BOX - 2 * PAD) / w, (BOX - 2 * PAD) / h);
  const ox = PAD + (BOX - 2 * PAD - w * s) / 2 - Math.min(...xs) * s;
  const oy = PAD + (BOX - 2 * PAD - h * s) / 2 - Math.min(...ys) * s;
  return xs.map((x, i) => ({ x: x * s + ox, y: ys[i] * s + oy }));
}

const n = (v: number) => v.toFixed(2);

function polygon(pts: Pt[]): string {
  return pts.map((p, i) => `${i ? "L" : "M"}${n(p.x)} ${n(p.y)}`).join(" ") + "Z";
}

/**
 * The two faces, and dark is a selected palette rather than an inversion — the rule
 * `tokens/colors.css` states and this file has to follow, because it restates the values.
 */
const LIGHT = { surface: "#fcfcfb", state: "#2a78d6", pencil: "#eb6834", graphite: "#0b0b0b" };
const DARK = { surface: "#1a1a19", state: "#3987e5", pencil: "#d95926", graphite: "#ffffff" };

/**
 * The drawing, without a document around it.
 *
 * Three shapes and no more. The state, the pencil over the meridian strip, and the graphite at the
 * point. The pencil has no eraser and no ferrule: a dark block at the top gives the mark two dark
 * ends, and at 32 pixels a reader cannot tell which end writes.
 */
function shapes(c: typeof LIGHT): string {
  const pts = fitted();
  const west = pts[0].x; // the meridian, at the top where it leaves the Michigan line
  const top = Math.min(...pts.map((p) => p.y));
  const foot = pts[pts.length - 1].y; // where the meridian meets the river
  const low = Math.max(...pts.map((p) => p.y));
  const point = foot + (low - foot) * POINT;
  const shoulder = point - (point - foot) * GRAPHITE;
  return [
    `<path class="state" d="${polygon(pts)}" fill="${c.state}"/>`,
    // Before the barrel, so the barrel's own edge stays clean and the line reads as ground showing
    // through rather than as a stroke drawn onto the pencil.
    `<rect class="keyline" x="${n(west + PENCIL_W)}" y="${n(top)}" width="${n(KEYLINE)}" ` +
      `height="${n(foot - top)}" fill="${c.surface}"/>`,
    `<path class="pencil" d="M${n(west)} ${n(top)} H${n(west + PENCIL_W)} V${n(foot)} ` +
      `L${n(west + PENCIL_W / 2)} ${n(point)} L${n(west)} ${n(foot)}Z" fill="${c.pencil}"/>`,
    `<rect class="facet" x="${n(west + PENCIL_W * FACET_AT)}" y="${n(top)}" ` +
      `width="${n(FACET_W)}" height="${n(foot - top)}" fill="${c.surface}"/>`,
    `<path class="graphite" d="M${n(west + PENCIL_W * 0.24)} ${n(shoulder)} ` +
      `L${n(west + PENCIL_W / 2)} ${n(point)} L${n(west + PENCIL_W * 0.76)} ${n(shoulder)}Z" ` +
      `fill="${c.graphite}"/>`,
  ].join("\n  ");
}

/**
 * # The second concept: the state as the calculator
 *
 * The Fair School Funding Plan is, literally, a workbook — `fy26-calculator.xlsx` and
 * `fy27-calculator.xlsx` are in this repository's corpus, and every figure the site publishes is
 * something that workbook computes. So a calculator is not a generic "school" motif borrowed from
 * a clip-art sheet; it is the instrument the subject is actually about.
 *
 * The face is a display over a keypad, and it is PLACED rather than positioned by eye: the largest
 * rectangle at a calculator's proportion that fits inside the outline, centred on the state's own
 * area centroid. That matters because the state's interior is not a rectangle. The largest
 * rectangle that fits anywhere hugs the western meridian and is 35.5 by 30.6 — and a face placed
 * there sits visibly left of the mass, because the eye centres on the middle of the shape rather
 * than on the middle of the biggest hole in it.
 */

/** A calculator is taller than it is wide. This is that proportion, as width over height. */
const FACE_ASPECT = 0.85;

/** Ray casting. Whether a point falls inside the fitted outline. */
function contains(pts: Pt[], x: number, y: number): boolean {
  let inside = false;
  for (let i = 0; i < pts.length; i += 1) {
    const a = pts[i];
    const b = pts[(i + 1) % pts.length];
    if (a.y > y !== b.y > y && x < a.x + ((y - a.y) * (b.x - a.x)) / (b.y - a.y)) inside = !inside;
  }
  return inside;
}

/** The area centroid, by the shoelace form. Not the bounding box's middle, which is not the same. */
function centroid(pts: Pt[]): Pt {
  let a = 0;
  let cx = 0;
  let cy = 0;
  for (let i = 0; i < pts.length; i += 1) {
    const p = pts[i];
    const q = pts[(i + 1) % pts.length];
    const cross = p.x * q.y - q.x * p.y;
    a += cross;
    cx += (p.x + q.x) * cross;
    cy += (p.y + q.y) * cross;
  }
  a /= 2;
  return { x: cx / (6 * a), y: cy / (6 * a) };
}

/**
 * The face plate.
 *
 * Feasibility is monotone in width — the height follows from the aspect, so a wider plate is
 * strictly harder to fit — which is what makes the binary search below valid rather than merely
 * fast. Containment is tested on the perimeter: a rectangle whose whole edge is inside a simple
 * polygon is inside it.
 */
function plate(pts: Pt[]): { x: number; y: number; w: number; h: number } {
  const cx = centroid(pts).x;
  const top = Math.min(...pts.map((p) => p.y));
  const bottom = Math.max(...pts.map((p) => p.y));
  const fits = (w: number): number | null => {
    const h = w / FACE_ASPECT;
    for (let y = top; y + h <= bottom; y += 0.2) {
      let ok = true;
      for (let t = 0; t <= 8 && ok; t += 1) {
        const s = t / 8;
        ok =
          contains(pts, cx - w / 2 + s * w, y) &&
          contains(pts, cx - w / 2 + s * w, y + h) &&
          contains(pts, cx - w / 2, y + s * h) &&
          contains(pts, cx + w / 2, y + s * h);
      }
      if (ok) return y;
    }
    return null;
  };
  let lo = 4;
  let hi = 48;
  let best = { x: 0, y: 0, w: 0, h: 0 };
  for (let i = 0; i < 24; i += 1) {
    const mid = (lo + hi) / 2;
    const y = fits(mid);
    if (y === null) hi = mid;
    else {
      lo = mid;
      best = { x: cx - mid / 2, y, w: mid, h: mid / FACE_ASPECT };
    }
  }
  return best;
}

/** The display takes this much of the plate's height; the rest is keypad. */
const DISPLAY = 0.3;
const FACE_GAP = 2.5;
const KEY_COLS = 3;
const KEY_ROWS = 2;

/**
 * The calculator face, drawn on the state.
 *
 * One accent and it goes on the display, which is the largest element after the state itself and
 * therefore the one that survives the most reduction. The alternative — a light display with the
 * equals key in the accent — is the more literal calculator and loses at small sizes twice over:
 * the key becomes a two-pixel dot, and a light display with nothing in it reads as a hole in the
 * state rather than as a screen.
 */
function calculator(c: typeof LIGHT): string {
  const pts = fitted();
  const f = plate(pts);
  const dh = f.h * DISPLAY;
  const kw = (f.w - FACE_GAP * (KEY_COLS - 1)) / KEY_COLS;
  const kh = (f.h - dh - FACE_GAP * KEY_ROWS) / KEY_ROWS;
  const out = [
    `<path class="state" d="${polygon(pts)}" fill="${c.state}"/>`,
    `<rect class="display" x="${n(f.x)}" y="${n(f.y)}" width="${n(f.w)}" height="${n(dh)}" ` +
      `rx="2" fill="${c.pencil}"/>`,
  ];
  for (let row = 0; row < KEY_ROWS; row += 1) {
    for (let col = 0; col < KEY_COLS; col += 1) {
      out.push(
        `<rect class="key" x="${n(f.x + col * (kw + FACE_GAP))}" ` +
          `y="${n(f.y + dh + FACE_GAP + row * (kh + FACE_GAP))}" ` +
          `width="${n(kw)}" height="${n(kh)}" rx="1.6" fill="${c.surface}"/>`,
      );
    }
  }
  return out.join("\n  ");
}

/** The dark face, as a style block. See the note in `favicon.svg` about what does not read it. */
/**
 * The dark face, as a style block, carrying only the classes the drawing it wraps actually uses.
 *
 * The two concepts are competing proposals rather than one family, so they do not share a shape
 * vocabulary: a rule for `.pencil` in the calculator's file would match nothing, which is the same
 * dead rule `app.css` once carried for a retired claim mark and had to be found by hand later.
 *
 * See the note in `favicon.svg` about what does not read this block.
 */
const darkBlock = (...classes: Array<keyof typeof DARK_FILL>) =>
  `  <style>\n` +
  `    @media (prefers-color-scheme: dark) {\n` +
  classes.map((c) => `      .${c} { fill: ${DARK_FILL[c]}; }\n`).join("") +
  `    }\n` +
  `  </style>\n`;

/** Which token each drawn class takes in the dark face. A key is a hole, so it takes the ground. */
const DARK_FILL = {
  surface: DARK.surface,
  state: DARK.state,
  pencil: DARK.pencil,
  graphite: DARK.graphite,
  display: DARK.pencil,
  key: DARK.surface,
  keyline: DARK.surface,
  facet: DARK.surface,
} as const;

const head = (concept: string) => `<!--
  GENERATED by design/logo/outline.ts. Edit that file, not this one: the check mode of that script
  re-emits every file here and fails on one that was edited in place.

  (The flag that runs it cannot be named in this comment. XML forbids a double hyphen inside one,
  which this file learned by failing to render rather than by failing to build.)

  ${concept}

  The prefers-color-scheme block is the dark face. resvg does not evaluate media queries, so
  anything rasterizing this gets the light one, which is correct for a home screen whose background
  the site does not control.
-->
`;

const PENCIL_NOTE =
  "The state, with the Indiana meridian drawn as a pencil. The silhouette is still Ohio: the\n" +
  "  pencil occupies the meridian strip rather than sitting beside it, and only the point leaves\n" +
  "  the boundary.";

/**
 * The site's icon carries a second note, because of where it sits rather than what it draws.
 *
 * `public/favicon.svg` is the single source for the icon at every size — `src/lib/og/mark.ts`
 * reads it and the two PNG routes rasterize it — so it is the one output of this script that
 * something else in the repository depends on by path. It is emitted here rather than copied here,
 * and `mise run //:generated` runs the check, for the reason that file's own predecessor gave
 * about hand-exported PNGs: the way an agreement between two copies breaks is invisible to
 * whoever broke it.
 */
const FAVICON_NOTE =
  PENCIL_NOTE +
  "\n\n  This file is the site's icon at every size: `src/lib/og/mark.ts` reads it, and\n" +
  "  `icon-32.png` and `apple-touch-icon.png` are rasterized from it. `mise run //:generated`\n" +
  "  fails if it drifts from its source.";

const CALC_NOTE =
  "The state as the calculator the funding plan actually is. The display and keypad are placed\n" +
  "  rather than positioned: the largest rectangle at a calculator's proportion that fits inside\n" +
  "  the outline, centred on the state's own area centroid.";

function icon(note = PENCIL_NOTE): string {
  return (
    head(note) +
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${BOX} ${BOX}" role="img" ` +
    `aria-label="Ohio school funding">\n` +
    darkBlock("surface", "state", "keyline", "pencil", "facet", "graphite") +
    `  <rect class="surface" width="${BOX}" height="${BOX}" rx="12" fill="${LIGHT.surface}"/>\n` +
    `  ${shapes(LIGHT)}\n</svg>\n`
  );
}

/** The same drawing with no ground under it, for a header or a lockup. */
function bare(): string {
  return (
    head(PENCIL_NOTE) +
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${BOX} ${BOX}" role="img" ` +
    `aria-label="Ohio school funding">\n` +
    darkBlock("state", "keyline", "pencil", "facet", "graphite") +
    `  ${shapes(LIGHT)}\n</svg>\n`
  );
}

/**
 * The horizontal lockup.
 *
 * The wordmark is set in the body stack, NOT in the display serif, and that is the site's own rule
 * rather than a preference: `tokens/typography.css` reserves the serif for headings and the lead
 * and excludes anything "drawn inside an `<svg>`", because a platform stack is a promise an SVG
 * cannot keep — the file renders in whatever the reading machine happens to have. The preview card
 * gets a serif headline because satori turns it into outlines first; this file has no such step.
 */
function lockup(): string {
  const H = 44;
  const markH = 36;          // the mark sits to the wordmark, not to the full height of the box
  const gap = 12;
  const w = markH + gap + 214;
  return (
    head(PENCIL_NOTE) +
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${H}" role="img" ` +
    `aria-label="Ohio school funding">\n` +
    `  <style>\n` +
    `    .word { font-family: "IBM Plex Sans", ui-sans-serif, system-ui, -apple-system, ` +
    `"Segoe UI", Roboto, sans-serif; fill: #0b0b0b; }\n` +
    `    .lede { font-size: 19px; font-weight: 600; letter-spacing: -0.012em; }\n` +
    `    .tail { font-size: 19px; font-weight: 400; }\n` +
    `    @media (prefers-color-scheme: dark) {\n` +
    `      .state { fill: ${DARK.state}; }\n` +
    `      .keyline { fill: ${DARK.surface}; }\n` +
    `      .pencil { fill: ${DARK.pencil}; }\n` +
    `      .facet { fill: ${DARK.surface}; }\n` +
    `      .graphite { fill: ${DARK.graphite}; }\n` +
    `      .word { fill: #ffffff; }\n` +
    `    }\n` +
    `  </style>\n` +
    `  <g transform="translate(0 ${((H - markH) / 2).toFixed(2)}) ` +
    `scale(${(markH / BOX).toFixed(4)})">\n    ${shapes(LIGHT)}\n  </g>\n` +
    // `xml:space` and an ordinary space, rather than the non-breaking entity that reads as the
    // obvious fix. SVG collapses the ordinary space and the two words run together — but librsvg,
    // which is one of the rasterizers an asset like this gets thrown at, drops the entity too and
    // renders "Ohioschool". Only the attribute is honoured by both. Giving the second tspan its own
    // `x` would work everywhere and is worse: it hard-codes a measurement of the word before it,
    // which is a different font's business on every machine that opens this file.
    `  <text class="word" xml:space="preserve" x="${markH + gap}" y="29">` +
    `<tspan class="lede">Ohio</tspan><tspan class="tail"> school funding</tspan></text>\n` +
    `</svg>\n`
  );
}

/** The calculator, on its rounded ground. */
function calcIcon(): string {
  return (
    head(CALC_NOTE) +
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${BOX} ${BOX}" role="img" ` +
    `aria-label="Ohio school funding">\n` +
    darkBlock("surface", "state", "display", "key") +
    `  <rect class="surface" width="${BOX}" height="${BOX}" rx="12" fill="${LIGHT.surface}"/>\n` +
    `  ${calculator(LIGHT)}\n</svg>\n`
  );
}

/** The calculator with no ground under it. */
function calcBare(): string {
  return (
    head(CALC_NOTE) +
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${BOX} ${BOX}" role="img" ` +
    `aria-label="Ohio school funding">\n` +
    darkBlock("state", "display", "key") +
    `  ${calculator(LIGHT)}\n</svg>\n`
  );
}

const HERE = dirname(fileURLToPath(import.meta.url));
const FILES: ReadonlyArray<readonly [string, () => string]> = [
  ["../../web/public/favicon.svg", () => icon(FAVICON_NOTE)],
  ["mark-bare.svg", bare],
  ["lockup.svg", lockup],
  ["calculator.svg", calcIcon],
  ["calculator-bare.svg", calcBare],
];

const check = process.argv.includes("--check");
let bad = 0;
for (const [name, build] of FILES) {
  const want = build();
  const path = join(HERE, name);
  if (check) {
    let have = "";
    try {
      have = readFileSync(path, "utf8");
    } catch {
      have = "";
    }
    if (have !== want) {
      // Named from the repository root rather than from this directory. The favicon's entry is
      // written as `../../web/public/favicon.svg`, and that is not a path anyone reading a red
      // gate wants to resolve in their head.
      const shown = relative(join(HERE, "../.."), path);
      console.error(`${shown} does not match what design/logo/outline.ts emits. Re-run it.`);
      bad += 1;
    }
  } else {
    writeFileSync(path, want);
    console.log(`wrote ${name}`);
  }
}
if (check && bad === 0) console.log(`${FILES.length} files match.`);
process.exit(bad === 0 ? 0 : 1);
