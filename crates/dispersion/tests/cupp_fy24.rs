//! Equity statistics over the real FY2024 District Profile Report ("Cupp Report").
//!
//! The fixture is 606 Ohio traditional school districts as published by the Department of
//! Education and Workforce, read through [`dispersion::profile`] — which is the crate's one
//! reader of that file, and was four private ones until #157. These tests are the corpus's
//! empirical equity findings, pinned so they cannot drift silently: if a later fixture refresh
//! moves them, the test says so and the corpus node has to be revised deliberately.

use dispersion::profile::{self, column, paired, ProfileDistrict};
use dispersion::{wealth_neutrality, Dispersion};

fn districts() -> Vec<ProfileDistrict> {
    profile::districts()
}

#[test]
fn fixture_covers_the_full_set_of_traditional_districts() {
    let ds = districts();
    assert_eq!(ds.len(), 606, "Ohio traditional school districts in FY2024");
}

/// The corpus recorded "how many districts sit at the floor" as a first-order open question.
/// This is the answer: 170 of 606, or 28%.
#[test]
fn one_district_in_four_sits_exactly_at_the_twenty_mill_floor() {
    let ds = districts();
    let reporting = column(&ds, |d| d.effective_class1_millage).len();
    let at_floor = ds.iter().filter(|d| d.at_twenty_mill_floor()).count();
    assert_eq!(at_floor, 170);
    assert!((at_floor as f64 / reporting as f64 - 0.281).abs() < 0.005);
}

/// Twenty districts report an effective Class 1 rate below 20 mills. Six of them simply never
/// voted 20 — the floor cannot lift a rate above what voters approved, which is exactly the
/// guard the `millage` crate encodes.
#[test]
fn districts_below_the_floor_that_never_voted_twenty_mills() {
    let ds = districts();
    let below: Vec<&ProfileDistrict> = ds.iter().filter(|d| d.below_twenty_mill_floor()).collect();
    assert_eq!(below.len(), 20);

    let never_voted_twenty = below
        .iter()
        .filter(|d| d.never_voted_twenty_mills())
        .count();
    assert_eq!(never_voted_twenty, 6);
}

/// The remaining fourteen are unexplained by the corpus's model of the floor: they voted
/// between 23 and 48 mills yet report an effective Class 1 rate just under 20. Pinned as a
/// known anomaly so a future refinement of the floor model has something to test against.
#[test]
fn fourteen_districts_sit_just_below_the_floor_unexplained() {
    let ds = districts();
    let anomalies: Vec<&ProfileDistrict> = ds
        .iter()
        .filter(|d| d.below_twenty_mill_floor() && !d.never_voted_twenty_mills())
        .collect();
    assert_eq!(anomalies.len(), 14);
    for d in &anomalies {
        let eff = d
            .effective_class1_millage
            .expect("below the floor implies a rate");
        assert!(
            (19.7..19.995).contains(&eff),
            "{} sits at {eff}, outside the observed anomaly band",
            d.name
        );
    }
}

/// Effective millage never meaningfully exceeds voted millage — but five districts exceed it
/// by up to 0.0004 mills, which is published rounding rather than a violation (a voted rate
/// of 22.6499 against an effective rate of 22.65). The tolerance is set just above the
/// observed artifact so a genuine inversion would still fail.
#[test]
fn no_district_reports_effective_millage_meaningfully_above_its_voted_millage() {
    let mut rounding_artifacts = 0;
    for d in districts() {
        if let (Some(eff), Some(voted)) = (d.effective_class1_millage, d.current_operating_millage)
        {
            if eff > voted {
                rounding_artifacts += 1;
                assert!(
                    eff - voted < 0.001,
                    "{} reports {eff} effective against {voted} voted — too large to be rounding",
                    d.name
                );
            }
        }
    }
    assert_eq!(rounding_artifacts, 5, "published rounding artifacts");
}

/// One district reports no operating expenditure per pupil. Pinned so that a fixture refresh
/// which silently changes coverage is caught rather than absorbed.
#[test]
fn exactly_one_district_reports_no_operating_expenditure() {
    let ds = districts();
    let missing: Vec<&str> = ds
        .iter()
        .filter(|d| d.operating_expenditure_per_pupil.is_none())
        .map(|d| d.name.as_str())
        .collect();
    assert_eq!(missing.len(), 1);
    assert!(missing[0].starts_with("Southern Local (046441)"));
}

/// Ohio's operating spending dispersion, FY2024.
#[test]
fn operating_expenditure_dispersion() {
    let ds = districts();
    let d = Dispersion::of(&column(&ds, |d| d.operating_expenditure_per_pupil)).unwrap();

    // 605, not 606 — one district reports no operating expenditure. See the test above.
    assert_eq!(d.n, 605);
    assert!(
        (d.median - 15_646.0).abs() < 50.0,
        "median was {}",
        d.median
    );
    assert!(
        (d.coefficient_of_variation - 0.202).abs() < 0.01,
        "CV was {}",
        d.coefficient_of_variation
    );
    assert!(
        (d.restricted_range_ratio - 1.846).abs() < 0.02,
        "federal range ratio was {}",
        d.restricted_range_ratio
    );
    // The top half runs further above the median than the bottom half runs below it: Ohio's
    // spending distribution is right-skewed, so a mean-based summary overstates the typical
    // district.
    assert!(d.verstegen_index - 1.0 > 1.0 - d.mcloone_index);
}

/// The wealth-neutrality test. State aid does fall as property wealth rises — equalisation is
/// real — but wealth explains only about 30% of the variation in state aid per pupil.
#[test]
fn state_aid_compensates_for_property_wealth_but_only_partly() {
    let ds = districts();
    let (wealth, aid) = paired(
        &ds,
        |d| d.valuation_per_pupil,
        |d| d.state_revenue_per_pupil,
    );
    let w = wealth_neutrality(&wealth, &aid).unwrap();

    assert!(w.n >= 600);
    assert!(
        (w.correlation - -0.549).abs() < 0.02,
        "correlation was {}",
        w.correlation
    );
    assert!(w.slope < 0.0, "aid must fall as wealth rises");
    assert!(
        w.r_squared < 0.35,
        "wealth should leave most variation unexplained, r2 was {}",
        w.r_squared
    );
}

/// The finding that reframes the equity question: state aid tracks *poverty* more closely than
/// it tracks *property wealth*. Under a formula that adds resident income to valuation and
/// weights economically disadvantaged pupils, that is the design working — and it means a
/// wealth-neutrality test run on valuation alone understates how much targeting the system does.
#[test]
fn state_aid_tracks_poverty_more_closely_than_property_wealth() {
    let ds = districts();
    let (wealth, aid) = paired(
        &ds,
        |d| d.valuation_per_pupil,
        |d| d.state_revenue_per_pupil,
    );
    let (poverty, aid2) = paired(
        &ds,
        |d| d.economically_disadvantaged,
        |d| d.state_revenue_per_pupil,
    );

    let by_wealth = wealth_neutrality(&wealth, &aid).unwrap();
    let by_poverty = wealth_neutrality(&poverty, &aid2).unwrap();

    assert!(
        by_poverty.correlation > 0.0,
        "poorer districts get more aid"
    );
    assert!(
        (by_poverty.correlation - 0.630).abs() < 0.02,
        "poverty correlation was {}",
        by_poverty.correlation
    );
    assert!(
        by_poverty.correlation.abs() > by_wealth.correlation.abs(),
        "poverty ({:.3}) should bind more tightly than valuation ({:.3})",
        by_poverty.correlation,
        by_wealth.correlation
    );
}

/// Local revenue is the opposite: it tracks property wealth strongly, which is the mechanism
/// the state formula exists to offset.
#[test]
fn local_revenue_tracks_property_wealth_strongly() {
    let ds = districts();
    let (wealth, local) = paired(
        &ds,
        |d| d.valuation_per_pupil,
        |d| d.local_revenue_per_pupil,
    );
    let w = wealth_neutrality(&wealth, &local).unwrap();
    assert!(
        w.correlation > 0.7,
        "local revenue should track wealth closely, got {}",
        w.correlation
    );
    assert!(w.slope > 0.0);
}

/// Ohio has many small districts, so the average district and the average student's district
/// are not the same thing.
#[test]
fn enrollment_weighting_moves_the_statewide_average() {
    let ds = districts();
    let (epp, adm) = paired(
        &ds,
        |d| d.operating_expenditure_per_pupil,
        |d| d.enrolled_adm,
    );

    let unweighted = Dispersion::of(&epp).unwrap().mean;
    let weighted = dispersion::weighted_mean(&epp, &adm).unwrap();
    assert!(
        (unweighted - weighted).abs() > 200.0,
        "unweighted {unweighted:.0} vs weighted {weighted:.0}"
    );
}
