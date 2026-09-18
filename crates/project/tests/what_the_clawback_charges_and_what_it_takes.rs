//! The open-enrolment clawback has two sizes, and the corpus reported the larger one as the
//! smaller one's name.
//!
//! `[I] = max([H2] − [I1] − [H], 0)` clamps at zero. So being *charged* the adjustment and
//! *losing* it are different events, and the gap between them is not small: 43 districts are
//! charged $5.11 million and 22 lose $3.04 million. The other 21 are charged against a guarantee
//! that was already nothing.
//!
//! This existed as one figure, computed as the sum of `[I1]` and labelled "Total withheld by the
//! open-enrolment clawback". The arithmetic was right and the label named a different quantity.

use project::panel::panel;

/// Districts charged an adjustment, and what they are charged in aggregate.
#[test]
fn forty_three_districts_are_charged_five_million() {
    let panel = panel();
    let charged: Vec<_> = panel
        .iter()
        .filter(|r| r.transition.open_enrollment_adjustment > 0.0)
        .collect();
    assert_eq!(
        charged.len(),
        43,
        "districts charged an open-enrolment adjustment"
    );
    let total: f64 = charged
        .iter()
        .map(|r| r.transition.open_enrollment_adjustment)
        .sum();
    assert!(
        (total - 5_110_049.66).abs() < 0.01,
        "total charged: {total:.2}"
    );
}

/// And what the guarantee actually loses, which is two thirds of it.
#[test]
fn twenty_two_of_them_lose_three_million() {
    let panel = panel();
    let reduced: Vec<_> = panel
        .iter()
        .filter(|r| r.open_enrollment_withheld() > 0.005)
        .collect();
    assert_eq!(reduced.len(), 22, "districts whose guarantee is reduced");
    let total: f64 = reduced.iter().map(|r| r.open_enrollment_withheld()).sum();
    assert!(
        (total - 3_037_536.64).abs() < 0.01,
        "total withheld: {total:.2}"
    );

    // The gap is the whole reason the two figures are kept apart.
    let charged: f64 = panel
        .iter()
        .map(|r| r.transition.open_enrollment_adjustment)
        .sum();
    assert!(
        charged - total > 2_000_000.0,
        "the charge exceeds the loss by over two million: {:.2}",
        charged - total
    );
}

/// The twenty-first district is not the twenty-second, and the difference is the clamp.
///
/// **West Muskingum Local** is charged $146,074.11 and loses $7,105.69, because $7,105.69 is the
/// whole of the guarantee it had. It is left at exactly zero, so every test of the form
/// "guarantee > 0 after the clawback" reports twenty-one districts and misses it — which is what
/// `Transition::guarantee_before_clawback` does, and why its claim has been narrowed.
#[test]
fn the_clamp_is_the_twenty_second_district() {
    let panel = panel();
    let clamped: Vec<_> = panel
        .iter()
        .filter(|r| r.open_enrollment_withheld() > 0.005 && r.guarantee <= 0.005)
        .collect();
    assert_eq!(clamped.len(), 1, "exactly one district is clamped to zero");

    let west = clamped[0];
    assert_eq!(west.name, "West Muskingum Local");
    assert!(
        (west.transition.open_enrollment_adjustment - 146_074.11).abs() < 0.01,
        "charged: {:.2}",
        west.transition.open_enrollment_adjustment
    );
    assert!(
        (west.open_enrollment_withheld() - 7_105.69).abs() < 0.01,
        "withheld: {:.2}",
        west.open_enrollment_withheld()
    );

    // The instrument that misses it, asserted rather than described.
    assert_eq!(
        west.transition.guarantee_before_clawback(west.guarantee),
        None,
        "the Transition-level function cannot see the clamped district"
    );
}

/// A district charged against no guarantee loses nothing, and there are 21 of them.
#[test]
fn a_charge_against_no_guarantee_takes_nothing() {
    let panel = panel();
    let untouched: Vec<_> = panel
        .iter()
        .filter(|r| {
            r.transition.open_enrollment_adjustment > 0.0 && r.open_enrollment_withheld() <= 0.005
        })
        .collect();
    assert_eq!(untouched.len(), 21);
    for r in &untouched {
        assert_eq!(
            r.guarantee_before_clawback(),
            0.0,
            "{}: charged but with no guarantee to charge against",
            r.name
        );
    }
}
