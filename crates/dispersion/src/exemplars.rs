//! A district's fifteen-year revenue history in constant dollars, and where it stands today.
//!
//! # The four things two nodes said they could not see
//!
//! `education-agency/perrysburg-exempted-village` closes on "Not yet held: valuation per pupil,
//! millage, guarantee status, and anything before FY2024." `education-agency/toledo-city` closes
//! the same way. The two are the corpus's worked pair — the case its denominator rule was
//! established on — and neither could say what either district taxes, what it is worth, or what
//! has happened to it.
//!
//! Three of the four are in [`crate::profile`] and the FY2027 model, and the fourth is in
//! [`crate::ohio_panel`], which runs from FY2009. Nothing had to be fetched.
//!
//! # What fifteen years say about the pair
//!
//! Both districts lost state aid in real terms, and only one of them could replace it.
//!
//! | FY2009 → FY2024, per pupil | Toledo | Perrysburg |
//! |---|--:|--:|
//! | state revenue, real | −34.3% | −22.8% |
//! | local revenue, real | −8.1% | **+22.3%** |
//! | enrolment | −20.6% | +20.0% |
//!
//! That is the equity claim stated as a history of two districts rather than as a coefficient:
//! the state's real contribution fell for both, and the wealthy district made it up out of a tax
//! base that grew while the poor district's shrank. Neither is at the twenty-mill floor and
//! neither is under-taxing itself — see [`standing`].
//!
//! # The break in Toledo's series, which is not a policy fact
//!
//! Toledo's state revenue per pupil in FY2024 dollars runs about $15,000 through FY2015 and about
//! $10,500 from FY2016, a **one-year drop of roughly 30% that never reverses**. It is not
//! Toledo's alone and it is not statewide: 27 districts holding 6.7% of the panel's pupils fall
//! more than 20% on a sustained measure, while the median district *rises* 9.6%. The tail is
//! dominated by small districts with large industrial tax bases — Green Local, Botkins, Ridgemont,
//! Otsego, Carey — which is the profile of a tangible personal property district, and
//! `revenue-stream/tpp-replacement-payments` records what happened to those payments as its own
//! open question. The cause is not established here and this module does not assert one.
//!
//! What *is* established is that the break does not disturb anything the corpus publishes off
//! this panel. [`crate::ohio_panel::equalization_by_year`] cuts by quartile and takes an
//! unweighted mean over about 152 districts, so the poorest quartile's state revenue per pupil
//! goes *up* across FY2016 — 7,331 to 7,495 — and the band `doctrine/equity` states holds
//! throughout. A quartile mean is one tiny district wide and also six enormous districts deep;
//! the same property that let a five-pupil island move it hides a 30% fall in Ohio's fourth
//! largest district.
//!
//! # Two cautions on the series itself
//!
//! **`state_revenue` is not foundation aid.** It is every dollar the state sent, which includes
//! the state share of a school construction project. `current_spending` is `TCURELSC` and excludes
//! capital outlay, so a district in a build year shows revenue far above spending — Carey
//! Exempted Village reports $21.0m of state revenue against $8.0m of current spending in FY2015.
//! A single year of one district's state revenue cannot be read as its aid.
//!
//! **FY2014 is absent** from the panel, as `metric/per-pupil-operating-expenditure` records. Every
//! change measured here is between years the panel holds.

use crate::{composition, ohio_panel, profile};
use edfund_core::FiscalYear;

use crate::survey_basis::{self, Basis};

/// Toledo City, IRN 044909 — the corpus's high-need exemplar.
pub const TOLEDO: &str = "044909";

/// Perrysburg Exempted Village, IRN 045583 — the other half of the pair.
pub const PERRYSBURG: &str = "045583";

/// Cleveland Municipal, IRN 043786 — the corpus's third traditional exemplar.
pub const CLEVELAND: &str = "043786";

/// Electronic Classroom of Tomorrow, IRN 133413 — closed in January 2018.
///
/// A community school and therefore never `comparable`, which is why it is invisible to every
/// district measure in this crate and present in the panel all the same. The panel carries it for
/// FY2009 through FY2017 and stops, because the school did.
pub const ECOT: &str = "133413";

/// The first and last fiscal years the panel holds, which is the span every change here spans.
pub const SPAN: [u16; 2] = [2009, 2024];

/// One district-year of the survey, per pupil.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Year {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// Fall membership on the Bureau's count.
    pub enrolment: f64,
    /// State revenue per pupil **as the survey publishes it** — every dollar the state sent,
    /// capital included.
    ///
    /// Not one quantity across FY2016: gross of the community-school deduct before it and net of
    /// it afterwards. Read [`Self::state_on`] instead of this field for anything that spans that
    /// year, and see [`crate::survey_basis`] for what reading it directly cost the corpus.
    pub state: f64,
    /// What the district paid community schools, per pupil, where the survey reports it.
    pub deduct: Option<f64>,
    /// Local revenue per pupil.
    pub local: f64,
    /// Federal revenue per pupil.
    pub federal: f64,
    /// Current spending per pupil, which excludes capital outlay. `None` where unreported.
    pub spending: Option<f64>,
}

impl Year {
    /// State revenue per pupil on one basis, `None` where this year cannot be put on it.
    ///
    /// [`Basis::Net`] is the money the district kept and reaches FY2024; [`Basis::Gross`] is the
    /// formula amount before the deduct and stops at FY2021. Either is a series and
    /// [`Self::state`] is not.
    #[must_use]
    pub fn state_on(&self, basis: Basis) -> Option<f64> {
        let reported = || {
            (survey_basis::DEDUCT_REPORTED[0]..=survey_basis::DEDUCT_REPORTED[1])
                .contains(&self.fiscal_year)
                .then_some(self.deduct)
                .flatten()
        };
        match basis {
            Basis::AsPublished => Some(self.state),
            Basis::Net if self.fiscal_year >= survey_basis::ADJUSTED_FROM => Some(self.state),
            Basis::Net => Some(self.state - reported()?),
            Basis::Gross if self.fiscal_year < survey_basis::ADJUSTED_FROM => Some(self.state),
            Basis::Gross => Some(self.state + reported()?),
        }
    }

    /// Total revenue over current spending, both per pupil.
    ///
    /// Above one for almost every district in almost every year, because the numerator carries
    /// capital money and the denominator does not. Far above one is a construction year.
    #[must_use]
    pub fn revenue_over_spending(&self) -> Option<f64> {
        let spending = self.spending.filter(|s| *s > 0.0)?;
        Some((self.state + self.local + self.federal) / spending)
    }
}

/// One district's whole run of the panel, oldest first.
///
/// Empty for an IRN the panel does not carry.
#[must_use]
pub fn history(irn: &str) -> Vec<Year> {
    let mut years: Vec<Year> = ohio_panel::panel()
        .into_iter()
        .filter(|row| row.irn == irn && row.enrollment > 0.0)
        .map(|row| Year {
            fiscal_year: row.fiscal_year,
            enrolment: row.enrollment,
            state: row.state_revenue / row.enrollment,
            deduct: row.charter_payments.map(|paid| paid / row.enrollment),
            local: row.local_revenue / row.enrollment,
            federal: row.federal_revenue / row.enrollment,
            spending: row.current_spending.map(|s| s / row.enrollment),
        })
        .collect();
    years.sort_by_key(|year| year.fiscal_year);
    years
}

/// A change across the panel's span, both ways.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Change {
    /// The first year's value.
    pub from: f64,
    /// The last year's value.
    pub to: f64,
    /// Change as published, as a fraction.
    pub nominal: f64,
    /// Change after removing price growth, as a fraction.
    ///
    /// The one that answers the question anyone asking it means. Prices rose 45.7% across this
    /// span, so a nominal series that looks flat has lost a third of its purchasing power.
    pub real: f64,
}

/// How one per-pupil series moved across [`SPAN`], nominally and in real terms.
///
/// # Panics
///
/// If the panel does not carry `irn` in both of [`SPAN`]'s years, or if the deflator has no index
/// for one of them — both of which mean the fixtures moved under this module. Use
/// [`change_between`] for an agency whose run is shorter than the panel's, which for a school
/// that closed is every one of them.
#[must_use]
pub fn change(irn: &str, pick: fn(&Year) -> f64) -> Change {
    change_between(irn, SPAN[0], SPAN[1], pick).expect("the panel carries this district throughout")
}

/// How a district's state revenue per pupil moved between two years, on one basis.
///
/// The correction the corpus's fifteen-year state figures needed and did not have. `None` where
/// either year is outside the basis — which for [`Basis::Net`] means FY2009, so a state-column
/// change that starts at [`SPAN`]'s first year cannot be corrected at all and has to be restated
/// from FY2010.
#[must_use]
pub fn state_change(irn: &str, from_year: u16, to_year: u16, basis: Basis) -> Option<Change> {
    let years = history(irn);
    let at = |year: u16| {
        years
            .iter()
            .find(|y| y.fiscal_year == year)
            .and_then(|y| y.state_on(basis))
    };
    let (from, to) = (at(from_year)?, at(to_year)?);
    if from == 0.0 {
        return None;
    }
    let real = deflator::CpiSeries::cpi_u_june()
        .real_growth(from, FiscalYear(from_year), to, FiscalYear(to_year))
        .ok()?;
    Some(Change {
        from,
        to,
        nominal: to / from - 1.0,
        real: real.value,
    })
}

/// The same between any two years the panel holds for `irn`.
///
/// `None` where either year is missing for that agency, or where the first value is zero — which
/// is the case for a school that opened or closed inside the span.
#[must_use]
pub fn change_between(
    irn: &str,
    from_year: u16,
    to_year: u16,
    pick: fn(&Year) -> f64,
) -> Option<Change> {
    let years = history(irn);
    let first = years.iter().find(|y| y.fiscal_year == from_year)?;
    let last = years.iter().find(|y| y.fiscal_year == to_year)?;
    let (from, to) = (pick(first), pick(last));
    if from == 0.0 {
        return None;
    }
    let cpi = deflator::CpiSeries::cpi_u_june();
    let real = cpi
        .real_growth(from, FiscalYear(from_year), to, FiscalYear(to_year))
        .ok()?;
    Some(Change {
        from,
        to,
        nominal: to / from - 1.0,
        real: real.value,
    })
}

/// The year one series peaks, and its value there.
///
/// `None` for an agency the panel does not carry. Ties go to the earlier year.
#[must_use]
pub fn peak(irn: &str, pick: fn(&Year) -> f64) -> Option<(u16, f64)> {
    history(irn)
        .into_iter()
        .map(|y| (y.fiscal_year, pick(&y)))
        .reduce(|best, next| if next.1 > best.1 { next } else { best })
}

/// State revenue and enrolment summed over the comparable districts, by fiscal year.
///
/// The denominator a community school's draw has to be read against: under the deduct mechanism
/// that operated until the Fair School Funding Plan, every dollar a community school received was
/// subtracted from a resident district's foundation payment, so its share of *this* total is what
/// Ohio's districts gave up to it.
#[must_use]
pub fn district_state_revenue() -> std::collections::BTreeMap<u16, (f64, f64)> {
    let mut out: std::collections::BTreeMap<u16, (f64, f64)> = std::collections::BTreeMap::new();
    for row in ohio_panel::panel().iter().filter(|r| r.comparable) {
        let entry = out.entry(row.fiscal_year).or_default();
        entry.0 += row.state_revenue;
        entry.1 += row.enrollment;
    }
    out
}

/// An agency's state revenue as a fraction of what Ohio's districts received that year.
///
/// `None` where the panel does not carry the agency in `year`.
#[must_use]
pub fn share_of_district_state_revenue(irn: &str, year: u16) -> Option<f64> {
    let row = history(irn).into_iter().find(|y| y.fiscal_year == year)?;
    let (total, _) = *district_state_revenue().get(&year)?;
    (total > 0.0).then(|| row.state * row.enrolment / total)
}

/// Where an agency ranks by state revenue among every Ohio agency the panel carries that year.
///
/// One-based, largest first, and over *all* agencies rather than the comparable ones — a
/// community school is not a district and the question is how much money it received.
#[must_use]
pub fn rank_by_state_revenue(irn: &str, year: u16) -> Option<usize> {
    let mut agencies: Vec<(f64, String)> = ohio_panel::panel()
        .into_iter()
        .filter(|r| r.fiscal_year == year)
        .map(|r| (r.state_revenue, r.irn))
        .collect();
    agencies.sort_by(|a, b| b.0.total_cmp(&a.0));
    agencies
        .iter()
        .position(|(_, key)| key == irn)
        .map(|index| index + 1)
}

/// Where a district stands now, on the four measures its node recorded as not held.
#[derive(Debug, Clone, PartialEq)]
pub struct Standing {
    /// The district's published name.
    pub name: String,
    /// Assessed valuation per pupil, TY2023.
    pub valuation_per_pupil: f64,
    /// Total current operating millage — the rate voters approved.
    pub voted_millage: f64,
    /// Effective Class I operating millage — the rate anyone actually pays.
    pub effective_millage: f64,
    /// Whether the effective rate sits on the twenty-mill floor.
    pub at_floor: bool,
    /// Classroom share of operating spending, FY2025, in points.
    pub classroom_share: f64,
    /// Where that share sits among the 606 districts, as a percentile.
    pub classroom_share_percentile: f64,
}

impl Standing {
    /// The fraction of voted millage H.B. 920's reduction factors remove.
    #[must_use]
    pub fn rollback(&self) -> f64 {
        1.0 - self.effective_millage / self.voted_millage
    }
}

/// A district's wealth, tax rate, floor status and spending composition.
///
/// Returns `None` for an IRN missing from the profile report, the function file, or the report
/// card — the three panels [`composition::frame`] joins.
#[must_use]
pub fn standing(irn: &str) -> Option<Standing> {
    let profile = profile::districts().into_iter().find(|d| d.irn == irn)?;
    let mut shares: Vec<(f64, String)> = composition::frame()
        .into_iter()
        .map(|d| (d.classroom_share(), d.irn))
        .collect();
    shares.sort_by(|a, b| a.0.total_cmp(&b.0));
    let position = shares.iter().position(|(_, key)| key == irn)?;
    Some(Standing {
        name: profile.name.clone(),
        valuation_per_pupil: profile.valuation_per_pupil?,
        voted_millage: profile.current_operating_millage?,
        effective_millage: profile.effective_class1_millage?,
        at_floor: profile.at_twenty_mill_floor(),
        classroom_share: shares[position].0,
        #[allow(clippy::cast_precision_loss)]
        classroom_share_percentile: 100.0 * position as f64 / (shares.len() - 1) as f64,
    })
}

/// How far a district's state revenue per pupil stepped at FY2016, and whether it stayed there.
///
/// The measure is the mean of FY2016-FY2019 against the mean of FY2012, FY2013 and FY2015 — three
/// years either side of the break, so a single construction year cannot produce one. Returned as
/// a fraction, ordered smallest first, over the comparable districts the panel carries in all
/// seven years.
///
/// The FY2016 step **as the survey publishes it**, which is not one basis across that year.
///
/// Returned as `(irn, step, FY2016 pupils)`, smallest first. Kept under this name because it is
/// the measure `education-agency/toledo-city` published — twenty-seven districts falling more
/// than a fifth, holding 6.70% of the panel's pupils, against a median district rising 9.56% —
/// and a withdrawal has to be able to quote what it withdraws.
///
/// **Do not read it as a funding change.** From FY2016 the Bureau nets each district's
/// community-school deduct out of its state revenue and before FY2016 it does not, so this
/// measure charges a district its own deduct. Every one of Ohio's big-city districts gained state
/// money across the step and this shows four of the five losing a fifth. Use
/// [`crate::survey_basis::step`] with [`Basis::Gross`] or [`Basis::Net`] instead.
#[must_use]
pub fn fy2016_step() -> Vec<(String, f64, f64)> {
    crate::survey_basis::step(crate::survey_basis::Basis::AsPublished)
}
