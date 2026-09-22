//! What the plan's two size-dependent terms do to a population defined by decline, and what
//! holds the rest of it off the floor.
//!
//! [`crate::lost_pupils`] placed the enrollment cluster on a wealth axis and found it **64
//! siblings**: formula districts shrinking at the cluster's own median rate or faster, poorer
//! than it, and carried over the floor by a per-pupil term of −0.316 against the cluster's
//! −0.043. It did not say which line carries them. [`crate::size_terms`] priced the two terms of
//! the plan that read enrolment — R.C. 3317.011's staffing floors and R.C. 3317.0217's capacity
//! tier — and cut the panel on size. This module joins the two: both counterfactuals were
//! already computed and both populations were already committed, so nothing here is estimated.
//!
//! # 1. The two terms hold 35 of the 64 off the floor
//!
//! [`table`] runs [`size_terms::striking_out`] and reads the counts off it. Dollars are what the
//! change takes from the population, over the base cost enrolled ADM
//! [`size_terms::sextiles`] orders on — the same denominator, so a per-pupil figure here and a
//! band there are the same per-pupil figure:
//!
//! | struck out | newly guaranteed statewide | of the 64 siblings | worth to the siblings, per pupil | to the cluster |
//! |---|--:|--:|--:|--:|
//! | staffing floors | 46 | 16 | $346 | $123 |
//! | capacity tier | 87 | 23 | $708 | $261 |
//! | both | 136 | **35** | **$1,054** | $384 |
//!
//! The cluster's column is a comparison and not a second finding: the cluster is already on the
//! guarantee, so no member of it is newly anything, and what the column reports is that the two
//! terms are worth **2.7 times as much per pupil** to the siblings as to the districts the floor
//! already holds. The siblings are **35 of the 136** the pair moves statewide — a quarter of a
//! counterfactual that is a different formula for every district, not a measurement of theirs.
//!
//! # 2. It is size that catches them, and the sibling axis adds nothing to it
//!
//! Unconditionally the siblings look like a population the size terms single out: **35 of 64**
//! move against **101 of 251** other formula districts. Conditioned on size they are not.
//! [`by_sextile`] counts the newly guaranteed inside each of [`size_terms::sextiles`]' six
//! bands, siblings against everyone else:
//!
//! | ordered on base cost enrolled ADM | 1 | 2 | 3 | 4 | 5 | 6 |
//! |---|--:|--:|--:|--:|--:|--:|
//! | siblings moved / siblings | 22/25 | 7/10 | 6/10 | 0/5 | 0/11 | 0/3 |
//! | others moved / others | 49/58 | 36/52 | 12/20 | 4/36 | 0/41 | 0/44 |
//!
//! Band for band the two rows are the same rate, and the fourth band runs the other way. Give
//! each band's sibling count that band's *non-sibling* rate and the prediction is
//! [`expected_from_size`]: **34.60** against the 35 observed. The decline that defines the
//! siblings is worth four tenths of a district once size is held.
//!
//! So the answer to "are the 35 a subset the size-term arc's sextile view could not see" is no.
//! They are what that view already says: small districts, at the rate small districts move. What
//! the sextile view cannot see is *which* of its small districts are shrinking — and that is the
//! join, not a correction to it. The siblings are a population because they are poor and
//! shrinking; the two terms reach them because they are small, and the three coincide.
//!
//! # 3. The other 29 are carried by the state share, not by DPIA
//!
//! The 29 the pair does not move are the larger siblings — a median **1,846 ADM** against 636
//! for the 35 — so neither term is binding on them by construction. Their formula aid stands
//! **$68,572,895** over the floor, $1,144 a pupil. [`carried`] takes each component of `[H]`
//! against that gap district by district, and [`covering`] counts the districts a component
//! could cover alone:
//!
//! | component of `[H]` | covers the gap alone | aggregate | per pupil |
//! |---|--:|--:|--:|
//! | base cost state share | **28** of 29 | $208,123,388 | $3,471 |
//! | targeted assistance | 14 | $73,675,045 | $1,229 |
//! | special education | 9 | $49,207,338 | $821 |
//! | DPIA | 4 | $36,497,037 | $609 |
//! | gifted | 1 | $2,821,415 | $47 |
//! | career-technical | 0 | $2,895,915 | $48 |
//! | English learners | 0 | $759,100 | $13 |
//!
//! It is the state share of base cost. DPIA alone reaches **4** of the 29 and is smaller than
//! the gap for the other 25. That is the per-pupil raise itself and not a categorical: these are
//! districts the plan's cost side lifted far enough above an FY2020 base that no one weight is
//! load-bearing.
//!
//! **A component larger than the gap is not a component that pays it.** Every district here is
//! held above the floor by the whole of `[H]`, and striking any one line moves the multiple for
//! every district in the state, not only for these. The column is a reach — what a repeal of that
//! line alone would be enough to do — and the question it answers is which lines are even in the
//! running.
//!
//! # 4. The population the arc calls "the 153"
//!
//! [`cluster_and_siblings`] is the union #435 and #436 name: the enrollment cluster's 89 and the
//! 64, **153** districts. It is not the same set as "districts shrinking at the cluster's median
//! rate or faster", which is [`shrinking_at_the_cluster_rate`] and holds **169** — 64 formula,
//! 45 of the cluster's own 89, 31 at the minimum state share and 29 of the capacity cluster.
//! A median cuts its own population in half, so only 45 of the 89 clear their own threshold, and
//! 60 guaranteed districts outside the cluster shrink at least that fast.
//!
//! Both are defensible populations and they are 153 and 169. Reporting a threshold change on
//! "the 153" means the union, which is a set the guarantee's own membership helps define; on the
//! 169 it means a cut on decline alone. The two answer different questions and the arc has been
//! writing one count for both.

use std::collections::{BTreeMap, BTreeSet};

use edfund_core::{Adm, Dollars};

use crate::lost_pupils::{self, Population, Standing};
use crate::panel::DistrictRecord;
use crate::size_terms::{self, Change, BANDS};

/// The three counterfactuals the table prices, in the order it prints them.
pub const CHANGE_SETS: [&[Change]; 3] = [
    &[Change::StaffingFloorsStruckOut],
    &[Change::CapacityTierStruckOut],
    &[
        Change::StaffingFloorsStruckOut,
        Change::CapacityTierStruckOut,
    ],
];

/// What a set of [`Change`]s does to one population.
#[derive(Debug, Clone, PartialEq)]
pub struct Reach {
    /// Districts in the population the counterfactual reaches.
    pub districts: usize,
    /// How many of them the guarantee already pays, where nothing can be *newly* anything.
    pub already_guaranteed: usize,
    /// How many the changes put on the guarantee.
    pub newly_guaranteed: usize,
    /// Formula aid the changes take from them, summed. Negative for a change that pays.
    pub aid_removed: Dollars,
    /// Their base cost enrolled ADM — the count [`size_terms::sextiles`] orders on, so that a
    /// per-pupil figure here and a band there are the same per-pupil figure.
    pub adm: Adm,
}

impl Reach {
    /// [`Self::aid_removed`] over [`Self::adm`].
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm > 0.0 {
            self.aid_removed / self.adm
        } else {
            0.0
        }
    }
}

/// Run a set of changes and read it off for one population.
///
/// The population is given as standings because that is what [`lost_pupils`] hands back; a
/// district of it the panel does not carry is dropped rather than counted at zero, and
/// [`Reach::districts`] is the count actually reached.
#[must_use]
pub fn reach(panel: &[DistrictRecord], changes: &[Change], population: &[&Standing]) -> Reach {
    let wanted: BTreeSet<&str> = population.iter().map(|s| s.irn.as_str()).collect();
    let adm: BTreeMap<&str, Adm> = panel
        .iter()
        .map(|r| (r.irn.as_str(), r.base_cost_adm()))
        .collect();
    let rows: Vec<size_terms::Counterfactual> = size_terms::striking_out(panel, changes)
        .into_iter()
        .filter(|c| wanted.contains(c.irn.as_str()))
        .collect();
    Reach {
        districts: rows.len(),
        already_guaranteed: rows.iter().filter(|c| c.guaranteed).count(),
        newly_guaranteed: rows.iter().filter(|c| c.newly_guaranteed()).count(),
        aid_removed: rows.iter().map(|c| -c.aid_delta).sum(),
        adm: rows
            .iter()
            .filter_map(|c| adm.get(c.irn.as_str()).copied())
            .sum(),
    }
}

/// One row of the table: a counterfactual, statewide and on the two populations.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// What was struck out.
    pub changes: &'static [Change],
    /// Districts the change puts on the guarantee across all 609.
    pub statewide: usize,
    /// The 64 siblings.
    pub siblings: Reach,
    /// The enrollment cluster's 89, for comparison.
    pub cluster: Reach,
}

/// The three counterfactuals, statewide and on both populations.
///
/// Reads `46 / 87 / 136` statewide and `16 / 23 / 35` of the 64.
#[must_use]
pub fn table(panel: &[DistrictRecord], rows: &[Standing]) -> Vec<Row> {
    let siblings = lost_pupils::siblings(rows);
    let cluster: Vec<&Standing> = rows
        .iter()
        .filter(|s| s.population == Population::EnrollmentLoss)
        .collect();
    CHANGE_SETS
        .iter()
        .map(|changes| Row {
            changes,
            statewide: size_terms::striking_out(panel, changes)
                .iter()
                .filter(|c| c.newly_guaranteed())
                .count(),
            siblings: reach(panel, changes, &siblings),
            cluster: reach(panel, changes, &cluster),
        })
        .collect()
}

/// One ADM band, with the siblings separated from everyone else.
///
/// Both pairs count formula districts only: a district the guarantee already pays cannot be
/// newly guaranteed, and leaving it in the denominator would report the same rate for a
/// population half of which is not at risk of moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    /// 1 for the smallest.
    pub rank: usize,
    /// Siblings the changes move, and siblings in the band.
    pub siblings: (usize, usize),
    /// The same for every other formula district in it.
    pub others: (usize, usize),
}

impl Band {
    /// The non-sibling rate, which is what size alone predicts for this band.
    #[must_use]
    pub fn others_rate(&self) -> f64 {
        if self.others.1 == 0 {
            0.0
        } else {
            self.others.0 as f64 / self.others.1 as f64
        }
    }
}

/// The newly guaranteed by ADM band, siblings against the rest.
///
/// With both terms struck out: siblings `22/25, 7/10, 6/10, 0/5, 0/11, 0/3` against others
/// `49/58, 36/52, 12/20, 4/36, 0/41, 0/44`.
#[must_use]
pub fn by_sextile(
    panel: &[DistrictRecord],
    rows: &[Standing],
    changes: &[Change],
) -> [Band; BANDS] {
    let siblings: BTreeSet<String> = lost_pupils::siblings(rows)
        .into_iter()
        .map(|s| s.irn.clone())
        .collect();
    let band_of: BTreeMap<&str, usize> = size_terms::sextiles(panel)
        .iter()
        .enumerate()
        .flat_map(|(band, records)| {
            records
                .iter()
                .map(move |record| (record.irn.as_str(), band))
                .collect::<Vec<_>>()
        })
        .collect();
    let mut counts = [(0usize, 0usize, 0usize, 0usize); BANDS];
    for row in size_terms::striking_out(panel, changes) {
        if row.guaranteed {
            continue;
        }
        let Some(&band) = band_of.get(row.irn.as_str()) else {
            continue;
        };
        let cell = &mut counts[band];
        if siblings.contains(&row.irn) {
            cell.1 += 1;
            cell.0 += usize::from(row.newly_guaranteed());
        } else {
            cell.3 += 1;
            cell.2 += usize::from(row.newly_guaranteed());
        }
    }
    std::array::from_fn(|index| Band {
        rank: index + 1,
        siblings: (counts[index].0, counts[index].1),
        others: (counts[index].2, counts[index].3),
    })
}

/// How many siblings would move if each band's siblings moved at that band's non-sibling rate.
///
/// **34.60 against 35 observed**, which is the whole of section 2: conditioned on size, being a
/// sibling is worth four tenths of a district.
#[must_use]
pub fn expected_from_size(bands: &[Band; BANDS]) -> f64 {
    bands
        .iter()
        .map(|band| band.siblings.1 as f64 * band.others_rate())
        .sum()
}

/// The siblings a set of changes leaves off the guarantee.
#[must_use]
pub fn unmoved<'a>(
    panel: &[DistrictRecord],
    rows: &'a [Standing],
    changes: &[Change],
) -> Vec<&'a Standing> {
    let moved: BTreeSet<String> = size_terms::striking_out(panel, changes)
        .into_iter()
        .filter(size_terms::Counterfactual::newly_guaranteed)
        .map(|c| c.irn)
        .collect();
    lost_pupils::siblings(rows)
        .into_iter()
        .filter(|s| !moved.contains(&s.irn))
        .collect()
}

/// The components of `[H]` a gap is taken against, in the order the table prints them.
pub const COMPONENTS: [&str; 7] = [
    "base cost state share",
    "targeted assistance",
    "special education",
    "DPIA",
    "English learners",
    "gifted",
    "career-technical",
];

/// One district's distance from the floor, against each component of the aid that clears it.
#[derive(Debug, Clone, PartialEq)]
pub struct Carried {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// Base cost enrolled ADM.
    pub adm: Adm,
    /// `[H] − [H2]`, what the district stands above its FY2020 base.
    pub gap: Dollars,
    /// The seven components, in [`COMPONENTS`] order. They sum to `[H]`.
    pub components: [Dollars; COMPONENTS.len()],
}

impl Carried {
    /// Which components are individually larger than the gap.
    ///
    /// A reach and not an attribution: the district is held above the floor by the whole of
    /// `[H]`, and what this says is that a repeal of that line alone would be enough — see the
    /// caution in the module documentation.
    #[must_use]
    pub fn covers(&self) -> [bool; COMPONENTS.len()] {
        std::array::from_fn(|index| self.components[index] > self.gap)
    }
}

/// Each district's gap against the seven components.
///
/// # Panics
///
/// If a standing is not in the panel, which one taken from it cannot be.
#[must_use]
pub fn carried(panel: &[DistrictRecord], population: &[&Standing]) -> Vec<Carried> {
    let by_irn: BTreeMap<&str, &DistrictRecord> =
        panel.iter().map(|r| (r.irn.as_str(), r)).collect();
    population
        .iter()
        .map(|s| {
            let record = by_irn[s.irn.as_str()];
            let c = &record.categoricals;
            Carried {
                irn: s.irn.clone(),
                name: s.name.clone(),
                adm: record.base_cost_adm(),
                gap: s.formula - s.floor,
                components: [
                    record.base_cost_state_share,
                    c.targeted_assistance,
                    c.special_education,
                    c.dpia,
                    c.english_learners,
                    c.gifted,
                    c.career_technical,
                ],
            }
        })
        .collect()
}

/// How many districts each component could cover alone, in [`COMPONENTS`] order.
///
/// Over the 29 the two size terms leave: `28, 14, 9, 4, 0, 1, 0`.
#[must_use]
pub fn covering(rows: &[Carried]) -> [usize; COMPONENTS.len()] {
    std::array::from_fn(|index| rows.iter().filter(|row| row.covers()[index]).count())
}

/// Each component summed over a population, in [`COMPONENTS`] order.
#[must_use]
pub fn component_totals(rows: &[Carried]) -> [Dollars; COMPONENTS.len()] {
    std::array::from_fn(|index| rows.iter().map(|row| row.components[index]).sum())
}

/// The enrollment cluster and its siblings: the union the arc calls "the 153".
///
/// A union of two differently-defined sets — the whole cluster, and the formula districts past
/// the cluster's *median* rate — and so not the same population as
/// [`shrinking_at_the_cluster_rate`]. See section 4.
#[must_use]
pub fn cluster_and_siblings(rows: &[Standing]) -> Vec<&Standing> {
    let siblings: BTreeSet<&str> = lost_pupils::siblings(rows)
        .into_iter()
        .map(|s| s.irn.as_str())
        .collect();
    rows.iter()
        .filter(|s| s.population == Population::EnrollmentLoss || siblings.contains(s.irn.as_str()))
        .collect()
}

/// Every district shrinking at the cluster's median rate or faster, whatever the guarantee does
/// for it.
///
/// **169**, not the 153 above: a median cuts its own population in half, so 44 of the cluster's
/// 89 are below their own threshold, and 60 guaranteed districts outside the cluster are above
/// it.
#[must_use]
pub fn shrinking_at_the_cluster_rate(rows: &[Standing]) -> Vec<&Standing> {
    let threshold = lost_pupils::cluster_median_enrollment_term(rows);
    rows.iter()
        .filter(|s| s.terms.enrollment_term >= threshold)
        .collect()
}
