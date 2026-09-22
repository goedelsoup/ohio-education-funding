//! The two terms for all 609 districts, the wealth gradient among the ones that lost pupils, and
//! what a fitted partition would have found.
//!
//! Issue #396 asked for either a multivariate partition across all 609 districts or a decision
//! not to build one. [`project::lost_pupils`] is the module under test, and these tests are the
//! measurement the decision rests on: the identity from `guarantee_origin` extended to every
//! district it reaches, a cut on wealth among the districts that lost pupils, the cluster's
//! siblings outside the guarantee, and a k-means on six profile variables — run once, from
//! known-answer tests in `dispersion::partition`, to show what it recovers.
//!
//! # What it found
//!
//! Among the 514 districts with fewer pupils than in FY2020, the guarantee holds 15 of the
//! least-wealthy fifth and 81 of the wealthiest. The per-pupil term crosses zero between the
//! third and fourth fifths; the enrollment cluster sits on the crossing. Sixty-four formula
//! districts lost pupils at the cluster's own rate and are poorer than it — the plan's per-pupil
//! raise carried them over the floor. And a fitted partition on six profile variables never
//! holds a majority of the cluster in any cell, at any `k` from 2 to 9, from either seeding.

use std::collections::BTreeMap;

use edfund_core::FiscalYear;
use project::guarantee_origin::{self, Origin};
use project::lost_pupils::{
    self, by_capacity, ceilings, cluster_median_enrollment_term, fitted, fitted_from_typology,
    neighbours, profile, siblings, siblings_headroom, standings, typology_ceilings, who,
    Population, Standing, FEATURES,
};
use project::panel::{panel, DistrictRecord};
use project::rolling_anchor::{self, Anchor};
use project::series::{Method, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT};

fn of(rows: &[Standing], population: Population) -> Vec<&Standing> {
    rows.iter().filter(|s| s.population == population).collect()
}

/// **The identity reaches 607 districts, names the two it cannot, and agrees with the split.**
#[test]
fn the_identity_reaches_every_district_but_two_and_agrees_with_the_guaranteed_split() {
    let districts = panel();
    assert_eq!(districts.len(), 609);
    let (rows, unreached) = standings(&districts);
    assert_eq!(rows.len(), 607, "reached");
    assert_eq!(
        unreached
            .iter()
            .map(|u| (u.irn.as_str(), u.why))
            .collect::<Vec<_>>(),
        vec![
            ("046599", "no funding base published"),
            ("047787", "the enrollment index does not reach it"),
        ],
        "{unreached:?}"
    );
    assert_eq!(unreached[0].name, "Richmond Heights Local");
    assert_eq!(unreached[1].name, "Buckeye Local");

    // The four populations are the arc's own counts, less one district on each side.
    let count = |p: Population| of(&rows, p).len();
    assert_eq!(count(Population::Formula), 314);
    assert_eq!(
        count(Population::MinimumShare),
        106,
        "107 less Buckeye Local"
    );
    assert_eq!(count(Population::EnrollmentLoss), 89);
    assert_eq!(count(Population::CapacityGrowth), 98);
    // Richmond Heights is a formula district whose base the sheet does not publish; Buckeye
    // Local is one of the 107, and the one district the survey does not reach in both years —
    // `guarantee_origin::enrollment_index` names it, and the 187 never included it.
    let richmond = districts.iter().find(|r| r.irn == "046599").unwrap();
    assert!(!richmond.on_guarantee() && richmond.guarantee_floor() == 0.0);
    let buckeye = districts.iter().find(|r| r.irn == "047787").unwrap();
    assert!(buckeye.on_guarantee() && buckeye.at_minimum_state_share());
    assert_eq!(
        districts.iter().filter(|r| r.on_guarantee()).count() - 1,
        293
    );

    // Where both are defined they are the same numbers, and the origin is the same origin.
    let index = guarantee_origin::enrollment_index(&districts);
    let by_irn: BTreeMap<&str, &DistrictRecord> =
        districts.iter().map(|r| (r.irn.as_str(), r)).collect();
    let mut compared = 0;
    for s in rows.iter().filter(|s| s.population.held()) {
        let split = guarantee_origin::decompose(by_irn[s.irn.as_str()], &index)
            .expect("a guaranteed district decomposes");
        assert!((split.total() - s.terms.total()).abs() < 1e-12, "{}", s.irn);
        assert!((split.per_pupil_term - s.terms.per_pupil_term).abs() < 1e-12);
        assert_eq!(split.origin(), s.terms.origin());
        compared += 1;
    }
    assert_eq!(compared, 293);
    // And the identity holds for every row, held or not.
    for s in &rows {
        assert!((s.terms.total() - s.terms.multiple.ln()).abs() < 1e-12);
        assert!((s.terms.multiple - s.floor / s.formula).abs() < 1e-9);
        assert_eq!(s.terms.multiple > 1.0, s.population.held(), "{}", s.irn);
    }
    // A formula district's terms have no majority to take.
    assert!(of(&rows, Population::Formula)
        .iter()
        .all(|s| s.terms.origin().is_none()));
    assert_eq!(
        guarantee_origin::terms(by_irn["046599"], &index),
        None,
        "no floor, no multiple"
    );
}

/// **Off the guarantee, the per-pupil raise outran the lost pupils.**
#[test]
fn off_the_guarantee_the_per_pupil_raise_outran_the_lost_pupils() {
    let districts = panel();
    let (rows, _) = standings(&districts);
    let formula = of(&rows, Population::Formula);
    assert_eq!(formula.len(), 314);
    assert_eq!(formula.iter().filter(|s| s.lost_pupils()).count(), 252);
    let paid_more_per_pupil = formula
        .iter()
        .filter(|s| s.terms.per_pupil_term < 0.0)
        .count();
    assert_eq!(paid_more_per_pupil, 310);
    let exceptions: Vec<&&Standing> = formula
        .iter()
        .filter(|s| s.terms.per_pupil_term >= 0.0)
        .collect();
    assert_eq!(exceptions.len(), 4);
    assert!(
        exceptions.iter().all(|s| !s.lost_pupils()),
        "every district off the floor with a lower per-pupil payment grew: {:?}",
        exceptions.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
    // Every formula district has headroom, by construction.
    assert!(formula.iter().all(|s| s.headroom() > 0.0));
}

/// **Among the districts that lost pupils, the guarantee's reach rises with wealth, and the
/// cluster sits where the per-pupil term crosses zero.**
#[test]
fn among_the_districts_that_lost_pupils_the_guarantee_runs_with_wealth() {
    let districts = panel();
    let (rows, _) = standings(&districts);
    let lost = who(&rows, true);
    let grew = who(&rows, false);
    assert_eq!(lost.len(), 514);
    assert_eq!(grew.len(), 93);

    let fifths = by_capacity(&lost);
    assert_eq!(
        fifths.iter().map(|f| f.districts).collect::<Vec<_>>(),
        vec![102, 102, 102, 102, 106]
    );
    assert_eq!(
        fifths.iter().map(|f| f.held).collect::<Vec<_>>(),
        vec![15, 26, 56, 84, 81],
        "held, least wealthy first"
    );
    assert_eq!(
        fifths
            .iter()
            .map(|f| f.by_population[Population::EnrollmentLoss.index()])
            .collect::<Vec<_>>(),
        vec![15, 25, 40, 7, 2],
        "the enrollment cluster"
    );
    assert_eq!(
        fifths
            .iter()
            .map(|f| f.by_population[Population::CapacityGrowth.index()])
            .collect::<Vec<_>>(),
        vec![0, 1, 16, 67, 4],
        "the capacity cluster"
    );
    assert_eq!(
        fifths
            .iter()
            .map(|f| f.by_population[Population::MinimumShare.index()])
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 10, 75],
        "the minimum share"
    );
    // The per-pupil term crosses zero between the third and fourth fifths.
    let terms: Vec<f64> = fifths.iter().map(|f| f.median_per_pupil_term).collect();
    assert!(terms[0] < 0.0 && terms[1] < 0.0 && terms[2] < 0.0);
    assert!(terms[3] > 0.0 && terms[4] > 0.0);
    assert!((terms[0] + 0.2740).abs() < 0.0005, "{terms:?}");
    assert!((terms[2] + 0.0742).abs() < 0.0005, "{terms:?}");
    assert!((terms[3] - 0.3441).abs() < 0.0005, "{terms:?}");
    assert!((terms[4] - 0.5719).abs() < 0.0005, "{terms:?}");
    // And the held rate is monotone in wealth until the top, where the minimum share takes over.
    assert!(fifths[0].held < fifths[1].held && fifths[1].held < fifths[2].held);
    assert!(fifths[2].held < fifths[3].held);
    // The enrollment term itself is flat across the fifths: wealth is not a proxy for decline.
    let decline: Vec<f64> = fifths.iter().map(|f| f.median_enrollment_term).collect();
    let spread = decline.iter().copied().fold(f64::MIN, f64::max)
        - decline.iter().copied().fold(f64::MAX, f64::min);
    assert!(spread < 0.02, "{decline:?}");

    // The growing districts show the same gradient with the lost pupils taken out.
    let growing = by_capacity(&grew);
    assert_eq!(
        growing.iter().map(|f| f.held).collect::<Vec<_>>(),
        vec![0, 1, 2, 14, 14]
    );
}

/// **The cluster's siblings are poorer, and the plan lifted them over the floor.**
#[test]
fn the_clusters_siblings_are_poorer_and_the_plan_lifted_them_over_the_floor() {
    let districts = panel();
    let (rows, _) = standings(&districts);
    let threshold = cluster_median_enrollment_term(&rows);
    assert!((threshold - 0.126_881).abs() < 1e-6, "{threshold}");
    assert!(
        ((1.0 / threshold.exp()) - 0.8808).abs() < 0.0001,
        "11.9% fewer pupils"
    );

    let sibs = siblings(&rows);
    assert_eq!(sibs.len(), 64);
    let cluster = profile(&of(&rows, Population::EnrollmentLoss));
    let them = siblings_headroom(&districts, &sibs);
    let s = &them.profile;
    assert_eq!(cluster.districts, 89);
    assert!((cluster.adm - 196_869.46).abs() < 0.01);
    assert!((s.adm - 81_391.98).abs() < 0.01, "{}", s.adm);

    // Poorer on the profile's measure, on the DPIA share, on capacity, and on state share.
    assert!((cluster.median_disadvantaged - 0.5215).abs() < 0.0001);
    assert!(
        (s.median_disadvantaged - 0.5817).abs() < 0.0001,
        "{}",
        s.median_disadvantaged
    );
    assert!(s.median_poverty > cluster.median_poverty);
    assert!((cluster.median_capacity - 5183.86).abs() < 0.01);
    assert!(
        (s.median_capacity - 4235.15).abs() < 0.01,
        "{}",
        s.median_capacity
    );
    assert!((cluster.median_state_share - 0.396).abs() < 0.0005);
    assert!(
        (s.median_state_share - 0.497).abs() < 0.0005,
        "{}",
        s.median_state_share
    );
    // Shrinking as fast, paid far more per pupil.
    assert!(s.median_enrollment_term >= threshold);
    assert!((cluster.median_per_pupil_term + 0.0432).abs() < 0.0001);
    assert!(
        (s.median_per_pupil_term + 0.3158).abs() < 0.0001,
        "{}",
        s.median_per_pupil_term
    );
    assert!(
        (them.median_formula_per_pupil - 8542.0).abs() < 1.0,
        "{}",
        them.median_formula_per_pupil
    );
    assert!(
        (them.median_base_per_pupil - 5776.0).abs() < 1.0,
        "{}",
        them.median_base_per_pupil
    );

    // Their distance from the floor.
    assert!(
        (them.headroom - 100_805_627.62).abs() < 0.01,
        "{}",
        them.headroom
    );
    assert!(
        (them.median_headroom - 1_201_355.45).abs() < 0.01,
        "{}",
        them.median_headroom
    );
    assert!(
        (them.median_headroom_per_pupil - 1050.22).abs() < 0.01,
        "{}",
        them.median_headroom_per_pupil
    );
    assert!(
        (them.median_formula_over_floor - 1.1794).abs() < 0.0001,
        "{}",
        them.median_formula_over_floor
    );
    assert_eq!(them.falling, 62);
    assert!(
        (them.median_years_to_floor - 4.14).abs() < 0.01,
        "{}",
        them.median_years_to_floor
    );
    // The headroom table and the identity agree on what the gap is.
    for sib in &sibs {
        assert!(sib.formula - sib.floor > 0.0);
    }
    let by_identity: f64 = sibs.iter().map(|s| s.formula - s.floor).sum();
    assert!((by_identity - them.headroom).abs() < 0.01);

    // At the shipped damping, the enacted anchor holds five of them by FY2032.
    let walk = rolling_anchor::walk(
        &districts,
        FiscalYear(2032),
        Method::Shrunk {
            rate: 0.0,
            damping: DEFAULT_DAMPING,
            weight: DEFAULT_SHRINK_WEIGHT,
            toward: 0.0,
        },
        Anchor::Fixed,
    );
    let held = sibs
        .iter()
        .filter(|s| walk.get(&s.irn).is_some_and(|w| w.on_the_floor()))
        .count();
    assert_eq!(held, 5);
    let held_of_all_formula = of(&rows, Population::Formula)
        .iter()
        .filter(|s| walk.get(&s.irn).is_some_and(|w| w.on_the_floor()))
        .count();
    assert_eq!(
        held_of_all_formula, 18,
        "294 + 18 is the 312 the corpus states for FY2032"
    );
}

/// **A fitted partition on six profile variables finds the cluster at no `k`.**
#[test]
fn a_fitted_partition_finds_the_cluster_at_no_k() {
    let districts = panel();
    let (rows, _) = standings(&districts);
    assert_eq!(FEATURES.len(), 6);
    let (kept, columns) = lost_pupils::features(&rows);
    assert_eq!(kept.len(), 607, "every row publishes all six");
    assert!(columns.iter().all(|c| c.len() == 607));

    let one_cell = ceilings(&vec![0; 607], &rows.iter().collect::<Vec<_>>());
    assert_eq!(one_cell.membership, 314);
    assert_eq!(one_cell.four_way, 314);
    assert_eq!(one_cell.cluster, 518);
    assert_eq!(one_cell.cluster_members, 89);
    let trivial = one_cell.districts - one_cell.cluster_members;

    let mut membership = Vec::new();
    let mut four_way = Vec::new();
    for k in 2..=9 {
        let fit = fitted(&rows, k);
        assert_eq!(fit.k, k);
        assert!(
            fit.ceilings.cluster <= trivial + 1,
            "k={k}: the cluster is a majority somewhere ({} against {trivial})",
            fit.ceilings.cluster
        );
        assert!(fit.iterations < 100, "k={k} took {} passes", fit.iterations);
        membership.push(fit.ceilings.membership);
        four_way.push(fit.ceilings.four_way);
        if k == 2 {
            // Farthest-first seeds at the extremes, and the extreme is an island.
            assert_eq!(fit.singletons, 1);
        }
    }
    assert_eq!(membership, vec![314, 359, 379, 387, 385, 424, 415, 432]);
    assert_eq!(four_way, vec![314, 314, 314, 314, 314, 346, 338, 340]);

    // The baseline: the department's 2013 typology, scored the same way.
    let typology = typology_ceilings(&rows);
    assert_eq!(typology.membership, 384);
    assert_eq!(typology.four_way, 333);
    assert_eq!(typology.cluster, 518);
    assert_eq!(
        membership[7] - typology.membership,
        48,
        "nine cells against nine categories"
    );
    assert!(
        (membership[7] as f64) / 607.0 < 0.75,
        "and neither reaches three quarters"
    );

    // Started from the typology's own centres rather than the extremes, the same answer on the
    // cluster.
    let seeded = fitted_from_typology(&rows);
    assert_eq!(
        seeded.k, 9,
        "every one of the 607 carries a code, so nine cells"
    );
    assert_eq!(seeded.ceilings.membership, 415);
    assert_eq!(seeded.ceilings.four_way, 362);
    assert_eq!(seeded.ceilings.cluster, 518, "the trivial ceiling exactly");
    assert_eq!(seeded.singletons, 1, "the island again");
    assert!(
        seeded.ceilings.cluster <= trivial + 1,
        "{}",
        seeded.ceilings.cluster
    );
    assert!(
        seeded.ceilings.membership >= typology.membership,
        "a descent from the baseline does not lose to it: {} against {}",
        seeded.ceilings.membership,
        typology.membership
    );

    // Nothing fitted: the nearest neighbour.
    let near = neighbours(&rows);
    assert_eq!(near.districts, 607);
    assert_eq!(
        near.by_population[Population::EnrollmentLoss.index()],
        (31, 89)
    );
    assert_eq!(near.membership, 449);
    // Chance for the cluster is its share of the population, 89 of 607; the observed rate is
    // more than twice it.
    let (hits, members) = near.by_population[Population::EnrollmentLoss.index()];
    let observed = hits as f64 / members as f64;
    let chance = members as f64 / near.districts as f64;
    assert!(
        observed > 2.0 * chance,
        "a region, if a thin one: {observed} against {chance}"
    );
    // The two origins the identity names are no more separable from each other than the
    // cluster is from everyone: the capacity cluster's neighbour is a member 61 times of 98.
    assert_eq!(
        near.by_population[Population::CapacityGrowth.index()],
        (61, 98)
    );
    assert_eq!(
        near.by_population[Population::MinimumShare.index()],
        (71, 106)
    );
    assert_eq!(near.by_population[Population::Formula.index()], (226, 314));
    let _ = Origin::EnrollmentLoss;
}
