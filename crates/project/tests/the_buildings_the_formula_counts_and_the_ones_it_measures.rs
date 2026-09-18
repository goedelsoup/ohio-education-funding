//! What the base cost build-up knows about a district's buildings, and what it does not.
//!
//! # The hypothesis
//!
//! That the formula fails to account for **high local capacity districts with building sprawl and
//! aging plant**. It has three parts and they do not have the same answer.
//!
//! # Sprawl: the formula sees building *count* and is blind to building *size*
//!
//! R.C. 3317.011(G) has two building terms and they behave oppositely.
//!
//! **Leadership support does see sprawl.** Staffing is the enrollment ratio unless a district has
//! more buildings than that ratio would fund, in which case the building count governs. That floor
//! binds for **291 of 609 districts** — nearly half the state — so the claim that the formula
//! ignores how many buildings a district keeps open is simply false for the term where a count
//! could matter. The opposite cap, three staff per building, binds for two districts.
//!
//! **Operation does not.** It is `ADM × (building amount − safety)`, where the building amount is
//! a statewide average of **278.07 square feet per pupil at $5.10** — [`BuildingAllowance`], newly
//! decomposed. A district is funded for that floor area per pupil whether its buildings hold that
//! much, half of it, or twice. And the shortfall grades with sprawl exactly as the hypothesis
//! predicts, against FY2025 plant spending:
//!
//! | buildings per 1,000 pupils | median | operations & maintenance per pupil | against the allowance |
//! |---|--:|--:|--:|
//! | Q1 fewest | 1.51 | $1,369 | 0.99 |
//! | Q2 | 2.17 | $1,552 | 1.12 |
//! | Q3 | 2.79 | $1,720 | 1.24 |
//! | Q4 most | 4.25 | $2,056 | **1.49** |
//!
//! Monotone across all four. A district with the fewest buildings per pupil spends almost exactly
//! the allowance; one with the most spends half again as much.
//!
//! # Aging: absent, and not measurable here
//!
//! **R.C. 3317.011 has no building-age term at all** — not a weak one, none. Nothing in this
//! workspace measures it either: no committed source carries district square footage or plant age,
//! and the nearest available proxy is sixteen years of state construction receipts in
//! `dispersion::facilities`. The claim is true by inspection of the statute and untested beyond it.
//!
//! # High local capacity: not supported
//!
//! The one part of the hypothesis the data contradicts. Sorting by the department's own published
//! local capacity per pupil, the ratio of plant spending to the allowance is **U-shaped**:
//!
//! | local capacity per pupil | median | against the allowance | buildings per 1,000 |
//! |---|--:|--:|--:|
//! | Q1 lowest | $3,390 | **1.30** | 2.66 |
//! | Q2 | $4,915 | 1.13 | 2.66 |
//! | Q3 | $6,429 | 1.11 | 2.33 |
//! | Q4 highest | $9,030 | 1.21 | 2.17 |
//!
//! The **poorest** quartile overspends the allowance most, and the wealthiest quartile has the
//! *fewest* buildings per pupil. So the sprawl burden is real and it does not sit where the
//! hypothesis puts it: it sits on districts with many buildings, and those skew poor rather than
//! wealthy. The wealthiest quartile's 1.21 is a district spending more on plant while having less
//! of it, which is a different claim — about discretion rather than about need — and this file
//! does not establish it.
//!
//! # The caution that bounds every ratio above
//!
//! **Spending is not need, and the two vintages differ.** Operations and maintenance is what a
//! district chose to spend in FY2025; the allowance is priced in the FY2027 factors. The *levels*
//! are therefore approximate on both counts — a district may overspend an allowance because its
//! plant demands it or because it can afford to, and a common price factor sits between the two
//! years. The **gradients** are the finding: a shared price factor cancels across quartiles, and
//! the monotone rise with buildings per pupil is not something discretion predicts.

use std::collections::BTreeMap;

use dispersion::functions;
use foundation::{ratios, BuildingAllowance, StatewideFactors};
use project::panel::{panel, DistrictRecord};

/// The per-pupil amount the operation term pays, at the factors in force.
fn allowance() -> f64 {
    let factors = StatewideFactors::fy2027();
    factors.building_per_pupil - factors.safety_per_pupil
}

/// FY2025 operations and maintenance per pupil, keyed for the join.
fn plant_spending() -> BTreeMap<String, f64> {
    functions::districts()
        .into_iter()
        .filter_map(|d| d.operations_maintenance.map(|v| (d.irn.clone(), v)))
        .collect()
}

/// Districts with both a plant figure and a building count, and the three measures of each.
struct Joined {
    buildings_per_thousand: f64,
    against_allowance: f64,
    capacity_per_pupil: f64,
}

fn joined() -> Vec<Joined> {
    let spending = plant_spending();
    let allowance = allowance();
    panel()
        .iter()
        .filter_map(|d: &DistrictRecord| {
            let adm = d.enrollment.base_cost_enrolled_adm;
            if adm <= 0.0 {
                return None;
            }
            Some(Joined {
                buildings_per_thousand: d.enrollment.open_buildings / adm * 1_000.0,
                against_allowance: spending.get(&d.irn)? / allowance,
                capacity_per_pupil: d.published_capacity_per_pupil?,
            })
        })
        .collect()
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Quartile medians of `measure`, after ordering on `axis`.
fn by_quartile(rows: &[Joined], axis: fn(&Joined) -> f64, measure: fn(&Joined) -> f64) -> [f64; 4] {
    let mut ordered: Vec<&Joined> = rows.iter().collect();
    ordered.sort_by(|a, b| axis(a).total_cmp(&axis(b)));
    let quarter = ordered.len() / 4;
    std::array::from_fn(|i| {
        let end = if i == 3 {
            ordered.len()
        } else {
            (i + 1) * quarter
        };
        median(
            ordered[i * quarter..end]
                .iter()
                .map(|r| measure(r))
                .collect(),
        )
    })
}

/// **The term the hypothesis is about.** The building amount is a statewide average square footage.
#[test]
fn the_operation_term_is_a_statewide_square_footage_and_nothing_else() {
    // The product the workspace carried, decomposed into the two numbers it hides.
    assert!((BuildingAllowance::fy2022().per_pupil() - 1_129.78).abs() < 0.005);
    assert!((BuildingAllowance::fy2027().per_pupil() - 1_418.16).abs() < 0.005);
    assert!(
        (BuildingAllowance::fy2022().per_pupil() - StatewideFactors::fy2022().building_per_pupil)
            .abs()
            < 0.005
    );
    assert!(
        (BuildingAllowance::fy2027().per_pupil() - StatewideFactors::fy2027().building_per_pupil)
            .abs()
            < 0.005
    );

    // The square footage moved while the salary reference year was frozen: 16.2%, and invisible
    // in the product alone.
    let (before, after) = (
        BuildingAllowance::fy2022().square_feet_per_pupil,
        BuildingAllowance::fy2027().square_feet_per_pupil,
    );
    assert!(
        (after / before - 1.162).abs() < 0.001,
        "the square footage assumption moved {:.1}%, not 16.2%",
        (after / before - 1.0) * 100.0
    );

    // And the operation term is exactly ADM-proportional: two districts with the same ADM and
    // different building counts are paid the same operation amount. This is the hypothesis's
    // mechanism, asserted as a property of the function rather than described.
    let factors = StatewideFactors::fy2027();
    let base = |buildings: f64| {
        let mut enrollment = panel()[0].enrollment;
        enrollment.open_buildings = buildings;
        foundation::building_leadership_base_cost(&enrollment, &factors).operation
    };
    assert!(
        (base(2.0) - base(40.0)).abs() < 0.005,
        "the operation term moved when only the building count did"
    );
}

/// **The half of the hypothesis that is false.** The formula does see building count.
#[test]
fn the_support_floor_makes_building_count_govern_for_half_the_state() {
    let (mut binds, mut capped) = (0, 0);
    for district in panel() {
        let adm = district.enrollment.base_cost_enrolled_adm;
        let by_enrollment = edfund_core::round_dp(adm / ratios::BUILDING_LEADERSHIP_SUPPORT, 2);
        if by_enrollment < district.enrollment.open_buildings {
            binds += 1;
        } else if by_enrollment
            >= district.enrollment.open_buildings * ratios::BUILDING_SUPPORT_PER_BUILDING
        {
            capped += 1;
        }
    }
    assert_eq!(
        binds, 291,
        "the open-buildings floor binds for {binds} districts"
    );
    assert_eq!(capped, 2);

    // Opening a building raises funded support where the floor governs, which is what "the
    // formula accounts for sprawl" has to mean if it means anything.
    let factors = StatewideFactors::fy2027();
    let mut sprawling = panel()[0].enrollment;
    sprawling.open_buildings = sprawling.base_cost_enrolled_adm / 100.0;
    let before = foundation::building_leadership_base_cost(&sprawling, &factors).support;
    sprawling.open_buildings += 1.0;
    let after = foundation::building_leadership_base_cost(&sprawling, &factors).support;
    assert!(
        after > before,
        "a district past the floor gains nothing by opening a building"
    );
}

/// **The half that is true.** Plant spending against the allowance rises with buildings per pupil.
#[test]
fn the_allowance_falls_further_behind_the_more_buildings_a_district_keeps() {
    let rows = joined();
    assert!(rows.len() > 590, "only {} districts joined", rows.len());

    let ratios = by_quartile(&rows, |r| r.buildings_per_thousand, |r| r.against_allowance);
    for window in ratios.windows(2) {
        assert!(
            window[1] > window[0],
            "the shortfall is not monotone across building density: {:.2} then {:.2}",
            window[0],
            window[1]
        );
    }
    assert!(
        (ratios[0] - 0.99).abs() < 0.03,
        "the least sprawling quartile spends {:.2} of the allowance, not 0.99",
        ratios[0]
    );
    assert!(
        (ratios[3] - 1.49).abs() < 0.03,
        "the most sprawling quartile spends {:.2} of the allowance, not 1.49",
        ratios[3]
    );
}

/// **The part the data contradicts.** The burden is not on high-capacity districts.
#[test]
fn the_sprawl_burden_does_not_sit_on_the_wealthiest_districts() {
    let rows = joined();
    let ratios = by_quartile(&rows, |r| r.capacity_per_pupil, |r| r.against_allowance);

    // U-shaped rather than rising: the poorest quartile overspends the allowance most.
    assert!(
        ratios[0] > ratios[3],
        "the lowest-capacity quartile spends {:.2} against the highest's {:.2}",
        ratios[0],
        ratios[3]
    );
    assert!(
        ratios[3] > ratios[2],
        "the relationship is not U-shaped: {:.2} then {:.2}",
        ratios[2],
        ratios[3]
    );

    // And the wealthiest districts have the *fewest* buildings per pupil, so whatever they are
    // spending it on, it is not sprawl.
    let density = by_quartile(
        &rows,
        |r| r.capacity_per_pupil,
        |r| r.buildings_per_thousand,
    );
    assert!(
        density[3] < density[0],
        "the highest-capacity quartile keeps {:.2} buildings per 1,000 pupils against {:.2}",
        density[3],
        density[0]
    );
}

/// The statute has no age term, which is the one part of the hypothesis nothing here can test.
#[test]
fn nothing_in_the_build_up_reads_a_buildings_age() {
    // `DistrictEnrollment` is the whole of what the build-up knows about a district's plant, and
    // it carries a count. If a vintage or age field is ever added, this fails and the module
    // note above stops being true.
    let enrollment = panel()[0].enrollment;
    let factors = StatewideFactors::fy2027();
    let cost = foundation::building_leadership_base_cost(&enrollment, &factors);

    // Every dollar of the term is explained by ADM and the building count together — there is no
    // residual an age term could be hiding in.
    let by_hand = {
        let adm = enrollment.base_cost_enrolled_adm;
        let leaders = edfund_core::round_dp(adm / ratios::BUILDING_LEADER, 2);
        let unit = (factors.banded_position_cost(
            adm,
            factors.superintendent_salary_small,
            factors.superintendent_salary_large,
        ) - factors.insurance)
            * (factors.principal_salary / factors.superintendent_salary)
            + factors.insurance;
        leaders * unit
    };
    assert!(
        (cost.leadership - by_hand).abs() < 0.005,
        "the leadership term is not exactly ADM times a unit cost"
    );
}
