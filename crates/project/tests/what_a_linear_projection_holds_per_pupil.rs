//! What `report::forecast` holds fixed when it substitutes a projected enrolment, and what
//! holding it is worth.
//!
//! # The question
//!
//! Issue #407: `policy::apply` computes base cost aid as `base cost state share × (projected ADM
//! ÷ modelled ADM)`, which is exactly linear in the pupil count and therefore holds **base cost
//! per pupil and local capacity per pupil** at their FY2027 values. That is exact at modelled
//! enrolment, which is where every deterministic result in the crate is computed, and
//! `report::forecast` is the one caller for which the scope note is false.
//!
//! # The two terms, and the one the issue did not reach for
//!
//! Base cost aid is a **difference** of two per-pupil quantities, and neither is constant in the
//! count. For a shrinking district base cost per pupil rises — R.C. 3317.011's staffing floors
//! are a `max` and do not follow a roll down, which is
//! `what_the_staffing_minimums_are_worth_and_where_they_stop.rs`'s wedge — and local capacity per
//! pupil rises too, because R.C. 3317.017's wealth blend is a dollar amount over a shrinking
//! denominator, which is `the_denominator_that_is_larger_than_the_model.rs`'s local charge. The
//! two errors do not cancel, they compete.
//!
//! At FY2032, current law, the method the feed runs:
//!
//! | statewide realized aid | |
//! |---|--:|
//! | at observed enrolment | $7,281.2m |
//! | projected, linear — what the feed publishes | $7,233.1m |
//! | projected, base cost per pupil recomputed alone | $7,244.4m |
//! | projected, capacity per pupil recomputed alone | $7,192.1m |
//! | **projected, both** | **$7,201.5m** |
//!
//! **The capacity term is 3.6 times the base cost term and carries the sign.** #407 reached for
//! the base cost one, on the strength of #389 having just measured it; it is the smaller of the
//! two and it points the other way.
//!
//! # What it is worth
//!
//! **−$31.7m, 0.44% of projected aid — and 66% of the −$48.1m enrollment effect the forecast
//! exists to report.** Not a rounding artefact, and not something the interval already covers in
//! the sense that matters, because the interval moves too: both terms are convex in the count, so
//! the correction is larger at the low end of the enrollment band than at the high end, and the
//! band *widens* rather than shifting. $6,957.9m–$7,581.9m becomes $6,853.4m–$7,739.6m, and the
//! `(+/-4.3%)` the corpus publishes for FY2032 becomes `(+/-6.2%)`.
//!
//! `forecast`'s own comment that the enrollment band's low end is the aid band's low end
//! survives: aid still falls with the count and no end is reordered.
//!
//! # The sign is the difference of the two terms, exactly
//!
//! For a district on formula and off the minimum state share, the sign of the error is the sign
//! of `Δ base cost per pupil − Δ capacity per pupil` for **252 of 252 districts** at FY2032.
//! Zero exceptions, because those two terms are the whole mechanism. So the population does not
//! split by size: **233 districts are overstated and 65 understated**, the understated ones being
//! those whose wedge opens faster than their denominator shrinks, plus the growing districts, for
//! which every sign reverses.
//!
//! # Who it can reach: exactly the districts the guarantee does not pay
//!
//! The guarantee is a `max`, so a guaranteed district is inert. At FY2032 the linear path puts
//! **312** districts on the guarantee and leaves **297** on formula — and all 297 move, against
//! **one** of the 312, the single district the correction lifts off the guarantee. The bound is
//! not approximate: the exposed population is the formula population. The correction then sends
//! **15** districts the other way onto the guarantee, which is the staffing-minimums finding that
//! a falling formula side is absorbed there, arriving from a second direction. **120 districts
//! move by more than 1% of their aid and 10 by more than 5%.**
//!
//! # And the averaged count is a cushion the projection spends
//!
//! R.C. 3317.011 funds base cost on `max(mean of three years, the current year)`, so a declining
//! district is funded on a count above the children it teaches — statewide **1.0165** current-year
//! pupils' worth in the FY2027 model. The shipped projection puts that at **1.0000** by FY2032,
//! because the damping flattens the series and a flat series has no average to sit above. Base
//! cost ADM therefore falls **2.86%** over the horizon while enrolled ADM falls **1.26%**, and
//! **58% of the whole correction is the cushion closing** rather than pupils leaving: carried
//! one-for-one with enrolled ADM the correction is −$13.3m instead of −$31.7m.
//!
//! A property of the projection rather than of the statute, and the corpus already holds the
//! other half of it — at the published horizon the trend is simply gone. What cannot be said is
//! that the averaging absorbs this error. At this horizon it more than doubles it.
//!
//! # What is not established, and why `forecast` is left alone
//!
//! Which of the two runs is right. The linear path holds a district's **wealth per pupil** fixed,
//! which is the arithmetic of a tax base that shrinks exactly as fast as its children leave; the
//! corrected path holds its **wealth** fixed in nominal dollars for the whole horizon. Neither is
//! a forecast of an Ohio tax base and this repository has none — 60% of the capacity blend is
//! assessed valuation, and `project`'s own crate note records that valuation cannot be projected
//! from the committed data.
//!
//! So $31.7m is the width of the interval a projected local share sits in, rather than a
//! correction waiting to be applied. Publishing the width is the claim; picking an end would move
//! every band in the feed on an assumption about property values nothing here has measured.

use edfund_core::FiscalYear;
use foundation::{aggregate_base_cost, StatewideFactors};
use project::panel::{panel, DistrictRecord};
use project::policy::Policy;
use project::projected_base_cost::{base_cost_adm, forecast, movements, restated, Terms};
use project::report::{
    enrollment_growth_prior, forecast as linear_forecast, projected_series, window_ending_at,
};
use project::series::{Method, Prior, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT, ONE_SIGMA};

/// A cent, the tolerance every dollar identity here is asserted at.
const CENT: f64 = 0.01;

/// The horizon the corpus states the current-law band at, and the one every figure here is on.
const HORIZON: FiscalYear = FiscalYear(2032);

/// The method the feed runs, with the per-district `toward` left for the record to fill in.
fn shipped() -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: DEFAULT_DAMPING,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    }
}

fn prior(districts: &[DistrictRecord]) -> Prior {
    enrollment_growth_prior(districts, ONE_SIGMA)
}

/// **The base cost count is an identity on the three enrolled-ADM columns, not a model.**
///
/// `max(mean of the three, the current year)` reproduces the department's published base cost
/// enrolled ADM for every district in the model. The whole restatement rests on being able to
/// carry that count forward, so it is checked before anything is carried anywhere.
#[test]
fn the_base_cost_count_reproduces_for_every_district_in_the_model() {
    let districts = panel();
    let mut worst: f64 = 0.0;
    for record in &districts {
        worst = worst.max((base_cost_adm(record.adm_history) - record.base_cost_adm()).abs());
    }
    // Two decimals of published ADM over three years is a third of a hundredth at worst.
    assert!(worst < 0.0001, "worst residual {worst} pupils");

    // And the `max` is not decorative: it binds for the growing districts and no others.
    let binding = districts
        .iter()
        .filter(|r| {
            let [a, b, c] = r.adm_history;
            c > (a + b + c) / 3.0
        })
        .count();
    assert_eq!(binding, 105);
}

/// **The restatement is the identity at modelled enrolment, and `Terms::NEITHER` is inert.**
///
/// Both halves matter. The first says the reconstruction of base cost aid from R.C. 3317.011 and
/// R.C. 3317.017 agrees with the department's published dollars, so a difference at a projected
/// count is the projection and not the reconstruction. The second says the hook
/// `report::forecast_with` carries changes nothing when it is asked to change nothing — which is
/// what makes the corrected run a comparison against the committed path rather than against a
/// second loop.
#[test]
fn the_restatement_is_the_identity_where_the_department_computed_it() {
    let districts = panel();
    let factors = StatewideFactors::fy2027();
    for record in &districts {
        let count = record.base_cost_adm();
        let restatement = restated(record, [count, count, count], Terms::BOTH, &factors);
        assert!(
            (restatement.base_cost_state_share - record.base_cost_state_share).abs() < CENT,
            "{}: {} against {}",
            record.name,
            restatement.base_cost_state_share,
            record.base_cost_state_share
        );
        assert!(
            (restatement.core_foundation_funding - record.core_foundation_funding).abs() < CENT
        );
    }

    let inert = forecast(
        &districts,
        &Policy::current_law(),
        HORIZON,
        shipped(),
        prior(&districts),
        Terms::NEITHER,
    );
    let committed = linear_forecast(
        &districts,
        &Policy::current_law(),
        HORIZON,
        shipped(),
        prior(&districts),
    );
    assert!((inert.realized_aid - committed.realized_aid).abs() < CENT);
    assert!((inert.low - committed.low).abs() < CENT);
    assert!((inert.high - committed.high).abs() < CENT);
    assert_eq!(inert.on_guarantee, committed.on_guarantee);
}

/// **The reconstruction needs no calibration for 608 districts, and Akron needs 0.27%.**
///
/// Akron City's published base cost state share divides out to 18,892.45 pupils — `[a] Enrolled
/// ADM` — rather than the 18,842.45 of `[b3] FY26 Enrolled ADM` that R.C. 3317.017(B) names. It
/// is also the only district in the model whose two counts differ at all, which is what
/// `DistrictRecord::categorical_enrolled_adm` already carries it as the example of. The
/// department used the other column for one district, and this is where that shows up.
#[test]
fn one_district_multiplies_its_state_share_by_the_other_enrolled_adm() {
    let districts = panel();
    let factors = StatewideFactors::fy2027();
    let mut needing_calibration = Vec::new();
    for record in &districts {
        let Some(capacity) = record.published_capacity_per_pupil else {
            continue;
        };
        let count = record.base_cost_adm();
        if count <= 0.0 || record.current_year_adm <= 0.0 {
            continue;
        }
        let per_pupil =
            aggregate_base_cost(&record.enrollment.scaled_to(count), &factors).aggregate / count;
        let reconstructed =
            (per_pupil - capacity).max(per_pupil * project::panel::MINIMUM_STATE_SHARE);
        let published = record.base_cost_state_share / record.current_year_adm;
        if (published / reconstructed - 1.0).abs() > 1e-4 {
            needing_calibration.push((record.name.clone(), published / reconstructed - 1.0));
        }
    }
    assert_eq!(needing_calibration.len(), 1);
    let (name, off) = &needing_calibration[0];
    assert_eq!(name, "Akron City");
    assert!((off - 0.002_653).abs() < 1e-5, "{off}");

    // And the count it actually divides by is the one four columns away.
    let akron = districts
        .iter()
        .find(|r| r.name == "Akron City")
        .expect("Akron is in the model");
    let residual = akron.base_cost_per_pupil
        - akron
            .published_capacity_per_pupil
            .expect("Akron carries a published capacity");
    let implied = akron.base_cost_state_share / residual;
    assert!(
        (implied - akron.categorical_enrolled_adm).abs() < 0.05,
        "{implied}"
    );
    assert!((implied - akron.current_year_adm).abs() > 49.0);

    // One district, because the two counts coincide everywhere else.
    assert_eq!(
        districts
            .iter()
            .filter(|r| (r.categorical_enrolled_adm - r.current_year_adm).abs() > 0.005)
            .count(),
        1
    );
}

/// **The two terms compete, and the one the issue named is the smaller.**
///
/// The table in the module note, asserted. Base cost per pupil rising pays *more* aid than the
/// linear path; capacity per pupil rising over a shrinking denominator pays much less; the net is
/// the second minus most of the first.
#[test]
fn the_capacity_term_carries_the_sign_and_is_three_and_a_half_times_the_other() {
    let districts = panel();
    let policy = Policy::current_law();
    let prior = prior(&districts);
    let run = |terms| forecast(&districts, &policy, HORIZON, shipped(), prior, terms).realized_aid;

    let linear = run(Terms::NEITHER);
    let base_cost_only = run(Terms::BASE_COST);
    let capacity_only = run(Terms::CAPACITY);
    let both = run(Terms::BOTH);

    assert!((linear - 7_233_134_576.71).abs() < CENT, "{linear}");
    assert!(
        (base_cost_only - linear - 11_254_806.45).abs() < CENT,
        "{base_cost_only}"
    );
    assert!(
        (capacity_only - linear + 40_995_630.93).abs() < CENT,
        "{capacity_only}"
    );
    assert!((both - linear + 31_675_951.21).abs() < CENT, "{both}");

    // The claim the module makes about which one matters, as a ratio rather than two dollar
    // figures a reader has to divide.
    let base_cost_term = base_cost_only - linear;
    let capacity_term = linear - capacity_only;
    assert!(base_cost_term > 0.0 && capacity_term > 0.0);
    assert!(
        (capacity_term / base_cost_term - 3.6425).abs() < 0.0005,
        "{}",
        capacity_term / base_cost_term
    );
}

/// **It is two thirds of the effect the forecast exists to report.**
///
/// The comparison that decides whether the approximation is fine: not the correction against the
/// total, which makes anything look small, but the correction against the *movement* the total is
/// reported for.
#[test]
fn the_correction_is_two_thirds_of_the_enrollment_effect() {
    let districts = panel();
    let policy = Policy::current_law();
    let prior = prior(&districts);
    let observed = project::report::Totals::of(&project::policy::apply_all(&districts, &policy));
    let linear = linear_forecast(&districts, &policy, HORIZON, shipped(), prior);
    let corrected = forecast(&districts, &policy, HORIZON, shipped(), prior, Terms::BOTH);

    let enrollment_effect = linear.realized_aid - observed.realized_aid;
    let correction = corrected.realized_aid - linear.realized_aid;
    assert!(
        (enrollment_effect + 48_093_014.94).abs() < CENT,
        "{enrollment_effect}"
    );
    assert!(
        (correction / enrollment_effect - 0.6586).abs() < 0.0005,
        "{}",
        correction / enrollment_effect
    );

    // And a tenth of a percent short of half a percent of the level, which is the figure that
    // makes it look negligible and is the wrong comparison.
    assert!((correction / linear.realized_aid + 0.004_379).abs() < 5e-6);
}

/// **The band widens rather than shifting, because both terms are convex in the count.**
///
/// The low end of the enrollment interval is where the district has shrunk most, so it is where
/// the correction is largest; the high end is where it is smallest, and for a district projected
/// to grow it reverses. A forecast that scales a curved function from its centre therefore
/// understates its own interval.
#[test]
fn correcting_the_two_terms_widens_the_aid_band_by_two_fifths() {
    let districts = panel();
    let policy = Policy::current_law();
    let prior = prior(&districts);
    let linear = linear_forecast(&districts, &policy, HORIZON, shipped(), prior);
    let corrected = forecast(&districts, &policy, HORIZON, shipped(), prior, Terms::BOTH);

    assert!((linear.low - 6_957_859_359.94).abs() < CENT);
    assert!((linear.high - 7_581_865_913.57).abs() < CENT);
    assert!(
        (corrected.low - 6_853_439_138.80).abs() < CENT,
        "{}",
        corrected.low
    );
    assert!(
        (corrected.high - 7_739_639_450.06).abs() < CENT,
        "{}",
        corrected.high
    );

    // Both ends move outward: the band is wider, not lower.
    assert!(corrected.low < linear.low && corrected.high > linear.high);
    let widening = (corrected.high - corrected.low) / (linear.high - linear.low) - 1.0;
    assert!((widening - 0.4202).abs() < 0.0005, "{widening}");

    // Which is the figure the corpus publishes, moved.
    assert!((linear.half_width() - 0.043_135).abs() < 5e-6);
    assert!(
        (corrected.half_width() - 0.061_529).abs() < 5e-6,
        "{}",
        corrected.half_width()
    );

    // And the ordering `report::forecast` relies on survives: a smaller district still draws
    // less aid, so the enrollment band's low end is still the aid band's low end.
    assert!(corrected.low < corrected.realized_aid && corrected.realized_aid < corrected.high);
}

/// **The sign of a district's error is the difference of the two terms, with no exceptions.**
///
/// Not a tendency and not a correlation: for every district the two terms can reach — on formula
/// at both ends, and not held at the minimum state share, where only base cost is in the answer —
/// the prediction is right. A district on the guarantee is inert, and one at the floor is moved by
/// the base cost term alone.
#[test]
fn the_two_terms_predict_every_sign_they_can_reach() {
    let districts = panel();
    let policy = Policy::current_law();
    let moved = movements(&districts, &policy, HORIZON, shipped(), prior(&districts));
    assert_eq!(moved.len(), 609);

    let by_irn = |irn: &str| {
        districts
            .iter()
            .find(|r| r.irn == irn)
            .expect("in the panel")
    };
    let (mut reachable, mut wrong) = (0, 0);
    let (mut guaranteed, mut at_floor) = (0, 0);
    for movement in &moved {
        if (movement.restated - movement.linear).abs() <= 1.0 {
            continue;
        }
        if movement.on_guarantee || movement.on_guarantee_restated {
            guaranteed += 1;
            continue;
        }
        if by_irn(&movement.irn).at_minimum_state_share() {
            at_floor += 1;
            continue;
        }
        reachable += 1;
        if movement.predicted() != (movement.restated > movement.linear) {
            wrong += 1;
        }
    }
    assert_eq!((reachable, wrong), (252, 0));
    assert_eq!((guaranteed, at_floor), (16, 30));
}

/// **The population it reaches, and the guarantee that absorbs part of it.**
///
/// The bound #407 asked for before anything was priced.
#[test]
fn the_guarantee_absorbs_it_and_the_rest_reaches_fewer_than_half_the_state() {
    let districts = panel();
    let policy = Policy::current_law();
    let prior = prior(&districts);
    let moved = movements(&districts, &policy, HORIZON, shipped(), prior);

    let touched = moved
        .iter()
        .filter(|m| (m.restated - m.linear).abs() > 1.0)
        .count();
    assert_eq!(touched, 298);
    assert_eq!(
        moved.iter().filter(|m| m.on_guarantee).count(),
        312,
        "the linear path's guarantee count, which the corpus binds"
    );
    assert_eq!(
        moved.iter().filter(|m| m.on_guarantee_restated).count(),
        326
    );
    assert_eq!(
        moved
            .iter()
            .filter(|m| !m.on_guarantee && m.on_guarantee_restated)
            .count(),
        15,
        "a falling formula side is absorbed by the guarantee, from a second direction"
    );
    assert_eq!(
        moved
            .iter()
            .filter(|m| m.on_guarantee && !m.on_guarantee_restated)
            .count(),
        1,
        "and one district is lifted off it, which is why 312 becomes 326 and not 327"
    );

    // The bound, and it is exact rather than approximate: the exposed population is the formula
    // population. Every district the guarantee does not pay moves, and one that it does.
    let on_formula = moved.iter().filter(|m| !m.on_guarantee).count();
    assert_eq!(on_formula, 297);
    assert_eq!(
        moved
            .iter()
            .filter(|m| !m.on_guarantee && (m.restated - m.linear).abs() > 1.0)
            .count(),
        on_formula
    );
    assert_eq!(
        moved
            .iter()
            .filter(|m| m.on_guarantee && (m.restated - m.linear).abs() > 1.0)
            .count(),
        1
    );

    // How hard it lands on the ones it reaches.
    assert_eq!(
        moved.iter().filter(|m| m.relative().abs() > 0.01).count(),
        120
    );
    assert_eq!(
        moved.iter().filter(|m| m.relative().abs() > 0.05).count(),
        10
    );

    // And the split, which is not by size.
    let overstated = moved
        .iter()
        .filter(|m| m.restated - m.linear < -1.0)
        .count();
    let understated = moved.iter().filter(|m| m.restated - m.linear > 1.0).count();
    assert_eq!((overstated, understated), (233, 65));
}

/// **The averaged count is a cushion, and the damped projection spends it.**
///
/// R.C. 3317.011 funds base cost on a three-year average, so a district in decline is funded on
/// more pupils than it teaches. That cushion is worth 1.65% of current-year enrolled ADM
/// statewide today and **nothing** at FY2032, because the damping flattens the series and a flat
/// series has no average to sit above. So base cost enrolled ADM falls more than twice as far as
/// enrolled ADM does over the horizon, and most of the correction is that rather than the count.
///
/// Carry the base cost count one-for-one with enrolled ADM — the projection a reader would write
/// who had not noticed the averaging — and the correction is $13.3m rather than $31.7m.
#[test]
fn most_of_the_correction_is_a_cushion_closing_rather_than_pupils_leaving() {
    let districts = panel();
    let policy = Policy::current_law();
    let prior = prior(&districts);
    let factors = StatewideFactors::fy2027();

    let (mut modelled, mut projected, mut enrolled_now, mut enrolled_then) = (0.0, 0.0, 0.0, 0.0);
    for record in &districts {
        let series = projected_series(record, HORIZON, shipped(), prior);
        let window = window_ending_at(&series, HORIZON, |p| p.point);
        modelled += record.base_cost_adm();
        projected += base_cost_adm(window);
        enrolled_now += record.current_year_adm;
        enrolled_then += window[2];
    }
    // The cushion, opened and closed.
    assert!((modelled / enrolled_now - 1.016_453).abs() < 5e-6);
    assert!((projected / enrolled_then - 1.000_046).abs() < 5e-6);
    // And so the funded count falls more than twice as far as the taught one.
    assert!((projected / modelled - 1.0 + 0.028_559).abs() < 5e-6);
    assert!((enrolled_then / enrolled_now - 1.0 + 0.012_621).abs() < 5e-6);

    let linear = linear_forecast(&districts, &policy, HORIZON, shipped(), prior).realized_aid;
    let corrected =
        forecast(&districts, &policy, HORIZON, shipped(), prior, Terms::BOTH).realized_aid;

    // The same restatement with the base cost count made to track enrolled ADM exactly.
    let one_for_one = project::report::forecast_with(
        &districts,
        &policy,
        HORIZON,
        shipped(),
        prior,
        |record, window| {
            let tracked = record.base_cost_adm() * window[2] / record.current_year_adm;
            restated(record, [tracked, tracked, tracked], Terms::BOTH, &factors)
        },
    )
    .realized_aid;

    assert!(
        (one_for_one - linear + 13_310_998.55).abs() < CENT,
        "{one_for_one}"
    );
    // Both are corrections in the same direction, and the averaged one is the larger. The statute
    // does not absorb this error at this horizon; it more than doubles it.
    assert!(corrected - linear < one_for_one - linear);
    assert!(
        ((one_for_one - linear) / (corrected - linear) - 0.4202).abs() < 0.0005,
        "{}",
        (one_for_one - linear) / (corrected - linear)
    );
}

/// **The window the restatement is handed is the window the forecast itself used.**
///
/// A guard on the seam rather than a finding: `movements` re-derives each district's projection
/// from `report::projected_series`, and if that ever stopped agreeing with what `forecast_with`
/// hands the hook, every row above would explain a total it no longer came from.
#[test]
fn the_window_a_district_is_restated_at_is_the_one_the_forecast_projects() {
    let districts = panel();
    let prior = prior(&districts);
    let factors = StatewideFactors::fy2027();
    let mut checked = 0;
    for record in &districts {
        let series = projected_series(record, HORIZON, shipped(), prior);
        let window = window_ending_at(&series, HORIZON, |p| p.point);
        // The window's last member is the count `forecast` multiplies by, and the three together
        // are the count R.C. 3317.011 funds on.
        let projected = series
            .iter()
            .find(|p| p.fiscal_year == HORIZON)
            .expect("the horizon is in the series");
        assert!((window[2] - projected.point).abs() < 1e-9);
        assert!(base_cost_adm(window) > 0.0 || record.base_cost_adm() <= 0.0);
        // And a window at the modelled count restates to the modelled figures.
        let count = record.base_cost_adm();
        let identity = restated(record, [count, count, count], Terms::BOTH, &factors);
        assert!((identity.base_cost_state_share - record.base_cost_state_share).abs() < CENT);
        checked += 1;
    }
    assert_eq!(checked, 609);
}
