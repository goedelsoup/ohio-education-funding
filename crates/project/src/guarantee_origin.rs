//! Why a district is on the guarantee, when the minimum state share is not the answer.
//!
//! `the_minimum_share_the_guarantee_has_already_paid_past.rs` settled one half of the temporary
//! transitional aid guarantee. **107 of the 294 districts it pays sit on the 10% minimum state
//! share**, are held at a median 2.40x their formula amount, and hold $410.6m — 46.7% of the
//! instrument. For them the minimum is dead letter and the guarantee is preserved wealth.
//!
//! This module is about the other **187**, which hold **$468,337,128.87** and **465,086.26** of
//! current-year enrolled ADM. Nothing in the corpus explained them, and the reason the wealth
//! account does not reach them is that most of them are not wealthy.
//!
//! # The population, and the choice behind it
//!
//! `294 - 107 = 187` is arithmetic. The predicate is
//! [`DistrictRecord::on_guarantee`] **and not** [`DistrictRecord::at_minimum_state_share`], which
//! is the exact complement of the settled population *on the same two predicates it was defined
//! by*. See [`above_the_minimum`].
//!
//! Two alternatives were available and are not used. "Guaranteed and not *explained* by the
//! minimum share" needs a threshold on explanation and would make the population a function of
//! that threshold. And the any-device predicate [`crate::hold_harmless::held`] is wider — 326
//! districts, of whom **32 draw `[K]` or `[F]` without being on the guarantee at all** — so a
//! population built on it would mix in districts this instrument does not pay. The guarantee is
//! the instrument under examination, so `[I] > 0` is the membership test.
//!
//! # The multiple factors exactly, and that is the partition
//!
//! A guaranteed district is paid `[H2] - [I1]`, so its multiple over formula aid is
//!
//! ```text
//! multiple = (FY2020 base) / (FY2027 formula aid)
//! ```
//!
//! Divide each side by its own pupil count and the multiple separates into two factors that
//! reconstruct it exactly:
//!
//! ```text
//! ln(multiple) = ln(pupils FY2020 / pupils now)  +  ln(aid per pupil FY2020 / aid per pupil now)
//!                └──── the enrollment term ────┘    └──────── the per-pupil term ────────────┘
//! ```
//!
//! This is an identity, not a fit: [`Decomposition::total`] equals `ln(multiple)` to floating
//! point for every district, and [`Decomposition::enrollment_share`] is the fraction of it the
//! first term carries. **No estimator was added to this workspace** — see the note on clustering
//! below.
//!
//! The two terms are two different things the state could do something about, and they point
//! opposite ways:
//!
//! - **The enrollment term** is the district having fewer children than in FY2020, against an
//!   anchor that does not move. The formula's need side shrank; the floor did not.
//! - **The per-pupil term** is the FY2027 formula paying less for each child than the FY2020
//!   regime paid. On this population that is the state share falling as local capacity outruns a
//!   frozen cost side — see [`crate::prior_model`], where capacity moves **+9.34%** in one year
//!   against a base cost that moves **+0.05%**.
//!
//! # The 187 are two populations, and they need opposite responses
//!
//! Split on which term carries the majority of `ln(multiple)` — a majority, not a tuned constant:
//!
//! | | districts | guarantee | median multiple | median state share | median capacity/pupil | median index | median econ. disadvantaged |
//! |---|--:|--:|--:|--:|--:|--:|--:|
//! | [`Origin::EnrollmentLoss`] | 89 | $108,568,921.47 | 1.0813 | 39.6% | $5,184 | 0.8808 | 52.2% |
//! | [`Origin::CapacityGrowth`] | 98 | $359,768,207.40 | 1.6108 | 18.9% | $6,730 | 0.9239 | 41.6% |
//!
//! They are nearly disjoint on state share — **83 of the 89 above 25%, 77 of the 98 below it** —
//! and a single cut at 25% reproduces the partition on **160 of 187**. That is not a redundancy: it
//! says the same instrument is doing two different jobs at the two ends of one axis the formula
//! already computes, which is what makes the conflation legible and fixable.
//!
//! **[`Origin::CapacityGrowth`] is preserved advantage and is the 107's own future.** **90 of the
//! 98** were already on the guarantee in the FY2026 model, their state share is falling 6.6 points
//! a year, and at that rate the median one reaches the 10% floor in **1.37 years** — against 5.20
//! for the other cluster — at which point it joins the 107 and the minimum share becomes dead
//! letter for it too. The guarantee is
//! holding it at a FY2020 level the formula has deliberately moved away from. Retiring the
//! instrument is the response, and [`crate::hold_harmless::absorption`] says what `[K]` would do
//! with 90.9% of it.
//!
//! **[`Origin::EnrollmentLoss`] is a candidate blind spot.** These districts are poor — 52.2%
//! economically disadvantaged against a statewide model in which the guarantee runs to the
//! wealthy — they have lost **11.9%** of their enrollment since the anchor year, and their median
//! multiple is 1.081: the guarantee is not preserving an advantage for them, it is absorbing a
//! contraction. For **61** of them, holding **$62,532,245.73**, the per-pupil term is
//! **negative** — the FY2027 formula pays *more* for each child than the FY2020 regime did, and
//! they are on the guarantee for no reason but having fewer children. That statement needs no
//! threshold at all.
//!
//! # What the formula fails to measure for them
//!
//! Not need: [`DistrictRecord::base_cost_per_pupil`] is flat across the wealth distribution, and
//! flat across this population's ADM quartiles too — $8,840 / $8,256 / $8,117 / $8,128 against a
//! multiple of 1.244 / 1.299 / 1.282 / 1.220, which is no gradient at all.
//!
//! What it fails to measure is **how fast a district's costs can follow its enrollment down**.
//! The only smoothing in the need side is the three-year average in
//! [`DistrictRecord::base_cost_adm`], which spreads a loss over three years and then finishes.
//! Buildings, the [`foundation`] staffing minimums, and transportation routes do not follow on
//! that schedule.
//!
//! And the formula is **asymmetric about this on its face**. `[M] Enrollment Growth Supplement`
//! pays [`crate::panel::ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL`] — $250 a pupil — to a district
//! whose enrollment rose by [`crate::panel::ENROLLMENT_GROWTH_THRESHOLD`], 3%. There is no
//! counterpart for a district whose enrollment fell by 12%. The plan has an explicit adjustment
//! for one direction of enrollment change and none for the other, and what is catching the other
//! direction is a temporary instrument that Ohio describes as transitional and that R.C. 3317.019
//! now dates to FY2027. `the_supplements_outside_the_formula.rs` records `[M]` as "a need
//! adjustment, not a floor"; this is the same sentence read from the other side.
//!
//! # What does not separate them
//!
//! **The staffing-minimum cliff does not appear inside this population.** It is real in the
//! statewide rate — the guarantee reaches 16.8% of the smallest ADM sextile and **69.3%** of the
//! third, at a median 1,145 ADM — but among the 187 the multiple is flat across ADM quartiles
//! (above) and so is the origin split. The cliff decides *whether* a small district is caught,
//! not *why* one of the 187 is.
//!
//! **Resident-versus-enrolled capacity does not separate them either.** The divergence
//! `local-capacity`'s `PupilCount` names is present — the ratio of Table SD-1's resident ADM to the
//! profile report's enrolled ADM is 1.0951 for [`Origin::EnrollmentLoss`] against 1.0532 for the
//! districts the guarantee does not pay — but it is a few points, not the factor of two that type's
//! documentation warns about, and it does not order the two clusters apart from each other
//! (1.0951 against 1.0595, with the 107 at 1.0768 between them). The channel that would settle it
//! per district is the voucher and community-school deduct, which
//! [`crates/README.md`](../../README.md) records as unbuilt and unreachable.
//!
//! **ODE's own typology does not recover the split.** Its best possible accuracy on this
//! population, assigning every one of the nine categories to whichever cluster is larger within
//! it, is **130 of 187 — 69.5%**, against 85.6% for a single cut on state share. It is a 2013
//! vintage and it is a description of districts rather than of what the formula does to them.
//!
//! # No clustering estimator was added
//!
//! The brief for this work left the choice open, and the decomposition above is the reason not to
//! take it. The partition here is an **identity plus a majority**: the two terms multiply back to
//! the district's own multiple, so there is nothing to fit, no feature scaling to defend, and no
//! initialization to seed. A k-means over standardized features would have produced the same two
//! groups with an extra layer of choices between the data and the claim, and the output of this
//! module is meant to inform a policy design rather than to fit one.
//!
//! What was checked instead is that the partition beats the baseline the corpus already holds,
//! which is the typology above, and that it survives the measurement choice below.
//!
//! Issue #396 then asked the statewide form of the question: whether a multivariate partition
//! across all 609 districts, unconditioned on the guarantee, would find a blind spot none of the
//! guarantee-based partitions can see. [`crate::lost_pupils`] answers it — the identity extends
//! to 607 districts through [`terms`], the candidate blind spot above turns out to be the third
//! wealth fifth of the districts that lost pupils, with 64 poorer siblings the plan's per-pupil
//! raise carried over the floor, and a fitted partition run once on six profile variables holds
//! a majority of the cluster in no cell at any `k`. The estimator that showed so is
//! [`dispersion::partition`], and no claim reasons from its output.
//!
//! # The enrollment index, and why it is chained
//!
//! The anchor is FY2020 and the department's own enrolled ADM series
//! ([`crate::counts::adm_history`]) begins at FY2023. The only Ohio enrollment observation that
//! reaches FY2020 is the F-33 survey's `V33` fall membership, which stops at FY2024 and is **not
//! the same measure** as enrolled ADM.
//!
//! So [`enrollment_index`] carries a **rate** across the seam and never a count, which is the rule
//! [`dispersion::ohio_panel::long_run_enrollment_rates`] states for exactly this pair of series:
//!
//! | leg | source | measure | span |
//! |---|---|---|---|
//! | first | [`dispersion::ohio_panel`] | `V33` fall membership | FY2020 - FY2024 |
//! | second | [`DistrictRecord::adm_history`] | enrolled ADM | FY2024 - FY2026 |
//!
//! The product is an index of FY2026 enrollment on FY2020, and it is the ratio of two ratios
//! rather than of two counts.
//!
//! **This is the one measurement choice the partition is sensitive to, and it is stated rather
//! than buried.** Ending the index at the funded three-year average instead of at FY2026 moves **8**
//! of the 187 and the split reads 81/106 instead of 89/98. Ending it at FY2024 — dropping the
//! second leg, and so measuring four years of a six-year interval — moves **29**, and the split
//! reads 60/127. The first is a robustness check and passes; the second is a shorter span
//! answering a different question, and it is reported here so that a reader who prefers it can
//! see what it costs. Every claim above that does not depend on the cut — the dollars, the state
//! share disjointness, the trajectory to the floor, the typology ceiling — is unaffected by all
//! three.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};

use crate::panel::DistrictRecord;
use crate::prior_model;

/// The fiscal year the guarantee anchors to, and the first leg's start.
pub const ANCHOR: u16 = 2020;

/// The last year the F-33 survey reaches, where the two legs are joined.
pub const SEAM: u16 = 2024;

/// Which of the two terms carries the majority of a district's log multiple.
///
/// A majority, so the boundary is `1/2` and nothing was chosen. A district whose multiple is
/// exactly 1.0 has no log multiple to divide and cannot be classified; [`decompose`] returns
/// `None` for it rather than assigning it a side, and no district in the population is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Origin {
    /// The district has fewer pupils than in FY2020, against an anchor that does not move.
    ///
    /// A candidate blind spot: the plan adjusts explicitly for enrollment *growth* and not at
    /// all for decline, and the guarantee is silently carrying the other direction.
    EnrollmentLoss,
    /// The FY2027 formula pays less per pupil than the FY2020 regime did, because local capacity
    /// has outrun a frozen cost side and the state share has fallen with it.
    ///
    /// Preserved advantage, and the same phenomenon as the 107 caught a year or two earlier.
    CapacityGrowth,
}

impl Origin {
    /// The word for it, for a table that has to name it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EnrollmentLoss => "enrollment loss",
            Self::CapacityGrowth => "capacity growth",
        }
    }

    /// Whether this origin is a floor preserving an advantage the formula has moved away from.
    ///
    /// The distinction the whole module exists to draw: a device that preserves advantage and a
    /// device that compensates for something the formula fails to measure need **opposite**
    /// responses, and in FY2027 they are one instrument.
    #[must_use]
    pub const fn is_preserved_advantage(self) -> bool {
        matches!(self, Self::CapacityGrowth)
    }
}

/// The guaranteed districts the minimum state share does not account for.
///
/// `[I] > 0` and not at the floor — the exact complement, on the same two predicates, of the
/// population settled at
/// `scenario-delta/tests/the_minimum_share_the_guarantee_has_already_paid_past.rs`.
#[must_use]
pub fn above_the_minimum(panel: &[DistrictRecord]) -> Vec<&DistrictRecord> {
    panel
        .iter()
        .filter(|r| r.on_guarantee() && !r.at_minimum_state_share())
        .collect()
}

/// FY2026 enrollment as an index on FY2020, keyed by IRN.
///
/// Chained across two measures, as the module note sets out: the F-33 survey's fall membership
/// from [`ANCHOR`] to [`SEAM`], times the department's own enrolled ADM from [`SEAM`] to FY2026.
/// Both legs are ratios, so nothing compares a count from one series against a count from the
/// other.
///
/// A district the survey does not reach in both years is **absent** rather than carried at 1.0,
/// which would be a claim that its enrollment held steady. One district is absent: Buckeye Local
/// (IRN 047787), which is why this is keyed on IRN — there are three Buckeye Locals, and two of
/// them are present.
#[must_use]
pub fn enrollment_index(panel: &[DistrictRecord]) -> BTreeMap<String, f64> {
    // Resolved through LEAID the way `long_run_enrollment_rates` does: the survey's IRN column is
    // populated only where the FY2022-23 directory still carries the agency, and a year that
    // predates the directory would otherwise key on an empty string.
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
    // The second leg, from the department's own count, and multiplied in as a ratio so that no
    // enrolled ADM is ever divided by a fall membership.
    panel
        .iter()
        .filter_map(|record| {
            let start = *first.get(&record.irn)?;
            let seam = *last.get(&record.irn)?;
            let [at_seam, _, latest] = record.adm_history;
            if start <= 0.0 || at_seam <= 0.0 {
                return None;
            }
            Some((record.irn.clone(), (seam / start) * (latest / at_seam)))
        })
        .collect()
}

/// One district's multiple, split into the two things that produce it.
///
/// The two terms sum to [`Decomposition::total`] by construction. Nothing here is fitted.
#[derive(Debug, Clone, PartialEq)]
pub struct Decomposition {
    /// The district, keyed the only way district figures may be keyed.
    pub irn: String,
    /// `realized / formula` — what the guarantee holds the district at, over what the FY2027
    /// formula computes for it.
    pub multiple: f64,
    /// FY2026 enrollment on FY2020, from [`enrollment_index`].
    pub enrollment_index: f64,
    /// `ln(1 / index)`: the part of the multiple the district's lost pupils account for.
    pub enrollment_term: f64,
    /// `ln(multiple) - enrollment_term`: the part the formula's lower per-pupil payment accounts
    /// for. **Negative** where the FY2027 formula pays more per pupil than the FY2020 regime did.
    pub per_pupil_term: f64,
}

impl Decomposition {
    /// `ln(multiple)`, which the two terms reconstruct exactly.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.enrollment_term + self.per_pupil_term
    }

    /// The fraction of the log multiple the enrollment term carries.
    ///
    /// Above 1 where the per-pupil term is negative — the enrollment loss alone would have put
    /// the district further above the formula than it actually is.
    #[must_use]
    pub fn enrollment_share(&self) -> f64 {
        self.enrollment_term / self.total()
    }

    /// Which term carries the majority.
    ///
    /// `None` where the district is not held above the formula: a log multiple at or below zero
    /// has no majority to take, and [`terms`] reaches such districts. [`decompose`] never
    /// returns one, so on its output this is always `Some`.
    #[must_use]
    pub fn origin(&self) -> Option<Origin> {
        if self.total() <= 0.0 {
            None
        } else if self.enrollment_share() >= 0.5 {
            Some(Origin::EnrollmentLoss)
        } else {
            Some(Origin::CapacityGrowth)
        }
    }
}

/// The same two terms for **any** district, held above the formula or not.
///
/// The identity does not need the guarantee: `[H2] − [I1]` is published for 608 of 609 districts
/// ([`DistrictRecord::guarantee_floor`]), so `floor / formula` is a multiple for every one of
/// them, below 1.0 where the formula pays more than the FY2020 base and the guarantee owes
/// nothing. Its logarithm still splits into the enrollment term and the per-pupil term, and the
/// split still reconstructs it exactly. For a guaranteed district the floor *is* its realized
/// aid, so this agrees with [`decompose`] wherever both are defined.
///
/// `None` where the floor is unpublished, the formula computes nothing, or [`enrollment_index`]
/// does not reach the district. The negative of [`Decomposition::total`] is then the district's
/// headroom above its floor, in logs. [`crate::lost_pupils`] is the caller.
#[must_use]
pub fn terms(record: &DistrictRecord, index: &BTreeMap<String, f64>) -> Option<Decomposition> {
    let floor = record.guarantee_floor();
    if floor <= 0.0 || record.core_foundation_funding <= 0.0 {
        return None;
    }
    let multiple = floor / record.core_foundation_funding;
    let enrollment_index = *index.get(&record.irn)?;
    if enrollment_index <= 0.0 {
        return None;
    }
    let enrollment_term = (1.0 / enrollment_index).ln();
    Some(Decomposition {
        irn: record.irn.clone(),
        multiple,
        enrollment_index,
        enrollment_term,
        per_pupil_term: multiple.ln() - enrollment_term,
    })
}

/// Split one district's multiple.
///
/// `None` where the district is not held above the formula — there is no multiple to divide — or
/// where [`enrollment_index`] does not reach it. [`terms`] without the first condition.
#[must_use]
pub fn decompose(record: &DistrictRecord, index: &BTreeMap<String, f64>) -> Option<Decomposition> {
    terms(record, index).filter(|split| split.multiple > 1.0)
}

/// One cluster of the population, with what it holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    /// Which mechanism carries the majority for its members.
    pub origin: Origin,
    /// How many districts.
    pub districts: usize,
    /// Their guarantee, in dollars.
    pub dollars: Dollars,
    /// Their current-year enrolled ADM.
    pub adm: Adm,
    /// The upper-middle multiple, on [`dispersion::median`]'s convention.
    pub median_multiple: f64,
    /// The upper-middle state share of base cost.
    pub median_state_share: f64,
    /// The upper-middle [`enrollment_index`].
    pub median_enrollment_index: f64,
    /// How many members the per-pupil term is negative for — the formula pays *more* per pupil
    /// than the FY2020 regime did, and only the lost pupils put them on the floor.
    pub formula_now_pays_more_per_pupil: usize,
}

/// The population's two clusters, [`Origin::EnrollmentLoss`] first, and whatever the index
/// cannot reach.
///
/// The third return is the districts [`decompose`] returns `None` for, carried rather than
/// dropped: a partition that silently loses members is how a cluster count becomes a number
/// nobody can reproduce.
#[must_use]
pub fn clusters(panel: &[DistrictRecord]) -> (Vec<Cluster>, Vec<String>) {
    let index = enrollment_index(panel);
    let mut members: BTreeMap<Origin, Vec<(&DistrictRecord, Decomposition)>> = BTreeMap::new();
    let mut unreached = Vec::new();
    for record in above_the_minimum(panel) {
        match decompose(record, &index) {
            Some(split) => members
                .entry(
                    split
                        .origin()
                        .expect("decompose only returns a multiple above one"),
                )
                .or_default()
                .push((record, split)),
            None => unreached.push(record.irn.clone()),
        }
    }
    let clusters = [Origin::EnrollmentLoss, Origin::CapacityGrowth]
        .into_iter()
        .filter_map(|origin| {
            let rows = members.get(&origin)?;
            Some(Cluster {
                origin,
                districts: rows.len(),
                dollars: rows.iter().map(|(r, _)| r.guarantee).sum(),
                adm: rows.iter().map(|(r, _)| r.current_year_adm).sum(),
                median_multiple: upper_middle(rows.iter().map(|(_, s)| s.multiple).collect()),
                median_state_share: upper_middle(
                    rows.iter().map(|(r, _)| r.state_share_fraction()).collect(),
                ),
                median_enrollment_index: upper_middle(
                    rows.iter().map(|(_, s)| s.enrollment_index).collect(),
                ),
                formula_now_pays_more_per_pupil: rows
                    .iter()
                    .filter(|(_, s)| s.per_pupil_term <= 0.0)
                    .count(),
            })
        })
        .collect();
    (clusters, unreached)
}

/// The upper-middle value, which is [`dispersion::median`]'s convention and not Python's.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// How many years until this district's state share reaches the minimum, at the rate the two
/// committed models show it falling.
///
/// A linear extrapolation of a **one-year** difference, so it is a trajectory and not a forecast:
/// [`crate::prior_model`] is the only interval the Fair School Funding Plan can be observed over,
/// and one interval supports no curvature. It is reported because the question it answers is
/// ordinal — which cluster reaches the floor first — and that ordering does not depend on the
/// functional form.
///
/// `None` where the district is already at the floor, where its share is not falling, or where
/// the FY2026 model does not carry it.
#[must_use]
pub fn years_to_the_floor(record: &DistrictRecord, prior: &prior_model::Prior) -> Option<f64> {
    let now = record.state_share_fraction();
    if record.at_minimum_state_share() {
        return None;
    }
    let fall = prior.state_share_percentage - now;
    (fall > 0.0).then(|| (now - crate::panel::MINIMUM_STATE_SHARE) / fall)
}
