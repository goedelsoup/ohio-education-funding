//! The FY2016 question `education-agency/toledo-city` opened, and the guess it opened with.
//!
//! The node recorded the tail as "mostly small districts with large industrial tax bases — which
//! is the profile of a district losing tangible personal property revenue", marked the cause
//! `[open]`, and pointed at `revenue-stream/tpp-replacement-payments`.
//!
//! # The guess is refuted and the answer is a different formula
//!
//! There is no relationship to industrial property at all. What the step does track is a
//! district's **total** assessed value — the quantity one mill's yield is proportional to — and
//! FY2016 is the first year of H.B. 64's capacity aid, which LSC's greenbook describes as
//! targeting "smaller districts with relatively low total property valuation" on "the amount a
//! district can raise with one mill".
//!
//! # And the tail is a second thing capacity aid cannot do
//!
//! Capacity aid adds money. Among the fourteen largest districts the fall is by poverty and not by
//! size: the five big-city districts lose 21.8% to 38.8% while five wealthy suburbs of the same
//! size lose under 6%. It survives measuring from FY2013 instead of the anomalous FY2015, and it
//! is not a transfer to community schools, whose state revenue is flat across the break.

use dispersion::fy2016::{self, District};

/// **The withdrawal.** The step has no relationship to industrial property.
#[test]
fn the_step_does_not_track_industrial_property_at_all() {
    let industrial = fy2016::against(|d| d.industrial_share);
    let business = fy2016::against(|d| d.business_share);
    assert!(
        (industrial - 0.090).abs() < 0.005 && (business - 0.035).abs() < 0.005,
        "industrial {industrial:+.4}, business {business:+.4}"
    );
    // Both are the wrong sign for the tangible-personal-property reading, which needs
    // industrial districts to have fallen.
    assert!(
        industrial > 0.0 && business > 0.0,
        "an industry-heavy district fell *less*, not more"
    );

    // And the gradient is flat, so the near-zero correlation is not two tails cancelling.
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

    // The deep fallers are, if anything, *less* industrial than everyone else.
    let districts = fy2016::frame();
    let mean = |slice: Vec<&District>| {
        slice.iter().map(|d| d.business_share).sum::<f64>() / slice.len() as f64
    };
    let deep = mean(districts.iter().filter(|d| d.step < -0.20).collect());
    let rest = mean(districts.iter().filter(|d| d.step >= -0.20).collect());
    assert!(
        deep < rest,
        "deep fallers carry {deep:.4} against everyone else's {rest:.4}"
    );
}

/// **The finding.** What it tracks is total valuation, which is what capacity aid keys on.
#[test]
fn the_step_tracks_total_valuation_and_the_gradient_is_monotone() {
    let total = fy2016::against(|d| d.total_valuation.ln());
    assert!(
        (total + 0.342).abs() < 0.005,
        "the step against log total valuation is {total:+.4}"
    );
    assert!(
        total < fy2016::against(|d| d.valuation_per_pupil),
        "total valuation should beat wealth per pupil"
    );

    // Smallest total valuation first: every fifth gains, and each gains less than the one below.
    let gradient = fy2016::quintiles_by(|d| d.total_valuation);
    assert!(
        (gradient[0] - 0.241).abs() < 0.005 && (gradient[4] - 0.042).abs() < 0.005,
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

    // And in one model it is the term that carries it.
    let fit = fy2016::model();
    assert!(
        (fit.standardized[0] + 0.291).abs() < 0.006 && fit.t_statistics[1] < -6.0,
        "total valuation is {:+.4} at t {:+.2}",
        fit.standardized[0],
        fit.t_statistics[1]
    );
    assert!(
        fit.standardized[0].abs() > fit.standardized[1].abs() * 2.0,
        "wealth per pupil is {:+.4} against total valuation's {:+.4}",
        fit.standardized[1],
        fit.standardized[0]
    );
    assert!(
        fit.t_statistics[3].abs() < 2.0,
        "the disadvantage term reaches t {:+.2}, so poverty is doing work here after all",
        fit.t_statistics[3]
    );
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

/// The tail is the other half, and capacity aid cannot produce it.
#[test]
fn among_the_largest_districts_the_fall_is_by_poverty_and_not_by_size() {
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

    // The single-year move, which is the one the node states.
    let single = |district: &District| {
        let before = fy2016::state_per_pupil(&district.irn, 2015).expect("FY2015");
        fy2016::state_per_pupil(&district.irn, 2016).expect("FY2016") / before - 1.0
    };
    for name in urban {
        assert!(
            single(of(name)) < -0.20,
            "{name} moved {:+.4}",
            single(of(name))
        );
    }
    for name in suburban {
        assert!(
            single(of(name)) > -0.07,
            "{name} moved {:+.4}",
            single(of(name))
        );
    }

    // Poverty separates them and size does not: every one of these ten is in the largest
    // fourteen, so size is held roughly fixed by construction.
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
}

/// And it is not the FY2015 bump doing the work.
#[test]
fn the_urban_fall_survives_measuring_from_fy2013() {
    let from_2013 = |irn: &str| {
        let before = fy2016::state_per_pupil(irn, 2013).expect("FY2013");
        fy2016::state_per_pupil(irn, 2016).expect("FY2016") / before - 1.0
    };
    let sectors = fy2016::by_sector();
    let panel_pupils = |year: u16| {
        dispersion::ohio_panel::panel()
            .iter()
            .filter(|r| r.comparable && r.fiscal_year == year)
            .map(|r| r.enrollment)
            .sum::<f64>()
    };
    let statewide = |year: u16| sectors[&year].0 / panel_pupils(year);

    // The state as a whole gains between FY2013 and FY2016.
    let state_move = statewide(2016) / statewide(2013) - 1.0;
    assert!(
        (state_move - 0.054).abs() < 0.005,
        "statewide state revenue per pupil moved {state_move:+.4}"
    );

    // And the big-city districts lose a fifth to a third against it.
    for (name, irn, floor) in [
        ("Cleveland", "043786", -0.32),
        ("Columbus", "043802", -0.27),
        ("Toledo", "044909", -0.21),
        ("Cincinnati", "043752", -0.18),
    ] {
        let moved = from_2013(irn);
        assert!(
            moved < floor,
            "{name} moved {moved:+.4}, which no longer clears {floor}"
        );
    }
    // While a wealthy suburb of the same size gains.
    assert!(from_2013("046110") > 0.10, "Lakota should have gained");
}

/// **And it is not a transfer to community schools**, which the deduct mechanism invites.
#[test]
fn nothing_moved_between_districts_and_community_schools_across_the_break() {
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

    // Districts fell by $850m and the community schools did not receive it: the two halves
    // summed move together, so the total itself moved.
    let district_move = district_2016 - district_2015;
    assert!(
        district_move / 1e9 < -0.8,
        "districts moved {:.3}bn",
        district_move / 1e9
    );
    let community_move = community_2016 - community_2015;
    assert!(
        community_move.abs() < district_move.abs() * 0.05,
        "the community half moved {:.3}bn against the districts' {:.3}bn",
        community_move / 1e9,
        district_move / 1e9
    );
}
