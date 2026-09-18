//! The cost-input refresh H.B. 96 declined, priced across the panel rather than on one district.
//!
//! # The dial this is, and the two it is not
//!
//! `funding-regime/fair-school-funding-plan` records that the plan reaches FY2027 at 100% of its
//! computed amount, and that the amount is computed from stale inputs. Those are **three**
//! separate freezes and the corpus says so:
//!
//! 1. the **salary reference year**, held at FY2022 by H.B. 96 after H.B. 33 refreshed it there;
//! 2. the **categorical multiplicands** — the statewide average base cost per pupil and its
//!    career-technical companion — held at their FY2024 values;
//! 3. the **phase-in**, which was not frozen at all and ran to 100% on the original schedule.
//!
//! This module moves the first and only the first. The phase-in is not a candidate — it arrived
//! where Cupp and Patterson wrote it — so "the current budget against the original plan" is a
//! question about input years, and the largest input is the classroom teacher salary.
//!
//! # What is perturbed, and why it is a lower bound
//!
//! [`FY2022_TEACHER_SALARY`] to [`FY2024_TEACHER_SALARY`], through
//! [`foundation::teacher_salary_refresh_delta`], which is the closed form for the whole effect of
//! that one input: the salary reaches base cost through the classroom, special and
//! professional-development terms and nothing else. The other nine salary categories have risen
//! too and are held here, so **every figure below is a floor on what a real refresh would do.**
//! `crates/foundation/examples/input_year_refresh.rs` states the same bound for one district.
//!
//! # Why this is not a [`crate::policy::Policy`]
//!
//! A refresh is **not a uniform scale.** It moves each district's base cost in proportion to its
//! own funded teaching positions, and positions per pupil vary with grade mix and with the
//! six-teacher special-education minimum — `foundation::refresh` measures the spread at about 1.4
//! between the least and most affected district per pupil. [`Policy::base_cost_scale`] is one
//! number for the whole state, so a policy cannot express this, and running it as a uniform scale
//! would answer an incidence question with an assumption about incidence.
//!
//! So each district is run through [`crate::policy::apply`] against **its own** scale factor. The
//! machinery is the verified one; only the resolution of the lever differs.
//!
//! # Three limits, each of which would produce a wrong reading
//!
//! **The categorical term moves with the district's own factor, and it is 14.6% of the answer.**
//! [`crate::policy::apply`] scales base-cost-denominated categoricals by `base_cost_scale`, and
//! the multiplicand those are priced in is a *statewide* average — so strictly it should move by
//! the statewide factor. The factors are tightly clustered (1.0218 to 1.0408 against a statewide
//! 1.0395, median 1.0396), so the approximation is close. The channel itself is not small: $32.2m
//! of the $220.3m, carried separately on [`Refreshed::categorical_delta`] precisely so that a
//! reader who holds H.B. 96's separate FY2024 freeze against it can take it back out.
//!
//! **Base cost is computed on three-year ADM and paid on current-year ADM.** That is R.C.
//! 3317.017's own asymmetry, not this module's, and [`crate::policy::apply`] follows it. It means
//! a district losing pupils collects a smaller share of its own base cost increase than a growing
//! one — which cuts against, not for, any claim that a refresh favours declining districts.
//!
//! **A district on the guarantee collects nothing** unless the increase carries its formula amount
//! past its FY2020 floor. That is not a limitation of the measurement; it is most of the answer.
//! See [`Reach`].

use edfund_core::{Adm, Dollars};
use foundation::StatewideFactors;

use crate::panel::DistrictRecord;
use crate::policy::{apply, Outcome, Policy, Statewide};

/// The reference-year salary H.B. 96 carries forward, as the department's FY27 calculator prices it.
pub const FY2022_TEACHER_SALARY: Dollars = StatewideFactors::fy2027().teacher_salary;

/// The FY2024 statewide average a refresh would read.
///
/// `foundation`'s constant rather than a second copy: it was duplicated across two test files and
/// a corpus node before #157, which is the pattern this re-export exists to avoid repeating.
pub const FY2024_TEACHER_SALARY: Dollars = foundation::FY2024_TEACHER_SALARY;

/// Smallest change counted as movement: half a cent, matching `scenario_delta::MOVED`.
pub const MOVED: Dollars = 0.005;

/// What the refresh did to one district.
#[derive(Debug, Clone, PartialEq)]
pub struct Refreshed {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Current-year enrolled ADM, the count both runs were computed at.
    pub adm: Adm,
    /// The district's own base cost increase, before any share is taken of it.
    ///
    /// The gross quantity. A district receives the state's share of this and no more, and for a
    /// district on the minimum share that is a tenth of it.
    pub base_cost_delta: Dollars,
    /// The categorical repricing the same refresh causes, which arrives as aid directly.
    ///
    /// Special education, English learners and career-technical are each `weight × the statewide
    /// average base cost per pupil × count × state share`, and the department publishes them net
    /// of the share — so when the refresh moves that average, the whole of this lands as aid
    /// rather than a share of it. $32.2m statewide against $465.0m of base cost, which is 14.6%
    /// of the total and too large to leave inside a single "increase".
    ///
    /// **It is a separate freeze in law.** H.B. 96 holds the multiplicand at FY2024 under its own
    /// provision, so a budget that refreshed only the salary reference year would not move this.
    /// It is carried and added here because `.yidam/decisions/scenario-models-ohio.yml` holds that
    /// a lever moving all 609 districts may not hold a statewide average fixed and call the result
    /// a cost — and it is carried *separately* so a caller who disagrees can subtract it.
    pub categorical_delta: Dollars,
    /// Realized aid under current law.
    pub baseline: Dollars,
    /// Realized aid with the salary input refreshed.
    pub refreshed: Dollars,
    /// Whether the guarantee determined aid under current law.
    pub on_guarantee_before: bool,
    /// Whether it determines aid after the refresh.
    pub on_guarantee_after: bool,
}

impl Refreshed {
    /// Change in realized aid. Never negative: a cost input rising cannot lower computed aid.
    #[must_use]
    pub fn dollars(&self) -> Dollars {
        self.refreshed - self.baseline
    }

    /// Change per current-year pupil.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.dollars() / self.adm
    }

    /// Whether the refresh moved this district's aid at all.
    #[must_use]
    pub fn moved(&self) -> bool {
        self.dollars().abs() > MOVED
    }

    /// Whether the guarantee paid this district before and after, so the refresh cannot reach it.
    ///
    /// The count that belongs beside every total here, for the reason `scenario_delta::Reach`
    /// gives: a statewide figure quoted without it overstates how far the change travels.
    #[must_use]
    pub const fn held_throughout(&self) -> bool {
        self.on_guarantee_before && self.on_guarantee_after
    }

    /// Everything the refresh adds to this district's computed requirement.
    ///
    /// Base cost and the categorical repricing together. The denominator [`Self::captured`]
    /// divides by, and the reason it is a method rather than a field: the two halves reach aid
    /// through different routes and a reader has to be able to see them apart.
    #[must_use]
    pub fn computed_increase(&self) -> Dollars {
        self.base_cost_delta + self.categorical_delta
    }

    /// The share of its own computed increase the district actually collects.
    ///
    /// `None` where the increase is zero. About one for a district the state funds at the margin,
    /// a tenth for one on the minimum state share, and **zero** for one the guarantee holds —
    /// which is the distribution this module exists to show.
    ///
    /// It can exceed one slightly. Base cost is computed on three-year ADM and paid on
    /// current-year ADM, so a district whose enrollment is rising collects the increase against a
    /// larger count than the one that generated it. The same asymmetry is why a *declining*
    /// district collects less than its own costing implies, which is the finding.
    #[must_use]
    pub fn captured(&self) -> Option<f64> {
        let increase = self.computed_increase();
        (increase.abs() > MOVED).then(|| self.dollars() / increase)
    }
}

/// How far the refresh reaches, with the counts that qualify any total taken from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reach {
    /// Districts in the comparison.
    pub districts: usize,
    /// Districts whose aid rises.
    pub gainers: usize,
    /// Districts the refresh does not move at all.
    pub unmoved: usize,
    /// Districts the guarantee paid under both runs.
    pub held_throughout: usize,
    /// Districts the refresh carried off the guarantee onto the formula.
    pub lifted_off: usize,
}

/// A dollar total with the reach it was computed over.
///
/// The pairing `scenario_delta::Aggregate` enforces, for the same reason: "the refresh is worth
/// $X" and "the refresh is worth $X and does not touch N districts" are different claims.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Total {
    /// Change in aid across the districts counted.
    pub dollars: Dollars,
    /// Their gross base cost increase, before any share.
    pub base_cost_dollars: Dollars,
    /// The categorical repricing across them, which arrives as aid directly.
    pub categorical_dollars: Dollars,
    /// Enrolled ADM across them.
    pub adm: Adm,
    /// How far it reaches.
    pub reach: Reach,
}

impl Total {
    /// The total spread across every pupil counted.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.dollars / self.adm
    }

    /// Share of districts the refresh moves.
    #[must_use]
    pub fn reaches(&self) -> f64 {
        if self.reach.districts == 0 {
            return 0.0;
        }
        self.reach.gainers as f64 / self.reach.districts as f64
    }

    /// Everything the refresh adds to the computed requirement across these districts.
    #[must_use]
    pub fn computed_increase(&self) -> Dollars {
        self.base_cost_dollars + self.categorical_dollars
    }

    /// Share of that computed increase which becomes state aid.
    #[must_use]
    pub fn captured(&self) -> f64 {
        let increase = self.computed_increase();
        if increase.abs() <= MOVED {
            return 0.0;
        }
        self.dollars / increase
    }
}

/// The scale factor the refresh implies for one district.
///
/// One where the district has no base cost, rather than a division by zero: the model has no such
/// district and the guard is against a later vintage that does.
#[must_use]
pub fn scale_for(record: &DistrictRecord) -> f64 {
    if record.aggregate_base_cost > 0.0 {
        1.0 + base_cost_delta(record) / record.aggregate_base_cost
    } else {
        1.0
    }
}

/// One district's base cost increase under the refresh, before any state share is taken.
#[must_use]
pub fn base_cost_delta(record: &DistrictRecord) -> Dollars {
    foundation::teacher_salary_refresh_delta(
        record.funded_classroom_teachers + record.funded_special_teachers,
        FY2022_TEACHER_SALARY,
        FY2024_TEACHER_SALARY,
        &StatewideFactors::fy2027(),
    )
}

/// Run the refresh across a panel, one district-specific scale factor at a time.
///
/// # Panics
///
/// Never. A district with zero aggregate base cost is left at a scale of one rather than dividing
/// by it; the model has none and the guard is against a later vintage that does.
#[must_use]
pub fn across(panel: &[DistrictRecord]) -> Vec<Refreshed> {
    let law = Policy::current_law();
    // One resolution for the whole panel. No lever here moves a statewide statistic — the DPIA
    // blend and the supplemental rate are the two that do and neither is touched — so the same
    // resolution serves both runs, and computing it per district would be computing a different
    // and wrong number. See `Statewide::under`.
    let statewide = Statewide::under(panel, &law);

    panel
        .iter()
        .map(|record| {
            let delta = base_cost_delta(record);
            let scale = scale_for(record);
            let before = apply(record, &law, &statewide, record.current_year_adm);
            let after = apply(
                record,
                &Policy {
                    base_cost_scale: scale,
                    ..law
                },
                &statewide,
                record.current_year_adm,
            );
            Refreshed {
                irn: record.irn.clone(),
                name: record.name.clone(),
                adm: record.current_year_adm,
                base_cost_delta: delta,
                // Exactly what `apply` adds to the categorical term: it reprices the
                // base-cost-denominated ones by the scale and leaves the rest alone.
                categorical_delta: record.base_cost_denominated_categoricals() * (scale - 1.0),
                baseline: before.realized_aid,
                refreshed: after.realized_aid,
                on_guarantee_before: before.on_guarantee,
                on_guarantee_after: after.on_guarantee,
            }
        })
        .collect()
}

/// The refresh across the committed panel.
#[must_use]
pub fn across_panel() -> Vec<Refreshed> {
    across(&crate::panel::panel())
}

/// Total the districts a predicate selects, with the reach recomputed over that subset.
///
/// The reach is recomputed rather than inherited so that a subset total reports how far it
/// reaches *within the subset* — carrying the statewide reach onto a subset produces a figure
/// whose two halves describe different populations.
pub fn total<F>(rows: &[Refreshed], mut include: F) -> Total
where
    F: FnMut(&Refreshed) -> bool,
{
    let selected: Vec<&Refreshed> = rows.iter().filter(|row| include(row)).collect();
    Total {
        dollars: selected.iter().map(|row| row.dollars()).sum(),
        base_cost_dollars: selected.iter().map(|row| row.base_cost_delta).sum(),
        categorical_dollars: selected.iter().map(|row| row.categorical_delta).sum(),
        adm: selected.iter().map(|row| row.adm).sum(),
        reach: Reach {
            districts: selected.len(),
            gainers: selected.iter().filter(|row| row.dollars() > MOVED).count(),
            // Counted rather than taken as a residual, for the reason issue #125 records: as a
            // residual every "the classes partition the panel" assertion becomes an identity.
            unmoved: selected.iter().filter(|row| !row.moved()).count(),
            held_throughout: selected.iter().filter(|row| row.held_throughout()).count(),
            lifted_off: selected
                .iter()
                .filter(|row| row.on_guarantee_before && !row.on_guarantee_after)
                .count(),
        },
    }
}

/// Convenience for the statewide figure.
#[must_use]
pub fn statewide(rows: &[Refreshed]) -> Total {
    total(rows, |_| true)
}

/// An `Outcome` under the refresh, for a caller that wants the whole computation rather than the
/// two aid figures.
///
/// Kept because [`Refreshed`] deliberately carries only what a reach question needs, and a caller
/// asking what happened to one district's minimum-share status or transportation needs the rest.
#[must_use]
pub fn outcome(record: &DistrictRecord, statewide: &Statewide) -> Outcome {
    apply(
        record,
        &Policy {
            base_cost_scale: scale_for(record),
            ..Policy::current_law()
        },
        statewide,
        record.current_year_adm,
    )
}
