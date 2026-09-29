//! R.C. 3317.017 run against the department's published answer for all 609 districts.
//!
//! This crate was written from the statute and its unit tests prove the arithmetic matches its
//! own doc comments. What none of them could prove is that it describes what Ohio does — the
//! corpus recorded the calculator as unimplementable for eleven phases, on the belief that no
//! income data was held anywhere.
//!
//! It was held all along, on two sheets of the FY2027 workbook nothing read. So the measure can
//! now be run and set beside `[b1] Per Pupil Capacity Amount`, which is the department computing
//! the same thing from the same inputs. Agreement is a real check on both.
//!
//! Cited by `corpus/formula-component/fsfp-local-capacity-measure.yml`.

use local_capacity::{benchmark_ratio, local_capacity, statewide_median_income, CapacityInputs};
use project::panel::{panel, DistrictRecord};

/// The statute's arguments, every one of them read rather than reconstructed.
///
/// Reading them is the right habit and the reason given for it was wrong. Both the statewide
/// median income and the benchmark ratio *are* published on the `Local_Capacity` sheet, and both
/// are also exactly derivable from the 609 district medians beside them —
/// [`both_statewide_constants_are_the_panel_reading_itself`] derives them. The earlier note here
/// said a reconstruction gave 41,502 and 1.4151 against a published 54,546.64 and 1.46504, and
/// called the benchmark "a *discretionary* number ... there is no reason a reconstruction should
/// recover it". Neither half holds: the reconstruction was run on the *Ohio* median income column
/// rather than the federal one, which is the same substitution this file's header records as the
/// 4.4% error, and the benchmark is a **rank** — R.C. 3317.017(A)(4)(c) says so in as many words.
/// A wrong input was read as evidence that the quantity was underivable.
fn inputs(record: &DistrictRecord) -> Option<CapacityInputs> {
    Some(CapacityInputs {
        valuation_recent: record.valuation_three_year[0],
        valuation_three_year: record.valuation_three_year,
        agi_recent: record.agi_three_year[0],
        agi_three_year: record.agi_three_year,
        federal_median_income: record.median_income?,
        tax_returns: record.tax_returns?,
        statewide_median_income: record.statewide_median_income?,
        benchmark_ratio: record.benchmark_ratio?,
        base_cost_enrolled_adm: record.base_cost_adm(),
    })
}

/// The calculator reproduces the department's published capacity exactly, for every district.
///
/// Exact is the right standard here and it is met: 609 of 609, worst residual **6.364e-6** —
/// six ten-thousandths of a percent, comfortably inside the workbook's own rounding. The median
/// district agrees to within 1e-10 and 608 of the 609 sit inside 1e-6.
///
/// An earlier version of this test allowed a 5% band, because the first attempt reconstructed the
/// statewide median income and the benchmark ratio instead of reading them and came out 4.4%
/// light. The band was not tolerance for a hard problem; it was tolerance for four wrong inputs.
/// Reading them makes the residual vanish, and a test that had kept the band would have gone on
/// passing while concealing that.
///
/// It did keep the band. The inputs were fixed and this prose was written, but the assertions
/// underneath — a median window of `0.95..=1.00`, 70% within 5%, 97% within 10% — were left as
/// they were, so for an era the doc comment and the test disagreed and the doc was the accurate
/// one. The bound is now the measured residual, and it was checked by breaking it: a 2e-5
/// perturbation fails, where the band would have passed anything short of a 5% error.
#[test]
fn the_statutory_blend_reproduces_the_departments_capacity() {
    let records = panel();

    let mut ratios: Vec<f64> = Vec::new();
    for record in &records {
        let (Some(published), Some(input)) = (record.published_capacity_per_pupil, inputs(record))
        else {
            continue;
        };
        if published <= 0.0 {
            continue;
        }
        let computed = local_capacity(&input).expect("positive ADM and statewide income");
        ratios.push(computed.per_pupil / published);
    }

    assert_eq!(
        ratios.len(),
        records.len(),
        "every district in the panel carries a published capacity to check against; {} of {} \
         resolved, so either an input went missing or the fixture changed shape",
        ratios.len(),
        records.len()
    );

    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let worst = ratios.iter().map(|r| (r - 1.0).abs()).fold(0.0, f64::max);

    // Measured across all 609 districts: worst 6.364e-6, median within 1e-10, 608 of 609
    // inside 1e-6. The bound is set an order of magnitude above the worst so ordinary
    // floating-point drift does not fail it, and three orders below the 5% band this
    // assertion replaced — which had gone on passing for an era whose inputs were wrong.
    assert!(
        worst < 1e-5,
        "the computed capacity differs from the published one by {worst:.3e} at worst. \
         Exact is the standard here and it was met at 6.364e-6; a residual this size means an \
         input vintage changed or the benchmark stopped being read rather than reconstructed. \
         Widening this bound would conceal that, which is what the 5% band it replaced did."
    );
}

/// The sliding scale is real: the rate rises with income and stops at the statutory cap.
///
/// The part of the measure that is easiest to state and hardest to believe without checking. A
/// district at half the state's median income is charged at roughly half the rate of one at the
/// benchmark, so the measure is progressive in rate as well as in base.
#[test]
fn the_capacity_rate_rises_with_income_and_stops_at_two_and_a_half_percent() {
    let records = panel();
    let benchmark = records
        .iter()
        .find_map(|r| r.benchmark_ratio)
        .expect("the sheet publishes it");

    let mut by_income: Vec<(f64, f64)> = records
        .iter()
        .filter_map(|r| {
            let input = inputs(r)?;
            let result = local_capacity(&input).ok()?;
            Some((result.income_ratio, result.capacity_rate))
        })
        .collect();
    by_income.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let poorest = by_income[0];
    let richest = by_income[by_income.len() - 1];
    assert!(
        poorest.1 < richest.1,
        "the rate should rise with income: {poorest:?} against {richest:?}"
    );
    assert!(
        richest.1 <= local_capacity::RATE_AT_BENCHMARK + 1e-9,
        "nothing may exceed the statutory cap of {}",
        local_capacity::RATE_AT_BENCHMARK
    );

    // Every district at or above the benchmark sits exactly at the cap, which is what makes it a
    // cap rather than a trend.
    let at_cap = by_income
        .iter()
        .filter(|(ratio, _)| *ratio >= benchmark)
        .all(|(_, rate)| (*rate - local_capacity::RATE_AT_BENCHMARK).abs() < 1e-9);
    assert!(
        at_cap,
        "districts above the benchmark should all be at the cap"
    );

    // And a district at the state median is charged the scale's value there.
    let at_median = by_income
        .iter()
        .min_by(|a, b| (a.0 - 1.0).abs().partial_cmp(&(b.0 - 1.0).abs()).unwrap())
        .expect("a panel");
    assert!(
        (at_median.1 - local_capacity::RATE_AT_STATE_MEDIAN).abs() < 0.0005,
        "a district at the state median should pay {}, not {}",
        local_capacity::RATE_AT_STATE_MEDIAN,
        at_median.1
    );
}

/// **Which branch of the statutory minimum each district lands on, and why the split is lopsided.**
///
/// R.C. 3317.017(A)(1)(a) takes the lesser of a district's three-year average valuation and its
/// most recent taxable value. Which one wins is decided by direction: under rising valuations the
/// average is lower and the district is charged on a lagged figure; under falling valuations the
/// most recent is lower and the fall is taken at once.
///
/// **602 of 609 districts are on the lagged branch.** The seven on the other are the ones whose
/// valuation fell, and none of them fell far — the largest drop is 1.15%, Eastwood Local.
///
/// The rule therefore never charges a district on the higher of its two measures. It is a floor on
/// the state's share dressed as a smoothing device, and the asymmetry is invisible in any single
/// district's figures.
#[test]
fn the_statutory_minimum_lags_increases_and_takes_decreases_at_once() {
    let panel = project::panel::panel();
    let (mut lagged, mut immediate) = (0_usize, 0_usize);
    let mut deepest_fall = 1.0_f64;

    for record in &panel {
        let years = record.valuation_three_year;
        if years.iter().any(|v| *v <= 0.0) {
            continue;
        }
        let average = years.iter().sum::<f64>() / 3.0;
        let recent = years[0];
        if average < recent {
            lagged += 1;
        } else {
            immediate += 1;
            deepest_fall = deepest_fall.min(recent / average);
        }
    }

    assert_eq!(lagged, 602);
    assert_eq!(immediate, 7);
    assert_eq!(lagged + immediate, 609);
    // No district is exactly on the boundary, so the branch is always decided rather than tied.
    assert!(
        (0.98..0.999).contains(&deepest_fall),
        "deepest fall {deepest_fall:.4}"
    );
}

/// The interpolated middle of the scale, against the department's own published rate.
///
/// The cap and the floor were both checked against real districts; the sliding part between
/// them was checked only for monotonicity. A wrong slope would have left every one of these
/// districts wrong by a little and nothing would have failed.
#[test]
fn the_interpolated_rate_matches_the_published_one_district_by_district() {
    let records = panel();
    let benchmark = records
        .iter()
        .find_map(|r| r.benchmark_ratio)
        .expect("the sheet publishes it");

    let mut checked = 0usize;
    let mut worst = 0.0f64;
    for record in &records {
        let (Some(published), Some(input)) = (record.published_capacity_rate, inputs(record))
        else {
            continue;
        };
        let result = local_capacity(&input).expect("positive ADM and statewide income");

        // Only the interpolated branch: strictly between the state median and the benchmark.
        if result.income_ratio <= 1.0 || result.income_ratio >= benchmark {
            continue;
        }
        checked += 1;
        worst = worst.max((result.capacity_rate - published).abs());
    }

    assert!(
        checked > 0,
        "no district sits strictly between the state median and the benchmark, so this test \
         would pass without exercising the branch it exists for"
    );
    assert!(
        worst < 1e-6,
        "{checked} districts sit on the interpolated part of the scale and the worst differs \
         from the department's published rate by {worst:.3e}. The slope of that interpolation \
         is what this pins; nothing else in the workspace asserts a value on it."
    );
}

/// **Neither statewide constant is handed down; both are the panel reading itself.**
///
/// The two arguments this crate takes as given — `[I7]`, the statewide median income that is the
/// income ratio's denominator, and `[C5]`, the benchmark ratio that tops out the sliding scale —
/// were recorded here and in `project::panel::record` as published rather than derivable, the
/// benchmark as outright "discretionary". R.C. 3317.017(A)(4) defines both as functions of the
/// district medians the same fixture carries:
///
/// - **(A)(4)(a)** — "the median of the median federal adjusted gross incomes determined for all
///   districts statewide". The median of the 609 district medians is **$54,546.6375**, which is
///   `[I7]` to the last digit the fixture holds. Not close: equal.
/// - **(A)(4)(c)–(d)** — rank the districts by their ratios and take the fortieth highest. That is
///   **Westlake City** at **1.46503636**, which is `[C5]`.
///
/// So the whole measure is computable from one column, and nothing in it is a number somebody
/// chose. The claim that it was rested on a single reconstruction run against the *Ohio* median
/// income rather than the federal one — the same substitution that made this file's first
/// attempt 4.4% light, counted twice because its second consequence was read as a fact about the
/// statute rather than as the first consequence again.
///
/// Pinning it matters beyond the correction: a derivable benchmark moves when the income
/// distribution moves, with nobody voting, which is a different kind of parameter from a rate
/// somebody sets. It is also the check that the panel's income column is the one the department
/// ranked, since a fixture holding any other column could not produce these two numbers.
#[test]
fn both_statewide_constants_are_the_panel_reading_itself() {
    let records = panel();

    let medians: Vec<f64> = records.iter().filter_map(|r| r.median_income).collect();
    assert_eq!(
        medians.len(),
        609,
        "every district carries a median income; the statute's (A)(4)(a) is a median over all of \
         them and a partial column would silently give a different one"
    );

    // 609 is odd, so the statutory median is a district's own figure rather than an interpolation
    // between two — which is why this reproduces exactly rather than approximately.
    let derived = statewide_median_income(&medians).expect("609 districts");
    let published = records
        .iter()
        .find_map(|r| r.statewide_median_income)
        .expect("the sheet publishes it");
    assert!(
        (derived - published).abs() < 1e-9,
        "R.C. 3317.017(A)(4)(a) makes the denominator the median of the district medians. \
         Derived {derived:.4}, published {published:.4}. These agreed bit for bit when this was \
         written; a difference means the panel's income column is no longer the one the \
         department ranked."
    );
    assert!(
        (derived - 54_546.637_5).abs() < 1e-4,
        "and in absolute terms it is $54,546.6375, not the $41,502 an earlier pass reported \
         from the Ohio median column; derived {derived:.4}"
    );

    let fortieth = benchmark_ratio(&medians, published).expect("609 districts and a denominator");
    let benchmark = records
        .iter()
        .find_map(|r| r.benchmark_ratio)
        .expect("the sheet publishes it");
    assert!(
        (fortieth - benchmark).abs() < 1e-8,
        "R.C. 3317.017(A)(4)(c)-(d) makes the benchmark the fortieth highest ratio. Derived \
         {fortieth:.8}, published {benchmark:.8}. The residual when this was written was 4.5e-9, \
         which is the fixture storing the published figure to eight places."
    );

    // The rank is a rank: the districts on either side of the fortieth are not tied with it, so
    // the benchmark is one district's ratio rather than a plateau that any of several would give.
    let mut ratios: Vec<f64> = medians.iter().map(|m| m / published).collect();
    ratios.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert!(
        ratios[38] > fortieth && fortieth > ratios[40],
        "the fortieth highest is strictly between its neighbours at {:.8} and {:.8}",
        ratios[38],
        ratios[40]
    );
}
