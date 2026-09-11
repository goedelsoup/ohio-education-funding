//! What moved in the panel's state column at FY2016, measured rather than characterised.
//!
//! # Two characterisations, both withdrawn, and one finding that survives both withdrawals
//!
//! `education-agency/toledo-city` recorded that the FY2016 tail was "mostly small districts with
//! large industrial tax bases — which is the profile of a district losing tangible personal
//! property revenue", and marked the cause open. This module refuted that, found the median
//! district's rise in H.B. 64's capacity aid, and left the tail open against three named
//! non-explanations.
//!
//! **The tail was never there.** The panel's state column is gross of the community-school
//! deduct through FY2015 and net of it from FY2016, because that is the year the Census Bureau
//! began removing the double count Ohio's deduct creates. Every measure below is therefore stated
//! on [`BASIS`], gross in both eras, and on that basis **thirteen of Ohio's fourteen largest
//! districts gained** across the step. Columbus goes from −29.22% to +14.93%, Dayton from −3.50%
//! to +32.14%. [`crate::survey_basis`] holds the evidence and the list of what it overturned.
//!
//! # The industrial reading stays refuted, and more clearly
//!
//! Against Table SD-1's own tax-base composition the step has no useful relationship to
//! industrial property on either basis: `r = +0.12` on industrial share here and `+0.02` on
//! industrial, mineral and public-utility together, both the wrong sign for a district losing
//! tangible personal property revenue, and the quintile gradient on business property spans six
//! points with no order to it. See [`against`] and [`quintiles_by`].
//!
//! # What the step does track, and what that no longer isolates
//!
//! The strongest correlate is still the **log of a district's total assessed value**, at
//! `r = −0.2691`, and the gradient is still monotone and steep at the bottom: the
//! smallest-valuation fifth of districts gains 26.79% and the largest 10.60%, with every fifth
//! gaining. Total valuation is not a natural quantity to key a formula on, and one Ohio formula
//! keys on it. FY2016 is H.B. 64's first year, and LSC's greenbook describes its new component in
//! exactly these terms: capacity aid "targets funding to **smaller districts with relatively low
//! total property valuation**", is "based on **the amount a district can raise with one mill**",
//! and is "provided to districts that raise less than the median amount", on a sliding scale. A
//! measure of total valuation is what one mill's yield *is*, and the discriminating test — total
//! valuation beating *wealth per pupil*, which nothing else in the formula prefers — survives
//! the correction at −0.199 against −0.120 standardised.
//!
//! **What does not survive is the claim that it was the only thing happening.** On the published
//! column a disadvantage term reached `t = +1.38` and this module recorded that poverty was doing
//! no work in the step. On one basis it reaches **+3.92**, standardised +0.161, third of three
//! terms that all clear two. That is what a year which raised capacity aid *and* the money that
//! follows need should look like, and the published version could not see it because the
//! adjustment had subtracted the largest deducts — which belong to the poorest districts — out of
//! the outcome.
//!
//! # And the community-school reading, which was right for the wrong reason
//!
//! This module ruled out a transfer to community schools by observing that their state revenue is
//! flat across the break — $0.952bn in FY2015 against $0.938bn in FY2016 — while districts fell
//! $850m, and concluded that "nothing moved between them, so the total itself moved". The first
//! half is correct and the conclusion is backwards. Nothing moved because nothing *had* to: the
//! money was already in the community schools' column and was being counted a second time in the
//! districts'. The total fell by the size of the double count. See [`by_sector`], which is kept
//! for that reason.

use std::collections::BTreeMap;

use crate::survey_basis::{Basis, ADJUSTED_FROM};
use crate::{ohio_panel, profile, sd1, Regression};

/// The basis every measure here is computed on.
///
/// Gross of the community-school deduct in both eras. The published column is gross before
/// FY2016 and net from FY2016, which is the whole reason this module had to be rewritten — see
/// [`crate::survey_basis`]. Gross rather than net because the question is what the *formula*
/// awarded a district, and the deduct happens after the formula has run.
pub const BASIS: Basis = Basis::Gross;

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
/// composition and not for level — and the result it produces is a near-zero of the wrong sign
/// rather than a weak signal in the expected direction, which is the reading a proxy can
/// support.
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

    crate::survey_basis::step(BASIS)
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

/// A district's state revenue per pupil in one fiscal year, on a chosen basis.
///
/// `None` where the panel does not carry the district that year, or where the year cannot be put
/// on the basis asked for. [`Basis::AsPublished`] is what the corpus's FY2016 figures were
/// computed on and is kept so a withdrawal can quote them; nothing new should read it across
/// FY2016.
#[must_use]
pub fn state_per_pupil(irn: &str, year: u16, basis: Basis) -> Option<f64> {
    ohio_panel::panel()
        .iter()
        .find(|r| r.irn == irn && r.fiscal_year == year && r.enrollment > 0.0)
        .and_then(|r| basis.per_pupil(r))
}

/// What the districts' state column lost across the break, against the deduct that explains it.
///
/// Returned as `(fall, deduct)` in dollars: the fall in comparable districts' state revenue from
/// FY2015 to FY2016 as published, and the community-school deduct those same districts reported
/// in FY2016. The two are one quantity seen from either side, which is what makes the fall a
/// change in what is counted rather than a change in what was paid.
#[must_use]
pub fn fall_against_deduct() -> (f64, f64) {
    let panel = ohio_panel::panel();
    let total = |year: u16| -> f64 {
        panel
            .iter()
            .filter(|r| r.comparable && r.fiscal_year == year)
            .map(|r| r.state_revenue)
            .sum()
    };
    let deduct: f64 = panel
        .iter()
        .filter(|r| r.comparable && r.fiscal_year == ADJUSTED_FROM)
        .filter_map(|r| r.charter_payments)
        .sum();
    (total(ADJUSTED_FROM) - total(2015), deduct)
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
