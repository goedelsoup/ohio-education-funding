//! What each priced decline response pays the 64 siblings, and whether a response is for the
//! floor's population or for the districts that lost pupils.
//!
//! Every figure here is a join over pricings that already existed:
//! `anchor_incidence::under_supplement` and `rolling_anchor::walk` produce the per-district arms,
//! `lost_pupils::siblings` and `size_incidence` produce the populations. No lever is priced that
//! was not priced before, and the bands are `anchor_incidence::banded`'s own, so a fifth here is
//! the fifth `who_a_capped_or_rolling_anchor_reaches` reads.

use std::collections::BTreeSet;

use edfund_core::FiscalYear;
use project::anchor_incidence::{self as incidence, Axis, Movement, Supplement};
use project::decline_adjustment::Line;
use project::decline_reach::{self as reach, Reach};
use project::lost_pupils::{self, Population, Standing};
use project::panel::{self, DistrictRecord};
use project::rolling_anchor::Anchor;
use project::series::{Method, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT};
use project::size_incidence;

/// A cent, for a dollar comparison.
const CENT: f64 = 0.01;

/// Five bands, as #424 cut them.
const FIFTHS: usize = 5;

/// The projection the arc ships: the damped shrink `rolling_anchor` defaults to.
fn shipped() -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: DEFAULT_DAMPING,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    }
}

/// The same with the damping off, which is the frame the 1% cap was priced in.
fn undamped() -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: 1.0,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    }
}

/// The enrollment cluster: the 89 the floor caught.
fn cluster(rows: &[Standing]) -> Vec<&Standing> {
    rows.iter()
        .filter(|row| row.population == Population::EnrollmentLoss)
        .collect()
}

fn frames(districts: &[DistrictRecord]) -> Vec<(&'static str, Vec<Movement>)> {
    vec![
        (
            "mirror beside",
            incidence::under_supplement(districts, Supplement::Mirror(Line::Beside)),
        ),
        (
            "mirror inside [H]",
            incidence::under_supplement(districts, Supplement::Mirror(Line::Foundation)),
        ),
        (
            "rolling pupil count",
            incidence::under_supplement(districts, Supplement::RollingCount),
        ),
        (
            "dated phase-down",
            incidence::under_supplement(districts, Supplement::DatedPhaseDown),
        ),
        (
            "ratchet FY2032",
            reach::against_the_enacted_anchor(
                districts,
                FiscalYear(2032),
                shipped(),
                Anchor::PriorYear,
            ),
        ),
        (
            "2% cap FY2032",
            reach::against_the_enacted_anchor(
                districts,
                FiscalYear(2032),
                shipped(),
                Anchor::Decayed { factor: 0.98 },
            ),
        ),
        (
            "1% cap FY2036",
            reach::against_the_enacted_anchor(
                districts,
                FiscalYear(2036),
                undamped(),
                Anchor::Decayed { factor: 0.99 },
            ),
        ),
    ]
}

fn assert_dollars(got: f64, want: f64, what: &str) {
    assert!(
        (got - want).abs() < CENT,
        "{what}: {got:.2} against {want:.2}"
    );
}

/// **The seven instruments priced on the 64 and on the 89.** The table the issue asks for, and
/// the two zeros in it are the answer to the question behind it: a ratchet reaches 59 of the
/// siblings and none of the cluster, a dated phase-down reaches 7 of the cluster and none of the
/// siblings. Nothing is recomputed — each row is `reach::over` filtering a frame that already
/// existed.
#[test]
fn the_seven_instruments_priced_on_the_siblings_and_on_the_cluster() {
    let districts = panel::panel();
    let (rows, _) = lost_pupils::standings(&districts);
    let siblings = reach::irns(&lost_pupils::siblings(&rows));
    let caught = reach::irns(&cluster(&rows));
    assert_eq!(siblings.len(), 64, "the siblings");
    assert_eq!(caught.len(), 89, "the cluster");

    let frames = frames(&districts);
    let want: [(&str, usize, f64, f64, usize, f64, f64); 7] = [
        (
            "mirror beside",
            60,
            19_692_848.91,
            241.95,
            81,
            46_020_643.67,
            233.76,
        ),
        (
            "mirror inside [H]",
            59,
            19_112_356.87,
            234.82,
            14,
            4_195_897.42,
            21.31,
        ),
        (
            "rolling pupil count",
            60,
            22_359_204.12,
            274.71,
            29,
            3_902_794.07,
            19.82,
        ),
        ("dated phase-down", 0, 0.0, 0.00, 7, -695_295.72, -3.53),
        ("ratchet FY2032", 59, 14_794_631.13, 186.81, 0, 0.0, 0.00),
        (
            "2% cap FY2032",
            2,
            -18_947.48,
            -0.24,
            18,
            -3_949_559.44,
            -20.61,
        ),
        (
            "1% cap FY2036",
            40,
            22_526_490.66,
            334.67,
            13,
            -2_590_697.29,
            -15.86,
        ),
    ];
    for (label, moved, dollars, per_pupil, cluster_moved, cluster_dollars, cluster_per_pupil) in
        want
    {
        let frame = &frames.iter().find(|(name, _)| *name == label).unwrap().1;
        let mine = reach::over(frame, &siblings);
        let theirs = reach::over(frame, &caught);
        assert_eq!(mine.districts, 64, "{label}: the siblings are all carried");
        assert_eq!(theirs.districts, 89, "{label}: the cluster is all carried");
        assert_eq!(mine.moved, moved, "{label}: siblings moved");
        assert_eq!(theirs.moved, cluster_moved, "{label}: cluster moved");
        assert_dollars(mine.delta, dollars, &format!("{label}: to the siblings"));
        assert_dollars(
            theirs.delta,
            cluster_dollars,
            &format!("{label}: to the cluster"),
        );
        assert!(
            (mine.per_pupil() - per_pupil).abs() < CENT,
            "{label}: {:.2} a pupil to the siblings against {per_pupil:.2}",
            mine.per_pupil()
        );
        assert!(
            (theirs.per_pupil() - cluster_per_pupil).abs() < CENT,
            "{label}: {:.2} a pupil to the cluster against {cluster_per_pupil:.2}",
            theirs.per_pupil()
        );
    }
}

/// **The two zeros are structural, not small.** A ratchet pays the cluster nothing because the
/// enacted guarantee already holds every one of the 89 and a second floor under a floor is
/// slack; a dated phase-down pays the siblings nothing because it rolls a list drawn on a date
/// and they are not on it. Neither number is a near-miss: both are exactly zero, and no district
/// of the blind population moves at all.
#[test]
fn a_ratchet_and_a_dated_phase_down_are_each_blind_to_one_of_the_two_populations() {
    let districts = panel::panel();
    let (rows, _) = lost_pupils::standings(&districts);
    let siblings = reach::irns(&lost_pupils::siblings(&rows));
    let caught = reach::irns(&cluster(&rows));
    let frames = frames(&districts);
    let frame = |label: &str| {
        frames
            .iter()
            .find(|(name, _)| *name == label)
            .map(|(_, movements)| movements.clone())
            .unwrap()
    };

    let ratchet = reach::over(&frame("ratchet FY2032"), &caught);
    assert_eq!(ratchet.moved, 0, "the ratchet moves a cluster district");
    assert_eq!(ratchet.gainers + ratchet.losers, 0);
    assert!(ratchet.delta.abs() < CENT, "{:.4}", ratchet.delta);

    let dated = reach::over(&frame("dated phase-down"), &siblings);
    assert_eq!(dated.moved, 0, "the dated phase-down moves a sibling");
    assert!(dated.delta.abs() < CENT, "{:.4}", dated.delta);

    // And the family that is keyed to pupils rather than to a floor reaches both.
    for label in ["mirror beside", "rolling pupil count"] {
        let movements = frame(label);
        assert!(reach::over(&movements, &siblings).moved >= 60, "{label}");
        assert!(reach::over(&movements, &caught).moved >= 29, "{label}");
    }
}

/// **The siblings, district by district, before any fifth.** The issue asks for this order
/// because the 64 are small and a band's per-pupil figure hides them. Four of the 64 the rolling
/// count does not move at all, and the largest single payment is Maple Heights City's
/// $1,780,414 — a 2,612-pupil district. Under the mirror the order is district size instead,
/// because that shape pays $250 on every enrolled pupil of an eligible district rather than on
/// the pupils it lost, and 56 of the 64 receive the rate exactly.
#[test]
fn the_largest_sibling_payments_are_small_districts_and_four_are_not_reached() {
    let districts = panel::panel();
    let (rows, _) = lost_pupils::standings(&districts);
    let siblings = reach::irns(&lost_pupils::siblings(&rows));
    let frames = frames(&districts);
    let rolling = &frames
        .iter()
        .find(|(name, _)| *name == "rolling pupil count")
        .unwrap()
        .1;

    let ordered = reach::per_district(rolling, &siblings);
    assert_eq!(ordered.len(), 64);
    let top: Vec<&str> = ordered.iter().take(3).map(|m| m.name.as_str()).collect();
    assert_eq!(
        top,
        ["Maple Heights City", "Mansfield City", "Euclid City"],
        "the three largest sibling payments under the rolling count"
    );
    assert_dollars(
        ordered[0].delta(),
        1_780_413.91,
        "Maple Heights City under the rolling count",
    );
    let unmoved: Vec<&str> = ordered
        .iter()
        .filter(|m| m.delta().abs() <= CENT)
        .map(|m| m.name.as_str())
        .collect();
    assert_eq!(
        unmoved,
        [
            "St Bernard-Elmwood Place City",
            "McComb Local",
            "Western Reserve Local",
            "Liberty Local"
        ],
        "the siblings a rolling count does not reach"
    );

    // The mirror pays $250 on every enrolled pupil of an eligible district, not on the pupils it
    // lost, so under it the order is district size and the rate is flat.
    let inside = &frames
        .iter()
        .find(|(name, _)| *name == "mirror inside [H]")
        .unwrap()
        .1;
    let by_size = reach::per_district(inside, &siblings);
    assert_eq!(by_size[0].name, "Newark City");
    let flat = by_size
        .iter()
        .filter(|m| (m.per_pupil() - 250.00).abs() < CENT)
        .count();
    assert_eq!(flat, 56, "siblings paid the rate exactly");
}

/// **How much of #424's progressive gradient lands on the 64.** Roughly a third of the
/// least-wealthy fifth's dollars, and the gradient survives their removal: take the siblings out
/// of the rolling count's bands and the poorest fifth still pays $119.29 a pupil against the
/// wealthiest's $8.76. The bands are `anchor_incidence::banded`'s, and the splits sum back to
/// `anchor_incidence::by`'s bands, so this is a cut of #424's own figures and not a second
/// measurement of them.
#[test]
fn the_gradient_is_not_mostly_the_siblings_and_outlives_taking_them_out() {
    let districts = panel::panel();
    let (rows, _) = lost_pupils::standings(&districts);
    let siblings = reach::irns(&lost_pupils::siblings(&rows));
    let mine = reach::irns(&cluster(&rows));

    let rolling = incidence::under_supplement(&districts, Supplement::RollingCount);
    let bands = incidence::by(&rolling, Axis::Valuation, FIFTHS, &mine);
    let splits = reach::split_by(&rolling, Axis::Valuation, FIFTHS, &mine, &siblings);
    for (band, split) in bands.iter().zip(&splits) {
        assert_eq!(band.districts, split.districts, "the bands disagree");
        assert_dollars(band.delta, split.delta, "the bands' dollars disagree");
    }

    let counts: Vec<usize> = splits.iter().map(|s| s.inside).collect();
    assert_eq!(
        counts,
        [22, 17, 11, 6, 7],
        "the siblings themselves sit at the poor end"
    );
    assert_dollars(
        splits[0].inside_delta,
        13_135_541.49,
        "the siblings' share of the least-wealthy fifth",
    );
    assert_dollars(
        splits[0].delta,
        34_075_370.14,
        "the least-wealthy fifth's dollars",
    );
    assert!(
        (splits[0].inside_share() - 0.38549).abs() < 0.00001,
        "{:.4}",
        splits[0].inside_share()
    );

    let outside: Vec<f64> = splits
        .iter()
        .map(|s| (s.outside_per_pupil() * 100.0).round() / 100.0)
        .collect();
    assert_eq!(
        outside,
        [119.29, 64.63, 27.14, 8.06, 8.76],
        "the rolling count by valuation with the siblings taken out"
    );
    for pair in outside[..4].windows(2) {
        assert!(pair[0] > pair[1], "the gradient reverses: {pair:?}");
    }

    // The mirror inside [H] pays the siblings flat, so its whole gradient is everyone else.
    let inside = incidence::under_supplement(&districts, Supplement::Mirror(Line::Foundation));
    let splits = reach::split_by(&inside, Axis::Valuation, FIFTHS, &mine, &siblings);
    let theirs: Vec<f64> = splits
        .iter()
        .map(|s| (s.inside_per_pupil() * 100.0).round() / 100.0)
        .collect();
    assert_eq!(theirs, [244.06, 247.35, 222.89, 189.68, 242.77]);
    let others: Vec<f64> = splits
        .iter()
        .map(|s| (s.outside_per_pupil() * 100.0).round() / 100.0)
        .collect();
    assert_eq!(others, [92.73, 66.50, 39.36, 8.57, 12.93]);
    assert!(
        theirs.iter().cloned().fold(0.0, f64::max) / theirs.iter().cloned().fold(1e9, f64::min)
            < 1.4,
        "the siblings are not paid flat: {theirs:?}"
    );
}

/// **A mirror inside `[H]` is self-extinguishing for the siblings, and the date arrives first.**
/// Absorption on the 64 is 0.0295 against the cluster's 0.9088, and inside the 64 it is 0.0789
/// on the half nearest the floor against 0.0000 on the half furthest — the shape pays a district
/// until the district arrives. The median sibling arrives about FY2031, four years after the
/// FY2027 date R.C. 3317.019 sets, which is why a dated phase-down reaches none of them.
#[test]
fn the_shape_paid_inside_the_foundation_line_expires_as_the_siblings_reach_the_floor() {
    let districts = panel::panel();
    let (rows, _) = lost_pupils::standings(&districts);
    let siblings = reach::irns(&lost_pupils::siblings(&rows));
    let caught = reach::irns(&cluster(&rows));

    let mine = reach::absorption(&districts, Line::Foundation, &siblings);
    let theirs = reach::absorption(&districts, Line::Foundation, &caught);
    assert_eq!((mine.paid, theirs.paid), (60, 81));
    assert!(
        (mine.absorbed_share() - 0.029477).abs() < 0.000001,
        "{mine:?}"
    );
    assert!(
        (theirs.absorbed_share() - 0.908826).abs() < 0.000001,
        "{theirs:?}"
    );
    assert_eq!(mine.reduced, 4, "siblings that lose anything inside [H]");
    assert_eq!(theirs.extinguished, 67, "cluster districts paid nothing");

    // Beside the floors nothing is absorbed, for either population, by construction.
    for population in [&siblings, &caught] {
        let beside = reach::absorption(&districts, Line::Beside, population);
        assert!(beside.absorbed_share().abs() < 1e-9);
        assert_eq!(beside.reduced, 0);
    }

    let (nearer, further) = reach::by_headroom(&districts, Line::Foundation, &siblings);
    assert!(
        nearer.absorbed_share() > further.absorbed_share(),
        "absorption does not rise as the floor nears: {:.4} against {:.4}",
        nearer.absorbed_share(),
        further.absorbed_share()
    );
    assert!(further.absorbed_share().abs() < 1e-9, "{further:?}");

    let clock = reach::clock(&districts, &siblings);
    assert_eq!((clock.falling, clock.never), (62, 2));
    assert!((clock.median_years - 4.14).abs() < 0.005, "{clock:?}");
    assert_eq!(clock.within, (31, 40));

    // Under the shipped damping they mostly never arrive; undamped, most do.
    assert_eq!(
        reach::on_the_floor(&districts, FiscalYear(2036), shipped(), &siblings),
        5
    );
    assert_eq!(
        reach::on_the_floor(&districts, FiscalYear(2036), undamped(), &siblings),
        38
    );
    assert_eq!(
        reach::on_the_floor(&districts, FiscalYear(2036), shipped(), &caught),
        89,
        "the cluster is on the floor in every year"
    );
}

/// **The union and the rate-defined population, both, because they are not the same 153.** The
/// 153 is `cluster_and_siblings`, a union; the 169 is `shrinking_at_the_cluster_rate`, which is
/// what the phrase "districts shrinking at the cluster's rate" actually names. A shape's
/// per-pupil figure differs between them, so the distinction is load-bearing here and not only
/// in the size arc.
#[test]
fn the_union_of_the_two_populations_and_the_rate_defined_one_price_differently() {
    let districts = panel::panel();
    let (rows, _) = lost_pupils::standings(&districts);
    let union = reach::irns(&size_incidence::cluster_and_siblings(&rows));
    let rate = reach::irns(&size_incidence::shrinking_at_the_cluster_rate(&rows));
    assert_eq!(union.len(), 153);
    assert_eq!(rate.len(), 169);
    assert!(
        union.difference(&rate).count() > 0 && rate.difference(&union).count() > 0,
        "neither population contains the other"
    );

    let frames = frames(&districts);
    let priced = |label: &str, population: &BTreeSet<String>| -> Reach {
        let frame = &frames.iter().find(|(name, _)| *name == label).unwrap().1;
        reach::over(frame, population)
    };
    for (label, on_union, on_rate) in [
        ("mirror beside", 236.16, 243.96),
        ("rolling pupil count", 94.38, 95.45),
        ("ratchet FY2032", 54.63, 53.28),
    ] {
        let one = priced(label, &union).per_pupil();
        let two = priced(label, &rate).per_pupil();
        assert!((one - on_union).abs() < CENT, "{label}: {one:.2}");
        assert!((two - on_rate).abs() < CENT, "{label}: {two:.2}");
    }
    assert_dollars(
        priced("mirror beside", &union).delta,
        65_713_492.58,
        "the mirror beside, over the 153",
    );
    assert_dollars(
        priced("mirror beside", &rate).delta,
        69_915_878.09,
        "the mirror beside, over the 169",
    );
}
