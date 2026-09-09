//! Ohio's forty-nine joint vocational school districts, and whether a candidate roster is whole.
//!
//! # Why this needs two crates
//!
//! A joint vocational district is not in any panel of districts. The federal directory types it
//! `4` — a *regional education service agency*, the code Ohio's educational service centres also
//! carry — so [`dispersion::lea_directory`] is where it is named, and the Auditor's district
//! finances are where it is measured. Neither crate alone can check the other, and the check is
//! the point.
//!
//! # The reconciliation
//!
//! Table SD-1 publishes each school district's taxes charged twice, with and without joint
//! vocational operating levies; the difference is the levy that district's parcels paid to its
//! joint vocational district. So a joint vocational district's members are exactly the districts
//! whose rows carry its levy — and a *candidate* roster can be tested by summing those levies and
//! comparing against what the district itself booked.
//!
//! Two things have to be right for that comparison to mean anything. The gross has to include the
//! rollback and homestead reimbursements the state pays, because R.C. 319.301(B)(4) defines taxes
//! charged as prior to those reductions and the Auditor books them on a separate line — leaving
//! them out biases every comparison by about an eighth. And the tax year has to be a quiet one:
//! in a revaluation year every roster misses by more, whether or not it is complete.

use std::collections::BTreeSet;

use dispersion::sd1::{self, TaxRow};
use edfund_core::FiscalYear;

/// The tax years the reconciliation is taken over — the two the abstract carries that are not
/// revaluation years across most of Ohio.
pub const QUIET_TAX_YEARS: [u16; 2] = [2021, 2022];

/// How far a complete roster lands from the district's own books, as a share.
///
/// Not a precision claim. Taxes charged for a tax year are collected across two fiscal years, so
/// even an exact roster misses by a few per cent. The band is put at eight because that is where
/// the widest gap in the distribution is: twelve rosters land between 0.7% and 7.3%, and the
/// thirteenth is at 14.3%. Nothing sits in between, which is what makes the count a reading of
/// the data rather than of the threshold — and
/// `the_tolerance_sits_in_the_widest_gap` below fails if that stops being true.
pub const TOLERANCE: f64 = 0.08;

/// One joint vocational district's candidate roster, tested.
#[derive(Debug, Clone, PartialEq)]
pub struct Reconciliation {
    /// The joint vocational district's IRN.
    pub irn: String,
    /// Its name as the directory files it.
    pub name: String,
    /// The county whose districts were taken as the candidate roster.
    pub county: String,
    /// How far the roster's levies fall from the district's own gross property tax, as a share,
    /// one per [`QUIET_TAX_YEARS`]. Positive means the roster charges more than the district
    /// booked, so it holds districts belonging to somebody else.
    pub misses: Vec<f64>,
}

impl Reconciliation {
    /// Whether every year lands inside [`TOLERANCE`].
    #[must_use]
    pub fn whole(&self) -> bool {
        self.misses.len() == QUIET_TAX_YEARS.len()
            && self.misses.iter().all(|m| m.abs() < TOLERANCE)
    }
}

/// The joint vocational levy charged against one district's real property, in dollars.
#[must_use]
pub fn levy(row: &TaxRow) -> f64 {
    sd1::joint_vocational_tax(row).unwrap_or(0.0)
}

/// What a joint vocational district booked in property tax for one fiscal year, gross of the
/// rollback and homestead reductions the state reimburses.
#[must_use]
pub fn gross_property_tax(irn: &str, fiscal_year: u16) -> Option<f64> {
    let panel = crate::finances::finances();
    let district = crate::finances::for_district(&panel, irn)?;
    let year = district.year(FiscalYear(fiscal_year))?;
    Some(year.property_tax? + year.property_tax_allocation?)
}

/// Every joint vocational district whose own name contains exactly one Ohio county, reconciled
/// against the whole of that county.
///
/// The pairing is deliberately weak — it is the only roster a name gives you — and thirteen of
/// the twenty-four fail it. What the failures carry is a direction: a county summing high holds
/// districts that belong to another joint vocational district, and one summing low has members
/// outside it.
#[must_use]
pub fn county_named() -> Vec<Reconciliation> {
    let counties: BTreeSet<String> = sd1::rows()
        .iter()
        .map(|row| row.county.to_ascii_uppercase())
        .collect();

    let mut out = Vec::new();
    for (irn, name) in dispersion::lea_directory::joint_vocational_districts() {
        let words: BTreeSet<String> = name
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|word| !word.is_empty())
            .map(str::to_ascii_uppercase)
            .collect();
        let named: Vec<&String> = counties.iter().filter(|c| words.contains(*c)).collect();
        let [county] = named.as_slice() else { continue };

        let misses: Vec<f64> = QUIET_TAX_YEARS
            .iter()
            .filter_map(|tax_year| {
                let charged: f64 = sd1::county(county, *tax_year).into_iter().map(levy).sum();
                let booked = gross_property_tax(&irn, tax_year + 1)?;
                (charged > 0.0 && booked > 0.0).then(|| charged / booked - 1.0)
            })
            .collect();
        out.push(Reconciliation {
            irn: irn.clone(),
            name,
            county: (*county).clone(),
            misses,
        });
    }
    out.sort_by(|a, b| a.county.cmp(&b.county));
    out
}

/// Of those, the ones whose county is the whole membership.
#[must_use]
pub fn reconciling() -> Vec<Reconciliation> {
    county_named()
        .into_iter()
        .filter(Reconciliation::whole)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The band separates two groups rather than splitting one.
    ///
    /// The twelve that reconcile run to 7.3% and the nearest failure is at 14.3%, so the cut sits
    /// in a gap twice the width of the band's own slack. If anything ever lands inside that gap
    /// the threshold has started doing the work the data was doing, and the count stops meaning
    /// what it says.
    #[test]
    fn the_tolerance_sits_in_the_widest_gap() {
        let mut worst_whole = 0.0_f64;
        let mut nearest_miss = f64::MAX;
        for row in county_named() {
            let Some(worst) = row.misses.iter().map(|m| m.abs()).reduce(f64::max) else {
                continue;
            };
            if row.whole() {
                worst_whole = worst_whole.max(worst);
            } else {
                nearest_miss = nearest_miss.min(worst);
            }
        }
        assert!(
            worst_whole < TOLERANCE,
            "worst reconciling miss {worst_whole}"
        );
        assert!(
            nearest_miss > TOLERANCE + 0.06,
            "nearest failure {nearest_miss} is not clear of the band"
        );
    }
}
