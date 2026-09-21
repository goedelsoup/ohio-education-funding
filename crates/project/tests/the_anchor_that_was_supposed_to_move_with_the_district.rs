//! What a rolling anchor for the guarantee costs, which is more than the fixed one and not less.
//!
//! [#390](https://github.com/goedelsoup/ohio-education-funding/issues/390) recorded the design
//! question under the whole guarantee arc: R.C. 3317.019 holds a district at **FY2020**, a fixed
//! anchor receding into the past is what makes a temporary device permanent, and a rolling anchor
//! *"is the obvious alternative and has never been priced here"*. It also recorded why — a rolling
//! anchor was thought to need several consecutive years of comparable aid per district, which the
//! corpus does not have.
//!
//! [`project::rolling_anchor`] prices it. The data objection turns out not to bind, the
//! hypothesis the question rested on turns out to run the other way, and what survives is a
//! different shape and the same conclusion #400 reached about a different one.
//!
//! # The data objection does not bind, because a price is not a backtest
//!
//! A rolling anchor is **recursive**: the floor in a year is a function of the aid the model paid
//! in the year before it. So it walks forward out of the FY2027 model under the projection this
//! crate already has, and needs no historical aid at all. Three consecutive comparable years is
//! what a *backtest* needs.
//!
//! And they would not support one. The FY2025 payment report, the FY2026 model and the FY2027
//! panel state the same FY2020 funding base for **all 609** districts, which is the check that
//! they are the same districts on the same lines. But `[Hb]`, the formula's own output, falls a
//! median **5.2%** then **6.0%**, in **549** and **540** of the 609, and the two legs agree across
//! districts at only **0.545**. A district trend would repeat. This is the regime moving, which is
//! what #390 warned it would be.
//!
//! # A prior-year anchor is a ratchet, and ratchets are expensive
//!
//! `realized = max(formula, anchor)`, so last year's realized aid is at or above last year's
//! formula amount by construction, and an anchor set to it means aid can never fall — for
//! anybody, ever. The fixed anchor catches districts below a level chosen once; a rolling one
//! catches every district whose formula amount falls at all, and in Ohio most of them do most
//! years.
//!
//! | FY2027, anchor replaced | `[I]` statewide | districts held |
//! |---|--:|--:|
//! | the FY2020 funding base, as enacted | $878,954,627 | 294 |
//! | FY2026 realized aid | $1,025,415,233 | 455 |
//! | the FY2025-FY2026 mean | $1,082,158,897 | 446 |
//! | FY2026 `[Hb]` | $540,591,743 | **540** |
//!
//! # And for the 89 it is the same number to the cent
//!
//! #390's argument was that *"50 of 89 districts crossed onto the guarantee between FY2026 and
//! FY2027; under a prior-year anchor most of them would never have crossed at all, because the
//! anchor would have moved with them."* The anchor does move with them. It moves **up**, because
//! a `max` has no other direction, and then holds. Every member of the cluster is on the floor in
//! FY2027, so a ratchet started there is the FY2020 base: walked to FY2032 the two rules give the
//! cluster **$144,560,557.55**, **89** districts held, and the same total state support to the
//! cent.
//!
//! # A decay is the shape the question was reaching for
//!
//! "No district's foundation aid falls more than `x`% in a year" is date-free, reaches a district
//! that crosses in any year, follows a district **down**, and extinguishes itself for one whose
//! enrollment settles.
//!
//! # And `[K]` absorbs it, which is #400's answer to a different question
//!
//! Cutting the cluster's guarantee from $144.6m to $45.2m with a 2% cap takes **$3,949,559** of
//! total state support: **96.0% absorbed**. The share of what the cluster receives that rides on a
//! hold-harmless does not move at all — 26.94% under the enacted anchor at FY2036 against 26.81%
//! under every decay priced. **Changing the anchor changes which hold-harmless pays, not whether
//! one does**, and it moves the money from R.C. 3317.019 to the uncodified Section 265.225.
//!
//! # The identity this rests on
//!
//! [`rolling_anchor::walk`] at [`Anchor::Fixed`] reproduces
//! [`decline_adjustment::nothing`](project::decline_adjustment::nothing) exactly — the whole
//! FY2032 and FY2036 table #400 published, to the cent, from a recursion rather than a single
//! call. That is what licenses reading the other rules' figures as the same quantity.

use edfund_core::FiscalYear;
use project::decline_adjustment;
use project::enrollment_decline;
use project::panel::{self, DistrictRecord};
use project::report::enrollment_growth_prior;
use project::rolling_anchor::{self, Anchor, Held, Observed};
use project::series::{Method, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT, ONE_SIGMA};
use std::collections::BTreeSet;

/// The fixture stores dollars to the cent, so a reproduction may differ by rounding and nothing
/// more. The same constant `the_adjustment_the_floors_absorb` uses on the same quantities.
const CENT: f64 = 0.01;

/// The shipped projection method, as `the_adjustment_the_floors_absorb` builds it.
fn shipped() -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: DEFAULT_DAMPING,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    }
}

/// The same with the damping off — #400's convention for the honest end of the drift.
fn undamped() -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: 1.0,
        weight: DEFAULT_SHRINK_WEIGHT,
        toward: 0.0,
    }
}

/// The 89 districts on the guarantee for no reason but having fewer children than in FY2020.
fn cluster(panel: &[DistrictRecord]) -> BTreeSet<String> {
    enrollment_decline::cluster(panel)
        .into_iter()
        .map(|record| record.irn.clone())
        .collect()
}

/// One walk, summarized statewide and over the cluster.
fn walk(panel: &[DistrictRecord], through: u16, method: Method, anchor: Anchor) -> (Held, Held) {
    let year = FiscalYear(through);
    let standings = rolling_anchor::walk(panel, year, method, anchor);
    let mine = cluster(panel);
    (
        rolling_anchor::summarize(year, standings.values()),
        rolling_anchor::summarize(year, standings.values().filter(|s| mine.contains(&s.irn))),
    )
}

/// The fixed anchor walked year by year is the fixed anchor computed in one step.
///
/// The identity everything else rests on. `decline_adjustment::nothing` calls `policy::apply`
/// once at the target year because a constant floor needs no intervening years; this module has
/// to compute all of them because a rolling floor does. They must agree where the floor is
/// constant, and they agree to the cent on every quantity #400 published.
#[test]
fn walking_a_fixed_anchor_reproduces_the_single_step_it_replaces() {
    let districts = panel::panel();
    let prior = enrollment_growth_prior(&districts, ONE_SIGMA);
    for (through, method) in [(2032u16, shipped()), (2036, undamped())] {
        let published = decline_adjustment::nothing(&districts, FiscalYear(through), method, prior);
        let (_, walked) = walk(&districts, through, method, Anchor::Fixed);

        assert!(
            (walked.adm - published.adm).abs() < 0.05,
            "FY{through} ADM: {:.1} walked against {:.1}",
            walked.adm,
            published.adm
        );
        assert!(
            (walked.guarantee - published.guarantee).abs() < CENT,
            "FY{through} guarantee: {:.2} walked against {:.2}",
            walked.guarantee,
            published.guarantee
        );
        assert!(
            (walked.transition_supplement - published.transition_supplement).abs() < CENT,
            "FY{through} [K]: {:.2} against {:.2}",
            walked.transition_supplement,
            published.transition_supplement
        );
        assert!(
            (walked.total_state_support - published.total_state_support).abs() < CENT,
            "FY{through} total state support: {:.2} against {:.2}",
            walked.total_state_support,
            published.total_state_support
        );
        assert_eq!(
            walked.on_the_floor, published.on_the_floor,
            "FY{through} districts held"
        );
        assert!(
            (walked.held_share() - published.held_share()).abs() < 1e-9,
            "FY{through} held share"
        );
    }
}

/// The three published years are the same districts on the same lines, and are not a series.
///
/// #390 said the comparability of the FY2025 report "is the first thing to establish, not
/// assume". Established: the join holds perfectly and the quantity does not.
#[test]
fn the_three_published_years_join_exactly_and_still_cannot_carry_a_district_series() {
    let districts = panel::panel();
    let measured = rolling_anchor::comparability(&districts);

    assert_eq!(
        measured.districts, 609,
        "every panel district is in all three"
    );
    assert_eq!(
        measured.funding_base_identical, measured.districts,
        "and all 609 state the same FY2020 funding base in all three reports"
    );

    assert!(
        (measured.computed_median_first_leg + 0.0523).abs() < 0.002
            && (measured.computed_median_second_leg + 0.0595).abs() < 0.002,
        "[Hb] should fall a median 5.2% then 6.0%, falls {:+.4} then {:+.4}",
        measured.computed_median_first_leg,
        measured.computed_median_second_leg
    );
    assert_eq!(measured.computed_falls_first_leg, 549);
    assert_eq!(measured.computed_falls_second_leg, 540);

    assert!(
        (measured.leg_correlation - 0.545).abs() < 0.02,
        "the two legs should agree at about 0.545 across districts, agree at {:.3}",
        measured.leg_correlation
    );
    assert!(
        !measured.supports_a_backtest(),
        "so the three years cannot carry a backtest of a rolling anchor"
    );
}

/// Replacing the anchor with the district's own prior year costs more and reaches more.
///
/// The first half of #390's answer, on the modelled year with nothing projected: one number
/// replaced in the year the crossing actually happened.
#[test]
fn a_prior_year_anchor_holds_a_hundred_and_sixty_one_more_districts_than_the_enacted_one() {
    let districts = panel::panel();
    let enacted = rolling_anchor::at_the_modelled_year(&districts, Observed::Fy2020Base);
    assert_eq!(enacted.districts, 609);
    assert!(
        (enacted.guarantee - 878_954_627.38).abs() < CENT && enacted.on_the_floor == 294,
        "the control should be the published guarantee: {:.2} over {} districts",
        enacted.guarantee,
        enacted.on_the_floor
    );

    let rolling = rolling_anchor::at_the_modelled_year(&districts, Observed::PriorYearRealized);
    assert!(
        (rolling.guarantee - 1_025_415_233.33).abs() < CENT,
        "a prior-year anchor writes {:.2}",
        rolling.guarantee
    );
    assert_eq!(rolling.on_the_floor, 455);
    assert_eq!(
        rolling.on_the_floor - enacted.on_the_floor,
        161,
        "which is 161 districts the enacted anchor does not reach"
    );
    assert!(
        rolling.guarantee > enacted.guarantee,
        "and it is the more expensive rule, not the cheaper one"
    );

    // The two-year mean is more expensive again: averaging in FY2025 raises the floor where aid
    // has been falling, which is the whole population under discussion.
    let mean = rolling_anchor::at_the_modelled_year(&districts, Observed::TwoYearMeanRealized);
    assert!(
        mean.guarantee > rolling.guarantee,
        "a two-year mean should be dearer still: {:.2} against {:.2}",
        mean.guarantee,
        rolling.guarantee
    );

    // On the comparable quantity it is cheaper in dollars and wider in population than either.
    let computed = rolling_anchor::at_the_modelled_year(&districts, Observed::PriorYearComputed);
    assert!(
        computed.guarantee < enacted.guarantee && computed.on_the_floor > rolling.on_the_floor,
        "on [Hb] the rolling anchor holds {} districts for {:.2}",
        computed.on_the_floor,
        computed.guarantee
    );
    assert_eq!(computed.on_the_floor, 540);
}

/// For the 89 the two anchors are indistinguishable, which is what refutes the hypothesis.
///
/// Not approximately: the same guarantee to the cent, the same count, the same total support.
/// Every member is on the floor in FY2027, so a ratchet started from FY2027 realized aid *is* the
/// FY2020 base. A `max` only ever moves an anchor up.
#[test]
fn the_crossing_argument_does_not_survive_the_anchor_moving_with_the_district() {
    let districts = panel::panel();
    let (fixed_all, fixed_cluster) = walk(&districts, 2032, shipped(), Anchor::Fixed);
    let (rolling_all, rolling_cluster) = walk(&districts, 2032, shipped(), Anchor::PriorYear);

    assert_eq!(fixed_cluster.districts, 89);
    assert!(
        (fixed_cluster.guarantee - 144_560_557.55).abs() < CENT,
        "{:.2}",
        fixed_cluster.guarantee
    );
    assert!(
        (rolling_cluster.guarantee - fixed_cluster.guarantee).abs() < CENT,
        "the two anchors should agree to the cent over the 89: {:.2} against {:.2}",
        rolling_cluster.guarantee,
        fixed_cluster.guarantee
    );
    assert!(
        (rolling_cluster.total_state_support - fixed_cluster.total_state_support).abs() < CENT,
        "and on total state support too"
    );
    assert_eq!(rolling_cluster.on_the_floor, fixed_cluster.on_the_floor);
    assert_eq!(rolling_cluster.on_the_floor, 89);

    // Statewide they are not the same rule at all.
    assert_eq!(fixed_all.on_the_floor, 312);
    assert_eq!(rolling_all.on_the_floor, 565);
    assert!(
        rolling_all.guarantee > fixed_all.guarantee,
        "{:.2} against {:.2}",
        rolling_all.guarantee,
        fixed_all.guarantee
    );

    assert!(Anchor::PriorYear.is_a_ratchet());
    assert!(!Anchor::Fixed.is_a_ratchet());
    assert!(!Anchor::Decayed { factor: 0.98 }.is_a_ratchet());
}

/// A cap on the annual fall is the shape that follows a district down, and it is priced.
#[test]
fn a_cap_on_the_annual_fall_prices_out_at_four_rates() {
    let districts = panel::panel();
    let expected = [
        // factor, statewide guarantee, statewide held, cluster guarantee, cluster held
        (0.99, 784_791_925.32, 277, 85_028_780.12, 71),
        (0.98, 665_088_656.83, 246, 45_151_279.08, 47),
        (0.97, 572_939_686.01, 218, 27_264_596.76, 26),
        (0.95, 423_983_112.51, 185, 12_027_817.90, 6),
    ];
    let mut previous: Option<(f64, usize)> = None;
    for (factor, statewide, held, cluster_guarantee, cluster_held) in expected {
        let (all, mine) = walk(&districts, 2032, shipped(), Anchor::Decayed { factor });
        assert!(
            (all.guarantee - statewide).abs() < CENT,
            "at {factor} the statewide guarantee is {:.2}, expected {statewide:.2}",
            all.guarantee
        );
        assert_eq!(
            all.on_the_floor, held,
            "statewide districts held at {factor}"
        );
        assert!(
            (mine.guarantee - cluster_guarantee).abs() < CENT,
            "at {factor} the cluster holds {:.2}",
            mine.guarantee
        );
        assert_eq!(mine.on_the_floor, cluster_held, "the 89 held at {factor}");

        if let Some((dearer, wider)) = previous {
            assert!(
                all.guarantee < dearer && all.on_the_floor < wider,
                "a harder cap should cost less and reach fewer: {factor}"
            );
        }
        previous = Some((all.guarantee, all.on_the_floor));
    }
}

/// It is a different population rather than a weaker floor, and statewide the sign can flip.
///
/// A decay is anchored on *current* aid, so a district well above its FY2020 base gets a higher
/// floor than the enacted rule gives it. At FY2036 a 1% cap holds more districts than the enacted
/// anchor while writing $220m less guarantee, and statewide total state support is *higher*.
#[test]
fn a_gentle_cap_holds_more_districts_than_the_enacted_anchor_for_less_money() {
    let districts = panel::panel();
    let (enacted, _) = walk(&districts, 2036, undamped(), Anchor::Fixed);
    let (capped, _) = walk(
        &districts,
        2036,
        undamped(),
        Anchor::Decayed { factor: 0.99 },
    );

    assert_eq!(enacted.on_the_floor, 388);
    assert_eq!(capped.on_the_floor, 469);
    assert!(
        capped.guarantee < enacted.guarantee,
        "{:.2} against {:.2}",
        capped.guarantee,
        enacted.guarantee
    );
    assert!(
        capped.total_state_support > enacted.total_state_support,
        "and total state support is higher under the cap: {:.2} against {:.2}",
        capped.total_state_support,
        enacted.total_state_support
    );
    assert!(
        rolling_anchor::absorbed(&enacted, &capped) > 1.0,
        "so more than the whole of the guarantee cut comes back, which is what above-1.0 means"
    );
}

/// `[K]` absorbs almost all of it for the cluster, which is #400's answer on a new instrument.
#[test]
fn the_backstop_absorbs_the_anchors_shape_as_it_absorbed_the_adjustments() {
    let districts = panel::panel();
    let (enacted_all, enacted_cluster) = walk(&districts, 2032, shipped(), Anchor::Fixed);
    let (capped_all, capped_cluster) = walk(
        &districts,
        2032,
        shipped(),
        Anchor::Decayed { factor: 0.98 },
    );

    let taken = enacted_cluster.total_state_support - capped_cluster.total_state_support;
    assert!(
        (taken - 3_949_559.44).abs() < CENT,
        "the 2% cap should take {taken:.2} of the cluster's total state support"
    );
    let absorbed = rolling_anchor::absorbed(&enacted_cluster, &capped_cluster);
    assert!(
        (absorbed - 0.960).abs() < 0.002,
        "which is 96.0% of a $99.4m cut absorbed; measured {absorbed:.4}"
    );
    let statewide = rolling_anchor::absorbed(&enacted_all, &capped_all);
    assert!(
        (statewide - 0.763).abs() < 0.002,
        "and 76.3% statewide; measured {statewide:.4}"
    );
    assert!(
        capped_cluster.transition_supplement > enacted_cluster.transition_supplement,
        "the money arrives on [K] instead: {:.2} against {:.2}",
        capped_cluster.transition_supplement,
        enacted_cluster.transition_supplement
    );
}

/// No anchor shape moves the quantity the do-nothing case was actually about.
///
/// #400's honest baseline is not a shortfall but a provenance: a quarter of what this cluster
/// receives ends up riding on a hold-harmless, one of which is uncodified and renewed a biennium
/// at a time. Every rule priced here leaves that share within a seventh of a point of where it
/// found it, because what comes off `[I]` goes on `[K]`.
#[test]
fn every_anchor_leaves_the_same_share_of_the_clusters_money_on_a_hold_harmless() {
    let districts = panel::panel();
    let (_, enacted) = walk(&districts, 2036, undamped(), Anchor::Fixed);
    assert!(
        (enacted.held_share() - 0.2694).abs() < 0.0001,
        "the enacted anchor holds 26.94%, holds {:.4}",
        enacted.held_share()
    );

    for factor in [0.99_f64, 0.98, 0.97, 0.95] {
        let (_, capped) = walk(&districts, 2036, undamped(), Anchor::Decayed { factor });
        assert!(
            capped.guarantee < enacted.guarantee * 0.7,
            "at {factor} the guarantee should be well down: {:.2}",
            capped.guarantee
        );
        assert!(
            (capped.held_share() - enacted.held_share()).abs() < 0.0015,
            "and the held share should not move: {:.4} at {factor} against {:.4} enacted",
            capped.held_share(),
            enacted.held_share()
        );
    }
}

/// No lever, and every rule says what it rewards at the margin.
///
/// The two constraints #400 answered for its four shapes, asked again of these three.
#[test]
fn no_anchor_rule_needs_a_policy_field_and_each_says_what_it_rewards() {
    for anchor in [
        Anchor::Fixed,
        Anchor::PriorYear,
        Anchor::Decayed { factor: 0.98 },
    ] {
        assert!(
            !anchor.needs_a_lever(),
            "an anchor rule is a per-district function of a per-district history"
        );
        assert!(
            anchor.rewards_at_the_margin().len() > 40,
            "and says what it pays for one more pupil lost"
        );
    }
    assert_ne!(
        Anchor::Fixed.rewards_at_the_margin(),
        Anchor::Decayed { factor: 0.98 }.rewards_at_the_margin(),
        "the decay is the only one that charges for losing pupils faster than the cap"
    );
}
