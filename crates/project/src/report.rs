//! Running a scenario, and reporting simulation apart from forecast.
//!
//! # The rule this module exists to enforce
//!
//! Simulation and projection are not the same epistemic act. Re-running the formula with a
//! changed parameter is deterministic: given the inputs, the answer is the answer. Projecting
//! enrollment five years out is a forecast, with everything that implies.
//!
//! A [`Run`] therefore reports two things and never their sum as a headline. The **policy
//! effect** is what the levers do at observed enrollment — exact, reproducible, and the number
//! a legislator is actually asking about. The **enrollment effect** is what the same policy
//! does at projected enrollment, with an interval. They are printed on separate lines with the
//! interval attached to only one of them, because a combined figure inherits the forecast's
//! error while looking like the simulation's precision.

use edfund_core::{Dollars, FiscalYear};

use crate::panel::DistrictRecord;
use crate::policy::{apply, apply_all, Outcome, Policy};
use crate::series::{project, standard_deviation, Basis, Method, Prior, ONE_SIGMA};

/// Statewide totals for one set of outcomes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Totals {
    /// Districts counted.
    pub districts: usize,
    /// Districts the guarantee pays.
    pub on_guarantee: usize,
    /// Districts whose base cost aid is set by the minimum state share.
    pub at_minimum_state_share: usize,
    /// Total realized state aid.
    pub realized_aid: Dollars,
    /// Total formula aid, before the guarantee.
    pub formula_aid: Dollars,
    /// Total guarantee top-up.
    pub guarantee: Dollars,
    /// Total ADM.
    pub adm: f64,
}

impl Totals {
    /// Aggregate a set of outcomes.
    #[must_use]
    pub fn of(outcomes: &[Outcome]) -> Self {
        Self {
            districts: outcomes.len(),
            on_guarantee: outcomes.iter().filter(|o| o.on_guarantee).count(),
            at_minimum_state_share: outcomes.iter().filter(|o| o.at_minimum_state_share).count(),
            realized_aid: outcomes.iter().map(|o| o.realized_aid).sum(),
            formula_aid: outcomes.iter().map(|o| o.formula_aid).sum(),
            guarantee: outcomes.iter().map(|o| o.guarantee).sum(),
            adm: outcomes.iter().map(|o| o.adm).sum(),
        }
    }
}

/// The deterministic half: what the levers do at observed enrollment.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyEffect {
    /// Totals under current law.
    pub baseline: Totals,
    /// Totals under the policy.
    pub policy: Totals,
    /// Per-district outcomes under the policy.
    pub outcomes: Vec<Outcome>,
}

impl PolicyEffect {
    /// Change in total state aid.
    #[must_use]
    pub fn cost(&self) -> Dollars {
        self.policy.realized_aid - self.baseline.realized_aid
    }

    /// Districts whose aid rises.
    #[must_use]
    pub fn gainers(&self) -> usize {
        self.outcomes.iter().filter(|o| o.delta() > 0.005).count()
    }

    /// Districts whose aid falls.
    #[must_use]
    pub fn losers(&self) -> usize {
        self.outcomes.iter().filter(|o| o.delta() < -0.005).count()
    }

    /// Districts the policy does not reach at all.
    ///
    /// The number worth leading with for anything that changes the formula, because the
    /// guarantee is what makes it large.
    #[must_use]
    pub fn unmoved(&self) -> usize {
        self.outcomes.len() - self.gainers() - self.losers()
    }
}

/// The forecast half: the same policy at projected enrollment.
#[derive(Debug, Clone, PartialEq)]
pub struct EnrollmentEffect {
    /// The fiscal year projected to.
    pub fiscal_year: FiscalYear,
    /// Central estimate of total realized aid.
    pub realized_aid: Dollars,
    /// Low end, from the projection interval.
    pub low: Dollars,
    /// High end.
    pub high: Dollars,
    /// Projected total ADM.
    pub adm: f64,
    /// Districts on the guarantee at projected enrollment.
    pub on_guarantee: usize,
    /// The method used for every district's enrollment.
    pub method: Method,
    /// The dispersion the interval rests on.
    pub prior: Prior,
}

/// A scenario run: a policy, its deterministic effect, and optionally its forecast effect.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// The levers.
    pub policy: Policy,
    /// What they do at observed enrollment. Exact.
    pub policy_effect: PolicyEffect,
    /// What they do at projected enrollment. A forecast, and absent unless asked for.
    pub enrollment_effect: Option<EnrollmentEffect>,
}

/// Cross-sectional standard deviation of annual enrollment growth across the panel.
///
/// This is the prior every district's projection interval rests on. It is not that district's
/// variability — three observations cannot give that — it is how much districts differ from one
/// another, used as a floor. Districts with a partial history are skipped rather than treated as
/// having grown from zero.
#[must_use]
pub fn enrollment_growth_prior(panel: &[DistrictRecord], z: f64) -> Prior {
    let rates: Vec<f64> = panel
        .iter()
        .filter_map(|record| {
            let [first, _, last] = record.adm_history;
            (first > 0.0 && last > 0.0).then(|| (last / first).powf(0.5) - 1.0)
        })
        .collect();
    Prior {
        sigma: standard_deviation(&rates),
        z,
        source: "cross-sectional spread of district annual enrolled-ADM growth, FY2024-FY2026",
    }
}

/// Run a policy against the panel at observed enrollment.
#[must_use]
pub fn simulate(panel: &[DistrictRecord], policy: &Policy) -> PolicyEffect {
    let outcomes = apply_all(panel, policy);
    PolicyEffect {
        baseline: Totals::of(&apply_all(panel, &Policy::current_law())),
        policy: Totals::of(&outcomes),
        outcomes,
    }
}

/// Run a policy at enrollment projected to `through`.
///
/// The interval is built by re-running the whole formula at the low and high ends of every
/// district's enrollment band rather than by scaling the central answer. That matters because
/// the guarantee is a `max`: a district can be on formula at high enrollment and on the
/// guarantee at low enrollment, and the aid curve has a kink there that no scaling reproduces.
#[must_use]
pub fn forecast(
    panel: &[DistrictRecord],
    policy: &Policy,
    through: FiscalYear,
    method: Method,
    prior: Prior,
) -> EnrollmentEffect {
    let mut point = 0.0;
    let mut low = 0.0;
    let mut high = 0.0;
    let mut adm = 0.0;
    let mut on_guarantee = 0;

    for record in panel {
        let series = project(&record.adm_observations(), through, method, prior);
        let Some(projected) = series
            .iter()
            .find(|p| p.fiscal_year == through && p.basis == Basis::Projected)
        else {
            continue;
        };
        let at = |value: f64| apply(record, policy, value);
        let central = at(projected.point);
        point += central.realized_aid;
        // A smaller district draws less aid, so the enrollment band's low end is the aid band's
        // low end. That holds because every lever here is monotone in ADM.
        low += at(projected.low).realized_aid;
        high += at(projected.high).realized_aid;
        adm += projected.point;
        if central.on_guarantee {
            on_guarantee += 1;
        }
    }

    EnrollmentEffect {
        fiscal_year: through,
        realized_aid: point,
        low,
        high,
        adm,
        on_guarantee,
        method,
        prior,
    }
}

/// Run a policy, deterministically and — if `through` is given — as a forecast too.
#[must_use]
pub fn run(
    panel: &[DistrictRecord],
    policy: &Policy,
    through: Option<FiscalYear>,
    method: Method,
) -> Run {
    Run {
        policy: *policy,
        policy_effect: simulate(panel, policy),
        enrollment_effect: through.map(|year| {
            forecast(
                panel,
                policy,
                year,
                method,
                enrollment_growth_prior(panel, ONE_SIGMA),
            )
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::panel;
    use crate::policy::GuaranteeRule;
    use crate::series::DEFAULT_DAMPING;

    #[test]
    fn current_law_costs_nothing_and_moves_no_district() {
        let panel = panel();
        let effect = simulate(&panel, &Policy::current_law());
        assert!(effect.cost().abs() < 1.0, "{}", effect.cost());
        assert_eq!(effect.gainers(), 0);
        assert_eq!(effect.losers(), 0);
        assert_eq!(effect.unmoved(), panel.len());
    }

    #[test]
    fn removing_the_guarantee_saves_exactly_what_the_guarantee_costs() {
        let panel = panel();
        let effect = simulate(
            &panel,
            &Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
        );
        assert!((effect.cost() + effect.baseline.guarantee).abs() < 1.0);
        assert_eq!(effect.policy.on_guarantee, 0);
        assert_eq!(effect.gainers(), 0);
        assert_eq!(effect.losers(), effect.baseline.on_guarantee);
    }

    #[test]
    fn a_base_cost_increase_leaves_most_districts_untouched() {
        // The finding the guarantee produces, restated as a lever: raising base cost reaches
        // only districts the formula pays, and the formula pays about half of them.
        let panel = panel();
        let effect = simulate(
            &panel,
            &Policy {
                base_cost_scale: 1.05,
                ..Policy::current_law()
            },
        );
        assert!(effect.cost() > 0.0);
        assert_eq!(effect.losers(), 0, "raising base cost cannot cut anyone");
        assert!(
            effect.unmoved() > panel.len() / 3,
            "only {} of {} districts unmoved",
            effect.unmoved(),
            panel.len()
        );
    }

    #[test]
    fn the_growth_prior_is_positive_and_small() {
        let prior = enrollment_growth_prior(&panel(), ONE_SIGMA);
        assert!(
            prior.sigma > 0.001 && prior.sigma < 0.15,
            "sigma {} is not a plausible enrollment growth dispersion",
            prior.sigma
        );
        assert!(prior.source.contains("cross-sectional"));
    }

    #[test]
    fn a_forecast_carries_an_interval_that_brackets_its_point() {
        let panel = panel();
        let effect = forecast(
            &panel,
            &Policy::current_law(),
            FiscalYear(2032),
            Method::Damped {
                rate: 0.0,
                damping: DEFAULT_DAMPING,
            },
            enrollment_growth_prior(&panel, ONE_SIGMA),
        );
        assert!(effect.low < effect.realized_aid && effect.realized_aid < effect.high);
        assert_eq!(effect.fiscal_year, FiscalYear(2032));
    }

    #[test]
    fn the_forecast_interval_is_narrower_than_the_enrollment_interval_that_drives_it() {
        // Because the guarantee absorbs enrollment loss for half the state. This is the
        // clearest single statement of what the guarantee does to a projection: it makes state
        // cost much less sensitive to enrollment than the enrollment forecast is uncertain.
        let panel = panel();
        let prior = enrollment_growth_prior(&panel, ONE_SIGMA);
        let effect = forecast(
            &panel,
            &Policy::current_law(),
            FiscalYear(2032),
            Method::Damped {
                rate: 0.0,
                damping: DEFAULT_DAMPING,
            },
            prior,
        );
        let aid_width = (effect.high - effect.low) / effect.realized_aid;
        let enrollment_width = 2.0 * prior.spread(2032 - 2026);
        assert!(
            aid_width < enrollment_width,
            "aid band {aid_width:.4} should be tighter than enrollment band {enrollment_width:.4}"
        );
    }

    #[test]
    fn a_run_keeps_the_two_halves_apart() {
        let panel = panel();
        let run = run(
            &panel,
            &Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            Some(FiscalYear(2030)),
            Method::Damped {
                rate: 0.0,
                damping: DEFAULT_DAMPING,
            },
        );
        // The deterministic half has no interval; the forecast half has one. There is
        // deliberately no field holding their sum.
        assert!(run.policy_effect.cost() < 0.0);
        let enrollment = run.enrollment_effect.expect("asked for");
        assert!(enrollment.high > enrollment.low);
    }

    #[test]
    fn no_forecast_is_produced_unless_one_is_asked_for() {
        let run = run(&panel(), &Policy::current_law(), None, Method::LastObserved);
        assert!(run.enrollment_effect.is_none());
    }

    #[test]
    fn totals_add_up_across_the_panel() {
        let panel = panel();
        let outcomes = apply_all(&panel, &Policy::current_law());
        let totals = Totals::of(&outcomes);
        assert_eq!(totals.districts, panel.len());
        assert!((totals.realized_aid - totals.formula_aid - totals.guarantee).abs() < 1.0);
    }
}
