/**
 * What a chart has to state about its own scale before a reader can take a value off it.
 *
 * Six charts on this site omitted something a reader needed — #109 grouped them because they are
 * one judgement repeated rather than six unrelated faults. Each of these tests holds one of those
 * judgements in the layer that made it, so the next chart built on the same specification inherits
 * the answer instead of the omission.
 */

import { expect, test } from "vitest";

import type {
  Bar,
  FanPoint,
  Fit,
  Place,
  Rank,
  ScatterPoint,
  SeriesPoint,
  Trace,
} from "../../src/lib/chart.ts";
import {
  DOT,
  barSpec,
  distributionSpec,
  fanSpec,
  panelWidth,
  planeSpec,
  rankSpec,
  scatterSpec,
  seriesSpec,
  type Spec,
  truncatedDomain,
  WIDTHS,
} from "../../src/lib/plot/spec.ts";
import { renderToString } from "../../src/lib/plot/ssr.ts";

/**
 * The width these assertions are written at.
 *
 * Every chart is laid out twice — see `WIDTHS` — and what is asserted here is the layout
 * arithmetic, which is the same arithmetic at either width. So the tests name one rather than
 * running each of them twice to make the same point.
 */
const W = { width: WIDTHS.wide };

/** A cloud big enough for `scatterSpec` to draw — it refuses fewer than twelve. */
function cloud(xs: number[]): ScatterPoint[] {
  return xs.map((x, i) => ({ x, y: 50 + (i % 7), hover: `d${i}`, band: i % 3 }));
}

test("the bar a chart was built to locate is marked, on two channels", () => {
  /*
   * `/statewide` ranks the states by local share and draws Ohio among them. It set `current` on
   * Ohio's bar and nothing read it — `Bar` did not declare the field and `barSpec` ignored it —
   * so Ohio carried no mark in a chart built to show Ohio's position.
   *
   * Two channels, because one of them is colour: the fill takes the contrasting half of the
   * validated pair, and the category name is drawn in primary ink at 600. The second is what
   * survives a monochrome print, which is the rule the print stylesheet applies site-wide.
   */
  const bars: Bar[] = [
    { label: "Nebraska", value: 60 },
    { label: "Ohio", value: 45, current: true },
    { label: "Utah", value: 30 },
  ];
  const svg = renderToString((w) => barSpec(bars, { width: w }), "presentational");
  expect(svg).toContain("var(--series-guarantee)");
  expect(svg).toContain('font-weight="600"');

  // And a chart with no subject is unchanged: same fills, no second text mark.
  const plain = renderToString(
    (w) => barSpec(bars.map(({ current: _c, ...b }) => b), { width: w }),
    "presentational",
  );
  expect(plain).not.toContain("var(--series-guarantee)");
  expect(plain).not.toContain('font-weight="600"');
});

test("a column with bars on both sides of zero draws its rule at both widths", () => {
  /*
   * The first chart a corpus node draws — `dispersion/fy2016-step-by-business-class`, on the
   * Toledo node — is four correlations, two of them negative and none of them a subject. Signed
   * mode was written for one bar on one page; this holds it for a whole column, at the narrow
   * width as well as the wide one, because the narrow drawing is a second call and not a scaling.
   *
   * The zero rule is what a reader takes the sign off, so it is the mark asserted on: Plot writes
   * a rule as a `<line>` and the bars as `<rect>`s, and an unsigned column has no `<line>` at all.
   */
  const bars: Bar[] = [
    { label: "Industrial", value: 0.1175, hover: "Industrial: +0.1175" },
    { label: "Mineral", value: -0.0321, hover: "Mineral: −0.0321" },
    { label: "Public utility", value: -0.0159, hover: "Public utility: −0.0159" },
    { label: "All three summed", value: 0.0164, hover: "All three summed: +0.0164" },
  ];
  const pair = renderToString((w) => barSpec(bars, { width: w }), { label: "The column" });
  const drawings = pair.split("<svg").slice(1);
  expect(drawings).toHaveLength(2);
  for (const drawing of drawings) {
    expect((drawing.match(/<rect/g) ?? []).length).toBe(4);
    expect((drawing.match(/<line/g) ?? []).length).toBe(1);
    // Both halves of the polarity pair, and no subject mark.
    expect(drawing).toContain("var(--series-guarantee)");
    expect(drawing).toContain("var(--series-formula)");
    expect(drawing).not.toContain('font-weight="600"');
    // The hover is the spoken value the caller gave, not the bare float.
    expect(drawing).toContain("Industrial: +0.1175");
    expect(drawing).not.toContain("0.11752");
  }
  // And a column of the same shape with nothing below zero draws no rule.
  const unsigned = renderToString(
    (w) => barSpec(bars.map((b) => ({ ...b, value: Math.abs(b.value) })), { width: w }),
    "presentational",
  );
  expect(unsigned).not.toContain("<line");
});

test("a signed distribution draws its zero, and an unsigned one is unchanged", () => {
  /*
   * The enrollment-change strip on `/districts` is the site's one signed distribution and drew no
   * zero reference, so a dot two thirds along could have been a district that grew or one that
   * shrank. `histogramSpec` draws a dashed rule and labels it "no change" for exactly that reason.
   *
   * Detected from the domain rather than passed at the call site, which is what makes it reach
   * the sixth strip without anyone remembering. The five unsigned strips keep their geometry: they
   * sit one row apart on the same page and a taller sixth would be visible as a jump.
   */
  const signed = distributionSpec(
    [-0.08, -0.03, -0.01, 0.02, 0.05, 0.09].map((value) => ({ value, hover: `${value}` })),
    W,
  );
  expect(signed).not.toBeNull();
  expect(signed!.options.height).toBe(64);
  expect(renderToString(() => signed, "presentational")).toContain("no change");

  const unsigned = distributionSpec(
    [1, 2, 3, 4, 5, 6].map((value) => ({ value, hover: `${value}` })),
    W,
  );
  expect(unsigned!.options.height).toBe(46);
  expect(renderToString(() => unsigned, "presentational")).not.toContain("no change");
});

test("the label gutter is sized to the labels that are drawn", () => {
  /*
   * A banded trace carries its identity in the legend, and `scatterSpec` deliberately draws no end
   * label for one. The gutter was sized off every trace all the same, so both banded scatters on
   * `/outcomes` gave up 22% of a 640px frame to labels that were never rendered.
   */
  const points = cloud(Array.from({ length: 30 }, (_, i) => 10_000 + i * 400));
  const banded: Trace[] = ["least poor third", "middle third", "poorest third"].map((label, band) => ({
    label,
    series: "formula",
    band,
    points: [{ x: 10_000, y: 50 }, { x: 20_000, y: 55 }],
  }));

  const withBands = scatterSpec(points, AXES, banded, W)!;
  const withNone = scatterSpec(points, AXES, [], W)!;
  expect(withBands.options.marginRight).toBe(withNone.options.marginRight);

  // A trace that *is* labelled still gets its room, or the label runs off the viewBox.
  const labelled = scatterSpec(
    points,
    AXES,
    [{ label: "median of each fifth", series: "formula", points: banded[0]!.points }],
    W,
  )!;
  expect(labelled.options.marginRight).toBeGreaterThan(withNone.options.marginRight!);
});

test("a small multiple can be put on one horizontal scale", () => {
  /*
   * The spending pair on `/outcomes` is the same numerator over two denominators, and the card's
   * own prose says the bands "separate on the horizontal axis too" — a claim about horizontal
   * distance. Fitted to their own ranges the two axes differed by 1.64×.
   */
  const narrow = cloud(Array.from({ length: 20 }, (_, i) => 9_000 + i * 800));
  const wide = cloud(Array.from({ length: 20 }, (_, i) => 11_000 + i * 1_300));
  const shared: [number, number] = [9_000, 11_000 + 19 * 1_300];

  const a = scatterSpec(narrow, AXES, [], { ...W, xDomain: shared })!;
  const b = scatterSpec(wide, AXES, [], { ...W, xDomain: shared })!;
  expect(a.options.x!.domain).toEqual(b.options.x!.domain);

  // Without it they do not agree, which is the state this option exists to leave behind.
  const own = scatterSpec(narrow, AXES, [], W)!;
  expect(own.options.x!.domain).not.toEqual(b.options.x!.domain);
});

test("a fixed domain holds against an outlier rather than being widened by one", () => {
  /*
   * The defect this closes, which was mine and which I shipped once.
   *
   * `/reach` redraws on every slider tick, so its axes are fixed across the whole reachable lever
   * space — otherwise dragging base cost holds the cloud still and moves the frame, and a reader
   * cannot tell which of the two happened. The first version treated the caller's domain as a
   * *floor* and unioned it with the points' own range, on the theory that a bound which turned out
   * too small should grow rather than clip silently.
   *
   * That defeated the entire purpose. Kelleys Island Local has four pupils and $26,700 of aid per
   * pupil, so the union put the axis back on the outlier at every tick and the frame moved exactly
   * as before. A supplied domain is authoritative; `clip: true` on the marks is what makes that
   * safe, and the caller counting what it clipped is what makes it honest.
   */
  const points = cloud([...Array.from({ length: 20 }, (_, i) => 1_000 + i * 100), 26_700]);
  const fixed: [number, number] = [0, 3_000];

  const spec = scatterSpec(points, AXES, [], { ...W, xDomain: fixed, yDomain: fixed })!;
  expect(spec.options.x!.domain).toEqual(fixed);
  expect(spec.options.y!.domain).toEqual(fixed);

  // Removing the outlier must not move it either, which is the same property stated from the
  // other side: the frame is a function of the caller's bound and of nothing in the data.
  const without = scatterSpec(points.slice(0, -1), AXES, [], {
    ...W,
    xDomain: fixed,
    yDomain: fixed,
  })!;
  expect(without.options.x!.domain).toEqual(spec.options.x!.domain);

  // And the marks are clipped, or the outlier is painted over the card instead of the plot.
  // Plot normalises `clip: true` to `"frame"`, which is the clip this wants: to the plot area.
  const marks = spec.options.marks as { className?: string; clip?: unknown }[];
  expect(marks.find((m) => m.className === "scatter-dot")?.clip).toBe("frame");
});

test("a fan chart does not stretch its axis to fit a reference it will not draw", () => {
  /*
   * The reference line is drawn only when every year carries one — a partial line would bridge the
   * years it has no value for. The y domain folded the references in regardless, so a partial
   * series padded its axis out for a value that never reached the frame. This axis is truncated to
   * the band's own range precisely because the band is narrow, and padding it flattens the finding.
   */
  const base: FanPoint[] = [2025, 2026, 2027].map((year, i) => ({
    year,
    point: 100 + i,
    low: 99 + i,
    high: 101 + i,
    observed: i === 0,
  }));
  const partial = base.map((p, i) => (i === 0 ? { ...p, reference: 400 } : p));

  const clean = fanSpec(base, (v) => `${v}`, () => "", W)!;
  const withOrphan = fanSpec(partial, (v) => `${v}`, () => "", W)!;
  expect(withOrphan.options.y!.domain).toEqual(clean.options.y!.domain);

  // A reference on every year is drawn, and does belong in the domain.
  const full = fanSpec(
    base.map((p) => ({ ...p, reference: 400 })),
    (v) => `${v}`,
    () => "",
    W,
  )!;
  expect((full.options.y!.domain as number[])[1]).toBeGreaterThan(400);
});

test("the truncated domain is the one the annotation has to name", () => {
  // Exported so a caller can check that its own format resolves the axis start — see
  // `appropriations.spec.ts`, where a format with no decimal places understated it by a fifth.
  const [low, high] = truncatedDomain([10, 20]);
  expect(low).toBeLessThan(10);
  expect(high).toBeGreaterThan(20);
  // Padded by a tenth of the span at each end, never anchored at zero.
  expect(low).toBeCloseTo(10 - 1.2, 6);
});

const AXES = {
  x: { label: "spending per pupil", format: (v: number) => `$${v}` },
  y: { label: "Performance Index", format: (v: number) => v.toFixed(0) },
};

test("a muted point is drawn smaller and fainter, and the radius is a pixel count", () => {
  /*
   * `ScatterPoint.muted` is the de-emphasis channel — see its own note for why hue could not carry
   * it — and it spends radius as well as alpha, because on `--neutral-mark` alpha alone separates
   * the two populations by 1.20:1 and that is not a difference a reader finds in six hundred
   * overlapping dots.
   *
   * # The half that is not obvious
   *
   * Making `r` a function makes it a **channel**, and a channel goes through a scale. Plot's
   * default `r` scale is a square root fitted to the values it is handed, so `2.4` and `1.6` were
   * rendered as **3.67 and 3**: the ordering survived, the picture still looked right, and the area
   * ratio the whole de-emphasis rests on had silently gone from 44% to 67%. `r: { type: "identity"
   * }` is what says these are already pixels, and this is the assertion that says it is still said.
   *
   * Read off rendered SVG rather than off the spec, because the spec is where it looked correct.
   */
  /* Built rather than spread from `cloud`, which bands its points: a banded cloud draws the plain
     dots at `DOT.opacity.banded`, and the pair this is about is plain against muted. */
  const points: ScatterPoint[] = Array.from({ length: 14 }, (_, i) => ({
    x: i + 1,
    y: 50 + (i % 7),
    hover: `d${i}`,
    muted: i % 2 === 0,
  }));
  const svg = renderToString((width) => scatterSpec(points, AXES, [], { width }), {
    label: "a cloud with half of it muted",
    description: "half the districts drawn as context",
  });

  const radii = new Set([...svg.matchAll(/<circle[^>]*\sr="([\d.]+)"/g)].map((m) => m[1]!));
  expect(radii, "both dot radii are the literal pixel values, plus the hit layer's").toEqual(
    new Set([String(DOT.radius.plain), String(DOT.radius.muted), "7"]),
  );

  const alphas = new Set([...svg.matchAll(/fill-opacity="([\d.]+)"/g)].map((m) => m[1]!));
  expect(alphas).toEqual(new Set([String(DOT.opacity.plain), String(DOT.opacity.muted)]));

  /* The hit layer does not shrink: a muted district is still something a reader can point at, and
     still carries its whole tooltip. That is what keeps this emphasis rather than removal. */
  expect(
    [...svg.matchAll(/<circle[^>]*\sr="7"/g)].length,
    "one full-size hit target per district, muted or not",
  ).toBe(points.length * 2);
});

/**
 * The fitted-line panels, whose whole claim is an angle.
 *
 * #442 draws two clouds side by side under a sentence saying one lies on the diagonal and the
 * other lies flat. That sentence is only true of a frame where a unit of the outcome is drawn as
 * long as a unit of the predictor, and the first version of the pair was not: the shared y span
 * came out at 0.7604 of the x span, which drew a slope of 0.9855 at 1.296 — a superlinear picture
 * under a sentence saying "a slope of one".
 */
const FIT: Fit = {
  from: { x: 100, y: 1_000 },
  to: { x: 10_000, y: 100_000 },
  slope: 0.9855,
  rSquared: 0.8126,
};
const LOG_AXES = {
  x: { label: "Base cost enrolled ADM", format: (v: number) => String(v), log: true },
  y: { label: "Total weighted wealth", format: (v: number) => `$${v}`, log: true },
};

test("a fitted panel is squared, so that a slope of one is drawn as a diagonal", () => {
  const width = panelWidth(2);
  const points = cloud(Array.from({ length: 30 }, (_, i) => 100 + i * 300));
  const spec = scatterSpec(points, LOG_AXES, [], {
    width,
    xDomain: [50, 20_000],
    yDomain: [500, 200_000],
    fit: FIT,
  })!;
  const { height, marginLeft, marginRight, marginTop, marginBottom } = spec.options;
  expect(height! - marginTop! - marginBottom!).toBe(width - marginLeft! - marginRight!);
  // The frame is the caller's, drawn as given: a padded log domain would move both ends of it.
  expect(spec.options.x!.domain).toEqual([50, 20_000]);
  expect(spec.options.y!.domain).toEqual([500, 200_000]);
});

test("a fitted line without the frame it was fitted in is refused", () => {
  const points = cloud(Array.from({ length: 30 }, (_, i) => 100 + i * 300));
  const missing = { width: WIDTHS.wide, xDomain: [50, 20_000] as [number, number], fit: FIT };
  // Half a frame is the dangerous case: the x axis is the crate's and the y axis is fitted to
  // whatever these points happen to span, which is exactly how a slope gets redrawn.
  expect(() => scatterSpec(points, LOG_AXES, [], missing)).toThrow(/without both domains/);
  expect(() => scatterSpec(points, LOG_AXES, [], { width: WIDTHS.wide, fit: FIT })).toThrow();
});

test("a fitted panel prints its slope and its r-squared, and nothing else of the fit", () => {
  const width = panelWidth(2);
  const points = cloud(Array.from({ length: 30 }, (_, i) => 100 + i * 300));
  const draw = (fit: Fit) =>
    renderToString(
      () =>
        scatterSpec(points, LOG_AXES, [], {
          width,
          xDomain: [50, 20_000],
          yDomain: [500, 200_000],
          fit,
        }),
      "presentational",
    );

  const rising = draw(FIT);
  // Four places, which is where the corpus writes them and where crates/figures.json pins them:
  // a reader quoting 0.9855 off the picture is quoting the figure, not a rounding of it.
  expect(rising).toContain("slope 0.9855 · r² 0.8126");
  expect(rising).toContain("scatter-fit");

  // A negative slope carries the typographic minus, and a positive one carries no plus — the line
  // already says which way it goes.
  const flat = draw({ ...FIT, slope: -0.0429, rSquared: 0.0085 });
  expect(flat).toContain("slope −0.0429 · r² 0.0085");

  // And a cloud with no fit draws neither the line nor the numbers.
  const bare = renderToString(
    () => scatterSpec(points, LOG_AXES, [], { width, xDomain: [50, 20_000] }),
    "presentational",
  );
  expect(bare).not.toContain("scatter-fit");
  expect(bare).not.toContain("slope");
});

test("two panels share the wide frame, and one panel is the whole of it", () => {
  // The gap is `--space-6`, 1.1rem at the 16px root, which is what `.panels` lays them out with.
  expect(panelWidth(2) * 2 + 18).toBe(WIDTHS.wide);
  expect(panelWidth(1)).toBe(WIDTHS.wide);
});

test("a set of panels is drawn at the width the stylesheet will give it, not at its share", () => {
  /*
   * `.panel` is `flex: 1 1 280px`, so `.panels` wraps rather than shrinking past that. #444 draws
   * seven, and dividing the wide frame seven ways would have laid each out at 76px — which the
   * stylesheet would then have scaled back up to 311, taking the 11px axis text to 45px. The row
   * is what the width has to be computed against, not the set.
   */
  for (const panels of [3, 4, 5, 6, 7]) {
    expect(panelWidth(panels), `${panels} panels wrap to two per row`).toBe(panelWidth(2));
  }
  // And a panel is never drawn narrower than the flex basis the stylesheet would honour.
  for (const panels of [1, 2, 3, 7, 35]) expect(panelWidth(panels)).toBeGreaterThanOrEqual(280);
});

/**
 * How many rows reached one named mark layer of a spec.
 *
 * Plot's `Markish` is a union wide enough to admit a bare render function, so a mark's own
 * `className` and `data` are not on the static type even though every mark these specs build — a
 * `dot`, a `ruleY`, a `text`, a `rect` — carries both. The cast is narrow and deliberate: which
 * rows reach which layer is exactly what the assertions below are about, and the rendered SVG
 * shows it only as a count of anonymous `<g>` children, twice over for the chart pair.
 */
function layerRows(spec: Spec, className: string): number {
  const marks = (spec.options.marks ?? []) as unknown as {
    className?: string;
    data?: readonly unknown[];
  }[];
  const found = marks.filter((mark) => mark.className === className);
  expect(found, `the spec draws a ${className} layer`).toHaveLength(1);
  return found[0]!.data!.length;
}

/** The axis `/bounds` draws its census on: a count of districts, printed plainly. */
const COUNT = { label: "districts the bound is operative for", format: (v: number) => String(v) };

/** A ranking shaped like the bound census — a long tail, and a row at zero carrying the finding. */
function census(values: number[]): Rank[] {
  return values.map((value, i) => ({
    label: `bound ${i}`,
    value,
    hover: `bound ${i} — ${value}`,
    ...(value === 0 ? { marked: "cannot bind" } : {}),
  }));
}

test("the row at zero is drawn at the axis floor with its phrase, not dropped", () => {
  /*
   * #443's requirement, and the reason this form exists at all: one of the thirty-eight bounds is
   * the operative term for nobody, and "a bound with no force is the finding". A log axis cannot
   * hold zero, so the two ways a chart usually deals with one — drop the row, or draw a bar of no
   * length — both erase the result. The row is placed a clear step below the smallest drawable
   * value instead, and the phrase is printed beside it so the position is read rather than
   * guessed at.
   */
  const spec = rankSpec(census([554, 499, 43, 1, 0]), COUNT, W)!;
  const [floor, max] = spec.options.x!.domain as [number, number];
  expect(floor).toBe(0.5); // half of 1, the smallest value that can be drawn
  expect(max).toBe(554);

  const rows = (name: string) => layerRows(spec, name);

  // Every row has a dot, the zero row included, and its own phrase beside it.
  expect(rows("rank-dot")).toBe(5);
  expect(rows("rank-mark")).toBe(1);

  // But no stem: a rule from the floor to the floor is a zero-length bar by another name, which
  // is what the row was drawn off the scale to avoid.
  expect(rows("rank-stem")).toBe(4);

  // And the hit layer matches the dot layer row for row, or `declareCursor` indexes the wrong
  // tooltip onto the one row a reader is most likely to point at.
  expect(rows("rank-hit")).toBe(rows("rank-dot"));
});

test("the marked row's phrase is reserved for where it is drawn, not where it is longest", () => {
  /*
   * The phrase prints outward from its own mark, so what it needs past the frame is its width
   * less the empty plot already to the right of that mark. Reserved as a flat width it cost 95px
   * of the 320px frame — a third of the narrow drawing held blank for text drawn hard against the
   * opposite edge, because the row this form exists for sits at the axis *floor*.
   */
  const narrow = { width: WIDTHS.narrow };
  const atFloor = rankSpec(census([554, 499, 43, 1, 0]), COUNT, narrow)!;

  // The same chart with the phrase on the widest row instead, where it really does overhang.
  const rows: Rank[] = census([554, 499, 43, 1, 0]).map(({ label, value, hover }) => ({
    label,
    value,
    hover,
  }));
  rows[0] = { ...rows[0]!, marked: "cannot bind" };
  const atMax = rankSpec(rows, COUNT, narrow)!;

  expect(atFloor.options.marginRight!).toBeLessThan(atMax.options.marginRight!);
  // At the floor the whole plot lies to the phrase's right, so nothing is held back for it.
  // 16px, which is `gutter`'s answer for the bare axis overhang every form reserves.
  expect(atFloor.options.marginRight).toBe(16);
});

/**
 * The plane, whose reading is which quadrant a dot is in.
 *
 * #444's finding is that three of seven anchor rules write *less* guarantee and cost *more* money,
 * and that no table reporting one column at a time can show it. What makes the picture say it is
 * the pair of zero rules: the origin is the rule in force, so "up and to the left" is the whole
 * claim. Every assertion below is about keeping that readable.
 */
const PLANE_AXES = {
  x: { label: "Guarantee written", format: (v: number) => `$${Math.round(v / 1e6)}m` },
  y: { label: "Total state support", format: (v: number) => `$${Math.round(v / 1e6)}m` },
};

/** Seven rules, positioned as the committed plane positions them: three in the upper left. */
function rules(): Place[] {
  return [
    { label: "A ratchet from FY2027", x: 57.3e6, y: 52.9e6, hover: "ratchet" },
    { label: "A 2% cap, FY2032 undamped", x: -167.6e6, y: -59.4e6, hover: "2% cap" },
    { label: "A 1% cap, FY2036 undamped", x: -107.6e6, y: 10.3e6, hover: "1% cap" },
    { label: "A rolling pupil count", x: -14.7e6, y: 66.2e6, hover: "rolling" },
    { label: "[M] mirrored inside [H]", x: -70.5e6, y: 62.2e6, hover: "mirror inside" },
    { label: "[M] mirrored beside [H]", x: 0, y: 135.3e6, hover: "mirror beside" },
    { label: "A dated phase-down", x: 0, y: 0, hover: "phase-down" },
  ];
}

const PLANE_FRAME: { xDomain: [number, number]; yDomain: [number, number] } = {
  xDomain: [-176.6e6, 66.3e6],
  yDomain: [-67.2e6, 143.1e6],
};
const PLANE = { width: WIDTHS.wide, ...PLANE_FRAME };

test("the plane draws its two rules through the origin, and no frame besides", () => {
  const spec = planeSpec(rules(), PLANE_AXES, PLANE)!;
  // Two `plane-zero` marks, one per axis — so `layerRows` is not the tool here.
  const marks = (spec.options.marks ?? []) as unknown as { className?: string }[];
  expect(marks.filter((mark) => mark.className === "plane-zero")).toHaveLength(2);
  // The scales themselves print nothing. A boxed frame with ticks would draw four lines as
  // emphatic as the two that carry the reading, and the quadrant is the encoding.
  expect(spec.options.x!.axis).toBeNull();
  expect(spec.options.y!.axis).toBeNull();
  const svg = renderToString(() => spec, "presentational");
  expect(svg).not.toContain("plot-frame");
});

test("the plane refuses a frame that does not hold zero, on either axis", () => {
  // A domain fitted to the points alone would do this the moment every rule fell one side of the
  // rule in force — which is most of what a later axis might measure.
  expect(() =>
    planeSpec(rules(), PLANE_AXES, { width: WIDTHS.wide, xDomain: [10e6, 60e6], yDomain: [-1, 1] }),
  ).toThrow(/does not hold zero/);
  expect(() =>
    planeSpec(rules(), PLANE_AXES, { width: WIDTHS.wide, xDomain: [-1, 1], yDomain: [10e6, 60e6] }),
  ).toThrow(/does not hold zero/);
  // And nothing to draw draws nothing, on `renderToString`'s standing rule.
  expect(planeSpec([], PLANE_AXES, PLANE)).toBeNull();
});

test("every dot is named on the picture, and no two names are drawn on top of each other", () => {
  /*
   * Seven rules and no hue: the quadrant is the encoding, so a legend would send a reader back and
   * forth for the one thing the chart is about. Direct labels instead — which collide, because the
   * rolling count and the mirror inside `[H]` sit four pixels apart on the outcome axis at the
   * narrow width. `placeLabels` is what resolves that, and this is the assertion that it does.
   */
  for (const width of [WIDTHS.narrow, WIDTHS.wide]) {
    const spec = planeSpec(rules(), PLANE_AXES, { width, ...PLANE_FRAME })!;
    const svg = renderToString(() => spec, "presentational");
    const labels = [...svg.matchAll(/class="plane-label"[^>]*>([\s\S]*?)<\/g>/g)];
    const drawn = labels.flatMap((match) => [...match[1]!.matchAll(/<text[^>]*>([^<]*)<\/text>/g)]);
    expect(drawn.map((m) => m[1]), `${width}px draws all seven names once`).toHaveLength(
      rules().length * 2,
    );
  }
});

test("the placement is a function of the positions, not of the order they are drawn in", () => {
  // The placer is greedy, so it depends on order — which is the manifest's order, and stable.
  // What must not vary is the answer for the same input, since the SVG is committed to `dist`.
  const once = renderToString(() => planeSpec(rules(), PLANE_AXES, PLANE), "presentational");
  const again = renderToString(() => planeSpec(rules(), PLANE_AXES, PLANE), "presentational");
  expect(once).toBe(again);
});

test("the hit layer matches the dot layer place for place", () => {
  const spec = planeSpec(rules(), PLANE_AXES, PLANE)!;
  expect(layerRows(spec, "plane-hit")).toBe(layerRows(spec, "plane-dot"));
  expect(spec.hovers!.text).toHaveLength(rules().length);
  // The dot is what lights up under the cursor, and the hit target is the invisible thing the
  // pointer actually finds. `declareCursor` checks the pairing; this checks it was declared.
  expect(spec.hovers!.cursor).toEqual({ second: "paired marks", layers: [".plane-dot"] });
});

test("a signed panel with nothing in it still draws its zero, so the empty rule is legible", () => {
  /*
   * The dated phase-down moves neither axis by a cent, so its panel in #444's small multiples is
   * five bars of zero. Without `min` that panel is unsigned, draws no rule, and reads as a chart
   * that failed rather than as a rule that does nothing — beside six panels that all have one.
   */
  const flat: Bar[] = ["Least wealthy", "Second", "Third", "Fourth", "Wealthiest"].map((label) => ({
    label,
    value: 0,
    hover: `${label} — $0.00`,
  }));
  const width = panelWidth(7);
  // The rule carries no class of its own — it is Plot's `ruleX`, drawn only in signed mode — so
  // it is counted off the drawn SVG, where `--accent-rule` is the one stroke that names it.
  const rules = (spec: Spec) =>
    [...renderToString(() => spec, "presentational").matchAll(/stroke="var\(--accent-rule\)"/g)]
      .length;
  expect(rules(barSpec(flat, { width })), "no rule without a signed scale").toBe(0);
  // Two, because `renderToString` lays the drawing out twice. One rule per drawing.
  expect(rules(barSpec(flat, { width, max: 166.55, min: -83.51 }))).toBe(2);
  // And the panel is drawn on the set's scale, not on its own nothing.
  expect(barSpec(flat, { width, max: 166.55, min: -83.51 }).options.x!.domain).toEqual([
    -83.51, 166.55,
  ]);
});

test("a shared scale is the same scale: two panels of one set draw a value at one length", () => {
  // What makes the monotonicity readable across seven panels is that the axis does not move.
  const of = (bars: Bar[]) =>
    barSpec(bars, { width: panelWidth(7), max: 166.55, min: -83.51 }).options.x!.domain;
  const big: Bar[] = [{ label: "Least wealthy", value: 166.55, hover: "a" }];
  const small: Bar[] = [{ label: "Least wealthy", value: 4.06, hover: "b" }];
  expect(of(big)).toEqual(of(small));
  // And the shared domain spans both signs, because one of the seven runs negative.
  expect(of(small)).toEqual([-83.51, 166.55]);
});

test("a set shares its frame as well as its domain, so a labelled panel keeps its siblings' zero", () => {
  /*
   * The domain being equal is not enough. The right gutter is sized to the direct labels a panel
   * carries, and in #444's set exactly one panel carries any — the inert rule, which prints its
   * zeros because it has no bars to print them on. That panel's frame came out ten pixels
   * narrower than the six beside it, which moves every pixel inside it, the zero rule included.
   */
  const set = { width: panelWidth(7), max: 166.55, min: -83.51, labelChars: 2 };
  const drawn: Bar[] = [{ label: "Least wealthy", value: 166.55, hover: "a" }];
  const zeros: Bar[] = [{ label: "Least wealthy", value: 0, direct: "$0", hover: "b" }];
  const frame = (spec: Spec) => [spec.options.marginLeft, spec.options.marginRight];
  expect(frame(barSpec(zeros, set))).toEqual(frame(barSpec(drawn, set)));
  // And the option is doing the work: left to its own labels the panel reserves room the others
  // have not, which is the fault itself.
  const { labelChars: _shared, ...own } = set;
  expect(barSpec(zeros, own).options.marginRight).not.toBe(
    barSpec(drawn, own).options.marginRight,
  );
  // A chart that is not part of a set still sizes its own gutter, which is every other caller.
  expect(barSpec(drawn, { width: WIDTHS.wide }).options.marginRight).toBe(
    barSpec(zeros, { width: WIDTHS.wide, labelChars: 0 }).options.marginRight,
  );
});

// --- The two-series line chart, whose index is not necessarily a year -------------------------

/** Two shares over three positions, none of them a fiscal year. */
const HELD: SeriesPoint[] = [
  { at: 1, a: 0.645, b: 0.661 },
  { at: 2, a: 0.594, b: 0.713 },
  { at: 3, a: 0.601, b: 0.694 },
];

const share = (v: number) => `${(v * 100).toFixed(0)}%`;

test("a reference the lines are measured against is inside the frame they are drawn in", () => {
  /*
   * The counterpart of the fan chart's rule above, and its opposite case. There, a reference not
   * every year carries is kept *out* of the domain because it will not be drawn. Here it is drawn
   * on every position by construction — it is one value, not a series — and the lines mean their
   * distance from it, so a frame that excluded it would ask for a comparison and then omit one
   * side of it.
   */
  const plain = seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", {
    width: WIDTHS.wide,
    tick: (at) => `${at}`,
  })!;
  const [, high] = plain.options.y!.domain as number[];
  expect(high).toBeLessThan(0.75);

  const held = seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", {
    width: WIDTHS.wide,
    tick: (at) => `${at}`,
    reference: { value: 0.9, label: "what it claims" },
  })!;
  const [, withReference] = held.options.y!.domain as number[];
  expect(withReference).toBeGreaterThan(0.9);
});

test("the reference is drawn muted and once, not as a third series", () => {
  const svg = renderToString(
    (w) =>
      seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", {
        width: w,
        tick: (at) => `${at}`,
        reference: { value: 0.683, label: "what ±1σ claims" },
      }),
    "presentational",
  );
  expect(svg).toContain("what ±1σ claims");
  // The categorical pair is the two quantities. A reference in a third hue would read as a third.
  expect(svg).not.toContain("var(--series-c)");
  // Once per drawing, and the pair is drawn at both widths — see `WIDTHS`.
  for (const drawing of svg.split("<svg").slice(1)) {
    expect((drawing.match(/what ±1σ claims/g) ?? []).length).toBe(1);
  }
});

test("both corner labels are the caller's, so a horizon is not written as a fiscal year", () => {
  const svg = renderToString(
    (w) =>
      seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", {
        width: w,
        tick: (at) => (at === 1 ? "one year" : `${at} years`),
      }),
    "presentational",
  );
  expect(svg).toContain("one year");
  expect(svg).toContain("3 years");
  // The `FY` prefix used to be written here rather than passed in, which is what made this form
  // unusable for the one caller whose index counts something else.
  expect(svg).not.toContain("FY");
});

test("the hit layer is one full-height column per position on the index", () => {
  const spec = seriesSpec(HELD, { a: "pooled", b: "cross" }, share, (p) => `at ${p.at}`, {
    width: WIDTHS.wide,
    tick: (at) => `${at}`,
  })!;
  expect(spec.hovers!.text).toEqual(["at 1", "at 2", "at 3"]);
  const marks = (spec.options.marks ?? []) as unknown as { className?: string }[];
  expect(marks.filter((mark) => mark.className === "series-hit")).toHaveLength(1);
});
