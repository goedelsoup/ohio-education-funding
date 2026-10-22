//! A district's share of a school building, read from its rank — and the rounding the statute
//! never names.
//!
//! # The question
//!
//! `program/classroom-facilities-assistance` recorded the share schedule as "in the Commission's
//! rules" and not held. It is in statute: R.C. 3318.011(D) divides the ranking into percentiles,
//! 3318.01(K) makes the percentile the share, and 3318.032(D) caps it at ninety-five. What the
//! three do not say is how a rank among 609 districts becomes a whole percentile, and the answer
//! moves a district's bill by a point of its project cost.
//!
//! # What settles it
//!
//! The section's own two anchors. The first percentile has to contain the lowest-ranked districts
//! and the one-hundredth the highest; floor and nearest both leave the lowest ranks in a zeroth
//! percentile the section does not have. The ceiling is the only one of the three that does not.
//! A project the Commission approved in July 2025 then discriminates among the readings where
//! the anchors cannot: New Philadelphia City, rank 271 of 609 on the FY2025 list, pays exactly
//! 45.00% — the ceiling, where floor and nearest both say 44.

use dispersion::equity_rank::{
    at_cap, districts, fy2026_rank, greenbook_flat, list, lowered_by_cap, percentile,
    percentile_by, project_support_fy2026, rank_of, required_percent, status_total, worked_example,
    Rounding, CAP_PERCENT,
};

/// Only the ceiling puts every rank into a percentile from 1 to 100 on both held lists.
#[test]
fn only_the_ceiling_has_no_zeroth_percentile() {
    for year in [2024, 2025] {
        let n = districts(year);
        assert_eq!(n, 609, "FY{year}");
        let lowest = |rounding| percentile_by(1, n, rounding);
        assert_eq!(lowest(Rounding::Ceiling), 1);
        assert_eq!(lowest(Rounding::Floor), 0);
        assert_eq!(lowest(Rounding::Nearest), 0);
        assert_eq!(percentile(n, n), 100);
        // And no percentile holds more than one per cent of the districts, rounded up.
        let mut counts = [0u32; 101];
        for row in list(year) {
            counts[percentile(row.rank, n) as usize] += 1;
        }
        assert_eq!(counts[0], 0);
        assert!(
            counts[1..].iter().all(|&c| (6..=7).contains(&c)),
            "{counts:?}"
        );
    }
}

/// How many districts each zeroth percentile would hold, which is the size of the defect.
#[test]
fn floor_strands_six_districts_and_nearest_three() {
    let n = districts(2025);
    let stranded = |rounding| {
        list(2025)
            .iter()
            .filter(|row| percentile_by(row.rank, n, rounding) == 0)
            .count()
    };
    assert_eq!(stranded(Rounding::Floor), 6);
    assert_eq!(stranded(Rounding::Nearest), 3);
    assert_eq!(stranded(Rounding::Ceiling), 0);
}

/// The Commission's own example: rank 307 "would have a state/local split of 50/50". On 609
/// districts that is 50.4%; the ceiling says 51, so the example states the rule's shape and not
/// its rounding.
#[test]
fn the_commissions_example_is_a_point_short_of_the_rule() {
    let (rank, local) = worked_example();
    assert_eq!((rank, local), (307, 50));
    assert_eq!(status_total(), 609);
    assert_eq!(percentile(rank, 609), 51);
    assert_eq!(percentile_by(rank, 609, Rounding::Floor), local);
}

/// The published pair the ceiling reproduces and the other two readings do not.
///
/// The amounts are the Commission's for New Philadelphia City's project as its 10 July 2025
/// minutes state them, transcribed here: the minutes are archived nowhere this repository can
/// fetch from, so they are constants in a test and not a fixture, and the corpus binds only the
/// half of the pair that comes from the list.
#[test]
fn new_philadelphia_pays_the_ceiling() {
    const LOCAL: u64 = 65_468_819;
    const TOTAL: u64 = 145_486_265;
    // Exactly 45.00%, to the dollar the Commission rounds to.
    assert_eq!((LOCAL * 10_000).div_ceil(TOTAL), 4_500);
    let rank = rank_of(2025, "New Philadelphia City SD", "Tuscarawas").unwrap();
    assert_eq!(rank, 271);
    assert_eq!(percentile(rank, 609), 45);
    assert_eq!(percentile_by(rank, 609, Rounding::Floor), 44);
    assert_eq!(percentile_by(rank, 609, Rounding::Nearest), 44);
}

/// Two more pairs from the same unarchived minutes, read against the list for the fiscal year
/// before each meeting: Madison-Plains Local at 83% in July 2025 and its first segment at 85% in
/// September 2024. Neither discriminates between readings, but both agree with the ceiling.
#[test]
fn madison_plains_pays_its_percentile_on_both_lists() {
    let fy2025 = rank_of(2025, "Madison-Plains Local SD", "Madison").unwrap();
    let fy2024 = rank_of(2024, "Madison-Plains Local SD", "Madison").unwrap();
    assert_eq!((fy2025, fy2024), (504, 516));
    assert_eq!(required_percent(fy2025, 609), 83);
    assert_eq!(required_percent(fy2024, 609), 85);
}

/// The cap: 37 districts pay ninety-five per cent, and for 31 of them it is lower than their
/// percentile.
#[test]
fn the_cap_lowers_thirty_one_districts() {
    for year in [2024, 2025] {
        assert_eq!(at_cap(year), 37, "FY{year}");
        assert_eq!(lowered_by_cap(year), 31, "FY{year}");
    }
    assert_eq!(CAP_PERCENT, 95);
    assert_eq!(required_percent(573, 609), 95);
    assert_eq!(required_percent(572, 609), 94);
}

/// The FY2026 ranks survive only in the status edition, beside each district's programme status.
#[test]
fn the_status_edition_holds_the_unarchived_fy2026_ranks() {
    assert_eq!(
        fy2026_rank("New Philadelphia City SD", "Tuscarawas"),
        Some(269)
    );
    assert_eq!(fy2026_rank("Warren City SD", "Trumbull"), Some(1));
    assert_eq!(fy2026_rank("Kelleys Island Local SD", "Erie"), Some(609));
}

/// H.B. 96's two changes to the channel, as LSC's enacted greenbook states them.
#[test]
fn the_greenbook_states_both_changes() {
    let flat = greenbook_flat();
    assert!(flat.contains(
        "The budget eliminates the latter method of determining a school district’s local share"
    ));
    assert!(flat.contains("the district’s required level of indebtedness"));
    assert_eq!(project_support_fy2026(), 20_000_000);
    for condition in [
        "a county with a population of more than 100,000 people",
        "another county with a population of less than 50,000 people",
        "originally constructed prior to June 30, 1925",
        "the state share must be at least 90% of the cost of the project",
        "apply for funding by December 31, 2025",
    ] {
        assert!(flat.contains(condition), "{condition}");
    }
}
