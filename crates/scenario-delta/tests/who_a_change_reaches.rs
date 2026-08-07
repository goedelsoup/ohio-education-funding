//! What the winners-and-losers table says once it exists.
//!
//! The [`scenario-delta` skill](../../../.yidam/skills/scenario-delta.md) makes three claims
//! about how a funding delta has to be reported, and each was an assertion until this crate
//! could measure it. These tests are the measurements.
//!
//! Two perturbations are used throughout, chosen because they are the two levers Ohio actually
//! argues about and because they behave in opposite directions on every axis here:
//!
//! - **A base cost increase.** A uniform scale sized to a $465.0M statewide computed increase,
//!   which is what the department's own FY2027 model gives for refreshing the classroom teacher
//!   salary to FY2024. It is uniform where the real refresh is not — the real one runs from
//!   $338.62 to $471.10 per pupil because the small-district staffing minimums bind — so this is
//!   the right size and the wrong shape, and no claim below rests on the shape.
//! - **Removing the guarantee.** Exact, since the guarantee is carried per district.

use project::panel::{panel, DistrictRecord};
use project::policy::{GuaranteeRule, Policy};
use scenario_delta::{across, Axis, ScenarioDelta, Standing};

/// The statewide computed base cost increase the department's FY2027 model produces for a
/// classroom-teacher salary refresh to FY2024. Verified in `foundation`.
const COMPUTED_INCREASE: f64 = 465.0e6;

fn scale(districts: &[DistrictRecord]) -> f64 {
    let aggregate: f64 = districts.iter().map(|r| r.aggregate_base_cost).sum();
    1.0 + COMPUTED_INCREASE / aggregate
}

fn refresh(districts: &[DistrictRecord]) -> ScenarioDelta {
    ScenarioDelta::between(
        districts,
        &Policy::current_law(),
        &Policy {
            base_cost_scale: scale(districts),
            ..Policy::current_law()
        },
    )
}

fn removal(districts: &[DistrictRecord]) -> ScenarioDelta {
    ScenarioDelta::between(
        districts,
        &Policy::current_law(),
        &Policy {
            guarantee: GuaranteeRule::Removed,
            ..Policy::current_law()
        },
    )
}

/// A district paid by the guarantee under both policies receives **exactly** the same dollars —
/// not approximately, and not usually.
#[test]
fn a_perturbation_to_the_formula_moves_a_held_district_by_zero() {
    let districts = panel();
    let delta = refresh(&districts);
    let held: Vec<_> = delta
        .deltas
        .iter()
        .filter(|d| d.standing.off_formula())
        .collect();

    assert_eq!(
        held.len(),
        256,
        "districts held on the guarantee throughout"
    );
    for district in &held {
        assert_eq!(
            district.dollars(),
            0.0,
            "{} moved by {}",
            district.name,
            district.dollars()
        );
    }
    // Exactly zero, not within a tolerance: the guarantee is a `max` against a fixed baseline, so
    // when it wins under both policies the same number comes out of the same branch twice.
    assert_eq!(delta.total().reach.held_throughout, held.len());
}

/// The one lever that reaches them, and it reaches nobody else.
#[test]
fn only_a_change_to_the_guarantee_touches_the_districts_the_guarantee_pays() {
    let districts = panel();
    let delta = removal(&districts);
    let total = delta.total();

    assert_eq!(total.reach.held_throughout, 0);
    assert_eq!(total.reach.losers, 294, "every guaranteed district loses");
    assert_eq!(total.reach.gainers, 0);
    assert_eq!(
        total.reach.unmoved, 315,
        "every formula district is untouched"
    );
    // The identity in the other direction: removal takes back precisely what the guarantee pays.
    assert!(
        (total.dollars / 1e6 + 879.0).abs() < 1.0,
        "removal saves ${:.1}M",
        total.dollars / 1e6
    );
}

/// **The off-formula count is not a footnote.** Under current law a base cost increase does not
/// reach 42% of Ohio's districts, and the guarantee is the entire reason.
#[test]
fn a_base_cost_increase_does_not_reach_two_in_five_districts() {
    let districts = panel();
    let total = refresh(&districts).total();

    assert_eq!(total.reach.districts, 609);
    assert_eq!(total.reach.losers, 0, "raising base cost cannot cut anyone");
    assert_eq!(total.reach.gainers, 353);
    assert_eq!(total.reach.unmoved, 256);
    // `unmoved` and `held_throughout` are the same set here, which is worth pinning because they
    // are different quantities and only coincide when the guarantee is the sole obstruction.
    assert_eq!(total.reach.unmoved, total.reach.held_throughout);
    assert!(
        (total.reaches() - 0.580).abs() < 0.005,
        "reaches {:.3}",
        total.reaches()
    );
    // 38 districts are lifted off the guarantee by an increase this size and become formula
    // districts. That is the mechanism by which a large enough increase eventually shrinks the
    // guarantee population, and at this size it is 38 of 294.
    assert_eq!(total.reach.lifted_off, 38);
    assert_eq!(total.reach.pushed_on, 0);
}

/// **Two orderings, always** — and the reason is not decorative. For a base cost increase the
/// two tables share *no districts at all* in their top hundred.
#[test]
fn the_two_orderings_of_a_base_cost_increase_have_nothing_in_common() {
    let districts = panel();
    let delta = refresh(&districts);
    let ordering = delta.orderings();

    let overlap = |n: usize| {
        let dollars: std::collections::BTreeSet<&str> = ordering
            .by_dollars
            .iter()
            .take(n)
            .map(|d| d.irn.as_str())
            .collect();
        ordering
            .by_per_pupil
            .iter()
            .take(n)
            .filter(|d| dollars.contains(d.irn.as_str()))
            .count()
    };
    assert_eq!(overlap(10), 0);
    assert_eq!(overlap(50), 0);
    assert_eq!(overlap(100), 0, "the top hundreds are disjoint");

    // The two ends of the same policy: the largest dollar mover is a large suburban district, and
    // the hardest-hit per pupil is the smallest district in Ohio at 3.5 pupils.
    let biggest = ordering.by_dollars[0];
    let hardest = ordering.by_per_pupil[0];
    assert_ne!(biggest.irn, hardest.irn);
    assert!(biggest.adm > 20_000.0 && hardest.adm < 10.0);
    assert!(hardest.per_pupil() > biggest.per_pupil() * 4.0);
}

/// The disagreement is a property of the lever, not a law. Removing the guarantee produces two
/// orderings that overlap substantially, because a guarantee is already close to a per-pupil
/// quantity.
#[test]
fn the_orderings_agree_more_when_the_lever_is_the_guarantee() {
    let districts = panel();
    let delta = removal(&districts);
    let ordering = delta.orderings();
    let dollars: std::collections::BTreeSet<&str> = ordering
        .by_dollars
        .iter()
        .take(100)
        .map(|d| d.irn.as_str())
        .collect();
    let common = ordering
        .by_per_pupil
        .iter()
        .take(100)
        .filter(|d| dollars.contains(d.irn.as_str()))
        .count();
    assert_eq!(common, 66);
    // Still not the same table, which is why both are returned rather than one with a note.
    assert!(common < 100);
}

/// **A base cost increase is steeply progressive across wealth**, and the reason is the
/// guarantee rather than the state share floor the scenario node credits.
#[test]
fn a_base_cost_increase_reaches_poor_districts_and_not_wealthy_ones() {
    let districts = panel();
    let delta = refresh(&districts);
    let incidence = across(&delta.deltas, &Axis::valuation(), |d| d.valuation_per_pupil);

    let per_pupil: Vec<f64> = incidence.strata.iter().map(|s| s.per_pupil()).collect();
    assert!(
        (per_pupil[0] - 305.63).abs() < 1.0,
        "Q1 {:.2}",
        per_pupil[0]
    );
    assert!((per_pupil[4] - 20.81).abs() < 1.0, "Q5 {:.2}", per_pupil[4]);
    assert!(
        (incidence.gradient().expect("Q1 gets money") - 0.068).abs() < 0.005,
        "the wealthiest quintile receives 6.8% per pupil of what the poorest does"
    );

    // The mechanism, visible in the same table: the guarantee removes wealthy districts from the
    // formula's reach almost entirely, and touches the poorest quintile hardly at all.
    let held: Vec<usize> = incidence.strata.iter().map(|s| s.held_throughout).collect();
    assert_eq!(held, vec![10, 31, 39, 88, 88]);
    assert!(
        held[4] > held[0] * 8,
        "guarantee dependence rises steeply with wealth"
    );
}

/// The mirror image, and the more politically loaded reading of the same fact: **removing the
/// guarantee costs wealthy districts about seven times per pupil what it costs poor ones.**
#[test]
fn removing_the_guarantee_falls_hardest_on_wealthy_districts() {
    let districts = panel();
    let delta = removal(&districts);
    let incidence = across(&delta.deltas, &Axis::valuation(), |d| d.valuation_per_pupil);

    let per_pupil: Vec<f64> = incidence.strata.iter().map(|s| s.per_pupil()).collect();
    for band in &per_pupil {
        assert!(*band < 0.0, "every band loses");
    }
    assert!(
        (per_pupil[0] + 136.84).abs() < 2.0,
        "Q1 {:.2}",
        per_pupil[0]
    );
    assert!(
        (per_pupil[3] + 1025.91).abs() < 2.0,
        "Q4 {:.2}",
        per_pupil[3]
    );
    assert!(
        (incidence.gradient().expect("Q1 loses money") - 6.85).abs() < 0.1,
        "gradient {:?}",
        incidence.gradient()
    );

    // Not monotone at the top: Q4 loses more per pupil than Q5. The very wealthiest districts
    // are small and their formula amounts are already floored, so there is less distance between
    // formula and baseline to take away. A claim that the guarantee's benefit rises without
    // limit in wealth is wrong, and this is where it breaks.
    assert!(
        per_pupil[4] > per_pupil[3],
        "Q5 loses less per pupil than Q4"
    );
}

/// The state share axis, and the mass point that equal-count binning cannot represent.
#[test]
fn the_lowest_state_share_band_is_exactly_the_districts_on_the_floor() {
    let districts = panel();
    let delta = refresh(&districts);
    let incidence = across(&delta.deltas, &Axis::state_share(), |d| {
        Some(d.state_share_fraction)
    });

    let floor = &incidence.strata[0];
    assert_eq!(floor.districts, 138, "the minimum state share population");
    assert_eq!(
        floor.districts,
        districts
            .iter()
            .filter(|r| r.at_minimum_state_share())
            .count(),
        "the band is the floor, not an approximation of it"
    );
    // Uneven by construction and correctly so: forcing equal counts would split the floor.
    let counts: Vec<usize> = incidence.strata.iter().map(|s| s.districts).collect();
    assert_eq!(counts, vec![138, 105, 122, 122, 122]);
    assert_eq!(counts.iter().sum::<usize>(), districts.len());

    // And the incidence across it is the sharpest gradient in this file: a district at the floor
    // collects $9.35 per pupil from an increase that pays a high-state-share district $273.50.
    assert!((floor.per_pupil() - 9.35).abs() < 0.5);
    assert!((incidence.strata[4].per_pupil() - 273.50).abs() < 1.0);
}

/// Where a computed base cost increase goes, and the two mechanisms between "computed" and
/// "delivered" that a base cost figure alone does not show.
///
/// The corpus's own estimate of what a refresh delivers applies the guarantee and **not** the
/// minimum state share — recorded as an open item on the scenario node. This is that item,
/// measured: the floor is not a small correction.
#[test]
fn less_than_half_of_a_computed_base_cost_increase_is_delivered_as_aid() {
    let districts = panel();
    let bump = scale(&districts) - 1.0;
    let law = Policy::current_law();
    let scaled = Policy {
        base_cost_scale: scale(&districts),
        ..law
    };
    let unguaranteed = Policy {
        guarantee: GuaranteeRule::Removed,
        ..law
    };

    let computed: f64 = districts.iter().map(|r| r.aggregate_base_cost * bump).sum();
    // What every district would collect if the whole per-pupil increase passed through.
    let full_pass: f64 = districts
        .iter()
        .map(|r| r.base_cost_per_pupil * bump * r.current_year_adm)
        .sum();
    // What the formula produces once the floor applies, with the guarantee out of the way.
    let formula = ScenarioDelta::between(
        &districts,
        &unguaranteed,
        &Policy {
            guarantee: GuaranteeRule::Removed,
            ..scaled
        },
    )
    .total()
    .dollars;
    let delivered = ScenarioDelta::between(&districts, &law, &scaled)
        .total()
        .dollars;

    assert!((computed / 1e6 - 465.0).abs() < 0.1);
    // Base cost is computed on a three-year average ADM and the state share is paid on the
    // current year, so a shrinking state does not collect the whole aggregate increase.
    assert!(
        ((computed - full_pass) / 1e6 - 7.6).abs() < 0.5,
        "ADM denominator mismatch ${:.1}M",
        (computed - full_pass) / 1e6
    );
    assert!(
        ((full_pass - formula) / 1e6 - 118.7).abs() < 1.0,
        "the minimum state share withholds ${:.1}M",
        (full_pass - formula) / 1e6
    );
    assert!(
        ((formula - delivered) / 1e6 - 141.6).abs() < 1.0,
        "the guarantee absorbs ${:.1}M",
        (formula - delivered) / 1e6
    );
    assert!(
        (delivered / 1e6 - 197.1).abs() < 1.0,
        "delivered ${:.1}M",
        delivered / 1e6
    );
    assert!(
        (delivered / computed - 0.424).abs() < 0.005,
        "42% of a computed base cost increase becomes state aid"
    );
}

/// The two obstructions overlap almost entirely at the top of the wealth distribution, so they
/// cannot be added together as though they were independent.
#[test]
fn the_floor_and_the_guarantee_are_the_same_districts_where_they_bind() {
    let districts = panel();
    let mut by_wealth: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|r| r.valuation_per_pupil.is_some())
        .collect();
    by_wealth.sort_by(|a, b| {
        a.valuation_per_pupil
            .partial_cmp(&b.valuation_per_pupil)
            .expect("no NaN valuations")
    });

    let top = &by_wealth[by_wealth.len() * 4 / 5..];
    let held = top.iter().filter(|r| r.on_guarantee()).count();
    let floored = top.iter().filter(|r| r.at_minimum_state_share()).count();
    let both = top
        .iter()
        .filter(|r| r.on_guarantee() && r.at_minimum_state_share())
        .count();
    assert_eq!((held, floored, both), (90, 95, 73));

    let bottom = &by_wealth[..by_wealth.len() / 5];
    assert_eq!(
        bottom.iter().filter(|r| r.at_minimum_state_share()).count(),
        0,
        "no district in the poorest quintile is anywhere near the floor"
    );
    assert_eq!(bottom.iter().filter(|r| r.on_guarantee()).count(), 15);
}

/// **No cut, however deep, pushes a district onto the guarantee — and that is a hole in the
/// model rather than a fact about Ohio.**
///
/// This test was written to exercise [`Standing::PushedOn`] and instead established that the
/// variant is unreachable. The reason is in the panel: a guarantee baseline is that district's
/// FY2020 receipt, and the model reveals it *only for districts currently on the guarantee* —
/// see `DistrictRecord::guarantee_baseline`, which returns `None` otherwise, because the
/// guarantee is the only thing that discloses the figure.
///
/// So the 315 districts the formula pays are modelled with **no floor beneath them at all**.
/// Every one of them has an FY2020 baseline; it simply sits below their current formula amount,
/// which is why they are on formula. A simulated cut lets them fall past it.
#[test]
fn a_cut_lets_formula_districts_fall_through_a_floor_the_model_cannot_see() {
    let districts = panel();
    for phase_in in [0.5, 0.25, 0.05] {
        let delta = ScenarioDelta::between(
            &districts,
            &Policy::current_law(),
            &Policy {
                phase_in_base_cost: phase_in,
                phase_in_categorical: phase_in,
                ..Policy::current_law()
            },
        );
        let total = delta.total();
        assert_eq!(
            total.reach.pushed_on, 0,
            "at a {phase_in} phase-in the model still catches nobody"
        );
        assert_eq!(
            total.reach.losers, 315,
            "every formula district, and only those"
        );
        assert_eq!(total.reach.held_throughout, 294);

        // The formula district falls by the full cut. A real one would stop at its FY2020 level.
        let worst = delta
            .deltas
            .iter()
            .filter(|d| d.standing == Standing::FormulaThroughout)
            .filter_map(scenario_delta::Delta::percent)
            .fold(0.0_f64, f64::min);
        assert!(
            (worst + (1.0 - phase_in)).abs() < 0.001,
            "worst formula district lost {worst:.3} at a {phase_in} phase-in"
        );
    }

    // What that costs the reader of a cut scenario: the districts with no modelled floor hold
    // 46% of Ohio's students, so a simulated cut overstates its own savings by an amount this
    // corpus cannot bound. `PushedOn` is kept in the enum for that reason — deleting it would
    // remove the only place the gap is named.
    let unfloored: f64 = districts
        .iter()
        .filter(|r| !r.on_guarantee())
        .map(|r| r.current_year_adm)
        .sum();
    let all: f64 = districts.iter().map(|r| r.current_year_adm).sum();
    assert!(
        (unfloored / all - 0.461).abs() < 0.005,
        "unfloored share of ADM {:.3}",
        unfloored / all
    );
}
