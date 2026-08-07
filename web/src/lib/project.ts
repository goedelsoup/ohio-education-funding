/**
 * Carrying enrollment forward, re-derived in the browser.
 *
 * This is a second implementation of `crates/project/src/series.rs` and of `forecast()` in
 * `crates/project/src/report.rs`, for the same reason `policy.ts` is a second implementation of
 * the formula: moving a lever must not need a round trip. And it stays honest the same way — the
 * feed carries Rust-computed {@link ../lib/types.ts | ForecastCheckpoint}s and this page refuses
 * to draw a band until it has reproduced every one of them, against the real panel, on load.
 *
 * Keep this a line-for-line mirror of the Rust. Any cleverness here is a liability.
 *
 * # Three observations is not a time series
 *
 * Each district has three enrolled-ADM observations. That is enough to fit a trend and nowhere
 * near enough to say how wrong the trend might be. So the interval does not come from the
 * district's own history; it comes from the cross-sectional spread of district growth rates —
 * we do not know how variable this district is, but we know how much districts differ from one
 * another, and that is a defensible floor. See {@link growthPrior}.
 */

import { apply, type Policy } from "./policy.ts";
import type { District } from "./types.ts";

/** One observed value in a fiscal year. */
export interface Observation {
  fiscalYear: number;
  value: number;
}

/** Which method to carry a series forward with. Mirrors `Method::label()` in the Rust. */
export type MethodKind = "last-observed" | "cagr" | "damped" | "linear";

/** A method with its parameters fitted to a particular series. */
export type Method =
  | { kind: "last-observed" }
  | { kind: "cagr"; rate: number }
  | { kind: "damped"; rate: number; damping: number }
  | { kind: "linear"; slope: number; intercept: number };

/** The dispersion a projection's interval rests on. */
export interface Prior {
  /** Standard deviation of the annual growth rate, as a fraction. */
  sigma: number;
  /** Standard deviations spanned on each side. */
  z: number;
}

/** A projected value with the interval that produced it. */
export interface Projected {
  fiscalYear: number;
  point: number;
  low: number;
  high: number;
  /** An observed year carries no interval; `low === point === high`. */
  observed: boolean;
}

/**
 * The multiplicative half width after `horizon` years.
 *
 * Growth errors compound, so the band widens with the square root of the horizon rather than
 * staying fixed — the standard random-walk result, and the reason a five-year projection is not
 * five times as uncertain as a one-year one.
 */
export function spread(prior: Prior, horizon: number): number {
  return prior.z * prior.sigma * Math.sqrt(horizon);
}

/** Standard deviation over `n − 1`. */
export function standardDeviation(values: number[]): number {
  if (values.length < 2) return 0;
  const n = values.length;
  const mean = values.reduce((a, b) => a + b, 0) / n;
  const sum = values.reduce((a, b) => a + (b - mean) ** 2, 0);
  return Math.sqrt(sum / (n - 1));
}

function compoundRate(from: number, to: number, years: number): number {
  if (from <= 0 || to <= 0 || years <= 0) return 0;
  return Math.pow(to / from, 1 / years) - 1;
}

/** Slope per year and fitted value at the last observed year. */
function leastSquares(observations: Observation[]): [number, number] {
  const n = observations.length;
  const xs = observations.map((o) => o.fiscalYear);
  const meanX = xs.reduce((a, b) => a + b, 0) / n;
  const meanY = observations.reduce((a, o) => a + o.value, 0) / n;
  let numerator = 0;
  let denominator = 0;
  for (let i = 0; i < n; i++) {
    numerator += (xs[i]! - meanX) * (observations[i]!.value - meanY);
    denominator += (xs[i]! - meanX) ** 2;
  }
  const slope = denominator === 0 ? 0 : numerator / denominator;
  return [slope, meanY + slope * (xs[n - 1]! - meanX)];
}

/**
 * Resolve a requested method against what the data can support.
 *
 * One point cannot support a trend, whatever was asked for, and saying so in the returned method
 * is better than carrying it flat silently. The Rust has one more case — an assumed rate, which
 * survives a single observation because it never came from the data — and the feed never uses it,
 * so it is not mirrored here.
 */
export function fit(
  observations: Observation[],
  kind: MethodKind,
  damping: number,
): Method {
  if (observations.length < 2) return { kind: "last-observed" };
  const first = observations[0]!;
  const last = observations[observations.length - 1]!;
  const years = last.fiscalYear - first.fiscalYear;

  switch (kind) {
    case "last-observed":
      return { kind: "last-observed" };
    case "cagr":
      return { kind: "cagr", rate: compoundRate(first.value, last.value, years) };
    case "damped":
      return {
        kind: "damped",
        rate: compoundRate(first.value, last.value, years),
        damping,
      };
    case "linear": {
      const [slope, intercept] = leastSquares(observations);
      return { kind: "linear", slope, intercept };
    }
  }
}

/** Carry `base` forward `horizon` years under a fitted method. */
export function advance(base: number, method: Method, horizon: number): number {
  switch (method.kind) {
    case "last-observed":
      return base;
    case "cagr":
      return base * Math.pow(1 + method.rate, horizon);
    case "damped": {
      // Per year, not once: the rate decays as it is applied. Enrollment trends do not persist,
      // and undamped extrapolation is the standard way to produce a confident absurd number.
      let value = base;
      let step = method.rate;
      for (let i = 0; i < horizon; i++) {
        value *= 1 + step;
        step *= method.damping;
      }
      return value;
    }
    case "linear":
      return method.intercept + method.slope * horizon;
  }
}

/**
 * Fit a method to a series and carry it forward to `through`.
 *
 * Observed years come back unchanged and without an interval; years past the last observation
 * carry one. An empty series projects nothing rather than zero.
 */
export function projectSeries(
  observations: Observation[],
  through: number,
  kind: MethodKind,
  damping: number,
  prior: Prior,
): Projected[] {
  const out: Projected[] = observations.map((o) => ({
    fiscalYear: o.fiscalYear,
    point: o.value,
    low: o.value,
    high: o.value,
    observed: true,
  }));
  const last = observations[observations.length - 1];
  if (!last) return out;

  const fitted = fit(observations, kind, damping);
  for (let year = last.fiscalYear + 1; year <= through; year++) {
    const horizon = year - last.fiscalYear;
    const point = advance(last.value, fitted, horizon);
    const half = spread(prior, horizon);
    out.push({
      fiscalYear: year,
      point,
      // A multiplicative band: growth uncertainty scales the level, so a large district and a
      // small one get the same *proportional* interval, which is the claim being made. An
      // additive band would imply the opposite.
      low: point * Math.exp(-half),
      high: point * Math.exp(half),
      observed: false,
    });
  }
  return out;
}

/** A district's three enrolled-ADM observations, dated. */
export function observations(d: District, baseYear: number): Observation[] {
  const n = d.adm_history.length;
  return d.adm_history.map((value, i) => ({
    fiscalYear: baseYear - (n - 1) + i,
    value,
  }));
}

/**
 * Cross-sectional standard deviation of annual enrollment growth across the panel.
 *
 * Districts with a partial history are skipped rather than treated as having grown from zero.
 * Recomputed here rather than read from the feed's `sigma`: if the two disagree the forecast
 * checkpoints will not reproduce, which is a louder failure than a quietly different band.
 */
export function growthPrior(districts: District[], z: number): Prior {
  const rates: number[] = [];
  for (const d of districts) {
    const first = d.adm_history[0]!;
    const last = d.adm_history[d.adm_history.length - 1]!;
    if (first > 0 && last > 0) rates.push(Math.pow(last / first, 0.5) - 1);
  }
  return { sigma: standardDeviation(rates), z };
}

/** The statewide result of running a policy at projected enrollment. */
export interface EnrollmentEffect {
  fiscalYear: number;
  /** Total realized aid at the central enrollment estimate. */
  realizedAid: number;
  /** Total realized aid at the low end of the enrollment band. */
  low: number;
  /** Total realized aid at the high end. */
  high: number;
  adm: number;
  onGuarantee: number;
  /** True for the anchor year, where enrollment is published rather than projected. */
  observed: boolean;
}

/**
 * Run a policy at enrollment projected to `through`.
 *
 * The interval is built by re-running the whole formula at each end of every district's
 * enrollment band rather than by scaling the central answer. That matters because the guarantee
 * is a `max`: a district can be on formula at high enrollment and on the guarantee at low
 * enrollment, and the aid curve has a kink there that no scaling reproduces.
 */
export function forecast(
  districts: District[],
  policy: Policy,
  through: number,
  baseYear: number,
  kind: MethodKind,
  damping: number,
  prior: Prior,
  modelMinimumStateShare: number,
): EnrollmentEffect {
  let point = 0;
  let low = 0;
  let high = 0;
  let adm = 0;
  let onGuarantee = 0;

  for (const d of districts) {
    const series = projectSeries(
      observations(d, baseYear),
      through,
      kind,
      damping,
      prior,
    );
    const projected = series.find((p) => p.fiscalYear === through && !p.observed);
    if (!projected) continue;
    const at = (value: number) => apply(d, policy, value, modelMinimumStateShare);
    const central = at(projected.point);
    point += central.realizedAid;
    // A smaller district draws less aid, so the enrollment band's low end is the aid band's low
    // end. That holds because every lever here is monotone in ADM.
    low += at(projected.low).realizedAid;
    high += at(projected.high).realizedAid;
    adm += projected.point;
    if (central.onGuarantee) onGuarantee++;
  }

  return { fiscalYear: through, realizedAid: point, low, high, adm, onGuarantee, observed: false };
}

/**
 * The whole path: every observed enrollment year, then the forecast out to `through`.
 *
 * # Why the observed years are on it
 *
 * A band that begins at a point and opens rightward, with nothing to its left, asks the reader
 * to take the starting value on faith and gives them no way to judge the trend it was fitted
 * from. The three observed years are the evidence for the forecast, and they are exact — the
 * formula is run at published enrolled ADM, so they carry no band at all.
 *
 * That makes the seam between measurement and forecast the most important thing on the chart,
 * and it is drawn as one: solid line and no band to the left of it, dashed line inside a band to
 * the right. See {@link ../lib/chart.ts | fanChart}.
 *
 * Only enrollment varies across the whole path. The formula is the same FY2027 one at every
 * point, so the x axis is the enrollment year rather than a year of policy — nothing here
 * forecasts the formula, the appropriation, or valuation.
 */
export function forecastPath(
  districts: District[],
  policy: Policy,
  through: number,
  baseYear: number,
  kind: MethodKind,
  damping: number,
  prior: Prior,
  modelMinimumStateShare: number,
): EnrollmentEffect[] {
  const path: EnrollmentEffect[] = [];

  const years = districts[0]?.adm_history.length ?? 0;
  for (let index = 0; index < years; index++) {
    let aid = 0;
    let adm = 0;
    let onGuarantee = 0;
    for (const d of districts) {
      const value = d.adm_history[index] ?? 0;
      const outcome = apply(d, policy, value, modelMinimumStateShare);
      aid += outcome.realizedAid;
      adm += value;
      if (outcome.onGuarantee) onGuarantee++;
    }
    path.push({
      fiscalYear: baseYear - (years - 1) + index,
      realizedAid: aid,
      // An observed year has no interval. Not a zero-width one — none: the enrollment it was
      // computed at is published, so there is nothing to be uncertain about.
      low: aid,
      high: aid,
      adm,
      onGuarantee,
      observed: true,
    });
  }

  for (let year = baseYear + 1; year <= through; year++) {
    path.push(
      forecast(
        districts,
        policy,
        year,
        baseYear,
        kind,
        damping,
        prior,
        modelMinimumStateShare,
      ),
    );
  }
  return path;
}
