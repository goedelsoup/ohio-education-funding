//! Which terms of the Fair School Funding Plan depend on how many pupils a district has, and
//! what they do to the guarantee.
//!
//! [`crate::staffing_minimums`] answered half of a shape and named the residual rather than
//! leaving it implied. The shape is the statewide guarantee rate by ADM sextile, which is
//! **non-monotone**: lowest among the smallest districts, highest in the third. R.C. 3317.011's
//! staffing floors are half of it. Issue #409 asked what the other half is, and ruled out the
//! obvious candidate on the way: base cost's only size-dependent terms are the floors and the
//! two banded salaries, and the bands are continuous ramps between 500 and 4,000 ADM, so they
//! cannot produce a peak at 1,192.
//!
//! **That reasoning is right and its premise is too narrow. The plan has a second size-dependent
//! term and it is not in base cost.**
//!
//! # 1. Targeted assistance's capacity tier is an enrolment measurement wearing a wealth label
//!
//! R.C. 3317.0217 credits a district with `[C]`, eight mills of however far its **total** weighted
//! wealth falls below the **statewide median district's total** weighted wealth
//! ([`TA_MEDIAN_WEIGHTED_WEALTH`], $392,151,306.63). Both sides of that comparison are whole-district
//! dollars. Nothing in it is per pupil.
//!
//! So a district with few pupils has a small tax base, scores as far below the median district,
//! and is paid for it — however wealthy each of its pupils is. Regressed across all 609
//! districts:
//!
//! | outcome, in logs | on `ln(base cost enrolled ADM)` | slope | r² |
//! |---|---|--:|--:|
//! | total weighted wealth `[A]` | | **0.9855** | **0.8126** |
//! | weighted wealth per resident pupil `[D]` | | -0.0429 | 0.0085 |
//!
//! A slope of one and four fifths of the variance: the tier's index is very nearly a measurement
//! of enrolment. The same districts' wealth **per pupil** has no size gradient at all — a slope
//! of -0.04 and under a hundredth of the variance. The quantity the tier says it is measuring
//! does not vary with size; the quantity it actually reads is size.
//!
//! The tier is worth **$334,036,712.23**, 24.5% of targeted assistance's $1.36bn. It reaches 299
//! districts at a median 876 ADM and **no district above 3,259.83 ADM** — which is the first
//! district of the largest sextile. Per pupil, by sextile: $2,349, $1,616, $433, and then nothing.
//!
//! # 2. It is the residual, and the two terms together are the whole shape
//!
//! [`striking_out`] runs the panel with either or both struck out. Ordered on base cost enrolled
//! ADM, the count of districts the guarantee pays:
//!
//! | | 1 | 2 | 3 | 4 | 5 | 6 | first-to-third gap |
//! |---|--:|--:|--:|--:|--:|--:|--:|
//! | as the plan stands | **18** | 39 | 71 | 60 | 49 | 57 | **53** |
//! | staffing floors struck out | 48 | 51 | 74 | 61 | 49 | 57 | 26 |
//! | capacity tier struck out | 52 | 72 | 88 | 63 | 49 | 57 | 36 |
//! | **both struck out** | **89** | 82 | **89** | 64 | 49 | 57 | **0** |
//!
//! The gap the floors leave behind closes to **zero**. With both terms gone the guarantee reaches
//! the same share of the smallest sextile as of the third, and the rate falls with size from
//! there. Nothing else was moved to get that, and neither counterfactual needs an estimate: see
//! the note on what is held below.
//!
//! # 3. The size cliff the corpus already knew about is the half that pushes the other way
//!
//! `the_remaining_categoricals.rs` recorded the tier's explicit size provision — nothing below
//! 200 ADM, 5% from 200 to 400, ramping to full only at 600
//! ([`capacity_size_share`](crate::panel::TargetedAssistance::capacity_size_share)).
//! It is natural to read that as the mechanism here. It is the opposite of it.
//!
//! The ramp **withholds** $91,859,433 from the 73 districts it touches, all of them small.
//! [`Change::CapacityTierRampRemoved`] pays them in full and the smallest band's guarantee count
//! falls from 48 back to **20**, two short of the 18 the plan itself produces: **28 of the 30
//! districts the floors' removal added to that band go straight back off the guarantee**. The
//! drafters bounded the tier's size dependence with the one provision that is visibly about size,
//! and left the size dependence in the index, where nothing names it.
//!
//! # 4. It is the denominator and not the rate
//!
//! [`Change::CapacityTierMeasuredPerPupil`] changes one thing: the shortfall is taken against
//! [`TA_MEDIAN_WEALTH_PER_PUPIL`] — $276,708.97, the figure the tier's own sibling `[F]` already
//! indexes against — and multiplied by the district's enrolled ADM. Same section, same eight
//! mills, same published median, and **$24.2m less** in aggregate.
//!
//! The sextile row becomes `86 / 74 / 86 / 56 / 45 / 56`, first-to-third gap **0**. Holding the
//! rate and very nearly the money fixed and changing only what the shortfall is measured against
//! removes the whole residual, which is as direct as an attribution to a denominator gets.
//!
//! This is a measurement and not a proposal. No lever was added, nothing here is wired into
//! [`crate::policy`], and the restatement is a duplicate of `[F]`'s form at a third rate — which
//! is a reason to think the General Assembly wanted two different measurements, not that one of
//! them is a mistake. What it establishes is that the residual is in the comparison `[B]` makes
//! and nowhere else.
//!
//! # 5. Why the curve turns over above the third sextile, which is the other open question
//!
//! Not either size term: both have stopped binding by the fourth sextile. It is the anchor's own
//! shape, and issue #409 named this candidate first.
//!
//! `[H2]` per FY2020 pupil declines **monotonically** across the six sextiles — $6,478, $5,877,
//! $4,960, $4,589, $3,812, $3,176, the smallest sextile at **2.04x** the largest. It is a Bridge
//! formula figure resting on an FY2011 Evidence-Based Model run, so the gradient is a property of
//! a superseded regime and not of this plan; see
//! [`guarantee-funding-base`](../../../.yidam/corpus/parameter/guarantee-funding-base.yml).
//!
//! Being monotone is exactly why it is not the answer to the first question — a monotone gradient
//! cannot produce a peak — and it is the whole of the downward force. [`Profile`] demeans the two
//! sides in logs and [`with_a_side_flattened`] recounts with one side's sextile profile removed:
//! flatten the anchor and the curve **rises** with size, `56 / 59 / 88 / 70 / 82 / 90`; flatten
//! formula aid and it **falls**, `97 / 90 / 58 / 71 / 45 / 16`. The observed curve is the two
//! against each other, and the size terms are what tip the balance below 3,260 ADM.
//!
//! # 6. The third candidate, refuted
//!
//! Issue #409 also asked whether local capacity growth correlates with size —
//! [`crate::prior_model`] has capacity moving +9.34% in a year against a base cost moving +0.05%,
//! and if that growth were size-tilted it would be a mechanism. It is not. Median published
//! capacity per pupil, FY2026 to FY2027, by sextile: **10.01%, 9.09%, 9.76%, 9.01%, 9.54%,
//! 8.84%**. No gradient, and the ordering does not even alternate consistently.
//!
//! The *level* of local capacity does vary across the sextiles and the guarantee is sensitive to
//! it. Its **growth** is flat, which is what the candidate was about.
//!
//! # What the counterfactuals hold, and which way that cuts
//!
//! [`Change::StaffingFloorsStruckOut`] is [`crate::staffing_minimums::without_the_floors`]
//! unchanged, and carries its bound: the base-cost-denominated categoricals are priced on a
//! statewide average a repeal would lower, so its 46 districts are a lower bound.
//!
//! **The capacity tier's counterfactual has no such bound and needs no estimate.** `[C]` is paid
//! in full rather than at a state share, it feeds no statewide average — [`TA_MEDIAN_WEIGHTED_WEALTH`]
//! is an input computed from district wealth, not from anything the tier pays — and FY2027's
//! phase-in is 100%, so a dollar off `[C]` is a dollar off `[H]` exactly. Striking it out is
//! arithmetic on published columns.
//!
//! What every row here holds is the anchor. `[H2]` is a FY2020 figure and contains no part of
//! either term: R.C. 3317.011 was enacted for FY2022 and R.C. 3317.0217's merged tiers for the
//! same year. That asymmetry is the whole mechanism — both terms sit on one side of a `max`.

use std::collections::BTreeMap;

use dispersion::{least_squares, Regression};
use edfund_core::{Adm, Dollars};

use crate::guarantee_origin;
use crate::hold_harmless::CENT;
use crate::panel::categoricals::{
    TA_CAPACITY_RATE, TA_MEDIAN_WEALTH_PER_PUPIL, TA_MEDIAN_WEIGHTED_WEALTH,
};
use crate::panel::DistrictRecord;
use crate::staffing_minimums;

/// How many bands the panel is cut into.
///
/// Six because that is the cut `the_other_half_of_the_guarantee_and_the_two_things_it_pays_for.rs`
/// measured the shape on and issue #389 tested against. Nothing here chose it.
pub const BANDS: usize = 6;

/// The panel in [`BANDS`] bands, ordered on base cost enrolled ADM.
///
/// **The count every threshold in R.C. 3317.011 is against**, and the one the whole of this
/// module and [`crate::staffing_minimums`] order on. The older file orders its own sextile row on
/// *current-year* enrolled ADM and reads `17 / 41 / 70 / 61 / 48 / 57`; the two denominators put a
/// handful of districts in adjacent bands, neither row is wrong, and a comparison across them is.
///
/// The last band takes the remainder, so it holds 104 of 609 where the rest hold 101.
#[must_use]
pub fn sextiles(panel: &[DistrictRecord]) -> Vec<Vec<&DistrictRecord>> {
    let mut ordered: Vec<&DistrictRecord> = panel.iter().collect();
    ordered.sort_by(|a, b| a.base_cost_adm().total_cmp(&b.base_cost_adm()));
    let band = ordered.len() / BANDS;
    (0..BANDS)
        .map(|s| {
            let end = if s == BANDS - 1 {
                ordered.len()
            } else {
                (s + 1) * band
            };
            ordered[s * band..end].to_vec()
        })
        .collect()
}

/// One alteration to a size-dependent term, applied to the whole panel at once.
///
/// Each moves formula aid and nothing else. They compose: passing two to [`striking_out`] applies
/// both, and they reach disjoint channels — the floors move base cost aid, the tier moves a
/// categorical — so nothing is counted twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Change {
    /// Every staffing floor struck out of R.C. 3317.011, as
    /// [`crate::staffing_minimums::without_the_floors`] computes it.
    StaffingFloorsStruckOut,
    /// Targeted assistance's capacity tier `[C]` struck out of R.C. 3317.0217.
    CapacityTierStruckOut,
    /// The capacity tier paid at its full eight mills to every district, with
    /// [`TargetedAssistance::capacity_size_share`](crate::panel::TargetedAssistance::capacity_size_share)'s
    /// 200/400/600 ramp removed and its index left alone.
    ///
    /// The only change here that *raises* aid.
    CapacityTierRampRemoved,
    /// The capacity tier's shortfall taken per resident pupil against
    /// [`TA_MEDIAN_WEALTH_PER_PUPIL`] and multiplied by enrolled ADM, rather than taken between
    /// two whole-district totals.
    ///
    /// The tier's own sibling `[F]` already indexes against that published median, so no figure
    /// is invented. The ramp goes with the total it bounded.
    CapacityTierMeasuredPerPupil,
}

impl Change {
    /// The word for it, for a table that has to name it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::StaffingFloorsStruckOut => "staffing floors struck out",
            Self::CapacityTierStruckOut => "capacity tier struck out",
            Self::CapacityTierRampRemoved => "capacity tier ramp removed",
            Self::CapacityTierMeasuredPerPupil => "capacity tier measured per pupil",
        }
    }
}

/// What the capacity tier would pay this district at its full rate, with the size ramp removed.
///
/// Eight mills of the shortfall below [`TA_MEDIAN_WEIGHTED_WEALTH`], clamped at zero — which is
/// R.C. 3317.0217's own arithmetic with one multiplier dropped.
#[must_use]
pub fn capacity_tier_unramped(record: &DistrictRecord) -> Dollars {
    TA_CAPACITY_RATE
        * (TA_MEDIAN_WEIGHTED_WEALTH - record.targeted_assistance.weighted_wealth).max(0.0)
}

/// What the capacity tier would pay this district if its shortfall were measured per pupil.
///
/// Eight mills of the shortfall of the district's weighted wealth **per resident pupil** below
/// [`TA_MEDIAN_WEALTH_PER_PUPIL`], times enrolled ADM. The two pupil counts are the section's own
/// — `[D]` divides by resident ADM and `[F]` multiplies by enrolled ADM, one line apart — so this
/// changes the denominator of the comparison and not which counts the section uses.
#[must_use]
pub fn capacity_tier_per_pupil(record: &DistrictRecord) -> Dollars {
    TA_CAPACITY_RATE
        * (TA_MEDIAN_WEALTH_PER_PUPIL - record.targeted_assistance.wealth_per_pupil).max(0.0)
        * record.current_year_adm
}

/// One district under a stated set of [`Change`]s.
#[derive(Debug, Clone, PartialEq)]
pub struct Counterfactual {
    /// The district, keyed the only way district figures may be keyed.
    pub irn: String,
    /// Its name, for a table a reader can check.
    pub name: String,
    /// Base cost enrolled ADM, the count the bands are ordered on.
    pub adm: Adm,
    /// What the changes do to its formula aid. Negative for every change but
    /// [`Change::CapacityTierRampRemoved`].
    pub aid_delta: Dollars,
    /// Whether the guarantee pays it as the plan stands.
    pub guaranteed: bool,
    /// Whether the guarantee would pay it under the changes.
    pub guaranteed_after: bool,
}

impl Counterfactual {
    /// Whether the changes are what put this district on the guarantee.
    #[must_use]
    pub fn newly_guaranteed(&self) -> bool {
        !self.guaranteed && self.guaranteed_after
    }

    /// Whether the changes are what take it off.
    #[must_use]
    pub fn no_longer_guaranteed(&self) -> bool {
        self.guaranteed && !self.guaranteed_after
    }
}

/// Run the whole panel under a set of changes.
///
/// # Why this is a subtraction and not a re-run
///
/// FY2027's phase-in is 100%, so `[H] Foundation Funding` is the computed amount itself and a
/// dollar off a component is a dollar off `[H]`. Under any lower phase-in it would be the
/// phase-in's share of the dollar, and this would have to interpolate — see
/// [`crate::policy::Policy::phase_in_general`]. The staffing floors are the exception and go through
/// [`crate::staffing_minimums::without_the_floors`], because lowering base cost per pupil meets
/// the minimum state share and that is not linear.
///
/// The order of `changes` does not matter and repeats are idempotent in effect but not in
/// arithmetic, so each variant is applied at most once.
#[must_use]
pub fn striking_out(panel: &[DistrictRecord], changes: &[Change]) -> Vec<Counterfactual> {
    let floors: BTreeMap<String, Dollars> = if changes.contains(&Change::StaffingFloorsStruckOut) {
        staffing_minimums::without_the_floors(panel)
            .into_iter()
            .map(|row| (row.irn, row.aid_delta))
            .collect()
    } else {
        BTreeMap::new()
    };
    panel
        .iter()
        .map(|record| {
            let tier = record.targeted_assistance.capacity_amount;
            let mut delta = 0.0;
            for change in [
                Change::StaffingFloorsStruckOut,
                Change::CapacityTierStruckOut,
                Change::CapacityTierRampRemoved,
                Change::CapacityTierMeasuredPerPupil,
            ] {
                if !changes.contains(&change) {
                    continue;
                }
                delta += match change {
                    // Absent for a district with no positive enrolment on either denominator,
                    // where there is no per-pupil quantity to move and the floors take nothing.
                    Change::StaffingFloorsStruckOut => {
                        floors.get(&record.irn).copied().unwrap_or(0.0)
                    }
                    Change::CapacityTierStruckOut => -tier,
                    Change::CapacityTierRampRemoved => capacity_tier_unramped(record) - tier,
                    Change::CapacityTierMeasuredPerPupil => capacity_tier_per_pupil(record) - tier,
                };
            }
            Counterfactual {
                irn: record.irn.clone(),
                name: record.name.clone(),
                adm: record.base_cost_adm(),
                aid_delta: delta,
                guaranteed: record.on_guarantee(),
                guaranteed_after: record.core_foundation_funding + delta
                    < record.guarantee_floor() - CENT,
            }
        })
        .collect()
}

/// How many districts in each band the guarantee pays, under a set of changes.
///
/// The empty slice is the plan as it stands, and reads `[18, 39, 71, 60, 49, 57]`.
#[must_use]
pub fn guaranteed_by_sextile(panel: &[DistrictRecord], changes: &[Change]) -> [usize; BANDS] {
    let after: BTreeMap<String, bool> = striking_out(panel, changes)
        .into_iter()
        .map(|row| (row.irn, row.guaranteed_after))
        .collect();
    let mut counts = [0usize; BANDS];
    for (band, rows) in sextiles(panel).iter().enumerate() {
        counts[band] = rows
            .iter()
            .filter(|record| {
                after
                    .get(&record.irn)
                    .copied()
                    .unwrap_or(record.on_guarantee())
            })
            .count();
    }
    counts
}

/// The first-to-third gap, which is the non-monotonicity stated as one number.
///
/// 53 as the plan stands, 26 with the floors struck out, and 0 with both terms struck out.
#[must_use]
pub fn first_to_third_gap(counts: [usize; BANDS]) -> i64 {
    counts[2] as i64 - counts[0] as i64
}

/// Whether the capacity tier's index measures wealth or enrolment.
///
/// Two univariate regressions on `ln(base cost enrolled ADM)`, reported as a pair because either
/// alone is uninterpretable: a total that scales with enrolment is unremarkable, and it is
/// unremarkable *because* the per-pupil quantity beside it does not.
///
/// A regression here is a description of a cross-section and not an effect. Nothing is being
/// identified; what the pair establishes is that one of two published columns varies with size
/// and the other does not, which is a fact about the columns.
#[derive(Debug, Clone)]
pub struct WhatTheIndexMeasures {
    /// `ln([A] weighted wealth)` on `ln(ADM)`. Slope 0.9855, r² 0.8126.
    pub total: Regression,
    /// `ln([D] weighted wealth per resident pupil)` on `ln(ADM)`. Slope -0.0429, r² 0.0085.
    pub per_pupil: Regression,
}

/// Fit the pair.
///
/// Districts with a non-positive value on any of the three columns are dropped rather than
/// floored; there are none in the FY2027 panel, and a zero would be a log of negative infinity
/// rather than a small number.
///
/// # Panics
///
/// If either fit fails, which for a 609-row cross-section with one predictor means the fixture
/// has changed shape.
#[must_use]
pub fn what_the_index_measures(panel: &[DistrictRecord]) -> WhatTheIndexMeasures {
    let rows: Vec<&DistrictRecord> = panel
        .iter()
        .filter(|record| {
            record.base_cost_adm() > 0.0
                && record.targeted_assistance.weighted_wealth > 0.0
                && record.targeted_assistance.wealth_per_pupil > 0.0
        })
        .collect();
    let adm: Vec<f64> = rows.iter().map(|r| r.base_cost_adm().ln()).collect();
    let fit = |outcome: Vec<f64>| {
        least_squares(std::slice::from_ref(&adm), &outcome).expect("609 rows, one predictor")
    };
    WhatTheIndexMeasures {
        total: fit(rows
            .iter()
            .map(|r| r.targeted_assistance.weighted_wealth.ln())
            .collect()),
        per_pupil: fit(rows
            .iter()
            .map(|r| r.targeted_assistance.wealth_per_pupil.ln())
            .collect()),
    }
}

/// Which side of the comparison a [`Profile`] row belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// `[H2]` per FY2020 pupil — what the guarantee holds a district at.
    Anchor,
    /// FY2027 formula aid per current-year pupil, under whatever changes were applied.
    FormulaAid,
}

/// The two sides of the guarantee test, as a sextile profile in logs.
///
/// A district is on the guarantee when `[H2] > [H]`. Divide each side by its own pupil count and
/// the test separates into three terms that reconstruct it exactly — the identity
/// [`crate::guarantee_origin`] states, extended to the districts the guarantee does not pay:
///
/// ```text
/// ln([H2] / [H]) = ln(pupils FY2020 / pupils now) + ln(anchor per pupil) - ln(aid per pupil)
/// ```
///
/// Every row here is the band's median of one of those terms, **demeaned** against the panel's
/// own median of it. A flat row is a term that does not vary with size.
///
/// The enrolment row is flat — every band inside 0.015 of the panel median, against 0.28 for the
/// anchor — which is what rules out enrolment loss as the residual and is worth having measured
/// rather than assumed.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    /// `ln(pupils FY2020 / pupils now)`, demeaned.
    pub enrollment: [f64; BANDS],
    /// `ln([H2] per FY2020 pupil)`, demeaned.
    pub anchor: [f64; BANDS],
    /// `ln(formula aid per current-year pupil)`, demeaned.
    pub aid: [f64; BANDS],
    /// The panel median each row is demeaned against, in the same order.
    pub levels: [f64; 3],
    /// How many districts the profile is over.
    ///
    /// 607 of 609: Richmond Heights Local has no positive `[H2]` to take a log of and Buckeye
    /// Local (IRN 047787) is the one district [`guarantee_origin::enrollment_index`] cannot
    /// reach. Carried as a count rather than dropped silently.
    pub districts: usize,
}

/// One district's three terms, before they are reduced to a profile.
struct Terms {
    band: usize,
    enrollment: f64,
    anchor: f64,
    aid: f64,
}

/// Every district the identity can be stated for, under a set of changes.
fn terms(panel: &[DistrictRecord], changes: &[Change]) -> Vec<Terms> {
    let index = guarantee_origin::enrollment_index(panel);
    let after: BTreeMap<String, Dollars> = striking_out(panel, changes)
        .into_iter()
        .map(|row| (row.irn, row.aid_delta))
        .collect();
    let mut out = Vec::new();
    for (band, rows) in sextiles(panel).iter().enumerate() {
        for record in rows {
            if record.current_year_adm <= 0.0 || record.guarantee_floor() <= 0.0 {
                continue;
            }
            let Some(ratio) = index.get(&record.irn) else {
                continue;
            };
            let anchor_pupils = record.adm_history[2] / ratio;
            let aid =
                record.core_foundation_funding + after.get(&record.irn).copied().unwrap_or(0.0);
            if anchor_pupils <= 0.0 || aid <= 0.0 {
                continue;
            }
            out.push(Terms {
                band,
                enrollment: (1.0 / ratio).ln(),
                anchor: (record.guarantee_floor() / anchor_pupils).ln(),
                aid: (aid / record.current_year_adm).ln(),
            });
        }
    }
    out
}

/// The upper-middle value, which is [`dispersion::median`]'s convention and not Python's.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Demean each term's band medians against the panel's own.
///
/// # Panics
///
/// If any band is empty, which for a 609-row panel cut into six cannot happen.
#[must_use]
pub fn profile(panel: &[DistrictRecord], changes: &[Change]) -> Profile {
    let rows = terms(panel, changes);
    let row = |pick: &dyn Fn(&Terms) -> f64| -> ([f64; BANDS], f64) {
        let level = upper_middle(rows.iter().map(pick).collect());
        let mut out = [0.0; BANDS];
        for (band, cell) in out.iter_mut().enumerate() {
            *cell = upper_middle(
                rows.iter()
                    .filter(|r| r.band == band)
                    .map(pick)
                    .collect::<Vec<_>>(),
            ) - level;
        }
        (out, level)
    };
    let (enrollment, e) = row(&|r| r.enrollment);
    let (anchor, a) = row(&|r| r.anchor);
    let (aid, d) = row(&|r| r.aid);
    Profile {
        enrollment,
        anchor,
        aid,
        levels: [e, a, d],
        districts: rows.len(),
    }
}

/// Recount the guarantee with one side's size profile taken out of it.
///
/// Every district keeps its own deviation from its band's median and the band medians are set
/// equal: the quantity stops varying with size and goes on varying within a band. That is the
/// narrowest counterfactual that answers "does this side's size gradient produce the shape",
/// because it removes the gradient and nothing else.
///
/// It is a shape substitution rather than a policy: neither side can be flattened by any act, and
/// the anchor least of all. What it is for is attribution.
///
/// With both size terms struck out, flattening [`Side::Anchor`] gives `[56, 59, 88, 70, 82, 90]`
/// and flattening [`Side::FormulaAid`] gives `[97, 90, 58, 71, 45, 16]` — a rise and a fall, and
/// the observed curve is the two against each other.
#[must_use]
pub fn with_a_side_flattened(
    panel: &[DistrictRecord],
    changes: &[Change],
    side: Side,
) -> [usize; BANDS] {
    let rows = terms(panel, changes);
    let pick = |r: &Terms| match side {
        Side::Anchor => r.anchor,
        Side::FormulaAid => r.aid,
    };
    let level = upper_middle(rows.iter().map(pick).collect());
    let bands: Vec<f64> = (0..BANDS)
        .map(|band| {
            upper_middle(
                rows.iter()
                    .filter(|r| r.band == band)
                    .map(pick)
                    .collect::<Vec<_>>(),
            ) - level
        })
        .collect();
    let mut counts = [0usize; BANDS];
    for row in &rows {
        let adjust = bands[row.band];
        let total = match side {
            Side::Anchor => row.enrollment + (row.anchor - adjust) - row.aid,
            Side::FormulaAid => row.enrollment + row.anchor - (row.aid - adjust),
        };
        if total > 0.0 {
            counts[row.band] += 1;
        }
    }
    counts
}

/// The anchor's own per-pupil level in each band, in dollars.
///
/// `[H2]` over the district's FY2020 pupil count, which is its FY2026 count divided by
/// [`guarantee_origin::enrollment_index`]. Declines monotonically: $6,478 to $3,176, the smallest
/// band at 2.04x the largest.
///
/// Taking it against *current-year* ADM instead needs no index at all and gives the same shape —
/// $7,257 to $3,334, a ratio of 2.18 — so the gradient is not an artifact of the chained index,
/// which is the one measurement choice [`crate::guarantee_origin`] is sensitive to.
#[must_use]
pub fn anchor_per_pupil(panel: &[DistrictRecord]) -> [Dollars; BANDS] {
    let index = guarantee_origin::enrollment_index(panel);
    let mut out = [0.0; BANDS];
    for (band, rows) in sextiles(panel).iter().enumerate() {
        out[band] = upper_middle(
            rows.iter()
                .filter(|record| record.guarantee_floor() > 0.0)
                .filter_map(|record| {
                    let ratio = index.get(&record.irn)?;
                    Some(record.guarantee_floor() / (record.adm_history[2] / ratio))
                })
                .collect(),
        );
    }
    out
}

/// Median local capacity growth from the FY2026 model to the FY2027 one, in each band.
///
/// The published `[b1]` on both sides — [`crate::prior_model::Prior::capacity_per_pupil`] against
/// [`DistrictRecord::published_capacity_per_pupil`] — rather than the figure implied by
/// subtracting aid from base cost, which is `None` wherever the minimum state share binds and so
/// would select on exactly the districts this is about.
///
/// Flat: 10.01%, 9.09%, 9.76%, 9.01%, 9.54%, 8.84%.
#[must_use]
pub fn capacity_growth_by_sextile(panel: &[DistrictRecord]) -> [f64; BANDS] {
    let prior = crate::prior_model::by_irn();
    let mut out = [0.0; BANDS];
    for (band, rows) in sextiles(panel).iter().enumerate() {
        out[band] = upper_middle(
            rows.iter()
                .filter_map(|record| {
                    let was = prior.get(&record.irn)?.capacity_per_pupil;
                    let now = record.published_capacity_per_pupil?;
                    (was > 0.0).then(|| now / was - 1.0)
                })
                .collect(),
        );
    }
    out
}
