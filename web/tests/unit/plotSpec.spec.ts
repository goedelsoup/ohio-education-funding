/**
 * What a chart has to state about its own scale before a reader can take a value off it.
 *
 * Six charts on this site omitted something a reader needed — #109 grouped them because they are
 * one judgement repeated rather than six unrelated faults. Each of these tests holds one of those
 * judgements in the layer that made it, so the next chart built on the same specification inherits
 * the answer instead of the omission.
 */

import { expect, test } from "vitest";

import type { Bar, FanPoint, ScatterPoint, Trace } from "../../src/lib/chart.ts";
import {
  barSpec,
  distributionSpec,
  fanSpec,
  scatterSpec,
  truncatedDomain,
} from "../../src/lib/plot/spec.ts";
import { renderToString } from "../../src/lib/plot/ssr.ts";

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
  const svg = renderToString(barSpec(bars), "presentational");
  expect(svg).toContain("var(--series-guarantee)");
  expect(svg).toContain('font-weight="600"');

  // And a chart with no subject is unchanged: same fills, no second text mark.
  const plain = renderToString(barSpec(bars.map(({ current: _c, ...b }) => b)), "presentational");
  expect(plain).not.toContain("var(--series-guarantee)");
  expect(plain).not.toContain('font-weight="600"');
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
  );
  expect(signed).not.toBeNull();
  expect(signed!.options.height).toBe(64);
  expect(renderToString(signed, "presentational")).toContain("no change");

  const unsigned = distributionSpec(
    [1, 2, 3, 4, 5, 6].map((value) => ({ value, hover: `${value}` })),
  );
  expect(unsigned!.options.height).toBe(46);
  expect(renderToString(unsigned, "presentational")).not.toContain("no change");
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

  const withBands = scatterSpec(points, AXES, banded)!;
  const withNone = scatterSpec(points, AXES, [])!;
  expect(withBands.options.marginRight).toBe(withNone.options.marginRight);

  // A trace that *is* labelled still gets its room, or the label runs off the viewBox.
  const labelled = scatterSpec(points, AXES, [
    { label: "median of each fifth", series: "formula", points: banded[0]!.points },
  ])!;
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

  const a = scatterSpec(narrow, AXES, [], { xDomain: shared })!;
  const b = scatterSpec(wide, AXES, [], { xDomain: shared })!;
  expect(a.options.x!.domain).toEqual(b.options.x!.domain);

  // Without it they do not agree, which is the state this option exists to leave behind.
  const own = scatterSpec(narrow, AXES, [])!;
  expect(own.options.x!.domain).not.toEqual(b.options.x!.domain);
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

  const clean = fanSpec(base, (v) => `${v}`, () => "")!;
  const withOrphan = fanSpec(partial, (v) => `${v}`, () => "")!;
  expect(withOrphan.options.y!.domain).toEqual(clean.options.y!.domain);

  // A reference on every year is drawn, and does belong in the domain.
  const full = fanSpec(
    base.map((p) => ({ ...p, reference: 400 })),
    (v) => `${v}`,
    () => "",
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
