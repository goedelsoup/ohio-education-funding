//! What moved in the panel's state column at FY2016, measured rather than characterised.
//!
//! # The question, and the guess that has to be withdrawn
//!
//! `education-agency/toledo-city` records that Toledo's state revenue per pupil falls about 30%
//! at FY2016 and never recovers in real terms, that 27 districts fall more than a fifth on a
//! sustained measure while the median district *rises*, and that the tail is "mostly small
//! districts with large industrial tax bases — which is the profile of a district losing tangible
//! personal property revenue". It marked the cause open and pointed at
//! `revenue-stream/tpp-replacement-payments`.
//!
//! **That characterisation was five district names and it does not survive measurement.** Against
//! Table SD-1's own tax-base composition the step has no relationship to industrial property at
//! all: `r = +0.09` on industrial share, `+0.04` on industrial, mineral and public-utility
//! together, and the 27 deep fallers carry a *lower* business-property share than everyone else
//! — 0.1412 against 0.1517. The quintile gradient on business property is flat. See
//! [`against`] and [`quintiles_by`].
//!
//! # What it does track is total valuation, and that has a name
//!
//! The step's strongest correlate is the **log of a district's total assessed value**, at
//! `r = −0.342`, and the gradient is monotone: the smallest-valuation fifth of districts gains
//! 24.1% and the largest gains 4.2%. In a model carrying total valuation, valuation per pupil and
//! the disadvantaged share together, total valuation is the only one that matters much —
//! standardised −0.291 at t = −7.09, against −0.112 for wealth per pupil and a disadvantage term
//! that does not clear two.
//!
//! Total valuation is not a natural quantity to key a formula on, and one Ohio formula keys on it.
//! FY2016 is H.B. 64's first year, and LSC's greenbook describes its new component in exactly
//! these terms: capacity aid "targets funding to **smaller districts with relatively low total
//! property valuation**", is "based on **the amount a district can raise with one mill**", and is
//! "provided to districts that raise less than the median amount", on a sliding scale. A measure
//! of total valuation is what one mill's yield *is*.
//!
//! So the median district rising 9.6% is not a puzzle beside the tail; it is the same event. The
//! panel is showing capacity aid arriving.
//!
//! # And the tail is the other half, which capacity aid cannot explain
//!
//! Capacity aid adds money. It cannot remove 30% from anyone, and among the fourteen largest
//! districts the split is not by size but by poverty:
//!
//! | | FY2015 → FY2016 |
//! |---|--:|
//! | Columbus, Cleveland, Toledo, Cincinnati, Dayton | −21.8% to −38.8% |
//! | Olentangy, Lakota, Hilliard, Dublin, Mason | −0.3% to −6.0% |
//!
//! Wealthy suburbs of the same size barely move. And it is not the FY2015 bump doing the work:
//! measured from FY2013 instead, and the statewide figure rises 5.4% while Cleveland falls 33.0%,
//! Columbus 27.8%, Toledo 21.9% and Cincinnati 18.2% — and the suburbs gain 4% to 15%.
//!
//! **It is also not a transfer to community schools**, which is the reading the deduct mechanism
//! invites. Their state revenue is flat across the break — $0.952bn in FY2015 against $0.938bn in
//! FY2016 — and districts and community schools *summed* move together, so nothing moved between
//! them. See [`by_sector`].
//!
//! What removed a fifth to a third of the state's contribution to Ohio's high-poverty urban
//! districts between FY2013 and FY2016 is not established here. It is not the tangible personal
//! property phase-out, it is not capacity aid, and it is not the community-school deduction in
//! aggregate.

use std::collections::BTreeMap;

use crate::{ohio_panel, profile, sd1, Regression};

/// The years averaged as the level before the move.
///
/// Three of them, so a single construction year cannot make a step on its own — and FY2014 is
/// absent from the archive for every agency.
pub const BEFORE: [u16; 3] = [2012, 2013, 2015];

/// And the years averaged after it.
pub const AFTER: [u16; 4] = [2016, 2017, 2018, 2019];

/// The tax year whose base composition the step is read against.
///
/// A decade after the event, which is the caution this carries: SD-1 opens at TY2021. An
/// industrial district in 2021 was an industrial district in 2013, so it is a usable proxy for
/// composition and not for level — and the result it produces is a flat zero rather than a weak
/// signal, which is the reading a proxy can support.
pub const BASE_YEAR: u16 = 2021;

/// One district's step and the measures it is read against.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Mean state revenue per pupil over [`AFTER`] against [`BEFORE`], as a fraction.
    pub step: f64,
    /// Fall membership in FY2016.
    pub pupils: f64,
    /// Total assessed value, TY2021 — the quantity one mill's yield is proportional to.
    pub total_valuation: f64,
    /// Assessed value per pupil, from the profile report.
    pub valuation_per_pupil: f64,
    /// Economically disadvantaged share, in points.
    pub disadvantaged: f64,
    /// Industrial value as a fraction of total.
    pub industrial_share: f64,
    /// Industrial, mineral and public-utility value together, as a fraction of total.
    pub business_share: f64,
}

/// Every district the step and the tax base can both be computed for.
#[must_use]
pub fn frame() -> Vec<District> {
    let mut base: BTreeMap<String, (f64, f64, f64)> = BTreeMap::new();
    for row in sd1::rows().iter().filter(|r| r.tax_year == BASE_YEAR) {
        let Some(total) = row.total_value.filter(|t| *t > 0.0) else {
            continue;
        };
        let share = |value: Option<f64>| value.unwrap_or(0.0) / total;
        base.insert(
            row.irn.clone(),
            (
                total,
                share(row.industrial_value),
                share(row.industrial_value)
                    + share(row.mineral_value)
                    + share(row.public_utility_value),
            ),
        );
    }
    let profiles: BTreeMap<String, profile::ProfileDistrict> = profile::districts()
        .into_iter()
        .map(|d| (d.irn.clone(), d))
        .collect();

    crate::exemplars::fy2016_step()
        .into_iter()
        .filter_map(|(irn, step, pupils)| {
            let (total, industrial, business) = *base.get(&irn)?;
            let district = profiles.get(&irn)?;
            Some(District {
                name: district.name.clone(),
                irn,
                step,
                pupils,
                total_valuation: total,
                valuation_per_pupil: district.valuation_per_pupil?,
                disadvantaged: district.economically_disadvantaged? * 100.0,
                industrial_share: industrial,
                business_share: business,
            })
        })
        .collect()
}

/// The correlation between the step and one measure over [`frame`].
///
/// # Panics
///
/// If the frame is empty or the measure has no variation, both of which mean the fixtures moved.
#[must_use]
pub fn against(pick: fn(&District) -> f64) -> f64 {
    let districts = frame();
    let steps: Vec<f64> = districts.iter().map(|d| d.step).collect();
    let measure: Vec<f64> = districts.iter().map(pick).collect();
    crate::wealth_neutrality(&measure, &steps)
        .expect("the frame pairs")
        .correlation
}

/// Mean step within each fifth of the districts, ordered by `pick` ascending.
///
/// A correlation says whether a relationship exists and a monotone gradient says it is not two
/// tails pulling against each other.
///
/// # Panics
///
/// If the frame holds fewer than five districts.
#[must_use]
pub fn quintiles_by(pick: fn(&District) -> f64) -> [f64; 5] {
    let mut districts = frame();
    districts.sort_by(|a, b| pick(a).total_cmp(&pick(b)));
    let size = districts.len() / 5;
    assert!(size > 0, "the frame is too small to quintile");
    let mut out = [0.0; 5];
    for (index, slot) in out.iter_mut().enumerate() {
        let end = if index == 4 {
            districts.len()
        } else {
            (index + 1) * size
        };
        let slice = &districts[index * size..end];
        *slot = slice.iter().map(|d| d.step).sum::<f64>() / slice.len() as f64;
    }
    out
}

/// The step on log total valuation, valuation per pupil and disadvantage together.
///
/// `standardized[0]` is total valuation, which is the term capacity aid keys on.
///
/// # Panics
///
/// If the frame is degenerate or collinear.
#[must_use]
pub fn model() -> Regression {
    let districts = frame();
    let steps: Vec<f64> = districts.iter().map(|d| d.step).collect();
    let columns = vec![
        districts.iter().map(|d| d.total_valuation.ln()).collect(),
        districts
            .iter()
            .map(|d| d.valuation_per_pupil / 1000.0)
            .collect(),
        districts.iter().map(|d| d.disadvantaged).collect(),
    ];
    crate::least_squares(&columns, &steps).expect("the frame fits")
}

/// State revenue and enrolment by year, split into districts and everything else.
///
/// Returned as `(district_revenue, community_revenue, community_enrolment)`. The non-comparable
/// half is community schools, joint vocational districts and service centres — the fixture carries
/// all of Ohio's, which is what makes this sum meaningful for Ohio and for no other state.
#[must_use]
pub fn by_sector() -> BTreeMap<u16, (f64, f64, f64)> {
    let mut out: BTreeMap<u16, (f64, f64, f64)> = BTreeMap::new();
    for row in ohio_panel::panel() {
        let entry = out.entry(row.fiscal_year).or_default();
        if row.comparable {
            entry.0 += row.state_revenue;
        } else {
            entry.1 += row.state_revenue;
            entry.2 += row.enrollment;
        }
    }
    out
}

/// A district's state revenue per pupil in one fiscal year.
///
/// `None` where the panel does not carry it that year.
#[must_use]
pub fn state_per_pupil(irn: &str, year: u16) -> Option<f64> {
    ohio_panel::panel()
        .into_iter()
        .find(|r| r.irn == irn && r.fiscal_year == year && r.enrollment > 0.0)
        .map(|r| r.state_revenue / r.enrollment)
}

/// The `n` largest districts by FY2016 enrolment, largest first.
///
/// The comparison the tail turns on: at the same size, the high-poverty districts fall and the
/// wealthy suburbs do not.
#[must_use]
pub fn largest(n: usize) -> Vec<District> {
    let mut districts = frame();
    districts.sort_by(|a, b| b.pupils.total_cmp(&a.pupils));
    districts.truncate(n);
    districts
}
