//! How the enrollment cluster's decline actually runs, before anyone prices a response to it.
//!
//! [`crate::guarantee_origin`] found that the 187 guaranteed districts the minimum state share
//! does not explain are two populations, and that one of them — **89 districts holding
//! $108,568,921.47** and 196,869.5 enrolled ADM — is on the floor for no reason but having fewer
//! children than in FY2020. It named that cluster a candidate blind spot and stopped.
//!
//! This module characterizes it. Nothing here prices a response: the point is to establish what
//! shape a response would have to have, and two of the four findings below rule out shapes that
//! looked reasonable beforehand.
//!
//! # The decline is accelerating, not settling
//!
//! Split at the seam where the two enrollment series meet ([`legs`]), annualized so a four-year
//! span and a two-year span can be compared:
//!
//! | | FY2020-FY2024 | FY2024-FY2026 | still falling | accelerating |
//! |---|--:|--:|--:|--:|
//! | enrollment cluster | -1.88%/yr | **-2.74%/yr** | 87 of 89 | 59 of 89 |
//! | capacity cluster | -1.25%/yr | -1.63%/yr | 83 of 98 | 54 of 98 |
//! | districts the guarantee does not pay | -0.97%/yr | -1.13%/yr | 238 of 315 | 182 of 315 |
//!
//! Every group in Ohio is shrinking and every group is shrinking faster than it was. The
//! enrollment cluster is steepest on both legs and its gap over the others widens — it is not a
//! population that has absorbed a shock and levelled off, which is the shape a one-time rebase
//! would suit.
//!
//! # It is demographic, and there is nothing to recover
//!
//! **All 89 have been shrinking across the whole fourteen-year F-33 panel, and not one of them
//! is a recent-onset decline.** Of the 87 still falling, 87 are long-run decliners and **zero**
//! began falling after the anchor year. A policy response cannot recruit these pupils back
//! because no policy lost them.
//!
//! The one channel that would be contestable is pupils resident in the district and taught
//! somewhere else — open enrolment out, community schools, scholarships. [`provenance`] measures
//! it as SD-1's resident ADM over the profile report's enrolled ADM, and it is **present but not
//! the driver**:
//!
//! | | median | 90th pct | above 1.10 | above 1.20 |
//! |---|--:|--:|--:|--:|
//! | enrollment cluster | 1.0951 | 1.3579 | 43 of 89 | 20 of 89 |
//! | capacity cluster | 1.0595 | 1.1793 | 32 of 98 | 6 of 98 |
//! | districts the guarantee does not pay | 1.0532 | 1.2781 | 106 of 312 | 50 of 312 |
//!
//! The cluster sits highest, and the tails overlap so heavily that the ratio orders nothing. Read
//! it as a minority of the loss rather than an account of it. The channel that would settle it
//! per district is the voucher and community-school deduct, which
//! [`crates/README.md`](../../README.md) records as unbuilt.
//!
//! # The guarantee is recruiting this population, not releasing it
//!
//! This is the finding that bears on whether a temporary floor can ever be temporary. Compare the
//! FY2020 base against the formula's own output in each year, with the phase-in taken out of both
//! (see [`Divergence`]):
//!
//! | | median FY2026 | median FY2027 | widening | on the guarantee in FY2026 | crossed onto it |
//! |---|--:|--:|--:|--:|--:|
//! | enrollment cluster | **0.9834** | 1.0813 | 87 of 89 | **39 of 89** | **50** |
//! | capacity cluster | 1.3110 | 1.6108 | 97 of 98 | 90 of 98 | 8 |
//!
//! In FY2026 the median district of the enrollment cluster was **paid by the formula** — its base
//! sat below what the formula computed, so the guarantee owed it nothing. A year later the median
//! is above the base and 50 of the 89 have crossed onto the floor. The two measures partition the
//! cluster exactly: 39 already there plus 50 arriving is 89, and they are computed from different
//! columns.
//!
//! The capacity cluster is the opposite and is what a transitional instrument is supposed to look
//! like: a settled population, 90 of 98 already on it, which
//! [`guarantee_origin::years_to_the_floor`] puts a median 1.37 years from the minimum state
//! share, after which it joins the 107 the minimum share already explains.
//!
//! So the guarantee is doing two jobs on one instrument and they run on **opposite clocks**. One
//! population is draining out of it toward a floor that makes it permanent. The other is filling
//! it, at an accelerating rate, from districts the formula was paying a year ago. Retiring the
//! instrument on a date reaches the first. It reaches the second only by cutting districts whose
//! enrollment fell after the date was chosen.
//!
//! # What a response would have to reach
//!
//! The cluster is small money and concentrated: the **top ten districts hold 55.6%** of the
//! $108.6m, and the median member is a 1,247.5-ADM district drawing **$464.06 a pupil** — p10
//! $40.52, p90 $1,498.85. Half the population is held by an amount that would not survive
//! rounding in a budget act.
//!
//! Set that against [`crate::panel::ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL`]. `[M]` pays **$250 a
//! pupil** to a district whose enrollment rose past
//! [`crate::panel::ENROLLMENT_GROWTH_THRESHOLD`], 3%. The guarantee is paying the median
//! enrollment-loss district **1.86 times that rate** for the opposite movement — as a floor, to
//! whoever happens to cross it, with no threshold, no rate, and no statement anywhere that this
//! is what it is for.
//!
//! **The asymmetry is therefore not that Ohio declines to pay for enrollment decline. It is that
//! Ohio pays for it at roughly twice the rate it pays for growth, through an instrument that
//! names neither.** That is the sentence a design should start from, and it is not the sentence
//! the corpus held before this module.
//!
//! # What is deliberately not here
//!
//! No shape is priced and no lever is added. Four candidate shapes were named in the brief that
//! commissioned this — mirroring `[M]`, a rolling multi-year ADM in the pupil count, a fixed-term
//! self-extinguishing phase-down, and doing nothing — and the measurements above bear on which
//! are viable without choosing between them. The accelerating trend is evidence against a
//! one-time rebase; the crossing rate is evidence against any fixed-date instrument; neither is
//! evidence for a particular alternative.
//!
//! # The measurement seam, restated
//!
//! [`legs`] inherits the two-series problem [`guarantee_origin`] documents: the department's
//! enrolled ADM begins at FY2023 and the only count reaching FY2020 is the F-33 survey's `V33`
//! fall membership, which is not the same measure. Each leg is a **rate within one series** and
//! the two are never divided into each other. Their product is
//! [`guarantee_origin::enrollment_index`], which is why the cluster definition and these
//! trajectories cannot disagree.
//!
//! The fourteen-year rate is a third series read the same way — a rate, from
//! [`DistrictRecord::long_run_enrollment_rate`] — and it reaches every member of this cluster,
//! which is why the "no recent-onset decline" claim needs no missing-data caveat.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};

use crate::guarantee_origin::{self, Origin, ANCHOR, SEAM};
use crate::panel::DistrictRecord;
use crate::prior_model;

/// Years in the survey leg, `ANCHOR` to `SEAM`.
pub const SURVEY_YEARS: f64 = (SEAM - ANCHOR) as f64;

/// Years in the departmental leg, `SEAM` to FY2026 — the span of [`DistrictRecord::adm_history`].
pub const DEPARTMENTAL_YEARS: f64 = 2.0;

/// The districts this module characterizes: the enrollment-loss half of the 187.
#[must_use]
pub fn cluster(panel: &[DistrictRecord]) -> Vec<&DistrictRecord> {
    let index = guarantee_origin::enrollment_index(panel);
    guarantee_origin::above_the_minimum(panel)
        .into_iter()
        .filter(|record| {
            guarantee_origin::decompose(record, &index).and_then(|d| d.origin())
                == Some(Origin::EnrollmentLoss)
        })
        .collect()
}

/// One district's decline, split at the seam the two series meet at.
///
/// Both legs are annualized so that a four-year span and a two-year span can be compared. Neither
/// is a count crossing the seam: the first is `V33` fall membership on itself, the second is
/// enrolled ADM on itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Legs {
    /// The district.
    pub irn: String,
    /// Annualized rate from [`ANCHOR`] to [`SEAM`], on the survey's count.
    pub survey: f64,
    /// Annualized rate from [`SEAM`] to FY2026, on the department's.
    pub departmental: f64,
    /// The fourteen-year F-33 rate, where the survey reaches this district.
    pub long_run: Option<f64>,
}

impl Legs {
    /// Whether the decline is slower in the recent leg than in the older one.
    #[must_use]
    pub fn decelerating(&self) -> bool {
        self.departmental > self.survey
    }

    /// Whether the district is still losing pupils in the most recent leg.
    #[must_use]
    pub fn still_falling(&self) -> bool {
        self.departmental < 0.0
    }

    /// Whether the fourteen-year rate is negative — decline that predates the anchor year.
    ///
    /// A district shrinking for fourteen years is shrinking for reasons no school funding formula
    /// reaches. One that began shrinking after FY2020 is a different question.
    #[must_use]
    pub fn long_run_decline(&self) -> Option<bool> {
        self.long_run.map(|rate| rate < 0.0)
    }
}

/// Each district's decline split at the seam, keyed by IRN.
///
/// Built from the same two series and the same LEAID resolution as
/// [`guarantee_origin::enrollment_index`], whose product is these two legs multiplied together.
#[must_use]
pub fn legs(panel: &[DistrictRecord]) -> BTreeMap<String, Legs> {
    let rows = dispersion::ohio_panel::panel();
    let mut irn_of: BTreeMap<String, String> = BTreeMap::new();
    for row in rows.iter().filter(|r| r.comparable && !r.irn.is_empty()) {
        irn_of.insert(row.leaid.clone(), row.irn.clone());
    }
    let mut first: BTreeMap<String, f64> = BTreeMap::new();
    let mut last: BTreeMap<String, f64> = BTreeMap::new();
    for row in rows.iter().filter(|r| r.comparable && r.enrollment > 0.0) {
        let Some(irn) = irn_of.get(&row.leaid) else {
            continue;
        };
        if row.fiscal_year == ANCHOR {
            first.insert(irn.clone(), row.enrollment);
        } else if row.fiscal_year == SEAM {
            last.insert(irn.clone(), row.enrollment);
        }
    }
    panel
        .iter()
        .filter_map(|record| {
            let start = *first.get(&record.irn)?;
            let seam = *last.get(&record.irn)?;
            let [at_seam, _, latest] = record.adm_history;
            if start <= 0.0 || at_seam <= 0.0 {
                return None;
            }
            Some((
                record.irn.clone(),
                Legs {
                    irn: record.irn.clone(),
                    survey: (seam / start).powf(1.0 / SURVEY_YEARS) - 1.0,
                    departmental: (latest / at_seam).powf(1.0 / DEPARTMENTAL_YEARS) - 1.0,
                    long_run: record.long_run_enrollment_rate,
                },
            ))
        })
        .collect()
}

/// How far the FY2020 base stands above what the formula computes, in one year.
///
/// The phase-in is **not** in this ratio, and that is the point. FY2026 runs at 83.33% and FY2027
/// at 100%, so any quantity carrying the interpolation moves between the two years for a reason
/// that has nothing to do with the district. `[Hb] foundation_calculated` is the formula's own
/// output before the phase-in and before the guarantee, and at FY2027's 100% the phase-in returns
/// the computed amount unchanged — so [`DistrictRecord::core_foundation_funding`] is the same
/// quantity a year later.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Divergence {
    /// `[Ha] / [Hb]` in the FY2026 model.
    pub before: f64,
    /// The same ratio in FY2027.
    pub after: f64,
}

impl Divergence {
    /// Whether the formula moved further from the district's FY2020 base over the year.
    #[must_use]
    pub fn widening(&self) -> bool {
        self.after > self.before
    }

    /// The change, in multiples of the formula amount.
    #[must_use]
    pub fn change(&self) -> f64 {
        self.after - self.before
    }
}

/// One district's gap between the anchor and the formula, in FY2026 and FY2027.
///
/// `None` where the FY2026 model does not carry the district or either year computes nothing.
#[must_use]
pub fn divergence(record: &DistrictRecord, prior: &prior_model::Prior) -> Option<Divergence> {
    if prior.foundation_calculated <= 0.0 || record.core_foundation_funding <= 0.0 {
        return None;
    }
    Some(Divergence {
        before: prior.funding_base / prior.foundation_calculated,
        after: record.realized_aid() / record.core_foundation_funding,
    })
}

/// What the corpus can and cannot see about where a district's pupils went.
#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    /// The district.
    pub irn: String,
    /// Resident ADM over enrolled ADM. Pupils the district is responsible for and does not teach
    /// — open enrolment out, community schools, scholarships — raise this above 1.
    pub resident_over_enrolled: f64,
    /// The fourteen-year rate, where the survey reaches the district.
    pub long_run: Option<f64>,
}

/// The resident-over-enrolled ratio for every district the valuation frame reaches.
#[must_use]
pub fn provenance(panel: &[DistrictRecord]) -> BTreeMap<String, Provenance> {
    // Both counts come from the same frame, so the ratio never crosses a measure the way the
    // enrollment legs have to. SD-1's ADM over the profile report's enrolled ADM.
    let ratio: BTreeMap<String, f64> = dispersion::valuation::frame()
        .into_iter()
        .filter(|row| row.enrolled > 0.0 && row.resident > 0.0)
        .map(|row| (row.irn.clone(), row.resident / row.enrolled))
        .collect();
    panel
        .iter()
        .filter_map(|record| {
            let resident_over_enrolled = *ratio.get(&record.irn)?;
            Some((
                record.irn.clone(),
                Provenance {
                    irn: record.irn.clone(),
                    resident_over_enrolled,
                    long_run: record.long_run_enrollment_rate,
                },
            ))
        })
        .collect()
}

/// What a cluster's decline looks like, summarized the way a policy design would need it.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// How many districts.
    pub districts: usize,
    /// Their guarantee.
    pub dollars: Dollars,
    /// Their current-year enrolled ADM.
    pub adm: Adm,
    /// Upper-middle annualized rate before the seam.
    pub median_survey: f64,
    /// Upper-middle annualized rate after it.
    pub median_departmental: f64,
    /// How many are still losing pupils in the recent leg.
    pub still_falling: usize,
    /// How many are losing them more slowly than before the seam.
    pub decelerating: usize,
    /// How many have been shrinking across the whole fourteen-year panel.
    pub long_run_decline: usize,
    /// How many the fourteen-year panel does not reach.
    pub long_run_unknown: usize,
}

/// Summarize one set of districts.
#[must_use]
pub fn shape(rows: &[&DistrictRecord], legs: &BTreeMap<String, Legs>) -> Shape {
    let mine: Vec<&Legs> = rows.iter().filter_map(|r| legs.get(&r.irn)).collect();
    Shape {
        districts: rows.len(),
        dollars: rows.iter().map(|r| r.guarantee).sum(),
        adm: rows.iter().map(|r| r.current_year_adm).sum(),
        median_survey: upper_middle(mine.iter().map(|l| l.survey).collect()),
        median_departmental: upper_middle(mine.iter().map(|l| l.departmental).collect()),
        still_falling: mine.iter().filter(|l| l.still_falling()).count(),
        decelerating: mine.iter().filter(|l| l.decelerating()).count(),
        long_run_decline: mine
            .iter()
            .filter(|l| l.long_run_decline() == Some(true))
            .count(),
        long_run_unknown: mine.iter().filter(|l| l.long_run.is_none()).count(),
    }
}

/// The crates' median: sort, then take the upper middle. Never the mean of two.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty(), "no values to take a median of");
    values.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a measured rate"));
    values[values.len() / 2]
}
