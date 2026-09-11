//! The one parameter in the plan that moves without an act, across the one interval it can be
//! observed over.
//!
//! R.C. 3317.0212(C) and (D) name no dollar figure. Both transportation rates are **trimmed means
//! of what districts reported spending in the prior fiscal year**, so they are statistics over
//! district behaviour rather than decisions, and they move when that behaviour moves. Everything
//! else the corpus calls a parameter changes because somebody changed it.
//!
//! Two observations is the whole series, and the shortness is a finding rather than a gap: the
//! department publishes one calculator and replaces it in place, so FY2026 survives only in the
//! Internet Archive and FY2022 to FY2025 do not survive at all.
//!
//! # What one interval buys
//!
//! **The two rates move in opposite directions.** Per weighted rider, +4.88%. Per mile, −2.73%.
//! They are computed from the same districts' returns for the same year by the same trimming
//! rule, and they disagree about which way costs went.
//!
//! That matters because of what sits between them: between the two cost years these rates are
//! drawn from, Midwest diesel rose **17.5%**. The rider rate rose about a quarter as fast; the
//! mile rate fell. Whatever the mile base tracks, it is not fuel in any direct way — and the mile
//! base is what **350 districts and $308.8m of school bus payment** are actually paid on.
//!
//! The likely mechanism is that a per-mile rate is a *ratio* whose denominator is also moving:
//! districts driving more miles push the statewide cost-per-mile down even while total cost
//! rises. Two observations cannot confirm that and this file does not claim it.
//!
//! # And the appropriation began binding
//!
//! Special education transportation is multiplied by a proration factor, and the factor was
//! **1.0 in FY2026 and 0.917459740976215 in FY2027**. A published amount under a proration is not
//! what the formula says a district is owed; it is what was available, divided. The department's
//! FY2026 line-by-line explanation added the same language to the density supplement a year
//! earlier — see `the_department_explains_its_own_method`. Two transportation lines, two
//! consecutive years, the same pressure.
//!
//! # And what the factor divides by
//!
//! This file states the two proration factors and leaves them as observations. What each one is a
//! quotient *of* — the enacted earmark over a statewide allocation that reaches past the
//! calculator's own districts — is
//! [`the_denominator_that_is_larger_than_the_model`](the_denominator_that_is_larger_than_the_model.rs).
//! The fixture reader this file used to declare privately moved to
//! [`project::transport`](../src/transport.rs) when a third caller wanted it.

use project::transport::{rates, rates_for, Rates};

fn year(fy: u16) -> Rates {
    rates_for(fy).unwrap_or_else(|| panic!("FY{fy} is not in the committed series"))
}

/// The series is two years, and the values are the department's own.
///
/// FY2027's two rates are the ones `transportation.rs` reproduces across 609 districts, so this
/// also pins that the extractor and the hand-entered constants agree.
#[test]
fn the_series_is_two_years_and_carries_the_departments_own_figures() {
    let all = rates();
    assert_eq!(
        all.iter().map(|r| r.fiscal_year).collect::<Vec<_>>(),
        vec![2026, 2027],
        "FY2026 and FY2027 are the only years any published calculator survives for"
    );

    let (fy26, fy27) = (year(2026), year(2027));
    assert!((fy26.per_rider - 1275.0).abs() < 0.001, "FY2026 per rider");
    assert!((fy26.per_mile - 7.06).abs() < 0.001, "FY2026 per mile");
    assert!(
        (fy27.per_rider - project::panel::TRANSPORT_PER_RIDER).abs() < 0.001,
        "FY2027's extracted rate should equal the constant the panel is checked against"
    );
    assert!(
        (fy27.per_mile - project::panel::TRANSPORT_PER_MILE).abs() < 0.001,
        "and so should the mile rate"
    );
}

/// The two rates disagree about which way costs went.
///
/// The finding this file exists for. Stated as measured bounds rather than as a direction, so
/// that a later year narrowing or reversing it fails here rather than passing quietly.
#[test]
fn the_rider_rate_rose_and_the_mile_rate_fell_over_the_same_interval() {
    let (fy26, fy27) = (year(2026), year(2027));

    let rider = (fy27.per_rider - fy26.per_rider) / fy26.per_rider;
    let mile = (fy27.per_mile - fy26.per_mile) / fy26.per_mile;

    assert!(
        (0.048..=0.050).contains(&rider),
        "the per-rider rate should rise about 4.88%; it moves {:.2}%",
        rider * 100.0
    );
    assert!(
        (-0.028..=-0.026).contains(&mile),
        "the per-mile rate should fall about 2.73%; it moves {:.2}%",
        mile * 100.0
    );
    assert!(
        rider > 0.0 && mile < 0.0,
        "the two bases should disagree in sign; if they stop disagreeing, the argument that \
         the mile base does not track fuel needs re-making rather than restating"
    );
}

/// Special education transportation was whole in FY2026 and prorated in FY2027.
#[test]
fn the_appropriation_began_binding_on_special_education_transportation_in_fy2027() {
    let (fy26, fy27) = (year(2026), year(2027));

    assert!(
        (fy26.sped_proration - 1.0).abs() < 1e-9,
        "FY2026 paid the computed entitlement in full; it prorates at {}",
        fy26.sped_proration
    );
    assert!(
        fy27.sped_proration < 0.92 && fy27.sped_proration > 0.91,
        "FY2027 should prorate to about 0.9175; it is {}",
        fy27.sped_proration
    );
    assert!(
        (fy27.sped_proration - project::panel::TRANSPORT_SPED_PRORATION).abs() < 1e-9,
        "and it should be the factor the panel already carries"
    );
}

/// The state share floor is a phase-in, and both endpoints of the observable interval are on it.
#[test]
fn the_minimum_state_share_steps_by_one_twenty_fourth_a_year() {
    let (fy26, fy27) = (year(2026), year(2027));

    assert!(
        (fy26.minimum_state_share - 0.4583).abs() < 1e-4,
        "FY2026 is 45.83%"
    );
    assert!(
        (fy27.minimum_state_share - 0.50).abs() < 1e-9,
        "FY2027 is 50%"
    );

    // 4.1667 points a year, which is 1/24 — the phase-in H.B. 96 records as 41.67 → 45.83 → 50.
    let step = fy27.minimum_state_share - fy26.minimum_state_share;
    assert!(
        (step - 1.0 / 24.0).abs() < 1e-3,
        "the floor should step by a twenty-fourth; it steps {step:.5}"
    );
}

/// An inflation factor of 5% sits beside the rates in both years and is in no section.
///
/// Pinned rather than explained. It belongs to the same category as the FY2021 guarantee and the
/// 180-day multiplier — alive in the calculator, absent from permanent law — and the corpus
/// carries it as `[open]`. A test is here so that the day it stops being 0.05 is visible.
#[test]
fn an_inflation_factor_of_five_percent_is_in_the_model_and_in_no_statute() {
    for rate in rates() {
        assert!(
            (rate.inflation_factor - 0.05).abs() < 1e-9,
            "FY{}: the inflation factor is {}, not the 0.05 both models carry",
            rate.fiscal_year,
            rate.inflation_factor
        );
    }
}
