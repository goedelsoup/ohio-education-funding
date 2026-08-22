//! What the General Assembly appropriated to the schools before FY2002, read off the acts.
//!
//! # Why this is a separate module from [`crate::appropriations`]
//!
//! Different publisher and a different kind of document. Everything in the main series is the
//! Legislative Service Commission describing what an act did; this is the act. They are kept apart
//! so that a figure's provenance is a property of where it is read from rather than of a column,
//! and so that the join between them is written once and can be tested.
//!
//! # Four fiscal years, and the wall behind them
//!
//! FY1998 through FY2001. The main series begins at FY2002, so this extends it by four years and
//! then stops — not because nobody has looked further, but because the legislature's own version
//! index has seventeen entries and the oldest is the 122nd General Assembly. *DeRolph I* was
//! decided in March 1997 and falls inside it, so the first budget enacted after the decision is
//! reachable. The Foundation Program era and the equal yield formula are not.
//!
//! # A year can be appropriated twice, and the two readings are not alike
//!
//! Nine months after the Supreme Court held the funding system unconstitutional, Am. Sub. H.B. 215
//! appropriated FY1998 across fifty-three GRF education lines and FY1999 across **one**: `200405`,
//! Primary and Secondary Education Funding, $4,470,135,592, with fifty-one of the others at zero.
//! The act says why:
//!
//! > By January 15, 1998, the General Assembly shall develop a plan to provide itemized
//! > appropriations for the Department of Education for fiscal year 1999.
//!
//! The promise was kept. H.B. 650 itemised the year in February 1998 and H.B. 770 reprinted the
//! result in June with corrections of its own. So FY1999 has two enacted readings — one from the
//! act that opened the biennium and one from the act that governed it — about $175 million and
//! fifty line items apart.
//!
//! [`OPERATIVE`] names which act answers for a year and everything computed here reads through it.
//! The superseded reading is kept rather than discarded, because the *shape* of what H.B. 215 did
//! is the finding: [`deferred_years`] is the set of years an act appropriated without itemising,
//! and it is not empty.
//!
//! # The corroboration needed no new retrieval
//!
//! The H.B. 94 greenbook's FY1999 *actuals* in `appropriation-lines.csv` carry 141 line items and
//! **no `200405` at all**, with `200501` spending $3,035,363,396 against the $2,986,915,811
//! H.B. 770 appropriates it. A line appropriated $4.47 billion and spent nothing, because by the
//! time the money moved it had been given its proper names.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/session-law-lines.csv");

const EXPECTED_HEADER: &str =
    "general_assembly,bill,fiscal_year,fund_group,fund,line_item,title,amount";

/// The columns of [`EXPECTED_HEADER`], named where they are read.
mod column {
    pub const GENERAL_ASSEMBLY: usize = 0;
    pub const BILL: usize = 1;
    pub const FISCAL_YEAR: usize = 2;
    pub const FUND_GROUP: usize = 3;
    pub const FUND: usize = 4;
    pub const LINE_ITEM: usize = 5;
    pub const TITLE: usize = 6;
    pub const AMOUNT: usize = 7;
}

/// The first fiscal year any act in this repository appropriates.
pub const FIRST_YEAR: u16 = 1998;

/// The last, after which [`crate::appropriations`] answers.
pub const LAST_YEAR: u16 = 2001;

/// The line item that stands in for a whole year's itemisation.
pub const LUMP_SUM_ITEM: &str = "200405";

/// Which act's text answers for which fiscal year.
///
/// H.B. 770 for the first biennium and not H.B. 215, because H.B. 770 reprints Section 50 as
/// amended by H.B. 650 and then amends it again — so its columns are the ones that governed. The
/// difference is not cosmetic: FY1999 net of tax reimbursement reads $6.15bn as H.B. 215 enacted
/// it and $6.33bn as H.B. 770 left it, and the composition differs by fifty-odd line items.
pub const OPERATIVE: [(u16, &str); 4] = [
    (1998, "hb770"),
    (1999, "hb770"),
    (2000, "hb282"),
    (2001, "hb282"),
];

/// The act whose text governs a fiscal year, if this repository holds one.
#[must_use]
pub fn operative(fiscal_year: u16) -> Option<&'static str> {
    OPERATIVE
        .iter()
        .find(|(year, _)| *year == fiscal_year)
        .map(|(_, bill)| *bill)
}

/// Only the rows from the act that governs each year.
#[must_use]
pub fn operative_lines() -> Vec<Appropriation> {
    lines()
        .into_iter()
        .filter(|line| operative(line.fiscal_year) == Some(line.bill.as_str()))
        .collect()
}

/// The formula's own lines, keyed on fund as well as number.
///
/// **The number alone is not a key in this era.** `200-610` is `454 Guidance and Testing` and
/// `017 Base Cost Funding` in the same H.B. 770 table, and the greenbook resolves the collision by
/// renumbering the lottery line to `200612` from FY2002. So the formula's share is taken on the
/// pair, and a reader filtering on `200610` alone would add a testing programme to base cost.
///
/// The lottery half of the formula is renumbered twice inside four years — `200670 School
/// Foundation - Basic Allowance` in FY1998, `200610 Base Cost Funding` in FY1999, `200612` from
/// FY2000 — for the same money in the same fund. The FY1999 identity is confirmed from outside the
/// acts: the greenbook's FY1999 *actual* for `200612` is $666,093,028, which is what H.B. 770
/// appropriates `017 200610`, to the dollar.
const FOUNDATION: [(&str, &str); 4] = [
    ("GRF", "200501"),
    ("017", "200670"),
    ("017", "200610"),
    ("017", "200612"),
];

/// What the formula itself was appropriated, by year, under the operative act.
#[must_use]
pub fn foundation_funding() -> BTreeMap<u16, f64> {
    let mut out: BTreeMap<u16, f64> = BTreeMap::new();
    for line in operative_lines() {
        if FOUNDATION
            .iter()
            .any(|(fund, item)| *fund == line.fund && *item == line.line_item)
        {
            *out.entry(line.fiscal_year).or_default() += line.amount;
        }
    }
    out
}

/// How many line items the operative act funds in a year.
#[must_use]
pub fn items_by_year() -> BTreeMap<u16, usize> {
    let mut out: BTreeMap<u16, usize> = BTreeMap::new();
    for line in operative_lines() {
        if line.amount > 0.0 {
            *out.entry(line.fiscal_year).or_default() += 1;
        }
    }
    out
}

/// One appropriation line as one act states it, for one fiscal year.
#[derive(Debug, Clone, PartialEq)]
pub struct Appropriation {
    /// The General Assembly that passed the act.
    pub general_assembly: u16,
    /// The act, as the registry keys it.
    pub bill: String,
    /// The fiscal year the amount is for.
    pub fiscal_year: u16,
    /// The fund group heading it sits under.
    pub fund_group: String,
    /// The three-character fund code.
    pub fund: String,
    /// The six-digit line item, with the act's hyphen removed.
    pub line_item: String,
    /// Its title as the act gives it.
    pub title: String,
    /// The amount, in the dollars of its own year.
    pub amount: f64,
}

/// Every line in the committed extract.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against.
#[must_use]
pub fn lines() -> Vec<Appropriation> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            Some(Appropriation {
                general_assembly: row.str(column::GENERAL_ASSEMBLY).parse().ok()?,
                bill: row.str(column::BILL).to_string(),
                fiscal_year: row.str(column::FISCAL_YEAR).parse().ok()?,
                fund_group: row.str(column::FUND_GROUP).to_string(),
                fund: row.str(column::FUND).to_string(),
                line_item: row.str(column::LINE_ITEM).to_string(),
                title: row.str(column::TITLE).to_string(),
                amount: row.num(column::AMOUNT)?,
            })
        })
        .collect()
}

/// The appropriation to the department in one year, net of the tax reimbursement lines.
///
/// Net on the same rule [`crate::appropriations`] uses, so the two series are comparable at the
/// seam. The rule is keyed on the year as well as the number — see
/// [`crate::appropriations::is_tax_reimbursement`], which this era is the reason for.
#[must_use]
pub fn department_total() -> BTreeMap<u16, f64> {
    let mut out: BTreeMap<u16, f64> = BTreeMap::new();
    for line in operative_lines() {
        if crate::appropriations::is_tax_reimbursement(&line.line_item, line.fiscal_year) {
            continue;
        }
        *out.entry(line.fiscal_year).or_default() += line.amount;
    }
    out
}

/// Years an act appropriated without itemising, by the act that opened their biennium.
///
/// FY1999, and only FY1999 — under H.B. 215, which is superseded. Kept because the shape is the
/// finding, and because a later act doing the same thing should become visible rather than be
/// smoothed away by the vintage rule. Read through [`OPERATIVE`] this set moves no total.
#[must_use]
pub fn deferred_years() -> BTreeSet<u16> {
    lines()
        .into_iter()
        .filter(|line| line.line_item == LUMP_SUM_ITEM && line.amount > 0.0)
        .map(|line| line.fiscal_year)
        .collect()
}

/// One line's amount across every year, under the operative act for each.
#[must_use]
pub fn line_history(line_item: &str) -> BTreeMap<u16, f64> {
    operative_lines()
        .into_iter()
        .filter(|line| line.line_item == line_item)
        .map(|line| (line.fiscal_year, line.amount))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_acts_cover_four_fiscal_years_and_stop() {
        let years: Vec<u16> = department_total().keys().copied().collect();
        assert_eq!(years, (FIRST_YEAR..=LAST_YEAR).collect::<Vec<u16>>());
    }

    /// The join. These four years sit below FY2002 with no seam at the General Assembly boundary.
    #[test]
    fn the_series_joins_the_one_that_begins_at_fy2002() {
        let acts = department_total();
        let main = crate::appropriations::enacted_history(edfund_core::FiscalYear(2025));
        let fy2002 = main
            .iter()
            .find(|y| y.fiscal_year == 2002)
            .expect("the main series begins at FY2002");

        // Rising in every step, and rising into FY2002 rather than stepping at it. A break at the
        // seam would mean the two publishers are counting different things.
        let mut prior = 0.0;
        for year in FIRST_YEAR..=LAST_YEAR {
            let total = acts[&year];
            assert!(
                total > prior,
                "FY{year} is {total:.0} against FY{}'s {prior:.0}",
                year - 1
            );
            prior = total;
        }
        assert!(
            fy2002.nominal > prior,
            "FY2002 is {:.0} against FY2001's {prior:.0}",
            fy2002.nominal
        );
        // And the step into FY2002 is no larger than the steps inside the acts' own span, so
        // nothing at the seam looks like a change of definition.
        let inside = acts[&LAST_YEAR] - acts[&(LAST_YEAR - 1)];
        assert!(
            fy2002.nominal - prior < inside * 2.0,
            "FY2001 to FY2002 moves {:.0} against {inside:.0} inside the acts",
            fy2002.nominal - prior
        );
    }

    /// The finding, pinned: one act appropriated a year without itemising it, and said why.
    #[test]
    fn fy1999_was_appropriated_as_one_line_and_then_itemised() {
        // As H.B. 215 enacted it. `200405` carries the year and the formula's own line carries
        // nothing, which is the shape the act's prose explains.
        let as_passed: Vec<Appropriation> = lines()
            .into_iter()
            .filter(|l| l.bill == "hb215" && l.fiscal_year == 1999)
            .collect();
        let lump = as_passed
            .iter()
            .find(|l| l.line_item == LUMP_SUM_ITEM)
            .expect("H.B. 215 carries the lump");
        assert!(lump.amount > 4.0e9, "the lump is {:.0}", lump.amount);
        assert_eq!(
            as_passed
                .iter()
                .find(|l| l.line_item == "200501")
                .expect("the formula's line is in the table")
                .amount,
            0.0
        );
        assert_eq!(deferred_years(), [1999].into_iter().collect());

        // And as H.B. 770 left it: the lump struck to zero and the formula funded.
        let operative = line_history("200501");
        assert!(
            operative[&1999] > 2.9e9,
            "FY1999 base cost is {:.0} under the operative act",
            operative[&1999]
        );
        assert_eq!(line_history(LUMP_SUM_ITEM)[&1999], 0.0);
    }

    /// The vintage rule, which is the difference between two enacted answers for one year.
    #[test]
    fn the_operative_act_answers_and_the_superseded_one_is_kept() {
        for year in FIRST_YEAR..=LAST_YEAR {
            let bill = operative(year).expect("every year has an operative act");
            assert!(
                operative_lines()
                    .iter()
                    .any(|l| l.fiscal_year == year && l.bill == bill),
                "FY{year} has no rows from {bill}"
            );
        }
        // H.B. 215's rows are still in the fixture and reach no total.
        assert!(lines().iter().any(|l| l.bill == "hb215"));
        assert!(!operative_lines().iter().any(|l| l.bill == "hb215"));

        // The two readings of FY1999 differ by more than a rounding: about $175 million.
        let superseded: f64 = lines()
            .iter()
            .filter(|l| {
                l.bill == "hb215"
                    && l.fiscal_year == 1999
                    && !crate::appropriations::is_tax_reimbursement(&l.line_item, 1999)
            })
            .map(|l| l.amount)
            .sum();
        let governing = department_total()[&1999];
        assert!(
            governing - superseded > 1.0e8,
            "the two readings differ by {:.0}",
            governing - superseded
        );
    }

    /// The formula's share is keyed on the fund as well as the number, because the number collides.
    #[test]
    fn the_formulas_own_lines_do_not_pick_up_a_testing_programme() {
        let foundation = foundation_funding();
        for year in FIRST_YEAR..=LAST_YEAR {
            assert!(
                foundation[&year] > 2.0e9,
                "FY{year} foundation funding is {:.0}",
                foundation[&year]
            );
            assert!(foundation[&year] < department_total()[&year]);
        }
        // `200610` is Guidance and Testing under fund 454 in the same table, and it is small.
        // `200610` is Guidance and Testing under fund 454 in the same tables, half a million a
        // year against a base cost in the billions. Keyed on the number alone it would be added
        // to the formula's share every year.
        let stray: Vec<Appropriation> = operative_lines()
            .into_iter()
            .filter(|l| l.line_item == "200610" && l.fund != "017")
            .collect();
        assert_eq!(stray.len(), 4, "the colliding line is not in every year");
        for line in &stray {
            assert!(
                line.amount > 0.0 && line.amount < 1.0e6,
                "FY{} carries {:.0} on the colliding number",
                line.fiscal_year,
                line.amount
            );
        }
    }

    /// The zeros are the act's, not the reader's — most of the department goes to zero together.
    #[test]
    fn fy1999s_zeros_are_a_whole_column_and_not_a_missing_row() {
        let zeroed = lines()
            .into_iter()
            .filter(|l| l.fiscal_year == 1999 && l.fund == "GRF" && l.amount == 0.0)
            .count();
        assert!(
            zeroed > 40,
            "only {zeroed} GRF lines are zero in FY1999; a handful would be a parse failure, a \
             column of them is the act"
        );
    }

    /// What the lump was spent as, from a source already committed.
    ///
    /// The strongest evidence that the promise in H.B. 215 was kept, and it needs no new
    /// retrieval: the greenbook's FY1999 actuals carry no `200405` at all, and the foundation line
    /// the act appropriated nothing to spent three billion.
    #[test]
    fn the_lump_never_appears_in_what_was_actually_spent() {
        let actuals: Vec<crate::appropriations::Line> = crate::appropriations::lines()
            .into_iter()
            .filter(|l| l.fiscal_year == 1999 && l.kind == "actual")
            .collect();
        assert!(actuals.len() > 100, "{} FY1999 actuals", actuals.len());
        assert!(
            !actuals.iter().any(|l| l.line_item == LUMP_SUM_ITEM),
            "the lump line has an actual, so it was spent under its own name after all"
        );
        let foundation = actuals
            .iter()
            .find(|l| l.line_item == "200501")
            .expect("the foundation line has a FY1999 actual");
        assert!(
            foundation.amount > 3.0e9,
            "FY1999 foundation actual is {:.0}",
            foundation.amount
        );
    }

    /// The two acts are two publishers' worth of care apart, so the fund codes must survive both.
    #[test]
    fn every_line_carries_a_six_digit_item_and_a_fund() {
        for line in lines() {
            assert_eq!(line.line_item.len(), 6, "{line:?}");
            assert!(line.line_item.chars().all(|c| c.is_ascii_digit()));
            assert_eq!(line.fund.len(), 3, "{line:?}");
            assert!(!line.title.is_empty(), "{line:?}");
        }
    }
}
