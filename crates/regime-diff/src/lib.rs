//! Differencing two funding regimes at component level, and reporting what the decomposition
//! fails to explain.
//!
//! A district that gained under the Fair School Funding Plan gained through a specific
//! mechanism. It might be the district-specific base cost, or the shift from the charge-off to
//! local capacity, or the end of the community school deduct. Those have different policy
//! implications and different durability, and a single net delta conflates them.
//!
//! # The `replaces` edge is the whole of what makes this possible
//!
//! The corpus aligns two regimes' components through `replaces` edges, and where no edge exists
//! the components are not comparable. This crate refuses to guess an alignment: [`ALIGNED`] is
//! the registry of pairs the corpus actually declares, and [`UNALIGNED`] names every component
//! that has no counterpart, with the reason.
//!
//! **There is exactly one aligned pair.** FSFP's local capacity measure replaces the charge-off
//! local share. Base cost, the guarantee, and every categorical have no declared predecessor, so
//! a component-level regime diff covers one row of the calculation and cannot cover more.
//!
//! # What the residual measures, which is narrower than it first appears
//!
//! [`RegimeDiff::residual`] is the part of the total difference the aligned components do not
//! account for. The skill's rule is that a large residual means the mapping is wrong and the
//! result should not be trusted.
//!
//! On the run below it comes out **exactly zero for 463 of the 470 districts where both sides
//! can be valued**, and that is a check on the substitution rather than a finding about the
//! regimes. Holding base cost fixed means the local share is the only thing that *can* differ,
//! so a nonzero residual would mean the arithmetic had gone wrong. It did not. The seven
//! exceptions are districts where the charge-off ran past the base cost it was subtracted from,
//! and the residual is exactly that truncation.
//!
//! A zero residual here therefore says the one component was substituted cleanly. It does
//! **not** say the two regimes differ only in that component — a genuine Bridge-against-FSFP
//! diff would leave nearly everything unexplained, because the corpus has no Bridge components
//! at all. Coverage of the formula and correctness of the substitution are different questions
//! and only the second one is what this number answers.
//!
//! # What is actually computed here, and what it is not
//!
//! A **counterfactual at FY2027 inputs**: holding the Fair School Funding Plan's own computed
//! base cost fixed, what would a district's local share be under the charge-off, and what state
//! aid would result? That is a legitimate reading of "what each of two regimes would pay for one
//! agency and one fiscal period" — and it is emphatically *not* a reconstruction of any year the
//! charge-off actually governed. Those years need the era's formula amount, cost-of-doing-
//! business factor, and DPIA, none of which this corpus holds.

#![forbid(unsafe_code)]

pub mod charge_off;

use edfund_core::Dollars;
use project::panel::DistrictRecord;

pub use charge_off::{local_share_per_pupil, ChargeOffRate, RATES, TERMINAL_MILLS};

/// Two components the corpus's `replaces` graph declares comparable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    /// The corpus node for the later component.
    pub successor: &'static str,
    /// The corpus node for the component it replaces.
    pub predecessor: &'static str,
    /// What role both occupy in their regime's calculation.
    pub role: &'static str,
}

/// Every component pair the corpus aligns. There is one.
///
/// Sourced from the `replaces` edges in `.yidam/corpus/formula-component/`. A pair is added here
/// only when the corpus declares it, so this list and the graph cannot drift apart without a
/// test noticing — see `tests/`.
pub const ALIGNED: &[Alignment] = &[Alignment {
    successor: "fsfp-local-capacity-measure",
    predecessor: "charge-off-local-share",
    role: "how much of a district's cost the district itself bears",
}];

/// A component with no declared predecessor, and why comparing it would be a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unaligned {
    /// The corpus node.
    pub component: &'static str,
    /// Why no `replaces` edge exists.
    pub reason: &'static str,
}

/// Components a regime diff cannot reach, named so their absence is legible.
pub const UNALIGNED: &[Unaligned] = &[
    Unaligned {
        component: "fsfp-base-cost-calculation",
        reason: "The Bridge formula computed no base cost — it distributed by reference to prior \
                 years — so there is nothing for this to replace. The earlier foundation formula's \
                 per-pupil amount is a predecessor in role, and the corpus holds no node for it.",
    },
    Unaligned {
        component: "temporary-transitional-aid-guarantee",
        reason: "A guarantee against a frozen FY2020 level has no counterpart in the regime it \
                 guards against; it is the seam between regimes rather than a step inside either.",
    },
];

/// One aligned component, valued under both regimes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComponentDiff {
    /// The pair being compared.
    pub alignment: Alignment,
    /// Per pupil under the earlier regime, or `None` where the source cannot yield it.
    pub predecessor: Option<Dollars>,
    /// Per pupil under the later regime, or `None` where the source censors it.
    pub successor: Option<Dollars>,
}

impl ComponentDiff {
    /// Successor minus predecessor, or `None` if either side is unavailable.
    ///
    /// `None` rather than treating a missing side as zero. A censored quantity is not a small
    /// one — where the minimum state share binds, all that is known about local capacity is that
    /// it exceeds a threshold, and substituting zero would invert the comparison for exactly the
    /// districts where it is most interesting.
    #[must_use]
    pub fn difference(&self) -> Option<Dollars> {
        Some(self.successor? - self.predecessor?)
    }
}

/// Two regimes' answers for one district, decomposed as far as the corpus allows.
#[derive(Debug, Clone, PartialEq)]
pub struct RegimeDiff {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// The aligned components, valued.
    pub components: Vec<ComponentDiff>,
    /// State aid per pupil under the earlier regime's mechanism, at this regime's base cost.
    pub predecessor_total: Option<Dollars>,
    /// State aid per pupil as this regime computes it.
    pub successor_total: Option<Dollars>,
}

impl RegimeDiff {
    /// Total difference in state aid per pupil.
    #[must_use]
    pub fn total_difference(&self) -> Option<Dollars> {
        Some(self.successor_total? - self.predecessor_total?)
    }

    /// The sum of what the aligned components explain.
    #[must_use]
    pub fn attributed(&self) -> Option<Dollars> {
        self.components
            .iter()
            .map(ComponentDiff::difference)
            .try_fold(0.0, |sum, d| Some(sum + d?))
    }

    /// What the decomposition does not explain.
    ///
    /// The line the skill requires and the one a reader should look at first. A local share is
    /// subtracted from cost to give aid, so a rise in local capacity is a fall in state aid: the
    /// component difference enters the total with its sign reversed, and the residual is what is
    /// left after accounting for that.
    #[must_use]
    pub fn residual(&self) -> Option<Dollars> {
        Some(self.total_difference()? + self.attributed()?)
    }

    /// Share of the total difference the components explain, or `None` when nothing moved.
    #[must_use]
    pub fn attributed_share(&self) -> Option<f64> {
        let total = self.total_difference()?;
        if total.abs() < 0.005 {
            return None;
        }
        Some(-self.attributed()? / total)
    }

    /// Whether every aligned component could be valued on both sides.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.components.iter().all(|c| c.difference().is_some())
    }
}

/// Difference the charge-off against FSFP local capacity for one district, at FY2027 inputs.
///
/// The Fair School Funding Plan's own computed base cost is held fixed and only the local share
/// mechanism is substituted, which is what isolates the one component the corpus can align.
///
/// # What each side does at its extremes
///
/// The charge-off had **no minimum state share** — that is an FSFP invention — so a district
/// whose deemed local share exceeded its cost received nothing, and 81 districts are in that
/// position at 23 mills against FY2027 base cost. Ohio's answer was the charge-off supplement
/// rather than a floor, and this crate does not model it. The FSFP side does have a floor, and
/// where it binds the local capacity measure is censored: all that is recoverable is that
/// capacity exceeds a threshold. So the component row goes absent for those districts while the
/// totals stay computable, and the difference is visible with its cause unattributable. That is
/// the honest state of the comparison and [`RegimeDiff::residual`] reports it as `None`.
#[must_use]
pub fn at_fy2027(record: &DistrictRecord, mills: f64) -> RegimeDiff {
    let successor = record.implied_local_capacity_per_pupil();
    let predecessor = record
        .valuation_per_pupil
        .map(|valuation| charge_off::local_share_per_pupil(valuation, mills));

    // Base cost aid per pupil as the department computes it, on its own denominator.
    let successor_total = (record.current_year_adm > 0.0)
        .then(|| record.base_cost_state_share / record.current_year_adm);
    // The residual the charge-off leaves, floored at zero rather than at a minimum share.
    let predecessor_total = predecessor.map(|local| (record.base_cost_per_pupil - local).max(0.0));

    RegimeDiff {
        irn: record.irn.clone(),
        name: record.name.clone(),
        components: vec![ComponentDiff {
            alignment: ALIGNED[0],
            predecessor,
            successor,
        }],
        predecessor_total,
        successor_total,
    }
}

/// Run [`at_fy2027`] across a panel.
#[must_use]
pub fn panel_at_fy2027(panel: &[DistrictRecord], mills: f64) -> Vec<RegimeDiff> {
    panel
        .iter()
        .map(|record| at_fy2027(record, mills))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> Alignment {
        ALIGNED[0]
    }

    #[test]
    fn the_corpus_aligns_exactly_one_component_pair() {
        assert_eq!(ALIGNED.len(), 1);
        assert_eq!(ALIGNED[0].successor, "fsfp-local-capacity-measure");
        assert_eq!(ALIGNED[0].predecessor, "charge-off-local-share");
    }

    #[test]
    fn a_censored_component_makes_the_difference_absent_rather_than_zero() {
        let diff = ComponentDiff {
            alignment: pair(),
            predecessor: Some(5_000.0),
            successor: None,
        };
        assert_eq!(diff.difference(), None);
    }

    #[test]
    fn the_residual_is_what_the_components_do_not_explain() {
        // A district whose local share falls by $500 should, with base cost held fixed, gain
        // exactly $500 of state aid. Then the residual is zero and the components explain it all.
        let diff = RegimeDiff {
            irn: "000000".into(),
            name: "somewhere local".into(),
            components: vec![ComponentDiff {
                alignment: pair(),
                predecessor: Some(5_000.0),
                successor: Some(4_500.0),
            }],
            predecessor_total: Some(3_000.0),
            successor_total: Some(3_500.0),
        };
        assert_eq!(diff.total_difference(), Some(500.0));
        assert_eq!(diff.attributed(), Some(-500.0));
        assert_eq!(diff.residual(), Some(0.0));
        assert_eq!(diff.attributed_share(), Some(1.0));
        assert!(diff.is_complete());
    }

    #[test]
    fn a_residual_appears_when_something_outside_the_components_moved() {
        // Same component movement, but the total moved by more: a floor, a guarantee, or a
        // component with no `replaces` edge did the rest.
        let diff = RegimeDiff {
            irn: "000000".into(),
            name: "somewhere local".into(),
            components: vec![ComponentDiff {
                alignment: pair(),
                predecessor: Some(5_000.0),
                successor: Some(4_500.0),
            }],
            predecessor_total: Some(3_000.0),
            successor_total: Some(4_200.0),
        };
        assert_eq!(diff.residual(), Some(700.0));
        assert_eq!(diff.attributed_share(), Some(500.0 / 1_200.0));
    }

    #[test]
    fn every_unaligned_component_carries_a_reason() {
        for entry in UNALIGNED {
            assert!(
                entry.reason.len() > 40,
                "{} has no real reason recorded",
                entry.component
            );
        }
    }
}
