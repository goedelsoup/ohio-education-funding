//! What the appropriation does to the formula's own self-correction.
//!
//! R.C. 3317.0212 pays special education transportation on reported cost at the district's state
//! share, floored. So when a district's costs rise — fuel, drivers, routes — the section earns it
//! more the following year. That is the correction, and it is written into the statute.
//!
//! The earmark inside GRF ALI 200502 is not. It is a fixed dollar figure in a budget act, the
//! computed entitlement is divided into it, and FY2027's factor is **0.917459741** because the
//! entitlement exceeded the line. Nothing in R.C. 3317.0212 mentions any of this, and the factor
//! does not appear in the published per-pupil amount.
//!
//! # What these tests establish
//!
//! That the cap is not a detail at the margin. **A 20% rise in reported cost raises what districts
//! are actually paid by 1.06%**, because payments asymptote to the appropriation and the
//! appropriation does not move. The self-correction is very nearly inoperative once the line
//! binds, and FY2027 is a year in which it binds.
//!
//! # Why this is parameterised by cost and not by diesel
//!
//! Because the fuel pass-through is not identified and this repository has established that it
//! cannot be, from anything it can reach: `the_fuel_response_the_panel_cannot_identify` fits
//! eleven specifications of diesel against F-33 transportation spending and every one spans zero.
//! The blocker there is structural rather than statistical — identification needs price ×
//! miles-per-pupil, and the F-33 carries no mile column in any year.
//!
//! So `rise` is a rise in reported cost, whatever moved it. Composing a fuel share with these
//! curves is left to a reader who has one; nothing here supplies it.

use project::transport::{self, Proration};

/// FY2027's published factor, from the department's own rate sheet.
const PUBLISHED_FACTOR: f64 = 0.917_459_740_976_215;

fn at(rise: f64) -> Proration {
    transport::proration_under(2027, rise).expect("FY2027 is in the rate series")
}

/// Zero rise reproduces the year as published — factor and payment both.
///
/// The licence for every other row. A forward curve that does not pass through the observed point
/// is describing a different formula.
#[test]
fn the_curve_passes_through_the_published_year() {
    let base = at(0.0);
    assert!(
        (base.factor_districts_only - PUBLISHED_FACTOR).abs() < 1e-9,
        "{} is not the published factor",
        base.factor_districts_only
    );
    assert!((base.factor_uniform - PUBLISHED_FACTOR).abs() < 1e-9);

    let published = transport::special_education(2027).expect("FY2027");
    assert!(
        (base.paid - published.paid).abs() < 1.0,
        "{} is not the published payment {}",
        base.paid,
        published.paid
    );
    assert!((base.allocation - published.districts).abs() < 1.0);
}

/// The factor falls roughly a point and a half for every two and a half points of cost.
#[test]
fn the_factor_falls_as_reported_cost_rises() {
    let close = |got: f64, want: f64| assert!((got - want).abs() < 1e-5, "{got} is not {want}");
    close(at(0.05).factor_districts_only, 0.876_382);
    close(at(0.10).factor_districts_only, 0.838_826);
    close(at(0.20).factor_districts_only, 0.772_607);

    // Monotone, which is worth asserting rather than assuming: the factor is a ratio whose
    // denominator is the thing moving, so nothing about the arithmetic guarantees it locally.
    let mut previous = f64::INFINITY;
    for step in 0..=40 {
        let factor = at(f64::from(step) * 0.01).factor_districts_only;
        assert!(factor < previous, "the factor rose at {step}%");
        previous = factor;
    }
}

/// And inverting it: how much cost it takes to reach a given factor.
#[test]
fn the_rise_that_reaches_a_given_factor() {
    let close = |got: f64, want: f64| assert!((got - want).abs() < 1e-4, "{got} is not {want}");
    close(
        transport::rise_reaching(2027, 0.90).expect("reachable"),
        0.020_695,
    );
    close(
        transport::rise_reaching(2027, 0.85).expect("reachable"),
        0.084_662,
    );
    close(
        transport::rise_reaching(2027, 0.80).expect("reachable"),
        0.156_625,
    );

    // A factor above the year's own is not reachable by *raising* costs, and the inverse says so
    // rather than returning a negative rise that reads as a fall.
    assert_eq!(transport::rise_reaching(2027, 0.95), None);
    assert_eq!(transport::rise_reaching(2027, 1.10), None);
    assert_eq!(transport::rise_reaching(2027, 0.0), None);
}

/// **The finding.** A fifth more reported cost buys districts one per cent more money.
///
/// Payments are `allocation × appropriation / (allocation + remainder)`, which rises toward the
/// appropriation and never past it. So the earned entitlement and the received payment come apart
/// as fast as costs rise: at 20% the districts earn $54.3m they do not receive, against $16.4m at
/// the published point.
///
/// This is the sense in which an appropriation caps a statutory formula. Divisions (C) and (D)
/// promise reimbursement at the state share; the line item decides what fraction of the promise
/// is kept, and a reader of either document alone cannot see the other.
#[test]
fn a_twenty_per_cent_cost_rise_raises_payments_by_one_per_cent() {
    let base = at(0.0);
    let risen = at(0.20);

    let payment_growth = risen.paid / base.paid - 1.0;
    assert!(
        (0.010..0.011).contains(&payment_growth),
        "payments grew {payment_growth}"
    );

    // While the gap between earned and received more than triples.
    assert!((base.withheld - 16_430_549.69).abs() < 1.0);
    assert!((risen.withheld - 54_318_085.94).abs() < 1.0);
    assert!(risen.withheld > base.withheld * 3.0);
}

/// And the ceiling is the appropriation itself, approached and never crossed.
///
/// Stated at an absurd rise on purpose: the point is the shape of the function, not a forecast.
/// Doubling every district's reported cost raises the statewide payment by 3.2%.
#[test]
fn payments_asymptote_to_the_appropriation() {
    let published = transport::special_education(2027).expect("FY2027");
    let doubled = at(1.0);
    let tenfold = at(9.0);

    assert!(doubled.paid < published.appropriation);
    assert!(tenfold.paid < published.appropriation);
    assert!(tenfold.paid > doubled.paid);

    let doubling_growth = doubled.paid / at(0.0).paid - 1.0;
    assert!(
        (0.031..0.033).contains(&doubling_growth),
        "doubling cost grew payments {doubling_growth}"
    );

    // Within a rounding error of the line at ten times the cost, which is what "asymptote" means
    // here rather than a description of a limit nobody computes.
    assert!(published.appropriation - tenfold.paid < 1_300_000.0);
}

/// The two bounds on the remainder are close enough that the choice does not carry the finding.
///
/// LSC's denominator holds $13.3m the calculator's districts do not — county DD boards and
/// educational service centres. Whether their costs rise with the districts' is not established,
/// so both ends are computed; the spread is under a point at 20%, which is why the finding above
/// is stated without choosing between them.
#[test]
fn holding_the_remainder_fixed_or_moving_it_barely_changes_the_answer() {
    for rise in [0.05, 0.10, 0.20] {
        let proration = at(rise);
        let spread = proration.factor_districts_only - proration.factor_uniform;
        assert!(spread > 0.0, "the uniform bound should be the deeper one");
        assert!(spread < 0.009, "the bounds are {spread} apart at {rise}");
    }
}

/// FY2026 did not prorate, and that makes its curve a lower bound rather than a measurement.
///
/// The remainder is solved out of a factor below 1.0. At exactly 1.0 nothing is solved and
/// `outside_the_model` returns the headroom, which is an *upper* bound on the remainder — so the
/// factor computed from it is the deepest proration consistent with what is known.
///
/// The bound is still worth having: it says FY2026's remainder cannot have exceeded $11.78m,
/// where FY2027's is $13.29m. Two adjacent years of the same quantity, one measured and one
/// bounded, and they are consistent.
#[test]
fn the_unprorated_year_bounds_the_remainder_rather_than_measuring_it() {
    let fy2026 = transport::special_education(2026).expect("FY2026");
    let fy2027 = transport::special_education(2027).expect("FY2027");

    assert_eq!(fy2026.stated_factor, 1.0);
    assert!((fy2026.outside_the_model() - fy2026.headroom()).abs() < 1e-6);
    assert!((fy2026.headroom() - 11_778_159.68).abs() < 1.0);
    assert!((fy2027.outside_the_model() - 13_287_097.51).abs() < 1.0);

    // The bound does not contradict the measurement.
    assert!(fy2026.headroom() < fy2027.outside_the_model());

    // And the year is still usable at a rise, as a floor on the factor.
    let stressed = transport::proration_under(2026, 0.10).expect("FY2026");
    assert!(stressed.factor_districts_only < 1.0);
    assert!((stressed.factor_districts_only - 0.914_627).abs() < 1e-5);
}
