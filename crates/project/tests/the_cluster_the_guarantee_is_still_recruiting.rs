//! What the enrollment cluster's decline looks like, before any response to it is priced.
//!
//! [`project::guarantee_origin`] established that 89 of the 187 guaranteed districts the minimum
//! state share does not explain are on the floor for no reason but having fewer children than in
//! FY2020, and called them a candidate blind spot. These tests characterize that cluster on the
//! three questions a design has to answer first: how fast the decline runs, whether any of it is
//! recoverable, and whether the population is stable.
//!
//! The third answer is the one that matters. It is not stable, and it is not stable in the
//! direction that makes a temporary instrument permanent: 50 of the 89 crossed onto the guarantee
//! between FY2026 and FY2027, from a formula that was paying them the year before.

use project::enrollment_decline::{self as decline, Legs};
use project::guarantee_origin::{self, Origin};
use project::panel::{panel, DistrictRecord, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL};
use project::prior_model;

/// The crates' median: the upper middle, never the mean of two.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty());
    values.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    values[values.len() / 2]
}

fn of(rows: &[DistrictRecord], origin: Origin) -> Vec<&DistrictRecord> {
    let index = guarantee_origin::enrollment_index(rows);
    guarantee_origin::above_the_minimum(rows)
        .into_iter()
        .filter(|r| guarantee_origin::decompose(r, &index).is_some_and(|d| d.origin() == origin))
        .collect()
}

#[test]
fn the_cluster_is_the_one_guarantee_origin_named() {
    let districts = panel();
    let mine = decline::cluster(&districts);
    let theirs = of(&districts, Origin::EnrollmentLoss);

    assert_eq!(mine.len(), 89, "the enrollment cluster is 89 districts");
    assert_eq!(
        mine.iter().map(|r| r.irn.clone()).collect::<Vec<_>>(),
        theirs.iter().map(|r| r.irn.clone()).collect::<Vec<_>>(),
        "this module characterizes a different population than the one that found it"
    );

    let dollars: f64 = mine.iter().map(|r| r.guarantee).sum();
    assert!(
        (dollars - 108_568_921.47).abs() < 0.01,
        "the cluster holds ${dollars:.2}"
    );
}

#[test]
fn the_decline_is_steeper_after_the_seam_than_before_it() {
    let districts = panel();
    let legs = decline::legs(&districts);

    let loss = decline::shape(&decline::cluster(&districts), &legs);
    let growth = decline::shape(&of(&districts, Origin::CapacityGrowth), &legs);
    let untouched: Vec<&DistrictRecord> = districts.iter().filter(|r| !r.on_guarantee()).collect();
    let untouched = decline::shape(&untouched, &legs);

    // Every group is shrinking, and every group is shrinking faster than it was.
    for (name, shape) in [
        ("enrollment", &loss),
        ("capacity", &growth),
        ("untouched", &untouched),
    ] {
        assert!(
            shape.median_survey < 0.0 && shape.median_departmental < 0.0,
            "{name} is not shrinking on both legs"
        );
        assert!(
            shape.median_departmental < shape.median_survey,
            "{name} is not shrinking faster after the seam: {:.4} then {:.4}",
            shape.median_survey,
            shape.median_departmental
        );
    }

    assert!(
        (loss.median_survey - -0.018_804).abs() < 0.000_05,
        "the first leg runs at {:.6}",
        loss.median_survey
    );
    assert!(
        (loss.median_departmental - -0.027_444).abs() < 0.000_05,
        "the second runs at {:.6}",
        loss.median_departmental
    );

    // And it is steepest of the three on both legs, which is what makes it a cluster rather than
    // Ohio's general trend showing up in one subgroup.
    assert!(loss.median_survey < growth.median_survey);
    assert!(loss.median_survey < untouched.median_survey);
    assert!(loss.median_departmental < growth.median_departmental);
    assert!(loss.median_departmental < untouched.median_departmental);

    assert_eq!(loss.still_falling, 87, "still losing pupils");
    assert_eq!(loss.decelerating, 30, "losing them more slowly than before");
    assert_eq!(loss.districts - loss.decelerating, 59, "accelerating");
}

#[test]
fn not_one_of_them_began_shrinking_after_the_anchor_year() {
    let districts = panel();
    let legs = decline::legs(&districts);
    let cluster = decline::cluster(&districts);

    let mine: Vec<&Legs> = cluster.iter().filter_map(|r| legs.get(&r.irn)).collect();
    assert_eq!(
        mine.len(),
        89,
        "the fourteen-year panel reaches all of them"
    );

    let structural = mine
        .iter()
        .filter(|l| l.still_falling() && l.long_run_decline() == Some(true))
        .count();
    let recent = mine
        .iter()
        .filter(|l| l.still_falling() && l.long_run_decline() == Some(false))
        .count();

    assert_eq!(structural, 87, "shrinking across the whole panel");
    assert_eq!(
        recent, 0,
        "a district whose decline began after FY2020 would be the one a policy could have caused"
    );
    assert_eq!(
        mine.iter()
            .filter(|l| l.long_run_decline() == Some(true))
            .count(),
        89,
        "every member is a long-run decliner, still falling or not"
    );
}

#[test]
fn pupils_taught_elsewhere_are_present_and_are_not_the_account() {
    let districts = panel();
    let provenance = decline::provenance(&districts);

    let ratios = |rows: &[&DistrictRecord]| -> Vec<f64> {
        rows.iter()
            .filter_map(|r| provenance.get(&r.irn))
            .map(|p| p.resident_over_enrolled)
            .collect()
    };

    let loss = ratios(&decline::cluster(&districts));
    let growth = ratios(&of(&districts, Origin::CapacityGrowth));
    let untouched: Vec<&DistrictRecord> = districts.iter().filter(|r| !r.on_guarantee()).collect();
    let untouched = ratios(&untouched);

    let median = upper_middle(loss.clone());
    assert!(
        (median - 1.0951).abs() < 0.0001,
        "the cluster's resident-over-enrolled median is {median:.4}"
    );
    assert!(
        median > upper_middle(growth),
        "it sits above the other half"
    );
    assert!(
        median > upper_middle(untouched.clone()),
        "and above the districts the guarantee does not pay"
    );

    // But the tails overlap, which is what stops it being an account of the decline rather than
    // a minority of it. More of the untouched districts are past 1.20 than there are members of
    // this cluster past it at all.
    assert_eq!(loss.iter().filter(|v| **v > 1.20).count(), 20);
    assert_eq!(untouched.iter().filter(|v| **v > 1.20).count(), 50);
    assert!(
        untouched.iter().filter(|v| **v > 1.20).count() > loss.len() / 4,
        "the ratio would have to be rarer outside the cluster to explain membership of it"
    );
}

#[test]
fn fifty_of_the_eighty_nine_crossed_onto_the_guarantee_in_one_year() {
    let districts = panel();
    let prior = prior_model::by_irn();

    let split = |rows: &[&DistrictRecord]| -> (usize, usize, usize, f64, f64) {
        let divergences: Vec<_> = rows
            .iter()
            .filter_map(|r| prior.get(&r.irn).and_then(|p| decline::divergence(r, p)))
            .collect();
        (
            divergences.len(),
            rows.iter()
                .filter(|r| prior.get(&r.irn).is_some_and(|p| p.guarantee > 0.0))
                .count(),
            divergences
                .iter()
                .filter(|d| d.before < 1.0 && d.after > 1.0)
                .count(),
            upper_middle(divergences.iter().map(|d| d.before).collect()),
            upper_middle(divergences.iter().map(|d| d.after).collect()),
        )
    };

    let cluster = decline::cluster(&districts);
    let (reached, already, crossed, before, after) = split(&cluster);
    assert_eq!(reached, 89, "the FY2026 model carries every member");

    // In FY2026 the median member's FY2020 base sat BELOW what the formula computed for it: the
    // guarantee owed it nothing and the formula was paying it.
    assert!(
        before < 1.0,
        "the median member was on the guarantee a year earlier, at {before:.4}"
    );
    assert!((before - 0.9834).abs() < 0.0001, "{before:.4}");
    assert!((after - 1.0813).abs() < 0.0001, "{after:.4}");

    assert_eq!(already, 39, "already on the guarantee in FY2026");
    assert_eq!(crossed, 50, "crossed onto it between the two years");
    assert_eq!(
        already + crossed,
        cluster.len(),
        "the two counts are read off different columns and must still partition the cluster"
    );

    // The other half of the 187 is what a transitional instrument is supposed to look like: a
    // settled population, draining toward the minimum state share rather than filling up.
    let (_, settled, arriving, _, _) = split(&of(&districts, Origin::CapacityGrowth));
    assert_eq!(settled, 90);
    assert_eq!(arriving, 8);
    assert!(
        crossed > arriving * 6,
        "the two clusters are arriving at comparable rates, so they are one population after all"
    );
}

#[test]
fn the_gap_between_base_and_formula_is_widening_for_all_but_two() {
    let districts = panel();
    let prior = prior_model::by_irn();

    let divergences: Vec<_> = decline::cluster(&districts)
        .iter()
        .filter_map(|r| prior.get(&r.irn).and_then(|p| decline::divergence(r, p)))
        .collect();

    assert_eq!(
        divergences.iter().filter(|d| d.widening()).count(),
        87,
        "the formula is moving away from the FY2020 base for nearly every member"
    );
    let change = upper_middle(divergences.iter().map(|d| d.change()).collect());
    assert!(
        (change - 0.1031).abs() < 0.0001,
        "the median gap widened by {change:.4} of a formula amount in one year"
    );
}

#[test]
fn the_guarantee_pays_for_decline_at_twice_what_the_plan_pays_for_growth() {
    let districts = panel();
    let cluster = decline::cluster(&districts);

    let per_pupil = upper_middle(
        cluster
            .iter()
            .map(|r| r.guarantee / r.current_year_adm)
            .collect(),
    );
    assert!(
        (per_pupil - 464.06).abs() < 0.01,
        "the median member draws ${per_pupil:.2} a pupil"
    );

    // `[M]` is the plan's only explicit price on an enrollment change, and it runs the other way.
    let ratio = per_pupil / ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL;
    assert!(
        (ratio - 1.856).abs() < 0.005,
        "the guarantee pays {ratio:.3} times what [M] pays, per pupil"
    );
    assert!(
        per_pupil > ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL,
        "Ohio pays more per pupil for decline than for growth, through an instrument naming neither"
    );

    // And it is concentrated, so the headline understates how little most members draw.
    let mut dollars: Vec<f64> = cluster.iter().map(|r| r.guarantee).collect();
    dollars.sort_by(|a, b| b.partial_cmp(a).expect("no NaN"));
    let total: f64 = dollars.iter().sum();
    let top_ten: f64 = dollars.iter().take(10).sum();
    assert!(
        (top_ten / total - 0.556).abs() < 0.001,
        "the ten largest hold {:.1}% of the cluster",
        top_ten / total * 100.0
    );
}
