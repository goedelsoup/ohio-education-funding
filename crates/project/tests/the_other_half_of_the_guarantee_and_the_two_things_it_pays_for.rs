//! The 187 guaranteed districts the minimum state share does not reach, and why each one is held.
//!
//! `scenario-delta/tests/the_minimum_share_the_guarantee_has_already_paid_past.rs` settled 107 of
//! the 294 districts Ohio's temporary transitional aid guarantee pays: they sit on the 10%
//! minimum state share, the guarantee holds them at a median **2.40x** their formula amount, and
//! they hold **46.7%** of the instrument. The account is wealth, and it does not reach the rest.
//!
//! This file is the rest. **187 districts, $468,337,128.87, 465,086.26 ADM**, and
//! [`project::guarantee_origin`] is the module under test.
//!
//! # The result
//!
//! A guaranteed district's multiple over formula aid factors exactly into an enrollment term and
//! a per-pupil term, and the 187 split on which one carries the majority:
//!
//! | | districts | guarantee | ADM | median multiple | median state share | median capacity/pupil | median index | econ. disadv. |
//! |---|--:|--:|--:|--:|--:|--:|--:|--:|
//! | [`Origin::EnrollmentLoss`] | 89 | $108,568,921.47 | 196,869.46 | 1.0813 | 39.6% | $5,184 | 0.8808 | 52.2% |
//! | [`Origin::CapacityGrowth`] | 98 | $359,768,207.40 | 268,216.80 | 1.6108 | 18.9% | $6,730 | 0.9239 | 41.6% |
//!
//! **They need opposite responses.** [`Origin::CapacityGrowth`] is preserved advantage — 90 of
//! the 98 were already on the guarantee in FY2026, their state share falls 6.6 points a year, and
//! the median one is **1.37 years** from the floor the 107 already sit on. It is the 107 caught
//! one year earlier, and retiring the instrument is the answer.
//!
//! [`Origin::EnrollmentLoss`] is the candidate blind spot. These are poor districts — 52.2%
//! economically disadvantaged — that have lost **11.9%** of their enrollment since the anchor
//! year, held at a median 1.0813. For **61** of them, $62,532,245.73, the per-pupil term is
//! *negative*: the FY2027 formula pays **more** for each child than the FY2020 regime did, and
//! nothing but the missing children puts them on the floor. What the formula fails to measure for
//! them is how fast a district's costs can follow its enrollment down — and the plan is
//! asymmetric about it on its face, paying $250 a pupil for a 3% **rise** and nothing for a 12%
//! fall.
//!
//! # What was tried and does not separate them
//!
//! The staffing-minimum cliff is real statewide and absent inside this population. ODE's own 2013
//! typology tops out at 130 of 187. Resident-versus-enrolled capacity moves a few points and not
//! the factor of two its own type documentation warns about. Each is asserted below rather than
//! set aside in prose.
//!
//! # And one thing that does reproduce after all
//!
//! The settled file records a prior session's quartile table as not reproducing "under any binning
//! tried" — equal-count on valuation, on state share, and equal-pupil on valuation. It reproduces
//! to the digit on a fourth: quartiles cut on [`DistrictRecord::published_capacity_per_pupil`]
//! over the whole panel, with the medians taken over **only the guaranteed districts** in each
//! band. See [`the_quartile_table_the_settled_file_could_not_reproduce`], which is why the
//! capacity measure is the axis this module reasons on.

use std::collections::BTreeMap;

use dispersion::{profile, sd1, typology};
use project::guarantee_origin::{
    above_the_minimum, clusters, decompose, enrollment_index, years_to_the_floor, Origin, ANCHOR,
    SEAM,
};
use project::hold_harmless;
use project::panel::{
    panel, DistrictRecord, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL, ENROLLMENT_GROWTH_THRESHOLD,
    MINIMUM_STATE_SHARE,
};
use project::prior_model;

/// The state share either side of which the two clusters sit, used only to measure how nearly
/// disjoint they are. Nothing in the module reads it.
const DISJOINT_AT: f64 = 0.25;

fn upper_middle(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn of(panel: &[DistrictRecord], origin: Origin) -> Vec<&DistrictRecord> {
    let index = enrollment_index(panel);
    above_the_minimum(panel)
        .into_iter()
        .filter(|r| decompose(r, &index).and_then(|s| s.origin()) == Some(origin))
        .collect()
}

/// **The population, stated on the predicates that define it.**
#[test]
fn the_population_is_the_exact_complement_of_the_one_the_minimum_share_explains() {
    let districts = panel();
    assert_eq!(districts.len(), 609, "the FY2027 panel");

    let guaranteed = districts.iter().filter(|r| r.on_guarantee()).count();
    let at_floor = districts
        .iter()
        .filter(|r| r.on_guarantee() && r.at_minimum_state_share())
        .count();
    assert_eq!(guaranteed, 294);
    assert_eq!(at_floor, 107, "the population already settled");

    let rest = above_the_minimum(&districts);
    assert_eq!(rest.len(), guaranteed - at_floor);
    assert_eq!(rest.len(), 187, "the population this file is about");

    let dollars: f64 = rest.iter().map(|r| r.guarantee).sum();
    assert!(
        (dollars - 468_337_128.87).abs() < 1.0,
        "the 187 hold {dollars}"
    );
    let adm: f64 = rest.iter().map(|r| r.current_year_adm).sum();
    assert!((adm - 465_086.26).abs() < 0.01, "and teach {adm}");

    // Why the any-device predicate is not the membership test: it is a wider population, and the
    // difference is districts this instrument does not pay at all.
    let held = districts.iter().filter(|r| hold_harmless::held(r)).count();
    assert_eq!(held, 326, "districts some hold-harmless device reaches");
    let without = districts
        .iter()
        .filter(|r| hold_harmless::held(r) && !r.on_guarantee())
        .count();
    assert_eq!(without, 32, "held by `[K]` or `[F]` and not by `[I]`");
    assert_eq!(held - without, guaranteed);
}

/// **The decomposition is an identity, not a fit.**
///
/// If this ever fails, every cluster below is a description of arithmetic error.
#[test]
fn the_two_terms_reconstruct_every_districts_multiple_exactly() {
    let districts = panel();
    let index = enrollment_index(&districts);
    let population = above_the_minimum(&districts);

    let mut checked = 0;
    for record in &population {
        let split = decompose(record, &index).expect("every one of the 187 is decomposable");
        assert_eq!(split.irn, record.irn, "keyed on IRN, never on name");
        assert!(
            (split.total() - split.multiple.ln()).abs() < 1e-12,
            "{} splits into {} + {} against ln({})",
            record.irn,
            split.enrollment_term,
            split.per_pupil_term,
            split.multiple
        );
        assert!(
            (split.enrollment_term - (1.0 / split.enrollment_index).ln()).abs() < 1e-12,
            "the enrollment term is the index and nothing else"
        );
        checked += 1;
    }
    assert_eq!(checked, 187, "and none of them is unreachable");

    // The chained index is a rate across a seam, per the module note.
    assert_eq!(ANCHOR, 2020, "the year `[H2]` holds a district at");
    assert_eq!(SEAM, 2024, "the last year the F-33 survey reaches");
    assert_eq!(
        index.len(),
        608,
        "the index reaches every district but one: {}",
        index.len()
    );
    let missing: Vec<&str> = districts
        .iter()
        .filter(|r| !index.contains_key(&r.irn))
        .map(|r| r.irn.as_str())
        .collect();
    assert_eq!(
        missing,
        vec!["047787"],
        "and the one it misses is named by IRN, because there are three Buckeye Locals"
    );
}

/// **The finding: the 187 are two populations, and the smaller half holds three quarters of the
/// money.**
#[test]
fn the_one_hundred_and_eighty_seven_split_into_lost_pupils_and_grown_capacity() {
    let districts = panel();
    let (clusters, unreached) = clusters(&districts);
    assert!(
        unreached.is_empty(),
        "the partition drops nobody: {unreached:?}"
    );
    assert_eq!(clusters.len(), 2);

    // (origin, districts, dollars, adm, multiple, state share, index, negative per-pupil term)
    let expected = [
        (
            Origin::EnrollmentLoss,
            89,
            108_568_921.47,
            196_869.46,
            1.081_276,
            0.395_975,
            0.880_838,
            61,
        ),
        (
            Origin::CapacityGrowth,
            98,
            359_768_207.40,
            268_216.80,
            1.610_754,
            0.188_965,
            0.923_888,
            0,
        ),
    ];
    for (cluster, want) in clusters.iter().zip(expected) {
        let (origin, n, dollars, adm, multiple, share, index, negative) = want;
        assert_eq!(cluster.origin, origin);
        assert_eq!(cluster.districts, n, "{} districts", cluster.origin.label());
        assert!(
            (cluster.dollars - dollars).abs() < 1.0,
            "{} holds {}",
            cluster.origin.label(),
            cluster.dollars
        );
        assert!((cluster.adm - adm).abs() < 0.01, "and {} ADM", cluster.adm);
        assert!(
            (cluster.median_multiple - multiple).abs() < 1e-5,
            "median multiple {}",
            cluster.median_multiple
        );
        assert!(
            (cluster.median_state_share - share).abs() < 1e-5,
            "median state share {}",
            cluster.median_state_share
        );
        assert!(
            (cluster.median_enrollment_index - index).abs() < 1e-5,
            "median index {}",
            cluster.median_enrollment_index
        );
        assert_eq!(
            cluster.formula_now_pays_more_per_pupil,
            negative,
            "{} with a negative per-pupil term",
            cluster.origin.label()
        );
    }

    assert_eq!(clusters[0].districts + clusters[1].districts, 187);
    let total: f64 = clusters.iter().map(|c| c.dollars).sum();
    assert!((total - 468_337_128.87).abs() < 1.0);

    // Half the districts, three quarters of the money, and it is the preserved-advantage half.
    let concentration = clusters[1].dollars / total;
    assert!(
        (concentration - 0.7682).abs() < 0.0005,
        "capacity growth holds {concentration} of the 187's guarantee"
    );
    assert!(clusters[1].origin.is_preserved_advantage());
    assert!(!clusters[0].origin.is_preserved_advantage());
}

/// **The threshold-free version of the blind-spot claim.**
///
/// 61 districts are held above the formula while the formula pays them *more* per pupil than the
/// FY2020 regime did. No cut was made to get this number; the per-pupil term is simply negative.
#[test]
fn sixty_one_districts_are_held_up_by_nothing_but_their_missing_children() {
    let districts = panel();
    let index = enrollment_index(&districts);

    let bare: Vec<&DistrictRecord> = above_the_minimum(&districts)
        .into_iter()
        .filter(|r| decompose(r, &index).is_some_and(|s| s.per_pupil_term <= 0.0))
        .collect();
    assert_eq!(bare.len(), 61);
    let dollars: f64 = bare.iter().map(|r| r.guarantee).sum();
    assert!((dollars - 62_532_245.73).abs() < 1.0, "they hold {dollars}");

    // Every one of them is in the enrollment cluster, which is what makes the majority rule and
    // the sign rule agree about the districts that matter most to the argument.
    for record in &bare {
        assert_eq!(
            decompose(record, &index).and_then(|s| s.origin()),
            Some(Origin::EnrollmentLoss),
            "{} has a negative per-pupil term and is not in the enrollment cluster",
            record.irn
        );
    }

    // And the plan's own asymmetry, which is the mechanism statement: it adjusts for growth.
    assert!((ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL - 250.0).abs() < f64::EPSILON);
    assert!((ENROLLMENT_GROWTH_THRESHOLD - 0.03).abs() < f64::EPSILON);
    let median_loss = 1.0
        - upper_middle(
            of(&districts, Origin::EnrollmentLoss)
                .iter()
                .map(|r| index[&r.irn])
                .collect(),
        );
    assert!(
        median_loss > 3.0 * ENROLLMENT_GROWTH_THRESHOLD,
        "the median enrollment cluster district has lost {median_loss}, against a {ENROLLMENT_GROWTH_THRESHOLD} threshold in the other direction"
    );
    assert!(
        (median_loss - 0.119_162).abs() < 1e-5,
        "the median loss is {median_loss}"
    );
}

/// **The clusters are nearly disjoint on the one axis the formula already computes.**
#[test]
fn the_two_clusters_sit_at_opposite_ends_of_the_state_share_distribution() {
    let districts = panel();
    let index = enrollment_index(&districts);
    let population = above_the_minimum(&districts);

    let loss = of(&districts, Origin::EnrollmentLoss);
    let growth = of(&districts, Origin::CapacityGrowth);
    let above = |rows: &[&DistrictRecord]| {
        rows.iter()
            .filter(|r| r.state_share_fraction() > DISJOINT_AT)
            .count()
    };
    assert_eq!(above(&loss), 83, "of the 89 enrollment-driven districts");
    assert_eq!(above(&growth), 21, "of the 98 capacity-driven districts");

    // A single cut on state share recovers the partition on 160 of 187 — which is not a
    // redundancy but the point: one instrument is doing two jobs at two ends of one axis.
    let agreeing = population
        .iter()
        .filter(|r| {
            let by_cut = if r.state_share_fraction() > DISJOINT_AT {
                Origin::EnrollmentLoss
            } else {
                Origin::CapacityGrowth
            };
            decompose(r, &index).and_then(|s| s.origin()) == Some(by_cut)
        })
        .count();
    assert_eq!(
        agreeing, 160,
        "the cut agrees with the decomposition {agreeing} times"
    );
    assert!(
        f64::from(u16::try_from(agreeing).expect("under 187")) / 187.0 > 0.85,
        "which is {agreeing} of 187"
    );
}

/// **The capacity cluster is the 107's own future, and the enrollment cluster is not.**
#[test]
fn the_capacity_cluster_reaches_the_floor_in_a_year_and_the_enrollment_cluster_in_five() {
    let districts = panel();
    let prior = prior_model::by_irn();

    let years = |rows: &[&DistrictRecord]| {
        upper_middle(
            rows.iter()
                .filter_map(|r| prior.get(&r.irn).and_then(|p| years_to_the_floor(r, p)))
                .collect(),
        )
    };
    let already = |rows: &[&DistrictRecord]| {
        rows.iter()
            .filter(|r| prior.get(&r.irn).is_some_and(|p| p.guarantee > 0.0))
            .count()
    };

    let loss = of(&districts, Origin::EnrollmentLoss);
    let growth = of(&districts, Origin::CapacityGrowth);

    let to_floor = years(&growth);
    assert!(
        (to_floor - 1.3738).abs() < 0.0005,
        "the capacity cluster is {to_floor} years from the minimum"
    );
    let slow = years(&loss);
    assert!(
        (slow - 5.1995).abs() < 0.0005,
        "the enrollment cluster is {slow} years from it"
    );
    assert!(slow > 3.0 * to_floor, "{slow} against {to_floor}");

    // And it is not a new arrival: it was already on the guarantee before the phase-in reached
    // 100%, which is what makes it the settled population caught a year earlier rather than a
    // consequence of this year's change.
    assert_eq!(already(&growth), 90, "of 98 already guaranteed in FY2026");
    assert_eq!(already(&loss), 39, "of 89");

    // Every district in the population is still moving toward the floor, so neither cluster is
    // there by standing still.
    for record in growth.iter().chain(loss.iter()) {
        let p = prior
            .get(&record.irn)
            .expect("the FY2026 model carries all 609");
        assert!(
            p.state_share_percentage > record.state_share_fraction(),
            "{} state share rose, {} to {}",
            record.irn,
            p.state_share_percentage,
            record.state_share_fraction()
        );
    }
    assert!((MINIMUM_STATE_SHARE - 0.10).abs() < f64::EPSILON);
}

/// **What does not separate them, asserted rather than waved away.**
///
/// Three candidate mechanisms, each cheap and each named in the brief for this work. None of them
/// orders the 187.
#[test]
fn size_typology_and_the_resident_count_do_not_separate_the_population() {
    let districts = panel();
    let index = enrollment_index(&districts);
    let population = above_the_minimum(&districts);

    // 1. The staffing-minimum cliff is real statewide...
    let mut by_size: Vec<&DistrictRecord> = districts.iter().collect();
    by_size.sort_by(|a, b| a.current_year_adm.total_cmp(&b.current_year_adm));
    let sextile = by_size.len() / 6;
    let mut rates = Vec::new();
    let mut base_cost = Vec::new();
    for s in 0..6 {
        let upper = if s == 5 {
            by_size.len()
        } else {
            (s + 1) * sextile
        };
        let slice = &by_size[s * sextile..upper];
        rates.push(slice.iter().filter(|r| r.on_guarantee()).count());
        base_cost.push(upper_middle(
            slice.iter().map(|r| r.base_cost_per_pupil).collect(),
        ));
    }
    assert_eq!(
        rates,
        vec![17, 41, 70, 61, 48, 57],
        "guaranteed per ADM sextile"
    );
    assert!(
        rates[2] > rates[0] && rates[2] > rates[5],
        "the rate peaks in the middle: {rates:?}"
    );
    assert!(
        base_cost[0] > base_cost[5] * 1.2,
        "and the smallest districts have the highest base cost per pupil: {base_cost:?}"
    );

    // ...and it does not order the 187. Multiple is flat across their own ADM quartiles.
    let mut by_adm: Vec<&&DistrictRecord> = population.iter().collect();
    by_adm.sort_by(|a, b| a.current_year_adm.total_cmp(&b.current_year_adm));
    let quarter = by_adm.len() / 4;
    let mut multiples = Vec::new();
    for q in 0..4 {
        let upper = if q == 3 {
            by_adm.len()
        } else {
            (q + 1) * quarter
        };
        let slice = &by_adm[q * quarter..upper];
        multiples.push(upper_middle(
            slice
                .iter()
                .map(|r| r.realized_aid() / r.core_foundation_funding)
                .collect(),
        ));
    }
    let spread = multiples.iter().copied().fold(f64::MIN, f64::max)
        / multiples.iter().copied().fold(f64::MAX, f64::min);
    assert!(
        spread < 1.07,
        "the multiple spans {spread} across the 187's ADM quartiles: {multiples:?}"
    );

    // 2. ODE's typology, given every advantage — each of its categories assigned to whichever
    // cluster is larger inside it — still misses a third of the population.
    let assignments = typology::by_irn();
    let mut table: BTreeMap<u8, [usize; 2]> = BTreeMap::new();
    for record in &population {
        let Some(split) = decompose(record, &index) else {
            continue;
        };
        let code = assignments
            .get(&record.irn)
            .map_or(u8::MAX, |t| t.typology.code());
        table.entry(code).or_default()
            [usize::from(split.origin() == Some(Origin::CapacityGrowth))] += 1;
    }
    let ceiling: usize = table.values().map(|counts| counts[0].max(counts[1])).sum();
    assert_eq!(
        ceiling, 130,
        "the typology's best possible accuracy is {ceiling} of 187"
    );
    assert_eq!(typology::VINTAGE, 2013, "and it was measured in");

    // 3. Resident against enrolled pupils: present, a few points, and not a factor of two — and
    // it does not order the two clusters apart from each other.
    let profiles = profile::districts();
    let enrolled: BTreeMap<&str, f64> = profiles
        .iter()
        .filter_map(|p| Some((p.irn.as_str(), p.enrolled_adm?)))
        .collect();
    let abstracts = sd1::by_district();
    let ratio = |record: &DistrictRecord| -> Option<f64> {
        let taught = *enrolled.get(record.irn.as_str())?;
        let resident = abstracts
            .get(&record.irn)?
            .iter()
            .filter(|row| row.tax_year == 2023)
            .find_map(|row| row.adm)?;
        (taught > 0.0).then(|| resident / taught)
    };
    let median_ratio =
        |rows: &[&DistrictRecord]| upper_middle(rows.iter().filter_map(|r| ratio(r)).collect());
    let loss = of(&districts, Origin::EnrollmentLoss);
    let growth = of(&districts, Origin::CapacityGrowth);
    let unpaid: Vec<&DistrictRecord> = districts.iter().filter(|r| !r.on_guarantee()).collect();
    let (a, b, c) = (
        median_ratio(&loss),
        median_ratio(&growth),
        median_ratio(&unpaid),
    );
    assert!((a - 1.095_130).abs() < 1e-5, "enrollment cluster {a}");
    assert!((b - 1.059_533).abs() < 1e-5, "capacity cluster {b}");
    assert!((c - 1.053_194).abs() < 1e-5, "not guaranteed {c}");
    assert!(
        a - c < 0.05,
        "the divergence is {} of a pupil, not the factor of two the type's docs warn about",
        a - c
    );
}

/// The partition survives the one measurement choice it depends on, and does not survive a
/// shorter span — both stated, because the second is what a reader who prefers the F-33 series
/// alone would get.
#[test]
fn the_split_moves_eight_districts_on_the_funded_count_and_twenty_nine_on_a_shorter_span() {
    let districts = panel();
    let published = enrollment_index(&districts);
    let population = above_the_minimum(&districts);

    // The first leg alone: FY2020 to FY2024, four years of a six-year interval.
    let survey: BTreeMap<String, f64> = {
        let rows = dispersion::ohio_panel::panel();
        let mut irn_of: BTreeMap<String, String> = BTreeMap::new();
        for row in rows.iter().filter(|r| r.comparable && !r.irn.is_empty()) {
            irn_of.insert(row.leaid.clone(), row.irn.clone());
        }
        let (mut first, mut last): (BTreeMap<String, f64>, BTreeMap<String, f64>) =
            (BTreeMap::new(), BTreeMap::new());
        for row in rows.iter().filter(|r| r.comparable && r.enrollment > 0.0) {
            if let Some(irn) = irn_of.get(&row.leaid) {
                if row.fiscal_year == ANCHOR {
                    first.insert(irn.clone(), row.enrollment);
                }
                if row.fiscal_year == SEAM {
                    last.insert(irn.clone(), row.enrollment);
                }
            }
        }
        first
            .into_iter()
            .filter_map(|(irn, start)| Some((irn.clone(), last.get(&irn)? / start)))
            .collect()
    };
    // The same chain ending at the count the formula actually funds on, rather than at FY2026.
    let funded: BTreeMap<String, f64> = districts
        .iter()
        .filter_map(|r| {
            let leg = *survey.get(&r.irn)?;
            Some((r.irn.clone(), leg * (r.base_cost_adm() / r.adm_history[0])))
        })
        .collect();

    let count = |index: &BTreeMap<String, f64>| {
        let moved = population
            .iter()
            .filter(|r| {
                decompose(r, index).and_then(|s| s.origin())
                    != decompose(r, &published).and_then(|s| s.origin())
            })
            .count();
        let loss = population
            .iter()
            .filter(|r| {
                decompose(r, index).and_then(|s| s.origin()) == Some(Origin::EnrollmentLoss)
            })
            .count();
        (loss, population.len() - loss, moved)
    };

    assert_eq!(count(&published), (89, 98, 0), "the published index");
    assert_eq!(count(&funded), (81, 106, 8), "ending at the funded count");
    assert_eq!(
        count(&survey),
        (60, 127, 29),
        "the survey's four years alone"
    );
}

/// The quartile table the settled file could not reproduce, on the axis it did not try.
///
/// `the_minimum_share_the_guarantee_has_already_paid_past.rs` tried equal-count on valuation,
/// equal-count on state share, and equal-pupil on valuation, and recorded that the figures
/// reproduce under none of them. They reproduce exactly on a fourth: quartiles cut on
/// [`DistrictRecord::published_capacity_per_pupil`] over the **whole 609**, medians taken over
/// **only the guaranteed districts** in each band.
///
/// The distinction that makes it work is the one that file's own note points at: the earlier
/// attempts took medians over every district in a quartile, where most districts are not on the
/// guarantee at all and the multiple is therefore 1.000x.
#[test]
fn the_quartile_table_the_settled_file_could_not_reproduce() {
    let districts = panel();
    let mut rows: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|r| r.published_capacity_per_pupil.is_some())
        .collect();
    assert_eq!(
        rows.len(),
        609,
        "the department publishes capacity for all of them"
    );
    rows.sort_by(|a, b| {
        a.published_capacity_per_pupil
            .expect("filtered")
            .total_cmp(&b.published_capacity_per_pupil.expect("filtered"))
    });

    // (guaranteed districts in the band, base cost per pupil, state share, multiple held)
    let expected = [
        (21, 8231.0, 0.589, 1.05),
        (47, 8231.0, 0.380, 1.12),
        (109, 8163.0, 0.212, 1.46),
        (117, 8136.0, 0.100, 2.36),
    ];
    // And over all 609, which is the population the "need is flat" claim rests on.
    let everyone = [8242.0, 8278.0, 8188.0, 8139.0];

    let quarter = rows.len() / 4;
    for (q, (want, flat)) in expected.into_iter().zip(everyone).enumerate() {
        let upper = if q == 3 {
            rows.len()
        } else {
            (q + 1) * quarter
        };
        let band = &rows[q * quarter..upper];
        let (n, base_cost, share, multiple) = want;

        let guaranteed: Vec<&&DistrictRecord> = band.iter().filter(|r| r.on_guarantee()).collect();
        assert_eq!(
            guaranteed.len(),
            n,
            "guaranteed in capacity quartile {}",
            q + 1
        );
        let got_base = upper_middle(guaranteed.iter().map(|r| r.base_cost_per_pupil).collect());
        assert!(
            (got_base - base_cost).abs() < 1.0,
            "Q{} base cost per pupil {got_base} against {base_cost}",
            q + 1
        );
        let got_share = upper_middle(
            guaranteed
                .iter()
                .map(|r| r.state_share_fraction())
                .collect(),
        );
        assert!(
            (got_share - share).abs() < 0.0005,
            "Q{} state share {got_share} against {share}",
            q + 1
        );
        let got_multiple = upper_middle(
            guaranteed
                .iter()
                .map(|r| r.realized_aid() / r.core_foundation_funding)
                .collect(),
        );
        assert!(
            (got_multiple - multiple).abs() < 0.005,
            "Q{} multiple {got_multiple} against {multiple}",
            q + 1
        );

        let over_all = upper_middle(band.iter().map(|r| r.base_cost_per_pupil).collect());
        assert!(
            (over_all - flat).abs() < 1.0,
            "Q{} base cost per pupil over all 609 is {over_all} against {flat}",
            q + 1
        );
    }

    // Flat over the whole panel, 1.3% across the capacity distribution — the "need is not the
    // problem" claim, on this axis as on the others.
    let spread = everyone[0] / everyone[3] - 1.0;
    assert!(spread < 0.02, "base cost per pupil spans {spread}");
}
