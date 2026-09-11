//! The break the panel's state column has at FY2016, which belongs to the survey and not to Ohio.
//!
//! # The question this answers, and the answer being a withdrawal
//!
//! `education-agency/toledo-city` recorded that Toledo's state revenue per pupil falls about 30%
//! at FY2016 and never recovers, that twenty-seven districts fall more than a fifth while the
//! median district rises, and that the cause was not established. [`crate::fy2016`] then measured
//! the median rise against H.B. 64's capacity aid and left the tail open against three named
//! non-explanations — the tangible personal property phase-out, capacity aid itself, and a
//! transfer to community schools.
//!
//! The tail is none of those because **the tail is not an event**. From FY2016 the Census Bureau
//! subtracts a district's payments to community schools from its general formula assistance, and
//! before FY2016 it does not. What falls out of Ohio's state column at FY2016 is money the
//! district never kept, counted twice until that year and once afterwards.
//!
//! # The publisher says so, in its own state notes
//!
//! > Revenues for Ohio have been adjusted in the reported F-33 data to eliminate double counting
//! > of state funding for independent charter school LEAs. Ohio accounts for state funding of
//! > independent charter school LEAs within both the state revenues of those independent charter
//! > school LEAs and the state revenues of the (noncharter) regular local school districts that
//! > charter school students reside in. To mitigate this double counting, payments to charter
//! > schools (V92) were subtracted from general formula assistance state revenues (C01) for all
//! > regular, noncharter school districts.
//!
//! The note is **absent from the FY2015 documentation and present from the FY2016 documentation**,
//! which brackets the change to the year the data puts it in. It is also the only note of its kind
//! written for any state: Ohio's deduct is the reason the adjustment exists.
//!
//! # And the data locates it without the note
//!
//! [`transitions`] correlates each district's deduct per pupil against its change in state revenue
//! per pupil, for every consecutive pair of years the panel holds. A district's deduct predicts
//! its change at exactly one transition — **FY2015 to FY2016, at −0.2506** — and nowhere else;
//! the next largest reading in fourteen years is +0.1026, and putting the two years on one basis
//! takes the FY2016 figure to −0.0183. The per-district identity is the same fact seen closer:
//! Cleveland's general formula assistance is $10,258 per pupil in FY2015 and $6,721 in FY2016,
//! and $6,721 plus its deduct is $10,170.
//!
//! # What it overturned
//!
//! | FY2016 step | as published | gross of the deduct |
//! |---|--:|--:|
//! | Columbus | −29.22% | **+14.93%** |
//! | Toledo | −20.49% | **+13.88%** |
//! | Cleveland | −19.55% | **+7.87%** |
//! | Cincinnati | −15.77% | **+6.12%** |
//! | Dayton | −3.50% | **+32.14%** |
//! | districts falling more than a fifth | 27, 6.70% of pupils | 19, **1.66%** of pupils |
//! | median district | +9.56% | **+13.74%** |
//!
//! Every one of Ohio's big-city districts *gained* state money across the year the corpus
//! published that they lost a fifth to a third of it. The tail was the largest deducts in the
//! state, and the twenty-seven districts in it held a sixth as many pupils as it appeared.
//!
//! The reported **local** share carries the same break in the other direction, because the
//! adjustment reaches total revenue too: Ohio's districts appear to go from 50.19% locally funded
//! in FY2015 to 52.84% in FY2016, and on one basis they go from 52.39% to 52.84%. Five sixths of
//! a jump the corpus could have read as a shift onto the property tax is the Bureau's bookkeeping.
//!
//! **The capacity-aid finding survives**, and is cleaner without the artifact beside it: the step
//! still runs against the log of total assessed value at −0.30, the quintile gradient is still
//! monotone from the smallest-valuation fifth to the largest, and with the tail gone every fifth
//! gains rather than one fifth contradicting the mechanism. See [`crate::fy2016`].
//!
//! # The correction, and the two years it cannot be applied to
//!
//! [`Basis::Net`] is state money the district kept and [`Basis::Gross`] is the formula amount
//! before the deduct. Either is a series; the published column is neither.
//!
//! `V92` is written as reported, and in two years it is reported as a zero rather than as a
//! missing value. **FY2009** carries 108 districts at zero including Toledo and Dayton, which had
//! thousands of community-school pupils that year and report tens of millions in FY2010;
//! **FY2022** carries 585 of 613 at zero. A zero and an absence are indistinguishable in the
//! column, so [`Basis::Net`] begins at FY2010 and [`Basis::Gross`] ends at FY2021. Net reaches
//! FY2024 because the published column is already net there and needs no deduct at all — it is
//! the basis to prefer, and the one the corpus's fifteen-year figures should have been on.

use std::collections::BTreeMap;

use crate::ohio_panel::{self, PanelRow};

/// The first fiscal year the Bureau nets Ohio's community-school deduct out of state revenue.
pub const ADJUSTED_FROM: u16 = 2016;

/// The years the deduct column can be trusted, inclusive.
///
/// Outside them the survey accepts an unreported deduct as `0`, which no rule can tell from a
/// district that paid nothing. See the module docs.
pub const DEDUCT_REPORTED: [u16; 2] = [2010, 2021];

/// Which of the two quantities the published column is on either side of [`ADJUSTED_FROM`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// The column as the survey publishes it: gross of the deduct before [`ADJUSTED_FROM`] and
    /// net of it afterwards. Not a series. Kept so the break can be measured rather than only
    /// described.
    AsPublished,
    /// State money the district kept. Available FY2010 through the end of the panel.
    Net,
    /// The formula amount before the deduct. Available FY2009 through FY2021.
    Gross,
}

impl Basis {
    /// One row's state revenue on this basis, or `None` where the row cannot be put on it.
    #[must_use]
    pub fn of(self, row: &PanelRow) -> Option<f64> {
        let year = row.fiscal_year;
        let reported = || {
            (DEDUCT_REPORTED[0]..=DEDUCT_REPORTED[1])
                .contains(&year)
                .then_some(row.charter_payments)
                .flatten()
        };
        match self {
            Self::AsPublished => Some(row.state_revenue),
            // Already net from the adjustment year, so the later years need no deduct and the
            // basis reaches the end of the panel.
            Self::Net if year >= ADJUSTED_FROM => Some(row.state_revenue),
            Self::Net => Some(row.state_revenue - reported()?),
            // Symmetrically, the earlier years are already gross.
            Self::Gross if year < ADJUSTED_FROM => Some(row.state_revenue),
            Self::Gross => Some(row.state_revenue + reported()?),
        }
    }

    /// The same, per pupil. `None` where the row is not on this basis or reports no pupils.
    #[must_use]
    pub fn per_pupil(self, row: &PanelRow) -> Option<f64> {
        (row.enrollment > 0.0).then(|| self.of(row).map(|v| v / row.enrollment))?
    }
}

/// One consecutive pair of years, and how much of the move between them the deduct predicts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    /// The earlier year.
    pub from: u16,
    /// The later one, which is the next year the panel holds rather than `from + 1`.
    pub to: u16,
    /// Correlation between a district's deduct per pupil in `from` and its change in state
    /// revenue per pupil, on the column as published.
    pub as_published: f64,
    /// The same correlation with both years put on [`Basis::Gross`].
    pub corrected: f64,
    /// How many districts both years hold.
    pub districts: usize,
}

/// Every consecutive pair of years the deduct can be measured across, so the break locates itself.
///
/// A definitional change shows up here and a funding change does not: if the Bureau starts
/// netting the deduct out in one year, the districts with the largest deducts move down the most
/// in that year and in no other.
///
/// The pairs run FY2010 to FY2022 — eleven of them — because the earlier year has to carry a
/// believable deduct for the correlation to mean anything, and that is [`DEDUCT_REPORTED`]. The
/// corrected column is [`Basis::Gross`] where both years reach it and [`Basis::Net`] for the last
/// pair, which is the only pair where they differ in availability.
#[must_use]
pub fn transitions() -> Vec<Transition> {
    let mut by_year: BTreeMap<u16, Vec<PanelRow>> = BTreeMap::new();
    for row in ohio_panel::panel() {
        if row.comparable && row.enrollment > 0.0 {
            by_year.entry(row.fiscal_year).or_default().push(row);
        }
    }
    let years: Vec<u16> = by_year.keys().copied().collect();

    years
        .windows(2)
        .filter_map(|pair| {
            let (from, to) = (pair[0], pair[1]);
            if !(DEDUCT_REPORTED[0]..=DEDUCT_REPORTED[1]).contains(&from) {
                return None;
            }
            let consistent = if to <= DEDUCT_REPORTED[1] {
                Basis::Gross
            } else {
                Basis::Net
            };
            let later: BTreeMap<&str, &PanelRow> =
                by_year[&to].iter().map(|r| (r.irn.as_str(), r)).collect();

            let (mut deduct, mut published, mut corrected) = (Vec::new(), Vec::new(), Vec::new());
            for early in &by_year[&from] {
                let Some(late) = later.get(early.irn.as_str()) else {
                    continue;
                };
                let (Some(paid), Some(was), Some(now)) = (
                    early.charter_payments,
                    Basis::AsPublished.per_pupil(early),
                    Basis::AsPublished.per_pupil(late),
                ) else {
                    continue;
                };
                let (Some(gross_was), Some(gross_now)) =
                    (consistent.per_pupil(early), consistent.per_pupil(late))
                else {
                    continue;
                };
                if was <= 0.0 || gross_was <= 0.0 {
                    continue;
                }
                deduct.push(paid / early.enrollment);
                published.push(now / was - 1.0);
                corrected.push(gross_now / gross_was - 1.0);
            }

            let correlate = |ys: &[f64]| {
                crate::wealth_neutrality(&deduct, ys)
                    .ok()
                    .map(|fit| fit.correlation)
            };
            Some(Transition {
                from,
                to,
                as_published: correlate(&published)?,
                corrected: correlate(&corrected)?,
                districts: deduct.len(),
            })
        })
        .collect()
}

/// The transition the deduct explains, if exactly one does.
///
/// "Explains" is `|r| > 0.2` on the published column with the correction removing at least half
/// of it. The threshold separates rather than fits: the FY2016 transition reads −0.2506 and the
/// largest of the other thirteen reads +0.1026, so any cut between those two picks out the same
/// single year.
#[must_use]
pub fn break_year() -> Option<u16> {
    let found: Vec<u16> = transitions()
        .into_iter()
        .filter(|t| t.as_published.abs() > 0.2 && t.corrected.abs() * 2.0 < t.as_published.abs())
        .map(|t| t.to)
        .collect();
    (found.len() == 1).then(|| found[0])
}

/// Where a year's school money came from, over comparable districts, on one basis.
///
/// Returned as `(local, state)` shares of total revenue. Total revenue moves with the state
/// column, because the Bureau's adjustment reaches `TOTALREV` through `TSTREV` on every row —
/// so the reported *local* share carries the break too, in the opposite direction.
#[must_use]
pub fn shares(basis: Basis) -> BTreeMap<u16, (f64, f64)> {
    let mut totals: BTreeMap<u16, (f64, f64, f64)> = BTreeMap::new();
    for row in ohio_panel::panel().iter().filter(|r| r.comparable) {
        let Some(state) = basis.of(row) else { continue };
        let entry = totals.entry(row.fiscal_year).or_default();
        // The adjustment moved state revenue and total revenue by the same dollar, so restating
        // the state column restates the denominator with it. Local revenue is untouched.
        entry.0 += row.local_revenue;
        entry.1 += state;
        entry.2 += row.total_revenue - row.state_revenue + state;
    }
    totals
        .into_iter()
        .filter(|(_, (_, _, total))| *total > 0.0)
        .map(|(year, (local, state, total))| (year, (local / total, state / total)))
        .collect()
}

/// The FY2016 step — mean state revenue per pupil after against before — on a chosen basis.
///
/// Returned as `(irn, step, FY2016 pupils)`, smallest step first, which is the shape
/// [`crate::fy2016`] reads. A district enters only if it reports every year on the basis asked
/// for, so [`Basis::Gross`] and [`Basis::AsPublished`] cover the same districts and the
/// comparison between them is between measures rather than between populations.
#[must_use]
pub fn step(basis: Basis) -> Vec<(String, f64, f64)> {
    let before = [2012u16, 2013, 2015];
    let after = [2016u16, 2017, 2018, 2019];

    let mut by_district: BTreeMap<String, Vec<(u16, f64, f64)>> = BTreeMap::new();
    for row in ohio_panel::panel()
        .iter()
        .filter(|r| r.comparable && r.enrollment >= ohio_panel::MIN_ENROLMENT)
    {
        if let Some(per_pupil) = basis.per_pupil(row) {
            by_district.entry(row.irn.clone()).or_default().push((
                row.fiscal_year,
                per_pupil,
                row.enrollment,
            ));
        }
    }

    let mean = |rows: &[(u16, f64, f64)], years: &[u16]| -> Option<f64> {
        let found: Vec<f64> = years
            .iter()
            .filter_map(|y| rows.iter().find(|r| r.0 == *y).map(|r| r.1))
            .collect();
        (found.len() == years.len())
            .then(|| found.iter().sum::<f64>() / found.len() as f64)
            .filter(|m| *m > 0.0)
    };

    let mut out: Vec<(String, f64, f64)> = by_district
        .into_iter()
        .filter_map(|(irn, rows)| {
            let pupils = rows.iter().find(|r| r.0 == 2016)?.2;
            Some((
                irn,
                mean(&rows, &after)? / mean(&rows, &before)? - 1.0,
                pupils,
            ))
        })
        .collect();
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    out
}

/// How many districts fall more than `depth`, and what share of the state's pupils they hold.
///
/// The pair the corpus published as "twenty-seven districts, 6.7% of pupils". On one basis it is
/// a sixth of that by pupils, because five of the districts in it were the five largest deducts
/// in Ohio.
#[must_use]
pub fn fallers(basis: Basis, depth: f64) -> (usize, f64) {
    let steps = step(basis);
    let total: f64 = steps.iter().map(|(_, _, pupils)| pupils).sum();
    let deep: Vec<&(String, f64, f64)> = steps.iter().filter(|(_, s, _)| *s < -depth).collect();
    let held: f64 = deep.iter().map(|(_, _, pupils)| pupils).sum();
    (deep.len(), if total > 0.0 { held / total } else { 0.0 })
}

/// One district's deduct as a share of the state money it was credited with, in one year.
///
/// The quantity that makes the artifact's size legible: a district whose deduct is a third of its
/// credited state revenue appears to lose a third of that revenue in the adjustment year.
#[must_use]
pub fn deduct_share(irn: &str, year: u16) -> Option<f64> {
    let row = ohio_panel::panel()
        .into_iter()
        .find(|r| r.irn == irn && r.fiscal_year == year)?;
    let paid = (DEDUCT_REPORTED[0]..=DEDUCT_REPORTED[1])
        .contains(&year)
        .then_some(row.charter_payments)
        .flatten()?;
    let gross = Basis::Gross.of(&row)?;
    (gross > 0.0).then_some(paid / gross)
}
