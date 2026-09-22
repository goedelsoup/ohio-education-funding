//! Who a capped or rolling anchor reaches outside the enrollment cluster, and whether #400's
//! shapes reach the same districts.
//!
//! [#424](https://github.com/goedelsoup/ohio-education-funding/issues/424) recorded the gap #390
//! and #400 left twice: both priced their instruments statewide and against the **89** of
//! `enrollment_decline`'s cluster, and neither said who else the money reaches. Between 157 and
//! 253 districts change standing depending on the anchor rule, and at FY2036 a 1% cap holds more
//! districts than the enacted anchor while writing less guarantee and raising total state support.
//!
//! [`project::anchor_incidence`] answers it, on total state support, outside the cluster, by
//! valuation per pupil and by economically disadvantaged share.
//!
//! # The enacted anchor's population is the wealthy, so every alternative runs the other way
//!
//! Outside the cluster the FY2027 guarantee holds none of the 103 least-wealthy districts and 77
//! of the 105 wealthiest. Every rule here is a change from that one, so a rule that narrows the
//! floor cuts the wealthy and a rule that reaches past it pays the poor. A **ratchet** adds
//! $138.58 a pupil to the least-wealthy fifth and $3.75 to the wealthiest; a **2% cap** takes
//! nothing from the least-wealthy fifth and $83.51 a pupil from the fourth; the **1% cap at
//! FY2036** does both — +$141.73 at the bottom, −$55.40 at the top — and its $7.7m aggregate
//! increase is 112 districts gaining $54.0m against 119 losing $43.7m.
//!
//! # `[K]` has a gradient of its own
//!
//! The 2% cap takes $24.7m of guarantee off the poorest fifth and $1.6m of total state support:
//! 93.4% of the cut comes back on `[K]`, against 35.9% in the least-poor fifth. Incidence on `[I]`
//! is not incidence, which is the constraint #424 set and the reason the measure here is the wide
//! one.
//!
//! # And #400's shapes agree
//!
//! The rolling count and the mirror inside `[H]` are progressive by the same mechanism; the
//! mirror beside every floor is flat; the dated phase-down touches nobody outside the cluster.
//! No shape is regressive against the enacted anchor. The choice between them is the sign and
//! the size, not the direction — which is the fiscal framing both issues used.

use edfund_core::FiscalYear;
use project::anchor_incidence::{self as incidence, Axis, Band, Movement, Supplement};
use project::decline_adjustment::{self, Line};
use project::enrollment_decline;
use project::panel::{self, DistrictRecord, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL as RATE};
use project::rolling_anchor::{self, Anchor};
use project::series::{Method, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT};
use std::collections::BTreeSet;

/// A cent, the tolerance every dollar figure here is asserted at.
const CENT: f64 = 0.01;

/// The bands every cut is taken in, on both axes, so the two tables are comparable.
const FIFTHS: usize = 5;

/// The shipped projection, as `the_anchor_that_was_supposed_to_move_with_the_district` builds it.
fn shipped() -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: DEFAULT_DAMPING,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    }
}

/// The same with the damping off — the honest end of the drift, and where the sign flips.
fn undamped() -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: 1.0,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    }
}

/// The 89, as the set every cut leaves out.
fn cluster(districts: &[DistrictRecord]) -> BTreeSet<String> {
    enrollment_decline::cluster(districts)
        .into_iter()
        .map(|record| record.irn.clone())
        .collect()
}

/// One alternative against the enacted anchor, walked to a year, district by district.
fn against_enacted(
    districts: &[DistrictRecord],
    through: u16,
    method: Method,
    anchor: Anchor,
) -> Vec<Movement> {
    let year = FiscalYear(through);
    let enacted = rolling_anchor::walk(districts, year, method, Anchor::Fixed);
    let alternative = rolling_anchor::walk(districts, year, method, anchor);
    incidence::between_walks(districts, &enacted, &alternative)
}

/// Per-pupil change in each fifth, lowest first.
fn per_pupil(bands: &[Band]) -> Vec<f64> {
    bands.iter().map(Band::per_pupil).collect()
}

/// The per-pupil figures agree with the ones the module docs state, to the cent.
fn assert_per_pupil(bands: &[Band], expected: [f64; FIFTHS], what: &str) {
    for (band, want) in bands.iter().zip(expected) {
        assert!(
            (band.per_pupil() - want).abs() < CENT,
            "{what}, fifth {}: ${:.2} a pupil against ${want:.2}",
            band.rank,
            band.per_pupil()
        );
    }
}

/// **The ground.** Outside the cluster, the enacted floor holds the wealthy and not the poor.
///
/// The corpus's 13%, 37%, 68%, 76% quartile gradient with the enrollment cluster taken out, and
/// the fact every other test here rests on: an alternative rule is a change from this one.
#[test]
fn the_enacted_anchor_holds_none_of_the_least_wealthy_fifth_and_most_of_the_wealthiest() {
    let districts = panel::panel();
    let mine = cluster(&districts);
    // Any supplement frame carries current law as its control; the dated phase-down changes the
    // fewest districts, so it is the cleanest reading of it.
    let frame = incidence::under_supplement(&districts, Supplement::DatedPhaseDown);
    let by_wealth = incidence::by(&frame, Axis::Valuation, FIFTHS, &mine);

    let held: Vec<usize> = by_wealth.iter().map(|band| band.held_as_enacted).collect();
    assert_eq!(
        held,
        vec![0, 7, 44, 77, 77],
        "held as enacted, least wealthy first"
    );
    assert_eq!(by_wealth[0].districts, 103);
    assert_eq!(by_wealth[4].districts, 105);
    assert_eq!(
        held.iter().sum::<usize>(),
        294 - 89,
        "the 294 guaranteed districts less the cluster"
    );

    let by_poverty = incidence::by(&frame, Axis::Poverty, FIFTHS, &mine);
    let held: Vec<usize> = by_poverty.iter().map(|band| band.held_as_enacted).collect();
    assert_eq!(
        held,
        vec![56, 59, 54, 25, 11],
        "held as enacted, least poor first"
    );
    assert_eq!(
        by_poverty.iter().map(|band| band.districts).sum::<usize>(),
        520,
        "609 less the 89, every one of which publishes a DPIA percentage"
    );
}

/// **The identity.** Each supplement's per-district movements sum to the aggregate #400 published.
///
/// What licenses reading a cut of them as a cut of the same quantity: `mirror`, `rolling_count`
/// and `dated_phase_down` are reproduced to the cent from the per-district arms, statewide and
/// over the cluster.
#[test]
fn every_supplements_movements_sum_to_the_aggregate_it_was_priced_at() {
    let districts = panel::panel();
    let mine = cluster(&districts);
    let total = |frame: &[Movement]| -> f64 { frame.iter().map(Movement::delta).sum() };
    let over_cluster = |frame: &[Movement]| -> f64 {
        frame
            .iter()
            .filter(|movement| mine.contains(&movement.irn))
            .map(Movement::delta)
            .sum()
    };

    for line in [Line::Beside, Line::Foundation] {
        let published = decline_adjustment::mirror(&districts, RATE, line);
        let frame = incidence::under_supplement(&districts, Supplement::Mirror(line));
        assert!(
            (total(&frame) - published.statewide).abs() < CENT,
            "{}: {:.2} against {:.2}",
            line.label(),
            total(&frame),
            published.statewide
        );
        assert!((over_cluster(&frame) - published.cluster).abs() < CENT);
    }

    let published = decline_adjustment::rolling_count(&districts);
    let frame = incidence::under_supplement(&districts, Supplement::RollingCount);
    assert!((total(&frame) - published.statewide).abs() < CENT);
    assert!((over_cluster(&frame) - published.cluster).abs() < CENT);
    assert!((total(&frame) - 70_105_642.08).abs() < CENT);

    let published =
        decline_adjustment::dated_phase_down(&districts, project::policy::GuaranteeRule::Removed);
    let frame = incidence::under_supplement(&districts, Supplement::DatedPhaseDown);
    assert!((total(&frame) + published.actual).abs() < CENT);
    assert!(
        (over_cluster(&frame) - total(&frame)).abs() < CENT,
        "a dated phase-down of the cluster moves nobody outside it"
    );
}

/// **A ratchet adds money, and adds it at the bottom.**
///
/// A prior-year anchor catches every district whose formula amount falls, and the districts with
/// the most formula aid to lose are the poor ones. Monotone on both axes, nobody loses, and the
/// gain is broad.
#[test]
fn a_ratchet_pays_the_least_wealthy_fifth_thirty_seven_times_what_it_pays_the_wealthiest() {
    let districts = panel::panel();
    let mine = cluster(&districts);
    let frame = against_enacted(&districts, 2032, shipped(), Anchor::PriorYear);

    let by_wealth = incidence::by(&frame, Axis::Valuation, FIFTHS, &mine);
    assert_per_pupil(
        &by_wealth,
        [138.58, 77.16, 26.33, 5.15, 3.75],
        "a ratchet by valuation",
    );
    for pair in per_pupil(&by_wealth).windows(2) {
        assert!(pair[1] < pair[0], "the gradient reverses: {pair:?}");
    }
    assert!(by_wealth[0].per_pupil() / by_wealth[4].per_pupil() > 35.0);
    let enters: Vec<usize> = by_wealth.iter().map(|band| band.enters).collect();
    assert_eq!(
        enters,
        vec![91, 76, 46, 18, 20],
        "districts a ratchet newly holds"
    );
    assert!(by_wealth
        .iter()
        .all(|band| band.leaves == 0 && band.losers == 0));

    let by_poverty = incidence::by(&frame, Axis::Poverty, FIFTHS, &mine);
    assert_per_pupil(
        &by_poverty,
        [4.06, 11.34, 31.72, 51.87, 108.08],
        "a ratchet by disadvantaged share",
    );
    for pair in per_pupil(&by_poverty).windows(2) {
        assert!(pair[1] > pair[0], "the gradient reverses: {pair:?}");
    }

    let spread = incidence::spread(&frame, &mine);
    assert_eq!(spread.gainers, 253);
    assert_eq!(spread.losers, 0);
    assert!(
        (spread.gained - 52_907_613.45).abs() < CENT,
        "{:.2}",
        spread.gained
    );
    assert!(
        (spread.top_ten_share_of_gains - 0.2578).abs() < 0.0005,
        "{:.4}",
        spread.top_ten_share_of_gains
    );
    assert_eq!(spread.largest_gain.0, "Toledo City");
}

/// **A cap takes money, and takes it at the top — and `[K]` decides how much of it lands.**
///
/// A cap lets a held district's floor decline, so it is a cut to the districts the enacted
/// anchor holds. Nothing comes off the least-wealthy fifth, whose two held districts are fully
/// backstopped; the fourth fifth loses $83.51 a pupil. By poverty the poorest fifth's guarantee
/// falls $24.7m and its total state support $1.6m.
#[test]
fn a_cap_cuts_the_wealthy_and_the_backstop_returns_the_poorest_fifths_share() {
    let districts = panel::panel();
    let mine = cluster(&districts);
    let frame = against_enacted(
        &districts,
        2032,
        shipped(),
        Anchor::Decayed { factor: 0.98 },
    );

    let by_wealth = incidence::by(&frame, Axis::Valuation, FIFTHS, &mine);
    assert_per_pupil(
        &by_wealth,
        [0.0, -2.80, -62.88, -83.51, -77.00],
        "a 2% cap by valuation",
    );
    assert!(
        by_wealth[0].delta.abs() < CENT,
        "the least-wealthy fifth loses ${:.2} in total",
        by_wealth[0].delta
    );
    assert!(
        (by_wealth[0].guarantee_delta + 1_898_609.44).abs() < CENT,
        "having lost ${:.2} of guarantee",
        -by_wealth[0].guarantee_delta
    );
    assert!(
        (by_wealth[0].absorbed_share() - 1.0).abs() < 1e-9,
        "every dollar of which [K] returns"
    );
    assert_eq!(by_wealth[0].held_as_enacted, 2);

    let by_poverty = incidence::by(&frame, Axis::Poverty, FIFTHS, &mine);
    assert_per_pupil(
        &by_poverty,
        [-83.84, -101.52, -40.02, -21.98, -5.33],
        "a 2% cap by disadvantaged share",
    );
    let poorest = &by_poverty[4];
    let least_poor = &by_poverty[0];
    assert!((poorest.guarantee_delta + 24_704_573.54).abs() < CENT);
    assert!((poorest.delta + 1_630_876.47).abs() < CENT);
    assert!(
        (poorest.absorbed_share() - 0.9340).abs() < 0.0005,
        "the poorest fifth: {:.4} of the cut returned on [K]",
        poorest.absorbed_share()
    );
    assert!(
        (least_poor.absorbed_share() - 0.3589).abs() < 0.0005,
        "the least-poor fifth: {:.4}",
        least_poor.absorbed_share()
    );
    assert!(
        poorest.absorbed_share() > least_poor.absorbed_share() * 2.5,
        "the backstop catches the poor and not the wealthy"
    );

    let spread = incidence::spread(&frame, &mine);
    assert_eq!(spread.losers, 145);
    assert_eq!(spread.gainers, 1);
    assert_eq!(spread.largest_gain.0, "Kelleys Island Local");
    assert!(
        (spread.lost - 59_368_095.59).abs() < CENT,
        "{:.2}",
        spread.lost
    );
    assert_eq!(spread.largest_loss.0, "Lakota Local");
    let leaves: usize = by_wealth.iter().map(|band| band.leaves).sum();
    assert_eq!(
        leaves, 25,
        "districts the cap stops holding, outside the cluster"
    );
}

/// **The 1% cap's aggregate increase is a transfer, broad on both sides.**
///
/// At FY2036 undamped the cap holds more districts than the enacted anchor for less guarantee and
/// more total state support. The increase is 112 districts gaining $54.0m against 119 losing
/// $43.7m: the poor gain and the wealthy lose, and the gain is not a few large districts.
#[test]
fn the_gentle_caps_aggregate_increase_is_the_poor_gaining_more_than_the_wealthy_lose() {
    let districts = panel::panel();
    let mine = cluster(&districts);
    let frame = against_enacted(
        &districts,
        2036,
        undamped(),
        Anchor::Decayed { factor: 0.99 },
    );

    // The statewide sign flip #390 found, recovered as the sum of every district's movement.
    let statewide: f64 = frame.iter().map(Movement::delta).sum();
    assert!(
        (statewide - 7_696_746.34).abs() < CENT,
        "total state support is ${statewide:.2} higher under the 1% cap"
    );

    let by_wealth = incidence::by(&frame, Axis::Valuation, FIFTHS, &mine);
    assert_per_pupil(
        &by_wealth,
        [141.73, 92.63, -24.50, -56.69, -55.40],
        "a 1% cap at FY2036 by valuation",
    );
    assert!(by_wealth[0].per_pupil() > 0.0 && by_wealth[4].per_pupil() < 0.0);

    let by_poverty = incidence::by(&frame, Axis::Poverty, FIFTHS, &mine);
    assert_per_pupil(
        &by_poverty,
        [-68.15, -64.12, 8.73, 40.08, 123.40],
        "a 1% cap at FY2036 by disadvantaged share",
    );
    assert!(by_poverty[4].per_pupil() > 0.0 && by_poverty[0].per_pupil() < 0.0);

    let spread = incidence::spread(&frame, &mine);
    assert_eq!(spread.gainers, 112);
    assert_eq!(spread.losers, 119);
    assert!(
        (spread.gained - 53_969_540.19).abs() < CENT,
        "{:.2}",
        spread.gained
    );
    assert!(
        (spread.lost - 43_682_096.56).abs() < CENT,
        "{:.2}",
        spread.lost
    );
    assert!(
        spread.gained > spread.net() * 5.0 && spread.lost > spread.net() * 4.0,
        "the net is the difference of two numbers several times its size"
    );
    assert!(
        (spread.top_ten_share_of_gains - 0.3649).abs() < 0.0005,
        "the ten largest gainers hold {:.4} of the gain",
        spread.top_ten_share_of_gains
    );
    assert_eq!(spread.largest_gain.0, "Maple Heights City");
    assert!((spread.largest_gain.1 - 3_786_662.30).abs() < CENT);
    assert_eq!(spread.largest_loss.0, "Lakota Local");
    assert_eq!(
        by_wealth[0].gainers + by_wealth[1].gainers,
        77,
        "of the 112 gainers, in the two least-wealthy fifths"
    );

    // At 2% the same rule is a net cut again: the rate decides the sign and never the direction.
    let harder = against_enacted(
        &districts,
        2036,
        undamped(),
        Anchor::Decayed { factor: 0.98 },
    );
    let spread = incidence::spread(&harder, &mine);
    assert!(spread.net() < 0.0, "{:.2}", spread.net());
    assert!((spread.gained - 5_002_227.20).abs() < CENT);
    assert!((spread.lost - 54_559_220.13).abs() < CENT);
    let by_wealth = incidence::by(&harder, Axis::Valuation, FIFTHS, &mine);
    assert!(
        by_wealth[0].per_pupil() > 0.0 && by_wealth[4].per_pupil() < -80.0,
        "still the poor up and the wealthy down: {:?}",
        per_pupil(&by_wealth)
    );
}

/// **#400's shapes agree with this one.** None is regressive against the enacted anchor.
///
/// The rolling count and the mirror inside `[H]` are progressive for the reason the ratchet is —
/// the floors absorb them where the wealthy are held. The mirror beside every floor is the one
/// shape with no gradient, and the dated phase-down reaches nobody outside the cluster.
#[test]
fn the_four_supplements_run_the_same_way_or_flat_and_none_runs_the_other_way() {
    let districts = panel::panel();
    let mine = cluster(&districts);

    let rolling = incidence::under_supplement(&districts, Supplement::RollingCount);
    let by_wealth = incidence::by(&rolling, Axis::Valuation, FIFTHS, &mine);
    assert_per_pupil(
        &by_wealth,
        [166.55, 83.44, 35.67, 10.04, 11.43],
        "the rolling count by valuation",
    );
    let by_poverty = incidence::by(&rolling, Axis::Poverty, FIFTHS, &mine);
    assert_per_pupil(
        &by_poverty,
        [11.68, 21.14, 49.10, 73.67, 109.79],
        "the rolling count by disadvantaged share",
    );
    for pair in per_pupil(&by_poverty).windows(2) {
        assert!(pair[1] > pair[0], "the gradient reverses: {pair:?}");
    }

    let inside = incidence::under_supplement(&districts, Supplement::Mirror(Line::Foundation));
    let by_wealth = incidence::by(&inside, Axis::Valuation, FIFTHS, &mine);
    assert_per_pupil(
        &by_wealth,
        [114.22, 84.77, 50.09, 14.03, 20.42],
        "the mirror inside [H] by valuation",
    );
    let by_poverty = incidence::by(&inside, Axis::Poverty, FIFTHS, &mine);
    assert_per_pupil(
        &by_poverty,
        [25.40, 16.19, 48.90, 87.01, 78.69],
        "the mirror inside [H] by disadvantaged share",
    );

    let beside = incidence::under_supplement(&districts, Supplement::Mirror(Line::Beside));
    let by_wealth = incidence::by(&beside, Axis::Valuation, FIFTHS, &mine);
    assert_per_pupil(
        &by_wealth,
        [114.22, 101.69, 132.64, 115.44, 96.88],
        "the mirror beside every floor by valuation",
    );
    let flat = per_pupil(&by_wealth);
    let (low, high) = flat
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(*x), hi.max(*x)));
    assert!(
        high / low < 1.4,
        "no fifth is paid 40% more per pupil than another: {flat:?}"
    );
    assert!(
        (by_wealth[0].per_pupil()
            - incidence::by(&inside, Axis::Valuation, FIFTHS, &mine)[0].per_pupil())
        .abs()
            < CENT,
        "the two placements pay the least-wealthy fifth the same, because none of it is held"
    );
    let by_poverty = incidence::by(&beside, Axis::Poverty, FIFTHS, &mine);
    assert_per_pupil(
        &by_poverty,
        [91.36, 110.97, 144.27, 135.28, 99.10],
        "the mirror beside every floor by disadvantaged share",
    );

    let dated = incidence::under_supplement(&districts, Supplement::DatedPhaseDown);
    for axis in [Axis::Valuation, Axis::Poverty] {
        for band in incidence::by(&dated, axis, FIFTHS, &mine) {
            assert!(
                band.delta.abs() < CENT,
                "{}: fifth {}",
                axis.label(),
                band.rank
            );
            assert_eq!(band.gainers + band.losers, 0);
        }
    }

    // The direction, stated once for all seven: nothing here pays the wealthiest fifth more per
    // pupil than the least-wealthy fifth, and nothing cuts the least-wealthy fifth more than the
    // wealthiest.
    let ratchet = against_enacted(&districts, 2032, shipped(), Anchor::PriorYear);
    let capped = against_enacted(
        &districts,
        2032,
        shipped(),
        Anchor::Decayed { factor: 0.98 },
    );
    let gentle = against_enacted(
        &districts,
        2036,
        undamped(),
        Anchor::Decayed { factor: 0.99 },
    );
    for (label, frame) in [
        ("ratchet", &ratchet),
        ("2% cap", &capped),
        ("1% cap at FY2036", &gentle),
        ("rolling count", &rolling),
        ("mirror inside [H]", &inside),
        ("mirror beside", &beside),
        ("dated phase-down", &dated),
    ] {
        let bands = incidence::by(frame, Axis::Valuation, FIFTHS, &mine);
        assert!(
            bands[0].per_pupil() >= bands[4].per_pupil() - CENT,
            "{label} pays the wealthiest fifth more than the least wealthy: {:?}",
            per_pupil(&bands)
        );
    }
}
