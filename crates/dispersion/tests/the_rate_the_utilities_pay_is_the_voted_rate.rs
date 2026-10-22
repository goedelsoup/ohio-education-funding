//! What a reduction factor took away, measured per district, per class and per tax year.
//!
//! `parameter/tax-reduction-factor` needs the rate a district's levies were voted at beside the
//! rate each class of real property is actually charged. Table SD-1 publishes the second for both
//! classes and four tax years, and was read as publishing nothing for the first: the voted rate
//! sits in the FY2024 district profile, for one tax year.
//!
//! # The column that was already there
//!
//! R.C. 319.301 reduces taxes on real property. Public utility tangible property is not real
//! property, so it is charged the full rate — and the abstract's public utility millage matches
//! the profile's current operating millage in every one of the 606 districts the two share. That
//! makes it each district's gross current-expense rate in every tax year the abstract carries,
//! and a class's reduction is one minus its effective rate over that one.
//!
//! # What the class split shows
//!
//! R.C. 319.301(C) computes a factor for each class separately, and reappraisal raises Class I
//! values faster than Class II, so Class I is reduced further and its rate falls below Class II's.
//! The floor of (E)(2) applies "for either class", and the abstract has districts on it in Class
//! II alone.

use dispersion::profile;
use dispersion::sd1::{self, Class};

/// Half a unit of the two-decimal grain the profile and TY2023's workbook publish rates at.
const GRAIN: f64 = 0.005;

/// The abstract's utility millage is the profile's voted current operating millage, district by
/// district, for the one tax year both publish.
///
/// Inclusive at half a hundredth: TY2023's workbook rounds to two decimals and the profile writes
/// a third, so Girard's 44.755 is published as 44.75 and sits exactly on the edge.
#[test]
fn the_utility_millage_is_the_voted_current_operating_millage() {
    let mut shared = 0;
    for district in profile::districts() {
        let Some(voted) = district.current_operating_millage else {
            continue;
        };
        let Some(row) = sd1::at(&district.irn, 2023) else {
            continue;
        };
        let gross = sd1::gross_millage(&row).expect("a district with a voted rate has a gross one");
        assert!(
            (gross - voted).abs() <= GRAIN + 1e-9,
            "{} {}: utility {gross} against voted {voted}",
            district.irn,
            row.name
        );
        shared += 1;
    }
    assert_eq!(shared, 606);
}

/// No class is ever charged more than the gross rate, in any year: a reduction factor reduces.
#[test]
fn no_class_rate_exceeds_the_gross_rate() {
    for row in sd1::rows() {
        for class in [Class::One, Class::Two] {
            if let Some(reduction) = sd1::reduction(&row, class) {
                assert!(
                    reduction > -GRAIN,
                    "{} TY{} {class:?}: reduction {reduction}",
                    row.irn,
                    row.tax_year
                );
            }
        }
    }
}

/// Class I is reduced further than Class II in every tax year, the floor is reached in either
/// class, and the Class I floor count is the one `sd1::rates` already publishes.
#[test]
fn the_two_classes_are_reduced_separately() {
    let years: Vec<_> = sd1::tax_years()
        .into_iter()
        .filter_map(sd1::class_reductions)
        .collect();
    assert_eq!(years.len(), 4);
    for year in &years {
        assert!(year.class1_median > year.class2_median, "{year:?}");
        assert!(year.class2_above_class1 * 2 > year.districts, "{year:?}");
        assert!(
            year.class2_at_the_floor > year.both_at_the_floor,
            "{year:?}"
        );
        let rates = sd1::rates(year.tax_year).expect("the same year");
        assert_eq!(year.class1_at_the_floor, rates.at_the_floor);
    }
    assert_eq!(
        years
            .iter()
            .map(|year| year.class2_at_the_floor)
            .collect::<Vec<_>>(),
        [27, 55, 79, 74]
    );
    assert!(sd1::class_reductions(2020).is_none());
}

/// Wilmington City is on the floor in Class II and not in Class I, TY2024.
#[test]
fn a_district_can_be_on_the_floor_in_class_two_alone() {
    let row = sd1::at("045112", 2024).expect("Wilmington CSD, TY2024");
    assert!(row.name.to_uppercase().contains("WILMINGTON"));
    assert!((Class::Two.rate(&row).unwrap() - 20.0).abs() < GRAIN);
    assert!(Class::One.rate(&row).unwrap() > 20.0 + GRAIN);
    assert!(sd1::gross_millage(&row).unwrap() > 24.0);
}

/// Cleveland Heights-University Heights through Cuyahoga County's TY2024 reappraisal.
///
/// The gross rate does not move. Class I value rises two-fifths and Class II a tenth, and each
/// class's taxes charged rise by a few per cent — the new construction R.C. 319.301 leaves out of
/// carryover property — because each class's rate falls by what its own values rose.
#[test]
fn a_reappraisal_moves_each_class_rate_by_its_own_values() {
    let before = sd1::at("043794", 2023).expect("TY2023");
    let after = sd1::at("043794", 2024).expect("TY2024");
    assert_eq!(sd1::gross_millage(&before), sd1::gross_millage(&after));
    let growth = |pick: fn(Class, &sd1::TaxRow) -> Option<f64>, class: Class| {
        pick(class, &after).unwrap() / pick(class, &before).unwrap() - 1.0
    };
    let value1 = growth(Class::value, Class::One);
    let value2 = growth(Class::value, Class::Two);
    let taxes1 = growth(Class::taxes_charged, Class::One);
    let taxes2 = growth(Class::taxes_charged, Class::Two);
    assert!(value1 > 0.4 && value2 < 0.1, "{value1} {value2}");
    assert!((0.0..0.04).contains(&taxes1), "{taxes1}");
    assert!((0.0..0.04).contains(&taxes2), "{taxes2}");
    let r1 = Class::One.rate(&after).unwrap();
    let r2 = Class::Two.rate(&after).unwrap();
    assert!(r2 - r1 > 30.0, "{r1} {r2}");
}
