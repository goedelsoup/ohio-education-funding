//! The candidate for the districts below the twenty-mill floor, tested against a column that was
//! already committed.
//!
//! `parameter/twenty-mill-floor` records fourteen districts whose voters approved between 23 and
//! 48 mills and whose effective Class I rate sits just below twenty, and names the leading
//! explanation: the **pre-1982 joint vocational taxes** of R.C. 319.301(E)(1), a term the
//! corpus's model of the floor omits. It records the confirmation as blocked — "confirming it
//! needs TY1981 joint vocational current-expense rates by district", and "the membership lists
//! alone are unpopulated".
//!
//! # What the abstract already knew
//!
//! Table SD-1 publishes taxes charged twice, once excluding joint vocational operating levies and
//! once including them. The difference is the joint vocational levy on that district's parcels,
//! so the column pair *is* a membership list — see [`sd1::joint_vocational_tax`]. And because a
//! joint vocational district levies one rate per class across the whole of its territory, a group
//! of districts that fits one pair of class rates is one joint vocational district, which
//! [`sd1::joint_vocational_fit`] settles without naming it.
//!
//! # Why that decides the question
//!
//! The pre-1982 term is a property of the joint vocational district, not of the member. If it is
//! positive it binds every member that reaches the floor. Shelby County's eight districts are one
//! joint vocational district; four of them sit below twenty and a fifth sits at exactly twenty.
//! The term cannot be positive for four and zero for the fifth, so it is not what puts the four
//! below. Jefferson and Muskingum show the same shape, and six of the districts below the floor
//! are in no joint vocational district at all.
//!
//! The other named candidate does not survive either, for an unrelated reason recorded below: the
//! shortfall reproduces year over year to five decimal places, and a rounding residual in an
//! annually recomputed factor is a fresh draw each year.

use std::collections::BTreeMap;

use dispersion::sd1::{self, TaxRow};

/// The floor, in mills.
const FLOOR: f64 = 20.0;

/// Half a unit of the four-decimal rounding the abstract publishes rates at.
const PUBLISHED_HALF_UNIT: f64 = 5e-5;

/// The Class I rate implied by the charges, in mills — taxes over value, not the published
/// column.
///
/// The published column is used everywhere else in the corpus and is right; it is recomputed here
/// because TY2023's workbook publishes it to two decimals and the argument below turns on the
/// fifth.
fn computed_class1_rate(row: &TaxRow) -> Option<f64> {
    let value = row.class1_value?;
    let taxes = row.class1_taxes_charged?;
    (value > 0.0).then(|| taxes / value * 1000.0)
}

fn tax_year(year: u16) -> Vec<TaxRow> {
    sd1::rows()
        .into_iter()
        .filter(|row| row.tax_year == year)
        .collect()
}

/// The published rate is the charges over the value, so a rate below twenty is dollars that were
/// not charged rather than a rounding of the column.
///
/// This is the premise everything else rests on. If the published effective rate were a separate
/// certified quantity, a district could show 19.93 while its taxpayers were charged twenty mills,
/// and there would be nothing to explain.
#[test]
fn the_published_class1_rate_is_the_charges_over_the_value() {
    for year in sd1::tax_years() {
        // TY2023's workbook publishes the rate to two decimals; the others to four.
        let half_unit = if year == 2023 {
            5e-3
        } else {
            PUBLISHED_HALF_UNIT
        };
        let mut worst: f64 = 0.0;
        for row in tax_year(year) {
            let (Some(computed), Some(published)) = (computed_class1_rate(&row), row.class1_rate)
            else {
                continue;
            };
            worst = worst.max((computed - published).abs());
        }
        assert!(
            worst <= half_unit,
            "TY{year}: the published Class I rate departs from taxes over value by {worst} mills"
        );
    }
}

/// The abstract names which districts are in a joint vocational district, by arithmetic.
///
/// 501 of 611 carry a joint vocational levy in TY2024. The 110 that do not are not scattered:
/// they concentrate in the counties whose career-technical education is not delivered by a joint
/// vocational district, which is the shape a real membership signal has and noise does not.
#[test]
fn the_abstract_carries_the_membership_the_corpus_called_unpopulated() {
    let rows = tax_year(2024);
    assert_eq!(rows.len(), 611, "districts in the TY2024 abstract");

    let members = rows
        .iter()
        .filter(|row| sd1::joint_vocational_tax(row).is_some_and(|tax| tax > 0.5))
        .count();
    assert_eq!(members, 501, "districts carrying a joint vocational levy");
    assert_eq!(rows.len() - members, 110, "districts carrying none");

    let mut by_county: BTreeMap<String, usize> = BTreeMap::new();
    for row in &rows {
        if sd1::joint_vocational_tax(row).is_some_and(|tax| tax > 0.5) {
            continue;
        }
        *by_county
            .entry(row.county.to_ascii_uppercase())
            .or_default() += 1;
    }
    let mut ranked: Vec<(String, usize)> = by_county.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let top: Vec<&str> = ranked.iter().take(5).map(|(c, _)| c.as_str()).collect();
    assert_eq!(
        top,
        vec!["CUYAHOGA", "STARK", "HANCOCK", "SUMMIT", "FRANKLIN"],
        "the counties holding the most districts with no joint vocational levy"
    );
    let concentration: usize = ranked.iter().take(5).map(|(_, n)| n).sum();
    assert_eq!(
        concentration, 57,
        "districts with none in those five counties"
    );
}

/// One county's districts fit one pair of joint vocational class rates, in every tax year.
///
/// Two free parameters explaining eight districts' levies to within three thousandths of one per
/// cent, four years running, at rates that move each year. That is co-membership, and it is the
/// premise of the test after this one.
///
/// The rates matter as much as the residuals. R.C. 319.301(E)(3) floors a joint vocational
/// district at two mills, so a clean fit at two mills would be evidence of nothing — two floored
/// joint vocational districts fit the same plane. Shelby's Class I rate runs 3.47 down to 2.67
/// mills and its Class II rate above 4.6, well clear of the floor.
#[test]
fn shelby_countys_eight_districts_are_one_joint_vocational_district() {
    let expected = [
        (2021, 3.470_54, 5.170_18),
        (2022, 3.081_49, 5.164_83),
        (2023, 2.675_10, 4.688_40),
        (2024, 2.672_22, 4.693_03),
    ];
    for (year, class1, class2) in expected {
        let group = sd1::county("Shelby", year);
        assert_eq!(group.len(), 8, "Shelby districts in TY{year}");

        let fit = sd1::joint_vocational_fit(&group).expect("Shelby fits");
        assert!(
            (fit.class1_mills - class1).abs() < 5e-5 && (fit.class2_mills - class2).abs() < 5e-5,
            "TY{year}: fitted {} / {} against {class1} / {class2}",
            fit.class1_mills,
            fit.class2_mills
        );
        assert!(
            fit.worst_residual < 5e-5,
            "TY{year}: worst residual {} on eight districts",
            fit.worst_residual
        );
        assert!(
            fit.class1_mills > 2.5,
            "TY{year}: a fit at the two-mill floor would prove nothing, and this is {}",
            fit.class1_mills
        );
    }
}

/// The term cannot be positive for one member and zero for another, and in Shelby it would have
/// to be both.
///
/// Anna Local's Class I charges come to twenty mills within four millionths in every one of the
/// four tax years — the abstract charges whole dollars, so that is as pinned as arithmetic on this
/// table gets. In each of those years at least one district in the same joint vocational district
/// sits materially below twenty: Russia in TY2021, and Botkins by seven hundredths of a mill in
/// the three after it. The pre-1982 term is the joint vocational district's, so it is the same for
/// both. If it is positive, Anna cannot be at twenty; if it is zero, Botkins cannot be below.
#[test]
fn a_co_member_sits_at_exactly_twenty_while_others_sit_below_it() {
    /// Anna is at the floor to within five millionths of a mill, which is a tenth of the
    /// abstract's own published precision.
    const PINNED: f64 = 5e-6;

    /// Anna Local. The abstract renames districts between workbooks — "Anna Local SD" in TY2021
    /// and "ANNA LSD" after it — so the join is on the number.
    const ANNA: &str = "049759";

    for year in sd1::tax_years() {
        let shelby: BTreeMap<&str, f64> = sd1::county("Shelby", year)
            .into_iter()
            .filter_map(|row| computed_class1_rate(row).map(|rate| (row.irn.as_str(), rate)))
            .collect();

        let anna = shelby[ANNA];
        assert!(
            (anna - FLOOR).abs() < PINNED,
            "TY{year}: Anna's effective Class I rate is {anna}"
        );

        let deepest = shelby
            .values()
            .map(|rate| FLOOR - rate)
            .fold(0.0_f64, f64::max);
        assert!(
            deepest > 0.02,
            "TY{year}: the deepest Shelby shortfall is {deepest} mills, which rounding could carry"
        );
    }
}

/// Two more counties show the same shape, on the weaker evidence a floored fit gives.
///
/// Jefferson and Muskingum each fit one pair of class rates and each hold a district below the
/// floor beside districts at exactly twenty. Both fits land at two mills, which is where
/// R.C. 319.301(E)(3) puts a joint vocational district that has reached its own floor, so neither
/// is proof of co-membership on its own. They are recorded because the shape repeats, not because
/// either would carry the argument.
#[test]
fn the_same_shape_repeats_in_two_counties_whose_fits_sit_at_the_floor() {
    for (name, below_the_floor, at_the_floor) in [
        ("Jefferson", "BUCKEYE LSD (JEFFERSON CO.)", 3),
        ("Muskingum", "FRANKLIN LSD", 2),
    ] {
        let group = sd1::county(name, 2024);
        let fit = sd1::joint_vocational_fit(&group).unwrap_or_else(|| panic!("{name} fits"));
        assert!(
            fit.worst_residual < 5e-5,
            "{name}: worst residual {}",
            fit.worst_residual
        );
        assert!(
            (fit.class1_mills - 2.0).abs() < 5e-4,
            "{name}: Class I rate {} — this test is about a floored fit",
            fit.class1_mills
        );

        let rates: BTreeMap<&str, f64> = group
            .iter()
            .filter_map(|row| computed_class1_rate(row).map(|rate| (row.name.as_str(), rate)))
            .collect();
        assert!(
            rates[below_the_floor] < FLOOR - PUBLISHED_HALF_UNIT,
            "{below_the_floor} is at {}",
            rates[below_the_floor]
        );
        let pinned = rates
            .values()
            .filter(|rate| (**rate - FLOOR).abs() < PUBLISHED_HALF_UNIT)
            .count();
        assert_eq!(pinned, at_the_floor, "{name} districts at exactly twenty");
    }
}

/// Six districts below the floor are in no joint vocational district at all.
///
/// R.C. 319.301(E)(1) defines the term by reference to "the joint vocational school district of
/// which the school district is a part". A district that is part of none has nothing to put in it.
/// McComb is the second largest shortfall among the fourteen the corpus recorded, and it is one of
/// the six.
#[test]
fn six_districts_below_the_floor_pay_no_joint_vocational_tax() {
    let rows = tax_year(2024);
    let mut orphans: Vec<(String, f64)> = rows
        .iter()
        .filter(|row| sd1::joint_vocational_tax(row).is_some_and(|tax| tax <= 0.5))
        .filter_map(|row| {
            computed_class1_rate(row)
                .filter(|rate| *rate < FLOOR - PUBLISHED_HALF_UNIT)
                .map(|rate| (row.name.clone(), rate))
        })
        .collect();
    orphans.sort_by(|a, b| a.1.total_cmp(&b.1));

    let names: Vec<&str> = orphans.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "MIDDLE BASS LSD",
            "NORTH BASS LSD",
            "MC COMB LSD",
            "FT. RECOVERY LSD",
            "OSNABURG LSD",
            "MORGAN LSD",
        ],
        "districts below the floor with no joint vocational levy"
    );

    let mccomb = orphans
        .iter()
        .find(|(name, _)| name == "MC COMB LSD")
        .expect("McComb");
    assert!(
        (FLOOR - mccomb.1 - 0.1076).abs() < 5e-4,
        "McComb is short by {} mills",
        FLOOR - mccomb.1
    );
}

/// The shortfall is not a fresh draw each year, which is what the other candidate would be.
///
/// Rounding inside the reduction-factor computation was the corpus's second explanation and the
/// one it thought covered thirteen of the fourteen. R.C. 319.301(D)(1) recomputes the percentage
/// annually against that year's valuation, so a rounding residual would land somewhere new every
/// year. Seventeen districts are below the floor in all four tax years, and the closest of them
/// reproduce to the fifth decimal place across a revaluation.
#[test]
fn the_shortfall_reproduces_across_tax_years() {
    let mut series: BTreeMap<String, Vec<(u16, f64)>> = BTreeMap::new();
    for row in sd1::rows() {
        if let Some(rate) = computed_class1_rate(&row) {
            series
                .entry(row.irn.clone())
                .or_default()
                .push((row.tax_year, rate));
        }
    }
    let persistent: Vec<&String> = series
        .iter()
        .filter(|(_, years)| {
            years.len() == 4
                && years
                    .iter()
                    .all(|(_, rate)| *rate < FLOOR - PUBLISHED_HALF_UNIT)
        })
        .map(|(irn, _)| irn)
        .collect();
    assert_eq!(
        persistent.len(),
        17,
        "districts below the floor in all four tax years"
    );

    // Southern Local reproduces to five decimals four years running; Botkins across a year in
    // which its Class I value rose 0.88%.
    for (irn, spread) in [("046441", 3e-5), ("048819", 7e-4)] {
        let rates: Vec<f64> = series[irn].iter().map(|(_, rate)| *rate).collect();
        let (low, high) = (
            rates.iter().copied().fold(f64::MAX, f64::min),
            rates.iter().copied().fold(f64::MIN, f64::max),
        );
        assert!(
            high - low < spread,
            "{irn}: four years span {} mills — {rates:?}",
            high - low
        );
    }
}
