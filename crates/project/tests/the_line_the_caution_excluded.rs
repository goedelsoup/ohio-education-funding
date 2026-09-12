//! ALI 200604, and a caution applied in the wrong direction.
//!
//! `appropriations::FOUNDATION_LINES` sums the lines the formula is paid from, and its doc comment
//! carries the reason it is a constant with a note rather than a filter written inline:
//!
//! > An appropriation line item is **not** a stable identifier across this period — `200604` names
//! > three different programmes across three funds — so any series built by line number needs its
//! > succession established before it means anything.
//!
//! That is correct, and `200604` is the right example. But the succession was established for
//! `200501` — carried at exactly $0.00 through its overlap with `200550`, checked rather than
//! assumed — and never established for `200604`, whose *third* programme is the one titled
//! `Foundation Funding - All Students`. The caution was used to exclude the line instead of to
//! resolve it, and the excluded amount is $500m to $600m a year.
//!
//! # The three programmes, and why only one of them counts
//!
//! Adult Basic Education in fund 366 through FY2011; the Student Wellness and Success funds H.B.
//! 166 created in fund 5VS0 for FY2020-21, paid *alongside* a frozen foundation amount rather than
//! through it; and foundation funding from FY2022, when the Fair School Funding Plan pulled that
//! channel inside the formula. The first two are not foundation aid on anybody's definition. The
//! third is, and was missing.
//!
//! # Why the rule is the title and not a year
//!
//! The title is the evidence the succession is established *from*; a year is the same fact with
//! the evidence discarded, and would go quietly wrong the next time a line is retitled. It also
//! gets the boundary right for free, and the boundary is one year off from where a reader would
//! put it: H.B. 110 retitles the line, so the FY2021 *actual* it restates reads `Foundation
//! Funding - All Students` while the FY2021 *enacted* row — H.B. 166's, the one this series is
//! built from — still reads `Student Wellness and Success`.
//!
//! # What the two bases are for
//!
//! A level for FY2022 that omits the channel understates what the General Assembly appropriated.
//! A *movement* across FY2021-FY2022 that includes it books the channel's arrival as a
//! half-billion-dollar move in foundation aid. Both are avoidable, and not by one series: see
//! `Basis`, which follows `dispersion::survey_basis::Basis` in giving the comparable quantity its
//! own basis rather than dropping the year the definition changes.

use edfund_core::FiscalYear;
use project::appropriations::{
    self, Basis, FOLDED_CHANNEL, FOLDED_IN_FROM, FOUNDATION_FUNDING_TITLE, FOUNDATION_LINES,
};

/// The base year every real figure below is stated in.
const BASE: FiscalYear = FiscalYear(2025);

/// Nominal foundation funding by year on a basis, from the enacted series.
fn nominal(basis: Basis) -> Vec<(u16, f64)> {
    appropriations::foundation_history(basis, BASE)
        .into_iter()
        .map(|year| (year.fiscal_year, year.nominal))
        .collect()
}

/// The line really does name three programmes, which is the premise of the whole exclusion.
#[test]
fn the_folded_channel_names_three_programmes_across_three_funds() {
    let mut titles: Vec<String> = appropriations::enacted_lines()
        .into_iter()
        .filter(|line| line.line_item == FOLDED_CHANNEL)
        .map(|line| line.title)
        .collect();
    titles.sort();
    titles.dedup();
    assert_eq!(
        titles,
        vec![
            "Adult Basic Education".to_string(),
            FOUNDATION_FUNDING_TITLE.to_string(),
            "Student Wellness and Success".to_string(),
        ],
        "the succession this module resolves has changed shape"
    );
}

/// And the title rule selects exactly the third of them, from exactly FY2022.
///
/// The years matter as much as the count. FY2002-FY2011 are Adult Basic Education and FY2020-21
/// are Student Wellness; a rule that caught either would be booking two decades of adult education
/// as foundation aid, which is the failure the caution was written against.
#[test]
fn the_title_rule_selects_foundation_funding_and_starts_at_the_seam() {
    let covered: Vec<u16> = appropriations::enacted_lines()
        .into_iter()
        .filter(|line| line.line_item == FOLDED_CHANNEL && Basis::AsEnacted.covers(line))
        .map(|line| line.fiscal_year)
        .collect();
    assert_eq!(covered, vec![2022, 2023, 2024, 2025, 2026, 2027]);
    assert_eq!(covered[0], FOLDED_IN_FROM);

    // The FY2021 enacted row is the one that makes a year-based rule wrong, so it is asserted
    // rather than left to the list above: it exists, it is the wellness programme, and it is out.
    let fy2021: Vec<String> = appropriations::enacted_lines()
        .into_iter()
        .filter(|line| line.line_item == FOLDED_CHANNEL && line.fiscal_year == 2021)
        .map(|line| line.title)
        .collect();
    assert_eq!(fy2021, vec!["Student Wellness and Success".to_string()]);
}

/// The two bases agree everywhere before the seam and nowhere after it.
///
/// This is the property that makes `FormulaLines` usable for every comparison in the workspace:
/// it is not a different series, it is the same series with the widening removed.
#[test]
fn the_bases_diverge_only_from_the_seam() {
    let formula = nominal(Basis::FormulaLines);
    let enacted = nominal(Basis::AsEnacted);
    assert_eq!(formula.len(), enacted.len());

    for ((year, lines_only), (same_year, as_enacted)) in formula.iter().zip(&enacted) {
        assert_eq!(year, same_year);
        if *year < FOLDED_IN_FROM {
            assert!(
                (lines_only - as_enacted).abs() < 0.005,
                "FY{year} differs by {} and nothing should differ before FY{FOLDED_IN_FROM}",
                as_enacted - lines_only
            );
        } else {
            assert!(
                as_enacted - lines_only >= 500_000_000.0,
                "FY{year} adds only {}",
                as_enacted - lines_only
            );
        }
    }
}

/// What the exclusion was worth, year by year, as a share of what was published.
///
/// The share is the point rather than the dollars: this series is rendered on the site as
/// `foundation_funding / enacted`, so an omission in the numerator is a percentage a reader is
/// invited to compare against LSC's own.
#[test]
fn the_exclusion_was_between_a_seventeenth_and_a_fourteenth_of_the_published_series() {
    let formula = nominal(Basis::FormulaLines);
    let enacted = nominal(Basis::AsEnacted);
    let shares: Vec<f64> = formula
        .iter()
        .zip(&enacted)
        .filter(|((year, _), _)| *year >= FOLDED_IN_FROM)
        .map(|((_, lines_only), (_, as_enacted))| (as_enacted - lines_only) / lines_only)
        .collect();

    assert_eq!(shares.len(), 6, "FY2022 through FY2027");
    let smallest = shares.iter().copied().fold(f64::INFINITY, f64::min);
    let largest = shares.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (0.058..0.060).contains(&smallest),
        "smallest omitted share is {smallest}"
    );
    assert!(
        (0.072..0.073).contains(&largest),
        "largest omitted share is {largest}"
    );
}

/// The seam is a widening of the definition, not a jump in the money — and the numbers say which.
///
/// FY2022's move on `AsEnacted` is $677.7m nominal, of which $500.0m is the channel arriving. On
/// `FormulaLines` it is the $177.7m that actually moved. A median of movements computed over the
/// first would have one year in it that is mostly a definition change, which is why
/// `foundation_movements` is not offered a basis.
#[test]
fn the_seam_is_mostly_the_channel_arriving_rather_than_money_moving() {
    let at = |series: &[(u16, f64)], year: u16| {
        series
            .iter()
            .find(|(y, _)| *y == year)
            .expect("the year is in the series")
            .1
    };
    let formula = nominal(Basis::FormulaLines);
    let enacted = nominal(Basis::AsEnacted);

    let widened = at(&enacted, FOLDED_IN_FROM) - at(&enacted, FOLDED_IN_FROM - 1);
    let moved = at(&formula, FOLDED_IN_FROM) - at(&formula, FOLDED_IN_FROM - 1);
    assert!(
        (widened - 677_679_867.0).abs() < 1.0,
        "widened by {widened}"
    );
    assert!((moved - 177_679_867.0).abs() < 1.0, "moved by {moved}");
    assert!(
        (widened - moved - 500_000_000.0).abs() < 1.0,
        "the difference between the two bases at the seam is the line itself"
    );
}

/// And the constant the caution is attached to still holds the property the caution checked.
///
/// `200501` overlaps `200550` and is $0.00 throughout the overlap. Asserted here because this file
/// is about the one succession that was *not* established, and the contrast is the argument: the
/// same check, run on the other line, is what the exclusion should have been.
#[test]
fn the_predecessor_line_is_still_zero_across_its_whole_overlap() {
    let overlaps: Vec<(u16, f64)> = appropriations::enacted_lines()
        .into_iter()
        .filter(|line| line.line_item == FOUNDATION_LINES[0])
        .filter(|line| line.fiscal_year >= 2006)
        .map(|line| (line.fiscal_year, line.amount))
        .collect();
    assert!(!overlaps.is_empty(), "the overlap years are in the series");
    for (year, amount) in overlaps {
        assert_eq!(amount, 0.0, "FY{year} carries {amount} on the retired line");
    }
}
