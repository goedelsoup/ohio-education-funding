//! Ohio publishes assessed valuation per pupil twice, and the two do not agree.
//!
//! # The identity, and the divergence it isolates
//!
//! The Department of Taxation's Table SD-1 and the Department of Education's District Profile
//! Report carry **the same numerator**. Multiply the profile's valuation per pupil by its enrolled
//! ADM and SD-1's total taxable value comes back — exactly, on all 606 districts that join, for
//! the tax year the profile's valuation is drawn from. The two arms of the state value the same
//! property at the same dollar.
//!
//! They divide it by different children. Taxation counts the pupils **resident** in the district;
//! Education counts the ones it **teaches**. The difference is community school, scholarship and
//! open-enrolment-out participation, so it is widest in exactly the districts where this metric
//! does the most work in the aid formula, and it runs the same way every time: the urban districts
//! with the largest scholarship populations look **poorer** on Taxation's count and **wealthier**
//! on Education's.
//!
//! Because the numerators are identical, the entire gap is the denominator. That is what makes
//! this measurable rather than merely arguable, and it is why everything here is computed on
//! [`TAX_YEAR`] — the one year in which the identity holds against the profile this corpus has.
//!
//! # What it costs to get the year wrong
//!
//! `metric/assessed-valuation-per-pupil` published this divergence as a three-row table, a count of
//! districts agreeing within 2%, and a pair of statewide medians, labelled `[verified — TY2023]`.
//! The table has one TY2024 row and two TY2023 rows; the count and the medians reproduce only on
//! TY2024; the numerator identity holds only on TY2023. Nothing was fabricated and every figure is
//! somewhere in the fixture — they are simply not all from the same year.
//!
//! That is the trap `dot-sd1-school-district-taxes` records as having "caught four callers": the
//! fixture gained two tax years, and anything that reads it without naming one reads whichever the
//! iteration order hands back. This module names it in a constant and every figure it exports is
//! computed from that constant.
//!
//! # And the type is the other half of the fix
//!
//! [`local_capacity::PupilCount`] travels with a per-pupil valuation and
//! [`local_capacity::ValuationPerPupil::ratio_to`] refuses a comparison across bases. A ratio that
//! is right for most of the state and wrong by a factor of two for Youngstown is the failure this
//! metric has already had once.

use std::collections::BTreeMap;

use local_capacity::{PupilCount, ValuationPerPupil};

/// The tax year every figure here is computed on.
///
/// The profile report this corpus holds carries FY2023 assessed valuation, and SD-1's TY2023 rows
/// are the ones whose total taxable value it reproduces exactly. Any other year compares two
/// different valuations as well as two different pupil counts, which is one confound too many for
/// the divergence to be attributable.
pub const TAX_YEAR: u16 = 2023;

/// How close two per-pupil figures must be to count as agreeing.
pub const AGREEMENT: f64 = 0.02;

/// One district's assessed valuation per pupil, on both of the counts Ohio divides it by.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's name in Table SD-1.
    pub name: String,
    /// Enrolled ADM — the children the district teaches.
    pub enrolled: f64,
    /// SD-1's ADM — the children resident in the district.
    pub resident: f64,
    /// Total taxable value, from SD-1.
    pub valuation: f64,
}

impl District {
    /// Valuation per pupil on the Department of Education's count.
    ///
    /// # Panics
    ///
    /// Never in practice: the frame admits only districts with both counts positive.
    #[must_use]
    pub fn on_enrolled(&self) -> ValuationPerPupil {
        local_capacity::valuation_per_pupil(self.valuation, self.enrolled, PupilCount::Enrolled)
            .expect("the frame admits only positive counts")
    }

    /// And on the Department of Taxation's.
    ///
    /// # Panics
    ///
    /// Never in practice, for the same reason.
    #[must_use]
    pub fn on_resident(&self) -> ValuationPerPupil {
        local_capacity::valuation_per_pupil(self.valuation, self.resident, PupilCount::Resident)
            .expect("the frame admits only positive counts")
    }

    /// Resident pupils over enrolled — how many children the district is charged for per child it
    /// teaches.
    #[must_use]
    pub fn denominator_ratio(&self) -> f64 {
        self.resident / self.enrolled
    }

    /// Whether the two published figures agree within [`AGREEMENT`].
    #[must_use]
    pub fn agrees(&self) -> bool {
        let enrolled = self.on_enrolled().value;
        let resident = self.on_resident().value;
        (enrolled / resident - 1.0).abs() < AGREEMENT
    }
}

/// Every district Table SD-1 and the profile report can both name, on [`TAX_YEAR`].
#[must_use]
pub fn frame() -> Vec<District> {
    let enrolled: BTreeMap<String, f64> = crate::profile::districts()
        .into_iter()
        .filter_map(|district| Some((district.irn, district.enrolled_adm?)))
        .collect();

    crate::sd1::rows()
        .iter()
        .filter(|row| row.tax_year == TAX_YEAR)
        .filter_map(|row| {
            let teaches = *enrolled.get(&row.irn)?;
            let (resident, valuation) = (row.adm?, row.total_value?);
            (teaches > 0.0 && resident > 0.0 && valuation > 0.0).then(|| District {
                irn: row.irn.clone(),
                name: row.name.clone(),
                enrolled: teaches,
                resident,
                valuation,
            })
        })
        .collect()
}

/// How exactly the two agencies' numerators agree.
///
/// Returned as `(districts within a tenth of a per cent, districts compared)`. This is the claim
/// the whole module rests on: if the numerators were not identical the denominator could not be
/// blamed for the gap.
#[must_use]
pub fn numerators_agree() -> (usize, usize) {
    let frame = frame();
    let exact = frame
        .iter()
        .filter(|district| {
            let implied = district.on_enrolled().value * district.enrolled;
            (implied / district.valuation - 1.0).abs() < 0.001
        })
        .count();
    (exact, frame.len())
}

/// Districts whose two published figures agree within [`AGREEMENT`], and how many were compared.
#[must_use]
pub fn agreement() -> (usize, usize) {
    let frame = frame();
    (frame.iter().filter(|d| d.agrees()).count(), frame.len())
}

/// The statewide medians of the two figures, as `(enrolled, resident)`.
///
/// Close together, and that is what makes the divergence dangerous rather than obvious: a page
/// comparing one table's district figure against the other's median looks right for most of the
/// state.
///
/// # Panics
///
/// If the frame is empty, which means one of the two fixtures stopped joining.
#[must_use]
pub fn medians() -> (f64, f64) {
    let frame = frame();
    assert!(!frame.is_empty(), "the frame is empty");
    let median = |mut values: Vec<f64>| {
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    (
        median(frame.iter().map(|d| d.on_enrolled().value).collect()),
        median(frame.iter().map(|d| d.on_resident().value).collect()),
    )
}

/// One district by IRN, for a caller that wants a named case.
#[must_use]
pub fn district(irn: &str) -> Option<District> {
    frame().into_iter().find(|d| d.irn == irn)
}
