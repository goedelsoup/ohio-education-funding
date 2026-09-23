//! The two terms for every district, and who among the districts that lost pupils the guarantee
//! holds — the partition that did not need fitting.
//!
//! Issue #396 recorded what the arc that began at [`crate::guarantee_origin`] never attempted:
//! a multivariate partition of all 609 districts, unconditioned on guarantee membership. The
//! case for it was that every partition so far starts from the guarantee, so a blind spot that
//! does not put a district on the floor would be invisible to all of them. The case against was
//! that the identity had already found, interpretably, what a clustering run would have found for
//! the 187. Neither side had been measured.
//!
//! This module measures both. The identity is extended to every district it can reach, which is
//! **607 of 609**, and a fitted partition is run once on six profile variables — in
//! [`dispersion::partition`], against known answers first — to see whether it finds anything
//! the identity does not. The decision that follows is recorded in
//! [`the-partition-nobody-fitted`](../../../.yidam/decisions/the-partition-nobody-fitted.yml).
//!
//! # 1. The identity does not need the guarantee
//!
//! `[H2] − [I1]` is published for 608 districts, so `floor / formula` is a multiple for each,
//! below 1.0 where the formula pays more than the FY2020 base. [`guarantee_origin::terms`] splits
//! its logarithm into the enrollment term and the per-pupil term exactly as
//! [`guarantee_origin::decompose`] does for the guaranteed, and the two agree on all 293
//! guaranteed districts both reach. Two districts are unreachable and named in
//! [`Unreached`]: Richmond Heights publishes no funding base, and Buckeye Local (IRN 047787) is
//! the one district the chained enrollment index does not reach.
//!
//! With every district placed, the sign pattern is stark. Of the **314** districts the guarantee
//! does not pay, **252** have fewer pupils than in FY2020, and the FY2027 formula pays more per
//! pupil than the FY2020 regime did for **310** — the four exceptions are districts whose
//! enrollment *grew* enough to cover a smaller per-pupil payment. Nobody is off the guarantee
//! because the formula pays less per child; they are off it because the per-pupil raise outran
//! the pupils they lost.
//!
//! # 2. Among the districts that lost pupils, the guarantee's reach is a wealth gradient
//!
//! **514** of the 607 lost pupils since FY2020. Cut into fifths by published local capacity per
//! pupil, the FY2027 guarantee holds:
//!
//! | fifth by capacity per pupil | districts | held | enrollment cluster | capacity cluster | minimum share | median per-pupil term |
//! |---|--:|--:|--:|--:|--:|--:|
//! | least wealthy | 102 | **15** | 15 | 0 | 0 | −0.274 |
//! | second | 102 | 26 | 25 | 1 | 0 | −0.250 |
//! | third | 102 | 56 | **40** | 16 | 0 | −0.074 |
//! | fourth | 102 | 84 | 7 | **67** | 10 | +0.344 |
//! | wealthiest | 106 | 81 | 2 | 4 | **75** | +0.572 |
//!
//! Read down the held column: 15% of the least-wealthy shrinking districts, 76% of the
//! wealthiest. The per-pupil term crosses zero between the third and fourth fifths, and that
//! crossing is the whole shape. Below it the plan raised per-pupil aid by a quarter or more — the
//! districts it was written to reach — and the raise absorbed the lost pupils, so the floor owes
//! them nothing. Above it capacity outran a frozen cost side and the per-pupil term alone puts
//! them on the floor, lost pupils or not; the same fifths of the 93 *growing* districts read 0, 1,
//! 2, 14, 14 held. The enrollment cluster is the third fifth: the districts for which the raise
//! was just too small to cover the decline, so that the lost pupils tip them over. It is not "the
//! shrinking districts". It is the shrinking districts at the wealth where the plan's per-pupil
//! raise ran out.
//!
//! # 3. The cluster has siblings the floor does not hold, and they are poorer
//!
//! **64** formula districts lost pupils at the enrollment cluster's own median rate or faster —
//! the [`cluster_median_enrollment_term`], 11.9% since FY2020 — and are not on the guarantee.
//! Against the cluster ([`Siblings`]):
//!
//! | | districts | ADM | median econ. disadvantaged (profile) | median capacity/pupil | median state share | median per-pupil term |
//! |---|--:|--:|--:|--:|--:|--:|
//! | enrollment cluster | 89 | 196,869 | 52.2% | $5,184 | 39.6% | −0.043 |
//! | siblings | 64 | 81,392 | **58.2%** | **$4,235** | **49.7%** | **−0.316** |
//!
//! The siblings are poorer, have less capacity, and are paid a higher state share; the formula
//! pays them a median $8,542 a pupil against $5,776 in their FY2020 base. Their formula aid sits a
//! median 17.9% above the floor — $1,201,355 a district, $100.8m in all — and at their own
//! FY2024–FY2026 rate of decline, undamped, the median one reaches it in **4.1 years**; at the
//! shipped damping [`crate::rolling_anchor::walk`] holds **5** of the 64 by FY2032. They are the
//! same population as the cluster on every axis but wealth, and the mechanism the cluster was
//! named for — costs that cannot follow enrollment down, unadjusted by any plan component — is
//! present in them and absorbed by a component that was not designed to absorb it. That is the
//! blind spot outside the guarantee, and it is not a new one: it is the cluster's, seen at a
//! wealth where a different line pays for it.
//!
//! # 4. A fitted partition finds the cluster at no k
//!
//! Six profile variables, none of them a formula output — [`FEATURES`]: log enrolled ADM, the
//! enrollment term, log capacity per pupil, the DPIA share, base cost per pupil, and the
//! fourteen-year enrollment rate — standardized and partitioned by
//! [`dispersion::partition::k_means`] from deterministic seeds, at every `k` from 2 to 9:
//!
//! | | one cell | k = 2 | 4 | 6 | 9 | typology (nine cells) |
//! |---|--:|--:|--:|--:|--:|--:|
//! | ceiling on guarantee membership | 314 | 314 | 379 | 385 | **432** | 384 |
//! | ceiling on the four populations | 314 | 314 | 314 | 314 | 340 | 333 |
//! | ceiling on the cluster against the rest | 518 | 518 | 518 | 518 | 519 | 518 |
//!
//! A ceiling is the most a partition can score with every cell assigned to its majority
//! ([`dispersion::partition::majority_ceiling`]), and the one-cell column is what scoring
//! nothing looks like. On the cluster, no partition up to nine cells beats one cell by more than
//! a single district: at every `k` the cluster is a minority in every cell it lands in. Started
//! from the department's own typology's nine centres instead of from the extremes, the run
//! scores 518 on the cluster, 415 on membership and 362 on the four populations — the same
//! answer. And [`dispersion::partition::nearest_neighbours`] asks the question with nothing
//! fitted: a cluster member's nearest neighbour on the six variables is a member **31 of 89**
//! times, against 89 of 607 by chance — the cluster is a region of the profile, but a thin one
//! that no cell of any partition tried holds a majority of.
//!
//! What the fit does find is membership, weakly: at nine cells a k-means describes who is on the
//! guarantee 48 districts better than the 2013 typology does, and neither reaches three quarters.
//! That is the wealth gradient in section 2, rediscovered without the axis that explains it.
//!
//! # What follows
//!
//! No clustering estimator is a reasoning tool of this corpus. [`dispersion::partition`] stays
//! in the workspace so the table above is reproducible, and every claim built on this module is
//! built on the identity — a cut on wealth among the districts that lost pupils, which a reader
//! can check against the calculator with a sort. The multivariate partition #396 described was
//! run, and what it recovers the identity had already placed on an axis.
//!
//! # What holds the siblings off the floor
//!
//! [`crate::size_incidence`] answers it with [`crate::size_terms`]'s counterfactuals. The two
//! size-dependent terms hold 35 of the 64 off the guarantee — because they are small, not
//! because they are shrinking — and the base cost state share carries the other 29.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};

use crate::guarantee_origin::{self, Decomposition, Origin};
use crate::margin;
use crate::panel::DistrictRecord;
use crate::policy::Policy;

/// Which of the four populations a district is in, on the predicates the arc defined them by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Population {
    /// Paid by the formula: `[I] = 0`.
    Formula,
    /// On the guarantee and at the 10% minimum state share — the 107 the settled file explains.
    MinimumShare,
    /// On the guarantee above the minimum, the enrollment term carrying the majority.
    EnrollmentLoss,
    /// On the guarantee above the minimum, the per-pupil term carrying the majority.
    CapacityGrowth,
}

impl Population {
    /// All four, in the order the tables print them.
    pub const ALL: [Self; 4] = [
        Self::Formula,
        Self::MinimumShare,
        Self::EnrollmentLoss,
        Self::CapacityGrowth,
    ];

    /// The word for it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Formula => "formula",
            Self::MinimumShare => "minimum share",
            Self::EnrollmentLoss => "enrollment cluster",
            Self::CapacityGrowth => "capacity cluster",
        }
    }

    /// Its position in [`Self::ALL`], for a label the partition helpers can count.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Formula => 0,
            Self::MinimumShare => 1,
            Self::EnrollmentLoss => 2,
            Self::CapacityGrowth => 3,
        }
    }

    /// Whether the guarantee pays this population.
    #[must_use]
    pub const fn held(self) -> bool {
        !matches!(self, Self::Formula)
    }
}

/// One district with both terms, wherever it stands.
#[derive(Debug, Clone, PartialEq)]
pub struct Standing {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// Which population, on the arc's own predicates.
    pub population: Population,
    /// The two terms and the multiple they reconstruct, from [`guarantee_origin::terms`].
    pub terms: Decomposition,
    /// Current-year enrolled ADM.
    pub adm: Adm,
    /// `[H]`, what the FY2027 formula computes.
    pub formula: Dollars,
    /// `[H2] − [I1]`, the FY2020 base the guarantee holds to.
    pub floor: Dollars,
    /// Published local capacity per pupil, the wealth axis.
    pub capacity_per_pupil: Dollars,
    /// The state share of base cost.
    pub state_share: f64,
    /// The DPIA share — the blended economically disadvantaged count over enrolled ADM.
    pub poverty: f64,
    /// The profile report's economically disadvantaged share, where it publishes one; the
    /// measure [`guarantee_origin`]'s 52.2% is on.
    pub profile_disadvantaged: Option<f64>,
    /// Base cost per pupil, the need side.
    pub base_cost_per_pupil: Dollars,
    /// The fourteen-year F-33 enrollment rate.
    pub long_run_rate: Option<f64>,
}

impl Standing {
    /// Fewer pupils than in FY2020.
    #[must_use]
    pub fn lost_pupils(&self) -> bool {
        self.terms.enrollment_term > 0.0
    }

    /// How far the formula stands above the floor, in logs. Negative on the guarantee.
    #[must_use]
    pub fn headroom(&self) -> f64 {
        -self.terms.total()
    }
}

/// A district the identity cannot reach, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreached {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// Which input is missing.
    pub why: &'static str,
}

/// Every district the identity reaches, and the ones it does not.
///
/// The second return is carried rather than dropped, for the reason
/// [`guarantee_origin::clusters`] gives: a partition that silently loses members is a count
/// nobody can reproduce.
#[must_use]
pub fn standings(panel: &[DistrictRecord]) -> (Vec<Standing>, Vec<Unreached>) {
    let index = guarantee_origin::enrollment_index(panel);
    let profile: BTreeMap<String, f64> = dispersion::profile::districts()
        .into_iter()
        .filter_map(|p| Some((p.irn.clone(), p.economically_disadvantaged?)))
        .collect();
    let mut reached = Vec::new();
    let mut unreached = Vec::new();
    for record in panel {
        let Some(terms) = guarantee_origin::terms(record, &index) else {
            let why = if record.guarantee_floor() <= 0.0 {
                "no funding base published"
            } else if record.core_foundation_funding <= 0.0 {
                "no formula amount"
            } else {
                "the enrollment index does not reach it"
            };
            unreached.push(Unreached {
                irn: record.irn.clone(),
                name: record.name.clone(),
                why,
            });
            continue;
        };
        let population = if !record.on_guarantee() {
            Population::Formula
        } else if record.at_minimum_state_share() {
            Population::MinimumShare
        } else {
            match terms.origin() {
                Some(Origin::EnrollmentLoss) => Population::EnrollmentLoss,
                Some(Origin::CapacityGrowth) => Population::CapacityGrowth,
                // On the guarantee is `[I] > 0`, which is the floor above the formula, which is a
                // multiple above one. Anything else is a panel the arc's predicates do not fit.
                None => unreachable!("a guaranteed district has a multiple above one"),
            }
        };
        reached.push(Standing {
            irn: record.irn.clone(),
            name: record.name.clone(),
            population,
            terms,
            adm: record.current_year_adm,
            formula: record.core_foundation_funding,
            floor: record.guarantee_floor(),
            capacity_per_pupil: record.published_capacity_per_pupil.unwrap_or(0.0),
            state_share: record.state_share_fraction(),
            poverty: record.dpia.percentage,
            profile_disadvantaged: profile.get(&record.irn).copied(),
            base_cost_per_pupil: record.base_cost_per_pupil,
            long_run_rate: record.long_run_enrollment_rate,
        });
    }
    (reached, unreached)
}

/// The upper-middle value, which is the crates' convention and not Python's.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    if values.is_empty() {
        f64::NAN
    } else {
        values[values.len() / 2]
    }
}

/// The enrollment cluster's median enrollment term — the rate of loss its typical member is on
/// the floor for, and the threshold [`siblings`] is cut at.
///
/// A median rather than a chosen constant, so that "at the cluster's own rate" is what the words
/// say. `ln(1 / 0.8808)`, 11.9% since FY2020.
#[must_use]
pub fn cluster_median_enrollment_term(rows: &[Standing]) -> f64 {
    upper_middle(
        rows.iter()
            .filter(|s| s.population == Population::EnrollmentLoss)
            .map(|s| s.terms.enrollment_term)
            .collect(),
    )
}

/// The formula districts that lost pupils at the cluster's median rate or faster.
#[must_use]
pub fn siblings(rows: &[Standing]) -> Vec<&Standing> {
    let threshold = cluster_median_enrollment_term(rows);
    rows.iter()
        .filter(|s| s.population == Population::Formula && s.terms.enrollment_term >= threshold)
        .collect()
}

/// One fifth of a population on the wealth axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Fifth {
    /// 1 for the least wealthy, 5 for the wealthiest.
    pub rank: usize,
    /// Districts in the band.
    pub districts: usize,
    /// How many of them the guarantee holds.
    pub held: usize,
    /// How many of each [`Population`], in [`Population::ALL`] order.
    pub by_population: [usize; 4],
    /// Upper-middle capacity per pupil.
    pub median_capacity: Dollars,
    /// Upper-middle per-pupil term. The sign is the finding.
    pub median_per_pupil_term: f64,
    /// Upper-middle enrollment term.
    pub median_enrollment_term: f64,
    /// Upper-middle state share.
    pub median_state_share: f64,
}

/// The names the five bands are written by, least wealthy first.
///
/// Here rather than in each caller because a band is quoted by name — `figures.json` pins
/// `lost-pupils-third-fifth-held`, the prose says "the third fifth", and a drawing captions its
/// panel — and three spellings of one cut is how two of them come to disagree.
pub const FIFTHS: [&str; 5] = ["least wealthy", "second", "third", "fourth", "wealthiest"];

/// Cut a population into fifths by published capacity per pupil, least wealthy first, and keep
/// the **membership** rather than only what each band aggregates to.
///
/// Equal counts with the remainder in the wealthiest band, which is how
/// [`crate::anchor_incidence::by`] cuts its bands, so a fifth here and a fifth there are the same
/// fifth. [`by_capacity`] is this function summarised, and goes through it rather than beside it:
/// a drawing that cut its own fifths would be a second cut nobody could see disagreeing with the
/// first.
#[must_use]
pub fn fifths<'a>(rows: &[&'a Standing]) -> [Vec<&'a Standing>; 5] {
    let mut sorted: Vec<&'a Standing> = rows.to_vec();
    sorted.sort_by(|a, b| a.capacity_per_pupil.total_cmp(&b.capacity_per_pupil));
    let width = sorted.len() / 5;
    std::array::from_fn(|index| {
        if index == 4 {
            sorted[index * width..].to_vec()
        } else {
            sorted[index * width..(index + 1) * width].to_vec()
        }
    })
}

/// What each fifth of a population aggregates to, least wealthy first.
///
/// The cut is [`fifths`]'s.
#[must_use]
pub fn by_capacity(rows: &[&Standing]) -> [Fifth; 5] {
    let bands = fifths(rows);
    std::array::from_fn(|index| {
        let slice = &bands[index];
        let mut by_population = [0usize; 4];
        for s in slice {
            by_population[s.population.index()] += 1;
        }
        Fifth {
            rank: index + 1,
            districts: slice.len(),
            held: slice.iter().filter(|s| s.population.held()).count(),
            by_population,
            median_capacity: upper_middle(slice.iter().map(|s| s.capacity_per_pupil).collect()),
            median_per_pupil_term: upper_middle(
                slice.iter().map(|s| s.terms.per_pupil_term).collect(),
            ),
            median_enrollment_term: upper_middle(
                slice.iter().map(|s| s.terms.enrollment_term).collect(),
            ),
            median_state_share: upper_middle(slice.iter().map(|s| s.state_share).collect()),
        }
    })
}

/// The districts that lost pupils since FY2020, or the ones that did not.
#[must_use]
pub fn who(rows: &[Standing], lost: bool) -> Vec<&Standing> {
    rows.iter().filter(|s| s.lost_pupils() == lost).collect()
}

/// What one population looks like on the axes the cluster was described on.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    /// Districts.
    pub districts: usize,
    /// Their current-year enrolled ADM.
    pub adm: Adm,
    /// Upper-middle profile-report economically disadvantaged share, over those publishing one.
    pub median_disadvantaged: f64,
    /// Upper-middle DPIA share.
    pub median_poverty: f64,
    /// Upper-middle capacity per pupil.
    pub median_capacity: Dollars,
    /// Upper-middle state share.
    pub median_state_share: f64,
    /// Upper-middle per-pupil term.
    pub median_per_pupil_term: f64,
    /// Upper-middle enrollment term.
    pub median_enrollment_term: f64,
}

/// Describe a population.
#[must_use]
pub fn profile(rows: &[&Standing]) -> Profile {
    Profile {
        districts: rows.len(),
        adm: rows.iter().map(|s| s.adm).sum(),
        median_disadvantaged: upper_middle(
            rows.iter()
                .filter_map(|s| s.profile_disadvantaged)
                .collect(),
        ),
        median_poverty: upper_middle(rows.iter().map(|s| s.poverty).collect()),
        median_capacity: upper_middle(rows.iter().map(|s| s.capacity_per_pupil).collect()),
        median_state_share: upper_middle(rows.iter().map(|s| s.state_share).collect()),
        median_per_pupil_term: upper_middle(rows.iter().map(|s| s.terms.per_pupil_term).collect()),
        median_enrollment_term: upper_middle(
            rows.iter().map(|s| s.terms.enrollment_term).collect(),
        ),
    }
}

/// The siblings' distance from the floor, in dollars, per pupil and in years.
#[derive(Debug, Clone, PartialEq)]
pub struct Siblings {
    /// The population, described.
    pub profile: Profile,
    /// Formula aid over the floor, summed.
    pub headroom: Dollars,
    /// Upper-middle formula aid over the floor, per district.
    pub median_headroom: Dollars,
    /// Upper-middle formula aid over the floor, per pupil.
    pub median_headroom_per_pupil: Dollars,
    /// Upper-middle formula aid over the FY2020 base, as a ratio.
    pub median_formula_over_floor: f64,
    /// Upper-middle formula aid per current pupil.
    pub median_formula_per_pupil: Dollars,
    /// Upper-middle FY2020 base per FY2020 pupil, through the enrollment index.
    pub median_base_per_pupil: Dollars,
    /// How many are still losing pupils on the FY2024–FY2026 rate.
    pub falling: usize,
    /// Upper-middle years to the floor at that rate, undamped, over the falling.
    pub median_years_to_floor: f64,
}

/// The siblings' headroom, from [`margin::headroom`] at current law.
///
/// # Panics
///
/// If a sibling is missing from the headroom table, which a formula district cannot be.
#[must_use]
pub fn siblings_headroom(panel: &[DistrictRecord], siblings: &[&Standing]) -> Siblings {
    let table: BTreeMap<String, margin::Headroom> = margin::headroom(panel, &Policy::current_law())
        .into_iter()
        .map(|h| (h.irn.clone(), h))
        .collect();
    let rows: Vec<(&Standing, &margin::Headroom)> = siblings
        .iter()
        .map(|s| {
            (
                *s,
                table.get(&s.irn).expect("a formula district has headroom"),
            )
        })
        .collect();
    let years: Vec<f64> = rows.iter().filter_map(|(_, h)| h.years).collect();
    Siblings {
        profile: profile(siblings),
        headroom: rows.iter().map(|(_, h)| h.gap).sum(),
        median_headroom: upper_middle(rows.iter().map(|(_, h)| h.gap).collect()),
        median_headroom_per_pupil: upper_middle(rows.iter().map(|(s, h)| h.gap / s.adm).collect()),
        median_formula_over_floor: upper_middle(
            siblings.iter().map(|s| s.headroom().exp()).collect(),
        ),
        median_formula_per_pupil: upper_middle(
            siblings.iter().map(|s| s.formula / s.adm).collect(),
        ),
        // Per FY2020 pupil: the current count divided by the index is the anchor-year count.
        median_base_per_pupil: upper_middle(
            siblings
                .iter()
                .map(|s| s.floor / (s.adm / s.terms.enrollment_index))
                .collect(),
        ),
        falling: years.len(),
        median_years_to_floor: upper_middle(years),
    }
}

/// The six profile variables the fitted partition runs on, in column order.
///
/// None is a formula output. The enrollment term is the same rate the identity uses, and is
/// here because a partition asked to find a cluster defined partly by enrollment should be given
/// enrollment.
pub const FEATURES: [&str; 6] = [
    "log enrolled ADM",
    "enrollment term, FY2020 to FY2026",
    "log capacity per pupil",
    "DPIA share",
    "base cost per pupil",
    "fourteen-year enrollment rate",
];

/// The feature columns, and which rows publish all six.
///
/// Every row does at present, so the index vector is the identity; it is returned so that a
/// panel where that stops being true drops rows visibly rather than misaligning labels.
#[must_use]
pub fn features(rows: &[Standing]) -> (Vec<usize>, Vec<Vec<f64>>) {
    let mut kept = Vec::new();
    let mut columns: Vec<Vec<f64>> = vec![Vec::new(); FEATURES.len()];
    for (i, s) in rows.iter().enumerate() {
        let Some(rate) = s.long_run_rate else {
            continue;
        };
        if s.adm <= 0.0 || s.capacity_per_pupil <= 0.0 {
            continue;
        }
        kept.push(i);
        columns[0].push(s.adm.ln());
        columns[1].push(s.terms.enrollment_term);
        columns[2].push(s.capacity_per_pupil.ln());
        columns[3].push(s.poverty);
        columns[4].push(s.base_cost_per_pupil);
        columns[5].push(rate);
    }
    (kept, columns)
}

/// The most a partition can describe of three labellings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ceilings {
    /// Rows scored.
    pub districts: usize,
    /// Guarantee membership: held or not.
    pub membership: usize,
    /// The four [`Population`]s.
    pub four_way: usize,
    /// The enrollment cluster against everyone else.
    pub cluster: usize,
    /// How many rows are in the cluster — so `districts - cluster_members` is what one cell
    /// scores on the third line, and the number every other score is read against.
    pub cluster_members: usize,
}

/// Score a cell assignment against the three labellings.
#[must_use]
pub fn ceilings(assignment: &[usize], rows: &[&Standing]) -> Ceilings {
    let membership: Vec<usize> = rows
        .iter()
        .map(|s| usize::from(s.population.held()))
        .collect();
    let four_way: Vec<usize> = rows.iter().map(|s| s.population.index()).collect();
    let cluster: Vec<usize> = rows
        .iter()
        .map(|s| usize::from(s.population == Population::EnrollmentLoss))
        .collect();
    Ceilings {
        districts: rows.len(),
        membership: dispersion::partition::majority_ceiling(assignment, &membership),
        four_way: dispersion::partition::majority_ceiling(assignment, &four_way),
        cluster: dispersion::partition::majority_ceiling(assignment, &cluster),
        cluster_members: cluster.iter().sum(),
    }
}

/// One fitted partition, scored.
#[derive(Debug, Clone, PartialEq)]
pub struct Fitted {
    /// Cells.
    pub k: usize,
    /// What it can describe.
    pub ceilings: Ceilings,
    /// Cells holding one district.
    pub singletons: usize,
    /// The objective at the fixed point.
    pub inertia: f64,
    /// Passes to reach it.
    pub iterations: usize,
}

/// The standardized feature rows and the standings they belong to.
fn design(rows: &[Standing]) -> (Vec<&Standing>, Vec<Vec<f64>>) {
    let (kept, columns) = features(rows);
    let points = dispersion::partition::standardize(&columns)
        .expect("six columns with variation over hundreds of districts");
    (kept.iter().map(|&i| &rows[i]).collect(), points)
}

/// Run the partition at `k` from farthest-first seeds and score it.
///
/// # Panics
///
/// If `k` exceeds the rows, which no `k` a reader would ask for does.
#[must_use]
pub fn fitted(rows: &[Standing], k: usize) -> Fitted {
    let (kept, points) = design(rows);
    let seeds = dispersion::partition::farthest_first(&points, k).expect("k within the rows");
    let fit = dispersion::partition::k_means(&points, &seeds).expect("seeded and sized");
    Fitted {
        k,
        ceilings: ceilings(&fit.assignment, &kept),
        singletons: fit.singletons(),
        inertia: fit.inertia,
        iterations: fit.iterations,
    }
}

/// The department's typology as a labelling of the rows: its code, or a tenth label for a
/// district it does not classify.
fn typology_cells(kept: &[&Standing]) -> Vec<usize> {
    let assignments = dispersion::typology::by_irn();
    kept.iter()
        .map(|s| {
            assignments
                .get(&s.irn)
                .map_or(9, |t| usize::from(t.typology.code()))
        })
        .collect()
}

/// The typology scored as a partition — the baseline to beat.
#[must_use]
pub fn typology_ceilings(rows: &[Standing]) -> Ceilings {
    let (kept, _) = design(rows);
    ceilings(&typology_cells(&kept), &kept)
}

/// Run the partition from the typology's own centres and score it.
///
/// The seeds are the centroid of each typology cell on the six variables, so the run starts
/// from the baseline rather than from the extremes; `k` is the number of cells the typology
/// uses over these rows.
#[must_use]
pub fn fitted_from_typology(rows: &[Standing]) -> Fitted {
    let (kept, points) = design(rows);
    let cells = typology_cells(&kept);
    let k = cells.iter().copied().max().map_or(0, |c| c + 1);
    let seeds = dispersion::partition::centres_of(&points, &cells, k).expect("one label per row");
    let fit = dispersion::partition::k_means(&points, &seeds).expect("seeded and sized");
    Fitted {
        k,
        ceilings: ceilings(&fit.assignment, &kept),
        singletons: fit.singletons(),
        inertia: fit.inertia,
        iterations: fit.iterations,
    }
}

/// Whether each district's nearest neighbour on the six variables shares its population.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Neighbours {
    /// Rows.
    pub districts: usize,
    /// Rows whose neighbour is held if and only if they are.
    pub membership: usize,
    /// Per [`Population`], in [`Population::ALL`] order: members whose neighbour is a member, and
    /// members.
    pub by_population: [(usize, usize); 4],
}

/// The unfitted question: is a population a region of the profile at all?
#[must_use]
pub fn neighbours(rows: &[Standing]) -> Neighbours {
    let (kept, points) = design(rows);
    let nearest = dispersion::partition::nearest_neighbours(&points).expect("more than one row");
    let mut by_population = [(0usize, 0usize); 4];
    let mut membership = 0;
    for (i, s) in kept.iter().enumerate() {
        let other = kept[nearest[i]];
        let cell = &mut by_population[s.population.index()];
        cell.1 += 1;
        if other.population == s.population {
            cell.0 += 1;
        }
        if other.population.held() == s.population.held() {
            membership += 1;
        }
    }
    Neighbours {
        districts: kept.len(),
        membership,
        by_population,
    }
}
