//! Every bound in the modelled formula, and which of them are ever the operative term.
//!
//! # The question
//!
//! Issue #410, after #389 found by accident that R.C. 3317.011(F)(6)(c) states a floor that can
//! never be the operative term. How many others are there? Two predicates, kept apart because
//! they are different kinds of claim:
//!
//! - **cannot bind** — unreachable by construction, provable without data, a drafting fact;
//! - **has never bound** — reachable in principle and reached by no district in a committed
//!   year, a property of Ohio, and the more useful half of a repeal argument.
//!
//! The issue asked that a null result be reported as one, and that neither list be reported as
//! the other.
//!
//! # The answer
//!
//! **Thirty-eight bounds. (F)(6)(c) is the only one that cannot bind, and the second list is
//! empty.** Every reachable bound in the modelled formula is reached by at least one of the 609
//! districts in the department's FY2027 model. There is no policy written into the plan that has
//! never been applied; there is one sentence with no force.
//!
//! [`project::bounds`] is the census, one variant per bound, each stating its authority, its
//! operator, whether any input can reach it, and which side of the comparison the count is for.
//! The table it produces:
//!
//! | family | bound | on it |
//! |---|---|--:|
//! | base cost | six special teachers | 166 |
//! | | one guidance counselor | 278 |
//! | | five wellness and success staff | 276 |
//! | | two other district administrators | 328 |
//! | | two fiscal support staff | 372 |
//! | | one EMIS support employee | 554 |
//! | | **one leadership support staff — cannot bind** | **0** |
//! | | one building support staff per open building | 291 |
//! | | thirty-five fiscal support staff, ceiling | 3 |
//! | | three building support staff per building, ceiling | 2 |
//! | | size-banded salaries, small end (< 500 ADM) | 46 |
//! | | size-banded salaries, large end (> 4,000 ADM) | 78 |
//! | | base cost enrolled ADM, single year over the average | 105 |
//! | local capacity | capacity valuation, recent year over the average | 7 |
//! | | capacity income, recent year over the average | 12 |
//! | | capacity percentage capped at 0.025 — **fixed at the fortieth rank** | 40 |
//! | | minimum state share, 10% | 138 |
//! | categoricals | DPIA blended count capped at enrolled ADM | **1** |
//! | | gifted coordinator units, floor of 0.5 | 370 |
//! | | gifted coordinator units, ceiling of 8 | 3 |
//! | | K-8 gifted specialist units, floor of 0.3 | 101 |
//! | | 9-12 gifted specialist units, floor of 0.3 | 187 |
//! | | capacity tier zero at or above the median wealth | 304 |
//! | | capacity tier zero under 200 ADM | 5 |
//! | | capacity tier paid a fraction, 200 to 600 ADM | 68 |
//! | | wealth tier zero below an index of 0.8 | 171 |
//! | guarantee | the guarantee's floor clamped at zero | **1** |
//! | | the guarantee, holding the district | 294 |
//! | | decrease threshold, the floor of twenty governing | 499 |
//! | | the clawback clamped by a smaller guarantee | 22 of 43 |
//! | | the formula transition supplement, holding the district | 144 |
//! | transportation | state share paid at the 50% floor | 434 of 604 |
//! | | mile base governing over the rider base | 350 of 604 |
//! | | efficiency adjustment zero below an index of 1.0 | 198 of 604 |
//! | | efficiency adjustment capped at 15% | 61 of 604 |
//! | | density supplement zero at 28 riders per square mile | 212 of 604 |
//! | | transportation's own guarantee, holding the district | 38 |
//! | | special education transportation at the 50% floor | 411 of 563 |
//!
//! # Three things in the table worth a sentence each
//!
//! **Two bounds are reached by exactly one district**, which is the extreme case of reachable
//! and barely reached. The department's cap of the DPIA blended count at enrolled ADM reaches
//! Edgerton Local alone — 116 districts report more economically disadvantaged pupils than
//! enrolled ones, and Edgerton is the one whose 65/35 blend still exceeds its enrolment. R.C.
//! 3317.019(A)(1)'s clamp of the guarantee's floor at zero reaches Richmond Heights Local alone,
//! whose FY2020 funding base is negative after the deductions R.C. 3317.02(N)(1)(b) subtracts. A
//! third is nearly so: the clawback's clamp binds for 22 of the 43 districts charged and for one
//! of them, West Muskingum Local, only partially — `what_the_clawback_charges_and_what_it_takes.rs`.
//!
//! **One bound's population is fixed by construction.** R.C. 3317.017(A)(4)(d)(i) caps the local
//! capacity percentage at 0.025 for every district whose income ratio is at or above the
//! fortieth highest district's, so the cap is on exactly 40 districts whatever Ohio's incomes
//! are, ties aside. It is not a fact about the distribution; it is a rank wearing a ceiling.
//!
//! **A count of districts on a bound is not the count it moves.** The corpus already knew this
//! for the minimum state share — 138 on it, 31 moved, the guarantee absorbing the rest — and the
//! census adds one more instance. R.C. 3317.019(C)(1)'s floor of twenty pupils under the
//! clawback's decrease threshold *governs* the threshold for 499 districts, because ten per cent
//! of last year's open-enrolment count is under twenty for any district with fewer than 200 such
//! pupils. It *spares* **71** of them: districts that lost more than ten per cent and not more
//! than twenty, who would be charged under the percentage alone and are charged nothing.
//!
//! # What is not in the table
//!
//! Steps — a two-branch test on a boolean or a rank, which a `max` is not. Athletics eligibility
//! at R.C. 3317.011(A)(11) is dormant (all 609 qualify); the enrolment growth supplement's 3%
//! gate is cleared by 43; the performance supplement gates on ratings; the repealed supplemental
//! targeted assistance tier gates 36 districts into a payment of nothing. Each is measured in its
//! own file. And prorations, which are one `min` taken statewide rather than 609 comparisons.
//!
//! # The caution the issue asked for
//!
//! Every "cannot bind" is relative to the section as written. (F)(6)(c) is unreachable *because*
//! (F)(3)(c) floors its input; `foundation::minimums::the_leadership_support_minimum_cannot_bind_at_any_enrolment`
//! asserts it and this file re-asserts it over the panel. Strike (F)(3)(c) out and (F)(6)(c)
//! binds at once for every small district. The census reports what the section does, and a
//! counterfactual that removes one bound has to re-run it.

use foundation::minimums::{Ceiling, Minimum};
use project::bounds::{self, Bound, Family, Operator, Reach};
use project::panel::{panel, DistrictRecord, DPIA_BLEND};

/// The whole table, pinned.
#[test]
fn thirty_eight_bounds_and_where_each_one_is_the_operative_term() {
    let districts = panel();
    assert_eq!(districts.len(), 609);
    let rows = bounds::census(&districts);
    assert_eq!(rows.len(), 38);
    assert_eq!(Bound::all().len(), 38, "one row per bound");

    let expected: [(Bound, usize, usize); 38] = [
        (Bound::StaffingFloor(Minimum::SpecialTeachers), 609, 166),
        (Bound::StaffingFloor(Minimum::Counselors), 609, 278),
        (Bound::StaffingFloor(Minimum::Wellness), 609, 276),
        (Bound::StaffingFloor(Minimum::OtherAdministrators), 609, 328),
        (Bound::StaffingFloor(Minimum::FiscalSupport), 609, 372),
        (Bound::StaffingFloor(Minimum::Emis), 609, 554),
        (Bound::StaffingFloor(Minimum::LeadershipSupport), 609, 0),
        (Bound::StaffingFloor(Minimum::BuildingSupport), 609, 291),
        (Bound::StaffingCeiling(Ceiling::FiscalSupport), 609, 3),
        (Bound::StaffingCeiling(Ceiling::BuildingSupport), 609, 2),
        (Bound::SizeBandSmall, 609, 46),
        (Bound::SizeBandLarge, 609, 78),
        (Bound::BaseCostEnrolledAdm, 609, 105),
        (Bound::ValuationLesserOf, 609, 7),
        (Bound::IncomeLesserOf, 609, 12),
        (Bound::CapacityRateCeiling, 609, 40),
        (Bound::MinimumStateShare, 609, 138),
        (Bound::DpiaCountCap, 609, 1),
        (Bound::GiftedCoordinatorFloor, 609, 370),
        (Bound::GiftedCoordinatorCeiling, 609, 3),
        (Bound::GiftedSpecialistK8Floor, 609, 101),
        (Bound::GiftedSpecialist912Floor, 609, 187),
        (Bound::CapacityTierZero, 609, 304),
        (Bound::CapacityTierSizeCutoff, 609, 5),
        (Bound::CapacityTierSizeRamp, 609, 68),
        (Bound::WealthTierZero, 609, 171),
        (Bound::FundingBaseClampAtZero, 609, 1),
        (Bound::Guarantee, 609, 294),
        (Bound::DecreaseThresholdFloor, 609, 499),
        (Bound::ClawbackClampAtZero, 43, 22),
        (Bound::TransitionSupplement, 609, 144),
        (Bound::TransportationFloor, 604, 434),
        (Bound::MileBase, 604, 350),
        (Bound::EfficiencyZero, 604, 198),
        (Bound::EfficiencyCeiling, 604, 61),
        (Bound::DensityZero, 604, 212),
        (Bound::TransportationGuarantee, 609, 38),
        (Bound::SpecialEducationTransportFloor, 563, 411),
    ];
    for (row, (bound, population, operative)) in rows.iter().zip(expected) {
        assert_eq!(row.bound, bound, "the census is in statutory order");
        assert_eq!(
            (row.population, row.operative),
            (population, operative),
            "{}: {} of {}",
            bound.label(),
            row.operative,
            row.population
        );
    }

    // Every family is represented, and the operator vocabulary is the whole of it.
    for family in [
        Family::BaseCost,
        Family::LocalCapacity,
        Family::Categoricals,
        Family::Guarantee,
        Family::Transportation,
    ] {
        assert!(rows.iter().any(|r| r.bound.family() == family));
    }
    for operator in [
        Operator::Floor,
        Operator::Ceiling,
        Operator::ClampAtZero,
        Operator::GreaterOf,
        Operator::LesserOf,
    ] {
        assert!(rows.iter().any(|r| r.bound.operator() == operator));
    }
}

/// **The first list has one entry, and it is the one #389 found.**
#[test]
fn exactly_one_bound_cannot_bind_and_it_is_the_leadership_support_floor() {
    assert_eq!(
        bounds::cannot_bind(),
        vec![Bound::StaffingFloor(Minimum::LeadershipSupport)]
    );
    assert_eq!(
        Bound::StaffingFloor(Minimum::LeadershipSupport).reach(),
        Reach::Unreachable
    );
    assert_eq!(
        Bound::StaffingFloor(Minimum::LeadershipSupport).authority(),
        "R.C. 3317.011(F)(6)(c)"
    );

    // Re-asserted over the panel rather than over the function, so a reader who distrusts the
    // algebra has the 609 districts instead.
    let districts = panel();
    assert!(districts
        .iter()
        .all(|d| !Bound::StaffingFloor(Minimum::LeadershipSupport).binds(d)));
}

/// **The second list is empty.** Every reachable bound is reached, so there is no provision in
/// the modelled formula that Ohio has written and never applied.
#[test]
fn no_reachable_bound_is_reached_by_nobody() {
    let districts = panel();
    assert_eq!(bounds::never_bound(&districts), Vec::<Bound>::new());

    // Stated the other way round, so that the assertion above is not vacuous: every row whose
    // bound is reachable has a positive population and a positive count.
    let rows = bounds::census(&districts);
    for row in &rows {
        if row.bound.reach() == Reach::Unreachable {
            continue;
        }
        assert!(
            row.population > 0 && row.operative > 0,
            "{} is reachable and reached by {} of {}",
            row.bound.label(),
            row.operative,
            row.population
        );
    }
    assert_eq!(rows.iter().filter(|r| r.never_bound()).count(), 0);
    assert_eq!(
        rows.iter().filter(|r| r.operative == 0).count(),
        1,
        "the only zero in the table is the unreachable one"
    );
}

/// **Two bounds are reached by exactly one district each**, and both are named.
#[test]
fn two_bounds_are_reached_by_exactly_one_district() {
    let districts = panel();
    assert_eq!(
        bounds::reached_by_one(&districts),
        vec![
            (Bound::DpiaCountCap, "Edgerton Local".to_string()),
            (
                Bound::FundingBaseClampAtZero,
                "Richmond Heights Local".to_string()
            ),
        ]
    );

    // The DPIA cap is reachable because the economically disadvantaged count is on a different
    // basis from enrolled ADM and exceeds it in 116 districts; the 65/35 blend with the smaller
    // direct-certification count pulls all but one back under.
    let over: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|d| d.dpia.economically_disadvantaged_adm > d.categorical_enrolled_adm)
        .collect();
    assert_eq!(over.len(), 116);
    let edgerton = districts
        .iter()
        .find(|d| d.name == "Edgerton Local")
        .expect("Edgerton is in the panel");
    assert!(edgerton.dpia.count_at(DPIA_BLEND.1) > edgerton.categorical_enrolled_adm);
    assert!(
        (edgerton.dpia.weighted_adm - edgerton.categorical_enrolled_adm).abs() < 0.01,
        "and the department published the capped count"
    );

    // Richmond Heights is the one district whose funding base is negative, so the guarantee's
    // floor would be negative without the clamp and is zero with it.
    let richmond = districts
        .iter()
        .find(|d| d.name == "Richmond Heights Local")
        .expect("Richmond Heights is in the panel");
    assert!(richmond.transition.funding_base < 0.0);
    assert_eq!(richmond.guarantee_floor(), 0.0);
    assert_eq!(
        districts
            .iter()
            .filter(|d| d.transition.funding_base < 0.0)
            .count(),
        1
    );
}

/// **The capacity rate ceiling is on forty districts by construction**, because it is stated
/// against the fortieth-ranked district and not against a value.
#[test]
fn the_capacity_rate_ceiling_binds_for_the_fortieth_rank_and_no_other_number() {
    let districts = panel();
    assert_eq!(
        Bound::CapacityRateCeiling.reach(),
        Reach::Fixed(local_capacity::BENCHMARK_RANK)
    );
    assert_eq!(
        districts
            .iter()
            .filter(|d| Bound::CapacityRateCeiling.binds(d))
            .count(),
        40
    );
    assert!(bounds::capacity_ceiling_is_the_rank(&districts));
    assert_eq!(
        bounds::census(&districts)
            .iter()
            .filter(|r| matches!(r.bound.reach(), Reach::Fixed(_)))
            .count(),
        1,
        "and it is the only bound stated against a rank"
    );
}

/// **On a bound is not moved by it**, measured for the decrease threshold's floor of twenty.
#[test]
fn the_floor_of_twenty_governs_499_thresholds_and_spares_71_districts() {
    let districts = panel();
    assert_eq!(
        districts
            .iter()
            .filter(|d| Bound::DecreaseThresholdFloor.binds(d))
            .count(),
        499
    );
    assert_eq!(
        bounds::spared_by_the_decrease_threshold_floor(&districts),
        71
    );

    // None of the 71 is charged: that is what "spared" means, and it is the check that the
    // count is of districts the floor changed rather than districts it merely governs.
    let spared_and_charged = districts
        .iter()
        .filter(|d| {
            let lost = d.transition.open_enrollment_lost();
            lost > d.transition.open_enrollment_prior * 0.1
                && lost <= 20.0
                && d.transition.open_enrollment_adjustment > 0.0
        })
        .count();
    assert_eq!(spared_and_charged, 0);

    // The clamp on the charge itself: 22 of 43, of which one is the partial case the crate's own
    // record notes — West Muskingum, charged more than the guarantee it had.
    let charged: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|d| Bound::ClawbackClampAtZero.in_population(d))
        .collect();
    assert_eq!(charged.len(), 43);
    let clamped: Vec<&DistrictRecord> = charged
        .iter()
        .copied()
        .filter(|d| Bound::ClawbackClampAtZero.binds(d))
        .collect();
    assert_eq!(clamped.len(), 22);
    let partial: Vec<&str> = clamped
        .iter()
        .filter(|d| d.guarantee_before_clawback() > 0.0)
        .map(|d| d.name.as_str())
        .collect();
    assert_eq!(partial, vec!["West Muskingum Local"]);
}

/// **The three rules against a three-year average** put most of the state on the average and a
/// handful on the single year — 105, 7 and 12 — and the label says which side is counted.
#[test]
fn the_single_year_branches_are_the_minority_on_all_three_averaging_rules() {
    let districts = panel();
    assert_eq!(bounds::single_year_branches(&districts), (105, 7, 12));
    for bound in [
        Bound::BaseCostEnrolledAdm,
        Bound::ValuationLesserOf,
        Bound::IncomeLesserOf,
    ] {
        assert!(bound.label().contains("over the three-year average"));
        assert!(matches!(
            bound.operator(),
            Operator::GreaterOf | Operator::LesserOf
        ));
    }
    // The valuation split reproduces what `local-capacity` recorded from the department's own
    // sheet: 602 lagged, 7 immediate.
    assert_eq!(
        districts
            .iter()
            .filter(|d| !Bound::ValuationLesserOf.binds(d))
            .count(),
        602
    );
}

/// **The one step inside R.C. 3317.011 is dormant**, which is why it is not in the table.
#[test]
fn the_athletics_step_is_dormant_and_is_not_a_bound() {
    let districts = panel();
    assert!(bounds::athletics_step_is_dormant(&districts));
    assert!(Bound::all().iter().all(|b| !b.label().contains("athletic")));
}

/// Every bound cites a provision, and the provisions the committed Revised Code holds say what
/// the census says they say.
#[test]
fn every_bound_names_its_authority_and_the_statute_states_each_one() {
    let statute = include_str!("../fixtures/revised-code.txt");
    for bound in Bound::all() {
        assert!(
            !bound.authority().is_empty(),
            "{bound:?} names no authority"
        );
        assert!(!bound.label().is_empty());
    }
    for phrase in [
        // R.C. 3317.02(C): the greater of the previous year and the three-year average.
        "\"base cost enrolled ADM\" for a fiscal year means the greater of the following",
        // R.C. 3317.017(A)(1)(a) and (A)(2)(a): the minimum of recent and three-year average.
        "Determine the minimum of the district's three-year average valuation",
        // R.C. 3317.017(A)(4)(d)(i): the ceiling at the fortieth highest ratio.
        "the district with the fortieth highest ratio",
        // R.C. 3317.017(C): the 10% floor.
        "If the result is less than 0.10, the state share percentage shall be 0.10.",
        // R.C. 3317.051(A)(1)(a)-(c): the gifted unit floors and cap.
        "with a minimum of 0.5 units and a maximum of 8 units",
        "with a minimum of 0.3 units allocated for the district",
        // R.C. 3317.0217(B)(4)(a): the capacity tier's two zeroes.
        "The district's capacity index is less than 1.",
        "The district's enrolled ADM is less than 200.",
        // R.C. 3317.0217(C)(4)(a): the wealth tier's zero.
        "is less than 0.8, the district's wealth amount for that fiscal year shall be zero",
        // R.C. 3317.019(A)(1)-(2) and (C)(1)-(2): the guarantee's clamps and the threshold.
        "results in a negative number, the district's funding under division (A)(1)",
        "results in a negative number, the district's funding under division (A)(2)",
        "(a) Twenty;",
        "At no time, however, shall the amount paid to a district under division (A) of this \
         section be less than zero.",
        // R.C. 3317.0212(E)(1)(c) and (F)(3): the transportation base and the efficiency band.
        "Multiply the greater of the amounts calculated under divisions (E)(1)(a) and (b)",
        "If the district's efficiency index is equal to or greater than 1.5",
        "If the district's efficiency index is less than 1.0, the efficiency adjustment payment \
         shall be zero.",
    ] {
        assert!(
            statute.contains(phrase),
            "the committed Revised Code no longer says {phrase:?}"
        );
    }
}
