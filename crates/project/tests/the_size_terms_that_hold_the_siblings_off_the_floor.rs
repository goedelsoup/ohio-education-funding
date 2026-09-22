//! The two size-dependent terms, priced on the enrollment cluster's 64 siblings.
//!
//! Issue #435 carried an uncommitted probe: striking out R.C. 3317.011's staffing floors and
//! R.C. 3317.0217's capacity tier puts 35 of the 64 siblings on the guarantee. Both
//! counterfactuals were already committed in `project::size_terms` and the sibling set in
//! `project::lost_pupils`, so this is a join and nothing here is estimated.
//!
//! # What it found
//!
//! The table reproduces: 46, 87 and 136 districts newly guaranteed statewide; 16, 23 and 35 of
//! the 64; $346, $708 and $1,054 a pupil to the siblings against $123, $261 and $384 to the
//! cluster.
//!
//! And the mechanism is size and not decline. Band for band on base cost enrolled ADM the
//! siblings move at their band's non-sibling rate, and giving each band's siblings that rate
//! predicts **34.60** of the 35. The 29 the terms do not reach are the larger siblings — a
//! median 1,846 ADM against 636 — and what holds them off the floor is the state share of base
//! cost, not DPIA: the base cost share alone exceeds the gap for 28 of the 29 and DPIA for 4.
//!
//! The population the arc calls "the 153" is the cluster and the siblings unioned. "Districts
//! shrinking at the cluster's median rate or faster" is a different set and holds 169.

use project::lost_pupils::{standings, Population, Standing};
use project::panel::panel;
use project::size_incidence::{
    by_sextile, carried, cluster_and_siblings, component_totals, covering, expected_from_size,
    reach, shrinking_at_the_cluster_rate, table, unmoved, COMPONENTS,
};
use project::size_terms::{Change, BANDS};

/// Both terms struck out — the row every section below reads.
const BOTH: [Change; 2] = [
    Change::StaffingFloorsStruckOut,
    Change::CapacityTierStruckOut,
];

/// **The table #435 carried as a probe, committed.**
#[test]
fn the_two_size_terms_hold_thirty_five_of_the_sixty_four_off_the_floor() {
    let districts = panel();
    let (rows, _) = standings(&districts);
    let priced = table(&districts, &rows);
    assert_eq!(priced.len(), 3);

    assert_eq!(
        priced.iter().map(|r| r.statewide).collect::<Vec<_>>(),
        vec![46, 87, 136],
        "newly guaranteed statewide"
    );
    assert_eq!(
        priced
            .iter()
            .map(|r| r.siblings.newly_guaranteed)
            .collect::<Vec<_>>(),
        vec![16, 23, 35],
        "of the 64 siblings"
    );
    // Every row reaches all 64, and none of them is on the guarantee to begin with.
    assert!(priced
        .iter()
        .all(|r| r.siblings.districts == 64 && r.siblings.already_guaranteed == 0));
    // The cluster is already held, so nothing there is newly anything — its column is the
    // comparison and not a second finding.
    assert!(priced
        .iter()
        .all(|r| r.cluster.districts == 89 && r.cluster.already_guaranteed == 89));
    assert!(priced.iter().all(|r| r.cluster.newly_guaranteed == 0));

    let per_pupil = |pick: fn(&project::size_incidence::Row) -> f64| -> Vec<i64> {
        priced.iter().map(|r| pick(r).round() as i64).collect()
    };
    assert_eq!(
        per_pupil(|r| r.siblings.per_pupil()),
        vec![346, 708, 1054],
        "worth to the siblings, per base-cost pupil"
    );
    assert_eq!(
        per_pupil(|r| r.cluster.per_pupil()),
        vec![123, 261, 384],
        "and to the cluster"
    );
    // The pair is worth 2.7 times as much per pupil to the siblings as to the floor's own
    // population.
    let ratio = priced[2].siblings.per_pupil() / priced[2].cluster.per_pupil();
    assert!((ratio - 2.74).abs() < 0.01, "{ratio}");

    // Each term alone, and the two together: the dollars add, because the channels are disjoint.
    assert!(
        (priced[0].siblings.aid_removed + priced[1].siblings.aid_removed
            - priced[2].siblings.aid_removed)
            .abs()
            < 0.01
    );
    assert!(
        (priced[2].siblings.aid_removed - 88_665_455.90).abs() < 0.01,
        "{}",
        priced[2].siblings.aid_removed
    );
    // The counts do not: 16 and 23 make 35 and not 39, because seven districts either term
    // alone would move are moved once.
    assert!(
        priced[0].siblings.newly_guaranteed + priced[1].siblings.newly_guaranteed
            > priced[2].siblings.newly_guaranteed
    );

    // The siblings are 35 of the 136 the pair moves statewide, and that is the share to report.
    assert_eq!(priced[2].statewide, 136);
    let share = priced[2].siblings.newly_guaranteed as f64 / priced[2].statewide as f64;
    assert!((share - 0.257).abs() < 0.001, "{share}");
}

/// **Conditioned on size, being a sibling is worth four tenths of a district.**
#[test]
fn it_is_size_that_catches_them_and_the_sibling_axis_adds_nothing() {
    let districts = panel();
    let (rows, _) = standings(&districts);
    let bands = by_sextile(&districts, &rows, &BOTH);
    assert_eq!(bands.len(), BANDS);
    assert_eq!(
        bands.iter().map(|b| b.siblings).collect::<Vec<_>>(),
        vec![(22, 25), (7, 10), (6, 10), (0, 5), (0, 11), (0, 3)],
        "siblings moved of siblings, smallest band first"
    );
    assert_eq!(
        bands.iter().map(|b| b.others).collect::<Vec<_>>(),
        vec![(49, 58), (36, 52), (12, 20), (4, 36), (0, 41), (0, 44)],
        "and every other formula district"
    );
    // The bands carry the whole formula population and the whole of the 64.
    assert_eq!(bands.iter().map(|b| b.siblings.1).sum::<usize>(), 64);
    assert_eq!(bands.iter().map(|b| b.siblings.0).sum::<usize>(), 35);
    assert_eq!(bands.iter().map(|b| b.others.1).sum::<usize>(), 251);
    assert_eq!(bands.iter().map(|b| b.others.0).sum::<usize>(), 101);

    // Unconditionally the siblings look singled out: 54.7% against 40.2%.
    let sibling_rate: f64 = 35.0 / 64.0;
    let other_rate: f64 = 101.0 / 251.0;
    assert!((sibling_rate - 0.547).abs() < 0.001);
    assert!((other_rate - 0.402).abs() < 0.001);
    assert!(sibling_rate > other_rate);

    // Band for band they are not. Give each band's siblings that band's non-sibling rate and
    // the prediction is 34.60 of the 35 observed.
    let expected = expected_from_size(&bands);
    assert!((expected - 34.60).abs() < 0.005, "{expected}");
    assert!(
        (expected - 35.0).abs() < 1.0,
        "size alone predicts the count inside a district: {expected}"
    );
    // And the residual is not one-signed: the fourth band runs the other way.
    assert!(bands[0].siblings.0 as f64 / bands[0].siblings.1 as f64 > bands[0].others_rate());
    assert!(bands[3].others_rate() > 0.0 && bands[3].siblings.0 == 0);
    // The two largest bands move nobody at all, sibling or not.
    assert!(bands[4].siblings.0 == 0 && bands[4].others.0 == 0);
    assert!(bands[5].siblings.0 == 0 && bands[5].others.0 == 0);
}

/// **The 29 the terms do not reach are carried by the state share of base cost, not by DPIA.**
#[test]
fn the_remaining_twenty_nine_are_carried_by_the_state_share() {
    let districts = panel();
    let (rows, _) = standings(&districts);
    let rest = unmoved(&districts, &rows, &BOTH);
    assert_eq!(rest.len(), 29, "64 less the 35 the two terms move");

    // They are the larger siblings, which is why neither term binds on them.
    let median = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let moved_adm = {
        let moved: Vec<&Standing> = {
            let rest_irns: Vec<&str> = rest.iter().map(|s| s.irn.as_str()).collect();
            project::lost_pupils::siblings(&rows)
                .into_iter()
                .filter(|s| !rest_irns.contains(&s.irn.as_str()))
                .collect()
        };
        assert_eq!(moved.len(), 35);
        median(moved.iter().map(|s| s.adm).collect())
    };
    let rest_adm = median(rest.iter().map(|s| s.adm).collect());
    assert!((rest_adm - 1846.0).abs() < 0.5, "{rest_adm}");
    assert!((moved_adm - 635.6).abs() < 0.5, "{moved_adm}");
    assert!(rest_adm > 2.0 * moved_adm);

    let carry = carried(&districts, &rest);
    assert_eq!(carry.len(), 29);
    // The seven components reconstruct `[H]` for each of them.
    for (row, standing) in carry.iter().zip(&rest) {
        let total: f64 = row.components.iter().sum();
        assert!(
            (total - standing.formula).abs() < 0.05,
            "{}: {total} against {}",
            row.irn,
            standing.formula
        );
        assert!(row.gap > 0.0, "{} is a formula district", row.irn);
    }
    let gap: f64 = carry.iter().map(|row| row.gap).sum();
    let adm: f64 = carry.iter().map(|row| row.adm).sum();
    assert!((gap - 68_572_895.46).abs() < 0.01, "{gap}");
    assert!((gap / adm - 1144.0).abs() < 1.0, "{}", gap / adm);

    assert_eq!(
        COMPONENTS,
        [
            "base cost state share",
            "targeted assistance",
            "special education",
            "DPIA",
            "English learners",
            "gifted",
            "career-technical",
        ]
    );
    let covers = covering(&carry);
    assert_eq!(covers, [28, 14, 9, 4, 0, 1, 0]);
    // It is the state share, and DPIA alone reaches four of the twenty-nine.
    assert_eq!(covers[0], 28, "the base cost state share");
    assert_eq!(covers[3], 4, "DPIA");
    assert!(covers[0] > 6 * covers[3]);

    let totals = component_totals(&carry);
    assert!((totals[0] - 208_123_388.0).abs() < 1.0, "{}", totals[0]);
    assert!((totals[3] - 36_497_037.0).abs() < 1.0, "{}", totals[3]);
    // The base cost share is three times the gap in aggregate; DPIA is half of it.
    assert!((totals[0] / gap - 3.04).abs() < 0.01);
    assert!((totals[3] / gap - 0.53).abs() < 0.01);
    // Nothing here says a component pays the gap: the whole of `[H]` does. What it says is
    // which lines are large enough to be in the running at all.
    assert!(totals.iter().sum::<f64>() > gap);
}

/// **"The 153" is a union, and "shrinking at the cluster's rate" is 169.**
#[test]
fn the_hundred_and_fifty_three_is_not_the_population_that_shrinks_at_the_clusters_rate() {
    let districts = panel();
    let (rows, _) = standings(&districts);

    let union = cluster_and_siblings(&rows);
    assert_eq!(union.len(), 153, "the cluster's 89 and the siblings' 64");
    assert_eq!(
        union
            .iter()
            .filter(|s| s.population == Population::EnrollmentLoss)
            .count(),
        89
    );
    assert_eq!(
        union
            .iter()
            .filter(|s| s.population == Population::Formula)
            .count(),
        64
    );

    let shrinking = shrinking_at_the_cluster_rate(&rows);
    assert_eq!(shrinking.len(), 169, "not 153");
    let by = |p: Population| shrinking.iter().filter(|s| s.population == p).count();
    assert_eq!(by(Population::Formula), 64, "the siblings, by definition");
    assert_eq!(
        by(Population::EnrollmentLoss),
        45,
        "of the cluster's own 89"
    );
    assert_eq!(by(Population::MinimumShare), 31);
    assert_eq!(by(Population::CapacityGrowth), 29);
    // A median cuts its own population in half, so 44 of the 89 do not clear their own
    // threshold, and 60 guaranteed districts outside the cluster do.
    assert_eq!(89 - by(Population::EnrollmentLoss), 44);
    assert_eq!(
        by(Population::MinimumShare) + by(Population::CapacityGrowth),
        60
    );

    // Both terms struck out reaches the union without taking anyone off the floor.
    let on_the_union = reach(&districts, &BOTH, &union);
    assert_eq!(on_the_union.districts, 153);
    assert_eq!(on_the_union.already_guaranteed, 89);
    assert_eq!(on_the_union.newly_guaranteed, 35);
    let on_the_shrinking = reach(&districts, &BOTH, &shrinking);
    assert_eq!(on_the_shrinking.districts, 169);
    assert_eq!(on_the_shrinking.newly_guaranteed, 35, "the siblings again");
}
