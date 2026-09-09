//! Table SD-1 — the Department of Taxation's school district property tax abstract.
//!
//! # Why this is the second half of every millage claim
//!
//! `millage` is written from the statute: R.C. 319.301 says reduction factors hold a levy's
//! dollar yield roughly constant as valuation rises, and R.C. 319.301(D) says they may not carry
//! a rate below twenty mills. Nothing in that crate proves the arithmetic describes Ohio. This
//! table does: it carries, per district per tax year, every quantity `millage` takes and every
//! quantity it returns, so each claim becomes a statement that survives four tax years of
//! published data or does not.
//!
//! It is also the valuation series `regime-diff`'s charge-off work reads, and the taxes-charged
//! series the property-tax-share replication divides by district spending.
//!
//! # Charged is not received, and class is not class
//!
//! **Taxes charged for a tax year is not money received in a fiscal year.** The gap is largest
//! exactly where a levy has just passed, which is where a comparison is most likely to be drawn.
//!
//! **Class I is residential and agricultural; Class II is everything else.** They carry separate
//! values, separate charges and separate effective rates, and the twenty-mill floor is a Class I
//! rule. A figure that mixes them is not a rate on anything.
//!
//! # The JVSD column is a superset, and a membership list
//!
//! [`TaxRow::real_property_taxes_charged`] excludes joint vocational district operating levies
//! and [`TaxRow::real_property_taxes_charged_with_jvsd`] includes them. The JVSD levy is charged
//! to the same parcels; whether it belongs in "what this district's taxpayers pay for schools"
//! depends on the question, and both are carried so a caller has to answer it.
//!
//! The difference between them is also the one thing the abstract never states outright: which
//! joint vocational district a school district belongs to. It does not name one, and it does not
//! have to — a district in none reports the same number twice, and a group of districts whose
//! levies fit one pair of class rates is one joint vocational district. See
//! [`joint_vocational_tax`] and [`joint_vocational_fit`].

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The committed abstract.
pub const FIXTURE: &str = include_str!("../fixtures/sd1-district-taxes.csv");

/// The header this reader was written against.
pub const EXPECTED_HEADER: &str = "irn,district,county,tax_year,agricultural_value,\
residential_value,class1_value,mineral_value,industrial_value,commercial_value,\
railroad_value,class2_value,real_property_value,public_utility_value,total_value,\
class1_taxes_charged,class2_taxes_charged,real_property_taxes_charged,\
real_property_taxes_charged_with_jvsd,public_utility_taxes_charged,class1_rate,class2_rate,\
real_property_millage,public_utility_millage,value_per_pupil,adm";

/// One district's property tax abstract for one tax year.
///
/// Values and charges are in dollars; rates are in mills. The absences are real: a district with
/// no railroad property reports none rather than zero, and the Department of Taxation wrote
/// `N/A` in the TY2023 workbook and `NA` in TY2024 for the same thing.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxRow {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// The county the abstract files the district under.
    pub county: String,
    /// The tax year. Not a fiscal year: collections lag it.
    pub tax_year: u16,
    /// Agricultural taxable value.
    pub agricultural_value: Option<f64>,
    /// Residential taxable value.
    pub residential_value: Option<f64>,
    /// Class I taxable value — residential and agricultural, the base the twenty-mill floor
    /// governs.
    pub class1_value: Option<f64>,
    /// Mineral taxable value.
    pub mineral_value: Option<f64>,
    /// Industrial taxable value.
    pub industrial_value: Option<f64>,
    /// Commercial taxable value.
    pub commercial_value: Option<f64>,
    /// Railroad taxable value.
    pub railroad_value: Option<f64>,
    /// Class II taxable value — everything that is not Class I.
    pub class2_value: Option<f64>,
    /// Real property taxable value, both classes.
    pub real_property_value: Option<f64>,
    /// Public utility tangible taxable value.
    pub public_utility_value: Option<f64>,
    /// All taxable value.
    pub total_value: Option<f64>,
    /// Class I taxes charged for current expenses.
    pub class1_taxes_charged: Option<f64>,
    /// Class II taxes charged for current expenses.
    pub class2_taxes_charged: Option<f64>,
    /// Real property taxes charged, **excluding** joint vocational district operating levies.
    pub real_property_taxes_charged: Option<f64>,
    /// The same, including them. See the module note.
    pub real_property_taxes_charged_with_jvsd: Option<f64>,
    /// Public utility taxes charged.
    pub public_utility_taxes_charged: Option<f64>,
    /// Effective Class I rate in mills — after reduction factors, not the voted rate.
    pub class1_rate: Option<f64>,
    /// Effective Class II rate in mills.
    pub class2_rate: Option<f64>,
    /// Effective real property millage across both classes.
    pub real_property_millage: Option<f64>,
    /// Effective public utility millage.
    pub public_utility_millage: Option<f64>,
    /// Taxable value per pupil.
    pub value_per_pupil: Option<f64>,
    /// Average daily membership, as the abstract carries it.
    pub adm: Option<f64>,
}

/// Every row of the abstract, in file order.
///
/// # Panics
///
/// If the fixture's header is not [`EXPECTED_HEADER`], or a row's width differs from it — both
/// by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn rows() -> Vec<TaxRow> {
    cached().clone()
}

/// The fixture, parsed once.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and
/// the file is compiled in, and a lookup helper that re-read it per call turned a scan
/// over districts into a quadratic one.
fn cached() -> &'static Vec<TaxRow> {
    static ROWS: OnceLock<Vec<TaxRow>> = OnceLock::new();
    ROWS.get_or_init(parse)
}

fn parse() -> Vec<TaxRow> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            Some(TaxRow {
                irn: row.str(0).to_string(),
                name: row.str(1).to_string(),
                county: row.str(2).to_string(),
                tax_year: row.str(3).parse().ok()?,
                agricultural_value: row.num(4),
                residential_value: row.num(5),
                class1_value: row.num(6),
                mineral_value: row.num(7),
                industrial_value: row.num(8),
                commercial_value: row.num(9),
                railroad_value: row.num(10),
                class2_value: row.num(11),
                real_property_value: row.num(12),
                public_utility_value: row.num(13),
                total_value: row.num(14),
                class1_taxes_charged: row.num(15),
                class2_taxes_charged: row.num(16),
                real_property_taxes_charged: row.num(17),
                real_property_taxes_charged_with_jvsd: row.num(18),
                public_utility_taxes_charged: row.num(19),
                class1_rate: row.num(20),
                class2_rate: row.num(21),
                real_property_millage: row.num(22),
                public_utility_millage: row.num(23),
                value_per_pupil: row.num(24),
                adm: row.num(25),
            })
        })
        .collect()
}

/// One district's rows, in tax-year order.
///
/// Ordered because the recursion that predicts a rate from its predecessor needs consecutive
/// pairs, and a reader that took `[first, last]` off an unsorted vector silently stopped
/// matching anything when the abstract grew from two tax years to four.
#[must_use]
pub fn by_district() -> BTreeMap<String, Vec<TaxRow>> {
    let mut out: BTreeMap<String, Vec<TaxRow>> = BTreeMap::new();
    for row in cached().iter().cloned() {
        out.entry(row.irn.clone()).or_default().push(row);
    }
    for years in out.values_mut() {
        years.sort_by_key(|row| row.tax_year);
    }
    out
}

/// One district's row for one tax year.
#[must_use]
pub fn at(irn: &str, tax_year: u16) -> Option<TaxRow> {
    cached()
        .iter()
        .find(|row| row.irn == irn && row.tax_year == tax_year)
        .cloned()
}

/// Every tax year the abstract covers, ascending.
#[must_use]
pub fn tax_years() -> Vec<u16> {
    let years: std::collections::BTreeSet<u16> = cached().iter().map(|row| row.tax_year).collect();
    years.into_iter().collect()
}

/// The joint vocational district operating tax charged against a district's real property.
///
/// The abstract never names a district's joint vocational district, and the corpus recorded the
/// membership lists as unpopulated. It does not have to name it. The difference between the two
/// taxes-charged columns *is* the joint vocational levy on that district's parcels, so a district
/// that is part of no joint vocational district reports the same number twice and this returns
/// zero. Membership is a column that was already here.
#[must_use]
pub fn joint_vocational_tax(row: &TaxRow) -> Option<f64> {
    let with = row.real_property_taxes_charged_with_jvsd?;
    let without = row.real_property_taxes_charged?;
    Some((with - without).max(0.0))
}

/// One joint vocational district's two class rates, fitted to its member districts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JointVocationalFit {
    /// The Class I rate in mills.
    pub class1_mills: f64,
    /// The Class II rate in mills.
    pub class2_mills: f64,
    /// The largest relative error the fit leaves on any district in the group.
    pub worst_residual: f64,
}

/// Fit one pair of class rates to a group of districts' joint vocational taxes.
///
/// # Why a fit settles co-membership
///
/// A joint vocational district levies one rate per class across the whole of its territory, so
/// every member district's joint vocational tax is `class1_mills * class1_value +
/// class2_mills * class2_value` for the *same* two rates. Two unknowns explaining eight districts
/// to the dollar is not something a coincidence does; a group drawn from two joint vocational
/// districts leaves residuals in the tens of per cent.
///
/// The converse does not hold. Two joint vocational districts both pinned at the two-mill floor
/// of R.C. 319.301(E)(3) have the same rates, so a clean fit at `(2.0, 2.0)` is evidence of
/// nothing. A fit is a membership proof only where the rates it recovers are away from the floor.
///
/// Returns `None` if a district in the group is missing a value or reports no joint vocational
/// tax, or if the two class values are collinear across the group.
#[must_use]
pub fn joint_vocational_fit(rows: &[&TaxRow]) -> Option<JointVocationalFit> {
    let mut obs = Vec::with_capacity(rows.len());
    for row in rows {
        let (Some(one), Some(two)) = (row.class1_value, row.class2_value) else {
            return None;
        };
        let tax = joint_vocational_tax(row)?;
        if tax <= 0.0 {
            return None;
        }
        obs.push((one, two, tax));
    }
    let (mut s11, mut s12, mut s22, mut t1, mut t2) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for &(one, two, tax) in &obs {
        s11 += one * one;
        s12 += one * two;
        s22 += two * two;
        t1 += one * tax;
        t2 += two * tax;
    }
    let det = s11 * s22 - s12 * s12;
    if det.abs() < 1.0 {
        return None;
    }
    let rate1 = (s22 * t1 - s12 * t2) / det;
    let rate2 = (s11 * t2 - s12 * t1) / det;
    let worst = obs
        .iter()
        .map(|&(one, two, tax)| ((rate1 * one + rate2 * two) - tax).abs() / tax)
        .fold(0.0_f64, f64::max);
    Some(JointVocationalFit {
        class1_mills: rate1 * 1000.0,
        class2_mills: rate2 * 1000.0,
        worst_residual: worst,
    })
}

/// The districts of one county for one tax year, in file order.
#[must_use]
pub fn county(name: &str, tax_year: u16) -> Vec<&'static TaxRow> {
    cached()
        .iter()
        .filter(|row| row.tax_year == tax_year && row.county.eq_ignore_ascii_case(name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_abstract_carries_four_tax_years_of_districts() {
        let years = tax_years();
        assert_eq!(years.len(), 4, "tax years in the abstract: {years:?}");
        for year in years {
            let n = rows().iter().filter(|r| r.tax_year == year).count();
            assert!(n >= 600, "TY{year} carries {n} districts");
        }
    }

    /// The class values partition real property value, which is what makes a class rate a rate
    /// on something.
    #[test]
    fn the_two_classes_partition_real_property_value() {
        for row in rows() {
            let (Some(one), Some(two), Some(real)) =
                (row.class1_value, row.class2_value, row.real_property_value)
            else {
                continue;
            };
            assert!(
                (one + two - real).abs() < real * 0.001 + 1.0,
                "{} TY{}: {one} + {two} against {real}",
                row.irn,
                row.tax_year
            );
        }
    }

    /// The JVSD column is a superset of the one without it, never smaller.
    #[test]
    fn the_jvsd_column_never_falls_below_the_one_that_excludes_it() {
        for row in rows() {
            let (Some(without), Some(with)) = (
                row.real_property_taxes_charged,
                row.real_property_taxes_charged_with_jvsd,
            ) else {
                continue;
            };
            assert!(
                with >= without - 1.0,
                "{} TY{}: {with} with JVSD against {without} without",
                row.irn,
                row.tax_year
            );
        }
    }

    /// A district's years come back consecutive and ordered, which the rate recursion depends on.
    #[test]
    fn a_districts_years_are_ordered() {
        for (irn, years) in by_district() {
            let ordered: Vec<u16> = years.iter().map(|row| row.tax_year).collect();
            let mut sorted = ordered.clone();
            sorted.sort_unstable();
            assert_eq!(ordered, sorted, "{irn} is out of order");
        }
    }
}
