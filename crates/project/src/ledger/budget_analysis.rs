//! LSC's analysis of a budget bill, in both of the editions it is published in.
//!
//! The Legislative Service Commission analyses each operating budget twice. The **redbook**
//! covers the bill *as introduced*, so its amounts are the executive proposal; the **greenbook**
//! covers the same bill *as enacted*, so its amounts are the appropriation. The two documents are
//! structurally identical — same categories, same headings, same sentences, the same tables in the
//! same order — and differ in three words of column heading.
//!
//! That is what makes the redbook so easy to quote as though it were the law, and this corpus did
//! quote it that way: six claims across five programme nodes and two revenue-stream documents
//! cited the redbook, two of them carrying figures — $1.34 billion of an $11.15 billion foundation
//! aid total — that are the proposal and not the appropriation. [`Edition`] exists so a caller has
//! to name which document it is reading.
//!
//! # Why this is in the library
//!
//! Three integration tests parsed these two fixtures, each with its own row reader, and the three
//! did not agree about how to find a table. Two anchored with `find` and one with `rfind`, because
//! `rfind` is what it takes to get past the table of contents — and the file that used `find`
//! only worked because the heading it wanted has no contents entry. Nothing checked that any two
//! of them read the same row. There is one reader now, and the anchoring hazard is stated on
//! [`row`] rather than rediscovered. See #157 for the same move over ten other fixtures.
//!
//! Cited by `parameter/appropriation-proration-factor`, `revenue-stream/lottery-profits`,
//! `revenue-stream/casino-tax-distribution` and
//! `formula-component/fsfp-preschool-special-education`.

use edfund_core::Dollars;

/// LSC's analysis of the bill **as introduced**. Amounts are the executive proposal.
pub const REDBOOK: &str = include_str!("../../fixtures/dew-redbook.txt");

/// LSC's analysis of the same bill **as enacted**. Amounts are the appropriation.
pub const GREENBOOK: &str = include_str!("../../fixtures/dew-greenbook.txt");

/// Which of the two documents a figure comes from.
///
/// A figure without one of these attached is a figure that cannot be checked, because the two
/// documents print the same table under the same heading with different numbers in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    /// The redbook: the bill as introduced. A **proposal**.
    Introduced,
    /// The greenbook: the bill as enacted. An **appropriation**.
    Enacted,
}

impl Edition {
    /// The document.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::Introduced => REDBOOK,
            Self::Enacted => GREENBOOK,
        }
    }

    /// The word in the column heading that distinguishes the two, and the only thing on the page
    /// that does.
    #[must_use]
    pub const fn column_heading(self) -> &'static str {
        match self {
            Self::Introduced => "Introduced",
            Self::Enacted => "Appropriation",
        }
    }

    /// What the first column of a table is in this edition: an estimate before the year closed,
    /// an actual after it.
    ///
    /// The distinction is not cosmetic. FY2025's ALI 200540 line is a $198,850,000 **estimate** in
    /// the redbook and a $195,160,040 **actual** in the greenbook, and only the first is a budget
    /// figure comparable to an appropriation.
    #[must_use]
    pub const fn prior_year_heading(self) -> &'static str {
        match self {
            Self::Introduced => "Estimate",
            Self::Enacted => "Actual",
        }
    }
}

/// The foundation aid appropriation table: the five lines the formula is paid from.
pub const FOUNDATION_AID: &str = "Foundation Aid Appropriations";

/// The ALI whose remainder funds preschool special education.
pub const SPECIAL_EDUCATION_ENHANCEMENTS: &str = "Special Education Enhancements (ALI 200540)";

/// The bottom line of [`FOUNDATION_AID`].
pub const TOTAL_FOUNDATION_AID: &str = "Total foundation aid";

/// The lottery's line inside [`FOUNDATION_AID`] — the one itemisation that makes a substitution
/// readable rather than inferred.
pub const LOTTERY_LINE: &str = "Fund 7017 ALI 200612";

/// The residual earmark of [`SPECIAL_EDUCATION_ENHANCEMENTS`].
///
/// Spelled in full because six rows of the greenbook begin `Remainder`, one per line item with a
/// residual earmark, and the nearest of them to a careless anchor is foundation aid's — an
/// eight-billion figure where a hundred-and-fifty-million one belongs.
pub const PRESCHOOL_REMAINDER: &str = "Remainder – preschool special education";

/// The line total of [`SPECIAL_EDUCATION_ENHANCEMENTS`].
pub const ALI_200540_TOTAL: &str = "GRF ALI 200540 total";

/// The three columns every table in these documents carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    /// The year before the biennium — FY2025 in this edition of the documents. An **estimate** in
    /// the redbook and an **actual** in the greenbook; see [`Edition::prior_year_heading`].
    pub prior: Dollars,
    /// The first year of the biennium — FY2026.
    pub first: Dollars,
    /// The second year — FY2027.
    pub second: Dollars,
}

/// The dollar amounts on the first row under `section` whose label begins with `label`.
///
/// # How the section is anchored, and why it is not `find`
///
/// A section heading appears twice: once in the table of contents with dot leaders and a page
/// number, and once over the table itself. Anchoring on the first occurrence starts the row search
/// at the contents page and walks forward into whichever line item happens to come next — which is
/// how the first version of the proration test read foundation aid's eight-billion remainder as
/// preschool special education's. It failed only because the two years of an appropriation are
/// equal and those two were not.
///
/// So the anchor is the line that *ends* with the section name, which the contents entry never
/// does. There is exactly one such line per document, and this function insists on it: a
/// re-extraction that introduces a running header carrying the section name would otherwise pick a
/// silently different table.
///
/// # Panics
///
/// If the section heading is not found exactly once, if no row under it starts with `label`, or if
/// that row does not carry exactly three dollar amounts. Each of those is a document whose shape
/// has moved, and reading it anyway would produce figures that parse and are wrong.
#[must_use]
pub fn row(edition: Edition, section: &str, label: &str) -> Row {
    let text = edition.text();
    let mut headings = text
        .lines()
        .scan(0usize, |at, line| {
            let start = *at;
            *at += line.len() + 1;
            Some((start, line))
        })
        .filter(|(_, line)| line.trim_end().ends_with(section));

    let (at, _) = headings
        .next()
        .unwrap_or_else(|| panic!("no section heading {section:?}"));
    assert!(
        headings.next().is_none(),
        "{section:?} heads more than one table; the anchor no longer identifies one"
    );

    let amounts: Vec<Dollars> = text[at..]
        .lines()
        .skip(1)
        .find(|line| line.trim_start().starts_with(label))
        .unwrap_or_else(|| panic!("no row {label:?} under {section:?}"))
        .split_whitespace()
        .filter_map(|token| {
            token
                .strip_prefix('$')?
                .replace(',', "")
                .parse::<Dollars>()
                .ok()
        })
        .collect();

    assert_eq!(
        amounts.len(),
        3,
        "{label:?} under {section:?} carries {} amounts, not the three columns these tables have",
        amounts.len()
    );
    Row {
        prior: amounts[0],
        first: amounts[1],
        second: amounts[2],
    }
}

/// A row of the foundation aid table.
#[must_use]
pub fn foundation_aid(edition: Edition, label: &str) -> Row {
    row(edition, FOUNDATION_AID, label)
}

/// A row of the ALI 200540 earmark table.
#[must_use]
pub fn special_education_enhancements(edition: Edition, label: &str) -> Row {
    row(edition, SPECIAL_EDUCATION_ENHANCEMENTS, label)
}

/// What one row of the foundation aid table moved between the proposal and the act, in FY2026.
///
/// FY2026 rather than FY2027 because the greenbook's FY2025 column is an actual and the redbook's
/// is an estimate, so the first column is not a movement of the same quantity. Positive means the
/// legislature appropriated more than the executive proposed.
#[must_use]
pub fn enactment_movement(label: &str) -> Dollars {
    foundation_aid(Edition::Enacted, label).first - foundation_aid(Edition::Introduced, label).first
}

/// The same movement, for everything in the foundation aid table other than the lottery line.
///
/// This is the substitution argument as one subtraction. Between the proposal and the act the
/// lottery's contribution to foundation aid rose and the rest of the table fell, so the General
/// Assembly put in more lottery money and slightly less of everything else. It does not show
/// intent — the four GRF and non-lottery lines move for their own reasons — but it is the tightest
/// instance available of the thing the substitution critique describes.
#[must_use]
pub fn enactment_movement_off_the_lottery_line() -> Dollars {
    enactment_movement(TOTAL_FOUNDATION_AID) - enactment_movement(LOTTERY_LINE)
}

#[cfg(test)]
mod tests {
    use super::{
        enactment_movement, enactment_movement_off_the_lottery_line, foundation_aid, row, Edition,
        ALI_200540_TOTAL, FOUNDATION_AID, LOTTERY_LINE, PRESCHOOL_REMAINDER,
        SPECIAL_EDUCATION_ENHANCEMENTS, TOTAL_FOUNDATION_AID,
    };

    /// The hazard this reader exists to close, asserted rather than described.
    ///
    /// `Special Education Enhancements (ALI 200540)` appears twice in each document and only one
    /// of the two heads a table. If the contents entry ever stopped carrying dot leaders, both
    /// would end with the section name and [`row`] would panic instead of picking one.
    #[test]
    fn a_section_name_that_appears_twice_still_anchors_one_table() {
        for edition in [Edition::Introduced, Edition::Enacted] {
            let text = edition.text();
            assert_eq!(
                text.matches(SPECIAL_EDUCATION_ENHANCEMENTS).count(),
                2,
                "the contents entry or the heading has gone"
            );
            let remainder = row(edition, SPECIAL_EDUCATION_ENHANCEMENTS, PRESCHOOL_REMAINDER);
            // Not foundation aid's eight-billion remainder, which is what a contents anchor finds.
            assert!(remainder.first < 200_000_000.0, "{remainder:?}");
        }
    }

    /// The two documents are told apart by their column headings and by nothing else.
    #[test]
    fn each_edition_carries_the_heading_that_identifies_it() {
        for edition in [Edition::Introduced, Edition::Enacted] {
            assert!(
                edition.text().contains(edition.column_heading()),
                "{edition:?} lost the heading that says which document it is"
            );
            assert!(edition.text().contains(edition.prior_year_heading()));
        }
        assert!(
            !Edition::Enacted.text().contains("FY 2026        FY 2027\n                 Fund/ALI\n                                                  Estimate            Introduced"),
            "the greenbook now carries the redbook's column headings"
        );
    }

    /// The two years of an appropriation are stated separately, so an earmark that holds across
    /// them is visible as an equality rather than assumed.
    #[test]
    fn the_preschool_remainder_held_from_introduction_to_enactment() {
        let introduced = row(
            Edition::Introduced,
            SPECIAL_EDUCATION_ENHANCEMENTS,
            PRESCHOOL_REMAINDER,
        );
        let enacted = row(
            Edition::Enacted,
            SPECIAL_EDUCATION_ENHANCEMENTS,
            PRESCHOOL_REMAINDER,
        );
        assert_eq!(introduced.first, 153_976_832.0);
        assert_eq!(introduced.first, introduced.second);
        assert_eq!(enacted.first, introduced.first);
        assert_eq!(enacted.second, introduced.second);

        // And the prior-year column is the one thing that differs: an estimate against an actual,
        // a dollar apart.
        assert_eq!(introduced.prior, 147_500_000.0);
        assert_eq!(enacted.prior, 147_499_999.0);
    }

    /// The line total, from both editions, to the dollar.
    #[test]
    fn the_line_total_is_the_figure_the_catalog_reconciles_against() {
        let enacted = row(
            Edition::Enacted,
            SPECIAL_EDUCATION_ENHANCEMENTS,
            ALI_200540_TOTAL,
        );
        assert_eq!(enacted.first, 193_272_426.0);
        assert_eq!(enacted.second, 193_272_426.0);
        assert_eq!(enacted.prior, 195_160_040.0);
        assert_eq!(
            row(
                Edition::Introduced,
                SPECIAL_EDUCATION_ENHANCEMENTS,
                ALI_200540_TOTAL
            )
            .prior,
            198_850_000.0
        );
    }

    /// The substitution, as the two subtractions the corpus quotes.
    #[test]
    fn the_lottery_line_rose_by_more_than_the_total_did() {
        assert_eq!(enactment_movement(LOTTERY_LINE), 97_638_202.0);
        assert_eq!(enactment_movement(TOTAL_FOUNDATION_AID), 82_062_286.0);
        assert_eq!(enactment_movement_off_the_lottery_line(), -15_575_916.0);

        // Which is the same as saying the lottery's share of foundation aid is higher as enacted.
        let enacted = foundation_aid(Edition::Enacted, LOTTERY_LINE).first
            / foundation_aid(Edition::Enacted, TOTAL_FOUNDATION_AID).first;
        let introduced = foundation_aid(Edition::Introduced, LOTTERY_LINE).first
            / foundation_aid(Edition::Introduced, TOTAL_FOUNDATION_AID).first;
        assert!(enacted > introduced);
    }

    /// A row that does not carry three columns is a document whose shape has moved.
    #[test]
    #[should_panic(expected = "carries")]
    fn a_row_without_three_columns_is_refused() {
        let _ = row(Edition::Enacted, FOUNDATION_AID, "% change");
    }
}
