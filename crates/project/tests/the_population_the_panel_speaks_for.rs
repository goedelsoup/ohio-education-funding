//! Which districts the panel holds, which two it does not, and what that costs a total.
//!
//! # The question this answers
//!
//! Every statewide figure this workspace computes is a sum over [`panel()`], which is **609
//! districts**. The department pays **611**. Nothing asserted that gap, and the corpus quoted
//! both denominators — "294 of 609 districts" beside a $878,974,300 guarantee total that is the
//! department's 611-row sum, and "440 of 611" in a transportation node whose sibling figures
//! come from the 609.
//!
//! Neither number is wrong. What was missing is any statement of *which population a figure
//! describes*, in a repository whose method is that a figure can be walked back to what produced
//! it. These tests make the population an asserted fact rather than an assumption, so a claim
//! written against the wrong one has somewhere to fail.
//!
//! # The two, and why they are interesting
//!
//! **Middle Bass Local** and **North Bass Local**, Lake Erie island districts in Ottawa County.
//! The department computes no base cost for either — they are absent from `Base_Cost` and
//! `Base_Cost_adm` entirely — and the panel is keyed on that sheet, so they cannot enter it.
//!
//! North Bass is the one that matters. It receives **$19,672.64, all of it temporary transitional
//! aid guarantee**, against zero base cost, zero categoricals and zero transportation. It is the
//! guarantee's limit case: a district the formula computes *nothing* for, funded because it was
//! funded in FY2020. The panel cannot see it, so no figure in this workspace can state it, and
//! the guarantee total is short by exactly that amount.

use project::panel::panel;

/// The IRNs the department pays and this panel does not carry.
///
/// Hard-coded rather than derived, because deriving them needs the workbook and the whole point
/// is that the committed fixture cannot answer the question. If the extract ever gains them these
/// assertions fail, which is the correct outcome — the exclusion is a fact to revisit, not a
/// constant to preserve.
const EXCLUDED: [(&str, &str); 2] = [
    ("048959", "Middle Bass Local"),
    ("048967", "North Bass Local"),
];

/// What North Bass receives, all of it guarantee. `Summary_SFPR` `[I]` and `[R]` alike.
const NORTH_BASS_GUARANTEE: f64 = 19_672.64;

#[test]
fn the_panel_is_six_hundred_and_nine_costed_districts() {
    let panel = panel();
    assert_eq!(
        panel.len(),
        609,
        "the panel's population is load-bearing for every statewide total in this workspace"
    );

    // Every one of them has a base cost, which is what being in the panel means.
    for record in &panel {
        assert!(
            record.aggregate_base_cost > 0.0,
            "{} is in the panel with no base cost",
            record.name
        );
    }
}

#[test]
fn the_two_districts_the_department_pays_are_not_in_it() {
    let panel = panel();
    for (irn, name) in EXCLUDED {
        assert!(
            !panel.iter().any(|r| r.irn == irn),
            "{name} ({irn}) is in the panel; the exclusion note in \
             `connect::fixtures::fy27` and the catalog record both need revisiting"
        );
    }
}

/// The guarantee total is short by North Bass, and by exactly North Bass.
///
/// This is the assertion that would have caught the corpus pairing a 609-district count with the
/// department's 611-row sum. The difference is not rounding and not a mystery: it is one district
/// whose entire state funding is a hold-harmless.
#[test]
fn the_guarantee_total_differs_from_the_departments_by_one_island_district() {
    let panel = panel();
    let ours: f64 = panel.iter().map(|r| r.guarantee).sum();

    // The department's own `Summary_SFPR` figure, over the 611 districts that sheet holds.
    let departments = 878_974_300.0_f64;

    assert!(
        (ours - 878_954_627.38).abs() < 0.01,
        "panel guarantee total moved: {ours:.2}"
    );
    assert!(
        (departments - ours - NORTH_BASS_GUARANTEE).abs() < 1.0,
        "the gap to the department's total is {:.2}, which is no longer North Bass's {NORTH_BASS_GUARANTEE:.2}",
        departments - ours
    );
}

/// A count over the panel is a count over 609, whatever denominator the prose beside it uses.
///
/// `fsfp-transportation` says "440 of 611 districts sit on" the 50% transportation floor, which is
/// true of the department's population. The panel gives **438**, because the two island districts
/// are below the floor and are not here. Both figures are correct about their own population and
/// only one of them is reachable from this workspace; the node now says which.
#[test]
fn the_transportation_floor_count_is_the_panels_and_not_the_departments() {
    let panel = panel();
    let below = panel
        .iter()
        .filter(|r| r.state_share_fraction() < 0.5)
        .count();
    assert_eq!(
        below, 438,
        "the panel's count of districts under the transportation floor; the department's 611-row \
         count is 440 and the difference is Middle Bass and North Bass"
    );
}
