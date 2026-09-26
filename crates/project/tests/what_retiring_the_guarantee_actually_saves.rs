//! What the three guarantee policies save, under each of the three things §265.225 could do.
//!
//! `scenario/guarantee-phase-out` published its savings table against a model in which `[K]`, the
//! formula transition supplement of uncodified Section 265.225, did not exist. Those figures were
//! unbound prose, so nothing in this repository could see them go stale. This file is the pin.
//!
//! # The three answers, and why the node reports all of them
//!
//! `[I]` is a term in `[K]`'s own subtrahend, so the supplement **backstops the guarantee**. That
//! makes "retire the guarantee" three different bills:
//!
//! | | `[K]` as enacted | `[K]` repealed alongside | `[K]` rebased |
//! |---|--:|--:|--:|
//! | removal | −$79.8m | −$942.5m | −$253.6m |
//! | half phase-out | −$70.8m | −$503.1m | −$210.4m |
//! | guarantee rebased to 90% | −$66.2m | −$311.8m | −$148.5m |
//!
//! No column is the finding. The first is current law and says a guarantee retirement is
//! **nearly free and nearly policy-invariant** — three policies whose gross effect on foundation
//! aid spans $248.2m to $879.0m come to within $13.6m of each other once the backstop has
//! answered, which is a 3.5-fold range collapsing to a 1.2-fold one. The second is what
//! "retiring the guarantee" means as a policy, and it is where the range survives. A node
//! reporting one of them re-creates the defect: the gap between the columns *is* the finding.
//!
//! # The third column, which the node used to carry as `[open]`
//!
//! Repeal is not the only way a legislature that retires `[I]` can decline to pay it back through
//! `[K]`. §265.225 could stand and be recomputed against a base that never contained a guarantee:
//! the supplement still holds a district at its FY2021 total, but at an FY2021 total with the
//! previous formula's guarantee taken out of it. That is `Backstop::Rebased`, and it is the
//! reading the node refused, because `[L1]` is published as a single column and the department
//! never breaks out the guarantee inside it. The split is joined out of two other published files
//! by `project::transition_base`, which asserts the identity that licenses the join.
//!
//! It is the middle answer on every one of the three policies, and it is not a blend: the range
//! across the three policies is 1.71x, not 1.21x and not 3.54x, so a bill's choice of guarantee
//! rule still matters under it. Two of its properties are worth stating separately, because both
//! are easy to assume away — rebasing cuts `[K]` even with `[I]` left completely alone
//! (`−$21.3m`), and it is nowhere near additive with the guarantee bill it accompanies.
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

/// The three things §265.225 could do while `[I]` is being retired.
fn backstops() -> [Backstop; 3] {
    [Backstop::AsEnacted, Backstop::Repealed, Backstop::Rebased]
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

/// Every one of the nine figures is a **saving**, and the manifest states none of their signs.
#[test]
fn every_guarantee_policy_costs_the_state_less_under_all_three_answers() {
    for (label, rule) in rules() {
        for backstop in backstops() {
            let measured = cost(rule, backstop);
            assert!(
                measured < 0.0,
                "{label} under {backstop:?} priced {measured:.2}, which is not a saving"
            );
        }
    }
}

/// The nine figures the node quotes, to the cent.
#[test]
fn the_nine_savings_are_what_the_node_states() {
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
        (GuaranteeRule::Removed, Backstop::Rebased, -253_566_561.87),
        (
            GuaranteeRule::PhasedOut { remaining: 0.5 },
            Backstop::Rebased,
            -210_354_185.71,
        ),
        (
            GuaranteeRule::Rebased { factor: 0.9 },
            Backstop::Rebased,
            -148_465_354.04,
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
    // Rebasing `[K]` recovers part of the range and not most of it: 1.71x. It is the answer under
    // which a bill's choice of guarantee rule is neither decisive nor irrelevant.
    let rebased = under(Backstop::Rebased);
    assert!((1.5..2.0).contains(&rebased), "{rebased}");
    assert!(
        rebased > under(Backstop::AsEnacted) && rebased < under(Backstop::Repealed),
        "{rebased}"
    );
}

/// Rebasing `[K]` is a bill in its own right, and it is not additive with the guarantee bill.
///
/// Two readings of the third column that the figure invites and the model refutes. The first is
/// that it prices a guarantee retirement: it does not. Recomputing `[L1]` without the guarantee
/// cuts `[K]` by **$21.3m** with `[I]` left entirely alone, and takes the supplement's population
/// from 144 districts to **63** — because a district drawing `[K]` today is being held at a
/// FY2021 total that contains a guarantee it is *also* still being paid, and 81 of the 144 are
/// held there by nothing else. Rebasing is a live §265.225 amendment whether or not `[I]` moves.
///
/// The second is that the column is the sum of its parts: removal under a rebased `[K]` saves
/// $253.6m, and $79.8m + $21.3m is $101.1m. The interaction is the larger half of the figure,
/// because the same dollars cannot be taken out of the base and out of realized aid twice.
#[test]
fn rebasing_the_supplement_cuts_it_before_any_guarantee_bill_and_does_not_add() {
    let panel = panel();
    let alone = simulate(
        &panel,
        &Policy {
            backstop: Backstop::Rebased,
            ..Policy::current_law()
        },
    );
    assert!(
        (alone.cost() + 21_274_914.30).abs() < 0.01,
        "{:.2}",
        alone.cost()
    );
    assert!(
        (alone.policy.transition_supplement - 42_303_715.18).abs() < 0.01,
        "{:.2}",
        alone.policy.transition_supplement
    );
    assert!(
        (alone.baseline.transition_supplement - 63_578_629.48).abs() < 0.01,
        "{:.2}",
        alone.baseline.transition_supplement
    );
    // The whole of it is `[K]`: no district's realized aid or transportation moves.
    assert!(
        alone.foundation_cost().abs() < 0.01,
        "{:.2}",
        alone.foundation_cost()
    );
    assert!(
        alone.transportation_cost().abs() < 0.01,
        "{:.2}",
        alone.transportation_cost()
    );
    // And it is not a trim: it extinguishes the supplement for 81 of the 144 districts drawing it.
    let baseline = simulate(&panel, &Policy::current_law());
    let drawing = |effect: &project::report::PolicyEffect| {
        effect
            .outcomes
            .iter()
            .filter(|outcome| outcome.transition_supplement > 0.005)
            .count()
    };
    assert_eq!(drawing(&baseline), 144);
    assert_eq!(drawing(&alone), 63);

    // How many districts the supplement pays once the guarantee is removed, under each answer.
    // Two of the three are settled by construction — repeal pays nobody, and as enacted the
    // supplement picks up every one of the 294 districts the guarantee was holding. Only the
    // rebased count had to be computed, and it is 31 short of that.
    for (backstop, want) in [
        (Backstop::AsEnacted, 294),
        (Backstop::Rebased, 263),
        (Backstop::Repealed, 0),
    ] {
        let effect = simulate(
            &panel,
            &Policy {
                guarantee: GuaranteeRule::Removed,
                backstop,
                ..Policy::current_law()
            },
        );
        assert_eq!(drawing(&effect), want, "{backstop:?}");
    }

    let both = cost(GuaranteeRule::Removed, Backstop::Rebased);
    let parts = cost(GuaranteeRule::Removed, Backstop::AsEnacted) + alone.cost();
    assert!(both < parts - 100_000_000.0, "{both:.2} against {parts:.2}");
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
fn the_three_answers_reach_167_261_and_311_districts() {
    let panel = panel();
    // Whether a district is on the guarantee is a fact about current law, so it is read off a
    // current-law run — a policy run's own flag is false wherever the policy took the floor away.
    let law = simulate(&panel, &Policy::current_law());
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

        // Rebasing `[K]` rather than repealing it reaches 261, which is *short* of the 294 the
        // guarantee pays: 44 guaranteed districts are still held whole, by a supplement measured
        // against a base that no longer contains a guarantee. It reaches past the guaranteed
        // population all the same, by 11 districts that were never on the guarantee and lose the
        // `[K]` the old base was paying them — the same mechanism as repeal's 17, two thirds as
        // wide.
        let rebased = simulate(
            &panel,
            &Policy {
                guarantee: rule,
                backstop: Backstop::Rebased,
                ..Policy::current_law()
            },
        );
        assert_eq!(rebased.losers(), 261, "{label}");
        assert_eq!(rebased.unmoved(), 348, "{label}");
        assert_eq!(rebased.gainers(), 0, "{label}");
        assert!(
            rebased.losers() < alone.baseline.on_guarantee && rebased.losers() < both.losers(),
            "{label}"
        );
        let classed = |effect: &project::report::PolicyEffect, guaranteed: bool, cut: bool| {
            effect
                .outcomes
                .iter()
                .zip(law.outcomes.iter())
                .filter(|(outcome, base)| {
                    base.on_guarantee == guaranteed && (outcome.total_delta() < -0.005) == cut
                })
                .count()
        };
        assert_eq!(
            classed(&rebased, true, false),
            44,
            "{label}: still held whole"
        );
        assert_eq!(
            classed(&rebased, false, true),
            11,
            "{label}: cut without ever being on it"
        );
        assert_eq!(
            classed(&both, false, true),
            17,
            "{label}: repeal's wider version"
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
