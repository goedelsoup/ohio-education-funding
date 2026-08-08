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

use local_capacity::{local_capacity, CapacityInputs};
use project::panel::{panel, DistrictRecord};

/// The statute's arguments, every one of them read rather than reconstructed.
///
/// Both the statewide median income and the benchmark ratio are published on the
/// `Local_Capacity` sheet. Deriving them instead — the median of district medians, the 40th
/// highest reconstructed ratio — gives 41,502 and 1.4151 against a published 54,546.64 and
/// 1.46504, and leaves the whole calculation about 4% light. The benchmark in particular is a
/// *discretionary* number that sets the top of the scale; there is no reason a reconstruction
/// should recover it, and it did not.
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
/// Exact is the right standard here and it is met: 609 of 609 on both the per-pupil amount and
/// the rate, to within a hundredth of a percent, which is the workbook's own rounding.
///
/// An earlier version of this test allowed a 5% band, because the first attempt reconstructed the
/// statewide median income and the benchmark ratio instead of reading them and came out 4.4%
/// light. The band was not tolerance for a hard problem; it was tolerance for four wrong inputs.
/// Reading them makes the residual vanish, and a test that had kept the band would have gone on
/// passing while concealing that.
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

    assert!(
        ratios.len() > 590,
        "expected nearly every district: {}",
        ratios.len()
    );
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = ratios[ratios.len() / 2];

    assert!(
        (0.95..=1.00).contains(&median),
        "the computed capacity is a median {median:.4} of the published one. Outside this band \
         the systematic bias has moved, which means an input vintage changed or the benchmark \
         reconstruction stopped matching — either is worth looking at rather than widening"
    );

    let within_five = ratios.iter().filter(|r| (0.95..=1.05).contains(*r)).count();
    assert!(
        within_five * 10 >= ratios.len() * 7,
        "only {within_five} of {} districts land within five percent",
        ratios.len()
    );
    let within_ten = ratios.iter().filter(|r| (0.90..=1.10).contains(*r)).count();
    assert!(
        within_ten * 100 >= ratios.len() * 97,
        "only {within_ten} of {} districts land within ten percent",
        ratios.len()
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
