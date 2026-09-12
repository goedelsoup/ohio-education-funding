//! Who paid Ohio's community-school deduct, and how little of it was spread.
//!
//! # The question, and where it was recorded as unanswerable
//!
//! `education-agency/electronic-classroom-of-tomorrow` closes on this:
//!
//! > Which districts, and in what proportion, still needs the department's deduct file and is not
//! > established here.
//!
//! Part of it does not. [`crate::ohio_panel`] carries `V92` — what each district paid community
//! schools — for every comparable district in every year the survey reports, and
//! [`crate::survey_basis`] has already established which years that column can be read in. What
//! had not been done is reading it as a distribution rather than as a correction: `V92` entered
//! this repository to explain why Ohio's state column breaks at FY2016, was used for that, and
//! was never asked who was paying.
//!
//! What it cannot do is decompose a district's payment by receiving school, because the survey
//! asks for one number. So ECOT's own share stays open and this module bounds it instead — see
//! [`share_of_pool`], which is the statewide average a per-district apportionment would vary
//! around rather than a substitute for one.
//!
//! # The finding
//!
//! **604 of 615 districts paid, and ten of them paid 62.8% of it.** Both halves are true and the
//! first one on its own is misleading, which is the reason this module exists: the corpus says
//! the deduct was "spread across most districts in the state", and by count it was.
//!
//! Across the eleven reported years the deduct totals **$9.29 billion**. Columbus paid $1.459
//! billion of it, which is **35.3% of every state dollar Columbus was credited with** over the
//! same span. Cleveland, Toledo, Cincinnati and Dayton each ran between a quarter and 30%.
//!
//! # Two sides of one transaction, reported by different agencies
//!
//! The survey collects the deduct twice without meaning to. Districts report what they paid
//! community schools; community schools report what they received from the state. Under the
//! deduct mechanism those are the same dollars, and [`counterparty`] divides one by the other.
//!
//! It sits between **0.985 and 1.037 from FY2012 through FY2019** — two independent sets of
//! respondents describing the same money and landing within a few per cent. That is the strongest
//! corroboration this repository has that the column means what it is read as meaning.
//!
//! It also says where the column is weakest, and the ends are not the middle: **0.885 in FY2010**
//! and **1.077 in FY2021**. [`crate::survey_basis`] already excludes FY2009 because 108 districts
//! report a zero that cannot be true. FY2010 is inside its window and still 11.5% light against
//! its counterparty, so a level read off FY2010 is a floor rather than a measurement. Nothing here
//! narrows [`crate::survey_basis::DEDUCT_REPORTED`] — the years are usable — but a reader
//! comparing the first year of the window against the last should know that roughly a fifth of
//! the apparent growth across it is the column filling in.
//!
//! # What this module does not establish
//!
//! **That the deduct stopped in FY2022.** It looks as though it did: 942 of 968 agencies report
//! zero, with no blanks, in the year the Fair School Funding Plan moved community schools to
//! direct funding. But FY2009 has the same shape and there the zeros are demonstrably false —
//! Toledo and Dayton report nothing while carrying thousands of community-school pupils. A zero
//! and an unreported value are the same character in this column, so the FY2022 collapse is
//! *consistent with* the mechanism change and is not evidence of it, and
//! [`crate::survey_basis::DEDUCT_REPORTED`] ends at FY2021 for that reason. Confirming the change
//! needs a source that distinguishes the two, which this one does not.

use std::collections::{BTreeMap, BTreeSet};

use crate::lea_directory;
use crate::ohio_panel::{self, PanelRow};
use crate::survey_basis::{Basis, DEDUCT_REPORTED};

/// The CCD agency type Ohio's community schools are filed under.
///
/// Used here only to pick the receiving side out of the panel's non-comparable agencies, which it
/// does cleanly: type 7 is 99.9% of their state revenue in every reported year. The code carries a
/// succession problem of its own in the directory's early years — it is a catch-all bucket before
/// Ohio had community schools, and names regional data-processing co-operatives — but those
/// agencies report no revenue to this survey and so cannot reach anything below.
const CHARTER_AGENCY_TYPE: &str = "7";

/// Whether a year can be read at all.
fn reported(fiscal_year: u16) -> bool {
    (DEDUCT_REPORTED[0]..=DEDUCT_REPORTED[1]).contains(&fiscal_year)
}

/// What one district paid community schools in one year, against the state money it was credited
/// with before the payment left.
#[derive(Debug, Clone, PartialEq)]
pub struct Bearer {
    /// Ohio's district identifier.
    pub irn: String,
    /// `V92`, in dollars.
    pub paid: f64,
    /// State revenue on [`Basis::Gross`] — the formula amount before the deduct, which is the
    /// denominator that makes the payment legible as a share of anything.
    pub gross: f64,
}

impl Bearer {
    /// The share of its credited state money the district never kept.
    ///
    /// `None` where the district is credited with nothing, which no comparable district is.
    #[must_use]
    pub fn share(&self) -> Option<f64> {
        (self.gross > 0.0).then(|| self.paid / self.gross)
    }
}

/// One row's deduct and gross state revenue, where both can be had.
fn bearer(row: &PanelRow) -> Option<Bearer> {
    let paid = row.charter_payments?;
    Some(Bearer {
        irn: row.irn.clone(),
        paid,
        gross: Basis::Gross.of(row)?,
    })
}

/// Every comparable district's deduct in one year, largest payment first.
///
/// Empty outside [`DEDUCT_REPORTED`], rather than a series of zeros that would read as a finding.
#[must_use]
pub fn bearers(fiscal_year: u16) -> Vec<Bearer> {
    if !reported(fiscal_year) {
        return Vec::new();
    }
    let mut found: Vec<Bearer> = ohio_panel::panel()
        .iter()
        .filter(|row| row.fiscal_year == fiscal_year && row.comparable)
        .filter_map(bearer)
        .collect();
    found.sort_by(|a, b| b.paid.total_cmp(&a.paid).then_with(|| a.irn.cmp(&b.irn)));
    found
}

/// The same summed over every reported year, largest total first.
///
/// A district absent from a year contributes nothing to either side, so a district that closed
/// mid-window is compared on the years it existed for. The eleven years are not twelve: the
/// archive has no FY2014 for any state.
#[must_use]
pub fn cumulative() -> Vec<Bearer> {
    let mut totals: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for row in ohio_panel::panel() {
        if !reported(row.fiscal_year) || !row.comparable {
            continue;
        }
        if let Some(found) = bearer(&row) {
            let entry = totals.entry(found.irn).or_insert((0.0, 0.0));
            entry.0 += found.paid;
            entry.1 += found.gross;
        }
    }
    let mut found: Vec<Bearer> = totals
        .into_iter()
        .map(|(irn, (paid, gross))| Bearer { irn, paid, gross })
        .collect();
    found.sort_by(|a, b| b.paid.total_cmp(&a.paid).then_with(|| a.irn.cmp(&b.irn)));
    found
}

/// The statewide picture in one year.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Year {
    /// The survey year.
    pub fiscal_year: u16,
    /// Comparable districts the panel carries.
    pub districts: usize,
    /// How many of them report paying anything.
    pub payers: usize,
    /// What they paid between them, in dollars.
    pub paid: f64,
    /// What they were credited with before paying it, in dollars.
    pub gross: f64,
}

impl Year {
    /// The deduct as a share of what the state credited districts with.
    #[must_use]
    pub fn share_of_gross(&self) -> Option<f64> {
        (self.gross > 0.0).then(|| self.paid / self.gross)
    }

    /// The share of the year's deduct borne by districts that paid nothing at all.
    ///
    /// Zero by construction, and stated as a method so the complement reads as what it is: every
    /// dollar of this is borne by a payer, and [`Self::payers`] is how few of them there are.
    #[must_use]
    pub fn non_payers(&self) -> usize {
        self.districts.saturating_sub(self.payers)
    }
}

/// Every reported year of it.
#[must_use]
pub fn by_year() -> BTreeMap<u16, Year> {
    let mut years: BTreeMap<u16, Year> = BTreeMap::new();
    for row in ohio_panel::panel() {
        if !reported(row.fiscal_year) || !row.comparable {
            continue;
        }
        let Some(found) = bearer(&row) else { continue };
        let year = years.entry(row.fiscal_year).or_insert(Year {
            fiscal_year: row.fiscal_year,
            districts: 0,
            payers: 0,
            paid: 0.0,
            gross: 0.0,
        });
        year.districts += 1;
        if found.paid > 0.0 {
            year.payers += 1;
        }
        year.paid += found.paid;
        year.gross += found.gross;
    }
    years
}

/// The share of a sorted run of bearers borne by its largest `top`.
///
/// Takes the slice rather than a year so the same measure reads on [`cumulative`], and asserts
/// nothing about the order it is given — a caller passing an unsorted slice gets the share of the
/// first `top` of it, which is why both producers above sort.
///
/// `None` where nothing was paid.
#[must_use]
pub fn concentration(bearers: &[Bearer], top: usize) -> Option<f64> {
    let total: f64 = bearers.iter().map(|bearer| bearer.paid).sum();
    (total > 0.0).then(|| {
        bearers
            .iter()
            .take(top)
            .map(|bearer| bearer.paid)
            .sum::<f64>()
            / total
    })
}

/// The IRNs the directory files as community schools in one year.
fn receivers(fiscal_year: u16) -> BTreeSet<String> {
    lea_directory::panel()
        .into_iter()
        .filter(|agency| agency.opens == fiscal_year && agency.agency_type == CHARTER_AGENCY_TYPE)
        .map(|agency| agency.irn)
        .collect()
}

/// What community schools report receiving from the state, over what districts report paying them.
///
/// One if the survey's two sides describe the same transaction exactly. See the module docs for
/// what it actually reads, and for why that is the best evidence here that the column is sound.
///
/// `None` outside [`DEDUCT_REPORTED`], or where no district reports paying.
#[must_use]
pub fn counterparty(fiscal_year: u16) -> Option<f64> {
    if !reported(fiscal_year) {
        return None;
    }
    let panel = ohio_panel::panel();
    let receiving = receivers(fiscal_year);
    let received: f64 = panel
        .iter()
        .filter(|row| row.fiscal_year == fiscal_year && !row.comparable)
        .filter(|row| receiving.contains(&row.irn))
        .map(|row| row.state_revenue)
        .sum();
    let paid: f64 = bearers(fiscal_year).iter().map(|bearer| bearer.paid).sum();
    (paid > 0.0).then_some(received / paid)
}

/// One community school's state revenue as a share of the whole statewide deduct that year.
///
/// The number ECOT's open question needs a substitute for. It is not an apportionment: it says
/// what fraction of every district's deduct went to this school *on the statewide average*, and a
/// district whose residents chose it more often than average paid more of its own deduct to it.
/// Bounding it this way is what the survey supports; naming districts needs the department's file.
///
/// `None` outside [`DEDUCT_REPORTED`], or where the school is absent from the year.
#[must_use]
pub fn share_of_pool(irn: &str, fiscal_year: u16) -> Option<f64> {
    let received: f64 = ohio_panel::panel()
        .iter()
        .filter(|row| row.fiscal_year == fiscal_year && row.irn == irn)
        .map(|row| row.state_revenue)
        .sum();
    if received == 0.0 {
        return None;
    }
    let paid: f64 = bearers(fiscal_year).iter().map(|bearer| bearer.paid).sum();
    (paid > 0.0).then_some(received / paid)
}

/// One community school's draw as a share of the whole pool, across every year both are reported.
///
/// [`share_of_pool`] weighted by each year's pool rather than averaged across years, so a school
/// that grew is not given the same weight in a small year as in a large one. The years it does not
/// appear in are the pool's and not its: a school that closed mid-window is measured against the
/// pool while it existed.
///
/// Carries the same caveat as [`share_of_pool`] and for the same reason — it is a statewide
/// average, not an apportionment.
///
/// `None` where the school appears in no reported year.
#[must_use]
pub fn pool_share_across_window(irn: &str) -> Option<f64> {
    let panel = ohio_panel::panel();
    let mut drawn = 0.0;
    let mut pool = 0.0;
    for fiscal_year in DEDUCT_REPORTED[0]..=DEDUCT_REPORTED[1] {
        let received: f64 = panel
            .iter()
            .filter(|row| row.fiscal_year == fiscal_year && row.irn == irn)
            .map(|row| row.state_revenue)
            .sum();
        if received == 0.0 {
            continue;
        }
        let paid: f64 = panel
            .iter()
            .filter(|row| row.fiscal_year == fiscal_year && row.comparable)
            .filter_map(|row| row.charter_payments)
            .sum();
        drawn += received;
        pool += paid;
    }
    (pool > 0.0).then_some(drawn / pool)
}
