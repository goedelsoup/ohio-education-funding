/**
 * The browser projection, checked against the real feed.
 *
 * The load-bearing test is the first one: it runs every Rust-computed forecast through the
 * TypeScript against all 606 districts, at both ends of the band as well as the middle. A page
 * that reproduced the point and got the interval wrong would look exactly right, and that is the
 * failure this whole axis exists to prevent.
 *
 * The rest pin the properties the arithmetic is supposed to have, on constructed series where
 * the right answer is known by hand rather than by running the code.
 */

import { expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { currentLaw } from "../../src/lib/policy.ts";
import {
  advance,
  fit,
  forecast,
  forecastPath,
  growthPrior,
  observations,
  projectSeries,
  spread,
  standardDeviation,
  type Observation,
} from "../../src/lib/project.ts";
import { type FanPoint } from "../../src/lib/chart.ts";
import { fanSpec } from "../../src/lib/plot/spec.ts";
import { renderToString } from "../../src/lib/plot/ssr.ts";
import { compareForecast, isForecastVerified, verify } from "../../src/lib/verify.ts";
import type { Bundle } from "../../src/lib/types.ts";

const bundle: Bundle = JSON.parse(
  readFileSync(join(import.meta.dirname, "../../public/data/bundle.json"), "utf8"),
);
const meta = bundle.projection!;
const model = bundle.statewide.minimum_state_share;

const series = (start: number, values: number[]): Observation[] =>
  values.map((value, i) => ({ fiscalYear: start + i, value }));

const flat = { sigma: 0.02, z: 1 };

test("the feed carries a projection block and forecasts to check it with", () => {
  expect(meta).not.toBeNull();
  expect(meta.method).toBe("damped");
  expect(meta.checkpoints.length).toBeGreaterThan(0);
  expect(meta.horizon).toBeGreaterThan(meta.base_year);
});

test("every Rust-computed forecast is reproduced by the browser projection", () => {
  const failures = meta.checkpoints
    .map((c) => compareForecast(bundle, c))
    .filter((c) => !c.agrees);
  expect(failures.map((f) => `${f.label}: ${f.differences.join("; ")}`)).toEqual([]);
  expect(isForecastVerified(verify(bundle))).toBe(true);
});

test("the prior this page computes is the one the feed declares", () => {
  // Recomputed rather than read, so a page and a feed that disagreed would fail the forecasts
  // above. This checks the two have not silently drifted into agreeing about nothing.
  const prior = growthPrior(bundle.districts, meta.z);
  expect(Math.abs(prior.sigma - meta.sigma)).toBeLessThan(1e-6);
  expect(prior.sigma).toBeGreaterThan(0.001);
  expect(prior.sigma).toBeLessThan(0.15);
});

test("every district carries three dated enrolled-ADM observations", () => {
  // Not nullable in the contract, because a district silently dropped from a projection turns a
  // statewide total into a total over a subset.
  for (const d of bundle.districts) {
    expect(d.adm_history.length, d.name).toBe(3);
    const dated = observations(d, meta.base_year);
    expect(dated[0]!.fiscalYear, d.name).toBe(meta.base_year - 2);
    expect(dated[2]!.fiscalYear, d.name).toBe(meta.base_year);
    expect(dated[2]!.value, d.name).toBeCloseTo(d.current_year_adm, 6);
  }
});

test("observed years come back unchanged and without an interval", () => {
  const out = projectSeries(series(2024, [100, 98, 96]), 2026, "damped", 0.85, flat);
  expect(out.length).toBe(3);
  for (const p of out) {
    expect(p.observed).toBe(true);
    expect(p.low).toBe(p.point);
    expect(p.high).toBe(p.point);
  }
});

test("projected years are marked and bracket their point", () => {
  const out = projectSeries(series(2024, [100, 98, 96]), 2029, "cagr", 0.85, flat);
  const forecastYears = out.filter((p) => !p.observed);
  expect(forecastYears.length).toBe(3);
  for (const p of forecastYears) {
    expect(p.low).toBeLessThan(p.point);
    expect(p.point).toBeLessThan(p.high);
  }
});

test("the interval widens with the square root of the horizon, not linearly", () => {
  // Four years out is twice as wide as one year out. A band that grew linearly would be the
  // single most common way to overstate a long-horizon forecast's uncertainty.
  expect(spread(flat, 4) / spread(flat, 1)).toBeCloseTo(2, 10);
  const out = projectSeries(series(2024, [100, 100, 100]), 2030, "last-observed", 0.85, flat);
  const projected = out.filter((p) => !p.observed);
  const width = (i: number) =>
    (projected[i]!.high - projected[i]!.low) / (2 * projected[i]!.point);
  expect(width(3) / width(0)).toBeCloseTo(2, 1);
});

test("a compound rate is fitted from the endpoints", () => {
  const method = fit(series(2024, [100, 98, 96.04]), "cagr", 0.85);
  expect(method.kind).toBe("cagr");
  expect(method.kind === "cagr" && method.rate).toBeCloseTo(-0.02, 12);
  const out = projectSeries(series(2024, [100, 98, 96.04]), 2027, "cagr", 0.85, {
    sigma: 0,
    z: 1,
  });
  expect(out[out.length - 1]!.point).toBeCloseTo(94.1192, 6);
});

test("damping is applied per year rather than once", () => {
  // Off-by-one here is invisible at a one-year horizon and enormous at ten, which is why the
  // feed carries an FY2036 checkpoint.
  const damped = advance(100, { kind: "damped", rate: -0.02, damping: 0.85 }, 3);
  let expected = 100;
  let step = -0.02;
  for (let i = 0; i < 3; i++) {
    expected *= 1 + step;
    step *= 0.85;
  }
  expect(damped).toBeCloseTo(expected, 12);
  // And it stays above the undamped path for a declining series.
  expect(damped).toBeGreaterThan(advance(100, { kind: "cagr", rate: -0.02 }, 3));
});

test("a single observation can only be carried flat, whatever was asked for", () => {
  const out = projectSeries(series(2026, [500]), 2030, "cagr", 0.85, flat);
  const last = out[out.length - 1]!;
  expect(last.point).toBeCloseTo(500, 12);
});

test("an empty series projects nothing rather than zero", () => {
  expect(projectSeries([], 2030, "damped", 0.85, flat)).toEqual([]);
});

test("the interval is proportional, so it does not depend on district size", () => {
  const width = (value: number) => {
    const out = projectSeries(series(2024, [value, value]), 2030, "last-observed", 0.85, flat);
    const last = out[out.length - 1]!;
    return (last.high - last.low) / (2 * last.point);
  };
  expect(Math.abs(width(500) - width(50_000))).toBeLessThan(1e-12);
});

test("standard deviation needs two points", () => {
  expect(standardDeviation([])).toBe(0);
  expect(standardDeviation([1])).toBe(0);
  expect(standardDeviation([2, 4, 4, 4, 5, 5, 7, 9])).toBeCloseTo(2.138089, 5);
});

test("the aid band is tighter than the enrollment band that drives it", () => {
  // Because the guarantee absorbs enrollment loss for half the state. This is the clearest
  // single statement of what the guarantee does to a projection.
  const prior = growthPrior(bundle.districts, meta.z);
  const effect = forecast(
    bundle.districts,
    currentLaw(model),
    2032,
    meta.base_year,
    meta.method,
    meta.damping,
    prior,
    model,
  );
  const aid = (effect.high - effect.low) / effect.realizedAid;
  const enrollment = 2 * spread(prior, 2032 - meta.base_year);
  expect(aid).toBeLessThan(enrollment);
});

test("removing the guarantee widens the state's exposure to enrollment error", () => {
  // The corpus finding, reproduced in the browser from the feed. The guarantee is a shock
  // absorber; without it the same forecast error reaches the appropriation far more directly.
  const prior = growthPrior(bundle.districts, meta.z);
  const at = (guarantee: "as-enacted" | "removed") => {
    const effect = forecast(
      bundle.districts,
      { ...currentLaw(model), guarantee: { kind: guarantee } },
      2032,
      meta.base_year,
      meta.method,
      meta.damping,
      prior,
      model,
    );
    return (effect.high - effect.low) / (2 * effect.realizedAid);
  };
  const enacted = at("as-enacted");
  const removed = at("removed");
  expect(removed).toBeGreaterThan(enacted * 1.5);
});

test("the path carries every observed year before it starts forecasting", () => {
  const path = forecastPath(
    bundle.districts,
    currentLaw(model),
    meta.base_year + 3,
    meta.base_year,
    meta.method,
    meta.damping,
    growthPrior(bundle.districts, meta.z),
    model,
  );
  // Three observed years and three projected ones. The observed half is the evidence for the
  // forecast, and a chart that dropped it would ask the reader to take its starting value on
  // faith.
  expect(path.length).toBe(6);
  expect(path.map((p) => p.fiscalYear)).toEqual([2024, 2025, 2026, 2027, 2028, 2029]);

  const observed = path.filter((p) => p.observed);
  expect(observed.map((p) => p.fiscalYear)).toEqual([2024, 2025, 2026]);
  for (const point of observed) {
    // No interval at all, not a zero-width one: the enrollment is published.
    expect(point.low).toBe(point.realizedAid);
    expect(point.high).toBe(point.realizedAid);
  }

  // The seam is current law at the department's own enrollment — the number the simulation card
  // shows — so the chart joins the card above it rather than starting somewhere new.
  const seam = observed[observed.length - 1]!;
  expect(seam.fiscalYear).toBe(meta.base_year);
  expect(Math.abs(seam.realizedAid - bundle.statewide.realized_aid_total)).toBeLessThan(1);

  for (const point of path.filter((p) => !p.observed)) {
    expect(point.low).toBeLessThan(point.high);
  }
});

test("the observed years are the department's own enrolled ADM, not a re-derivation", () => {
  const path = forecastPath(
    bundle.districts,
    currentLaw(model),
    meta.base_year + 1,
    meta.base_year,
    meta.method,
    meta.damping,
    growthPrior(bundle.districts, meta.z),
    model,
  );
  const observed = path.filter((p) => p.observed);
  for (const [index, point] of observed.entries()) {
    const total = bundle.districts.reduce((sum, d) => sum + d.adm_history[index]!, 0);
    expect(Math.abs(point.adm - total), `FY${point.fiscalYear}`).toBeLessThan(1e-6);
  }
  // Enrollment fell over the three observed years, which is why the trend points down.
  expect(observed[0]!.adm).toBeGreaterThan(observed[observed.length - 1]!.adm);
});

test("the fan draws the observed run-up solid and outside the band", () => {
  const points: FanPoint[] = [
    { year: 2024, point: 104, low: 104, high: 104, observed: true },
    { year: 2025, point: 102, low: 102, high: 102, observed: true },
    { year: 2026, point: 100, low: 100, high: 100, observed: true },
    { year: 2027, point: 99, low: 95, high: 103, observed: false },
    { year: 2028, point: 98, low: 91, high: 105, observed: false },
  ];
  const svg = renderToString(
    fanSpec(points, (v: number) => `$${v.toFixed(0)}`, (p: FanPoint) => `FY${p.year}`),
   { label: "test" });
  // One solid polyline over the observed years, one dashed over the projected ones.
  expect(svg).toContain("fan-observed");
  expect(svg).toContain("fan-mid");
  // The band starts at the seam, not at the origin: it spans three projected years, so its
  // outline has six vertices rather than the ten a band opened at the origin would have.
  const band = /class="fan-band"[^>]*>\s*<path d="([^"]*)"/.exec(svg)?.[1] ?? "";
  expect(band).not.toBe("");
  expect(band.split("L").length).toBe(6);
  // The axis opens on the first observed year rather than on the forecast.
  expect(svg).toContain(">FY2024<");
  expect(svg).toContain(">FY2028<");
});

test("the fan chart labels both bounds and dashes the centre", () => {
  // The design rule, made checkable: the interval is the subject, so both ends are direct
  // labelled and the point is not. A solid centre line would read as the answer.
  const points: FanPoint[] = [
    { year: 2026, point: 100, low: 100, high: 100, observed: true },
    { year: 2027, point: 99, low: 95, high: 103, observed: false },
    { year: 2028, point: 98, low: 91, high: 105, observed: false },
  ];
  const svg = renderToString(
    fanSpec(points, (v: number) => `$${v.toFixed(0)}`, (p: FanPoint) => `FY${p.year}`),
   { label: "test" });
  expect(svg).toContain("fan-band");
  expect(svg).toContain("fan-mid");
  expect(svg).toContain(">$105<");
  expect(svg).toContain(">$91<");
  expect(svg).not.toContain(">$98<");
  // One hover target per year, and an anchor marking where measurement stops. Counted by the
  // tooltips actually attached rather than by the class, which Plot puts on the group once: the
  // property worth pinning is that every year is pointable, not how the marks are nested.
  expect(svg.match(/data-hover=/g)?.length).toBe(3);
  expect(svg).toContain("fan-anchor");
});

test("a fan chart of one point draws nothing rather than a degenerate axis", () => {
  const one: FanPoint[] = [{ year: 2026, point: 1, low: 1, high: 1, observed: true }];
  expect(fanSpec(one, String, () => "")).toBeNull();
  expect(renderToString(fanSpec(one, String, () => ""), { label: "test" })).toBe("");
});

test("every colour in a rendered chart is a custom property, not a literal", () => {
  // The rule that keeps build-time SVG theme-aware. A hex code here would look correct to
  // whoever wrote it — they are on a light page — and would be wrong for every reader on a dark
  // one, with nothing to re-render it. `ensureThemeable` throws rather than emit one; this pins
  // that the charts actually shipping satisfy it.
  const points: FanPoint[] = [
    { year: 2026, point: 100, low: 100, high: 100, observed: true },
    { year: 2027, point: 99, low: 95, high: 103, observed: false },
  ];
  const svg = renderToString(fanSpec(points, (v: number) => `$${v}`, () => ""), { label: "test" });
  expect(svg).toContain("var(--series-formula)");
  expect(svg.replace(/<style>[\s\S]*?<\/style>/g, "")).not.toMatch(/#[0-9a-f]{3,8}\b/i);
});
