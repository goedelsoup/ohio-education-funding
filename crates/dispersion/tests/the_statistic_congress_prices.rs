//! The coefficient of variation this crate has always computed, read as 20 U.S.C. 6337 reads it.
//!
//! `revenue-stream/title-i` records that the Education Finance Incentive Grant "weights allocations
//! by a state's own equity and effort, so Ohio's local-reliance position affects what Ohio
//! receives", and that "whether that has been material for Ohio is not established".
//!
//! The equity half is established here, and the premise is wrong in a way worth recording: the
//! statute never looks at the state-and-local split. What it looks at is the enrolment-weighted
//! coefficient of variation of per-pupil expenditure across a state's own agencies — the statistic
//! `crates/dispersion` exists to compute, and which the corpus has quoted for Ohio a dozen times
//! without ever having anything to compare it against.
//!
//! # Ninth of fifty-one, and on the edge of a band
//!
//! Ohio's factor is 0.1987 against a median state's 0.1587. It enters twice: the state's
//! per-formula-child rate is `1.30 - factor`, and the within-state poverty ladder steepens in
//! bands at 0.10 and 0.20. Ohio is the closest state in the country to the 0.20 edge and still
//! below it, and the one adjustment this fixture cannot apply pushes it further below.

use dispersion::equity_factor::{self as equity, Band};

/// Where Ohio sits, and the benchmark its own number never had.
#[test]
fn ohio_spreads_wider_than_forty_two_states_and_narrower_than_eight() {
    let ohio = equity::ohio();
    assert!(
        (ohio.factor - 0.198_729).abs() < 5e-6,
        "Ohio's equity factor is {:.6}",
        ohio.factor
    );
    assert_eq!(equity::rank("OH"), Some(9), "Ohio's rank, widest first");

    let all = equity::by_state();
    assert_eq!(all.len(), 51, "states and jurisdictions in the panel");
    let wider: Vec<&str> = all
        .iter()
        .take(8)
        .map(|s| s.state.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        wider,
        vec!["AK", "MA", "VT", "ID", "LA", "MT", "NY", "SD"],
        "the states that spread wider than Ohio"
    );

    // And the benchmark. Every equity finding in this corpus quotes a coefficient of variation
    // near 0.20; this is the first line that can say what 0.20 is near.
    let median = equity::median_factor();
    assert!(
        (median - 0.158_181).abs() < 5e-6,
        "the median state's factor is {median:.6}"
    );
    assert!(
        ohio.factor > median * 1.2,
        "Ohio at {:.4} against a median of {median:.4}",
        ohio.factor
    );
}

/// The filter is load-bearing, and the fixture is why.
///
/// The national file carries the comparable set for every state **plus** every Ohio agency,
/// including 357 community schools, JVSDs and service centres that no other state's rows include.
/// Computed over the file as it stands, Ohio reads 0.2267 and second in the nation — a fact about
/// which rows this repository chose to keep, not about America. This test pins the asymmetry so a
/// future reader cannot drop the filter and believe the answer.
#[test]
fn every_non_comparable_row_in_the_national_file_is_an_ohio_one() {
    let panel = dispersion::national_peers::national_panel();
    let extra: Vec<&dispersion::national_peers::NationalDistrict> =
        panel.iter().filter(|d| !d.comparable).collect();
    assert_eq!(extra.len(), 357, "non-comparable rows in the fixture");
    assert!(
        extra.iter().all(|d| d.state == "OH"),
        "a non-comparable row outside Ohio: {:?}",
        extra.iter().find(|d| d.state != "OH").map(|d| &d.leaid)
    );

    // What keeping them would do, computed rather than asserted from memory.
    let with_extras = weighted_cv(
        &panel
            .iter()
            .filter(|d| d.state == "OH" && d.enrollment > equity::MIN_ENROLMENT)
            .filter_map(|d| Some((d.enrollment, d.spending_per_pupil()?)))
            .collect::<Vec<_>>(),
    );
    assert!(
        (with_extras - 0.226_749).abs() < 5e-6,
        "Ohio over every row is {with_extras:.6}"
    );
    assert!(
        Band::of(with_extras) == Band::Steepest && equity::ohio().band() == Band::Middle,
        "the two readings should fall on opposite sides of the 0.20 edge"
    );
}

/// **The finding.** No state in the country sits closer to the 0.20 edge than Ohio.
#[test]
fn ohio_is_the_closest_state_to_the_edge_that_steepens_the_ladder() {
    let ohio = equity::ohio();
    assert_eq!(ohio.band(), Band::Middle);
    assert!(
        (ohio.distance_to_edge() - 0.001_271).abs() < 5e-6,
        "Ohio sits {:.6} from an edge",
        ohio.distance_to_edge()
    );

    // Ranked by distance to the nearer edge, only Washington is closer — and Washington is at the
    // *other* edge, 0.0001 above 0.10. Ohio is the closest state to 0.20 in either direction.
    let mut by_distance = equity::by_state();
    by_distance.sort_by(|a, b| a.distance_to_edge().total_cmp(&b.distance_to_edge()));
    let closest: Vec<&str> = by_distance
        .iter()
        .take(5)
        .map(|s| s.state.as_str())
        .collect();
    assert_eq!(closest, vec!["WA", "OH", "NC", "FL", "SD"]);
    assert!(
        by_distance[0].state == "WA" && by_distance[0].factor < 0.15,
        "Washington should be at the 0.10 edge, not the 0.20 one"
    );

    let nearest_to_twenty = equity::by_state()
        .into_iter()
        .min_by(|a, b| (a.factor - 0.20).abs().total_cmp(&(b.factor - 0.20).abs()))
        .expect("the panel is not empty");
    assert_eq!(nearest_to_twenty.state, "OH");
}

/// And it stays below the edge under the one statutory step the fixture can bound.
///
/// The statute counts a poverty child 1.4 times. The count it uses is the Census Bureau's
/// small-area estimate, which nothing here holds — so the bracket over-applies the adjustment
/// with Ohio's economically disadvantaged share, about three times the Census poverty rate, and
/// takes the pair as bounds rather than the point as an estimate.
#[test]
fn the_middle_band_survives_both_ends_of_the_adjustment() {
    let (adjusted, plain) = equity::ohio_bracket();
    assert!(
        (adjusted - 0.168_404).abs() < 5e-6 && (plain - 0.198_125).abs() < 5e-6,
        "the bracket is [{adjusted:.6}, {plain:.6}]"
    );
    assert!(
        adjusted < plain,
        "the adjustment should narrow Ohio's spread, not widen it"
    );
    assert_eq!(Band::of(adjusted), Band::Middle);
    assert_eq!(Band::of(plain), Band::Middle);

    // The adjustment narrows because Ohio's poorest districts are among its highest-spending, so
    // inflating their pupil count pulls them toward the mean. That is a fact about Ohio and not
    // about the arithmetic, and it is the reason the bracket runs the way it does.
    let profile = dispersion::profile::districts();
    let (poverty, spending) = dispersion::profile::paired(
        &profile,
        |d| d.economically_disadvantaged,
        |d| d.operating_expenditure_per_pupil,
    );
    let association =
        dispersion::wealth_neutrality(&poverty, &spending).expect("the profile report pairs");
    assert!(
        association.correlation > 0.3,
        "disadvantage against spending is r = {:.4}, so the adjustment need not narrow",
        association.correlation
    );

    // The unadjusted end here is over the districts that join the profile report, so it is not
    // quite the full-panel figure. Close enough that the bracket is about the adjustment.
    assert!((plain - equity::ohio().factor).abs() < 1e-3);
}

/// What the spread costs Ohio, on the multiplier rather than the band.
#[test]
fn the_spread_costs_ohio_three_and_a_half_per_cent_of_its_rate() {
    let ohio = equity::ohio();
    assert!(
        (ohio.state_rate() - 1.101_271).abs() < 5e-6,
        "Ohio's rate multiplier is {:.6}",
        ohio.state_rate()
    );

    let median_rate = equity::STATE_RATE_CONSTANT - equity::median_factor();
    let against_median = 1.0 - ohio.state_rate() / median_rate;
    assert!(
        (against_median - 0.0355).abs() < 5e-4,
        "Ohio is {against_median:.4} below a median state"
    );

    // And against the floor the statute grants a state that meets the disparity standard, which
    // caps its equity factor at 0.10.
    let floor_rate = equity::STATE_RATE_CONSTANT - equity::BAND_EDGES[0];
    let against_floor = 1.0 - ohio.state_rate() / floor_rate;
    assert!(
        (against_floor - 0.0823).abs() < 5e-4,
        "Ohio is {against_floor:.4} below a state at the 0.10 cap"
    );
}

/// The two directions the one number runs in.
#[test]
fn the_same_factor_cuts_the_rate_and_steepens_the_ladder() {
    // Wider spread, smaller rate: strictly decreasing across the ranking.
    let rates: Vec<f64> = equity::by_state().iter().map(|s| s.state_rate()).collect();
    assert!(
        rates.windows(2).all(|w| w[0] <= w[1]),
        "the rate should rise as the spread narrows"
    );

    // Wider spread, steeper ladder: every rung above the first grows band to band, and the top
    // rung doubles from the flattest band to the steepest.
    let ladders = [
        Band::Flattest.weights(),
        Band::Middle.weights(),
        Band::Steepest.weights(),
    ];
    assert!(ladders.iter().all(|w| (w[0] - 1.0).abs() < f64::EPSILON));
    for (rung, (flattest, middle, steepest)) in ladders[0]
        .iter()
        .zip(ladders[1].iter())
        .zip(ladders[2].iter())
        .map(|((a, b), c)| (a, b, c))
        .enumerate()
        .skip(2)
    {
        assert!(
            flattest < middle && middle < steepest,
            "rung {rung} does not steepen band to band"
        );
    }
    assert!((ladders[2][4] - ladders[0][4] * 2.0).abs() < f64::EPSILON);

    // The second rung is the exception, and it is in the statute: the flattest ladder's 1.75 sits
    // above the middle ladder's 1.5. Asserted so a future reader does not "fix" it.
    assert!(ladders[0][1] > ladders[1][1]);
}

/// The statute's own weighting is what puts Ohio near the edge.
///
/// Unweighted, Ohio's districts spread at 0.1779 and the question would not arise. The statute
/// weights "according to the number of pupils served", and Ohio's large districts are its extreme
/// ones — so the measure Congress specified is the one that finds Ohio unusual.
#[test]
fn weighting_by_pupils_is_what_finds_ohio_unusual() {
    let ohio: Vec<(f64, f64)> = dispersion::national_peers::national_panel()
        .iter()
        .filter(|d| d.state == "OH" && d.comparable && d.enrollment > equity::MIN_ENROLMENT)
        .filter_map(|d| Some((d.enrollment, d.spending_per_pupil()?)))
        .collect();
    assert_eq!(ohio.len(), 608);

    let unweighted = weighted_cv(&ohio.iter().map(|(_, x)| (1.0, *x)).collect::<Vec<_>>());
    assert!(
        (unweighted - 0.177_932).abs() < 5e-6,
        "Ohio unweighted is {unweighted:.6}"
    );
    assert!(
        equity::ohio().factor > unweighted,
        "weighting should widen Ohio, and it is the direction the finding rests on"
    );
    assert!(
        Band::of(unweighted) == Band::Middle && unweighted < 0.19,
        "unweighted, Ohio would be nowhere near the edge"
    );
}

/// The weighted coefficient of variation, restated here so the tests do not depend on the module
/// under test for the arithmetic they check it with.
fn weighted_cv(observations: &[(f64, f64)]) -> f64 {
    let weight: f64 = observations.iter().map(|(w, _)| w).sum();
    let mean: f64 = observations.iter().map(|(w, x)| w * x).sum::<f64>() / weight;
    let variance: f64 = observations
        .iter()
        .map(|(w, x)| w * (x - mean) * (x - mean))
        .sum::<f64>()
        / weight;
    variance.sqrt() / mean
}
