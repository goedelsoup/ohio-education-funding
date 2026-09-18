//! Who a cost-input refresh would actually pay, against who the argument says it would.
//!
//! # The hypothesis
//!
//! That the budget in force, measured against the plan Cupp and Patterson wrote, falls hardest on
//! districts with **large industrial tax bases and declining enrollment**. The phase-in cannot be
//! the mechanism — it ran to 100% on the original schedule — so the question is the input freeze,
//! and `project::refresh` prices it.
//!
//! # Half of it is right, and the other half runs backwards
//!
//! Refreshing the classroom teacher salary from FY2022 to FY2024 raises the computed requirement
//! by **$497.2m** — $465.0m of base cost and $32.2m of categorical repricing — and statewide aid
//! by **$220.3m**. The gap is the finding before any district is classified: **under half of a
//! cost increase becomes state aid**, because 254 of 609 districts are paid by the guarantee and a
//! floor does not move when the thing beneath it does.
//!
//! | | districts | reached | captured | per pupil |
//! |---|--:|--:|--:|--:|
//! | industrial > 10% | 27 | 63.0% | 52.9% | **$186.92** |
//! | mineral > 10% | 10 | 10.0% | **5.0%** | **$17.17** |
//! | public utility > 10% | 196 | 58.7% | 58.8% | $213.43 |
//! | *statewide* | 609 | 58.3% | 44.3% | $157.12 |
//!
//! **Industrial districts do better than average, not worse.** They capture 52.9% of their own
//! computed increase against a statewide 44.3%, and collect $186.92 per pupil against $157.12.
//! Whatever the freeze costs Ohio, it does not fall on them outstandingly.
//!
//! **Mineral districts are the population the claim describes.** Nine of the ten are held by the
//! guarantee throughout; they capture a twentieth of their increase and collect $17.17 a pupil.
//! `dispersion::tax_base` is where that substitution is established — the three business classes
//! are near-disjoint sets of districts and only this one behaves as argued.
//!
//! # Declining enrollment, though, is exactly right
//!
//! By two-year enrollment change, in quartiles, the gradient is monotone and steep:
//!
//! | | captured | per pupil | held by the guarantee |
//! |---|--:|--:|--:|
//! | Q1 fastest decline | 31.4% | $115.76 | 74 of 152 |
//! | Q2 | 42.7% | $151.26 | 73 |
//! | Q3 | 46.7% | $164.12 | 65 |
//! | Q4 growing | **51.7%** | **$180.13** | 42 of 153 |
//!
//! A district losing pupils collects under a third of the increase its own costing implies; a
//! growing one collects over half. So the hypothesis is right about enrollment and wrong about
//! industry, and the mechanism behind the half that is right is **not the tax base and not the
//! enrollment either** — it is the guarantee, which correlates with both and causes the result.
//! `funding-regime/fair-school-funding-plan` already records that property wealth rather than
//! enrollment decline is what puts a district on the guarantee.

use std::collections::BTreeMap;

use dispersion::tax_base::{self, Base, TaxBase};
use project::panel::{panel, DistrictRecord};
use project::refresh::{self, Refreshed, Total};

/// The refresh, computed once for every test below.
fn rows() -> Vec<Refreshed> {
    refresh::across_panel()
}

/// Business-property composition, keyed for the join.
fn composition() -> BTreeMap<String, TaxBase> {
    tax_base::by_irn()
}

/// The panel's districts split into quartiles by two-year enrollment change, worst first.
fn enrollment_quartiles() -> Vec<Vec<String>> {
    let mut ordered: Vec<(f64, String)> = panel()
        .iter()
        .map(|d: &DistrictRecord| {
            let (first, last) = (d.adm_history[0], d.adm_history[2]);
            let change = if first > 0.0 { last / first - 1.0 } else { 0.0 };
            (change, d.irn.clone())
        })
        .collect();
    ordered.sort_by(|a, b| a.0.total_cmp(&b.0));

    let quarter = ordered.len() / 4;
    (0..4)
        .map(|i| {
            let end = if i == 3 {
                ordered.len()
            } else {
                (i + 1) * quarter
            };
            ordered[i * quarter..end]
                .iter()
                .map(|(_, irn)| irn.clone())
                .collect()
        })
        .collect()
}

fn total_over(rows: &[Refreshed], irns: &[String]) -> Total {
    let set: std::collections::BTreeSet<&str> = irns.iter().map(String::as_str).collect();
    refresh::total(rows, |row| set.contains(row.irn.as_str()))
}

fn total_concentrated(rows: &[Refreshed], base: Base) -> Total {
    let composition = composition();
    refresh::total(rows, |row| {
        composition
            .get(&row.irn)
            .is_some_and(|c| c.concentrated(base))
    })
}

/// **The finding before any district is classified.** Under half of the increase becomes aid.
#[test]
fn a_base_cost_increase_is_not_an_aid_increase() {
    let rows = rows();
    let all = refresh::statewide(&rows);

    assert_eq!(all.reach.districts, 609);
    assert!(
        (all.base_cost_dollars / 1e6 - 465.0).abs() < 1.0,
        "statewide base cost rises ${:.1}m, not $465.0m",
        all.base_cost_dollars / 1e6
    );
    assert!(
        (all.dollars / 1e6 - 220.3).abs() < 1.0,
        "statewide aid rises ${:.1}m, not $220.3m",
        all.dollars / 1e6
    );

    // The gap is the guarantee. Every district the refresh fails to move is one the guarantee
    // pays under both runs — there is no other way to be unmoved by a lever that only ever
    // raises computed aid.
    assert_eq!(all.reach.unmoved, 254);
    assert_eq!(
        all.reach.held_throughout, all.reach.unmoved,
        "a district unmoved by a rise in its own base cost must be one a floor is paying"
    );
    assert_eq!(all.reach.lifted_off, 40);
    assert!(
        (all.captured() - 0.443).abs() < 0.005,
        "the state captures {:.1}% of the increase, not 44.3%",
        all.captured() * 100.0
    );
    // The two channels are carried apart because they are two freezes in law, not one.
    assert!(
        (all.categorical_dollars / 1e6 - 32.2).abs() < 0.5,
        "the categorical channel is ${:.1}m, not $32.2m",
        all.categorical_dollars / 1e6
    );
}

/// A refresh can only raise aid, and never past the whole of the increase it causes.
#[test]
fn no_district_loses_and_none_collects_more_than_its_own_increase() {
    for row in rows() {
        assert!(
            row.dollars() >= -refresh::MOVED,
            "{} loses ${:.2} to a cost input rising",
            row.name,
            -row.dollars()
        );
        if let Some(captured) = row.captured() {
            assert!(
                (-0.001..=1.001).contains(&captured),
                "{} captures {captured:.4} of its own computed increase",
                row.name
            );
        }
        // And a district off the guarantee before cannot be pushed onto it by a rise.
        assert!(
            !(row.on_guarantee_after && !row.on_guarantee_before),
            "{} was pushed onto the guarantee by an increase in computed aid",
            row.name
        );
    }
}

/// **The hypothesis, first clause: refuted, and in the opposite direction.**
#[test]
fn industrial_districts_do_better_than_average_under_a_refresh() {
    let rows = rows();
    let all = refresh::statewide(&rows);
    let industrial = total_concentrated(&rows, Base::Industrial);

    assert_eq!(industrial.reach.districts, 27);
    assert!(
        industrial.captured() > all.captured(),
        "industrial districts capture {:.1}% against a statewide {:.1}%",
        industrial.captured() * 100.0,
        all.captured() * 100.0
    );
    assert!(
        industrial.per_pupil() > all.per_pupil(),
        "industrial districts collect ${:.2} per pupil against a statewide ${:.2}",
        industrial.per_pupil(),
        all.per_pupil()
    );
}

/// **The population the claim describes is the mineral one, and the refresh cannot reach it.**
#[test]
fn the_refresh_does_not_reach_the_mineral_districts_at_all() {
    let rows = rows();
    let mineral = total_concentrated(&rows, Base::Mineral);

    assert_eq!(mineral.reach.districts, 10);
    assert_eq!(
        mineral.reach.held_throughout, 9,
        "nine of the ten mineral districts are paid by the guarantee under both runs"
    );
    assert!(
        mineral.captured() < 0.10,
        "mineral districts capture {:.1}% of their own computed increase",
        mineral.captured() * 100.0
    );
    assert!(
        mineral.per_pupil() < 25.0,
        "mineral districts collect ${:.2} per pupil",
        mineral.per_pupil()
    );

    // The comparison that makes the point: they are worse served than industrial districts by an
    // order of magnitude, on the same lever, in the same run.
    let industrial = total_concentrated(&rows, Base::Industrial);
    assert!(
        industrial.per_pupil() > mineral.per_pupil() * 5.0,
        "industrial ${:.2} against mineral ${:.2} per pupil",
        industrial.per_pupil(),
        mineral.per_pupil()
    );
}

/// **The hypothesis, second clause: confirmed, and monotone.**
#[test]
fn the_freeze_falls_hardest_on_districts_losing_pupils() {
    let rows = rows();
    let quartiles = enrollment_quartiles();
    let totals: Vec<Total> = quartiles.iter().map(|q| total_over(&rows, q)).collect();

    // Captured share rises monotonically from the fastest-declining quartile to the growing one.
    for window in totals.windows(2) {
        assert!(
            window[1].captured() > window[0].captured(),
            "captured share is not monotone: {:.1}% then {:.1}%",
            window[0].captured() * 100.0,
            window[1].captured() * 100.0
        );
    }
    // As does the per-pupil amount.
    for window in totals.windows(2) {
        assert!(
            window[1].per_pupil() > window[0].per_pupil(),
            "per-pupil aid is not monotone across the enrollment quartiles"
        );
    }

    let (worst, best) = (&totals[0], &totals[3]);
    assert!(
        (worst.captured() - 0.314).abs() < 0.01,
        "the fastest-declining quartile captures {:.1}%, not 31.4%",
        worst.captured() * 100.0
    );
    assert!(
        (best.captured() - 0.517).abs() < 0.01,
        "the growing quartile captures {:.1}%, not 51.7%",
        best.captured() * 100.0
    );

    // And the mechanism is visible in the same table: the guarantee holds 74 of the fastest
    // declining quartile and 42 of the growing one. The gradient in aid is that gradient.
    assert!(
        worst.reach.held_throughout > best.reach.held_throughout,
        "the guarantee holds {} of the declining quartile and {} of the growing one",
        worst.reach.held_throughout,
        best.reach.held_throughout
    );
}

/// The refresh is a lower bound, and the module says so rather than leaving it to be assumed.
#[test]
fn only_one_of_the_frozen_inputs_is_moved() {
    // The other nine salary categories are held at FY2022, so the statewide figure is a floor.
    let factors = foundation::StatewideFactors::fy2027();
    assert!((factors.teacher_salary - refresh::FY2022_TEACHER_SALARY).abs() < f64::EPSILON);
    const { assert!(refresh::FY2024_TEACHER_SALARY > refresh::FY2022_TEACHER_SALARY) };

    // And the categorical multiplicands are a separate freeze, at a third reference year again.
    // A refresh of those is not this lever and would reach a different set of districts.
    assert!(
        (project::panel::AVERAGE_BASE_COST_PER_PUPIL - 8_241.61).abs() < 0.01,
        "the categorical multiplicand is no longer the FY2024 figure this lever leaves alone"
    );
}
