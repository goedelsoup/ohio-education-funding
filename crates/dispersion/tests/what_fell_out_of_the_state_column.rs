//! The FY2016 question `education-agency/toledo-city` opened, and the two guesses it took.
//!
//! The node recorded the tail as "mostly small districts with large industrial tax bases — which
//! is the profile of a district losing tangible personal property revenue", marked the cause
//! `[open]`, and pointed at `revenue-stream/tpp-replacement-payments`. This file refuted that,
//! found capacity aid in the median district's rise, and left the tail open.
//!
//! # Both readings of the tail were wrong, because there is no tail
//!
//! The panel's state column is gross of the community-school deduct through FY2015 and net of it
//! from FY2016 — see `the_break_that_belonged_to_the_survey.rs`, which is where that is
//! established. Every measure here is now stated on [`fy2016::BASIS`], gross in both eras.
//!
//! # What survives the restatement, and what it costs
//!
//! The industrial reading stays refuted and more cleanly. The step still tracks a district's
//! **total** assessed value, still monotonically, and capacity aid is still the only Ohio
//! component that keys on that quantity — but the claim that it was the *only* thing happening
//! does not survive: on one basis a disadvantage term reaches `t = +3.92` where the published
//! version had it at `+1.38` and concluded poverty was doing no work.

use dispersion::fy2016::{self, District};
use dispersion::survey_basis::Basis;

/// **The first withdrawal.** The step has no relationship to industrial property.
#[test]
fn the_step_does_not_track_industrial_property_at_all() {
    let industrial = fy2016::against(|d| d.industrial_share);
    let business = fy2016::against(|d| d.business_share);
    assert!(
        (industrial - 0.1175).abs() < 0.0005 && (business - 0.0164).abs() < 0.0005,
        "industrial {industrial:+.4}, business {business:+.4}"
    );
    // Both are the wrong sign for the tangible-personal-property reading, which needs
    // industrial districts to have fallen.
    assert!(
        industrial > 0.0 && business > 0.0,
        "an industry-heavy district gained *more*, not less"
    );

    // And the gradient has no order to it, so the near-zero correlation is not two tails
    // cancelling: the business-heaviest fifth sits between the second and the fourth.
    let gradient = fy2016::quintiles_by(|d| d.business_share);
    let (low, high) = (
        gradient.iter().copied().fold(f64::INFINITY, f64::min),
        gradient.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    assert!(
        high - low < 0.09,
        "the business-property gradient spans {:.4}: {gradient:?}",
        high - low
    );
    assert!(
        gradient[4] < gradient[3],
        "the gradient is monotone after all: {gradient:?}"
    );
}

/// **The finding.** What it tracks is total valuation, which is what capacity aid keys on.
#[test]
fn the_step_tracks_total_valuation_and_the_gradient_is_monotone() {
    let total = fy2016::against(|d| d.total_valuation.ln());
    assert!(
        (total + 0.2691).abs() < 0.0005,
        "the step against log total valuation is {total:+.4}"
    );
    assert!(
        total < fy2016::against(|d| d.valuation_per_pupil),
        "total valuation should beat wealth per pupil"
    );

    // Smallest total valuation first: every fifth gains, and none gains more than the one below.
    let gradient = fy2016::quintiles_by(|d| d.total_valuation);
    assert!(
        (gradient[0] - 0.2679).abs() < 0.0005 && (gradient[4] - 0.1060).abs() < 0.0005,
        "the gradient runs {gradient:?}"
    );
    assert!(
        gradient.windows(2).all(|w| w[0] >= w[1]),
        "the gradient is not monotone: {gradient:?}"
    );
    assert!(
        gradient.iter().all(|mean| *mean > 0.0),
        "every fifth should gain on average: {gradient:?}"
    );

    // And in one model it is the largest of the three terms.
    let fit = fy2016::model();
    assert!(
        (fit.standardized[0] + 0.19945).abs() < 0.0005 && fit.t_statistics[1] < -4.0,
        "total valuation is {:+.5} at t {:+.2}",
        fit.standardized[0],
        fit.t_statistics[1]
    );
    // The discriminating comparison, and the one the correction leaves standing: capacity aid is
    // the only component that prefers *total* valuation to wealth per pupil, and total valuation
    // wins by two thirds again rather than by the factor of two published.
    assert!(
        fit.standardized[0].abs() > fit.standardized[1].abs() * 1.5,
        "wealth per pupil is {:+.4} against total valuation's {:+.4}",
        fit.standardized[1],
        fit.standardized[0]
    );
}

/// **The second withdrawal**, which is this file's own: poverty is in the step after all.
#[test]
fn the_disadvantage_term_clears_two_once_the_adjustment_is_out_of_the_outcome() {
    let fit = fy2016::model();
    assert!(
        (fit.standardized[2] - 0.16111).abs() < 0.0005 && fit.t_statistics[3] > 3.0,
        "disadvantage is {:+.5} at t {:+.2}",
        fit.standardized[2],
        fit.t_statistics[3]
    );
    // It is third of three terms that all clear two, which is what a year that raised capacity
    // aid and the money that follows need should look like. The published version could not see
    // it because the adjustment had taken the largest deducts — which belong to the poorest
    // districts — out of the outcome.
    assert!(
        fit.t_statistics[1..=3].iter().all(|t| t.abs() > 2.0),
        "not every term clears two: {:?}",
        fit.t_statistics
    );
    assert!(fit.standardized[2].abs() < fit.standardized[0].abs());
}

/// The mechanism is in a committed greenbook, in the same words the data is in.
#[test]
fn the_greenbook_describes_capacity_aid_as_a_function_of_total_valuation() {
    for phrase in [
        "targets funding to smaller districts with relatively low total property valuation",
        "based on the amount a district can raise with one mill",
        "provided to districts that raise less than the median amount",
    ] {
        assert!(
            project::rating_payments::quotes("hb64", phrase),
            "the greenbook does not carry \"{phrase}\""
        );
    }
}

/// **The third withdrawal.** The comparison that looked like poverty beating size was the deduct.
#[test]
fn among_the_largest_districts_the_published_split_by_poverty_is_the_deduct() {
    let largest = fy2016::largest(14);
    assert_eq!(largest.len(), 14);

    let of = |name: &str| {
        largest
            .iter()
            .find(|d| d.name.starts_with(name))
            .unwrap_or_else(|| panic!("{name} is not among the fourteen largest"))
    };
    let urban = ["Columbus", "Cleveland", "Cincinnati", "Toledo", "Dayton"];
    let suburban = ["Olentangy", "Lakota", "Hilliard", "Dublin", "Mason"];

    // As published, the single-year move splits them exactly as the node said.
    let published = |district: &District| {
        let before =
            fy2016::state_per_pupil(&district.irn, 2015, Basis::AsPublished).expect("FY15");
        fy2016::state_per_pupil(&district.irn, 2016, Basis::AsPublished).expect("FY16") / before
            - 1.0
    };
    for name in urban {
        assert!(
            published(of(name)) < -0.20,
            "{name} moved {:+.4}",
            published(of(name))
        );
    }
    for name in suburban {
        assert!(
            published(of(name)) > -0.07,
            "{name} moved {:+.4}",
            published(of(name))
        );
    }

    // And the split is the deduct, not the need. Poverty separates the two groups, and so does
    // the deduct, and it is the deduct that the measure was charging them.
    let poorest_suburb = suburban
        .iter()
        .map(|n| of(n).disadvantaged)
        .fold(f64::NEG_INFINITY, f64::max);
    let richest_urban = urban
        .iter()
        .map(|n| of(n).disadvantaged)
        .fold(f64::INFINITY, f64::min);
    assert!(
        richest_urban > poorest_suburb * 1.5,
        "the two groups are not separated by need: {richest_urban:.1} against {poorest_suburb:.1}"
    );
    let deduct =
        |name: &str| dispersion::survey_basis::deduct_share(&of(name).irn, 2015).expect("FY2015");
    assert!(
        urban.iter().all(|n| deduct(n) > 0.25) && suburban.iter().all(|n| deduct(n) < 0.06),
        "the deduct does not separate them: urban {:?}, suburban {:?}",
        urban.map(deduct),
        suburban.map(deduct)
    );

    // On one basis the split is gone: every one of the ten gains except Dublin, which loses a
    // third of a per cent.
    for name in urban.iter().chain(suburban.iter()) {
        let step = of(name).step;
        assert!(step > -0.01, "{name} steps {step:+.4}");
    }
}

/// And on one basis the fall the node opened the question about is not there to explain.
#[test]
fn the_urban_fall_does_not_survive_putting_fy2013_and_fy2016_on_one_basis() {
    let moved = |irn: &str, basis| {
        let before = fy2016::state_per_pupil(irn, 2013, basis).expect("FY2013");
        fy2016::state_per_pupil(irn, 2016, basis).expect("FY2016") / before - 1.0
    };

    // (name, IRN, as published, on one basis)
    for (name, irn, was, now) in [
        ("Cleveland", "043786", -0.3304, -0.0598),
        ("Columbus", "043802", -0.2781, 0.1712),
        ("Toledo", "044909", -0.2188, 0.1672),
        ("Cincinnati", "043752", -0.1823, 0.0586),
    ] {
        assert!(
            (moved(irn, Basis::AsPublished) - was).abs() < 0.0005,
            "{name} moved {:+.4} as published",
            moved(irn, Basis::AsPublished)
        );
        assert!(
            (moved(irn, Basis::Gross) - now).abs() < 0.0005,
            "{name} moved {:+.4} on one basis",
            moved(irn, Basis::Gross)
        );
    }
    // Three of the four gain and Cleveland loses six per cent over three years, which is an
    // ordinary number and not a thing to explain. A fifth to a third was the deduct.
    // While a wealthy suburb of the same size gains on both, which is why the contrast read as a
    // finding: only one side of it was being charged a deduct.
    for basis in [Basis::AsPublished, Basis::Gross] {
        assert!(moved("046110", basis) > 0.10, "Lakota should have gained");
    }
}

/// **And it is not a transfer to community schools** — which was right, for the wrong reason.
#[test]
fn nothing_moved_between_districts_and_community_schools_because_nothing_had_to() {
    let sectors = fy2016::by_sector();
    let (district_2015, community_2015, enrolment_2015) = sectors[&2015];
    let (district_2016, community_2016, enrolment_2016) = sectors[&2016];

    // The community half is flat in both money and children.
    assert!(
        (community_2015 / 1e9 - 0.952).abs() < 0.002
            && (community_2016 / 1e9 - 0.938).abs() < 0.002,
        "community state revenue went {:.3}bn to {:.3}bn",
        community_2015 / 1e9,
        community_2016 / 1e9
    );
    assert!(
        (community_2016 / community_2015 - 1.0).abs() < 0.02,
        "the community half moved {:+.4}",
        community_2016 / community_2015 - 1.0
    );
    assert!(enrolment_2016 < enrolment_2015, "and its roll did not grow");

    // This file read the districts' $850m fall against that flat half and concluded the total
    // itself moved. It did — and what left the total is the count, not the money. The fall is
    // the deduct the districts stopped being credited with, and it cannot exceed it.
    let district_move = district_2016 - district_2015;
    let (fall, deduct) = fy2016::fall_against_deduct();
    assert!((district_move - fall).abs() < 1.0);
    assert!(
        fall < 0.0 && fall.abs() < deduct && fall.abs() / deduct > 0.9,
        "the districts' ${:.3}bn fall against a ${:.3}bn deduct",
        fall / 1e9,
        deduct / 1e9
    );
}
