//! The share of a school building a district pays is its percentile, and the statute says so in
//! three sections.
//!
//! Issue #790. `program/classroom-facilities-assistance` recorded the share schedule as "in the
//! Commission's rules" and not held. It is in the Revised Code, and these tests pin the words the
//! corpus reads it from.
//!
//! # What these tests settle
//!
//! - **The rule is statutory.** R.C. 3318.011(D) divides the ranking into percentiles, 3318.01(K)
//!   multiplies the basic project cost by the percentile, and 3318.032(D) caps it at ninety-five.
//! - **H.B. 96 ended the net-indebtedness alternative, and left its definition.** 3318.032(A), at
//!   H.B. 96's effective date, makes the district's portion the required percentage alone. The
//!   "required level of indebtedness" of 3318.01(J) is still defined, and its own range agrees
//!   with the greenbook's 5.00% to 6.98%.
//! - **3318.032(E) no longer moves anything.** It takes the lesser of (A) and the greater of the
//!   required percentage and the previous project's share. With (A) equal to the required
//!   percentage, the lesser is (A) whatever the previous share was.

use dispersion::equity_rank::{percentile, percentile_by, required_percent, Rounding};
use project::statute;

fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_three_sections_are_the_versions_the_corpus_dates() {
    let rows = [
        (
            "3318.01",
            "January 1, 2026",
            "House Bill 96 - 136th General Assembly",
        ),
        (
            "3318.011",
            "October 3, 2023",
            "House Bill 33 - 135th General Assembly",
        ),
        (
            "3318.032",
            "September 30, 2025",
            "House Bill 96 - 136th General Assembly",
        ),
    ];
    for (number, effective, act) in rows {
        let section = statute::section(number);
        assert_eq!(section.effective, effective, "{number}");
        assert_eq!(section.legislation, act, "{number}");
    }
}

#[test]
fn the_share_is_one_per_cent_times_the_percentile() {
    let definitions = flat(statute::section("3318.01").body);
    assert!(definitions.contains(
        "(K) \"Required percentage of the basic project costs\" means one per cent of the basic \
         project costs times the percentile in which the school district ranks for the fiscal \
         year preceding the fiscal year in which the controlling board approved the district's \
         project."
    ));
    assert!(definitions.contains(
        "(Q) \"Percentile\" means the percentile in which the school district is ranked pursuant \
         to section 3318.011 of the Revised Code."
    ));
    let ranking = flat(statute::section("3318.011").body);
    assert!(ranking.contains(
        "(D) Divide such ranking into percentiles with the first percentile containing the one \
         per cent of school districts having the lowest three-year average adjusted valuations \
         per pupil and the one-hundredth percentile containing the one per cent of school \
         districts having the highest"
    ));
}

#[test]
fn the_portion_is_the_required_percentage_capped_at_ninety_five() {
    let portion = flat(statute::section("3318.032").body);
    assert!(portion.contains(
        "(A) Except as otherwise provided in divisions (B), (D), and (E) of this section, the \
         portion of the basic project cost supplied by the school district shall be the required \
         percentage of the basic project costs."
    ));
    assert!(portion.contains(
        "(D) At no time shall a school district's portion of the basic project cost be greater \
         than ninety-five per cent of the total basic project cost."
    ));
    assert!(!portion.contains("greater of the required percentage"));
    assert!(!portion.contains("required level of indebtedness"));
}

/// 3318.01(J): five per cent of valuation in the first percentile, plus two one-hundredths of a
/// per cent for each percentile above it. The first and one-hundredth percentiles give LSC's
/// 5.00% and 6.98%, and a zeroth percentile would give 4.98%, below the first.
#[test]
fn the_indebtedness_definition_survives_and_spans_the_greenbooks_range() {
    let definitions = flat(statute::section("3318.01").body);
    assert!(definitions.contains(
        "(1) In the case of school districts in the first percentile, five per cent of the \
         district's valuation"
    ));
    assert!(definitions.contains(
        "plus [two one-hundredths of one per cent multiplied by (the percentile in which the \
         district ranks for the fiscal year preceding the fiscal year in which the controlling \
         board approved the district's project minus one)]"
    ));
    let basis_points = |p: u32| 500 + 2 * (p - 1);
    assert_eq!(basis_points(percentile(1, 609)), 500);
    assert_eq!(basis_points(percentile(609, 609)), 698);
    assert_eq!(percentile_by(1, 609, Rounding::Floor), 0);
}

/// 3318.032(E) as H.B. 96 left it: the lesser of (A) and the greater of the required percentage
/// and the previous share. (A) is now the required percentage, so the lesser is always (A).
#[test]
fn the_prior_project_rule_reduces_to_division_a() {
    let portion = flat(statute::section("3318.032").body);
    assert!(portion.contains(
        "shall be the lesser of the following: (1) The portion calculated under division (A) of \
         this section; (2) The greater of the following: (a) The required percentage of the basic \
         project costs for the new project; (b) The percentage of the basic project cost paid by \
         the district for the previous project."
    ));
    for rank in [1, 271, 307, 504, 609] {
        let a = required_percent(rank, 609);
        for previous in [0, 5, 50, 95] {
            assert_eq!(
                a.min(a.max(previous)),
                a,
                "rank {rank}, previous {previous}"
            );
        }
    }
}
