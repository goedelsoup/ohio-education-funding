//! The department's enrolled ADM runs four years once both models are read, and the fourth one
//! exists twice under one name.
//!
//! # Why a fourth year was worth having
//!
//! [`project::series::DEFAULT_DAMPING`] is the share of a fitted growth rate a projection carries
//! into each further year. It shipped at 0.85 saying *"a convention, not an estimate. **Three
//! observations per district cannot identify a damping parameter**"*, and was refitted to **0.30**
//! out of sample on the F-33 panel's `V33` fall membership — a different measure over a different
//! population, because the department's own series had three points.
//!
//! It has four. The FY2026 model's `ADM Data` sheet publishes FY2023, FY2024 and FY2025; the
//! FY2027 model's publishes FY2024, FY2025 and FY2026. Two overlapping windows, and the overlap
//! agrees: FY2024 is identical for all 611 districts and FY2025 moves by 4.7 ADM statewide.
//!
//! Four observations give three annual growth rates, and three growth rates give two estimates of
//! how much of a district's growth carries into the next year — which is the quantity the damping
//! parameterises.
//!
//! # And then there is a second FY2023
//!
//! `Base_Enrollment Growth` heads a column `M1B FY23 Enrolled ADM`. It is the enrolment growth
//! supplement's base year and it is **not** the same number: it differs from `ADM Data`'s
//! `[b1] FY23 Enrolled ADM` in 608 of 611 districts, by up to 4.6%. The 162-column panel carries
//! it as `enrolled_adm_fy23`, beside `enrolled_adm_fy24`, `_fy25` and `_fy26`, which come from the
//! other family — so a reader taking those four columns as one series is splicing two bases at the
//! join.
//!
//! The splice is not harmless, and the size of the harm is the point of this file: persistence
//! comes out **+0.33** on the consistent series and **-0.05** on the spliced one, over the same
//! districts and the same years. One corroborates the published constant and the other contradicts
//! it.

use std::collections::BTreeMap;

use project::counts;
use project::panel;
use project::series::DEFAULT_DAMPING;

/// Pearson correlation, each series demeaned against its own year.
fn correlation(pairs: &[(f64, f64)]) -> f64 {
    let n = pairs.len() as f64;
    let mean_x = pairs.iter().map(|(a, _)| a).sum::<f64>() / n;
    let mean_y = pairs.iter().map(|(_, b)| b).sum::<f64>() / n;
    let covariance: f64 = pairs.iter().map(|(a, b)| (a - mean_x) * (b - mean_y)).sum();
    let spread_x: f64 = pairs
        .iter()
        .map(|(a, _)| (a - mean_x).powi(2))
        .sum::<f64>()
        .sqrt();
    let spread_y: f64 = pairs
        .iter()
        .map(|(_, b)| (b - mean_y).powi(2))
        .sum::<f64>()
        .sqrt();
    covariance / (spread_x * spread_y)
}

#[test]
fn the_two_models_publish_four_years_and_agree_where_they_overlap() {
    let history = counts::adm_history();
    assert_eq!(
        history.keys().copied().collect::<Vec<u16>>(),
        vec![2023, 2024, 2025, 2026]
    );
    for year in history.values() {
        assert_eq!(year.len(), 611);
    }

    // FY2024 is the older workbook's [b2] and the newer one's [b1]. Identical, every district.
    let restated = counts::adm_restatement(2024).expect("both models publish FY2024");
    assert_eq!((restated.identical, restated.paired), (611, 611));

    // FY2025 moves, and this is how much: the size of "the most recent observation in a model is
    // a departmental estimate", which the corpus could state and not measure.
    let restated = counts::adm_restatement(2025).expect("both models publish FY2025");
    assert_eq!((restated.identical, restated.paired), (471, 611));
    assert!(
        (restated.after - restated.before - 4.66).abs() < 0.1,
        "{} to {}",
        restated.before,
        restated.after
    );
    assert!(restated.worst < 0.004, "worst district {}", restated.worst);
}

#[test]
fn a_models_own_enrolled_adm_is_the_previous_years_and_the_department_says_so() {
    // `[a] Enrolled ADM` is the FY2026 model's own `[b3] FY25` column, for all 611 districts.
    assert_eq!(counts::current_is_the_third_column(2026), (611, 611));

    // Which the department's line-by-line explanation states outright, so the `Directions` sheet's
    // "Enrolled ADM (a) | FY26 (Aug #1)" names a *collection window* and not a year of data. Both
    // cautions about that sheet are needed: it is the maintained table of where numbers come from,
    // and what it names are collections.
    let editions = include_str!("../fixtures/dew-sfpr-line-by-line.txt");
    let flat: String = editions.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(flat.contains(
        "The Enrolled ADM is calculated for FY 2023, FY 2024, and FY 2025. The Base Cost Enrolled \
         ADM is the larger of the 3- year Average or the FY 2025 Enrolled ADM."
    ));
    assert!(flat.contains(
        "The Enrolled ADM is calculated for FY 2022, FY 2023, and FY 2024. The Base Cost Enrolled \
         ADM is the larger of the 3- year Average or the FY 2024 Enrolled ADM."
    ));

    // The FY2027 model breaks the identity in exactly one district, by exactly fifty pupils —
    // already recorded on `panel::Transportation`'s sibling field. No line-by-line explanation of
    // that model is published.
    assert_eq!(counts::current_is_the_third_column(2027), (610, 611));
}

#[test]
fn the_growth_supplements_fy2023_is_a_different_number_from_the_base_costs() {
    let series = counts::adm_history();
    let base_cost = &series[&2023];
    let supplement: BTreeMap<String, f64> = panel::panel()
        .iter()
        .map(|record| (record.irn.clone(), record.supplements.adm_fy23))
        .collect();

    let mut paired = 0;
    let mut identical = 0;
    let mut worst: f64 = 0.0;
    for (irn, theirs) in &supplement {
        let Some(ours) = base_cost.get(irn) else {
            continue;
        };
        paired += 1;
        if (ours - theirs).abs() < 1e-9 {
            identical += 1;
        } else if *ours != 0.0 {
            worst = worst.max(((theirs - ours) / ours).abs());
        }
    }
    assert_eq!(paired, 609, "the panel's population");
    assert_eq!(
        identical, 1,
        "two columns, one name, one agreement in the panel"
    );
    assert!(
        worst > 0.04,
        "the largest disagreement is {worst}, which would make this a rounding story"
    );
}

#[test]
fn the_persistence_the_damping_parameterises_is_a_third_and_the_splice_says_otherwise() {
    // On the consistent series: two windows, two estimates. The later one pairs the growth into
    // FY2025 with the growth into FY2026 and so touches no FY2023 column at all, which is why the
    // choice of FY2023 below cannot reach it.
    let measured = counts::adm_persistence();
    assert_eq!(measured.len(), 2);
    assert_eq!((measured[0].0, measured[0].1), (2024, 2025));
    assert_eq!((measured[1].0, measured[1].1), (2025, 2026));
    assert_eq!(measured[0].3, 609);
    assert!(
        (measured[0].2 - 0.327_930).abs() < 0.000_01,
        "{}",
        measured[0].2
    );
    assert!(
        (measured[1].2 - 0.341_205).abs() < 0.000_01,
        "{}",
        measured[1].2
    );

    // `DEFAULT_DAMPING` is the share of a growth rate a projection carries forward, fitted out of
    // sample on a different survey of a different population. Both windows land beside it.
    for (_, _, r, _) in &measured {
        assert!(
            (r - DEFAULT_DAMPING).abs() < 0.05,
            "{r} against a damping of {DEFAULT_DAMPING}"
        );
    }

    // Splice the growth supplement's FY2023 onto the base-cost family and the earlier window
    // changes sign. Same districts, same years, one column swapped.
    let history = counts::adm_history();
    let records = panel::panel();
    let spliced: Vec<(f64, f64)> = records
        .iter()
        .filter_map(|record| {
            let irn = &record.irn;
            let (fy24, fy25, fy26) = (
                history[&2024].get(irn)?,
                history[&2025].get(irn)?,
                history[&2026].get(irn)?,
            );
            let fy23 = record.supplements.adm_fy23;
            (fy23 > 0.0 && *fy24 > 0.0 && *fy25 > 0.0)
                .then(|| (fy24 / fy23 - 1.0, fy25 / fy24 - 1.0))
                .filter(|_| *fy26 > 0.0)
        })
        .collect();
    assert_eq!(spliced.len(), 609);
    let spliced = correlation(&spliced);
    assert!(
        (spliced + 0.048_823).abs() < 0.000_01,
        "spliced persistence {spliced}"
    );
    assert!(
        spliced < 0.0 && measured[0].2 > 0.3,
        "the splice reverses the sign: {spliced} against {}",
        measured[0].2
    );
}
