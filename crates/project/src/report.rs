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
    /// Total transportation aid.
    ///
    /// Carried beside realized aid rather than inside it, because the department publishes
    /// transportation as its own line and `[G] Total` is not part of core foundation funding.
    /// [`Self::total_state_support`] is where the two meet.
    pub transportation: Dollars,
    /// Total formula transition supplement, `[K]`.
    ///
    /// The third channel, carried for the reason transportation is: it is outside core foundation
    /// funding and inside `[R] Total State Support`. It is also the one that **responds to the
    /// others** — a lever that cuts realized aid or transportation is partly offset by this
    /// rising, so a cost quoted without it is the gross movement rather than the net.
    pub transition_supplement: Dollars,
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
            transportation: outcomes.iter().map(|o| o.transportation).sum(),
            transition_supplement: outcomes.iter().map(|o| o.transition_supplement).sum(),
        }
    }

    /// Realized aid, transportation and the transition supplement together.
    #[must_use]
    pub fn total_state_support(&self) -> Dollars {
        self.realized_aid + self.transportation + self.transition_supplement
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
    /// Change in total state aid, across every channel a lever reaches.
    ///
    /// # It is total state support and not realized aid, and the difference is a lever
    ///
    /// This read `policy.realized_aid - baseline.realized_aid` while every lever moved core
    /// foundation funding and nothing else, so the two were the same figure. The transportation
    /// floor is not in core foundation funding: a draft that moved it would have priced at
    /// **exactly zero**, which in a cost column reads as *this bill is free* — the reading
    /// [`crate::drafts::Priced::cost`] returns an `Option` to prevent, reintroduced underneath
    /// it by an aggregate that could not see the channel.
    #[must_use]
    pub fn cost(&self) -> Dollars {
        self.policy.total_state_support() - self.baseline.total_state_support()
    }

    /// The part of [`Self::cost`] that is core foundation funding.
    #[must_use]
    pub fn foundation_cost(&self) -> Dollars {
        self.policy.realized_aid - self.baseline.realized_aid
    }

    /// And the part that is transportation.
    #[must_use]
    pub fn transportation_cost(&self) -> Dollars {
        self.policy.transportation - self.baseline.transportation
    }

    /// Districts whose aid rises, across every channel a lever reaches.
    #[must_use]
    pub fn gainers(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| o.total_delta() > 0.005)
            .count()
    }

    /// Districts whose aid falls.
    #[must_use]
    pub fn losers(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| o.total_delta() < -0.005)
            .count()
    }

    /// Districts the policy does not reach at all.
    ///
    /// The number worth leading with for anything that changes the formula, because the
    /// guarantee is what makes it large.
    ///
    /// Counted, not subtracted. This was `outcomes.len() - gainers() - losers()`, which made
    /// `gainers + losers + unmoved == outcomes.len()` an identity — and that sum is asserted in
    /// `scenario-delta` as "the three classes must partition the panel", where it could not fail
    /// however the thresholds were set (#125). Written as its own filter, the sum is a real claim
    /// about the three predicates: that they leave no district in none of them and none in two.
    #[must_use]
    pub fn unmoved(&self) -> usize {
        self.outcomes
            .iter()
            .filter(|o| (-0.005..=0.005).contains(&o.total_delta()))
            .count()
    }
}

/// The forecast half: the same policy at projected enrollment.
///
/// # Two measures, and the projection has to carry both
///
/// Every field here came in a pair once [`PolicyEffect::cost`] widened to total state support.
/// A projection reported on [`Self::realized_aid`] alone is a projection of
/// [`crate::biennium::Measure::FoundationAid`], and `[K]` is outside that — so a policy whose
/// whole effect is to move money into `[K]` projects as though the money had gone.
///
/// That is not a rounding difference. `scenario/guarantee-phase-out` published *"removing the
/// guarantee nearly doubles the state's exposure to enrollment forecast error"* on the foundation
/// band — 4.31% against 7.77% — and on total state support the same two runs are 4.23% against
/// **4.16%**: the exposure does not widen at all, because `[K]` is itself a floor against a fixed
/// FY2021 total and inherits the shock-absorber role the guarantee gave up. The claim is true of
/// the hold-harmless *system* and false of the guarantee alone, and one measure cannot tell those
/// apart.
#[derive(Debug, Clone, PartialEq)]
pub struct EnrollmentEffect {
    /// The fiscal year projected to.
    pub fiscal_year: FiscalYear,
    /// Central estimate of total realized aid — `[H] + [I]`, the narrow measure.
    pub realized_aid: Dollars,
    /// Low end, from the projection interval.
    pub low: Dollars,
    /// High end.
    pub high: Dollars,
    /// Central estimate of total state support — realized aid, transportation and `[K]`.
    pub total_state_support: Dollars,
    /// Low end of the same band.
    ///
    /// Still the low end, and that is a property of the construction rather than a hope: a
    /// district `[K]` insulates is held at `[L1]` in total, so its contribution is *flat* in
    /// enrollment, and every other district's is increasing. `[K]` damps the band and cannot
    /// invert it. Asserted by `the_backstop_damps_the_band_without_inverting_it` in this module.
    pub total_low: Dollars,
    /// High end.
    pub total_high: Dollars,
    /// `[K]` at projected enrollment, central estimate.
    ///
    /// Carried because it is the interesting quantity in a projection of a guarantee retirement:
    /// it is $926.8m at FY2032 against $63.6m at FY2027, which is the state's exposure arriving
    /// somewhere else rather than going away.
    pub transition_supplement: Dollars,
    /// Projected total ADM.
    pub adm: f64,
    /// Districts on the guarantee at projected enrollment.
    pub on_guarantee: usize,
    /// The method used for every district's enrollment.
    pub method: Method,
    /// The dispersion the interval rests on.
    pub prior: Prior,
}

impl EnrollmentEffect {
    /// Half the foundation-aid band's width, as a share of the central estimate.
    ///
    /// The quantity the corpus writes as `(+/-4.3%)`. Zero when the central estimate is, which is
    /// not a band of zero width but a panel nothing projected.
    #[must_use]
    pub fn half_width(&self) -> f64 {
        if self.realized_aid == 0.0 {
            return 0.0;
        }
        (self.high - self.low) / 2.0 / self.realized_aid
    }

    /// The same on total state support, which is the measure a cost is quoted in.
    #[must_use]
    pub fn total_half_width(&self) -> f64 {
        if self.total_state_support == 0.0 {
            return 0.0;
        }
        (self.total_high - self.total_low) / 2.0 / self.total_state_support
    }
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
    // The same three on total state support, accumulated beside rather than derived after: `[K]`
    // is a function of each district's own projected aid, so a statewide supplement cannot be
    // recovered from a statewide total.
    let mut total = 0.0;
    let mut total_low = 0.0;
    let mut total_high = 0.0;
    let mut supplement = 0.0;
    let mut adm = 0.0;
    let mut on_guarantee = 0;

    // Resolved once against the whole panel, because the two statewide statistics a lever can
    // move are not properties of any one district. Resolved at modelled enrollment rather than
    // projected: a forecast moves counts, and rebasing the DPIA denominator on projected ADM
    // would price an enrollment change as a policy change.
    let statewide = crate::policy::Statewide::under(panel, policy);

    for record in panel {
        // Per district, because `Method::Shrunk` carries a `toward` the feed cannot choose once.
        let method = record.projection_method(method);
        let series = project(&record.adm_observations(), through, method, prior);
        let Some(projected) = series
            .iter()
            .find(|p| p.fiscal_year == through && p.basis == Basis::Projected)
        else {
            continue;
        };
        let at = |value: f64| apply(record, policy, &statewide, value);
        let central = at(projected.point);
        point += central.realized_aid;
        total += central.total_state_support();
        supplement += central.transition_supplement;
        // A smaller district draws less aid, so the enrollment band's low end is the aid band's
        // low end. That holds because every lever here is monotone in ADM — and it survives `[K]`,
        // which is flat in ADM for an insulated district and so damps the band without reordering
        // its ends.
        let at_low = at(projected.low);
        let at_high = at(projected.high);
        low += at_low.realized_aid;
        high += at_high.realized_aid;
        total_low += at_low.total_state_support();
        total_high += at_high.total_state_support();
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
        total_state_support: total,
        total_low,
        total_high,
        transition_supplement: supplement,
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
    use crate::policy::{Backstop, GuaranteeRule};
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

    /// Removing the guarantee does **not** save what the guarantee costs, and this test asserted
    /// for several phases that it did.
    ///
    /// `[K]`, the formula transition supplement, tops a district up to its FY2021 base from a
    /// total that already contains the guarantee — so a guarantee that stops being paid is a
    /// shortfall `[K]` makes good. The saving is a tenth of the headline and 127 of the 294
    /// guaranteed districts are made whole. See `crate::hold_harmless` and [`Backstop`].
    #[test]
    fn removing_the_guarantee_alone_saves_a_tenth_of_what_it_costs() {
        let panel = panel();
        let effect = simulate(
            &panel,
            &Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
        );
        assert_eq!(effect.policy.on_guarantee, 0);
        assert_eq!(
            effect.gainers(),
            0,
            "retiring a floor cannot pay anyone more"
        );

        // The saving is far short of the guarantee, and the shortfall is the backstop rising.
        let headline = -effect.baseline.guarantee;
        assert!(
            effect.cost() > headline * 0.2,
            "removing the guarantee saved ${:.1}m against a guarantee of ${:.1}m",
            effect.cost() / 1e6,
            effect.baseline.guarantee / 1e6
        );
        let absorbed = 1.0 - effect.cost() / headline;
        assert!(
            (absorbed - 0.909).abs() < 0.01,
            "the backstop absorbs {:.1}%, not 90.9%",
            absorbed * 100.0
        );

        // And it is the same districts: the ones `[K]` makes whole stop being losers.
        assert!(
            effect.losers() < effect.baseline.on_guarantee,
            "every guaranteed district still loses, so nothing was absorbed"
        );
        assert_eq!(effect.losers(), 167);
    }

    /// Repealing both devices saves what both cost, which is the run the old test meant.
    #[test]
    fn repealing_the_backstop_beside_it_saves_what_both_cost() {
        let panel = panel();
        let effect = simulate(
            &panel,
            &Policy {
                guarantee: GuaranteeRule::Removed,
                backstop: Backstop::Repealed,
                ..Policy::current_law()
            },
        );
        let both = effect.baseline.guarantee + effect.baseline.transition_supplement;
        assert!(
            (effect.cost() + both).abs() < 1.0,
            "saved ${:.2} against ${both:.2}",
            -effect.cost()
        );
        assert_eq!(effect.policy.on_guarantee, 0);
        assert_eq!(effect.gainers(), 0);

        // Seventeen districts draw `[K]` without being on the guarantee, so more districts lose
        // here than the guarantee alone ever reached.
        assert_eq!(effect.losers(), 311);
        assert!(effect.losers() > effect.baseline.on_guarantee);
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

    /// `[K]` damps the projected band and does not reorder its ends.
    ///
    /// Stated as a test rather than assumed in a comment because the two quantities pull opposite
    /// ways: realized aid falls with enrollment and `[K]` rises to fill the gap it leaves. What
    /// makes the ordering safe is that `[K]` is a top-up to a *ceiling* — a district it insulates
    /// contributes a flat `[L1]` at every enrollment, so the worst it can do is remove a district
    /// from the band rather than invert it.
    #[test]
    fn the_backstop_damps_the_band_without_inverting_it() {
        let panel = panel();
        let method = Method::Damped {
            rate: 0.0,
            damping: DEFAULT_DAMPING,
        };
        let prior = enrollment_growth_prior(&panel, ONE_SIGMA);
        for policy in [
            Policy::current_law(),
            Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
        ] {
            let effect = forecast(&panel, &policy, FiscalYear(2032), method, prior);
            assert!(
                effect.total_low < effect.total_state_support
                    && effect.total_state_support < effect.total_high,
                "band {} .. {} .. {}",
                effect.total_low,
                effect.total_state_support,
                effect.total_high
            );
            assert!(
                effect.total_half_width() < effect.half_width() + f64::EPSILON,
                "the wide measure is wider: {} vs {}",
                effect.total_half_width(),
                effect.half_width()
            );
        }
    }

    /// Retiring the guarantee does **not** widen the state's exposure to enrollment error.
    ///
    /// The corpus published the opposite — *"removing the guarantee nearly doubles the state's
    /// exposure to enrollment forecast error"* — on a band measured in foundation aid, where the
    /// claim is true: 4.31% becomes 7.77%. On total state support the same two runs are 4.23% and
    /// 4.16%, because `[K]` is a floor against a fixed FY2021 total and takes over the absorbing
    /// the guarantee stops doing. The budget-stability argument belongs to the hold-harmless system
    /// and not to the guarantee, and repealing both is what recovers the doubling.
    #[test]
    fn the_backstop_inherits_the_guarantees_budget_stability() {
        let panel = panel();
        let method = Method::Damped {
            rate: 0.0,
            damping: DEFAULT_DAMPING,
        };
        let prior = enrollment_growth_prior(&panel, ONE_SIGMA);
        let at = |policy: Policy| forecast(&panel, &policy, FiscalYear(2032), method, prior);

        let current = at(Policy::current_law());
        let removed = at(Policy {
            guarantee: GuaranteeRule::Removed,
            ..Policy::current_law()
        });
        let both = at(Policy {
            guarantee: GuaranteeRule::Removed,
            backstop: Backstop::Repealed,
            ..Policy::current_law()
        });

        // The claim as published, on the narrow measure: it nearly doubles.
        assert!(
            removed.half_width() > current.half_width() * 1.7,
            "{:.4} against {:.4}",
            removed.half_width(),
            current.half_width()
        );
        // And on the measure a cost is quoted in, it does not widen at all.
        assert!(
            removed.total_half_width() < current.total_half_width(),
            "{:.4} against {:.4}",
            removed.total_half_width(),
            current.total_half_width()
        );
        // The doubling is recovered only when the backstop goes too.
        assert!(
            both.total_half_width() > current.total_half_width() * 1.6,
            "{:.4} against {:.4}",
            both.total_half_width(),
            current.total_half_width()
        );
        // `[K]` is where the exposure went: $63.6m at FY2027, an order of magnitude more here.
        assert!(
            removed.transition_supplement > 900_000_000.0,
            "{}",
            removed.transition_supplement
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
