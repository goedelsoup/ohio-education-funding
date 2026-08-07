//! Winners and losers between two funding runs.
//!
//! [`project::policy`] answers what a policy does to one district. This crate answers the
//! question a legislature actually asks: **who gains, who loses, by how much, and how many
//! districts does this reach at all.**
//!
//! ```
//! use project::{panel::panel, policy::{GuaranteeRule, Policy}};
//! use scenario_delta::ScenarioDelta;
//!
//! let districts = panel();
//! let delta = ScenarioDelta::between(
//!     &districts,
//!     &Policy::current_law(),
//!     &Policy { guarantee: GuaranteeRule::Removed, ..Policy::current_law() },
//! );
//! let total = delta.total();
//! println!("{:+.0} across {} districts, {} untouched", total.dollars, total.reach.districts, total.reach.unmoved);
//! ```
//!
//! # Two rules, made structural rather than documentary
//!
//! The [`scenario-delta` skill](../../../.yidam/skills/scenario-delta.md) states two rules about
//! how a delta must be reported. Both are enforced here by the shape of the types rather than by
//! a note asking the caller to be careful, because a note is not a constraint.
//!
//! **The off-formula count is not a footnote.** A perturbation is inert for any district whose
//! payment is set by the guarantee rather than by the formula, and in FY2027 that is just under
//! half of Ohio. A statewide total quoted without it overstates how many districts a change
//! actually reaches. So there is no method returning a bare [`Dollars`]: every total is an
//! [`Aggregate`], which cannot be constructed without its [`Reach`]. A caller who wants the
//! headline number has the count of untouched districts in the same value.
//!
//! **Two orderings, always.** The largest dollar movers and the most-affected districts are
//! rarely the same set, and a table sorted one way is a different political argument from the
//! same table sorted the other. So there is no `top_by_dollars`: [`ScenarioDelta::orderings`]
//! returns [`Orderings`], which holds both. Taking one means having taken the other.
//!
//! # Why this does not project
//!
//! A delta is computed at **modelled enrollment only**. Running the same comparison at projected
//! enrollment would fold forecast error into a policy effect, and the two are different
//! epistemic acts — see [`project::report`], which keeps them in separate fields with the
//! interval attached to only one. A `ScenarioDelta` is the deterministic half. There is
//! deliberately no constructor that takes a projection.

#![forbid(unsafe_code)]

pub mod incidence;

use edfund_core::{Adm, Dollars};
use project::panel::DistrictRecord;
use project::policy::{apply, Outcome, Policy};

pub use incidence::{across, Axis, Incidence, Stratum};

/// Smallest change counted as movement: half a cent, the resolution the fixture carries.
///
/// Below this a difference is floating-point noise in a figure the department published to the
/// cent, and counting it as a winner would put hundreds of districts in a table for nothing.
pub const MOVED: Dollars = 0.005;

/// Where a district stood relative to the guarantee across the two runs.
///
/// Four cases rather than a boolean, because "off formula" is a property of a *pair* of runs.
/// The one that matters for reporting reach is [`Standing::HeldThroughout`]: those districts are
/// paid by the guarantee under both policies, so a change to the formula cannot touch them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Paid by the formula under both policies. The perturbation reaches it.
    FormulaThroughout,
    /// Paid by the guarantee under both policies.
    ///
    /// Inert for any perturbation that leaves the guarantee rule alone — and *only* then. A
    /// policy that re-bases or phases out the guarantee reaches exactly these districts and
    /// nobody else, which is why the guarantee is the lever with a different incidence from
    /// every other one.
    HeldThroughout,
    /// On the guarantee in the baseline and on the formula under the policy.
    LiftedOff,
    /// On the formula in the baseline and on the guarantee under the policy.
    ///
    /// **Unreachable with the committed panel, and kept so the reason has somewhere to live.** A
    /// guarantee baseline is a district's FY2020 receipt, and
    /// [`DistrictRecord::guarantee_baseline`](project::panel::DistrictRecord::guarantee_baseline)
    /// can disclose it only for districts *already* on the guarantee — being held there is the
    /// only thing that reveals the figure. The 315 districts the formula pays therefore have no
    /// modelled floor at all, though every one of them has a real one sitting below its current
    /// formula amount.
    ///
    /// The consequence runs one way: a simulated **cut** lets those districts fall past a floor
    /// that would in fact catch them, so it overstates its own savings. They hold 46% of Ohio's
    /// students. Pinned in `tests/who_a_change_reaches.rs`.
    PushedOn,
}

impl Standing {
    /// Whether the guarantee determined this district's aid in both runs.
    #[must_use]
    pub const fn off_formula(self) -> bool {
        matches!(self, Self::HeldThroughout)
    }

    fn of(baseline: &Outcome, perturbed: &Outcome) -> Self {
        match (baseline.on_guarantee, perturbed.on_guarantee) {
            (false, false) => Self::FormulaThroughout,
            (true, true) => Self::HeldThroughout,
            (true, false) => Self::LiftedOff,
            (false, true) => Self::PushedOn,
        }
    }
}

/// What the perturbation did to one district.
#[derive(Debug, Clone, PartialEq)]
pub struct Delta {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Current-year enrolled ADM both runs were computed at.
    pub adm: Adm,
    /// Realized aid under the baseline policy.
    pub baseline: Dollars,
    /// Realized aid under the perturbed policy.
    pub perturbed: Dollars,
    /// Where the district stood relative to the guarantee in each run.
    pub standing: Standing,
    /// Assessed valuation per pupil, FY2023. Carried for [`incidence`], `None` for the three
    /// districts the profile report does not cover.
    pub valuation_per_pupil: Option<Dollars>,
    /// The state's share of base cost under the baseline, as a fraction.
    pub state_share_fraction: f64,
}

impl Delta {
    /// Change in dollars. Positive is a gain to the district.
    #[must_use]
    pub fn dollars(&self) -> Dollars {
        self.perturbed - self.baseline
    }

    /// Change per current-year pupil.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.dollars() / self.adm
    }

    /// Change as a fraction of baseline aid, or `None` when the baseline is zero.
    ///
    /// `None` rather than infinity or zero. A district receiving nothing under the baseline and
    /// something under the policy has no percent change — the quantity is undefined, and both
    /// available lies about it would sort into a table and be read as a result. This is not
    /// hypothetical: removing the guarantee from a district whose formula amount is zero leaves
    /// it at zero.
    #[must_use]
    pub fn percent(&self) -> Option<f64> {
        (self.baseline.abs() > MOVED).then(|| self.dollars() / self.baseline)
    }

    /// Whether the perturbation moved this district's aid at all.
    #[must_use]
    pub fn moved(&self) -> bool {
        self.dollars().abs() > MOVED
    }
}

/// How far a total reaches. Always attached to one; never available on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reach {
    /// Districts in the comparison.
    pub districts: usize,
    /// Districts whose aid rises.
    pub gainers: usize,
    /// Districts whose aid falls.
    pub losers: usize,
    /// Districts the perturbation does not move.
    pub unmoved: usize,
    /// Districts the guarantee paid under both policies.
    ///
    /// The number the skill insists on reporting beside every total. It is not the same as
    /// [`Reach::unmoved`] — a formula district can also be unmoved, if the lever pulled does not
    /// happen to touch it — and the gap between the two is itself informative.
    pub held_throughout: usize,
    /// Districts the perturbation lifted off the guarantee onto the formula.
    pub lifted_off: usize,
    /// Districts the perturbation pushed from the formula onto the guarantee.
    pub pushed_on: usize,
}

/// A dollar total with the reach it was computed over.
///
/// The type exists so that a total cannot be quoted without its denominator. "This costs $879
/// million" and "this costs $879 million and does not reach 294 of 609 districts" are different
/// claims, and only the second one is complete.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aggregate {
    /// Change in total state aid.
    pub dollars: Dollars,
    /// Enrolled ADM across the districts counted.
    pub adm: Adm,
    /// How far it reaches.
    pub reach: Reach,
}

impl Aggregate {
    /// The total spread across every pupil in the comparison.
    ///
    /// Distinct from the mean of the per-district per-pupil figures, and smaller than it
    /// whenever the gain concentrates in small districts — which under Ohio's staffing minimums
    /// it does. Both are honest; they answer different questions.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.dollars / self.adm
    }

    /// Share of districts the perturbation moves.
    #[must_use]
    pub fn reaches(&self) -> f64 {
        if self.reach.districts == 0 {
            return 0.0;
        }
        (self.reach.gainers + self.reach.losers) as f64 / self.reach.districts as f64
    }
}

/// Both orderings of the same table.
///
/// Held together because they disagree, and because a caller who can reach for only one will
/// reach for whichever supports the point being made. Each vector holds every district, sorted
/// by descending magnitude of change; take from the front.
#[derive(Debug, Clone, PartialEq)]
pub struct Orderings<'a> {
    /// Largest absolute dollar movers first. Tracks district size.
    pub by_dollars: Vec<&'a Delta>,
    /// Largest absolute per-pupil movers first. Tracks how hard the change lands.
    pub by_per_pupil: Vec<&'a Delta>,
}

/// The comparison between two runs of the funding model over the same panel.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioDelta {
    /// The policy the comparison is against.
    pub baseline: Policy,
    /// The policy being tested.
    pub perturbed: Policy,
    /// One row per district, in panel order.
    pub deltas: Vec<Delta>,
}

impl ScenarioDelta {
    /// Compare two policies over the same panel, at modelled enrollment.
    ///
    /// Both runs are computed here rather than accepted as two outcome vectors, so they cannot
    /// be over different panels or in different orders. That mistake produces a table that looks
    /// right and pairs the wrong districts.
    #[must_use]
    pub fn between(panel: &[DistrictRecord], baseline: &Policy, perturbed: &Policy) -> Self {
        let deltas = panel
            .iter()
            .map(|record| {
                let before = apply(record, baseline, record.current_year_adm);
                let after = apply(record, perturbed, record.current_year_adm);
                Delta {
                    irn: record.irn.clone(),
                    name: record.name.clone(),
                    adm: record.current_year_adm,
                    baseline: before.realized_aid,
                    perturbed: after.realized_aid,
                    standing: Standing::of(&before, &after),
                    valuation_per_pupil: record.valuation_per_pupil,
                    state_share_fraction: record.state_share_fraction(),
                }
            })
            .collect();
        Self {
            baseline: *baseline,
            perturbed: *perturbed,
            deltas,
        }
    }

    /// The statewide total, with its reach.
    #[must_use]
    pub fn total(&self) -> Aggregate {
        self.aggregate(|_| true)
    }

    /// The total over the districts a predicate selects, with the reach over that subset.
    ///
    /// The reach is recomputed rather than inherited, so a subset total reports how far it
    /// reaches *within the subset*. Carrying the statewide reach onto a subset total is the
    /// obvious shortcut and produces a figure whose two halves describe different populations.
    #[must_use]
    pub fn aggregate<F>(&self, mut include: F) -> Aggregate
    where
        F: FnMut(&Delta) -> bool,
    {
        let rows: Vec<&Delta> = self.deltas.iter().filter(|d| include(d)).collect();
        let gainers = rows.iter().filter(|d| d.dollars() > MOVED).count();
        let losers = rows.iter().filter(|d| d.dollars() < -MOVED).count();
        Aggregate {
            dollars: rows.iter().map(|d| d.dollars()).sum(),
            adm: rows.iter().map(|d| d.adm).sum(),
            reach: Reach {
                districts: rows.len(),
                gainers,
                losers,
                unmoved: rows.len() - gainers - losers,
                held_throughout: rows.iter().filter(|d| d.standing.off_formula()).count(),
                lifted_off: rows
                    .iter()
                    .filter(|d| d.standing == Standing::LiftedOff)
                    .count(),
                pushed_on: rows
                    .iter()
                    .filter(|d| d.standing == Standing::PushedOn)
                    .count(),
            },
        }
    }

    /// Both orderings of the table. There is no method returning only one.
    #[must_use]
    pub fn orderings(&self) -> Orderings<'_> {
        Orderings {
            by_dollars: self.sorted_by(|d| d.dollars().abs()),
            by_per_pupil: self.sorted_by(|d| d.per_pupil().abs()),
        }
    }

    /// Descending by a magnitude, ties broken by IRN so the order is total and reproducible.
    fn sorted_by<F>(&self, magnitude: F) -> Vec<&Delta>
    where
        F: Fn(&Delta) -> f64,
    {
        let mut rows: Vec<&Delta> = self.deltas.iter().collect();
        rows.sort_by(|a, b| {
            magnitude(b)
                .partial_cmp(&magnitude(a))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.irn.cmp(&b.irn))
        });
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use project::panel::panel;
    use project::policy::GuaranteeRule;

    fn refresh() -> Policy {
        Policy {
            base_cost_scale: 1.05,
            ..Policy::current_law()
        }
    }

    #[test]
    fn a_policy_against_itself_moves_nothing() {
        let panel = panel();
        let delta = ScenarioDelta::between(&panel, &Policy::current_law(), &Policy::current_law());
        let total = delta.total();
        assert_eq!(total.dollars, 0.0);
        assert_eq!(total.reach.gainers, 0);
        assert_eq!(total.reach.losers, 0);
        assert_eq!(total.reach.unmoved, panel.len());
    }

    #[test]
    fn the_percent_change_is_absent_rather_than_wrong_when_the_baseline_is_zero() {
        let delta = Delta {
            irn: "000000".into(),
            name: "nowhere local".into(),
            adm: 100.0,
            baseline: 0.0,
            perturbed: 5_000.0,
            standing: Standing::FormulaThroughout,
            valuation_per_pupil: None,
            state_share_fraction: 0.0,
        };
        assert_eq!(delta.percent(), None);
        assert_eq!(delta.dollars(), 5_000.0);
        assert_eq!(delta.per_pupil(), 50.0);
    }

    #[test]
    fn a_subset_total_carries_the_subset_reach() {
        let panel = panel();
        let delta = ScenarioDelta::between(&panel, &Policy::current_law(), &refresh());
        let on_formula = delta.aggregate(|d| !d.standing.off_formula());
        assert_eq!(on_formula.reach.held_throughout, 0);
        assert!(on_formula.reach.districts < panel.len());
        // And the subset totals to the same dollars as the whole, because the districts it
        // excludes contribute nothing — which is the point of the exclusion.
        assert!((on_formula.dollars - delta.total().dollars).abs() < 1.0);
    }

    #[test]
    fn both_orderings_are_total_and_reproducible() {
        let panel = panel();
        let delta = ScenarioDelta::between(&panel, &Policy::current_law(), &refresh());
        let first = delta.orderings();
        let second = delta.orderings();
        assert_eq!(first.by_dollars, second.by_dollars);
        assert_eq!(first.by_per_pupil, second.by_per_pupil);
        assert_eq!(first.by_dollars.len(), panel.len());
        assert_eq!(first.by_per_pupil.len(), panel.len());
    }

    #[test]
    fn removing_the_guarantee_lifts_nobody_and_pushes_nobody() {
        // The four-way standing is not decoration. Removal collapses every guaranteed district
        // to its formula amount, so nobody crosses in either direction and `held_throughout` is
        // zero — the perturbation is precisely the one that reaches the off-formula population.
        let panel = panel();
        let delta = ScenarioDelta::between(
            &panel,
            &Policy::current_law(),
            &Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
        );
        let total = delta.total();
        assert_eq!(total.reach.held_throughout, 0);
        assert_eq!(total.reach.pushed_on, 0);
        assert_eq!(total.reach.lifted_off, total.reach.losers);
        assert!(total.dollars < 0.0);
    }
}
