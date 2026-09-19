//! Every device that holds an Ohio district above its formula amount in FY2027, and the three the
//! corpus has been dividing by one of.
//!
//! The corpus has treated the Fair School Funding Plan as having one hold-harmless: `[I]`, the
//! temporary transitional aid guarantee, $879.0m across 294 districts. Published shares are taken
//! against that. There are four devices.
//!
//! | Device | Line | Base | Anchor | Dollars | Districts | ADM | Placement | `policy::apply` |
//! |---|---|---|--:|--:|--:|--:|---|---|
//! | Temporary transitional aid guarantee | `[I]` | `[H2]` | FY2020 | $878,954,627.38 | 294 | 53.9% | inside foundation aid | recomputed |
//! | Formula transition supplement | `[K]` | `[L1]` | FY2021 | $63,578,629.47 | 144 | 23.5% | **outside foundation aid** | **omitted** |
//! | Transitional transportation aid | `[F]` | `[F1]` | FY2020 | $24,780,626.76 | 38 | 10.5% | inside `[J]` | passed through |
//! | Open-enrolment clawback | `[I1]` | — | prior year | −$5,110,049.66 | 43 | 14.4% | reduces `[I]` | passed through |
//!
//! Together the three that pay are **$967,313,883.61 to 326 districts, 58.0% of ADM** — against
//! the $879.0m and 294 every published share has used. The guarantee is 90.9% of the money and
//! 90.2% of the districts.
//!
//! # `[K]` is outside the measure, which is why nobody saw it
//!
//! `supplement_reach` settled the performance supplement by asking whether it sits inside `[H]
//! Foundation Funding`. That is the wrong boundary here, because `[I]` is outside `[H]` too: the
//! department's `[H]` is `[A]`–`[G]`, and both the guarantee and this supplement are separate
//! lines added at `[N] Total Formula Funding`. The boundary that matters is `Measure::FoundationAid`
//! — `[H] + [I]`, which is what `DistrictRecord::realized_aid` returns and what every published
//! share is computed on. `[K]` is outside *that*: invisible to the narrow measure, present in
//! `[R]`. The department's own line structure says so, and so does the `[R]` identity.
//!
//! # The devices are a ladder
//!
//! `[I]` tops `[H]` up to `[H2]`. `[K]` tops `[H] + [I] + [J]` up to `[L1]` — a total that already
//! contains the guarantee. For the 144 drawing it, `[H] + [I] + [J] + [K] = [L1]` to the cent, so
//! those districts are held at a **total** and a dollar taken off foundation funding is a dollar
//! `[K]` puts back. `[L]`, `[M]` and `[O]` sit outside the closure; the 144 are paid $13.3m of the
//! first two on top of a total that is otherwise frozen.
//!
//! # The finding: `[K]` is a backstop for the guarantee itself
//!
//! `[I]` is a term in `[K]`'s own subtrahend. So removing the guarantee does not remove the
//! hold-harmless — it moves it onto `[K]`, which rises to keep the district at `[L1]`.
//!
//! | Guarantee rule | reported cut | districts | after `[K]` | districts | absorbed |
//! |---|--:|--:|--:|--:|--:|
//! | `Removed` | $878,954,627.38 | 294 | $79,813,629.51 | 167 | **90.9%** |
//! | `PhasedOut { remaining: 0.5 }` | $439,477,313.69 | 294 | $70,818,745.29 | 167 | 83.9% |
//! | `Rebased { factor: 0.9 }` | $248,222,592.51 | 294 | $66,211,485.93 | 167 | 73.3% |
//!
//! **127 of the 294 are fully backstopped and only 17 are not backstopped at all.** The backstop
//! is mostly *latent*: Lakota Local draws no `[K]` today and would draw $20.2m of it without the
//! guarantee. A district is fully backstopped exactly when `[L1] >= [H2] + [J]` — when its FY2021
//! total base is at least its FY2020 foundation base plus what transportation pays now.
//!
//! This does not say a repeal would cost $79.8m. It says `GuaranteeRule::Removed` is
//! **underspecified**: repealing R.C. 3317.019(A)(1) and leaving H.B. 110 Section 265.225 standing
//! is what the model runs, and it is not a policy anyone would draft. A run that means to retire
//! the hold-harmless has to say what becomes of `[K]`, and until it does its saving is a number
//! between $79.8m and $879.0m with nothing choosing between them.
//!
//! # Three anchor years, not one
//!
//! Two of the three `FY21` headings in the workbook are stale. `the_year_the_column_header_names.rs`
//! established it for `[F1]` and `the_year_the_guarantee_holds_to.rs` for `[H2]`; both are the
//! FY2020 amount before Executive Order 2020-19D, under the two divisions of R.C. 3317.019. Only
//! `[L1]` is FY2021. `[H3]`, the DPIA slice that partitions `[H2]`, reaches back to FY2019.
//!
//! Two carriers of the retired reading survive that correction and are fixed alongside this file:
//! `the_supplements_outside_the_formula.rs` and `formula-component/fsfp-formula-transition-supplement`.
//!
//! # Which population, which measure
//!
//! Over the **609** districts the department's FY2027 model carries, not the 611 its payment
//! reports do. Dollars are the department's published columns. Absorption is measured with both
//! positions taken from the same `policy::apply` run, because recovering transportation through
//! its state share lands a cent away from the published total on fifteen districts and a finer
//! threshold turns that into phantom loss — see `hold_harmless::CENT`.

use project::hold_harmless::{self as hh, Anchor, Direction, Modelled, Placement, Position};
use project::panel::{panel, DistrictRecord};
use project::policy::{apply_all, GuaranteeRule, Outcome, Policy};

/// The department's own account of its own method, five editions of it.
const SFPR: &str = include_str!("../fixtures/dew-sfpr-line-by-line.txt");

/// LSC's analysis of H.B. 96 as enacted — the act that pays all four devices in FY2027.
const ENACTED: &str = include_str!("../fixtures/enacted-school-funding.txt");

/// A cent, which is what the published columns are stated to.
const CENT: f64 = 0.02;

/// What nine of those columns accumulate when they are added together.
///
/// The `[R]` identity sums nine independently rounded figures, so two cents of drift is the
/// arithmetic rather than a disagreement. Naming it keeps the looser bound from leaking into the
/// per-column assertions above, which hold at [`CENT`].
const SUMMED: f64 = 0.03;

/// The two positions `absorption` compares, from one run's four aid fields.
fn positions(outcome: &Outcome) -> (Position, Position) {
    (
        Position {
            realized_aid: outcome.realized_aid,
            transportation: outcome.transportation,
        },
        Position {
            realized_aid: outcome.baseline_realized_aid,
            transportation: outcome.baseline_transportation,
        },
    )
}

/// Run a policy and measure what `[K]` would absorb of it.
fn under(policy: Policy) -> hh::Absorption {
    let districts = panel();
    let outcomes = apply_all(&districts, &policy);
    hh::absorption(&districts, |record| {
        let outcome = outcomes
            .iter()
            .find(|outcome| outcome.irn == record.irn)
            .expect("every record produces an outcome");
        positions(outcome)
    })
}

/// The department adds the transition supplement to foundation funding rather than inside it.
///
/// `[N] Total Formula Funding` is defined in the FY2026 edition — the first whose lettering the
/// FY2027 model shares — as the guarantee, transportation, this supplement and the two flat
/// supplements "all added in the Foundation Funding (line 'H')". That sentence is the whole
/// inside-or-outside question, answered by the publisher.
#[test]
fn the_department_pays_the_supplement_outside_foundation_funding() {
    let flat: String = SFPR.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(
        flat.contains(
            "This amount is the sum of the funding for Temporary Transitional Aid Guarantee \
             (line \u{2018}I\u{2019}), Transportation Funding (line \u{2018}J\u{2019}), Formula \
             Transition Supplement (line \u{2018}K\u{2019}), Base Funding Supplement (line \
             \u{2018}L\u{2019}), Enrollment Growth Supplement (line \u{2018}M\u{2019}), all added \
             in the Foundation Funding (line \u{2018}H\u{2019})."
        ),
        "the FY2026 edition's [N] definition is not where this test expects it; the letters or \
         the wording moved"
    );

    // And [H] itself is only the phased-in components, which is why [I] is outside it too.
    assert!(
        flat.contains(
            "Funding is represented on lines \u{2018}A\u{2019} through \u{2018}N\u{2019}. The Core \
             Foundation Funding on line \u{2018}H\u{2019} is the sum of lines \u{2018}A\u{2019} \
             through \u{2018}G\u{2019}."
        ),
        "[H] is no longer stated as the sum of [A] through [G]"
    );
}

/// And the panel's own arithmetic agrees: `[K]` is an addend of `[R]`, not a part of `[H]`.
///
/// `[R] = [H] + [I] + [J] + [K] + [L] + [M] + [O] + [P] + [Q]` holds for all 609 districts. Drop
/// `[K]` and the identity fails by up to $12.4m, which is the check the department's prose cannot
/// give on its own.
#[test]
fn the_total_state_support_identity_needs_the_transition_supplement() {
    let districts = panel();
    let mut worst: f64 = 0.0;
    let mut worst_without: f64 = 0.0;

    for record in &districts {
        let without = record.core_foundation_funding
            + record.guarantee
            + record.transportation.total
            + record.supplements.base_funding
            + record.supplements.growth
            + record.performance.amount
            + record.preschool_special_education.total
            + record.transportation.special_education;
        let with = without + record.transition.transition_supplement;

        worst = worst.max((with - record.total_state_support).abs());
        worst_without = worst_without.max((without - record.total_state_support).abs());
    }

    assert!(
        worst <= SUMMED,
        "[R] does not reconcile with [K] in the sum; worst residual ${worst:.2}"
    );
    assert!(
        worst_without > 12_000_000.0,
        "dropping [K] should break the identity by millions; it breaks it by ${worst_without:.2}"
    );
}

/// Every device in the inventory, measured.
#[test]
fn the_inventory_pays_what_the_panel_says() {
    let districts = panel();
    assert_eq!(districts.len(), 609, "the panel moved");

    let expected = [
        ("[I]", 878_954_627.38, 294, 0.539_452),
        ("[K]", 63_578_629.47, 144, 0.235_254),
        ("[F]", 24_780_626.76, 38, 0.105_022),
        ("[I1]", 5_110_049.66, 43, 0.143_958),
    ];

    let measured = hh::reach(&districts);
    assert_eq!(measured.len(), expected.len());

    for (reach, (line, dollars, count, adm)) in measured.iter().zip(expected) {
        assert_eq!(reach.line, line);
        assert!(
            (reach.dollars - dollars).abs() <= CENT,
            "{line}: ${:.2} against ${dollars:.2}",
            reach.dollars
        );
        assert_eq!(reach.districts, count, "{line}: district count");
        assert!(
            (reach.adm_share - adm).abs() < 5e-6,
            "{line}: ADM share {:.6} against {adm:.6}",
            reach.adm_share
        );
    }
}

/// The denominator every "share of the guarantee" should have been taken against.
///
/// $967.3m to 326 districts, against $879.0m to 294. The guarantee is 90.9% of the money — large
/// enough that the existing figures are not wild, and small enough that the difference changes
/// the second digit of every one of them.
#[test]
fn the_hold_harmless_denominator_is_wider_than_the_guarantee() {
    let districts = panel();
    let combined = hh::combined(&districts);

    assert!(
        (combined.dollars - 967_313_883.61).abs() <= CENT,
        "combined ${:.2}",
        combined.dollars
    );
    assert_eq!(combined.districts, 326);
    assert!(
        (combined.adm_share - 0.580_263).abs() < 5e-6,
        "combined ADM share {:.6}",
        combined.adm_share
    );

    let guarantee: f64 = districts.iter().map(|record| record.guarantee).sum();
    let share = guarantee / combined.dollars;
    assert!(
        (share - 0.9087).abs() < 5e-4,
        "the guarantee is {share:.4} of hold-harmless money"
    );

    // And the population is wider by more than the guarantee's own overlap with the other two.
    let guaranteed = districts.iter().filter(|r| r.guarantee > 0.0).count();
    assert_eq!(guaranteed, 294);
    assert_eq!(
        combined.districts - guaranteed,
        32,
        "districts held by something other than the guarantee alone"
    );
}

/// No two devices hold the same set, and none is a subset of another.
#[test]
fn the_three_paying_devices_are_not_nested() {
    let districts = panel();
    let count =
        |predicate: fn(&DistrictRecord) -> bool| districts.iter().filter(|r| predicate(r)).count();

    assert_eq!(
        count(|r| r.guarantee > 0.0 && r.transition.transition_supplement > 0.0),
        127,
        "on both the guarantee and the transition supplement"
    );
    assert_eq!(
        count(|r| r.transition.transition_supplement > 0.0 && r.guarantee <= 0.0),
        17,
        "on the transition supplement and not the guarantee"
    );
    assert_eq!(
        count(|r| r.transportation.guarantee > 0.0 && r.guarantee <= 0.0),
        16,
        "on transportation's guarantee and not the formula's"
    );
    assert_eq!(
        count(|r| {
            r.transportation.guarantee > 0.0
                && r.guarantee <= 0.0
                && r.transition.transition_supplement <= 0.0
        }),
        15,
        "districts only transportation's guarantee reaches — the ones both other devices miss"
    );

    let off_the_guarantee: f64 = districts
        .iter()
        .filter(|r| r.guarantee <= 0.0)
        .map(|r| r.transition.transition_supplement)
        .sum();
    assert!(
        (off_the_guarantee - 4_066_776.92).abs() <= CENT,
        "${off_the_guarantee:.2} of [K] goes to districts the guarantee does not pay"
    );
}

/// `[K]` tops a district up to `[L1]` **in total**, so the ladder closes exactly.
#[test]
fn the_ladder_closes_on_the_fiscal_2021_base() {
    let districts = panel();
    let mut drawing = 0;
    let mut worst: f64 = 0.0;

    for record in districts
        .iter()
        .filter(|r| r.transition.transition_supplement > 0.0)
    {
        drawing += 1;
        let total = record.core_foundation_funding
            + record.guarantee
            + record.transportation.total
            + record.transition.transition_supplement;
        worst = worst.max((total - record.transition.fy21_funding_base).abs());
    }

    assert_eq!(drawing, 144);
    assert!(
        worst <= CENT,
        "[H] + [I] + [J] + [K] should equal [L1] for every district drawing it; worst ${worst:.4}"
    );

    // The three newest lines are outside that closure, so the 144 are paid on top of a frozen
    // total rather than inside it.
    let on_top: f64 = districts
        .iter()
        .filter(|r| r.transition.transition_supplement > 0.0)
        .map(|r| r.supplements.base_funding + r.supplements.growth)
        .sum();
    assert!(
        (on_top - 13_313_734.34).abs() <= CENT,
        "${on_top:.2} of [L] and [M] paid on top of the closure"
    );
}

/// `transition_supplement_under` reproduces the department's column at current law.
///
/// This is what licenses running it on an output the department never published. Without it the
/// absorption figures below would be a formula asserted rather than one checked.
#[test]
fn the_transition_supplement_reproduces_from_the_lines_it_subtracts() {
    let districts = panel();
    let mut worst: f64 = 0.0;
    let mut worst_name = String::new();

    for record in &districts {
        let computed = hh::transition_supplement_under(
            record,
            record.realized_aid(),
            record.transportation.total,
        );
        let residual = (computed - record.transition.transition_supplement).abs();
        if residual > worst {
            worst = residual;
            worst_name = record.name.clone();
        }
    }

    assert!(
        worst <= CENT,
        "[K] does not reproduce for all 609; worst ${worst:.4} at {worst_name}"
    );
}

/// `policy::apply` models three of the four devices and omits the one a lever can move.
#[test]
fn the_policy_model_omits_the_device_a_lever_moves() {
    let by_line: Vec<(&str, Modelled)> = hh::inventory()
        .into_iter()
        .map(|device| (device.line, device.modelled))
        .collect();

    assert_eq!(
        by_line,
        vec![
            ("[I]", Modelled::Recomputed),
            ("[K]", Modelled::Omitted),
            ("[F]", Modelled::PassedThrough),
            ("[I1]", Modelled::PassedThrough),
        ]
    );

    // The two pass-throughs are correct by design and the omission is not, and the difference is
    // whether a lever can move the quantity. [F] is a fixed historical amount added after the
    // state share; [I1] is a published charge against a prior year no policy here changes. [K] is
    // a function of the model's own output.
    let placement: Vec<Placement> = hh::inventory()
        .into_iter()
        .map(|device| device.placement)
        .collect();
    assert_eq!(
        placement,
        vec![
            Placement::FoundationAid,
            Placement::FormulaFunding,
            Placement::Transportation,
            Placement::FoundationAid,
        ],
        "[K] is the only device outside the measure every published share is computed on"
    );
}

/// A broad cut overstates both its saving and its reach, because `[K]` rises to meet it.
#[test]
fn a_broad_cut_overstates_its_saving_and_its_reach() {
    let absorption = under(Policy {
        base_cost_scale: 0.95,
        ..Policy::current_law()
    });

    assert!(
        (absorption.reported - 230_104_279.36).abs() < 1.0,
        "reported ${:.2}",
        absorption.reported
    );
    assert_eq!(absorption.reported_districts, 315);
    assert!(
        (absorption.actual - 218_882_112.68).abs() < 1.0,
        "actual ${:.2}",
        absorption.actual
    );
    assert_eq!(
        absorption.actual_districts, 299,
        "sixteen districts the run reports as losing lose nothing"
    );
    assert!(
        (absorption.share() - 0.0488).abs() < 5e-4,
        "[K] absorbs {:.4} of the cut",
        absorption.share()
    );
}

/// And the same in reverse: an increase is partly clawed back, because `[K]` falls.
#[test]
fn a_broad_increase_is_partly_clawed_back_by_the_same_device() {
    let districts = panel();
    let outcomes = apply_all(
        &districts,
        &Policy {
            base_cost_scale: 1.05,
            ..Policy::current_law()
        },
    );

    let mut reported = 0.0;
    let mut delivered = 0.0;
    for (record, outcome) in districts.iter().zip(&outcomes) {
        let gain = (outcome.realized_aid + outcome.transportation)
            - (outcome.baseline_realized_aid + outcome.baseline_transportation);
        if gain <= hh::CENT {
            continue;
        }
        let supplement =
            hh::transition_supplement_under(record, outcome.realized_aid, outcome.transportation);
        reported += gain;
        delivered += gain + (supplement - record.transition.transition_supplement);
    }

    assert!(
        (reported - 288_057_862.14).abs() < 1.0,
        "reported gain ${reported:.2}"
    );
    assert!(
        (delivered - 267_111_423.42).abs() < 1.0,
        "delivered ${delivered:.2}"
    );
    let offset = (reported - delivered) / reported;
    assert!(
        (offset - 0.0727).abs() < 5e-4,
        "[K] takes back {offset:.4} of an increase"
    );
}

/// The finding: `[K]` is a backstop for the guarantee itself, and it is mostly latent.
///
/// `[I]` is a term in `[K]`'s own subtrahend, so retiring the guarantee moves the money onto the
/// supplement rather than saving it. 127 of the 294 are fully backstopped; only 17 are not
/// backstopped at all; and most of the backstop belongs to districts drawing no `[K]` today.
#[test]
fn the_transition_supplement_backstops_the_guarantee_it_subtracts() {
    let districts = panel();

    let mut full = 0;
    let mut none = 0;
    let mut recovered = 0.0;
    let mut latent = 0;
    for record in districts.iter().filter(|r| r.guarantee > 0.0) {
        let without = hh::transition_supplement_under(
            record,
            record.realized_aid() - record.guarantee,
            record.transportation.total,
        );
        let gain = without - record.transition.transition_supplement;
        recovered += gain;
        // Classified by whether the backstop is whole, not by a ratio near one. Delphos City is
        // 99.937% backstopped -- short by $872.56 -- and a 0.999 threshold calls that full.
        if gain >= record.guarantee - CENT {
            full += 1;
        } else if gain <= CENT {
            none += 1;
        }
        if gain > hh::CENT && record.transition.transition_supplement <= 0.0 {
            latent += 1;
        }
    }

    assert_eq!(full, 127, "guaranteed districts [K] fully backstops");
    assert_eq!(none, 17, "guaranteed districts [K] does not reach at all");
    assert_eq!(
        294 - full - none,
        150,
        "and the rest are backstopped in part"
    );
    assert!(
        latent > 100,
        "only {latent} of the backstopped districts draw no [K] today; the backstop is supposed \
         to be mostly latent, which is why no published figure shows it"
    );

    let guarantee: f64 = districts.iter().map(|r| r.guarantee).sum();
    assert!(
        (recovered / guarantee - 0.9092).abs() < 5e-4,
        "[K] recovers {:.4} of the guarantee",
        recovered / guarantee
    );

    // A district is fully backstopped exactly when its FY2021 total base covers its FY2020
    // foundation base plus what transportation pays now.
    for record in districts.iter().filter(|r| r.guarantee > 0.0) {
        let covered = record.transition.fy21_funding_base
            >= record.guarantee_floor() + record.transportation.total - CENT;
        let without = hh::transition_supplement_under(
            record,
            record.realized_aid() - record.guarantee,
            record.transportation.total,
        );
        let fully = without - record.transition.transition_supplement >= record.guarantee - CENT;
        assert_eq!(
            covered, fully,
            "{}: [L1] >= [H2] + [J] should be exactly the full-backstop condition",
            record.name
        );
    }
}

/// And the same for every rule that retires the guarantee part-way.
#[test]
fn every_guarantee_rule_is_absorbed_by_the_device_the_model_omits() {
    for (rule, reported, actual, absorbed) in [
        (
            GuaranteeRule::Removed,
            878_954_627.38,
            79_813_629.51,
            0.9092,
        ),
        (
            GuaranteeRule::PhasedOut { remaining: 0.5 },
            439_477_313.69,
            70_818_745.29,
            0.8389,
        ),
        (
            GuaranteeRule::Rebased { factor: 0.9 },
            248_222_592.51,
            66_211_485.93,
            0.7333,
        ),
    ] {
        let absorption = under(Policy {
            guarantee: rule,
            ..Policy::current_law()
        });
        assert!(
            (absorption.reported - reported).abs() < 1.0,
            "{rule:?}: reported ${:.2} against ${reported:.2}",
            absorption.reported
        );
        assert!(
            (absorption.actual - actual).abs() < 1.0,
            "{rule:?}: actual ${:.2} against ${actual:.2}",
            absorption.actual
        );
        assert!(
            (absorption.share() - absorbed).abs() < 5e-4,
            "{rule:?}: absorbed {:.4} against {absorbed:.4}",
            absorption.share()
        );
        assert_eq!(absorption.reported_districts, 294);
        assert_eq!(
            absorption.actual_districts, 167,
            "{rule:?}: the reach is 167 districts and not 294"
        );
    }
}

/// The published minimum-state-share figures, restated with `[K]` in them.
///
/// `the_minimum_share_the_guarantee_has_already_paid_past.rs` reports $35.1m at a 5% minimum and
/// $54.4m at 0%. Both overstate, by 0.6% and 0.5%. The district count does **not** move: the one
/// district that drops out at a finer threshold drops out by less than a cent, which is the
/// transportation round-trip artefact and not a district.
#[test]
fn the_minimum_share_cut_is_overstated_by_half_a_per_cent() {
    for (minimum, reported, actual) in [
        (0.05, 35_099_477.35, 34_882_774.66),
        (0.0, 54_419_493.03, 54_140_141.78),
    ] {
        let absorption = under(Policy {
            minimum_state_share: minimum,
            ..Policy::current_law()
        });
        assert!(
            (absorption.reported - reported).abs() < 1.0,
            "minimum {minimum}: reported ${:.2}",
            absorption.reported
        );
        assert!(
            (absorption.actual - actual).abs() < 1.0,
            "minimum {minimum}: actual ${:.2}",
            absorption.actual
        );
        assert_eq!(absorption.reported_districts, 31);
        assert_eq!(
            absorption.actual_districts, 31,
            "the reach of the minimum share is 31 districts either way"
        );
    }
}

/// The 46.7% figure is right as written, and wrong the moment it is read as a share of the money.
///
/// The 107 districts at the minimum state share hold 46.72% of *the guarantee* — that is what
/// `the_minimum_share_the_guarantee_has_already_paid_past.rs` says and it is exact. They hold
/// **42.90%** of hold-harmless money, because they draw $4.3m of `[K]` and $64k of `[F]` as well.
///
/// Widening the denominator without widening the numerator gives 42.45% against all three devices
/// and 43.57% against the two that sit in the formula — two more numbers, neither of which answers
/// either question. The restatement this file licenses is the one that moves both halves.
#[test]
fn the_share_at_the_minimum_is_a_different_number_on_the_wider_denominator() {
    let districts = panel();
    let at_floor: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|record| record.at_minimum_state_share() && record.guarantee > 0.0)
        .collect();
    assert_eq!(at_floor.len(), 107);

    let guarantee: f64 = at_floor.iter().map(|record| record.guarantee).sum();
    assert!(
        (guarantee - 410_617_498.51).abs() < 1.0,
        "${guarantee:.2} of guarantee at the floor"
    );

    let combined = hh::combined(&districts);
    let statewide_guarantee: f64 = districts.iter().map(|record| record.guarantee).sum();
    let narrow = guarantee / statewide_guarantee;
    assert!(
        (narrow - 0.4672).abs() < 5e-4,
        "share of the guarantee {narrow:.4}"
    );

    let theirs: f64 = at_floor
        .iter()
        .map(|record| {
            record.guarantee
                + record.transition.transition_supplement
                + record.transportation.guarantee
        })
        .sum();
    let wide = theirs / combined.dollars;
    assert!(
        (wide - 0.4290).abs() < 5e-4,
        "share of all hold-harmless money {wide:.4}"
    );

    // The two numbers a restatement produces when only the denominator is widened. Neither is
    // wrong arithmetic; both answer a question nobody asked, which is the failure mode worth
    // pinning rather than describing.
    let mismatched = guarantee / combined.dollars;
    assert!(
        (mismatched - 0.4245).abs() < 5e-4,
        "guarantee-only numerator over all three devices: {mismatched:.4}"
    );

    let in_formula: f64 = districts
        .iter()
        .map(|record| record.guarantee + record.transition.transition_supplement)
        .sum();
    let narrower = guarantee / in_formula;
    assert!(
        (narrower - 0.4357).abs() < 5e-4,
        "guarantee-only numerator over the two devices inside the formula: {narrower:.4}"
    );
}

/// The 144 `[K]` already pays are insulated outright, which is the claim `insulated` makes.
///
/// Their cheque does not move under any cut to `[H]`, `[I]` or `[J]` — not approximately, exactly
/// — because `[K]` is whatever it takes to reach `[L1]`. This is the strongest hold-harmless in
/// the formula and it is the one the policy model does not carry.
#[test]
fn the_districts_already_drawing_the_supplement_do_not_move_at_all() {
    let districts = panel();
    assert_eq!(hh::insulated(&districts), 144);

    let outcomes = apply_all(
        &districts,
        &Policy {
            base_cost_scale: 0.90,
            ..Policy::current_law()
        },
    );

    let mut checked = 0;
    for (record, outcome) in districts.iter().zip(&outcomes) {
        if record.transition.transition_supplement <= 0.0 {
            continue;
        }
        checked += 1;
        let supplement =
            hh::transition_supplement_under(record, outcome.realized_aid, outcome.transportation);
        let before = record.realized_aid()
            + record.transportation.total
            + record.transition.transition_supplement;
        let after = outcome.realized_aid + outcome.transportation + supplement;
        assert!(
            (after - before).abs() <= CENT,
            "{}: a tenth off base cost moves this district by ${:.2}, and it should move it by \
             nothing",
            record.name,
            after - before
        );
    }
    assert_eq!(checked, 144);
}

/// The devices reach back to three years, and two of the three `FY21` headings are wrong.
#[test]
fn the_bases_reach_back_to_three_years_and_not_one() {
    let bases = hh::bases();
    let anchors: Vec<(&str, Anchor)> = bases
        .iter()
        .map(|base| (base.column, base.anchor))
        .collect();

    assert_eq!(
        anchors,
        vec![
            ("[H2]", Anchor::Fiscal2020),
            ("[H3]", Anchor::Fiscal2019),
            ("[L1]", Anchor::Fiscal2021),
            ("[F1]", Anchor::Fiscal2020),
        ]
    );

    // Only one of the two columns the workbook heads `FY21` is actually FY2021.
    let fy21_headed = bases
        .iter()
        .filter(|base| base.label.contains("FY21"))
        .count();
    let fy21_anchored = bases
        .iter()
        .filter(|base| base.anchor == Anchor::Fiscal2021)
        .count();
    assert_eq!((fy21_headed, fy21_anchored), (2, 1));

    // And the two that are not are the two divisions of one section.
    let by_3317_019 = bases
        .iter()
        .filter(|base| base.authority.starts_with("R.C. 3317.019"))
        .count();
    assert_eq!(
        by_3317_019, 1,
        "[F1] is the only base R.C. 3317.019 defines"
    );
}

/// `[H3]` partitions `[H2]` exactly, which is why it is a base and not a fifth device.
#[test]
fn the_disadvantaged_slice_partitions_the_funding_base_rather_than_adding_to_it() {
    let districts = panel();

    // The two slices sum to the base, which is the claim. Stating it as "the slice is no larger
    // than the base" instead fails on Richmond Heights Local, whose base is **negative**
    // (-$40,179.23, the FY2020 deductions exceeding the funding): its general slice is
    // -$193,759.65 and its DPIA slice a positive $153,580.42, and a partition permits that where
    // an inequality against a clamped base does not.
    let mut negative_base = 0;
    for record in &districts {
        let dpia = record.transition.funding_base_econ_dis;
        let general = record.transition.funding_base - dpia;
        assert!(
            (general + dpia - record.transition.funding_base).abs() <= CENT,
            "{}: the two slices do not sum to the base",
            record.name
        );
        if record.transition.funding_base < 0.0 {
            negative_base += 1;
            assert!(
                general < 0.0 && dpia > 0.0,
                "{}: the negative base should sit entirely in the general slice",
                record.name
            );
        }
    }
    assert_eq!(negative_base, 1, "exactly one district has a negative [H2]");

    // And no device measures against it: the phase-in dials it, which is a term in [H] and not a
    // line beside it.
    assert!(
        hh::inventory()
            .iter()
            .all(|device| device.base != Some("[H3]")),
        "no payable device measures against the DPIA slice"
    );
}

/// The two flat supplements are not hold-harmless devices, and saying why is part of the map.
///
/// `[L]` pays every district with no test of any kind — no historical amount, no shortfall. `[M]`
/// pays a district whose enrolment **rose**, which is the opposite comparison. Neither compares a
/// computed amount against a frozen one, which is what the four devices do.
#[test]
fn the_flat_supplements_are_need_adjustments_and_not_floors() {
    let districts = panel();

    let base_funding = districts
        .iter()
        .filter(|r| r.supplements.base_funding > 0.0)
        .count();
    assert_eq!(
        base_funding, 609,
        "[L] reaches every district, so it cannot be conditioned on a shortfall"
    );

    let growth: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|r| r.supplements.growth > 0.0)
        .collect();
    assert_eq!(growth.len(), 43);
    assert!(
        growth
            .iter()
            .all(|record| record.supplements.enrollment_change > 0.0),
        "every district drawing [M] grew; a hold-harmless pays the ones that shrank"
    );

    // The one place they meet: a district can draw [M] for growing and [I] for having shrunk
    // since FY2020, which is the two clocks the formula runs on rather than a contradiction.
    let both = districts
        .iter()
        .filter(|r| r.supplements.growth > 0.0 && r.guarantee > 0.0)
        .count();
    assert!(
        both > 0,
        "no district draws both, which would make the two comparisons the same comparison"
    );

    assert_eq!(
        hh::inventory().len(),
        4,
        "neither flat supplement is in the inventory"
    );
    assert!(
        hh::inventory()
            .iter()
            .filter(|device| device.direction == Direction::HoldsUp)
            .count()
            == 3,
        "three devices pay and one claws back"
    );
}

/// **The authority each device is paid under, which is not always the act that wrote it.**
///
/// Two corrections live here. The clawback cites division `(C)` and not `(B)` — `(B)` is the joint
/// vocational adjustment — and `[K]`'s section is uncodified law of H.B. 110 whose newest committed
/// text stops at FY2023, so naming only that section leaves the reader unable to see what pays it
/// in FY2027.
#[test]
fn the_act_that_keeps_each_device_alive_is_named_beside_the_one_that_wrote_it() {
    let section = project::statute::section("3317.019");
    let body = project::act::flat(section.body);

    // The section this repository holds is H.B. 96's, and it names its own years: the guarantee
    // is temporary in the Revised Code rather than only in practice.
    assert!(
        section.legislation.contains("House Bill 96"),
        "the extract's 3317.019 is {} rather than H.B. 96's",
        section.legislation
    );
    assert!(
        body.contains(
            "for fiscal years 2026 and 2027, the department of education and workforce \
shall pay temporary transitional aid"
        ),
        "R.C. 3317.019(A)(1) no longer names its own years"
    );

    // (B) is the joint vocational adjustment and (C) is the clawback, which is why the inventory
    // cites (C). Reading the divisions in order is what settles it.
    let b = body.find(
        "(B) If a local school district participates in the establishment of a joint \
vocational school district",
    );
    let c = body.find(
        r#"(C)(1) For purposes of division (C) of this section, a district's "decrease threshold""#,
    );
    assert!(
        b.is_some(),
        "R.C. 3317.019(B) is no longer the joint vocational adjustment"
    );
    assert!(
        c.is_some(),
        "R.C. 3317.019(C) is no longer the decrease threshold"
    );
    assert!(b < c, "the divisions are out of order");

    let clawback = hh::inventory()
        .iter()
        .find(|device| device.direction == Direction::ClawsBack)
        .copied()
        .expect("the inventory has a clawback");
    assert_eq!(clawback.authority, "R.C. 3317.019(C)");

    // And the threshold it is measured against counts open-enrolment students received — R.C.
    // 3317.03(A)(1)(b) — which is what makes the department's column name right.
    let counted = project::act::flat(project::statute::section("3317.03").body);
    assert!(
        counted.contains(
            "(b) Adjacent or other district students enrolled in the district under \
an open enrollment policy"
        ),
        "3317.03(A)(1)(b) is no longer the open-enrolment count the clawback is keyed on"
    );
}

/// **One clause of H.B. 96 extends both devices the backstop links.**
///
/// The legislative fact behind `absorption`: `[I]` and `[K]` are not independently maintained
/// instruments that happen to interact. The same act has carried them forward together.
#[test]
fn one_clause_extends_the_guarantee_and_its_backstop_together() {
    let enacted = project::act::flat(ENACTED);
    assert!(
        enacted.contains(
            "extends to FY 2026 and FY 2027 the payment of temporary transitional \
aid to school districts based on an FY 2020 funding base and a formula transition supplement \
based on an FY 2021 funding base"
        ),
        "H.B. 96 no longer extends both devices in one clause"
    );

    // Both authorities say so, so a reader of the table cannot take H.B. 110 for the payer.
    let supplement = hh::inventory()
        .iter()
        .find(|device| device.line == "[K]")
        .copied()
        .expect("the inventory has the supplement");
    assert!(
        supplement.authority.contains("265.225") && supplement.authority.contains("H.B. 96"),
        "the supplement's authority is {}, which names no act that pays it in FY2027",
        supplement.authority
    );
}

/// **`[K]` reaches further than this fixture does, so its total is a share and not a programme.**
#[test]
fn the_supplement_is_paid_to_schools_this_panel_does_not_carry() {
    let enacted = project::act::flat(ENACTED);
    assert!(
        enacted.contains(
            "based on an FY 2021 funding base to districts, community schools, and \
STEM schools"
        ),
        "H.B. 96 no longer pays the supplement beyond districts"
    );

    // The panel is districts only, so every figure in this file is the district share. Asserting
    // the population here keeps the caveat attached to the number rather than to the prose.
    assert_eq!(panel().len(), 609);
}
