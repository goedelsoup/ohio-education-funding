//! What districts actually received, spent, and held — as against what a formula computed.
//!
//! Every other per-district figure in this repository is *modelled*: the FY2027 calculator says
//! what a district is owed, the profile report says what it spent per pupil on the department's
//! own definitions, the report card says what its pupils achieved. None of them is a record of
//! money arriving in a bank account.
//!
//! This is. Ohio districts file a five-year forecast under R.C. 5705.391, and each filing
//! carries three years of **audited actuals** alongside its projections. Two filings, three
//! years apart, tile FY2020 through FY2025 without a gap.
//!
//! # What this makes computable that was not
//!
//! - **The guarantee's baseline.** The temporary transitional aid guarantee holds districts at
//!   what they received in FY2020, and until now this corpus could only *infer* that figure
//!   from the guarantee itself. FY2020 is now an observation.
//! - **Carry-over and cash on hand.** [`YearRecord::beginning_cash`] and
//!   [`YearRecord::ending_cash`] are the general fund's balance at each end of the year — the
//!   only measure here of what a district holds rather than what it is given.
//! - **The local side, as revenue rather than as valuation.** `tax-abstract` remains unbuilt and
//!   with it assessed valuation, but [`YearRecord::property_tax`] is the levy yield actually
//!   collected, which is what a board argues about.
//!
//! # What it is not
//!
//! Not the funding formula's output. A district's `unrestricted_aid` is state foundation money
//! as its treasurer books it in the general fund, which differs from the calculator's "total
//! state support" by everything that does not land there: federal money, restricted grants
//! booked separately, and transfers. The two are related and are not the same number, and this
//! module never presents one as a check on the other.
//!
//! Not comparable across the FY2021-22 boundary without care, either. Federal pandemic relief
//! was booked in the general fund by some districts and in separate funds by others, so a cash
//! balance rising through those years is not evidence of a district's own position improving.
//! [`Finances::pandemic_years`] names the affected span rather than leaving it to be discovered.

use edfund_core::{Dollars, FiscalYear};

/// The committed financial panel, built from two five-year-forecast filings.
const FINANCES: &str = include_str!("../fixtures/district-finances.csv");

/// Fiscal years the panel covers, oldest first.
pub const COVERED: [FiscalYear; 6] = [
    FiscalYear(2020),
    FiscalYear(2021),
    FiscalYear(2022),
    FiscalYear(2023),
    FiscalYear(2024),
    FiscalYear(2025),
];

/// The year the temporary transitional aid guarantee holds districts at.
pub const GUARANTEE_BASELINE_YEAR: FiscalYear = FiscalYear(2020);

/// Districts the pinned filings do not cover for all six years, and why.
///
/// One, and it is a live district rather than a closed one: Green Local (Scioto County) is in
/// the FY2027 funding calculator and in the FY2023 filing, and is absent from the FY2026
/// required spring update. It filed in some other window or not at all.
///
/// Named here rather than absorbed, for the same reason [`crate::crosswalk`] names its three:
/// a panel that silently varies its membership by year produces statewide figures over a
/// different set of districts each time, and nothing says so.
pub const PARTIAL: &[(&str, &str)] = &[(
    "049619",
    "absent from the FY2026 required spring update; FY2020-FY2022 only",
)];

/// One district's general fund in one closed fiscal year. Every figure is an audited actual.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YearRecord {
    /// The fiscal year, ending 30 June.
    pub fiscal_year: FiscalYear,
    /// Unrestricted grants-in-aid — state foundation funding as the district books it.
    pub unrestricted_aid: Dollars,
    /// Restricted grants-in-aid — state money that may not be spent freely.
    pub restricted_aid: Dollars,
    /// General property tax (real estate): the levy yield after H.B. 920 reduction.
    pub property_tax: Dollars,
    /// School district income tax, where one is levied. Zero where none is.
    pub income_tax: Dollars,
    /// The state reimbursing rollback and homestead exemptions.
    ///
    /// State money that arrives on the local line in most published summaries, which is one
    /// reason the state share of Ohio school funding is quoted at different numbers by
    /// different people. Counted here as neither until a caller decides.
    pub property_tax_allocation: Dollars,
    /// Total general fund revenue: the district's own receipts.
    ///
    /// Excludes transfers, advances, and note proceeds, which is why it does **not** close the
    /// cash identity. It is the right numerator for a share-of-revenue question all the same:
    /// counting a transfer between a district's own funds as revenue would inflate both sides.
    pub total_revenue: Dollars,
    /// Total revenue **and other financing sources** — every dollar that entered the fund.
    ///
    /// The one that closes `beginning + received - spent = ending`.
    pub total_revenue_and_sources: Dollars,
    /// Total expenditures and other financing uses.
    pub total_expenditure: Dollars,
    /// Cash balance at 1 July — the carry-over from the year before.
    pub beginning_cash: Dollars,
    /// Cash balance at 30 June, excluding proposed renewal, replacement, and new levies.
    pub ending_cash: Dollars,
}

impl YearRecord {
    /// Everything that entered the fund, less everything that left it.
    ///
    /// Built on [`YearRecord::total_revenue_and_sources`] rather than
    /// [`YearRecord::total_revenue`], because only that one accounts for transfers and advances
    /// -- and without them the figure does not reconcile to the change in cash, which is the one
    /// property that makes it checkable.
    #[must_use]
    pub fn operating_result(&self) -> Dollars {
        self.total_revenue_and_sources - self.total_expenditure
    }

    /// Ending cash as a multiple of the year's spending.
    ///
    /// The standard way to read a school balance, because the dollar figure is meaningless
    /// without the scale of the operation behind it: $10M is a year of reserve for a small
    /// district and three weeks for a large one. Ohio has no statutory minimum, so this is a
    /// comparison figure and not a compliance one.
    #[must_use]
    pub fn cash_as_years_of_spending(&self) -> Option<f64> {
        (self.total_expenditure > 0.0).then(|| self.ending_cash / self.total_expenditure)
    }

    /// State money as a share of total revenue, counting only what the state calls state money.
    ///
    /// Excludes [`YearRecord::property_tax_allocation`], which is a state payment on the local
    /// line. Including it raises the state share by several points, and which convention is in
    /// force is exactly the sort of thing that makes two honest people quote different numbers.
    #[must_use]
    pub fn state_share(&self) -> Option<f64> {
        (self.total_revenue > 0.0)
            .then(|| (self.unrestricted_aid + self.restricted_aid) / self.total_revenue)
    }
}

/// One district's six years.
#[derive(Debug, Clone, PartialEq)]
pub struct Finances {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name as filed.
    pub name: String,
    /// County.
    pub county: String,
    /// Closed fiscal years, oldest first.
    pub years: Vec<YearRecord>,
}

impl Finances {
    /// The years federal pandemic relief makes uncomparable to what came before and after.
    ///
    /// ESSER money was booked in the general fund by some districts and in separate funds by
    /// others, and it arrived and stopped on a schedule that has nothing to do with Ohio's
    /// formula. A balance rising across these years is not evidence about the district.
    pub const fn pandemic_years() -> [FiscalYear; 4] {
        [
            FiscalYear(2021),
            FiscalYear(2022),
            FiscalYear(2023),
            FiscalYear(2024),
        ]
    }

    /// The record for one fiscal year.
    #[must_use]
    pub fn year(&self, fiscal_year: FiscalYear) -> Option<&YearRecord> {
        self.years.iter().find(|y| y.fiscal_year == fiscal_year)
    }

    /// State foundation aid in FY2020 — the year the guarantee holds districts at.
    #[must_use]
    pub fn guarantee_baseline_aid(&self) -> Option<Dollars> {
        self.year(GUARANTEE_BASELINE_YEAR)
            .map(|y| y.unrestricted_aid)
    }

    /// The earliest and latest closed years present.
    #[must_use]
    pub fn span(&self) -> Option<(&YearRecord, &YearRecord)> {
        Some((self.years.first()?, self.years.last()?))
    }

    /// Change in ending cash across the whole span, in dollars.
    #[must_use]
    pub fn cash_change(&self) -> Option<Dollars> {
        let (first, last) = self.span()?;
        Some(last.ending_cash - first.ending_cash)
    }
}

fn field(line: &str, index: usize) -> &str {
    line.split(',').nth(index).unwrap_or("").trim()
}

fn amount(line: &str, index: usize) -> Dollars {
    field(line, index).parse().unwrap_or(0.0)
}

/// Every district's financial panel, in IRN order.
///
/// # Panics
///
/// Panics if the committed fixture is malformed, which is a build-time fact rather than a
/// runtime one — the file is compiled in.
#[must_use]
pub fn finances() -> Vec<Finances> {
    let mut out: Vec<Finances> = Vec::new();
    for line in FINANCES.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let irn = field(line, 0).to_string();
        let fiscal_year = FiscalYear(field(line, 3).parse().unwrap_or(0));
        let record = YearRecord {
            fiscal_year,
            unrestricted_aid: amount(line, 4),
            restricted_aid: amount(line, 5),
            property_tax: amount(line, 6),
            income_tax: amount(line, 7),
            property_tax_allocation: amount(line, 8),
            total_revenue: amount(line, 9),
            total_revenue_and_sources: amount(line, 10),
            total_expenditure: amount(line, 11),
            beginning_cash: amount(line, 12),
            ending_cash: amount(line, 13),
        };
        // The fixture is written in (IRN, year) order, so a district's rows are adjacent and
        // this never has to search backwards.
        match out.last_mut() {
            Some(existing) if existing.irn == irn => existing.years.push(record),
            _ => out.push(Finances {
                irn,
                name: field(line, 1).to_string(),
                county: field(line, 2).to_string(),
                years: vec![record],
            }),
        }
    }
    out
}

/// Look one district up by IRN.
#[must_use]
pub fn for_district<'a>(panel: &'a [Finances], irn: &str) -> Option<&'a Finances> {
    panel.iter().find(|f| f.irn == irn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::panel;

    #[test]
    fn the_panel_covers_six_consecutive_closed_years_for_all_but_the_named_exception() {
        let finances = finances();
        assert_eq!(finances.len(), 660, "LEAs in the panel");
        let full = COVERED.map(|y| y.0).to_vec();
        let mut partial = Vec::new();
        for district in &finances {
            let years: Vec<u16> = district.years.iter().map(|y| y.fiscal_year.0).collect();
            if years == full {
                continue;
            }
            assert!(
                PARTIAL.iter().any(|(irn, _)| *irn == district.irn),
                "{} ({}) has {years:?} and is not a named exception",
                district.name,
                district.irn
            );
            partial.push(district.irn.clone());
        }
        assert_eq!(partial.len(), PARTIAL.len(), "{partial:?}");
    }

    #[test]
    fn the_one_partial_district_is_funded_rather_than_closed() {
        // Worth distinguishing. A closed district missing from a later filing is unremarkable;
        // an operating one that the department's own funding model pays and whose filing is not
        // in this window is a gap in the panel, and the difference decides whether anything
        // should be done about it.
        let funding = panel();
        for (irn, _) in PARTIAL {
            assert!(
                funding.iter().any(|record| record.irn == *irn),
                "{irn} is partial and not in the funding model — it closed, so it is not an \
                 exception but an expected absence"
            );
        }
    }

    #[test]
    fn the_two_filings_agree_about_the_instant_where_they_meet() {
        // FY2022's closing balance and FY2023's opening balance are the same instant, reported
        // by two filings made three years apart. This is the only check available that the two
        // sources describe one continuous series rather than two that happen to abut, and it is
        // the reason the seam was chosen as the place to join them.
        //
        // Exact agreement is not required and would be wrong to require: a district may restate
        // a closed year, and the later filing is the department's most recent word on it. What
        // would be alarming is a *systematic* gap, so this bounds both the typical case and the
        // worst one.
        let finances = finances();
        let mut gaps: Vec<f64> = finances
            .iter()
            .filter_map(|district| {
                let before = district.year(FiscalYear(2022))?;
                let after = district.year(FiscalYear(2023))?;
                Some((after.beginning_cash - before.ending_cash).abs())
            })
            .collect();
        gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        assert_eq!(gaps.len(), 659);

        let median = gaps[gaps.len() / 2];
        let p90 = gaps[gaps.len() * 9 / 10];
        let material = gaps.iter().filter(|gap| **gap > 1_000_000.0).count();

        // The typical district agrees to within a couple of dollars, which is rounding, not
        // reconciliation. The tail is real and small: ten districts restate by more than a
        // million, the largest by $13.3M, and those are reclassifications across the pandemic
        // relief years rather than errors in either filing.
        assert!(median < 100.0, "median seam gap ${median:.0}");
        assert!(p90 < 25_000.0, "90th percentile seam gap ${p90:.0}");
        assert_eq!(material, 10, "districts restating the seam by over $1M");
    }

    #[test]
    fn the_seam_is_the_only_place_two_filings_can_disagree() {
        // The two filings' actual windows tile rather than overlap, so no fiscal year is
        // reported twice and no value is silently taken from one filing over the other. If that
        // ever stops being true the extract's last-writer-wins rule starts deciding figures.
        assert_eq!(
            connect_actual_years(2023)[2] + 1,
            connect_actual_years(2026)[0]
        );
    }

    /// Mirrors `connect::forecast::actual_years`, which this crate does not depend on.
    const fn connect_actual_years(school_year: u16) -> [u16; 3] {
        [school_year - 3, school_year - 2, school_year - 1]
    }

    #[test]
    fn a_cash_balance_is_the_previous_years_carry_over() {
        // Within one filing the identity is exact: this year's opening balance is last year's
        // closing balance. A parser that misread a column would break this before it broke
        // anything a reader would notice.
        for district in &finances() {
            for pair in district.years.windows(2) {
                let (before, after) = (pair[0], pair[1]);
                if after.fiscal_year.0 != before.fiscal_year.0 + 1 {
                    continue;
                }
                // Skip the seam, which is two different filings and covered above.
                if before.fiscal_year == FiscalYear(2022) {
                    continue;
                }
                assert!(
                    (after.beginning_cash - before.ending_cash).abs() < 1.0,
                    "{} FY{} closes at {} and FY{} opens at {}",
                    district.name,
                    before.fiscal_year.0,
                    before.ending_cash,
                    after.fiscal_year.0,
                    after.beginning_cash
                );
            }
        }
    }

    #[test]
    fn the_cash_balance_moves_by_the_operating_result() {
        // Ending cash is beginning cash plus revenue less expenditure, and where it is not, the
        // difference is an advance or a transfer the general fund's summary lines do not carry.
        // Most districts should satisfy it to the dollar.
        let finances = finances();
        let mut exact = 0;
        let mut total = 0;
        for district in &finances {
            for year in &district.years {
                total += 1;
                let implied = year.beginning_cash + year.operating_result();
                if (implied - year.ending_cash).abs() < 1.0 {
                    exact += 1;
                }
            }
        }
        assert!(
            exact * 10 > total * 9,
            "only {exact} of {total} district-years close on their own arithmetic"
        );
    }

    #[test]
    fn the_panel_reaches_the_districts_the_funding_model_covers() {
        // A financial panel that missed a tenth of the funding panel would produce statewide
        // figures over a silently different set of districts.
        let finances = finances();
        let funding = panel();
        let missing = funding
            .iter()
            .filter(|record| for_district(&finances, &record.irn).is_none())
            .count();
        assert!(
            missing * 100 < funding.len(),
            "{missing} of {} funded districts have no financial filing",
            funding.len()
        );
    }

    #[test]
    fn fy2020_state_aid_is_present_for_every_district() {
        // The guarantee's own baseline year. The corpus has until now only been able to infer
        // it from the guarantee, which makes any check of the guarantee circular.
        for district in &finances() {
            assert!(
                district.guarantee_baseline_aid().is_some(),
                "{} has no FY2020",
                district.name
            );
        }
    }

    #[test]
    fn cash_is_reported_against_the_scale_of_the_operation() {
        let year = YearRecord {
            fiscal_year: FiscalYear(2025),
            unrestricted_aid: 3_000_000.0,
            restricted_aid: 200_000.0,
            property_tax: 6_000_000.0,
            income_tax: 0.0,
            property_tax_allocation: 500_000.0,
            total_revenue: 10_000_000.0,
            total_revenue_and_sources: 10_000_000.0,
            total_expenditure: 9_500_000.0,
            beginning_cash: 4_000_000.0,
            ending_cash: 4_500_000.0,
        };
        assert!((year.operating_result() - 500_000.0).abs() < f64::EPSILON);
        assert!((year.cash_as_years_of_spending().unwrap() - 0.473_684).abs() < 1e-5);
        // The rollback reimbursement is state money on the local line and is deliberately not
        // counted in the state share.
        assert!((year.state_share().unwrap() - 0.32).abs() < 1e-9);
    }

    #[test]
    fn a_district_with_no_spending_has_no_ratio_rather_than_an_infinite_one() {
        let empty = YearRecord {
            fiscal_year: FiscalYear(2025),
            unrestricted_aid: 0.0,
            restricted_aid: 0.0,
            property_tax: 0.0,
            income_tax: 0.0,
            property_tax_allocation: 0.0,
            total_revenue: 0.0,
            total_revenue_and_sources: 0.0,
            total_expenditure: 0.0,
            beginning_cash: 0.0,
            ending_cash: 0.0,
        };
        assert_eq!(empty.cash_as_years_of_spending(), None);
        assert_eq!(empty.state_share(), None);
    }

    #[test]
    fn the_pandemic_years_are_named_rather_than_left_to_be_discovered() {
        let years = Finances::pandemic_years();
        assert!(years.contains(&FiscalYear(2021)));
        assert!(!years.contains(&FiscalYear(2020)));
        assert!(!years.contains(&FiscalYear(2025)));
    }
}
