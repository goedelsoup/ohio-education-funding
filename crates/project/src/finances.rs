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

/// Districts whose filing carries a year but not every line of it, and which lines.
///
/// [`PARTIAL`] names districts missing whole *years*, which a row count finds. This names the
/// other hole, which a row count cannot: a filing that reports a district's revenue and simply
/// has no expenditure or cash line for it. The row is present and full-width, so nothing about
/// its shape says anything is missing — only the `None`s do.
///
/// Toronto City is the one case in 660 districts. Its FY2023 filing omits lines 5.050, 7.010
/// and 7.020 entirely, so FY2020 through FY2022 have revenue and no spending and no balance.
/// It is not recoverable by re-fetching: the lines are absent from the published file. The
/// FY2026 filing carries the district in full, which is why only the first three years are
/// short — and its FY2023 opening balance of $8,140,842 is the one figure the seam recovers,
/// for FY2022's close alone.
///
/// Until FY2026 this read as `0`, and the district published $0 of expenditure against $9.86M
/// of revenue in the live feed for three years while every identity check passed: `0 - 0` is a
/// balance that carries over perfectly.
pub const INCOMPLETE: &[(&str, &str)] = &[(
    "044917",
    "the FY2023 filing carries no line 5.050, 7.010 or 7.020; FY2020-FY2022 have no \
     expenditure and no cash balances",
)];

/// One district's general fund in one closed fiscal year. Every figure is an audited actual.
///
/// Every amount is optional because a filing may simply not carry a line, and one does: see
/// [`INCOMPLETE`]. `None` is *not reported*, which is a different fact from zero and must stay
/// different — a district that omitted line 5.050 did not spend nothing, and summing its
/// absence as zero understates every statewide total it enters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YearRecord {
    /// The fiscal year, ending 30 June.
    pub fiscal_year: FiscalYear,
    /// Unrestricted grants-in-aid — state foundation funding as the district books it.
    pub unrestricted_aid: Option<Dollars>,
    /// Restricted grants-in-aid — state money that may not be spent freely.
    pub restricted_aid: Option<Dollars>,
    /// General property tax (real estate): the levy yield after H.B. 920 reduction.
    pub property_tax: Option<Dollars>,
    /// School district income tax, where one is levied. Zero where none is.
    pub income_tax: Option<Dollars>,
    /// The state reimbursing rollback and homestead exemptions.
    ///
    /// State money that arrives on the local line in most published summaries, which is one
    /// reason the state share of Ohio school funding is quoted at different numbers by
    /// different people. Counted here as neither until a caller decides.
    pub property_tax_allocation: Option<Dollars>,
    /// Total general fund revenue: the district's own receipts.
    ///
    /// Excludes transfers, advances, and note proceeds, which is why it does **not** close the
    /// cash identity. It is the right numerator for a share-of-revenue question all the same:
    /// counting a transfer between a district's own funds as revenue would inflate both sides.
    pub total_revenue: Option<Dollars>,
    /// Total revenue **and other financing sources** — every dollar that entered the fund.
    ///
    /// The one that closes `beginning + received - spent = ending`.
    pub total_revenue_and_sources: Option<Dollars>,
    /// Total expenditures and other financing uses.
    pub total_expenditure: Option<Dollars>,
    /// Cash balance at 1 July — the carry-over from the year before.
    pub beginning_cash: Option<Dollars>,
    /// Cash balance at 30 June, excluding proposed renewal, replacement, and new levies.
    pub ending_cash: Option<Dollars>,
}

impl YearRecord {
    /// Everything that entered the fund, less everything that left it.
    ///
    /// Built on [`YearRecord::total_revenue_and_sources`] rather than
    /// [`YearRecord::total_revenue`], because only that one accounts for transfers and advances
    /// -- and without them the figure does not reconcile to the change in cash, which is the one
    /// property that makes it checkable.
    #[must_use]
    pub fn operating_result(&self) -> Option<Dollars> {
        Some(self.total_revenue_and_sources? - self.total_expenditure?)
    }

    /// Ending cash as a multiple of the year's spending.
    ///
    /// The standard way to read a school balance, because the dollar figure is meaningless
    /// without the scale of the operation behind it: $10M is a year of reserve for a small
    /// district and three weeks for a large one. Ohio has no statutory minimum, so this is a
    /// comparison figure and not a compliance one.
    #[must_use]
    pub fn cash_as_years_of_spending(&self) -> Option<f64> {
        let (cash, spending) = (self.ending_cash?, self.total_expenditure?);
        (spending > 0.0).then_some(cash / spending)
    }

    /// State money as a share of total revenue, counting only what the state calls state money.
    ///
    /// Excludes [`YearRecord::property_tax_allocation`], which is a state payment on the local
    /// line. Including it raises the state share by several points, and which convention is in
    /// force is exactly the sort of thing that makes two honest people quote different numbers.
    #[must_use]
    pub fn state_share(&self) -> Option<f64> {
        let revenue = self.total_revenue?;
        let aid = self.unrestricted_aid? + self.restricted_aid?;
        (revenue > 0.0).then_some(aid / revenue)
    }
}

impl YearRecord {
    /// This year restated in `base`-year dollars.
    ///
    /// # Why nominal is not good enough here
    ///
    /// Every figure in this panel is nominal, and the span it covers — FY2020 to FY2025 —
    /// contains the sharpest price change in forty years: **CPI-U June rose 25.1%** across it.
    /// A district whose state aid rose 5% over those six years did not gain; it lost a fifth of
    /// its purchasing power. Any statement about this panel made in nominal dollars is not
    /// merely imprecise, it can have the wrong sign.
    ///
    /// The cash balance is converted too, on the same index. That is the right treatment for a
    /// question about what a balance *buys*, which is the question anyone asks of a reserve.
    ///
    /// The index is CPI-U, a general consumer index; school costs are majority compensation,
    /// for which the Employment Cost Index would be better and has shorter coverage. Any figure
    /// this produces must name the index, which is why [`deflator::CpiSeries::label`] exists.
    ///
    /// # Errors
    ///
    /// Returns [`deflator::DeflatorError`] if either year is absent from the series.
    pub fn in_dollars_of(
        &self,
        base: FiscalYear,
        cpi: &deflator::CpiSeries,
    ) -> Result<Self, deflator::DeflatorError> {
        // An absent figure stays absent: there is nothing to deflate, and inventing a zero here
        // would put the bug this guards against back in constant dollars.
        let at = |value: Option<Dollars>| match value {
            Some(value) => cpi
                .convert(value, self.fiscal_year, base)
                .map(|d| Some(d.value)),
            None => Ok(None),
        };
        Ok(Self {
            fiscal_year: self.fiscal_year,
            unrestricted_aid: at(self.unrestricted_aid)?,
            restricted_aid: at(self.restricted_aid)?,
            property_tax: at(self.property_tax)?,
            income_tax: at(self.income_tax)?,
            property_tax_allocation: at(self.property_tax_allocation)?,
            total_revenue: at(self.total_revenue)?,
            total_revenue_and_sources: at(self.total_revenue_and_sources)?,
            total_expenditure: at(self.total_expenditure)?,
            beginning_cash: at(self.beginning_cash)?,
            ending_cash: at(self.ending_cash)?,
        })
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
            .and_then(|y| y.unrestricted_aid)
    }

    /// The earliest and latest closed years present.
    #[must_use]
    pub fn span(&self) -> Option<(&YearRecord, &YearRecord)> {
        Some((self.years.first()?, self.years.last()?))
    }

    /// The whole panel restated in `base`-year dollars.
    ///
    /// # Errors
    ///
    /// Returns [`deflator::DeflatorError`] if any covered year is absent from the series.
    pub fn in_dollars_of(
        &self,
        base: FiscalYear,
        cpi: &deflator::CpiSeries,
    ) -> Result<Self, deflator::DeflatorError> {
        Ok(Self {
            irn: self.irn.clone(),
            name: self.name.clone(),
            county: self.county.clone(),
            years: self
                .years
                .iter()
                .map(|year| year.in_dollars_of(base, cpi))
                .collect::<Result<_, _>>()?,
        })
    }

    /// Real change in a measure across the span, as a fraction.
    ///
    /// The operation most Ohio school finance arguments actually need, and the one most often
    /// skipped. `real_change(|y| y.unrestricted_aid)` answers "did this district's state aid
    /// keep up with prices", which nominal figures cannot.
    ///
    /// # Errors
    ///
    /// Returns [`deflator::DeflatorError`] if either endpoint year is absent from the series.
    pub fn real_change(
        &self,
        cpi: &deflator::CpiSeries,
        pick: impl Fn(&YearRecord) -> Option<Dollars>,
    ) -> Result<Option<f64>, deflator::DeflatorError> {
        let Some((first, last)) = self.span() else {
            return Ok(None);
        };
        let (Some(first_value), Some(last_value)) = (pick(first), pick(last)) else {
            return Ok(None);
        };
        if first_value <= 0.0 {
            return Ok(None);
        }
        Ok(Some(
            cpi.real_growth(first_value, first.fiscal_year, last_value, last.fiscal_year)?
                .value,
        ))
    }
}

/// The header this reader's column positions are written against.
///
/// Asserted on every read. This module indexed thirteen numeric columns by bare position with
/// nothing checking the header at all, so a column inserted upstream would have moved
/// `ending_cash` into `beginning_cash` and every cash figure in the corpus would have been
/// wrong and parsed cleanly.
const EXPECTED_HEADER: &str = "irn,name,county,fiscal_year,unrestricted_aid,restricted_aid,\
property_tax,income_tax,property_tax_allocation,total_revenue,total_revenue_and_sources,\
total_expenditure,beginning_cash,ending_cash";

/// Every district's financial panel, in IRN order.
///
/// # Panics
///
/// Panics if the committed fixture is malformed, which is a build-time fact rather than a
/// runtime one — the file is compiled in.
#[must_use]
pub fn finances() -> Vec<Finances> {
    let mut out: Vec<Finances> = Vec::new();
    for row in edfund_core::csv::rows(FINANCES, EXPECTED_HEADER) {
        let irn = row.str(0).to_string();
        let fiscal_year = FiscalYear(row.str(3).parse().unwrap_or(0));
        let record = YearRecord {
            fiscal_year,
            // `num`, not `required`: an empty cell is a line the filing did not carry, and
            // `required` would read it as zero. See [`INCOMPLETE`].
            unrestricted_aid: row.num(4),
            restricted_aid: row.num(5),
            property_tax: row.num(6),
            income_tax: row.num(7),
            property_tax_allocation: row.num(8),
            total_revenue: row.num(9),
            total_revenue_and_sources: row.num(10),
            total_expenditure: row.num(11),
            beginning_cash: row.num(12),
            ending_cash: row.num(13),
        };
        // The fixture is written in (IRN, year) order, so a district's rows are adjacent and
        // this never has to search backwards.
        match out.last_mut() {
            Some(existing) if existing.irn == irn => existing.years.push(record),
            _ => out.push(Finances {
                irn,
                name: row.str(1).to_string(),
                county: row.str(2).to_string(),
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
                let before = district.year(FiscalYear(2022))?.ending_cash?;
                let after = district.year(FiscalYear(2023))?.beginning_cash?;
                Some((after - before).abs())
            })
            .collect();
        gaps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // 659 districts have both years; Toronto City has no FY2022 closing balance to compare
        // against, so it is one short of that. See [`INCOMPLETE`].
        assert_eq!(gaps.len(), 658);

        let median = gaps[gaps.len() / 2];
        let p90 = gaps[gaps.len() * 9 / 10];
        let material = gaps.iter().filter(|gap| **gap > 1_000_000.0).count();

        // The typical district agrees to within a couple of dollars, which is rounding, not
        // reconciliation. The tail is real and small: nine districts restate by more than a
        // million, the largest by $13.3M, and those are reclassifications across the pandemic
        // relief years rather than errors in either filing.
        //
        // It was ten while Toronto City's absent FY2022 closing balance was read as zero, and
        // the $8,140,842 "restatement" that produced was neither a restatement nor a
        // reclassification — it was the whole of the district's cash, measured against nothing.
        // The count is what carried the claim, so it is the count that had to move.
        assert!(median < 100.0, "median seam gap ${median:.0}");
        assert!(p90 < 25_000.0, "90th percentile seam gap ${p90:.0}");
        assert_eq!(material, 9, "districts restating the seam by over $1M");
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
                // A year with no reported balance has nothing to carry over. Skipping is not
                // leniency: the alternative reading, zero, satisfies this identity for every
                // pair of absent years and is exactly the misread this test exists to catch.
                let (Some(closing), Some(opening)) = (before.ending_cash, after.beginning_cash)
                else {
                    continue;
                };
                assert!(
                    (opening - closing).abs() < 1.0,
                    "{} FY{} closes at {} and FY{} opens at {}",
                    district.name,
                    before.fiscal_year.0,
                    closing,
                    after.fiscal_year.0,
                    opening
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
                let (Some(opening), Some(result), Some(closing)) = (
                    year.beginning_cash,
                    year.operating_result(),
                    year.ending_cash,
                ) else {
                    continue;
                };
                total += 1;
                if (opening + result - closing).abs() < 1.0 {
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
            unrestricted_aid: Some(3_000_000.0),
            restricted_aid: Some(200_000.0),
            property_tax: Some(6_000_000.0),
            income_tax: Some(0.0),
            property_tax_allocation: Some(500_000.0),
            total_revenue: Some(10_000_000.0),
            total_revenue_and_sources: Some(10_000_000.0),
            total_expenditure: Some(9_500_000.0),
            beginning_cash: Some(4_000_000.0),
            ending_cash: Some(4_500_000.0),
        };
        assert!((year.operating_result().unwrap() - 500_000.0).abs() < f64::EPSILON);
        assert!((year.cash_as_years_of_spending().unwrap() - 0.473_684).abs() < 1e-5);
        // The rollback reimbursement is state money on the local line and is deliberately not
        // counted in the state share.
        assert!((year.state_share().unwrap() - 0.32).abs() < 1e-9);
    }

    #[test]
    fn a_district_with_no_spending_has_no_ratio_rather_than_an_infinite_one() {
        let empty = YearRecord {
            fiscal_year: FiscalYear(2025),
            unrestricted_aid: Some(0.0),
            restricted_aid: Some(0.0),
            property_tax: Some(0.0),
            income_tax: Some(0.0),
            property_tax_allocation: Some(0.0),
            total_revenue: Some(0.0),
            total_revenue_and_sources: Some(0.0),
            total_expenditure: Some(0.0),
            beginning_cash: Some(0.0),
            ending_cash: Some(0.0),
        };
        assert_eq!(empty.cash_as_years_of_spending(), None);
        assert_eq!(empty.state_share(), None);
    }

    #[test]
    fn a_district_that_reported_nothing_is_not_a_district_that_reported_zero() {
        // The distinction the whole module turns on. A year with no expenditure line has no
        // operating result and no cash ratio -- rather than a result equal to its entire
        // revenue, and a ratio of nothing over nothing.
        let unreported = YearRecord {
            fiscal_year: FiscalYear(2022),
            unrestricted_aid: Some(6_388_201.0),
            restricted_aid: Some(239_556.0),
            property_tax: Some(2_257_818.0),
            income_tax: Some(0.0),
            property_tax_allocation: Some(380_181.0),
            total_revenue: Some(9_570_163.0),
            total_revenue_and_sources: Some(9_640_508.0),
            total_expenditure: None,
            beginning_cash: None,
            ending_cash: None,
        };
        assert_eq!(unreported.operating_result(), None);
        assert_eq!(unreported.cash_as_years_of_spending(), None);
        // The lines it does carry are unaffected: the absence is per line, not per year.
        assert!((unreported.state_share().unwrap() - 0.692_544).abs() < 1e-5);
    }

    #[test]
    fn the_only_lines_a_filing_omits_are_the_named_ones() {
        // A row count cannot find this: the fixture is uniform width, so a district missing a
        // line has a row of exactly the same shape as one that reported every line. Without
        // this test the hole reaches the feed and nothing says so, which is what happened.
        let mut holes: Vec<String> = Vec::new();
        for district in &finances() {
            let short = district.years.iter().any(|y| {
                [
                    y.unrestricted_aid,
                    y.restricted_aid,
                    y.property_tax,
                    y.income_tax,
                    y.property_tax_allocation,
                    y.total_revenue,
                    y.total_revenue_and_sources,
                    y.total_expenditure,
                    y.beginning_cash,
                    y.ending_cash,
                ]
                .iter()
                .any(Option::is_none)
            });
            if !short {
                continue;
            }
            assert!(
                INCOMPLETE.iter().any(|(irn, _)| *irn == district.irn),
                "{} ({}) omits a line and is not a named exception",
                district.name,
                district.irn
            );
            holes.push(district.irn.clone());
        }
        assert_eq!(holes.len(), INCOMPLETE.len(), "{holes:?}");
    }

    #[test]
    fn the_incomplete_district_keeps_the_lines_it_did_report() {
        // The correction must not throw the district's revenue away with its expenditure. Its
        // FY2020 state aid is the guarantee's own baseline year, and it is reported.
        let finances = finances();
        let toronto = for_district(&finances, "044917").expect("Toronto City is in the panel");
        let fy2020 = toronto.year(FiscalYear(2020)).expect("FY2020 is filed");
        assert_eq!(fy2020.unrestricted_aid, Some(6_092_893.0));
        assert_eq!(fy2020.total_revenue, Some(9_855_197.0));
        assert_eq!(fy2020.total_expenditure, None);
        assert_eq!(fy2020.ending_cash, None);
        assert_eq!(toronto.guarantee_baseline_aid(), Some(6_092_893.0));
        // The FY2026 filing carries the district in full, so the later years are whole.
        let fy2025 = toronto.year(FiscalYear(2025)).expect("FY2025 is filed");
        assert_eq!(fy2025.total_expenditure, Some(10_796_466.0));
        assert_eq!(fy2025.ending_cash, Some(11_910_201.0));
    }

    #[test]
    fn the_pandemic_years_are_named_rather_than_left_to_be_discovered() {
        let years = Finances::pandemic_years();
        assert!(years.contains(&FiscalYear(2021)));
        assert!(!years.contains(&FiscalYear(2020)));
        assert!(!years.contains(&FiscalYear(2025)));
    }
}
