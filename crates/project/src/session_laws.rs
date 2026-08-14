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
//! # FY1999 has an appropriation and does not have a breakdown
//!
//! [`lump_sum_years`] is the important function here. Nine months after the Supreme Court held the
//! funding system unconstitutional, Am. Sub. H.B. 215 appropriated FY1998 across fifty-three GRF
//! education lines and FY1999 across **one**: `200405`, Primary and Secondary Education Funding,
//! $4,470,135,592, with fifty-one of the other lines at zero. The act says why:
//!
//! > By January 15, 1998, the General Assembly shall develop a plan to provide itemized
//! > appropriations for the Department of Education for fiscal year 1999.
//!
//! So FY1999's zeros are claims and not gaps, and no per-line comparison may cross FY1998/FY1999.
//! [`comparable_years`] is the set that may.
//!
//! The promise was kept: H.B. 650 itemised the year in February 1998 and H.B. 770 corrected it in
//! June. Neither is wired — both print every amended row twice, struck and inserted, and H.B. 650
//! reprints H.B. 215's totals unchanged, so the reconciliation that guards every other extraction
//! here would pass against the superseded number. Until they are read, FY1999 carries the
//! appropriation **as enacted at passage** and says so.
//!
//! The corroboration that the promise was kept is already committed: the H.B. 94 greenbook's
//! FY1999 *actuals* in `appropriation-lines.csv` carry 141 line items and **no `200405` at all**,
//! with `200501` spending $3,035,363,396. A line appropriated $4.47 billion and spent nothing,
//! because by the time the money moved it had been given its proper names.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/session-law-lines.csv");

const EXPECTED_HEADER: &str =
    "general_assembly,bill,fiscal_year,fund_group,fund,line_item,title,amount";

/// The first fiscal year any act in this repository appropriates.
pub const FIRST_YEAR: u16 = 1998;

/// The last, after which [`crate::appropriations`] answers.
pub const LAST_YEAR: u16 = 2001;

/// The line item that stands in for a whole year's itemisation.
pub const LUMP_SUM_ITEM: &str = "200405";

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
    let mut rows = FIXTURE.lines();
    assert_eq!(
        rows.next().unwrap_or_default().trim(),
        EXPECTED_HEADER,
        "the session-law fixture header changed; update project::session_laws"
    );
    rows.filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            Some(Appropriation {
                general_assembly: f.first()?.parse().ok()?,
                bill: f.get(1)?.to_string(),
                fiscal_year: f.get(2)?.parse().ok()?,
                fund_group: f.get(3)?.to_string(),
                fund: f.get(4)?.to_string(),
                line_item: f.get(5)?.to_string(),
                title: f.get(6)?.to_string(),
                amount: f.get(7)?.parse().ok()?,
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
    for line in lines() {
        if crate::appropriations::is_tax_reimbursement(&line.line_item, line.fiscal_year) {
            continue;
        }
        *out.entry(line.fiscal_year).or_default() += line.amount;
    }
    out
}

/// Years whose appropriation is a single undifferentiated line rather than an itemisation.
///
/// FY1999, and only FY1999. Returned as a set rather than asserted as a constant so that a
/// consumer branches on the property instead of on the year.
#[must_use]
pub fn lump_sum_years() -> BTreeSet<u16> {
    lines()
        .into_iter()
        .filter(|line| line.line_item == LUMP_SUM_ITEM && line.amount > 0.0)
        .map(|line| line.fiscal_year)
        .collect()
}

/// The years whose line items may be compared with one another.
///
/// Everything this repository holds from the acts, less the years [`lump_sum_years`] names. A
/// per-line series drawn across a lump-sum year reports fifty-one programmes going to zero and
/// then returning, which is a fact about the act's drafting and not about any programme.
#[must_use]
pub fn comparable_years() -> Vec<u16> {
    let lump = lump_sum_years();
    (FIRST_YEAR..=LAST_YEAR)
        .filter(|year| !lump.contains(year))
        .collect()
}

/// One line's amount across every year the acts state it.
#[must_use]
pub fn line_history(line_item: &str) -> BTreeMap<u16, f64> {
    lines()
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

    /// The finding, pinned: one year of the biennium was not itemised, and the act says why.
    #[test]
    fn fy1999_is_appropriated_as_one_line_and_fy1998_is_not() {
        assert_eq!(lump_sum_years(), [1999].into_iter().collect());
        assert_eq!(comparable_years(), vec![1998, 2000, 2001]);

        let lump = line_history(LUMP_SUM_ITEM);
        assert_eq!(lump[&1998], 0.0, "the lump is FY1999's and not FY1998's");
        assert!(
            lump[&1999] > 4.0e9,
            "FY1999's lump is {:.0}, not the 4.47bn the act states",
            lump[&1999]
        );

        // And the formula's own line is the mirror image: funded in FY1998, zero in FY1999.
        let foundation = line_history("200501");
        assert!(foundation[&1998] > 2.0e9);
        assert_eq!(foundation[&1999], 0.0);
        assert!(foundation[&2000] > 3.0e9, "FY2000 itemises again");
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
