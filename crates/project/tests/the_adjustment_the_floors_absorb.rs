//! What a declining-enrollment adjustment costs, priced against the cluster that needs one.
//!
//! [`project::enrollment_decline`] characterized the 89 districts holding $108,568,921.47 that are
//! on the guarantee for having fewer children than in FY2020, named four candidate shapes, and
//! priced none of them. These tests price all four on total state support.
//!
//! The decision the pricing turns out to be about is not the rate. Every member of the cluster is
//! on the guarantee and 68 of the 89 are fully backstopped by `[K]`, so the same mirrored `[M]`
//! delivers $46.0m to them written on a new line beside `[L]`/`[M]`/`[O]` and $4.2m written
//! inside `[H]` — and 67 of its 81 eligible members then receive nothing at all.
//!
//! Two of the four turn out not to be instruments. The rolling pupil count delivers $19.82 a
//! pupil against the $464.06 the floor pays, and a dated phase-down retires $695,296 of the
//! $67.4m it reports, because `[K]` is standing behind the line it retires.

use edfund_core::FiscalYear;
use project::decline_adjustment::{
    self as pricing, eligible, fully_backstopped, Line, Shape, MIRROR_THRESHOLD,
};
use project::enrollment_decline as decline;
use project::panel::{panel, DistrictRecord, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL as RATE};
use project::policy::GuaranteeRule;
use project::report::enrollment_growth_prior;
use project::series::{Method, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT, ONE_SIGMA};

/// A cent, the tolerance every dollar figure here is asserted at.
const CENT: f64 = 0.01;

/// The crates' median: the upper middle, never the mean of two.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty());
    values.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    values[values.len() / 2]
}

/// The two floors the cluster sits behind, which is why placement is the decision.
#[test]
fn every_member_is_on_the_guarantee_and_most_are_backstopped_too() {
    let districts = panel();
    let cluster = decline::cluster(&districts);

    assert_eq!(cluster.len(), 89);
    assert!(
        cluster.iter().all(|record| record.on_guarantee()),
        "the cluster is defined as guaranteed districts, so a payment inside [H] meets a floor \
         for every one of them"
    );

    let drawing = cluster
        .iter()
        .filter(|record| record.transition.transition_supplement > 0.0)
        .count();
    let insulated = cluster
        .iter()
        .filter(|record| fully_backstopped(record))
        .count();
    assert_eq!(drawing, 69, "cluster members drawing [K] today");
    assert_eq!(
        insulated, 68,
        "cluster members whose FY2021 total is at or above [H2] + [J]"
    );

    // The two counts are near each other and are not the same test: one reads a published column,
    // the other compares the bases. A district can be insulated and draw nothing this year.
    assert!(
        insulated != drawing,
        "if these agreed there would be no reason to carry both"
    );

    let statewide = districts
        .iter()
        .filter(|record| fully_backstopped(record))
        .count();
    assert_eq!(
        statewide, 322,
        "of 609, so this is not a property of the 89"
    );
}

/// The finding: the same rule is worth eleven times as much to the cluster on one line as another.
#[test]
fn where_the_payment_is_written_decides_what_it_is_worth() {
    let districts = panel();

    let beside = pricing::mirror(&districts, RATE, Line::Beside);
    let inside = pricing::mirror(&districts, RATE, Line::Foundation);

    // Written outside every floor, the cheque is the payment.
    assert!(
        (beside.gross - beside.statewide).abs() < CENT,
        "a line outside [L1]'s subtrahend is absorbed by nothing; ${:.2} went missing",
        beside.absorbed()
    );
    assert!((beside.statewide - 181_321_579.23).abs() < CENT);
    assert!((beside.cluster - 46_020_643.67).abs() < CENT);
    assert_eq!(beside.paid, 383);
    assert_eq!(beside.in_full, 383);
    assert_eq!(beside.not_at_all, 0);

    // Written inside [H], the guarantee takes it first and [K] takes the rest.
    assert!((inside.statewide - 66_385_014.71).abs() < CENT);
    assert!((inside.cluster - 4_195_897.42).abs() < CENT);
    assert!(
        (inside.absorbed_share() - 0.6339).abs() < 0.0001,
        "{:.4} absorbed statewide",
        inside.absorbed_share()
    );
    assert!(
        (inside.cluster_absorbed_share() - 0.9088).abs() < 0.0001,
        "{:.4} absorbed in the cluster",
        inside.cluster_absorbed_share()
    );

    assert_eq!(inside.in_full, 154);
    assert_eq!(inside.partly, 29);
    assert_eq!(inside.not_at_all, 200);
    assert_eq!(
        inside.in_full + inside.partly + inside.not_at_all,
        inside.paid,
        "the three classes are counted separately and must still partition the districts paid"
    );

    // And the ratio the design decision actually rests on.
    assert!(
        beside.cluster / inside.cluster > 10.0,
        "placement moves the cluster's receipt by {:.1}x",
        beside.cluster / inside.cluster
    );
}

/// Symmetry in wording is not symmetry in money, because decline is the ordinary case.
#[test]
fn the_mirror_reaches_nine_times_the_population_m_does() {
    let districts = panel();

    let growers = districts
        .iter()
        .filter(|record| record.supplements.growth_eligible)
        .count();
    let decliners = districts.iter().filter(|record| eligible(record)).count();
    assert_eq!(growers, 43);
    assert_eq!(decliners, 383);
    assert!(
        decliners > growers * 8,
        "{decliners} against {growers}: the same threshold with its sign reversed is a different \
         instrument"
    );

    let paid_by_m: f64 = districts
        .iter()
        .map(|record| record.supplements.growth)
        .sum();
    let mirror = pricing::mirror(&districts, RATE, Line::Beside);
    assert!((paid_by_m - 39_379_553.15).abs() < CENT);
    assert!(
        (mirror.statewide / paid_by_m - 4.605).abs() < 0.001,
        "the mirror costs {:.3} times [M]",
        mirror.statewide / paid_by_m
    );

    // Most of it does not reach the districts it would be written for.
    assert!(
        (mirror.targeting() - 0.2538).abs() < 0.0001,
        "{:.4} of the mirror lands in the cluster",
        mirror.targeting()
    );

    // So replacing the floor with it costs nearly four times the floor.
    let calibrated = pricing::replacing_the_floor(&districts);
    assert!((calibrated.target - 108_568_921.47).abs() < CENT);
    assert!(
        (calibrated.rate - 589.78).abs() < 0.01,
        "${:.2} a pupil",
        calibrated.rate
    );
    assert!((calibrated.statewide - 427_762_124.28).abs() < CENT);
    assert!(
        (calibrated.multiple() - 3.940).abs() < 0.001,
        "{:.3} times the guarantee it would replace",
        calibrated.multiple()
    );
}

/// Both of `[M]`'s own defects transfer to its mirror, and the window one breaks membership.
#[test]
fn the_mirror_inherits_the_cliff_and_the_wrong_window() {
    let districts = panel();

    // `[M]`'s cliff was found on three hundredths of a point. Three tenths of a point below the
    // mirrored threshold holds fifteen districts and $13.5m.
    let cliff = pricing::cliff(&districts, 0.003, RATE);
    assert_eq!(cliff.districts, 15);
    assert!((cliff.forgone - 13_480_937.41).abs() < CENT);
    assert_eq!(cliff.largest.0, "Akron City");
    assert!(
        (cliff.largest.1 - 4_723_112.21).abs() < CENT,
        "${:.2}",
        cliff.largest.1
    );
    assert!(
        cliff.largest.1 > 430_477.0 * 10.0,
        "the mirrored cliff is an order larger than New Lexington's on the growth side"
    );

    // The window disagrees with the cluster's own definition: [M] measures from its FY2023 base
    // and the cluster is defined from FY2020.
    let cluster = decline::cluster(&districts);
    let missed: Vec<&&DistrictRecord> = cluster.iter().filter(|record| !eligible(record)).collect();
    assert_eq!(
        missed.len(),
        8,
        "cluster members the mirrored threshold does not reach"
    );
    let held: f64 = missed.iter().map(|record| record.guarantee).sum();
    assert!((held - 3_366_298.82).abs() < CENT, "${held:.2}");

    let growing: Vec<&&&DistrictRecord> = missed
        .iter()
        .filter(|record| record.supplements.enrollment_change > 0.0)
        .collect();
    assert_eq!(
        growing.len(),
        2,
        "two members read as growing on the instrument that would be written for them"
    );
    let growing_held: f64 = growing.iter().map(|record| record.guarantee).sum();
    assert!((growing_held - 503_461.00).abs() < CENT);

    // A threshold cannot be tuned out of this: the members it misses are not near the threshold.
    assert!(
        missed
            .iter()
            .filter(|record| record.supplements.enrollment_change > MIRROR_THRESHOLD + 0.005)
            .count()
            >= 5,
        "the misses are a window disagreement, not a boundary case"
    );
}

/// The shape that addresses the mechanism directly delivers $19.82 a pupil.
#[test]
fn the_rolling_count_substitutes_a_rule_for_a_floor_and_not_any_money() {
    let districts = panel();
    let rolling = pricing::rolling_count(&districts);

    assert!((rolling.statewide - 70_105_642.08).abs() < CENT);
    assert!((rolling.foundation - 65_165_625.65).abs() < CENT);
    assert!((rolling.categorical - 48_799_954.14).abs() < CENT);
    assert!((rolling.cluster - 3_902_794.07).abs() < CENT);
    assert_eq!(rolling.movers, 391);

    assert!(
        (rolling.targeting() - 0.0557).abs() < 0.0001,
        "{:.4} of the cost reaches the cluster",
        rolling.targeting()
    );

    // Against what the floor pays the same districts, per pupil.
    let cluster = decline::cluster(&districts);
    let adm: f64 = cluster.iter().map(|record| record.current_year_adm).sum();
    let per_pupil = rolling.cluster / adm;
    assert!(
        (per_pupil - 19.82).abs() < 0.01,
        "${per_pupil:.2} a pupil across the cluster"
    );
    let floor_per_pupil = upper_middle(
        cluster
            .iter()
            .map(|record| record.guarantee / record.current_year_adm)
            .collect(),
    );
    assert!(
        (floor_per_pupil - 464.06).abs() < CENT,
        "against the ${floor_per_pupil:.2} the floor pays the median member"
    );
    assert!(
        floor_per_pupil > per_pupil * 20.0,
        "the substitution is not a response to the guarantee at this scale"
    );

    // What it does do is structural: twenty members stop being held and start being computed.
    assert_eq!(rolling.lifted_off, 20);
    assert_eq!(
        rolling.cluster_lifted_off, 20,
        "every district the substitution carries off the guarantee is in the cluster"
    );

    // And the count substituted is the department's own, not a mean of the ADM columns.
    let disagreeing = districts
        .iter()
        .filter(|record| {
            let mean = record.adm_history.iter().sum::<f64>() / 3.0;
            (mean - record.base_cost_adm()).abs() > 1.0
        })
        .count();
    assert_eq!(
        disagreeing, 96,
        "reconstructing the three-year average from ADM Data is a different number, which is why \
         the published column is what is substituted"
    );
}

/// The crossing finding was the wrong objection to a dated instrument. `[K]` is the objection.
#[test]
fn a_dated_phase_down_retires_almost_nothing() {
    let districts = panel();
    let dated = pricing::dated_phase_down(&districts, GuaranteeRule::Removed);

    // The crossing split #383 established, recovered here as the rule's own reach.
    assert_eq!(dated.dated, 39, "members the FY2026 roll would have named");
    assert_eq!(dated.missed, 50, "members that arrived within the year");
    assert_eq!(
        dated.dated + dated.missed,
        89,
        "the rule's reach and its miss partition the cluster"
    );
    assert!((dated.missed_dollars - 41_175_946.26).abs() < CENT);

    // And what it actually retires.
    assert!((dated.reported - 67_392_975.21).abs() < CENT);
    assert!(
        (dated.actual - 695_295.72).abs() < CENT,
        "${:.2} of total state support",
        dated.actual
    );
    assert_eq!(dated.made_whole, 32, "of the 39 it reaches");
    assert!(
        dated.absorbed_share() > 0.98,
        "[K] puts back {:.4} of the reported saving",
        dated.absorbed_share()
    );

    // Undating it does not rescue the saving: the objection is the backstop, not the date.
    let everyone: f64 = decline::cluster(&districts)
        .iter()
        .map(|record| record.guarantee)
        .sum();
    assert!((everyone - 108_568_921.47).abs() < CENT);
    assert!(
        dated.actual * 10.0 < everyone / 10.0,
        "a date-keyed retirement of this population is not a saving that lands badly; it is not a \
         saving"
    );
}

/// Doing nothing holds the money flat and moves it onto the hold-harmlesses.
#[test]
fn nothing_holds_the_total_and_migrates_its_provenance() {
    let districts = panel();
    let prior = enrollment_growth_prior(&districts, ONE_SIGMA);
    let shipped = Method::Shrunk {
        rate: 0.0,
        damping: DEFAULT_DAMPING,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    };
    let undamped = Method::Shrunk {
        rate: 0.0,
        damping: 1.0,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    };

    let fy2032 = pricing::nothing(&districts, FiscalYear(2032), shipped, prior);
    let fy2036 = pricing::nothing(&districts, FiscalYear(2036), undamped, prior);

    // The total does not move. Every floor holds it.
    assert!((fy2032.total_state_support - 1_448_190_182.04).abs() < CENT);
    assert!((fy2036.total_state_support - 1_445_702_518.50).abs() < CENT);
    assert!(
        (fy2036.total_state_support / fy2032.total_state_support - 1.0).abs() < 0.005,
        "the total is flat across nine years and two methods"
    );

    // The enrollment does, and so the per-pupil amount does.
    assert!(fy2036.adm < fy2032.adm * 0.9, "{:.0}", fy2036.adm);
    assert!((fy2032.per_pupil() - 7_558.09).abs() < CENT);
    assert!((fy2036.per_pupil() - 8_850.12).abs() < CENT);

    // And the line the money arrives on migrates out of the formula.
    assert!((fy2032.guarantee - 144_560_557.55).abs() < CENT);
    assert!((fy2036.guarantee - 335_690_496.94).abs() < CENT);
    assert!(
        (fy2032.held_share() - 0.1276).abs() < 0.0001,
        "{:.4}",
        fy2032.held_share()
    );
    assert!(
        (fy2036.held_share() - 0.2694).abs() < 0.0001,
        "{:.4}",
        fy2036.held_share()
    );
    assert!(
        fy2036.held_share() > 0.25,
        "a quarter of what these districts receive would depend on [I] and [K] standing"
    );
    assert_eq!(fy2032.on_the_floor, 89);
    assert_eq!(fy2036.on_the_floor, 89);

    // The shipped method is the floor on the drift, not a competing estimate of it.
    let shipped_2036 = pricing::nothing(&districts, FiscalYear(2036), shipped, prior);
    assert!(
        shipped_2036.guarantee < fy2036.guarantee,
        "damping discards the trend, so the shipped projection under-states guarantee reliance"
    );
}

/// The marginal design is cheaper and is the one to refuse.
#[test]
fn paying_on_the_pupils_lost_costs_a_thirteenth_and_prices_the_wrong_thing() {
    let districts = panel();
    let roll = pricing::mirror(&districts, RATE, Line::Beside);
    let lost = pricing::marginal(&districts, RATE);

    assert!((lost.statewide - 14_236_934.77).abs() < CENT);
    assert!((lost.cluster - 4_088_478.76).abs() < CENT);
    assert!(
        roll.statewide / lost.statewide > 12.0,
        "the whole-roll design costs {:.1} times the marginal one",
        roll.statewide / lost.statewide
    );

    // Which is the point: the cheap one pays per child not enrolled, and the expensive one
    // charges an eligible district for each further child it loses.
    assert!(Shape::Mirror.rewards_at_the_margin().contains("whole roll"));
}

/// None of the four needs a `Policy` field, which was asked before the work began.
#[test]
fn no_shape_needs_a_lever() {
    for shape in [
        Shape::Mirror,
        Shape::RollingCount,
        Shape::DatedPhaseDown,
        Shape::Nothing,
    ] {
        assert!(
            !shape.needs_a_lever(),
            "{} would need a lever",
            shape.label()
        );
        assert!(
            !shape.rewards_at_the_margin().is_empty(),
            "{} does not say what it rewards at the margin",
            shape.label()
        );
    }

    // And the placements are distinguished, because that is the decision the pricing found.
    assert_ne!(Line::Foundation.label(), Line::Beside.label());
}

/// A rate of zero arrives as zero, which is what licenses the ladder on an unpublished output.
#[test]
fn the_ladder_is_the_identity_at_a_rate_of_nothing() {
    let districts = panel();
    for line in [Line::Foundation, Line::Beside] {
        let none = pricing::mirror(&districts, 0.0, line);
        assert_eq!(none.paid, 0, "{}", line.label());
        assert!(none.statewide.abs() < CENT);
        assert!(none.cluster.abs() < CENT);
    }
}
