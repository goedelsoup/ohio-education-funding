//! Fair School Funding Plan local capacity and state share, per R.C. 3317.017.
//!
//! This is the component that replaced the charge-off and the one that most changes which
//! districts win and lose. Where the charge-off asked what a fixed millage would raise against
//! assessed valuation, local capacity asks what a district's community can bear — blending
//! property wealth with two different income measures and scaling the result by a rate that
//! is itself progressive.
//!
//! # The blend
//!
//! ```text
//! local capacity per pupil = ((valuation/pupil × 0.6)
//!                           + (gross income/pupil × 0.2)
//!                           + (median income/pupil × 0.2)) × capacity rate
//! ```
//!
//! Splitting income into an aggregate term (total federal AGI, which rises with population and
//! with concentration at the top) and a median term (which does not) is what lets the measure
//! see a district whose totals look healthy because a few residents earn a great deal.
//!
//! # The rate is a sliding scale, not a constant
//!
//! The scaling rate is set by the ratio of the district's federal median income to the state's,
//! benchmarked against the **40th highest district** on that ratio. At or above the benchmark
//! the rate is 2.5%; at or below the state median it is the district's own ratio times 2.25%;
//! between the two it interpolates. A district at half the state median income is charged at
//! roughly 1.125%, a wealthy one at 2.5% — so the measure is progressive in rate as well as in
//! base.
//!
//! # Both wealth terms take the lesser of recent and three-year-average
//!
//! Deliberate protection: a district whose valuation spikes on a single reappraisal is not
//! immediately charged for it.
//!
//! Not modelled here: how the 40th-highest-district benchmark moves over time. It is one
//! discretionary number that sets the top of the entire scale.

#![forbid(unsafe_code)]

use edfund_core::{Adm, Dollars};

/// Property valuation and income inputs for one district, as the department compiles them.
///
/// Tax years are offset from fiscal years and the offsets differ between the valuation and
/// income terms — in the FY2022 calculation, valuation used TY2020 against a TY2018-2020
/// average while income used TY2019 against TY2017-2019. Callers are responsible for supplying
/// correctly-offset values; this crate does not know the calendar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapacityInputs {
    /// Assessed valuation for the most recent tax year available.
    pub valuation_recent: Dollars,
    /// Assessed valuation for the three most recent tax years, used as an average.
    pub valuation_three_year: [Dollars; 3],
    /// Federal adjusted gross income of district residents, most recent tax year.
    pub agi_recent: Dollars,
    /// Federal adjusted gross income for the three most recent tax years.
    pub agi_three_year: [Dollars; 3],
    /// District federal median income for the income reference year, **with the department's
    /// adjustment factor already applied**.
    ///
    /// The workbook's `[I5] TY23 Federal Median Income with ADJ Factor`. Federal, as the name has
    /// always said: an earlier pass here doubted that, on the strength of the district's Ohio
    /// median income fitting the published capacity to within a few percent. It fits because the
    /// two are correlated, not because it is the term — Columbus's federal median is $46,395
    /// against an Ohio median of $31,555, and substituting one for the other leaves the whole
    /// blend about 4% light. Fed the real figure, this crate reproduces the department's own
    /// capacity for all 609 districts exactly — worst residual 6.364e-6, pinned by
    /// `tests/against_the_departments_own_capacity.rs`.
    pub federal_median_income: Dollars,
    /// Number of tax returns filed in the district for the income reference year.
    pub tax_returns: f64,
    /// Statewide federal median income for the same year.
    pub statewide_median_income: Dollars,
    /// The income ratio of the 40th highest district — the benchmark that sets the top of the
    /// capacity rate scale.
    pub benchmark_ratio: f64,
    /// Base cost enrolled ADM, the denominator for every per-pupil term.
    pub base_cost_enrolled_adm: Adm,
}

/// The intermediate and final values of the local capacity calculation.
///
/// Intermediates are exposed because the department publishes them line by line, which is what
/// makes this calculation auditable against the payment report.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CapacityResult {
    /// Capacity valuation: the lesser of the recent year and the three-year average.
    pub capacity_valuation: Dollars,
    /// Capacity gross income: the lesser of the recent year and the three-year average.
    pub capacity_gross_income: Dollars,
    /// C1 — assessed valuation per pupil.
    pub valuation_per_pupil: Dollars,
    /// C2 — gross income per pupil.
    pub gross_income_per_pupil: Dollars,
    /// C3 — median income per pupil.
    pub median_income_per_pupil: Dollars,
    /// C4 — district median income as a ratio of the statewide figure.
    pub income_ratio: f64,
    /// C6 — the capacity rate produced by the sliding scale.
    pub capacity_rate: f64,
    /// C7 — local capacity per pupil.
    pub per_pupil: Dollars,
}

/// Why a capacity calculation could not be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityError {
    /// Base cost enrolled ADM is zero or negative; every per-pupil term would be undefined.
    NonPositiveAdm,
    /// Statewide median income is zero or negative, making the income ratio undefined.
    NonPositiveStatewideIncome,
    /// The benchmark ratio is at or below 1.0, which would divide by zero in the interpolated
    /// branch of the sliding scale.
    InvalidBenchmark,
}

impl core::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::NonPositiveAdm => "base cost enrolled ADM must be positive",
            Self::NonPositiveStatewideIncome => "statewide median income must be positive",
            Self::InvalidBenchmark => "benchmark income ratio must exceed 1.0",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for CapacityError {}

/// Weight on assessed property valuation in the capacity blend.
pub const VALUATION_WEIGHT: f64 = 0.6;
/// Weight on aggregate federal adjusted gross income.
pub const GROSS_INCOME_WEIGHT: f64 = 0.2;
/// Weight on federal median income.
pub const MEDIAN_INCOME_WEIGHT: f64 = 0.2;

/// Capacity rate applied at or above the 40th-highest-district benchmark.
pub const RATE_AT_BENCHMARK: f64 = 0.025;
/// Capacity rate coefficient applied at or below the statewide median income.
pub const RATE_AT_STATE_MEDIAN: f64 = 0.0225;

/// The minimum share of base cost the state pays, however wealthy the district.
///
/// Defined in [`edfund_core`] and re-exported here. It moved because `project` reads the
/// FY2027 figure as well, and importing this crate for one `f64` made a 5,000-line crate
/// depend on a calculator it never calls.
///
/// This floor is why [`state_share`] stops discriminating at the top of the wealth
/// distribution: a district whose local capacity exceeds 95% of its base cost and one whose
/// capacity exceeds it several times over both report the same share.
///
/// # It is not a constant, and treating it as one was wrong here
///
/// The percentage is set by each biennial budget, not fixed in permanent law, and it has
/// **doubled** between the two figures named here. This crate once hard-coded the FY2022
/// value and applied it to every year, which concealed that. Callers pass the year's value
/// to [`state_share`]; these constants name the two that are known.
pub use edfund_core::{MINIMUM_STATE_SHARE_FY2022, MINIMUM_STATE_SHARE_FY2027};

/// The lesser of a district's most recent year and its three-year average.
///
/// **R.C. 3317.017(A)(1)(a)**: "Determine the minimum of the district's three-year average
/// valuation for the fiscal year for which the calculation is made and the district's taxable
/// value for the most recent tax year for which data is available." The income side of the blend
/// takes the same shape.
///
/// This was inferred from the department's `Local_Capacity` sheet and carried without a citation
/// until `ohio-laws` was wired. It is the statute, and the statute is worth reading for the
/// asymmetry the arithmetic hides.
///
/// # It is a one-sided smoother, and which side depends on the direction
///
/// Under **rising** valuations the three-year average is the lower of the two, so the district is
/// charged on a lagged figure and the state pays more for as long as the lag runs. Under
/// **falling** valuations the most recent year is lower, so the fall is taken immediately and the
/// state pays more at once. The rule never charges a district on the higher of the two measures,
/// which makes it a floor on the state's share rather than a smoother in the usual sense: it
/// dampens increases and passes decreases straight through.
///
/// In the FY2027 model the split is **602 districts on the lagged branch and 7 on the immediate
/// one** — the seven whose valuation actually fell. See
/// `tests/against_the_departments_own_capacity.rs`.
///
/// # And the Fair School Funding Plan is doing what the charge-off did, by another device
///
/// The mechanism this replaced lagged reappraisal growth too, through
/// [recognized valuation](../../regime-diff/src/recognized_valuation.rs) — a three-year phase-in
/// of the inflationary increase. Two regimes, the same instinct about revaluation shocks, and two
/// different devices: the charge-off phased the increase in, the plan takes the minimum of an
/// average and the current. Worth holding beside each other, because a reader told only that the
/// plan "uses a three-year average" would not see that it is the successor to something.
fn lesser_of_recent_and_average(recent: f64, three_year: [f64; 3]) -> f64 {
    let average = three_year.iter().sum::<f64>() / 3.0;
    recent.min(average)
}

/// The sliding capacity rate, given a district's income ratio and the benchmark.
///
/// # Errors
///
/// Returns [`CapacityError::InvalidBenchmark`] if the benchmark does not exceed 1.0.
pub fn capacity_rate(income_ratio: f64, benchmark_ratio: f64) -> Result<f64, CapacityError> {
    if benchmark_ratio <= 1.0 {
        return Err(CapacityError::InvalidBenchmark);
    }
    Ok(if income_ratio >= benchmark_ratio {
        RATE_AT_BENCHMARK
    } else if income_ratio > 1.0 {
        ((income_ratio - 1.0) * (RATE_AT_BENCHMARK - RATE_AT_STATE_MEDIAN))
            / (benchmark_ratio - 1.0)
            + RATE_AT_STATE_MEDIAN
    } else {
        income_ratio * RATE_AT_STATE_MEDIAN
    })
}

/// Compute local capacity per pupil for one district.
///
/// # Errors
///
/// Returns [`CapacityError`] if ADM, statewide median income, or the benchmark are outside
/// their valid ranges.
pub fn local_capacity(inputs: &CapacityInputs) -> Result<CapacityResult, CapacityError> {
    if inputs.base_cost_enrolled_adm <= 0.0 {
        return Err(CapacityError::NonPositiveAdm);
    }
    if inputs.statewide_median_income <= 0.0 {
        return Err(CapacityError::NonPositiveStatewideIncome);
    }

    let capacity_valuation =
        lesser_of_recent_and_average(inputs.valuation_recent, inputs.valuation_three_year);
    let capacity_gross_income =
        lesser_of_recent_and_average(inputs.agi_recent, inputs.agi_three_year);

    let adm = inputs.base_cost_enrolled_adm;
    let valuation_per_pupil = capacity_valuation / adm;
    let gross_income_per_pupil = capacity_gross_income / adm;
    let median_income_per_pupil = (inputs.federal_median_income * inputs.tax_returns) / adm;

    let income_ratio = inputs.federal_median_income / inputs.statewide_median_income;
    let rate = capacity_rate(income_ratio, inputs.benchmark_ratio)?;

    let blended = valuation_per_pupil * VALUATION_WEIGHT
        + gross_income_per_pupil * GROSS_INCOME_WEIGHT
        + median_income_per_pupil * MEDIAN_INCOME_WEIGHT;

    Ok(CapacityResult {
        capacity_valuation,
        capacity_gross_income,
        valuation_per_pupil,
        gross_income_per_pupil,
        median_income_per_pupil,
        income_ratio,
        capacity_rate: rate,
        per_pupil: blended * rate,
    })
}

/// A district's state share of base cost, per R.C. 3317.017(B) and (C).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateShare {
    /// Total state share of base cost, in dollars.
    pub amount: Dollars,
    /// State share as a fraction of base cost per pupil, floored at 5%.
    pub percentage: f64,
    /// Whether the 5% floor was binding — that is, whether this district's computed share
    /// would have been lower without it.
    pub at_minimum: bool,
}

/// Compute the state share of base cost.
///
/// The state pays the difference between base cost per pupil and local capacity per pupil,
/// multiplied by **current-year** enrolled ADM — note that base cost and local capacity are
/// themselves computed from prior-year or three-year-average ADM, so a district with sharply
/// changing enrollment is funded on a blend of two different student counts.
///
/// No district receives less than `minimum_share` of its base cost — a figure each biennial
/// budget sets, and which doubled between FY2022 and FY2026. See
/// [`MINIMUM_STATE_SHARE_FY2022`] and [`MINIMUM_STATE_SHARE_FY2027`].
///
/// # Errors
///
/// Returns [`CapacityError::NonPositiveAdm`] if base cost per pupil is not positive.
pub fn state_share(
    base_cost_per_pupil: Dollars,
    local_capacity_per_pupil: Dollars,
    current_year_enrolled_adm: Adm,
    minimum_share: f64,
) -> Result<StateShare, CapacityError> {
    if base_cost_per_pupil <= 0.0 {
        return Err(CapacityError::NonPositiveAdm);
    }
    let residual = base_cost_per_pupil - local_capacity_per_pupil;
    let floor = base_cost_per_pupil * minimum_share;
    let at_minimum = residual < floor;
    let per_pupil = if at_minimum { floor } else { residual };
    Ok(StateShare {
        amount: per_pupil * current_year_enrolled_adm,
        percentage: per_pupil / base_cost_per_pupil,
        at_minimum,
    })
}

/// State share percentage alone, floored at 5%.
///
/// # Errors
///
/// As [`state_share`].
pub fn state_share_percentage(
    base_cost_per_pupil: Dollars,
    local_capacity_per_pupil: Dollars,
    minimum_share: f64,
) -> Result<f64, CapacityError> {
    Ok(state_share(
        base_cost_per_pupil,
        local_capacity_per_pupil,
        1.0,
        minimum_share,
    )?
    .percentage)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worked example published in the FY2022 School Finance Payment Report line-by-line
    /// explanation, Local Capacity report. Every intermediate is checked against the published
    /// value.
    fn published_fy2022_example() -> CapacityInputs {
        CapacityInputs {
            valuation_recent: 2_821_319_120.0,
            valuation_three_year: [2_821_319_120.0, 2_509_561_730.0, 2_370_737_970.0],
            agi_recent: 3_474_621_434.0,
            agi_three_year: [3_474_621_434.0, 3_316_079_643.0, 3_159_704_391.0],
            federal_median_income: 29_102.0,
            tax_returns: 82_227.0,
            statewide_median_income: 41_784.0,
            benchmark_ratio: 1.521_347_88,
            base_cost_enrolled_adm: 20_736.95,
        }
    }

    #[test]
    fn reproduces_published_capacity_valuation() {
        let r = local_capacity(&published_fy2022_example()).unwrap();
        // V1: the three-year average is lower than TY20 alone, so the average governs.
        assert!((r.capacity_valuation - 2_567_206_273.0).abs() < 1.0);
    }

    #[test]
    fn reproduces_published_capacity_gross_income() {
        let r = local_capacity(&published_fy2022_example()).unwrap();
        assert!((r.capacity_gross_income - 3_316_801_823.0).abs() < 1.0);
    }

    #[test]
    fn reproduces_published_per_pupil_intermediates() {
        let r = local_capacity(&published_fy2022_example()).unwrap();
        assert!(
            (r.valuation_per_pupil - 123_798.64).abs() < 0.01,
            "C1 was {}",
            r.valuation_per_pupil
        );
        assert!(
            (r.gross_income_per_pupil - 159_946.46).abs() < 0.01,
            "C2 was {}",
            r.gross_income_per_pupil
        );
        assert!(
            (r.median_income_per_pupil - 115_396.44).abs() < 0.01,
            "C3 was {}",
            r.median_income_per_pupil
        );
    }

    #[test]
    fn reproduces_published_income_ratio_and_rate() {
        let r = local_capacity(&published_fy2022_example()).unwrap();
        assert!(
            (r.income_ratio - 0.696_486_69).abs() < 1e-8,
            "C4 was {}",
            r.income_ratio
        );
        assert!(
            (r.capacity_rate - 0.015_670_95).abs() < 1e-8,
            "C6 was {}",
            r.capacity_rate
        );
    }

    /// The published bottom line: $2,027.00 of local capacity per pupil.
    #[test]
    fn reproduces_published_local_capacity_per_pupil() {
        let r = local_capacity(&published_fy2022_example()).unwrap();
        assert!(
            (r.per_pupil - 2_027.00).abs() < 0.01,
            "C7 was {}, expected 2027.00",
            r.per_pupil
        );
    }

    #[test]
    fn takes_the_lesser_of_recent_and_three_year_average() {
        // A district whose most recent valuation spikes is protected by the average.
        let spiked = lesser_of_recent_and_average(3_000.0, [3_000.0, 1_000.0, 1_000.0]);
        assert!((spiked - 1_666.666_666).abs() < 1e-5);
        // A district whose valuation falls is charged on the lower recent figure.
        let fallen = lesser_of_recent_and_average(1_000.0, [1_000.0, 3_000.0, 3_000.0]);
        assert!((fallen - 1_000.0).abs() < 1e-9);
    }

    #[test]
    fn capacity_rate_is_progressive_across_the_scale() {
        let benchmark = 1.5;
        let poor = capacity_rate(0.5, benchmark).unwrap();
        let median = capacity_rate(1.0, benchmark).unwrap();
        let rich = capacity_rate(2.0, benchmark).unwrap();
        assert!(
            (poor - 0.011_25).abs() < 1e-9,
            "half the state median pays 1.125%"
        );
        assert!((median - 0.0225).abs() < 1e-9);
        assert!((rich - RATE_AT_BENCHMARK).abs() < 1e-9);
        assert!(poor < median && median < rich);
    }

    #[test]
    fn capacity_rate_is_continuous_at_the_state_median() {
        let benchmark = 1.5;
        let just_below = capacity_rate(0.999_999, benchmark).unwrap();
        let just_above = capacity_rate(1.000_001, benchmark).unwrap();
        assert!(
            (just_below - just_above).abs() < 1e-7,
            "scale must not jump at 1.0"
        );
    }

    #[test]
    fn capacity_rate_caps_at_the_benchmark() {
        let far_above = capacity_rate(50.0, 1.5).unwrap();
        assert!((far_above - RATE_AT_BENCHMARK).abs() < 1e-12);
    }

    #[test]
    fn rejects_a_benchmark_that_does_not_exceed_one() {
        assert_eq!(
            capacity_rate(1.2, 1.0),
            Err(CapacityError::InvalidBenchmark)
        );
        assert_eq!(
            capacity_rate(1.2, 0.8),
            Err(CapacityError::InvalidBenchmark)
        );
    }

    #[test]
    fn rejects_non_positive_adm() {
        let mut inputs = published_fy2022_example();
        inputs.base_cost_enrolled_adm = 0.0;
        assert_eq!(local_capacity(&inputs), Err(CapacityError::NonPositiveAdm));
    }

    #[test]
    fn rejects_non_positive_statewide_income() {
        let mut inputs = published_fy2022_example();
        inputs.statewide_median_income = 0.0;
        assert_eq!(
            local_capacity(&inputs),
            Err(CapacityError::NonPositiveStatewideIncome)
        );
    }

    #[test]
    fn state_share_is_the_residual_after_local_capacity() {
        let share = state_share(7_000.0, 2_000.0, 1_000.0, MINIMUM_STATE_SHARE_FY2022).unwrap();
        assert!((share.amount - 5_000_000.0).abs() < 1e-6);
        assert!((share.percentage - 5.0 / 7.0).abs() < 1e-9);
        assert!(!share.at_minimum);
    }

    /// The floor is the reason state share percentage cannot discriminate among wealthy
    /// districts: two districts with very different capacity report the same 5%.
    #[test]
    fn state_share_floors_at_five_percent_for_wealthy_districts() {
        let merely_rich =
            state_share(7_000.0, 6_900.0, 1_000.0, MINIMUM_STATE_SHARE_FY2022).unwrap();
        let extremely_rich =
            state_share(7_000.0, 30_000.0, 1_000.0, MINIMUM_STATE_SHARE_FY2022).unwrap();
        assert!(merely_rich.at_minimum && extremely_rich.at_minimum);
        assert!((merely_rich.percentage - MINIMUM_STATE_SHARE_FY2022).abs() < 1e-12);
        assert!((extremely_rich.percentage - MINIMUM_STATE_SHARE_FY2022).abs() < 1e-12);
        assert!(
            (merely_rich.amount - extremely_rich.amount).abs() < 1e-9,
            "the floor erases the difference between them"
        );
    }

    #[test]
    fn state_share_percentage_never_falls_below_the_floor() {
        for capacity in [0.0, 1_000.0, 6_650.0, 7_000.0, 100_000.0] {
            let pct =
                state_share_percentage(7_000.0, capacity, MINIMUM_STATE_SHARE_FY2022).unwrap();
            assert!(
                pct >= MINIMUM_STATE_SHARE_FY2022 - 1e-12,
                "capacity {capacity} gave {pct}"
            );
        }
    }

    #[test]
    fn state_share_rejects_non_positive_base_cost() {
        assert_eq!(
            state_share(0.0, 100.0, 10.0, MINIMUM_STATE_SHARE_FY2022),
            Err(CapacityError::NonPositiveAdm)
        );
    }

    /// The middle branch of the scale, which is the one the workspace never checked a value on.
    ///
    /// Every other test of `capacity_rate` exercised the floor, the cap, or continuity at the
    /// two joins. Nothing asserted a rate strictly between them, so the *slope* of the
    /// interpolation was free to be wrong.
    #[test]
    fn the_rate_interpolates_linearly_between_the_median_and_the_benchmark() {
        let benchmark = 1.5;
        let span = RATE_AT_BENCHMARK - RATE_AT_STATE_MEDIAN;

        // Halfway from the state median to the benchmark is halfway up the rate span.
        let midpoint = capacity_rate(1.25, benchmark).unwrap();
        assert!(
            (midpoint - (RATE_AT_STATE_MEDIAN + span / 2.0)).abs() < 1e-12,
            "midpoint rate was {midpoint}"
        );
        assert!(
            (midpoint - 0.023_75).abs() < 1e-12,
            "and in absolute terms, {midpoint}"
        );

        // A quarter and three quarters of the way, to pin linearity rather than one point.
        for (fraction, ratio) in [(0.25, 1.125), (0.75, 1.375)] {
            let rate = capacity_rate(ratio, benchmark).unwrap();
            let expected = RATE_AT_STATE_MEDIAN + span * fraction;
            assert!(
                (rate - expected).abs() < 1e-12,
                "at income ratio {ratio} the rate should be {expected}, got {rate}"
            );
        }

        // The branch meets its neighbours at both ends.
        let just_above_median = capacity_rate(1.000_000_1, benchmark).unwrap();
        assert!((just_above_median - RATE_AT_STATE_MEDIAN).abs() < 1e-8);
        let just_below_benchmark = capacity_rate(benchmark - 1e-9, benchmark).unwrap();
        assert!((just_below_benchmark - RATE_AT_BENCHMARK).abs() < 1e-8);
    }
}

#[cfg(test)]
mod minimum_share_tests {
    use super::*;

    #[test]
    fn the_minimum_state_share_doubled_between_the_two_known_biennia() {
        // Not a refactor for its own sake. The FY2027 calculator's Notes sheet states 0.1 for
        // FY2026 and FY2027; the crate had 0.05 hard-coded from the FY2022 worked example and
        // applied it to every year.
        assert!((MINIMUM_STATE_SHARE_FY2027 - 2.0 * MINIMUM_STATE_SHARE_FY2022).abs() < 1e-12);
    }

    #[test]
    fn the_floor_binds_at_ten_percent_where_it_would_not_at_five() {
        // A district whose local capacity is 93% of base cost: computed share 7%, which clears
        // the FY2022 floor and is lifted by the FY2027 one. This is the band the change moved.
        let (base, capacity) = (10_000.0, 9_300.0);
        let old = state_share(base, capacity, 1.0, MINIMUM_STATE_SHARE_FY2022).unwrap();
        let new = state_share(base, capacity, 1.0, MINIMUM_STATE_SHARE_FY2027).unwrap();
        assert!(!old.at_minimum);
        assert!(new.at_minimum);
        assert!((old.percentage - 0.07).abs() < 1e-12);
        assert!((new.percentage - 0.10).abs() < 1e-12);
    }
}
