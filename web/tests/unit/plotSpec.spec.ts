/**
 * What a chart has to state about its own scale before a reader can take a value off it.
 *
 * Six charts on this site omitted something a reader needed — #109 grouped them because they are
 * one judgement repeated rather than six unrelated faults. Each of these tests holds one of those
 * judgements in the layer that made it, so the next chart built on the same specification inherits
 * the answer instead of the omission.
 */

import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { parseHTML } from "linkedom";
import { expect, test } from "vitest";

import type {
  Bar,
  FanPoint,
  Fit,
  Place,
  Range,
  Rank,
  ScatterPoint,
  SeriesPoint,
  Trace,
} from "../../src/lib/chart.ts";
import { compactMoney, pct } from "../../src/lib/format.ts";
import {
  axisTicks,
  DOT,
  barSpec,
  distributionSpec,
  fanSpec,
  MAX_SCALE,
  panelWidth,
  placeLabels,
  planeSpec,
  rangeSpec,
  rankSpec,
  ruleSwatch,
  scatterSpec,
  seriesSpec,
  type BarHue,
  type Box,
  type Spec,
  truncatedDomain,
  WIDTHS,
} from "../../src/lib/plot/spec.ts";
import { renderPanelToString, renderToString } from "../../src/lib/plot/ssr.ts";
import { INK, ORDINAL, SERIES, SERIES_TEXT, SUBJECT } from "../../src/lib/plot/tokens.ts";

/** How many layouts `renderToString` emits: one per width in `WIDTHS`. */
const DRAWINGS = Object.keys(WIDTHS).length;

/**
 * The width these assertions are written at.
 *
 * Every chart is laid out at each of `WIDTHS` — and what is asserted here is the layout
 * arithmetic, which is the same arithmetic at any of them. So the tests name one rather than
 * running each of them three times to make the same point.
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
   * Two channels: a `SUBJECT` ring drawn behind the bar, and the category name in primary ink at
   * 600. Not a change of fill (#652): the subject took the guarantee hue, so a chart of formula
   * aid said its subject was guaranteed, and an ink fill would put the cursor ring at 1:1.
   */
  const bars: Bar[] = [
    { label: "Nebraska", value: 60 },
    { label: "Ohio", value: 45, current: true },
    { label: "Utah", value: 30 },
  ];
  const svg = renderToString((w) => barSpec(bars, { width: w, hue: "formula" }), "presentational");
  const { document } = parseHTML(`<div>${svg}</div>`);
  const rings = [...document.querySelectorAll(".bar-subject")];
  expect(rings.length, "one ring per layout").toBe(DRAWINGS);
  for (const ring of rings) {
    expect(ring.getAttribute("fill")).toBe(SUBJECT);
    expect(ring.children.length, "on the one bar").toBe(1);
  }
  expect(svg, "the subject keeps the chart's fill").not.toContain("var(--series-guarantee)");
  expect(svg).toContain('font-weight="600"');

  // And a chart with no subject is unchanged: same fills, no ring, no second text mark.
  const plain = renderToString(
    (w) => barSpec(bars.map(({ current: _c, ...b }) => b), { width: w, hue: "formula" }),
    "presentational",
  );
  expect(plain).not.toContain("bar-subject");
  expect(plain).not.toContain('font-weight="600"');
});

test("a bar chart says what its bars are, and plain bars are no series", () => {
  /*
   * #652. Formula blue was every bar chart's default, so 11 of 13 sampled — cash held, casino
   * receipts, the share of districts *held by the guarantee* — wore the hue that means formula aid.
   * `hue` is required, so a caller cannot fall into one.
   */
  const bars: Bar[] = [
    { label: "a", value: 3 },
    { label: "b", value: 2 },
  ];
  const fill = (hue: BarHue) =>
    parseHTML(`<div>${renderToString((w) => barSpec(bars, { width: w, hue }), "presentational")}</div>`)
      .document.querySelector(".bar-fill")!
      .getAttribute("fill");
  expect(fill("formula")).toBe(SERIES.formula);
  expect(fill("guarantee")).toBe(SERIES.guarantee);
  expect(fill("plain")).toBe(INK.muted);
  expect(Object.values(SERIES), "plain is never a series").not.toContain(fill("plain"));
  // @ts-expect-error — a bar chart has to say what its bars mean.
  barSpec(bars, { width: WIDTHS.wide });
});

test("a lone plain series is drawn in no series' hue, and labels only its own line", () => {
  /*
   * #659. Cash held was a `hue: "plain"` bar chart and became a line; the line's first hue is
   * formula blue, which would have undone #652 on 610 pages.
   */
  const cash: SeriesPoint[] = [
    { at: 2020, a: 3, b: null },
    { at: 2021, a: 5, b: null },
    { at: 2022, a: 4, b: null },
  ];
  const svg = (plain: boolean) =>
    parseHTML(
      `<div>${renderToString(
        (w) => seriesSpec(cash, { a: "held", b: "" }, (v) => `$${v}`, () => "", { width: w, tick: (at) => `${at}`, plain }),
        "presentational",
      )}</div>`,
    ).document;
  const plain = svg(true);
  expect(plain.querySelector(".series-a")!.getAttribute("stroke")).toBe(INK.muted);
  expect(plain.querySelector(".series-end")!.getAttribute("fill")).toBe(INK.muted);
  expect(plain.querySelector(".series-start")!.getAttribute("fill")).toBe(INK.muted);
  // One end label, in the first of the chart's drawings: the empty second series has none.
  expect(plain.querySelector("svg")!.querySelectorAll(".series-end text")).toHaveLength(1);
  expect(plain.querySelector(".series-end text")!.textContent).toBe("Held $4");
  expect(svg(false).querySelector(".series-a")!.getAttribute("stroke")).toBe(SERIES.formula);
});

test("the subject is one colour whatever form it is drawn in", () => {
  /*
   * #652. Columbus was marked blue on its strips and orange on its fan, on one page: the strip
   * marker was formula blue, the bar and the rank dot guarantee orange.
   */
  const bars = renderToString(
    (w) => barSpec([{ label: "a", value: 2, current: true }, { label: "b", value: 1 }], { width: w, hue: "plain" }),
    "presentational",
  );
  const strip = renderToString(
    (w) =>
      distributionSpec(
        Array.from({ length: 30 }, (_, i) => ({ value: i, hover: `d${i}` })),
        { width: w, format: String, marker: { value: 12, label: "Columbus" } },
      ),
    "presentational",
  );
  const rank = renderToString(
    (w) =>
      rankSpec(
        [
          { label: "a", value: 5, hover: "a" },
          { label: "b", value: 3, hover: "b", marked: "this one" },
          { label: "c", value: 1, hover: "c" },
        ],
        { label: "count", format: String },
        { width: w },
      ),
    "presentational",
  );
  const colours = (svg: string, selector: string, attribute: string) =>
    [...parseHTML(`<div>${svg}</div>`).document.querySelectorAll(selector)].map(
      (el) => el.getAttribute(attribute) ?? el.parentElement?.getAttribute(attribute),
    );
  expect(colours(bars, ".bar-subject", "fill")).toEqual(Array(DRAWINGS).fill(SUBJECT));
  expect(colours(strip, ".dist-marker", "stroke")).toEqual(Array(DRAWINGS).fill(SUBJECT));
  const dots = colours(rank, ".rank-dot circle", "fill");
  expect(dots.filter((c) => c === SUBJECT).length, "the marked dot, once per layout").toBe(DRAWINGS);
  expect(dots, "and no subject in the guarantee hue").not.toContain(SERIES.guarantee);
});

/** Every `<text>` of a class in a rendered drawing, by its contents. */
function texts(svg: string, className: string): string[] {
  return [...parseHTML(`<div>${svg}</div>`).document.querySelectorAll(`g.${className} text`)].map(
    (t) => t.textContent ?? "",
  );
}

test("a strip's marker is labelled, as its comment always said it was", () => {
  /*
   * #654. The comment over the marker promised it was "labelled", and the marks under it were a
   * single rule. `Strip.astro` passed a label, `county.ts` and `outcomes.ts` passed one, and none of
   * the three was ever drawn: Franklin on its county strip was a coloured line with no name.
   */
  const strip = renderToString(
    (w) =>
      distributionSpec(
        Array.from({ length: 30 }, (_, i) => ({ value: i, hover: `d${i}` })),
        { width: w, format: String, marker: { value: 12, label: "Columbus City" } },
      ),
    "presentational",
  );
  expect(texts(strip, "dist-marker-label")).toEqual(Array(DRAWINGS).fill("Columbus City"));
});

test("a strip's reference is a dashed rule with its label, inside the domain", () => {
  /*
   * #654. A House seat's eighteen districts ran $110K to $259K and nothing on the strip said Ohio's
   * median is $248K — the seat is a poor one, and the figure could not say so.
   */
  const values = [110, 120, 140, 150, 170, 190, 200, 230, 259].map((value) => ({ value, hover: `${value}` }));
  const spec = distributionSpec(values, {
    ...W,
    format: String,
    reference: { value: 300, label: "Ohio median 300" },
  });
  const svg = renderToString(() => spec, "presentational");
  expect(texts(svg, "dist-reference-label")).toEqual(Array(DRAWINGS).fill("Ohio median 300"));
  expect(svg).toContain("dist-reference");
  // A reference beyond every member widens the domain to hold it, rather than drawing off the edge.
  const [, hi] = spec!.options.x!.domain as [number, number];
  expect(hi).toBeGreaterThan(300);

  // On the box's own median it is not a second rule: the label names the one already there.
  const onMedian = renderToString(
    () => distributionSpec(values, { ...W, format: String, reference: { value: 170, label: "median" } }),
    "presentational",
  );
  expect(texts(onMedian, "dist-reference-label")).toEqual(Array(DRAWINGS).fill("median"));
  expect(parseHTML(`<div>${onMedian}</div>`).document.querySelectorAll("g.dist-reference")).toHaveLength(0);
});

test("a strip names its two ends when asked, and only then", () => {
  /*
   * #654. The county card's sentence compares Whitehall with Grandview Heights, and the strip under
   * it drew them as two of sixteen unnamed dots.
   */
  const values = [121566, 150000, 200000, 260000, 474526].map((value, i) => ({
    value,
    hover: `${value}`,
    name: ["Whitehall City", "b", "c", "d", "Grandview Heights City"][i]!,
  }));
  const named = renderToString(() => distributionSpec(values, { ...W, format: compactMoney, ends: true }), "presentational");
  expect(texts(named, "dist-end-label")).toEqual(
    Array.from({ length: DRAWINGS }, () => ["Whitehall City", "Grandview Heights City"]).flat(),
  );
  const plain = distributionSpec(values, { ...W, format: compactMoney });
  expect(texts(renderToString(() => plain, "presentational"), "dist-end-label")).toEqual([]);
  // And a strip with no names is the height it always was, so `/districts`' six stay one row apart.
  expect(plain!.options.height).toBe(64);
});

test("two names that would collide take two rows, and the strip grows to hold them", () => {
  // The marker and the reference a few units apart: one row cannot hold both names.
  const values = Array.from({ length: 30 }, (_, i) => ({ value: i, hover: `d${i}` }));
  const apart = distributionSpec(values, {
    ...W,
    format: String,
    marker: { value: 3, label: "Marked" },
    reference: { value: 27, label: "Reference" },
  });
  const close = distributionSpec(values, {
    ...W,
    format: String,
    marker: { value: 14, label: "Marked district" },
    reference: { value: 15, label: "Ohio median 15" },
  });
  expect(apart!.options.height).toBe(64 + 15);
  expect(close!.options.height).toBe(64 + 30);
  expect(close!.options.marginTop).toBe(4 + 30);
});

test("a chart of change is fitted to its ratios and fills by polarity whatever their sign", () => {
  /*
   * The county chart on the Change tab (#651). `barSpec` floored its domain at 1, so every
   * county's change was drawn on 0 to +100% when the median county's largest move was 7%. And in a
   * county where every district rose, nothing was below zero, signed mode never engaged, and the
   * subject's *gain* was filled in guarantee orange — the hue the same chart gives a loss.
   */
  const rose: Bar[] = [
    { label: "Dublin City", value: 0.245 },
    { label: "Hilliard City", value: 0.031, current: true },
    { label: "Bexley City", value: 0.012 },
  ];
  const domain = barSpec(rose, { width: WIDTHS.wide, hue: "polarity" }).options.x!.domain as number[];
  expect(domain[1]).toBeLessThanOrEqual(1.25 * 0.245);
  // Without `polarity` a chart of counts keeps the floor it was written for.
  expect((barSpec(rose, { ...W, hue: "formula" }).options.x!.domain as number[])[1]).toBe(1);

  const svg = renderToString((w) => barSpec(rose, { width: w, hue: "polarity" }), "presentational");
  for (const drawing of svg.split("<svg").slice(1)) {
    expect(drawing, "a gain is never the loss hue").not.toContain("var(--series-guarantee)");
    expect((drawing.match(/<line/g) ?? []).length, "the zero rule").toBe(1);
    // The subject is marked by a ring and its bold name, on the one bar. The ring is its own
    // mark: #651 drew it as the bar's stroke, which the hover layer's CSS painted transparent.
    const { document } = parseHTML(`<svg${drawing}`);
    expect(document.querySelector(".bar-subject")?.children.length).toBe(1);
    expect(document.querySelector(".bar-fill")?.getAttribute("stroke") ?? "none").toBe("none");
    expect(drawing).toContain('font-weight="600"');
  }
});

test("a marked name wraps at the width it is drawn at, which is bold", () => {
  /*
   * Cuyahoga's chart on `/district/043786/change` marks Cleveland Municipal. At the phone width the
   * gutter is capped, the bold name fitted a budget measured on normal-weight text, and it painted
   * 10px past the left of the frame on CI (#639). Wrapped, it is two words on two lines.
   */
  const bars: Bar[] = [
    { label: "Cleveland Heights-University Heights City", value: 0.05 },
    { label: "Cleveland Municipal", value: -0.02, current: true },
    { label: "Bay Village City", value: 0.03 },
  ];
  const html = renderToString(
    (w) => barSpec(bars, { width: w, hue: "formula", scale: { format: String, says: "change" } }),
    "presentational",
  );
  const { document } = parseHTML(`<div>${html}</div>`);
  const marked = (at: string) =>
    [...document.querySelectorAll(`[data-at="${at}"] .bar-label.current tspan`)].map((t) => t.textContent);
  expect(marked("narrow")).toEqual(["Cleveland", "Municipal"]);
  // Where the gutter is not capped it is sized to the bold name, which therefore stays whole.
  expect(marked("wide")).toEqual([]);
});

test("a column with bars on both sides of zero draws its rule at every width", () => {
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
  const pair = renderToString((w) => barSpec(bars, { width: w, hue: "formula" }), { label: "The column" });
  const drawings = pair.split("<svg").slice(1);
  expect(drawings).toHaveLength(DRAWINGS);
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
    (w) => barSpec(bars.map((b) => ({ ...b, value: Math.abs(b.value) })), { width: w, hue: "formula" }),
    "presentational",
  );
  expect(unsigned).not.toContain("<line");
});

test("a signed distribution draws its zero, and an unsigned one does not", () => {
  /*
   * The enrollment-change strip on `/districts` is the site's one signed distribution and drew no
   * zero reference, so a dot two thirds along could have been a district that grew or one that
   * shrank. `histogramSpec` draws a dashed rule and labels it "No change" for exactly that reason.
   *
   * Detected from the domain rather than passed at the call site, which is what makes it reach
   * the sixth strip without anyone remembering. Every strip is one height, because every strip
   * carries a foot: the six sit one row apart on the same page and a taller one would be a jump.
   */
  const signed = distributionSpec(
    [-0.08, -0.03, -0.01, 0.02, 0.05, 0.09].map((value) => ({ value, hover: `${value}` })),
    { ...W, format: (v) => pct(v, 0) },
  );
  expect(signed).not.toBeNull();
  expect(signed!.options.height).toBe(64);
  expect(renderToString(() => signed, "presentational")).toContain("No change");

  const unsigned = distributionSpec(
    [1, 2, 3, 4, 5, 6].map((value) => ({ value, hover: `${value}` })),
    { ...W, format: String },
  );
  expect(unsigned!.options.height).toBe(64);
  expect(renderToString(() => unsigned, "presentational")).not.toContain("No change");
});

/** Each `.axis-foot` text in a rendered drawing, with the x its `transform` places it at. */
function footLabels(svg: string): { x: number; text: string }[] {
  const doc = parseHTML(`<div>${svg}</div>`).document;
  return [...doc.querySelectorAll("g.axis-foot text")].map((t) => {
    const at = /translate\(([-\d.]+)/.exec(t.getAttribute("transform") ?? "");
    return { x: at ? Number(at[1]) : Number(t.getAttribute("x") ?? NaN), text: t.textContent ?? "" };
  });
}

test("a strip's end labels sit at the values they name", () => {
  /*
   * The HTML scale row under a strip named the data's extremes at the edges of the box. The domain
   * is padded, so the edges are not the extremes: the "$896" at the right end of the aid strip
   * labelled empty space a few pixels past the last dot. The strip now states nice values, each
   * drawn at its own x, so a label is a claim about where it stands.
   */
  const values = [3120, 4410, 5000, 6200, 7400, 9100, 12800, 15500];
  const spec = distributionSpec(
    values.map((value) => ({ value, hover: `${value}` })),
    { ...W, format: compactMoney },
  );
  const svg = renderToString(() => spec, "presentational");
  const labels = footLabels(svg).filter((l) => /\$/.test(l.text));
  expect(labels.length).toBeGreaterThanOrEqual(2);
  const [lo, hi] = spec!.options.x!.domain as [number, number];
  const left = 2;
  const right = W.width - 2;
  for (const { x, text } of labels) {
    const value = Number(text.replace(/[$,]/g, "").replace(/K$/, "e3"));
    const expected = left + ((value - lo) / (hi - lo)) * (right - left);
    expect(Math.abs(x - expected), `${text} at ${x}, scale says ${expected}`).toBeLessThanOrEqual(1);
  }
});

test("a log axis over two decades labels the decades between its ends", () => {
  /*
   * A log axis labelled only at its two ends gives a reader no way to tell whether the middle of the
   * strip is ten times the left or a thousand; the interior powers of ten are what make the spacing
   * legible.
   */
  expect(axisTicks([3, 4200], true)).toEqual([5, 10, 100, 1000, 2000]);
  expect(axisTicks([3, 4200], true).length).toBeGreaterThanOrEqual(3);
  expect(axisTicks([0.6, 9.4])).toEqual([2, 8]);
});

/** Each drawn `<g class>` mark's shapes in a rendered drawing, at the position the transforms add up to. */
function placed(svg: string, className: string): { x: number; y: number; text: string; fill: string }[] {
  const doc = parseHTML(`<div>${svg.slice(0, svg.indexOf("</svg>") + 6)}</div>`).document;
  const shift = (el: Element | null) => {
    const at = /translate\(([-\d.]+),([-\d.]+)\)/.exec(el?.getAttribute("transform") ?? "");
    return at ? [Number(at[1]), Number(at[2])] : [0, 0];
  };
  return [...doc.querySelectorAll(`g.${className}`)].flatMap((g) => {
    const [gx, gy] = shift(g);
    return [...g.children].map((child) => {
      const [x, y] = shift(child);
      return { x: x! + gx!, y: y! + gy!, text: child.textContent ?? "", fill: g.getAttribute("fill") ?? "" };
    });
  });
}

test("a banded trace is named in the gutter, behind a swatch of its band", () => {
  /*
   * On `/outcomes` one legend keyed two banded scatters from above both — 171px above the first at
   * 1280, a screen above the second (#657). Each median line is named where it ends instead, in
   * the gutter so the names stay off the cloud, in text ink because the ramp's end steps are near
   * 2.2:1 against the card, and behind a swatch so the hue is still on the page.
   *
   * Three lines ending a fraction of a unit apart is the hard case: their names must not print on
   * each other, and each must still sit in the order its line ends.
   */
  const points = cloud(Array.from({ length: 30 }, (_, i) => 10_000 + i * 400));
  const banded: Trace[] = ["Least poor", "Middle", "Poorest"].map((label, band) => ({
    label,
    series: "formula",
    band,
    points: [{ x: 10_000, y: 50 }, { x: 20_000, y: 55 + band * 0.1 }],
  }));

  const spec = scatterSpec(points, AXES, banded, W)!;
  const bare = scatterSpec(points, AXES, [], W)!;
  // The names get their room, or they run off the viewBox.
  expect(spec.options.marginRight).toBeGreaterThan(bare.options.marginRight!);

  const svg = renderToString(() => spec, "presentational");
  const names = placed(svg, "scatter-trace-end");
  const swatches = placed(svg, "scatter-band-key");
  expect(names.map((n) => n.text)).toEqual(["Poorest", "Middle", "Least poor"]);
  expect(swatches.map((s) => s.fill)).toEqual([ORDINAL[2], ORDINAL[1], ORDINAL[0]]);

  const frame = W.width - spec.options.marginRight!;
  for (const [i, name] of names.entries()) {
    expect(name.fill, "text ink, not the band's hue").toBe(INK.secondary);
    expect(name.x, `${name.text} sits in the gutter`).toBeGreaterThan(frame);
    expect(swatches[i]!.x).toBeGreaterThan(frame);
    expect(swatches[i]!.x).toBeLessThan(name.x);
    expect(Math.abs(swatches[i]!.y - name.y), "a swatch on its name's line").toBeLessThan(0.01);
    expect(name.x + name.text.length * 7.2, `${name.text} inside the viewBox`).toBeLessThanOrEqual(W.width);
    if (i > 0) expect(name.y - names[i - 1]!.y, "clear of the name above").toBeGreaterThanOrEqual(13);
  }
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

test("a boundary is drawn under the cloud and does not move the frame it crosses", () => {
  /*
   * The third reference geometry, and the one that is a definition rather than a measurement.
   *
   * The guarantee's two terms are a plane whose multiple-of-one line is where `Decomposition::
   * origin` stops being defined — the districts below it are a third state, not a third category.
   * A reader has to be able to see where that line is, and the line is not fitted to anything: it
   * is `y = -x`, and it would be the same line on an empty chart.
   *
   * Which is exactly why it must not be allowed to size the frame. A boundary runs corner to
   * corner by construction, so a frame widened to contain it is a frame decided by a definition
   * instead of by the districts. It is clipped for the same reason the outlier above is.
   */
  const points = cloud(Array.from({ length: 20 }, (_, i) => 1_000 + i * 100));
  const rules = [
    { label: "A multiple of one", from: { x: -50_000, y: 53 }, to: { x: 50_000, y: 53 } },
  ];

  const without = scatterSpec(points, AXES, [], W)!;
  const withRule = scatterSpec(points, AXES, [], { ...W, rules })!;
  expect(withRule.options.x!.domain).toEqual(without.options.x!.domain);
  expect(withRule.options.y!.domain).toEqual(without.options.y!.domain);

  const marks = withRule.options.marks as { className?: string; clip?: unknown }[];
  const drawn = marks.filter((mark) => mark.className === "scatter-rule");
  expect(drawn).toHaveLength(1);
  expect(drawn[0]!.clip).toBe("frame");
  // Under the cloud, because it is what the cloud is read against — the same order the identity
  // and the fitted line are drawn in.
  expect(marks.indexOf(drawn[0]!)).toBeLessThan(
    marks.findIndex((mark) => mark.className === "scatter-dot"),
  );

  // Dashed, which is the one thing separating a definition from the two measured lines. And the
  // label is not painted: five panels of a spread share these, and an end label on a line that
  // crosses corner to corner crosses the cloud at every width. The legend carries the names.
  const svg = renderToString((width) => scatterSpec(points, AXES, [], { ...W, width, rules }), {
    label: "a cloud with a boundary on it",
    description: "the line where the two terms are equal",
  });
  expect(svg).toContain('stroke-dasharray="4 3"');
  expect(svg).toContain('aria-label="A multiple of one"');
  expect(svg).not.toContain(">A multiple of one<");
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

test("a fan with the formula's line draws it in the formula's hue and the band in the other", () => {
  /*
   * #607. Beside a reference, the band is what a guaranteed district receives and the reference is
   * what the formula computes. The fan drew them the other way round, so its key contradicted
   * "Where the aid comes from" on the same page. Alone, the band is formula aid and keeps its hue.
   */
  const years: FanPoint[] = [2025, 2026, 2027].map((year, i) => ({
    year,
    point: 100,
    low: 100,
    high: 100,
    observed: i === 0,
    reference: 90 - i,
  }));
  const strokes = (spec: Spec) => {
    const svg = renderToString(() => spec, { label: "a fan", description: "one district" });
    const { document } = parseHTML(svg);
    const of = (name: string) =>
      document.querySelector(`.${name}`)?.getAttribute("stroke") ?? null;
    return { reference: of("fan-reference"), seam: of("fan-anchor"), mid: of("fan-mid") };
  };
  const guaranteed = strokes(fanSpec(years, String, () => "", W)!);
  expect(guaranteed.reference).toBe(SERIES.formula);
  expect(guaranteed.seam).toBe(SERIES.guarantee);
  expect(guaranteed.mid).toBe(SERIES.guarantee);

  const alone = strokes(fanSpec(years.map(({ reference: _, ...p }) => p), String, () => "", W)!);
  expect(alone.reference).toBeNull();
  expect(alone.seam).toBe(SERIES.formula);
});

test("a full-height hit column carries the drawn value it anchors a tip at", () => {
  /*
   * #607's fourth defect. The fan's hit column spans the frame, so a tip hung from its foot sat
   * over the axis note and the key. Each column now carries `data-y`, the topmost line drawn that
   * year in the drawing's own units, and it has to fall inside the column it belongs to.
   */
  const years: FanPoint[] = [2025, 2026, 2027, 2028].map((year, i) => ({
    year,
    point: 100 - i,
    low: 100 - 2 * i,
    high: 100 + i,
    observed: i === 0,
  }));
  const svg = renderToString((width) => fanSpec(years, String, (p) => `${p.year}`, { width }), {
    label: "a fan",
    description: "one district",
  });
  const { document } = parseHTML(svg);
  const drawing = document.querySelector('[data-at="wide"] svg')!;
  const columns = [...drawing.querySelectorAll(".fan-hit > rect")];
  expect(columns).toHaveLength(years.length);
  const ys = columns.map((rect) => {
    const y = Number(rect.getAttribute("data-y"));
    const top = Number(rect.getAttribute("y"));
    expect(y).toBeGreaterThanOrEqual(top);
    expect(y).toBeLessThanOrEqual(top + Number(rect.getAttribute("height")));
    return y;
  });
  // The high edge rises year on year, so its anchor climbs the drawing — y falls.
  expect(ys).toEqual([...ys].sort((a, b) => b - a));
  expect(new Set(ys).size).toBe(years.length);
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
  ).toBe(points.length * DRAWINGS);
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
 * shows it only as a count of anonymous `<g>` children, once over for each of the pair's drawings.
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
  for (const width of Object.values(WIDTHS)) {
    const spec = planeSpec(rules(), PLANE_AXES, { width, ...PLANE_FRAME })!;
    const svg = renderToString(() => spec, "presentational");
    const labels = [...svg.matchAll(/class="plane-label"[^>]*>([\s\S]*?)<\/g>/g)];
    // A name broken in two is one `text` of two `tspan`s, and reads back as the name it was.
    const drawn = labels.flatMap((match) =>
      [...match[1]!.matchAll(/<text[^>]*>([\s\S]*?)<\/text>/g)].map((m) =>
        m[1]!.replace(/<tspan[^>]*>/g, " ").replace(/<\/tspan>/g, "").trim(),
      ),
    );
    expect(drawn, `${width}px draws all seven names once`).toHaveLength(rules().length * DRAWINGS);
    expect(new Set(drawn), `${width}px draws each name whole`).toEqual(
      new Set(rules().map((rule) => rule.label)),
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

test("a mark the manifest lists first does not take the only clear place of one listed later", () => {
  /*
   * The narrow TTAG plane as #609 drew it, in its own pixels. The rolling count is listed first
   * and was placed above its mark — clear at the time, and the one place the mirror inside `[H]`
   * then needed. The mirror overlapped it by half a pixel here and collided on the runner's font.
   * Placed the other way round, both are clear: the order is searched, not taken.
   */
  const marks = [
    { label: "A ratchet, FY2032", px: 252.7, py: 173.3 },
    { label: "A 2% cap, FY2032", px: 69.3, py: 336.7 },
    { label: "A 1% cap, FY2036 undamped", px: 118.2, py: 235.3 },
    { label: "A rolling pupil count", px: 193.9, py: 153.9 },
    { label: "[M] mirrored inside [H]", px: 148.4, py: 159.8 },
    { label: "[M] mirrored beside [L], [M], [O]", px: 205.9, py: 53.3 },
    { label: "A dated phase-down", px: 205.9, py: 250.3 },
  ];
  const frame = { left: 4, top: 39.6, right: 296, bottom: 350 };
  const axisEnds = [
    { left: 4, top: 42.6, right: 92, bottom: 55.6 },
    { left: 4, top: 337, right: 92, bottom: 350 },
  ];
  const placed = placeLabels(marks, frame, axisEnds);
  const covers = (a: Box, b: Box) =>
    Math.min(a.right, b.right) > Math.max(a.left, b.left) && Math.min(a.bottom, b.bottom) > Math.max(a.top, b.top);
  const through: string[] = [];
  placed.forEach((one, i) => {
    placed.slice(i + 1).forEach((other, j) => {
      if (covers(one.box, other.box)) through.push(`${marks[i]!.label} / ${marks[i + 1 + j]!.label}`);
    });
  });
  expect(through).toEqual([]);
});

test("the hit layer matches the dot layer place for place", () => {
  const spec = planeSpec(rules(), PLANE_AXES, PLANE)!;
  expect(layerRows(spec, "plane-hit")).toBe(layerRows(spec, "plane-dot"));
  expect(spec.hovers!.text).toHaveLength(rules().length);
  // The dot is what lights up under the cursor, and the hit target is the invisible thing the
  // pointer actually finds. `declareCursor` checks the pairing; this checks it was declared.
  expect(spec.hovers!.cursor).toEqual({ second: "paired marks", layers: [".plane-dot"] });
});

test("a cloud and a plane are walked left to right, whatever order their caller passed", () => {
  /*
   * The arrow keys walk the hit layer in document order, and the callers pass districts
   * alphabetically: ArrowRight on `/outcomes` went Manchester, Akron, Alliance, Ashland, 137px a
   * step on average (#614). The readings are in the drawn order, so following them is following
   * the walk.
   */
  const shuffled = cloud([9, 3, 7, 1, 12, 5, 11, 2, 8, 4, 10, 6].map((x) => x * 1000));
  const scatter = scatterSpec(shuffled, AXES, [], W)!;
  const xs = scatter.hovers!.text.map((h) => shuffled.find((p) => p.hover === h)!.x);
  expect(xs).toEqual([...xs].sort((a, b) => a - b));

  // A tie on x goes upwards, so two districts on one x are still adjacent stops.
  const plane = planeSpec(rules(), PLANE_AXES, PLANE)!;
  expect(plane.hovers!.text).toEqual([
    "2% cap",
    "1% cap",
    "mirror inside",
    "rolling",
    "phase-down",
    "mirror beside",
    "ratchet",
  ]);
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
  expect(rules(barSpec(flat, { width, hue: "formula" })), "no rule without a signed scale").toBe(0);
  // One rule per drawing `renderToString` lays out.
  expect(rules(barSpec(flat, { width, hue: "formula", max: 166.55, min: -83.51 }))).toBe(DRAWINGS);
  // And the panel is drawn on the set's scale, not on its own nothing.
  expect(barSpec(flat, { width, hue: "formula", max: 166.55, min: -83.51 }).options.x!.domain).toEqual([
    -83.51, 166.55,
  ]);
});

test("a shared scale is the same scale: two panels of one set draw a value at one length", () => {
  // What makes the monotonicity readable across seven panels is that the axis does not move.
  const of = (bars: Bar[]) =>
    barSpec(bars, { width: panelWidth(7), hue: "formula" as const, max: 166.55, min: -83.51 }).options.x!.domain;
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
  const set = { width: panelWidth(7), hue: "formula" as const, max: 166.55, min: -83.51, labelChars: 2 };
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
  expect(barSpec(drawn, { width: WIDTHS.wide, hue: "formula" }).options.marginRight).toBe(
    barSpec(zeros, { width: WIDTHS.wide, hue: "formula", labelChars: 0 }).options.marginRight,
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
  expect(svg).toContain("What ±1σ claims");
  // The categorical pair is the two quantities. A reference in a third hue would read as a third.
  expect(svg).not.toContain("var(--series-c)");
  // Once per drawing, and the pair is drawn at every one of `WIDTHS`.
  for (const drawing of svg.split("<svg").slice(1)) {
    expect((drawing.match(/What ±1σ claims/g) ?? []).length).toBe(1);
  }
});

test("the reference label sits at the right end on a halo, not where both lines start", () => {
  /*
   * #662. At the left end above the rule, the series lines ran through "What ±1σ claims to hold"
   * at every width on `/method`. A halo in the card's colour cuts a line that still crosses it.
   */
  const svg = renderToString(
    (w) =>
      seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", {
        width: w,
        tick: (at) => `${at}`,
        reference: { value: 0.683, label: "what ±1σ claims" },
      }),
    "presentational",
  );
  const { document } = parseHTML(`<div>${svg}</div>`);
  const labels = [...document.querySelectorAll("g.series-reference")].filter((g) => g.querySelector("text"));
  expect(labels).toHaveLength(DRAWINGS);
  for (const label of labels) {
    expect(label.getAttribute("text-anchor")).toBe("end");
    expect(label.getAttribute("stroke")).toBe(INK.surface);
  }
});

test("every dash in the chart forms comes from the rule tokens", () => {
  // #662: four dash patterns and two greys meant the same thing in different charts. A literal
  // here is a fifth.
  const source = readFileSync(resolve(process.cwd(), "src/lib/plot/spec.ts"), "utf8");
  expect(source.match(/strokeDasharray:\s*["'`]/g) ?? []).toEqual([]);
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
  expect(svg).toContain("One year");
  expect(svg).toContain("3 years");
  // The `FY` prefix used to be written here rather than passed in, which is what made this form
  // unusable for the one caller whose index counts something else.
  expect(svg).not.toContain("FY");
});

test("the reference label is wrapped to the frame, so it cannot reach the start labels' gutter", () => {
  /*
   * #655's CI failure. On `/method` at 375 the right-anchored "What ±1σ claims to hold" ran left out
   * of the 300px drawing's frame and through the lower line's "66%" start label in the gutter.
   * Measured with the file's own p90 glyph width, which is wider than CI's fonts paint.
   */
  const label = "what ±1σ claims to hold";
  for (const width of Object.values(WIDTHS)) {
    const spec = seriesSpec(HELD, { a: "pooled", b: "cross-district" }, share, () => "", {
      width,
      tick: (at) => `${at}`,
      reference: { value: 0.683, label },
    })!;
    const frame = width - Number(spec.options.marginLeft) - Number(spec.options.marginRight);
    const { document } = parseHTML(`<div>${drawingAt(() => spec)}</div>`);
    const lines = [...document.querySelectorAll("g.series-reference text")].flatMap((t) => {
      const spans = [...t.querySelectorAll("tspan")];
      return (spans.length ? spans : [t]).map((n) => n.textContent ?? "");
    });
    expect(lines.join(" "), `${width}px`).toBe("What ±1σ claims to hold");
    for (const line of lines) expect(line.length * 0.667 * 11, `${width}px "${line}"`).toBeLessThanOrEqual(frame);
  }
});

test("a series chart prints where each line starts, as well as where it ends", () => {
  /*
   * #655. Every dek over these charts is a start-to-end comparison — "fell from 45.9% to 34.5%" —
   * and only the end was drawn, so the start had to be read off a line with no axis.
   */
  const svg = renderToString(
    (w) => seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", { width: w, tick: (at) => `${at}` }),
    "presentational",
  );
  const { document } = parseHTML(`<div>${svg}</div>`);
  const starts = [...document.querySelectorAll("g.series-start")];
  expect(starts).toHaveLength(2 * DRAWINGS);
  for (const drawing of document.querySelectorAll("svg")) {
    const texts = [...drawing.querySelectorAll("g.series-start text")].map((t) => t.textContent);
    expect(texts.sort()).toEqual([share(HELD[0]!.a!), share(HELD[0]!.b!)].sort());
  }
  // In the hue's text token, and anchored at their end so they sit left of the line in the gutter.
  for (const start of starts) {
    expect([SERIES_TEXT.formula, SERIES_TEXT.guarantee]).toContain(start.getAttribute("fill"));
    expect(start.getAttribute("text-anchor")).toBe("end");
  }
});

test("a series that opens on a gap is labelled at its first value, inside the frame", () => {
  const gapped: SeriesPoint[] = [{ at: 1, a: 0.5, b: null }, ...HELD.slice(1)];
  for (const width of Object.values(WIDTHS)) {
    const svg = drawingAt(() =>
      seriesSpec(gapped, { a: "pooled", b: "cross" }, share, () => "", { width, tick: (at) => `${at}` }),
    );
    expect(svg).toContain(`>${share(HELD[1]!.b!)}<`);
    expect(overruns(svg), `${width}px`).toEqual([]);
  }
});

test("a fan given names writes them into its end labels, in their lines' text hues", () => {
  /*
   * #655. A guaranteed district's fan ended in "$114M" and "$87.2M", and which was which lived in
   * a key under the chart. The received label was also in primary ink beside a blue formula label.
   */
  const years: FanPoint[] = [2025, 2026, 2027].map((year, i) => ({
    year,
    point: 114e6,
    low: 114e6,
    high: 114e6,
    observed: i === 0,
    reference: 90e6 - i * 1e6,
  }));
  const labels = { high: "received", reference: "formula" };
  const svg = renderToString(
    (width) => fanSpec(years, compactMoney, () => "", { width, labels }),
    "presentational",
  );
  const { document } = parseHTML(`<div>${svg}</div>`);
  for (const drawing of document.querySelectorAll("svg")) {
    const ends = [...drawing.querySelectorAll("g.fan-bound text")].map((t) => [
      t.textContent,
      t.closest("[fill]")?.getAttribute("fill"),
    ]);
    expect(ends.sort()).toEqual(
      [
        [`Received ${compactMoney(114e6)}`, SERIES_TEXT.guarantee],
        [`Formula ${compactMoney(88e6)}`, SERIES_TEXT.formula],
      ].sort(),
    );
  }
  for (const width of Object.values(WIDTHS)) {
    const drawn = drawingAt(() => fanSpec(years, compactMoney, () => "", { width, labels }));
    expect(overruns(drawn), `${width}px`).toEqual([]);
  }
});

test("a fan drawn beside its reference brackets the gap between their ends and names it", () => {
  /*
   * #676. The guaranteed district's finding is the difference between "Received $114M" and
   * "Formula $87.2M", and it was printed only in the note under the chart.
   */
  const years: FanPoint[] = [2025, 2026, 2027].map((year, i) => ({
    year,
    point: 114e6,
    low: 114e6,
    high: 114e6,
    observed: i === 0,
    reference: 90e6 - i * 1.4e6,
  }));
  const labels = { high: "received", reference: "formula", gap: "guarantee" };
  // Through the caller's format, so a gutter of compact money is not bracketed in whole dollars.
  const format = (v: number) => `F${compactMoney(v)}`;
  const svg = renderToString(
    (width) => fanSpec(years, format, () => "", { width, labels }),
    "presentational",
  );
  const { document } = parseHTML(`<div>${svg}</div>`);
  for (const drawing of document.querySelectorAll("svg")) {
    const gap = [...drawing.querySelectorAll("g.fan-gap text")];
    expect(gap.map((t) => t.textContent)).toEqual([`+${format(114e6 - 87.2e6)} guarantee`]);
    // A measurement between the lines, so in muted ink and not either line's hue.
    expect(gap[0]!.closest("[fill]")?.getAttribute("fill")).toBe(INK.muted);
    expect(drawing.querySelectorAll("g.fan-gap line")).toHaveLength(1);
  }
  for (const width of Object.values(WIDTHS)) {
    const drawn = drawingAt(() => fanSpec(years, format, () => "", { width, labels }));
    expect(overruns(drawn), `${width}px`).toEqual([]);
  }

  // No bracket where the two ends print as one figure: "+$0" is a difference the chart cannot see.
  const level = years.map((p) => ({ ...p, reference: 114e6 + 1 }));
  const flat = renderToString(
    (width) => fanSpec(level, compactMoney, () => "", { width, labels }),
    "presentational",
  );
  expect(flat).not.toContain("fan-gap");
  // Nor without a reference, whatever names are passed.
  const alone = renderToString(
    (width) => fanSpec(years.map(({ reference: _, ...p }) => p), compactMoney, () => "", { width, labels }),
    "presentational",
  );
  expect(alone).not.toContain("fan-gap");
});

test("an open fan given bound names writes both into its end labels", () => {
  /*
   * #675. The /scenario fan ended in "$7.58B" and "$6.96B", and nothing on the drawing said those
   * were the top and bottom of the range.
   */
  const years: FanPoint[] = [2025, 2026, 2027].map((year, i) => ({
    year,
    point: 7.2e9,
    low: 7.2e9 - i * 0.12e9,
    high: 7.2e9 + i * 0.19e9,
    observed: i === 0,
  }));
  const labels = { high: "high", low: "low" };
  const svg = renderToString(
    (width) => fanSpec(years, compactMoney, () => "", { width, labels }),
    "presentational",
  );
  const { document } = parseHTML(`<div>${svg}</div>`);
  for (const drawing of document.querySelectorAll("svg")) {
    const ends = [...drawing.querySelectorAll("g.fan-bound text")].map((t) => t.textContent);
    expect(ends).toEqual([`High ${compactMoney(7.58e9)}`, `Low ${compactMoney(6.96e9)}`]);
  }
  for (const width of Object.values(WIDTHS)) {
    const drawn = drawingAt(() => fanSpec(years, compactMoney, () => "", { width, labels }));
    expect(overruns(drawn), `${width}px`).toEqual([]);
  }
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

test("no drawing is scaled up past the ceiling that keeps its type at body size", () => {
  // Every width a chart is drawn at — the three of `pair` and a panel's own — against the viewBox it
  // actually came out at, so a builder that ignored its width would fail here, not pass on it.
  const series = (w: number) =>
    seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", { width: w, tick: (at) => `${at}` });
  const drawn = renderToString(series, "presentational") + renderPanelToString(series, "presentational", panelWidth(2));
  const svgs = [...drawn.matchAll(/<svg\b[^>]*>/g)].map((m) => m[0]);
  expect(svgs).toHaveLength(4);
  for (const svg of svgs) {
    const box = Number(/viewBox="0 0 (\d+(?:\.\d+)?) /.exec(svg)?.[1]);
    const cap = Number(/max-width:(\d+(?:\.\d+)?)px/.exec(svg)?.[1]);
    expect(cap, svg).toBe(box * MAX_SCALE);
  }
  // 12 units is `BASE`'s type, the largest any form sets; 15px is the body text it must not pass.
  expect(12 * MAX_SCALE).toBeLessThanOrEqual(15);
});

test("each drawing takes over at its own width, from one already at its cap", () => {
  /*
   * #609. With the swap anywhere else, one of two things comes back: a band where the drawing
   * shown has stopped at its cap and its box runs on, or a drawing shown smaller than it was laid
   * out. The container queries are read from the stylesheet, because they are where the swap
   * actually happens.
   */
  const css = readFileSync(resolve(process.cwd(), "src/styles/app.css"), "utf8");
  const swaps = [...css.matchAll(/@container \(min-width: (\d+)px\)/g)].map((m) => Number(m[1]));
  expect(swaps).toEqual([WIDTHS.middle, WIDTHS.wide]);

  const order = [WIDTHS.narrow, WIDTHS.middle, WIDTHS.wide];
  for (let i = 1; i < order.length; i += 1) {
    const before = order[i - 1]! * MAX_SCALE;
    const at = order[i]!;
    // The one before has stopped growing by the swap, so the step in painted type is exactly
    // 1 - 1/MAX_SCALE, a fifth.
    expect(before, `${at}`).toBeLessThan(at);
    // And the band it cannot fill is never more than 15% of its box short.
    expect(before / at, `${at}`).toBeGreaterThan(0.85);
  }
});

/**
 * Every text mark in one drawing that would be painted past its viewBox, with how far.
 *
 * The browser measures this in `charts.spec.ts`; here there is no font, so a line is taken at
 * 0.667em a character — `EM_PER_CHAR` in `spec.ts`, the ninetieth percentile of what this site's
 * marks actually paint at — and a line box at 0.8em above its baseline and 0.25em below. Plot's
 * own output is read rather than the spec, so a translate on a group, a `dy`, a text anchor and a
 * wrapped `tspan` are all counted where they land. Loose enough to pass a label that fits, and
 * tight enough to fail the 139 units the TTAG plane's y title ran past its own panel.
 */
function overruns(svg: string): string[] {
  const { document } = parseHTML(`<!doctype html><html><body>${svg}</body></html>`);
  const root = document.querySelector("svg")!;
  const [, , width, height] = root.getAttribute("viewBox")!.split(" ").map(Number) as [
    number,
    number,
    number,
    number,
  ];
  const inherited = (el: Element, name: string): string | null =>
    el.closest(`[${name}]`)?.getAttribute(name) ?? null;
  const em = (value: string | null, size: number) =>
    value == null ? 0 : value.endsWith("em") ? parseFloat(value) * size : parseFloat(value);
  const out: string[] = [];
  for (const text of root.querySelectorAll("text")) {
    let x = 0;
    let y = 0;
    for (let el: Element | null = text; el && el !== root; el = el.parentElement) {
      const t = /translate\(([-\d.]+),\s*([-\d.]+)\)/.exec(el.getAttribute("transform") ?? "");
      if (t) {
        x += Number(t[1]);
        y += Number(t[2]);
      }
    }
    const size = Number(inherited(text, "font-size") ?? 10);
    const anchor = inherited(text, "text-anchor") ?? "start";
    const spans = [...text.querySelectorAll("tspan")];
    const lines = spans.length > 0 ? spans.map((s) => s.textContent ?? "") : [text.textContent ?? ""];
    const w = Math.max(...lines.map((line) => line.trim().length)) * 0.667 * size;
    const left = x - (anchor === "end" ? w : anchor === "middle" ? w / 2 : 0);
    // Baselines: the first line's `y`, and each later line one `dy` below the last.
    let baseline = y + em(spans[0]?.getAttribute("y") ?? text.getAttribute("y"), size);
    const first = baseline;
    for (const span of spans.slice(1)) baseline += em(span.getAttribute("dy"), size);
    const box = { left, right: left + w, top: first - 0.8 * size, bottom: baseline + 0.25 * size };
    const over = Math.max(-box.left, box.right - width, -box.top, box.bottom - height);
    if (over > 0.5) out.push(`"${lines.join(" / ").trim()}" by ${over.toFixed(1)}`);
  }
  return out;
}

/** The one drawing a fixed-width builder makes, out of the pair `renderToString` returns. */
function drawingAt(build: () => Spec | null): string {
  const svg = /<svg\b[\s\S]*?<\/svg>/.exec(renderToString(build, "presentational"));
  expect(svg, "the builder drew something").not.toBeNull();
  return svg![0];
}

/** An axis name as long as the TTAG plane's, which is 85 characters. */
const LONG_AXIS =
  "The per-pupil term: the FY2027 formula against the FY2020 regime, per pupil, in logs too";

test("an axis name as long as a sentence wraps inside the drawing at every width it is drawn at", () => {
  /*
   * #608. The dropped foot line was given "the whole width" and had the width less `marginLeft`;
   * the scatter's and the plane's y titles were single lines. On the TTAG node's two-panel plane
   * the left panel's y title ran 139 units into the right panel and its x title 51, and on `/bounds`
   * at 375 the rank chart's axis name started 173px in and ended past the viewport.
   */
  expect(LONG_AXIS.length).toBeGreaterThanOrEqual(85);
  const axis = { label: LONG_AXIS, format: (v: number) => String(v) };
  for (const width of [WIDTHS.narrow, panelWidth(2)]) {
    const rank = drawingAt(() => rankSpec(census([554, 499, 43, 1, 0]), axis, { width }));
    const scatter = drawingAt(() =>
      scatterSpec(cloud([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]), { x: axis, y: axis }, [], { width }),
    );
    const plane = drawingAt(() =>
      planeSpec(rules(), { x: axis, y: axis }, { width, ...PLANE_FRAME }),
    );
    expect(overruns(rank), `rank at ${width}`).toEqual([]);
    expect(overruns(scatter), `scatter at ${width}`).toEqual([]);
    expect(overruns(plane), `plane at ${width}`).toEqual([]);
  }
});

test("a wrapped axis name costs the margin its lines take, so the frame moves rather than the type", () => {
  // The top margin grows by a line per line of the y title and the foot by one per line of the
  // dropped x name, so nothing written above or below the frame is drawn into it.
  const short = { label: "Spending", format: (v: number) => String(v) };
  const long = { label: LONG_AXIS, format: (v: number) => String(v) };
  const at = (axes: { x: typeof short; y: typeof short }) =>
    planeSpec(rules(), axes, { width: WIDTHS.narrow, ...PLANE_FRAME })!.options;
  const plain = at({ x: short, y: short });
  const wrapped = at({ x: long, y: long });
  expect(wrapped.marginTop! - plain.marginTop!).toBeGreaterThan(0);
  expect((wrapped.marginTop! - plain.marginTop!) % 14).toBe(0);
  expect(wrapped.marginBottom! - plain.marginBottom!).toBeGreaterThan(16);
  expect((wrapped.marginBottom! - plain.marginBottom!) % 16).toBe(0);
});

test("a range row is its full height at any count, with the foot added rather than taken out", () => {
  /*
   * `height` was `rows × 14` with the foot's 22 units subtracted from it, so a six-row chart left
   * 10.3 units a row for 10-unit names and they ran into each other — 4.5px at 375 on the
   * base-cost node. Eighty-four rows hid it, which is why `/counties` never showed it.
   */
  const axis = { label: "Pupils", format: (v: number) => String(v) };
  for (const n of [3, 6, 84]) {
    const rows: Range[] = Array.from({ length: n }, (_, i) => ({
      label: `row ${i}`,
      low: 100 + i,
      high: 200 + i * 3,
      hover: `row ${i}`,
    }));
    for (const width of Object.values(WIDTHS)) {
      const { height, marginTop, marginBottom } = rangeSpec(rows, axis, { width })!.options;
      expect((height! - (marginTop ?? 0) - marginBottom!) / n, `${n} rows at ${width}`).toBeGreaterThanOrEqual(14);
    }
  }
});

test("a rank row is its full height at any count, as a range row is", () => {
  const { height, marginTop, marginBottom } = rankSpec(census([9, 3, 1]), COUNT, W)!.options;
  expect((height! - (marginTop ?? 0) - marginBottom!) / 3).toBeGreaterThanOrEqual(16);
});

test("a name with no room beside its mark is broken in two rather than drawn off the drawing", () => {
  /*
   * At the narrow width `[M] mirrored beside [L], [M], [O]` is 33 characters, about 242 units,
   * in a 218-unit frame: it fits at none of the four offsets. The placer fell through to the first
   * of them, drew it rightward from a mark at the frame's right edge, and the SVG cut it to
   * `[M] mirrored beside [L], [`.
   */
  const long = "[M] mirrored beside [L], [M], [O]";
  expect(long).toHaveLength(33);
  const places: Place[] = [
    ...rules(),
    { label: long, x: PLANE_FRAME.xDomain[1], y: 30e6, hover: "beside all three" },
  ];
  for (const width of [...Object.values(WIDTHS), panelWidth(2)]) {
    const svg = drawingAt(() => planeSpec(places, PLANE_AXES, { width, ...PLANE_FRAME }));
    expect(overruns(svg), `${width}px`).toEqual([]);
  }
});

test("a panel is never drawn at the width of a whole chart", () => {
  // `panelWidth(1)` is the wide frame, and a one-panel spread drawn once at it was the 640
  // drawing on a phone. `renderToString` is the route for a chart with no sibling.
  const series = (w: number) =>
    seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", { width: w, tick: (at) => `${at}` });
  expect(() => renderPanelToString(() => series(panelWidth(1)), "presentational", panelWidth(1))).toThrow(
    /renderToString/,
  );
  expect(renderPanelToString(() => series(panelWidth(2)), "presentational", panelWidth(2))).toContain("<svg");
});

test("a range chart sorted on a figure prints that figure on every row", () => {
  /*
   * #656. `/counties` sorts 84 rows widest span first by the high-to-low ratio and printed the
   * ratio nowhere, so the reader had to recover the order from segment lengths on a log axis.
   */
  const rows: Range[] = Array.from({ length: 12 }, (_, i) => ({
    label: `County ${i}`,
    low: 1000,
    high: 1000 * (8 - i * 0.5),
    hover: "",
  }));
  const axis = { label: "valuation per pupil", format: compactMoney, log: true };
  const spec = rangeSpec(rows, axis, { ...W, rowLabel: (r) => `${(r.high / r.low).toFixed(1)}×` })!;
  const svg = renderToString(() => spec, "presentational");
  const labels = [...parseHTML(`<div>${svg}</div>`).document.querySelectorAll(".range-row-label text")].map(
    (t) => t.textContent,
  );
  expect(labels, "one per row, in each layout").toHaveLength(rows.length * DRAWINGS);
  expect(labels[0]).toBe("8.0×");
  // And the widest of them has room past the frame, where the row holding the maximum draws it.
  expect(spec.options.marginRight).toBeGreaterThan(rangeSpec(rows, axis, W)!.options.marginRight as number);
});

test("a reference row is drawn hollow, so it does not read as one more member", () => {
  /*
   * #656. `/statewide` draws the six highest local shares and Ohio, and Ohio was the shortest bar
   * — read as the lowest, when Ohio is 7th of 51 and well above the national 43%. The national
   * figure is now a row of its own, outlined rather than filled.
   */
  const bars: Bar[] = [
    { label: "New Hampshire", value: 63, direct: "63%" },
    { label: "Ohio (7th of 51)", value: 52, direct: "52%", current: true },
    { label: "All 51, combined", value: 43.4, direct: "43%", reference: true },
  ];
  const { document } = parseHTML(
    `<div>${renderToString((w) => barSpec(bars, { width: w, hue: "plain" }), "presentational")}</div>`,
  );
  const outlines = [...document.querySelectorAll(".bar-reference")];
  expect(outlines).toHaveLength(DRAWINGS);
  for (const outline of outlines) {
    expect(outline.children).toHaveLength(1);
    expect(outline.getAttribute("fill")).toBe("none");
    expect(outline.getAttribute("stroke")).toBe(INK.muted);
  }
  // The hit bar is transparent on the reference row and keeps the chart's hue on the others.
  const fills = [...document.querySelector(".bar-fill")!.children].map((rect) => rect.getAttribute("fill"));
  expect(fills).toEqual([INK.muted, INK.muted, "transparent"]);
});

test("a range chart too tall to read against its foot states its scale at the top as well", () => {
  /*
   * #611: `/counties` draws 84 rows in 1470px and said what a position meant only at the bottom,
   * a screen and a half from the first row. Above the threshold the scale is repeated over the
   * frame; a short chart keeps the foot alone.
   */
  const rows = (n: number): Range[] =>
    Array.from({ length: n }, (_, i) => ({ label: `County ${i}`, low: 1000 + i * 50, high: 4000 + i * 90, hover: "" }));
  const axis = { label: "aid per pupil", format: compactMoney };
  const ends = (svg: string, cls: string) =>
    [...parseHTML(`<div>${svg}</div>`).document.querySelectorAll(`g.${cls} text`)].filter((t) =>
      /\$/.test(t.textContent ?? ""),
    ).length;

  const tall = renderToString(() => rangeSpec(rows(84), axis, W), "presentational");
  expect(ends(tall, "axis-foot")).toBeGreaterThanOrEqual(2 * DRAWINGS);
  expect(ends(tall, "axis-head")).toBeGreaterThanOrEqual(2 * DRAWINGS);
  expect(rangeSpec(rows(84), axis, W)!.options.marginTop).toBeGreaterThan(0);

  const short = renderToString(() => rangeSpec(rows(6), axis, W), "presentational");
  expect(ends(short, "axis-foot")).toBeGreaterThanOrEqual(2 * DRAWINGS);
  expect(ends(short, "axis-head")).toBe(0);
});

test("a spread's rules are told apart by dash, as their legend swatches are", () => {
  /*
   * #611: the TTAG spread draws "the floor equals the formula" and "the formula pays what the FY2020
   * regime did", and both were dashed `4 3` with one grey key swatch between them. Within one
   * figure every rule now has its own (stroke, dash) and its own swatch.
   */
  const points: ScatterPoint[] = Array.from({ length: 20 }, (_, i) => ({ x: i, y: i % 7, hover: "" }));
  const rules = [
    { label: "y = -x", from: { x: 0, y: 0 }, to: { x: 19, y: -19 } },
    { label: "y = 0", from: { x: 0, y: 0 }, to: { x: 19, y: 0 } },
  ];
  const svg = renderToString(() => scatterSpec(points, AXES, [], { ...W, rules }), "presentational");
  const drawn = [...parseHTML(`<div>${svg}</div>`).document.querySelectorAll("g.scatter-rule")]
    .slice(0, rules.length)
    .map((g) => {
      const path = g.querySelector("path");
      return `${g.getAttribute("stroke") ?? path?.getAttribute("stroke")} ${g.getAttribute("stroke-dasharray") ?? path?.getAttribute("stroke-dasharray")}`;
    });
  expect(drawn).toHaveLength(2);
  expect(new Set(drawn).size).toBe(2);
  expect(new Set(rules.map((_, at) => ruleSwatch(at))).size).toBe(2);
  expect(() => ruleSwatch(rules.length)).toThrow(/no dash of its own/);
});

/**
 * A chart's name is its finding, and the method is its description (#612).
 *
 * The description was appended to the name, and a screen reader speaks a name in full before a
 * reader can move past the chart — 577 characters on `/method`. A name ending in a full stop and
 * joined to that with ". " also read "down to 0..".
 */
test("a chart's description is a <desc>, not part of its name", () => {
  const svg = renderToString(
    (width) => barSpec([{ label: "One", value: 1, hover: "One: 1" }], { width, hue: "formula" }),
    { label: "A chart that ends in a full stop.", description: "How it was cut." },
  );
  const { document } = parseHTML(`<!doctype html><html><body>${svg}</body></html>`);
  const charts = [...document.querySelectorAll("svg.plot")];
  expect(charts.length).toBeGreaterThan(0);
  for (const chart of charts) {
    expect(chart.getAttribute("aria-label")).toBe("A chart that ends in a full stop");
    expect(chart.querySelector("desc")?.textContent).toBe("How it was cut.");
  }
});

/**
 * Every axis title, foot and end label is drawn in sentence case, whatever case the caller wrote
 * it in (#612): one page said "assessed valuation per pupil" and the next "Formula aid per pupil".
 */
test("a chart's own words start with a capital, whatever the caller wrote", () => {
  const svg = renderToString(
    (width) =>
      scatterSpec(
        cloud([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
        {
          x: { label: "assessed valuation per pupil", format: String },
          y: { label: "state aid per pupil", format: String },
        },
        [],
        { width },
      ),
    { label: "a cloud" },
  );
  expect(svg).toContain(">Assessed valuation per pupil<");
  expect(svg).toContain("State aid per pupil");
  expect(svg).not.toMatch(/>(assessed|state aid)/);
});

/**
 * A chart's words are ink, never a mark's colour (#600). The series hues are picked to separate
 * from each other, not to be read: at 12px the light guarantee mark measured 3.12:1 on the card,
 * the light formula mark 4.30:1. A label names its series in the text token of that hue, and the
 * line or dot beside it carries the mark colour.
 *
 * Rendered rather than read off the marks, so a fill passed as a function, or inherited from a
 * group, is caught as well as a constant.
 */
test("no chart text is filled with a series or ordinal mark colour", () => {
  const marks = new Set<string>([...Object.values(SERIES), ...ORDINAL]);
  const fan: FanPoint[] = [2025, 2026, 2027].map((year, i) => ({
    year,
    point: 100 + i,
    low: 99 + i,
    high: 101 + i,
    observed: i === 0,
    reference: 98 + i,
  }));
  const traced: Trace[] = (["formula", "guarantee"] as const).map((series, i) => ({
    label: `${series} trace`,
    series,
    points: [{ x: 10_000, y: 50 + i }, { x: 20_000, y: 55 + i }],
  }));
  const strip = [-0.08, -0.03, -0.01, 0.02, 0.05, 0.09].map((value) => ({ value, hover: `${value}` }));
  const builds: [string, (width: number) => Spec | null][] = [
    ["bar", (width) => barSpec([{ label: "Ohio", value: 45, current: true }, { label: "Utah", value: -3 }], { width, hue: "formula" })],
    ["scatter", (width) => scatterSpec(cloud(Array.from({ length: 30 }, (_, i) => i)), AXES, traced, { width })],
    ["plane", (width) => planeSpec(rules(), PLANE_AXES, { width, ...PLANE_FRAME })],
    ["rank", (width) => rankSpec(census([554, 499, 43, 1, 0]), COUNT, { width })],
    ["range", (width) => rangeSpec([{ label: "row", low: 1, high: 2, hover: "row" }], COUNT, { width })],
    ["distribution", (width) => distributionSpec(strip, { width, format: (v) => pct(v, 0) })],
    ["fan", (width) => fanSpec(fan, (v) => `${v}`, () => "", { width })],
    [
      "series",
      (width) =>
        seriesSpec(HELD, { a: "pooled", b: "cross" }, share, () => "", {
          width,
          tick: (at) => `${at}`,
          reference: { value: 0.683, label: "what it claims" },
        }),
    ],
  ];

  const offenders: string[] = [];
  let texts = 0;
  for (const [form, build] of builds) {
    const { document } = parseHTML(`<div>${renderToString(build, "presentational")}</div>`);
    for (const text of document.querySelectorAll("svg text")) {
      texts++;
      let fill: string | null = null;
      for (let at: Element | null = text; at && fill == null; at = at.parentElement) {
        fill = at.getAttribute("fill");
      }
      if (fill != null && marks.has(fill)) offenders.push(`${form}: "${text.textContent}" in ${fill}`);
    }
  }
  // Every form drew words, or the scan passed on nothing.
  expect(texts).toBeGreaterThan(builds.length * DRAWINGS);
  expect(offenders).toEqual([]);
});
