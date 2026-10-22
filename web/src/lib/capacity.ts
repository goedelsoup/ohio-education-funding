/**
 * R.C. 3317.017's local capacity, split into the three terms it is a blend of (#791).
 *
 * # What is computed here, and what is not
 *
 * Nothing about the formula. The feed carries each weighted term already multiplied by the
 * capacity rate (`crates/bundle::build::capacity_of`), so the weights and the rate rule live in
 * `crates/local-capacity` alone. This module does two things with what arrives:
 *
 * - it **checks** that the three terms add to the feed's total, and that the total is the
 *   department's published `[b1] Per Pupil Capacity Amount` (`regime.local_capacity`), which
 *   reaches the feed by a different path, read off the calculator rather than computed;
 * - it **recovers** each weight as `term / (base × rate)`, so the page can say "60%" without a
 *   60 written anywhere in the web layer.
 *
 * The first is the reason the table may be drawn at all. A decomposition whose parts do not add
 * to the figure the page already prints two cards above would be a second, unexplained number for
 * the same quantity. So a failure throws, and the build stops on the district that failed.
 */

import type { District } from "./types.ts";

/**
 * How far the three rated terms may miss their own total, in dollars per pupil.
 *
 * Each is written to the feed at four places, so three roundings can move the sum by 1.5e-4 at
 * most. Measured across all 609 districts: 1.0e-4. Ten times that is room for the serializer and
 * nothing else; a dropped term or a wrong weight is dollars out.
 */
export const CAPACITY_CLOSURE = 1e-3;

/**
 * How far the total may sit from the department's published capacity, as a share of it.
 *
 * The bound `xcheck/tests/against_the_departments_own_capacity.rs` holds the crate to, for the
 * same reason: the worst of the 609 is 6.364e-6, and the bound is an order of magnitude above it.
 */
export const CAPACITY_AGREEMENT = 1e-5;

/** One of the three terms the capacity is a blend of. */
export interface CapacityTerm {
  /** The measure before weighting, per base-cost pupil. */
  base: number;
  /**
   * The same measure in the year before's model, or `null` where that model has none. Shown, not
   * checked: the closure above is this year's, and the year before's rate is not in the feed.
   */
  prior: number | null;
  /** Its weight in the blend, recovered from the feed rather than typed. */
  weight: number;
  /** The weighted measure times the capacity rate, per base-cost pupil. */
  term: number;
  /** That term over the total: how much of the district's capacity this measure is. */
  share: number;
}

/** A district's capacity, term by term, once it has been found to close. */
export interface CapacityDecomposition {
  valuation: CapacityTerm;
  grossIncome: CapacityTerm;
  medianIncome: CapacityTerm;
  incomeRatio: number;
  rate: number;
  /** The three terms' sum, as the feed states it. */
  total: number;
  /** The department's own figure, which `total` was checked against. */
  published: number;
}

/**
 * The decomposition for one district, or `null` where the feed carries no capacity for it.
 *
 * @throws If the terms do not add to the total, or the total is not the published figure. Both
 *   are defects in the feed, not facts about the district, and the page does not render one.
 */
export function decomposeCapacity(
  d: Pick<District, "irn" | "capacity" | "regime">,
): CapacityDecomposition | null {
  const c = d.capacity;
  const published = d.regime?.local_capacity;
  if (!c || published == null) return null;

  const sum = c.valuation_term_per_pupil + c.gross_income_term_per_pupil + c.median_income_term_per_pupil;
  if (!(Math.abs(sum - c.per_pupil) <= CAPACITY_CLOSURE)) {
    throw new Error(
      `District ${d.irn}: the three capacity terms add to ${sum}, not the ${c.per_pupil} stated as their total.`,
    );
  }
  if (!(Math.abs(c.per_pupil - published) <= CAPACITY_AGREEMENT * Math.abs(published))) {
    throw new Error(
      `District ${d.irn}: capacity of ${c.per_pupil} per pupil is not the department's published ${published}.`,
    );
  }

  const term = (base: number, rated: number, prior: number | null): CapacityTerm => ({
    base,
    prior,
    // To the whole percent: the statute's weights are whole percents, and the quotient carries
    // the feed's rounding in its seventh place.
    weight: Math.round((rated / (base * c.capacity_rate)) * 100) / 100,
    term: rated,
    share: rated / c.per_pupil,
  });
  const p = c.prior;
  const valuation = term(c.valuation_per_pupil, c.valuation_term_per_pupil, p?.valuation_per_pupil ?? null);
  const grossIncome = term(
    c.gross_income_per_pupil,
    c.gross_income_term_per_pupil,
    p?.gross_income_per_pupil ?? null,
  );
  const medianIncome = term(
    c.median_income_per_pupil,
    c.median_income_term_per_pupil,
    p?.median_income_per_pupil ?? null,
  );
  const weights = valuation.weight + grossIncome.weight + medianIncome.weight;
  if (!(Math.abs(weights - 1) < 1e-9)) {
    throw new Error(`District ${d.irn}: the capacity weights recovered from the feed add to ${weights}, not 1.`);
  }

  return {
    valuation,
    grossIncome,
    medianIncome,
    incomeRatio: c.income_ratio,
    rate: c.capacity_rate,
    total: c.per_pupil,
    published,
  };
}
