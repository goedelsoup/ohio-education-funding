//! What the three guarantee policies save, under each of the two things §265.225 could do.
//!
//! `scenario/guarantee-phase-out` published its savings table against a model in which `[K]`, the
//! formula transition supplement of uncodified Section 265.225, did not exist. Those figures were
//! unbound prose, so nothing in this repository could see them go stale. This file is the pin.
//!
//! # The two answers, and why the node reports both
//!
//! `[I]` is a term in `[K]`'s own subtrahend, so the supplement **backstops the guarantee**. That
//! makes "retire the guarantee" two different bills:
//!
//! | | `[K]` as enacted | `[K]` repealed alongside |
//! |---|--:|--:|
//! | removal | −$79.8m | −$942.5m |
//! | half phase-out | −$70.8m | −$503.1m |
//! | rebase to 90% | −$66.2m | −$311.8m |
//!
//! Neither column is the finding. The left one is current law and says a guarantee retirement is
//! **nearly free and nearly policy-invariant** — three policies whose gross effect on foundation
//! aid spans $248.2m to $879.0m come to within $13.6m of each other once the backstop has
//! answered, which is a 3.5-fold range collapsing to a 1.2-fold one. The right one is what
//! "retiring the guarantee" means as a policy, and it is where the range survives. A node
//! reporting one of them re-creates the defect: the gap between the columns *is* the finding.
//!
//! # What the manifest carries and what this file carries
//!
//! `crates/figures.json` cannot hold a negative — `corpusFigures.ts`'s numeral regex captures no
//! sign — so it pins these as magnitudes with the direction in the key, the convention
//! `fund-the-plan-run-cut` set. The signs are pinned here, which is the half a magnitude cannot
//! state and the half #120 turned out to be about.

use project::hold_harmless::retirement;
use project::panel::panel;
use project::policy::{Backstop, GuaranteeRule, Policy};
use project::report::simulate;

/// The three rules the scenario node states, and the two backstop answers it prices them under.
fn rules() -> [(&'static str, GuaranteeRule); 3] {
    [
        ("removal", GuaranteeRule::Removed),
        (
            "half phase-out",
            GuaranteeRule::PhasedOut { remaining: 0.5 },
        ),
        ("rebase to 90%", GuaranteeRule::Rebased { factor: 0.9 }),
    ]
}

fn cost(guarantee: GuaranteeRule, backstop: Backstop) -> f64 {
    simulate(
        &panel(),
        &Policy {
            guarantee,
            backstop,
            ..Policy::current_law()
        },
    )
    .cost()
}

/// Every one of the six figures is a **saving**, and the manifest states none of their signs.
#[test]
fn every_guarantee_policy_costs_the_state_less_under_both_answers() {
    for (label, rule) in rules() {
        for backstop in [Backstop::AsEnacted, Backstop::Repealed] {
            let measured = cost(rule, backstop);
            assert!(
                measured < 0.0,
                "{label} under {backstop:?} priced {measured:.2}, which is not a saving"
            );
        }
    }
}

/// The six figures the node quotes, to the cent.
#[test]
fn the_six_savings_are_what_the_node_states() {
    let expected = [
        (GuaranteeRule::Removed, Backstop::AsEnacted, -79_813_629.47),
        (GuaranteeRule::Removed, Backstop::Repealed, -942_533_256.86),
        (
            GuaranteeRule::PhasedOut { remaining: 0.5 },
            Backstop::AsEnacted,
            -70_818_745.25,
        ),
        (
            GuaranteeRule::PhasedOut { remaining: 0.5 },
            Backstop::Repealed,
            -503_055_943.17,
        ),
        (
            GuaranteeRule::Rebased { factor: 0.9 },
            Backstop::AsEnacted,
            -66_211_485.88,
        ),
        (
            GuaranteeRule::Rebased { factor: 0.9 },
            Backstop::Repealed,
            -311_801_221.99,
        ),
    ];
    for (rule, backstop, want) in expected {
        let got = cost(rule, backstop);
        assert!(
            (got - want).abs() < 0.01,
            "{rule:?} under {backstop:?}: {got:.2}, expected {want:.2}"
        );
    }
}

/// Leaving `[K]` standing makes the three policies nearly the same bill.
///
/// This is the finding that replaces *"the three policies are not variants of each other"* — which
/// was true of the gross effect and is false of the cost. It is stated as a ratio rather than a
/// difference so that it keeps meaning something if the panel is refreshed.
#[test]
fn the_backstop_collapses_three_policies_into_one_price() {
    let under = |backstop| {
        let mut costs: Vec<f64> = rules().map(|(_, rule)| -cost(rule, backstop)).to_vec();
        costs.sort_by(f64::total_cmp);
        costs[costs.len() - 1] / costs[0]
    };
    // Gross of the backstop the three span 3.54x; net of it, 1.21x.
    assert!(
        under(Backstop::Repealed) > 3.0,
        "{}",
        under(Backstop::Repealed)
    );
    assert!(
        under(Backstop::AsEnacted) < 1.3,
        "{}",
        under(Backstop::AsEnacted)
    );
}

/// A rebase to 90% does not save a tenth of the guarantee, and the node said both $87.9m and
/// $248.2m.
///
/// $87.9m was a tenth of $879.0m, derived by hand and never computed; $248.2m was the model's own
/// figure for the fall in guarantee paid, which is the right number for the wrong quantity. The
/// guarantee is a `max`, so cutting the floor a tenth pushes **64** districts off it altogether
/// and takes their whole top-up rather than a tenth of it — 28.2% of the guarantee, not 10%.
#[test]
fn a_tenth_off_the_floor_is_not_a_tenth_off_the_guarantee() {
    let effect = simulate(
        &panel(),
        &Policy {
            guarantee: GuaranteeRule::Rebased { factor: 0.9 },
            ..Policy::current_law()
        },
    );
    let fall = effect.baseline.guarantee - effect.policy.guarantee;
    assert!((fall - 248_222_592.51).abs() < 0.01, "{fall:.2}");
    // Far more than a tenth, and exactly the fall in realized aid.
    assert!(fall / effect.baseline.guarantee > 0.28);
    assert!((effect.foundation_cost() + fall).abs() < 0.01);
    assert_eq!(
        effect.baseline.on_guarantee - effect.policy.on_guarantee,
        64,
        "districts pushed off the floor by a tenth"
    );
}

/// Whom the policies reach, which is the node's headline structural claim.
///
/// It survives for a guarantee-only lever and is *sharper* than published: the unreachable
/// population is not 315 districts but **442**, because the backstop makes 127 of the guaranteed
/// districts whole too. Extend the bill to `[K]` and the reach is 311 — past the 294 the guarantee
/// pays, by the 17 districts that draw `[K]` and were never on the guarantee.
#[test]
fn a_guarantee_only_lever_reaches_167_districts_and_both_devices_reach_311() {
    let panel = panel();
    for (label, rule) in rules() {
        let alone = simulate(
            &panel,
            &Policy {
                guarantee: rule,
                ..Policy::current_law()
            },
        );
        assert_eq!(alone.losers(), 167, "{label}");
        assert_eq!(
            alone.gainers(),
            0,
            "{label}: a floor cannot pay anyone more"
        );
        assert_eq!(alone.unmoved(), 442, "{label}");

        let both = simulate(
            &panel,
            &Policy {
                guarantee: rule,
                backstop: Backstop::Repealed,
                ..Policy::current_law()
            },
        );
        assert_eq!(both.losers(), 311, "{label}");
        assert_eq!(both.unmoved(), 298, "{label}");
        assert!(
            both.losers() > alone.baseline.on_guarantee,
            "{label}: reaches past the guaranteed population"
        );
    }
}

/// The incidence inverts: the cut lands on the wealthier guaranteed districts.
///
/// The node's published example was East Cleveland City at −$6,489 per pupil under a half
/// phase-out, presented as the largest per-pupil loser and as one of Ohio's poorest districts. On
/// total state support it loses **nothing** — `[K]` holds it at `[L1]` to the cent — and the
/// largest per-pupil loser of a removal is West Geauga Local, at $604,523 of valuation per pupil
/// against the statewide median of $248,097.
#[test]
fn the_largest_per_pupil_loser_is_a_wealthy_district_once_the_backstop_is_in_the_model() {
    let measured = retirement(&panel(), GuaranteeRule::Removed);
    assert_eq!(measured.largest_per_pupil_loss.0, "West Geauga Local");
    assert!(
        (measured.largest_per_pupil_loss.1 + 797.0).abs() < 1.0,
        "{:.2}",
        measured.largest_per_pupil_loss.1
    );
    assert!(
        (measured.median_loser_valuation - 318_480.0).abs() < 1.0,
        "{:.2}",
        measured.median_loser_valuation
    );
    assert!(
        (measured.median_made_whole_valuation - 257_278.0).abs() < 1.0,
        "{:.2}",
        measured.median_made_whole_valuation
    );
    // $799.1m absorbed, and most of it on districts that still lose.
    assert!(
        (measured.absorbed - 799_140_997.91).abs() < 0.01,
        "{:.2}",
        measured.absorbed
    );
    assert!(
        (measured.absorbed_making_whole - 308_936_650.36).abs() < 0.01,
        "{:.2}",
        measured.absorbed_making_whole
    );
}

/// The base cost table, on both measures, and the direction `[K]` moves it.
///
/// Raising base cost raises realized aid, which **reduces** `[K]` — so the wide measure is cheaper
/// than the narrow one here, the opposite of a guarantee cut. And the clawback is proportionally
/// largest on the smallest increase (4.7% of +2%, 3.6% of +20%), which steepens the marginal
/// gradient rather than flattening it. The node's fiscal-note warning therefore survives on the
/// measure a cost is quoted in, and is marginally stronger there than on foundation aid.
#[test]
fn a_base_cost_increase_is_clawed_back_hardest_at_its_smallest() {
    let panel = panel();
    let at = |scale: f64| {
        simulate(
            &panel,
            &Policy {
                base_cost_scale: scale,
                ..Policy::current_law()
            },
        )
    };
    let runs = [
        (1.02, 101_670_377.70, 106_645_239.28, 269),
        (1.05, 267_111_423.43, 288_057_862.14, 243),
        (1.10, 597_262_120.94, 639_701_359.86, 204),
        (1.20, 1_391_843_058.40, 1_443_268_152.36, 153),
    ];
    for (scale, total, foundation, on_guarantee) in runs {
        let effect = at(scale);
        assert!(
            (effect.cost() - total).abs() < 0.01,
            "{scale}: {:.2}",
            effect.cost()
        );
        assert!(
            (effect.foundation_cost() - foundation).abs() < 0.01,
            "{scale}: {:.2}",
            effect.foundation_cost()
        );
        assert_eq!(effect.policy.on_guarantee, on_guarantee, "{scale}");
        assert_eq!(effect.losers(), 0, "{scale}: raising base cost cuts nobody");
        // The clawback, which is a cost the state does not pay.
        assert!(effect.cost() < effect.foundation_cost(), "{scale}");
    }

    // Marginal cost per point of increase, which is the node's fourth column.
    let marginal = |from: f64, to: f64, points: f64| (at(to).cost() - at(from).cost()) / points;
    let first = at(1.02).cost() / 2.0;
    let last = marginal(1.10, 1.20, 10.0);
    assert!((first / 1e6 - 50.8).abs() < 0.1, "{:.2}", first / 1e6);
    assert!((last / 1e6 - 79.5).abs() < 0.1, "{:.2}", last / 1e6);
    assert!(
        marginal(1.02, 1.05, 3.0) > first && marginal(1.05, 1.10, 5.0) > marginal(1.02, 1.05, 3.0),
        "the gradient must be monotone for the fiscal-note warning to hold"
    );
    // Steeper on total state support than on foundation aid alone, because the clawback is
    // proportionally largest on the smallest increase.
    let foundation_first = at(1.02).foundation_cost() / 2.0;
    let foundation_last = (at(1.20).foundation_cost() - at(1.10).foundation_cost()) / 10.0;
    assert!(first / last < foundation_first / foundation_last);
}
