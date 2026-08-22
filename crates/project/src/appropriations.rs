//! What the General Assembly appropriated to the schools, year by year, in dollars of one year.
//!
//! # Why this is the first thing computed over the series
//!
//! The appropriation series covers FY2010 to FY2027, and CPI-U rose 47% across it. Ohio's school
//! funding argument is conducted almost entirely in nominal totals — "record investment in
//! education" is a true sentence about nearly every biennium in this period and an empty one —
//! so the first question worth asking of a fourteen-year appropriation history is whether it grew.
//!
//! [`deflator`] is the crate that answers it, and this module is the join. Nothing here is new
//! arithmetic; what it adds is that the join is made once, in one place, with the department's
//! own line-item structure preserved rather than collapsed into a headline.
//!
//! # What a total here does and does not mean
//!
//! These are **appropriations**, not payments. What a district received is the department's
//! payment report; what the General Assembly set aside for a line is this. The two differ for
//! ordinary reasons — an appropriation is a ceiling, and the formula's own proration factor
//! exists because at least one line has been a residual claimant — and a difference between them
//! is not an error in either.
//!
//! Two exclusions are built in and both are the publisher's own:
//!
//! - **The property tax reimbursement lines are not the department's budget.** They are numbered
//!   `200xxx` and LSC states they sit in the State Revenue Distributions section. `200903` alone
//!   is $1.3 billion a year, so including them overstates the department by roughly 9%.
//! - **FY2012-13 had no enacted figure in the workbook series at all**, because LSC serves that
//!   biennium's two workbook variants as the same file. That gap is now filled from a second
//!   publication — see below — and the workbook series still carries nothing for it.
//!
//! # Two sources, and which one answers for a year
//!
//! The workbooks and greenbooks carry an enacted figure for FY2002-FY2005, FY2008-FY2011 and
//! FY2014-FY2027 — everything except two bienniums. Those two are the ones
//! [`the-greenbook-series`](../../../.yidam/decisions/the-greenbook-series.yml) recorded as
//! unretrievable: **FY2006-07**, whose greenbook has no line-item table at all, and **FY2012-13**,
//! whose two workbook variants LSC serves as the same file.
//!
//! The [Catalog of Budget Line Items](../../../.yidam/catalog/lsc-catalog-of-budget-line-items.md)
//! carries all four, and contributes **only** those four. With them the enacted series is
//! continuous from FY2002 to FY2027 for the first time.
//!
//! Where both speak, the workbook series answers, and the Catalog is not consulted. That is not a
//! preference between publishers: they agree. Across 1,712 overlapping enacted claims the two
//! extractions do not differ by a cent — see
//! [`two_sources_for_one_appropriation`](../../tests/two_sources_for_one_appropriation.rs) — so
//! the rule exists to make the series reproducible rather than to arbitrate, and a year is
//! answered by exactly one document either way.
//!
//! The Catalog is used **only for enacted figures in years the workbook series does not cover**.
//! Its actuals are deliberately left out: those two sources *do* disagree, about one time in
//! twenty, because an actual is restated as a year closes. Mixing them would build a series whose
//! vintage changed halfway along without saying so.

use std::collections::BTreeMap;

use deflator::{Confidence, CpiSeries};
use edfund_core::FiscalYear;

/// The committed series.
const SERIES: &str = include_str!("../fixtures/appropriation-lines.csv");

/// The Catalog extract, which reaches back further.
const CATALOG: &str = include_str!("../fixtures/catalog-line-items.csv");

/// Line items that are numbered as the department's and are not part of its budget.
///
/// The greenbook's own account: these reimburse districts for tax the state stopped collecting,
/// and are carried in the State Revenue Distributions section rather than in the department's.
/// They are excluded from every total here and available through [`reimbursements`] for anyone
/// asking about them specifically, because they are a large and real flow to districts.
///
/// # Why there are six and not two
///
/// This list was first written from the FY2026-27 greenbook, where the class has exactly two
/// members, and it was wrong for every year before FY2018. The same reimbursements ran under
/// `200900`, `200901` and `200909` from FY2002 to FY2017, and `200901 Property Tax Allocation -
/// Education` alone was $1.14 billion in FY2014 — so a department total that excluded the modern
/// numbers and not the old ones was inflated by roughly $1.65 billion at the start of the window
/// and not at the end.
///
/// The cost of that was a headline with the wrong sign: the department's appropriation appeared
/// to fall in real terms across FY2014-FY2026 and does not. A rule derived from one end of a
/// series and applied to the whole of it is the hazard here, and it is not the first time in this
/// connector's work.
pub const TAX_REIMBURSEMENT: [&str; 7] = [
    "200417", "200900", "200901", "200902", "200903", "200906", "200909",
];

/// Whether a line is a tax reimbursement **in the year it is claimed for**.
///
/// # Why the list alone is not the rule
///
/// The docstring above records making a rule from one end of a series and applying it to the
/// whole. It then did it twice more in the same edit, and both survived until the session laws
/// were read.
///
/// **`200906 Tangible Tax Exemption - Education` was missing.** It is a reimbursement by the
/// definition above, it sits beside `200901` in every table from FY1998 to FY2011, and leaving it
/// out counted $73,500,000 inside the department in FY2002 — 0.94% — falling to $10,707,622 by
/// FY2009. It phases out, so it inflated the early years more than the late ones, which is the
/// same wrong-sign error the docstring describes catching once already.
///
/// **`200417` is not a tax line for most of its life.** It is `Professional Development` from
/// FY1998 to FY2003 and only becomes `Personal Property Tax Replacement Phase Out` in FY2024.
/// Excluding it by number alone was harmless while the series began at FY2002 and there was no
/// enacted figure for it; it stops being harmless the moment FY1998 is read, where H.B. 215
/// appropriates $14,370,077 to it. A rule keyed on the number would delete a real programme as a
/// tax reimbursement.
///
/// So the exclusion is keyed on the number **and** the year, and `200417` is a reimbursement only
/// from FY2024.
#[must_use]
pub fn is_tax_reimbursement(line_item: &str, fiscal_year: u16) -> bool {
    if line_item == "200417" {
        return fiscal_year >= 2024;
    }
    TAX_REIMBURSEMENT.contains(&line_item)
}

/// One appropriation line in one fiscal year.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// The fiscal year the amount belongs to.
    pub fiscal_year: u16,
    /// `enacted`, `actual` or `adjusted`.
    pub kind: String,
    /// The six-digit appropriation line item.
    pub line_item: String,
    /// Its title as the publisher gives it.
    pub title: String,
    /// The amount, in the dollars of `fiscal_year`.
    pub amount: f64,
}

/// Every line in the committed series.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, which means the extractor
/// changed and this reader did not.
#[must_use]
pub fn lines() -> Vec<Line> {
    let mut rows = SERIES.lines();
    let header = rows.next().unwrap_or_default();
    assert_eq!(
        header,
        "general_assembly,bill,fiscal_year,kind,source,documents,fund_group,fund,line_item,title,amount",
        "the appropriation fixture's columns changed and this reader did not"
    );
    rows.filter(|row| !row.trim().is_empty())
        .filter_map(|row| {
            let field: Vec<&str> = row.split(',').collect();
            // The title may contain a comma, so the amount is taken from the end and the title is
            // whatever sits between the line item and it.
            let amount = field.last()?.parse().ok()?;
            Some(Line {
                fiscal_year: field[2].parse().ok()?,
                kind: field[3].to_string(),
                line_item: field[8].to_string(),
                title: field[9..field.len() - 1].join(","),
                amount,
            })
        })
        .collect()
}

/// Enacted lines from the Catalog, for years the workbook series does not carry.
///
/// The Catalog restates six fiscal years per edition, so the same year appears in up to four of
/// them — but an enacted figure is printed by exactly one, the edition published when the biennium
/// was enacted. Later editions replace it with `Adj. Approp.`, which this never reads. So no
/// deduplication is needed here and none is done: if two editions ever claimed one year as
/// enacted, the count below would rise and the test on it would fail rather than silently double a
/// total.
#[must_use]
fn catalog_enacted() -> Vec<Line> {
    let covered: BTreeMap<u16, ()> = lines()
        .into_iter()
        .filter(|l| l.kind == "enacted")
        .map(|l| (l.fiscal_year, ()))
        .collect();
    CATALOG
        .lines()
        .skip(1)
        .filter(|row| !row.trim().is_empty())
        .filter_map(|row| {
            let field: Vec<&str> = row.split(',').collect();
            if field.len() < 7 || field[5] != "appropriation" {
                return None;
            }
            let fiscal_year: u16 = field[4].parse().ok()?;
            if covered.contains_key(&fiscal_year) {
                return None;
            }
            Some(Line {
                fiscal_year,
                kind: "enacted".to_string(),
                line_item: field[2].to_string(),
                title: field[3].to_string(),
                amount: field[6].parse().ok()?,
            })
        })
        .collect()
}

/// Every enacted line, from whichever of the two sources answers for its year.
#[must_use]
pub fn enacted_lines() -> Vec<Line> {
    let mut out: Vec<Line> = lines()
        .into_iter()
        .filter(|l| l.kind == "enacted")
        .collect();
    out.extend(catalog_enacted());
    out
}

/// One year of the department's appropriation, nominal and real.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Year {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// What was appropriated, in that year's dollars.
    pub nominal: f64,
    /// The same, in the base year's dollars. `None` where the index cannot reach the year.
    pub real: Option<f64>,
    /// How many line items the total is over.
    pub items: usize,
}

/// The department's enacted appropriation by year, restated into `base` dollars.
///
/// Excludes [`TAX_REIMBURSEMENT`]. Returns only the years an enacted figure exists for, which is
/// FY2002 onward and now without a gap. FY2006-07 and FY2012-13 come from the Catalog and every
/// other year from the workbooks — see the module note on which source answers for a year.
#[must_use]
pub fn enacted_history(base: FiscalYear) -> Vec<Year> {
    let cpi = CpiSeries::cpi_u_june();
    let mut totals: BTreeMap<u16, (f64, usize)> = BTreeMap::new();
    for line in enacted_lines() {
        if is_tax_reimbursement(&line.line_item, line.fiscal_year) {
            continue;
        }
        let entry = totals.entry(line.fiscal_year).or_insert((0.0, 0));
        entry.0 += line.amount;
        entry.1 += 1;
    }
    totals
        .into_iter()
        .map(|(fiscal_year, (nominal, items))| Year {
            fiscal_year,
            nominal,
            real: cpi
                .convert(nominal, FiscalYear(fiscal_year), base)
                .ok()
                .map(|deflated| deflated.value),
            items,
        })
        .collect()
}

/// One appropriation line's enacted history, restated into `base` dollars.
#[must_use]
pub fn line_history(line_item: &str, base: FiscalYear) -> Vec<Year> {
    let cpi = CpiSeries::cpi_u_june();
    let mut totals: BTreeMap<u16, f64> = BTreeMap::new();
    for line in lines() {
        if line.kind == "enacted" && line.line_item == line_item {
            *totals.entry(line.fiscal_year).or_default() += line.amount;
        }
    }
    totals
        .into_iter()
        .map(|(fiscal_year, nominal)| Year {
            fiscal_year,
            nominal,
            real: cpi
                .convert(nominal, FiscalYear(fiscal_year), base)
                .ok()
                .map(|deflated| deflated.value),
            items: 1,
        })
        .collect()
}

/// What the tax reimbursement lines carry, which the department's total excludes.
#[must_use]
pub fn reimbursements(base: FiscalYear) -> Vec<Year> {
    let cpi = CpiSeries::cpi_u_june();
    let mut totals: BTreeMap<u16, (f64, usize)> = BTreeMap::new();
    for line in lines() {
        if line.kind == "enacted" && is_tax_reimbursement(&line.line_item, line.fiscal_year) {
            let entry = totals.entry(line.fiscal_year).or_insert((0.0, 0));
            entry.0 += line.amount;
            entry.1 += 1;
        }
    }
    totals
        .into_iter()
        .map(|(fiscal_year, (nominal, items))| Year {
            fiscal_year,
            nominal,
            real: cpi
                .convert(nominal, FiscalYear(fiscal_year), base)
                .ok()
                .map(|deflated| deflated.value),
            items,
        })
        .collect()
}

/// Growth across a series of years, nominal and real, as fractions.
///
/// Returns `None` if either endpoint is missing a real figure — a growth rate computed from one
/// deflated endpoint and one nominal one is not a number anybody should be handed.
#[must_use]
pub fn growth(history: &[Year]) -> Option<(f64, f64, Confidence)> {
    let first = history.first()?;
    let last = history.last()?;
    if first.nominal <= 0.0 {
        return None;
    }
    let (first_real, last_real) = (first.real?, last.real?);
    if first_real <= 0.0 {
        return None;
    }
    Some((
        last.nominal / first.nominal - 1.0,
        last_real / first_real - 1.0,
        // Both endpoints ride the same index, and every point in it is verified against the
        // Bureau's own file — see `deflator::CpiSeries::cpi_u_june`.
        Confidence::Verified,
    ))
}

/// How one appropriation line moved across a window, in constant dollars.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// The line item.
    pub line_item: String,
    /// Its title in the later year.
    pub title: String,
    /// Real amount at the start of the window, in the base year's dollars.
    pub from: f64,
    /// And at the end.
    pub to: f64,
}

impl Change {
    /// The change in constant dollars — what the line gained or gave up.
    #[must_use]
    pub fn shift(&self) -> f64 {
        self.to - self.from
    }
}

/// Every line present as an enacted appropriation in both `from` and `to`, restated into `base`.
///
/// Lines that appear in only one of the two years are omitted rather than treated as a change
/// from or to zero. A line that did not exist yet and one that was abolished are different facts
/// from one that shrank, and a table ranking them together would put every creation and closure
/// above every real movement.
#[must_use]
pub fn changes(base: FiscalYear, from: u16, to: u16) -> Vec<Change> {
    let cpi = CpiSeries::cpi_u_june();
    let mut start: BTreeMap<String, f64> = BTreeMap::new();
    let mut end: BTreeMap<String, (f64, String)> = BTreeMap::new();
    for line in lines() {
        if line.kind != "enacted" || is_tax_reimbursement(&line.line_item, line.fiscal_year) {
            continue;
        }
        let Ok(real) = cpi.convert(line.amount, FiscalYear(line.fiscal_year), base) else {
            continue;
        };
        if line.fiscal_year == from {
            *start.entry(line.line_item.clone()).or_default() += real.value;
        } else if line.fiscal_year == to {
            let slot = end
                .entry(line.line_item.clone())
                .or_insert((0.0, line.title.clone()));
            slot.0 += real.value;
        }
    }
    start
        .into_iter()
        .filter_map(|(line_item, from_real)| {
            let (to_real, title) = end.get(&line_item)?;
            Some(Change {
                line_item,
                title: title.clone(),
                from: from_real,
                to: *to_real,
            })
        })
        .collect()
}
