//! The floors R.C. 3317.011 puts under a small district's funded staffing, priced and traced.
//!
//! # The question, and the half of it that holds
//!
//! Issue #389 put the staffing minimums forward as the explanation of a shape
//! `the_other_half_of_the_guarantee_and_the_two_things_it_pays_for.rs` had measured and left
//! unexplained: the guarantee rate by ADM sextile is **non-monotone**, reaching a sixth of the
//! smallest districts and two thirds of the third sextile. The proposed mechanism was a cliff —
//! "a district below a minimum is funded as though it were larger; a district just above it is
//! not".
//!
//! **The floors are doing the job. The cliff does not exist.**
//!
//! Strike every floor out of the section and 46 more districts fall onto the guarantee, 294
//! becoming 340, of which 42 are in the smallest two ADM sextiles. Ordered on base cost enrolled
//! ADM — the count every threshold is against — the sextile rate goes
//!
//! | | 1 | 2 | 3 | 4 | 5 | 6 |
//! |---|--:|--:|--:|--:|--:|--:|
//! | as the section stands | **18** | 39 | 71 | 60 | 49 | 57 |
//! | with the floors struck out | **48** | 51 | 74 | 61 | 49 | 57 |
//!
//! The first-to-third gap is 53 districts with the floors and 26 without, so they account for
//! about **half** of the non-monotonicity. This file read the other half as "not size" and it is:
//! it is targeted assistance's capacity tier, which measures size outside base cost, and
//! `the_second_size_term_the_plan_keeps_outside_base_cost.rs` closes the gap to zero by striking
//! both out.
//!
//! # Why there is no cliff, and what there is instead
//!
//! Every floor is `max(ADM / ratio, n)`. A `max` agrees with its own argument at the crossing
//! point, so the funded count meets the bare ratio *exactly* at the threshold — asserted as a
//! property of the function in [`foundation::minimums`] rather than observed in data. The jump
//! in aggregate base cost at each of the five ADM thresholds is between the **13th and the 90th
//! percentile** of the jumps the department's two-decimal rounding of funded position counts
//! produces at ADM values with no threshold near them. The largest jump anywhere in a 13,514-step
//! sweep is $6,054.61 at 4,398.76 ADM, which is not a threshold.
//!
//! What the floors do instead is open a wedge between the **marginal** and the **average** pupil.
//! At each district's own enrolment and its own grade composition:
//!
//! | ADM sextile | median ADM | marginal pupil ÷ average pupil |
//! |---|--:|--:|
//! | 1 | 533 | **0.642** |
//! | 2 | 871 | 0.747 |
//! | 3 | 1,192 | 0.874 |
//! | 4 | 1,621 | 0.980 |
//! | 5 | 2,482 | 1.009 |
//! | 6 | 5,246 | 0.994 |
//!
//! Monotone to 1.0 and then flat: the wedge closes at about 2,000 ADM and stays closed. **313 of
//! 608 districts have a marginal pupil worth less than 95% of their average pupil**, and the most
//! extreme is worth 0.169 of it. A district does not fall off a cliff by growing; it is paid a
//! smaller and smaller share of its average for each additional child until it stops being paid a
//! smaller one.
//!
//! This is not the marginal price `what_the_formula_pays_a_district_to_do.rs` measured. That one
//! is a year's: R.C. 3317.017(B) multiplies a per-pupil residual by current-year enrolled ADM, so
//! this year's aid is linear in this year's count and $8,516.08 is right. The wedge arrives on
//! the other clock, phasing in over three years as the loss works through the three-year average
//! in [`DistrictRecord::base_cost_adm`] and the per-pupil amount itself moves — and it is only
//! the base cost side of that clock, because local capacity per pupil rises on the same
//! denominator and pushes the other way. Two clocks, one pupil.
//!
//! # Two things nobody had looked at
//!
//! **One floor in the section cannot bind.** R.C. 3317.011(F)(6)(c) funds
//! `max((max(ADM / 750, 2) + 1) / 3, 1)` leadership support staff. The inner `max` is never below
//! 2, so the quotient is never below 1, so the outer `max` is never operative — at any enrolment,
//! including zero. A floor made unreachable by the floor standing under its own input. Small
//! districts' leadership support *is* raised, by $6.1m, and that is the administrator floor at
//! (F)(3) flowing through, not this provision.
//!
//! **The section floors what a small district cannot obviously need and not what it obviously
//! cannot staff.** Five staffed elements have no floor at all: librarians, building leaders, and
//! the three classroom teacher bands. Kelleys Island Local, 5.2 pupils, is funded for **6.00
//! special teachers, 0.22 classroom teachers, 0.01 building leaders and 0.01 librarians** — 27
//! times more art, music and PE teachers than classroom teachers, and a hundredth of a principal.
//!
//! # And they are already a declining-enrolment adjustment
//!
//! A `max` against a falling quotient switches **on** as a district shrinks, so crossing a
//! threshold downward is the moment protection arrives rather than the moment it leaves. Since
//! the guarantee's FY2020 anchor, 457 of 608 districts have gained uplift this way, worth $21.1m.
//!
//! For `the_cluster_the_guarantee_is_still_recruiting.rs`'s 89 districts — the ones on the
//! guarantee for no reason but having fewer children — it is **$6,421,058, 5.91% of the
//! $108,568,921 the guarantee pays them**, a median $30.81 a pupil against the $464.06 a pupil
//! the guarantee pays. `the_adjustment_the_floors_absorb.rs` priced four candidate shapes for a
//! declining-enrolment adjustment against that cluster. R.C. 3317.011 already contains one. It
//! is in kind rather than in cash, it is capped by district size rather than by rate, and it is
//! worth a seventeenth of the instrument currently doing the job.
//!
//! # Where the thresholds came from
//!
//! **Nothing committed says.** R.C. 3317.011 states each floor as a bare integer inside the
//! division that applies it, with no recital. LSC's greenbook for H.B. 110 — the act that
//! enacted the section — restates all seven in its own parameter table ("Minimum of six special
//! teachers per district", "Minimum of one counselor funded", "a student to staff ratio of 750
//! to 1, with a minimum of two") and offers no derivation for any of them either. Both documents
//! are committed, at `crates/project/fixtures/revised-code.txt` and
//! `crates/project/fixtures/lsc-education-greenbooks.txt`, and the test below asserts that each
//! floor appears in both so that the negative result is checkable rather than asserted.
//!
//! The contrast worth recording is that the *Evidence-Based Model* did state one. Its school
//! nurse wellness coordinator factor was zero below 418 ADM — a size cut-off written the other
//! way round, excluding small districts rather than protecting them — and the same greenbook
//! series carries it. So the drafting tradition had both shapes available.
//!
//! # What is not established here
//!
//! Whether the floors are set in the right places. Nothing in this repository carries a
//! district's actual staffing, so "six special teachers" cannot be compared against six special
//! teachers. What can be said is what the floors cost, who they reach, and that their shape is a
//! decay rather than a cliff — and that the one genuine discontinuity in the whole section, the
//! athletics eligibility test at (A)(11), is dormant: all 609 districts are eligible.

use foundation::minimums::Minimum;
use foundation::{aggregate_base_cost, ratios, DistrictEnrollment, StatewideFactors};
use project::enrollment_decline;
use project::panel::{panel, DistrictRecord};
use project::staffing_minimums::{
    acquired_since_anchor, beside_a_threshold, ceilings, census, wedge, without_the_floors,
};

/// A cent, the tolerance every dollar figure here is asserted at.
const CENT: f64 = 0.01;

/// The crates' median: the upper middle, never the mean of two.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty());
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Sextiles of the panel ordered on base cost enrolled ADM, which is what the thresholds read.
fn sextiles<'a>(rows: &[&'a DistrictRecord]) -> Vec<Vec<&'a DistrictRecord>> {
    let mut ordered: Vec<&DistrictRecord> = rows.to_vec();
    ordered.sort_by(|a, b| a.base_cost_adm().total_cmp(&b.base_cost_adm()));
    let sixth = ordered.len() / 6;
    (0..6)
        .map(|s| {
            let end = if s == 5 {
                ordered.len()
            } else {
                (s + 1) * sixth
            };
            ordered[s * sixth..end].to_vec()
        })
        .collect()
}

/// **What the floors are worth, floor by floor.**
#[test]
fn the_floors_add_one_hundred_and_eighty_nine_million_and_the_ceilings_take_two_back() {
    let districts = panel();
    let rows = census(&districts);

    let expected = [
        (Minimum::SpecialTeachers, 166, 29_452_976.22, 0.0),
        (Minimum::Counselors, 278, 10_412_370.45, 0.0),
        (Minimum::Wellness, 276, 47_295_408.07, 0.0),
        (
            Minimum::OtherAdministrators,
            328,
            25_623_591.26,
            6_089_611.23,
        ),
        (Minimum::FiscalSupport, 372, 23_431_300.48, 0.0),
        (Minimum::Emis, 554, 31_123_598.71, 0.0),
        (Minimum::LeadershipSupport, 0, 0.0, 0.0),
        (Minimum::BuildingSupport, 291, 16_046_026.36, 0.0),
    ];
    for (row, (minimum, binds, direct, induced)) in rows.iter().zip(expected) {
        assert_eq!(row.minimum, minimum);
        assert_eq!(
            row.binds,
            binds,
            "{} binds for {}",
            minimum.label(),
            row.binds
        );
        assert!(
            (row.direct - direct).abs() < CENT,
            "{} is worth {:.2}, not {direct:.2}",
            minimum.label(),
            row.direct
        );
        assert!((row.induced - induced).abs() < CENT);
    }

    // The EMIS floor is not a small-district provision at all: it reaches nine districts in ten.
    assert!(
        rows[5].binds * 10 > districts.len() * 9,
        "the EMIS floor binds for {} of {}",
        rows[5].binds,
        districts.len()
    );

    let floors: f64 = rows
        .iter()
        .map(project::staffing_minimums::Census::total)
        .sum();
    assert!(
        (floors - 189_474_882.77).abs() < CENT,
        "floors total {floors:.2}"
    );

    let taken: f64 = ceilings(&districts).iter().map(|c| c.2).sum();
    assert!(
        (taken - 2_328_807.95).abs() < CENT,
        "ceilings take {taken:.2}"
    );
    assert_eq!(
        ceilings(&districts).iter().map(|c| c.1).collect::<Vec<_>>(),
        vec![3, 2],
        "districts each ceiling reaches"
    );
    assert!((floors - taken - 187_146_074.83).abs() < CENT);
}

/// **The floor that cannot bind**, asserted over the panel as well as over the function.
///
/// `foundation::minimums` proves it for any enrolment; this says the 609 real districts agree,
/// so a reader who distrusts the algebra can check the census instead.
#[test]
fn one_floor_in_the_section_reaches_nobody_and_it_is_not_dead_weight() {
    let districts = panel();
    let rows = census(&districts);
    let dead: Vec<&_> = rows.iter().filter(|r| r.binds == 0).collect();
    assert_eq!(dead.len(), 1, "exactly one floor should reach nobody");
    assert_eq!(dead[0].minimum, Minimum::LeadershipSupport);
    assert!(!Minimum::LeadershipSupport.can_bind());

    // But the division it sits in *is* raised for small districts — by the administrator floor
    // above it, which R.C. 3317.011(F)(6)(a) reads as its input. Attributing that to this
    // provision would credit $6.1m to a sentence with no force.
    let induced = rows
        .iter()
        .find(|r| r.minimum == Minimum::OtherAdministrators)
        .expect("the administrator floor is in the census")
        .induced;
    assert!((induced - 6_089_611.23).abs() < CENT);
    assert!(rows.iter().filter(|r| r.induced != 0.0).count() == 1);
}

/// **The counterfactual.** Strike the floors out and the smallest sextile joins the guarantee.
#[test]
fn striking_the_floors_out_puts_forty_six_more_districts_on_the_guarantee() {
    let districts = panel();
    let rows = without_the_floors(&districts);
    assert_eq!(
        rows.len(),
        609,
        "every district has a per-pupil quantity to move"
    );

    let uplift: f64 = rows.iter().map(|r| r.uplift).sum();
    let delta: f64 = rows.iter().map(|r| r.aid_delta).sum();
    assert!((uplift - 187_146_074.83).abs() < CENT);
    assert!(
        (delta + 156_371_577.58).abs() < CENT,
        "aid delta {delta:.2}"
    );

    // 83.6% of the base cost, not the ~45% a statewide average state share would give. For a
    // district off the minimum share, base cost aid is `base cost − capacity` and capacity does
    // not move with base cost, so the state pays the *whole* of the uplift.
    assert!(
        (-delta / uplift - 0.8356).abs() < 0.0005,
        "the state pays {:.4} of the uplift",
        -delta / uplift
    );

    assert_eq!(rows.iter().filter(|r| r.guaranteed).count(), 294);
    assert_eq!(rows.iter().filter(|r| r.guaranteed_without).count(), 340);
    assert_eq!(rows.iter().filter(|r| r.newly_guaranteed()).count(), 46);
    assert_eq!(
        rows.iter()
            .filter(|r| r.guaranteed && !r.guaranteed_without)
            .count(),
        0,
        "removing a floor cannot lift a district off the guarantee"
    );

    let by_irn: std::collections::BTreeMap<&str, &_> =
        rows.iter().map(|r| (r.irn.as_str(), r)).collect();
    let bands = sextiles(&districts.iter().collect::<Vec<_>>());
    let count = |f: fn(&project::staffing_minimums::WithoutTheFloors) -> bool| {
        bands
            .iter()
            .map(|band| {
                band.iter()
                    .filter(|d| by_irn.get(d.irn.as_str()).is_some_and(|r| f(r)))
                    .count()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(count(|r| r.guaranteed), vec![18, 39, 71, 60, 49, 57]);
    assert_eq!(
        count(|r| r.guaranteed_without),
        vec![48, 51, 74, 61, 49, 57]
    );
    assert_eq!(
        count(project::staffing_minimums::WithoutTheFloors::newly_guaranteed),
        vec![30, 12, 3, 1, 0, 0]
    );

    // Which is half of the shape the issue asked about, stated as the comparison it asked for.
    let with = count(|r| r.guaranteed);
    let without = count(|r| r.guaranteed_without);
    assert_eq!(with[2] - with[0], 53);
    assert_eq!(without[2] - without[0], 26);
}

/// **No cliff.** Every threshold's jump sits inside the department's own rounding grain.
///
/// The sweep holds grade composition and puts the building count at exactly `ADM / 400`, so the
/// building support floor and the three-per-building ceiling are both inert and what is left is
/// the staffing floors alone.
#[test]
fn no_threshold_moves_base_cost_by_more_than_the_departments_own_rounding() {
    let factors = StatewideFactors::fy2027();
    let districts = panel();
    let seed = districts
        .iter()
        .find(|d| d.name.contains("Hardin Northern"))
        .expect("the seed district is in the panel")
        .enrollment;
    let at = |adm: f64| {
        aggregate_base_cost(
            &DistrictEnrollment {
                open_buildings: adm / ratios::BUILDING_LEADERSHIP_SUPPORT,
                ..seed.scaled_to(adm)
            },
            &factors,
        )
        .aggregate
    };
    let jump = |adm: f64| (at(adm + 0.01) - at(adm - 0.01)).abs();

    // The grain: the same measurement at 13,514 enrolments, stepped by an amount with no
    // relation to any threshold so that the sample is not aligned to one.
    let mut grain: Vec<(f64, f64)> = Vec::new();
    let mut adm = 200.0;
    while adm < 5_200.0 {
        grain.push((adm, jump(adm)));
        adm += 0.37;
    }
    assert_eq!(grain.len(), 13_514);
    let mut sorted: Vec<f64> = grain.iter().map(|g| g.1).collect();
    sorted.sort_by(f64::total_cmp);
    let percentile = |value: f64| {
        100.0 * sorted.iter().filter(|x| **x < value).count() as f64 / sorted.len() as f64
    };

    for minimum in Minimum::ALL {
        let Some(threshold) = minimum.threshold() else {
            continue;
        };
        let at_threshold = percentile(jump(threshold));
        assert!(
            (1.0..=90.0).contains(&at_threshold),
            "{} at {threshold} ADM jumps ${:.2}, the {at_threshold:.1}th percentile of the grain",
            minimum.label(),
            jump(threshold)
        );
    }

    // And the largest jump in the sweep is nowhere near a threshold, which is the check that
    // makes the percentiles mean something.
    let worst = grain
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("the sweep is not empty");
    assert!((worst.0 - 4_398.76).abs() < 0.01 && (worst.1 - 6_054.61).abs() < CENT);
    for minimum in Minimum::ALL {
        if let Some(threshold) = minimum.threshold() {
            assert!((worst.0 - threshold).abs() > 100.0);
        }
    }
}

/// **The wedge.** What the floors actually do is make the marginal pupil worth less than the
/// average one, by an amount that closes as the district grows.
#[test]
fn the_floors_open_a_wedge_between_the_marginal_and_the_average_pupil() {
    let factors = StatewideFactors::fy2027();
    let districts = panel();
    let measurable: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|d| d.base_cost_adm() > project::staffing_minimums::HALF_WINDOW)
        .collect();
    assert_eq!(
        measurable.len(),
        608,
        "only Kelleys Island is too small to difference"
    );

    let ratios: Vec<f64> = sextiles(&measurable)
        .iter()
        .map(|band| {
            upper_middle(
                band.iter()
                    .map(|d| wedge(&d.enrollment, d.base_cost_adm(), &factors).ratio())
                    .collect(),
            )
        })
        .collect();
    let expected = [0.6418, 0.7465, 0.8741, 0.9795, 1.0088, 0.9939];
    for (got, want) in ratios.iter().zip(expected) {
        assert!((got - want).abs() < 0.0005, "wedge ratios were {ratios:?}");
    }
    for window in ratios[..5].windows(2) {
        assert!(
            window[1] > window[0],
            "the wedge does not close monotonically: {ratios:?}"
        );
    }

    // It closes for good rather than reversing: the last two sextiles are at parity, so nothing
    // above about 2,000 ADM is paying for a floor.
    assert!((ratios[4] - 1.0).abs() < 0.01 && (ratios[5] - 1.0).abs() < 0.01);

    let pinched = measurable
        .iter()
        .filter(|d| wedge(&d.enrollment, d.base_cost_adm(), &factors).ratio() < 0.95)
        .count();
    assert_eq!(
        pinched, 313,
        "districts whose marginal pupil is below 95% of their average"
    );
}

/// **The floors are a declining-enrolment adjustment**, and the corpus priced four others
/// without noticing.
#[test]
fn shrinking_past_a_threshold_switches_a_floor_on_and_it_is_worth_a_seventeenth_of_the_guarantee() {
    let districts = panel();
    let acquired = acquired_since_anchor(&districts);
    assert_eq!(
        acquired.len(),
        608,
        "one district the enrollment index cannot reach"
    );

    let statewide: f64 = acquired.values().sum();
    assert!(
        (statewide - 21_076_864.69).abs() < CENT,
        "statewide {statewide:.2}"
    );
    assert_eq!(acquired.values().filter(|v| **v > 1.0).count(), 457);

    let cluster = enrollment_decline::cluster(&districts);
    assert_eq!(cluster.len(), 89);
    let gained: f64 = cluster.iter().filter_map(|d| acquired.get(&d.irn)).sum();
    let guarantee: f64 = cluster.iter().map(|d| d.guarantee).sum();
    assert!(
        (gained - 6_421_058.36).abs() < CENT,
        "the cluster gained {gained:.2}"
    );
    assert!((guarantee - 108_568_921.47).abs() < CENT);
    assert!(
        (gained / guarantee - 0.0591).abs() < 0.0005,
        "which is {:.4} of what the guarantee pays them",
        gained / guarantee
    );

    let per_pupil = upper_middle(
        cluster
            .iter()
            .filter_map(|d| acquired.get(&d.irn).map(|g| g / d.base_cost_adm()))
            .collect(),
    );
    assert!(
        (per_pupil - 30.81).abs() < 0.005,
        "median {per_pupil:.2} a pupil"
    );

    // The direction is the opposite of the one the issue expected, and that is the whole point:
    // a district shrinking through a threshold gains protection rather than losing it.
    assert!(
        acquired.values().filter(|v| **v < -1.0).count()
            < acquired.values().filter(|v| **v > 1.0).count(),
        "more districts should have gained uplift than lost it"
    );
}

/// **No bunching**, which is what a threshold nobody can manipulate should look like.
#[test]
fn what_sits_beside_a_threshold_is_the_enrolment_distribution_and_nothing_else() {
    let districts = panel();
    let beside = beside_a_threshold(&districts);
    assert_eq!(beside.len(), 5, "five floors have an ADM threshold");
    assert_eq!(
        beside.iter().map(|b| (b.1, b.2)).collect::<Vec<_>>(),
        vec![(19, 14), (23, 11), (23, 14), (21, 15), (5, 4)],
        "districts within 5% below and above each threshold"
    );

    // More below than above at all five — and so is the enrolment distribution generally. The
    // comparison that makes that a non-result is 148 control points with no threshold within
    // 10%, at which the same ratio has a median of 1.20 and 84 of them also lean below.
    let count = |lo: f64, hi: f64| {
        districts
            .iter()
            .filter(|d| d.base_cost_adm() >= lo && d.base_cost_adm() < hi)
            .count()
    };
    let thresholds: Vec<f64> = Minimum::ALL.iter().filter_map(|m| m.threshold()).collect();
    let mut controls: Vec<f64> = Vec::new();
    let mut centre = 400.0;
    while centre <= 6_000.0 {
        let near = thresholds
            .iter()
            .any(|t| (centre / t - 1.0f64).abs() <= 0.10);
        let (below, above) = (count(centre * 0.95, centre), count(centre, centre * 1.05));
        if !near && below + above >= 5 {
            controls.push(below as f64 / (above as f64).max(0.5));
        }
        centre += 25.0;
    }
    assert_eq!(controls.len(), 148);
    controls.sort_by(f64::total_cmp);
    assert!((upper_middle(controls.clone()) - 1.20).abs() < 0.005);

    for (minimum, below, above) in beside {
        let ratio = below as f64 / (above as f64).max(0.5);
        let percentile =
            100.0 * controls.iter().filter(|c| **c < ratio).count() as f64 / controls.len() as f64;
        assert!(
            (40.0..=85.0).contains(&percentile),
            "{} has {below} below and {above} above, the {percentile:.0}th percentile of the \
             controls — which would be a pile-up",
            minimum.label()
        );
    }
}

/// **What has no floor**, which is as much of the design as what does.
#[test]
fn the_two_positions_a_tiny_district_cannot_staff_in_fractions_have_no_floor_at_all() {
    let districts = panel();
    let smallest = districts
        .iter()
        .min_by(|a, b| a.base_cost_adm().total_cmp(&b.base_cost_adm()))
        .expect("the panel is not empty");
    assert_eq!(smallest.name, "Kelleys Island Local");
    assert!((smallest.base_cost_adm() - 5.2).abs() < 0.05);

    let funded = |count: f64, ratio: f64| edfund_core::round_dp(count / ratio, 2);
    let adm = smallest.base_cost_adm();
    assert!(
        (Minimum::SpecialTeachers.funded(&smallest.enrollment) - 6.0).abs() < CENT,
        "six special teachers for five pupils"
    );
    assert!((funded(adm, ratios::BUILDING_LEADER) - 0.01).abs() < 1e-9);
    assert!((funded(adm, ratios::LIBRARIAN) - 0.01).abs() < 1e-9);

    // (D)(1)(f) sums the five quotients and the department rounds the sum, which is a cent of
    // difference here and the reason the figure is 0.22 rather than 0.21.
    let classroom = edfund_core::round_dp(
        smallest.enrollment.kindergarten / ratios::KINDERGARTEN
            + smallest.enrollment.grades_1_3 / ratios::GRADES_1_3
            + smallest.enrollment.grades_4_8 / ratios::GRADES_4_8
            + smallest.enrollment.grades_9_12 / ratios::GRADES_9_12
            + smallest.enrollment.career_technical / ratios::CAREER_TECHNICAL,
        2,
    );
    assert!(
        (classroom - 0.22).abs() < 0.005,
        "and {classroom:.2} classroom teachers, against six special ones"
    );

    // The one genuine discontinuity in R.C. 3317.011 is (A)(11)'s athletics eligibility test,
    // which is a step and not a ramp — and it is dormant. If a district ever becomes ineligible
    // this fails, and the claim above stops being true.
    assert!(
        districts.iter().all(|d| d.enrollment.athletics_eligible),
        "every district is an eligible school district, so the only step never fires"
    );
}

/// **Where the numbers came from**, asserted as the positive half of a negative result.
///
/// Both documents state every floor. Neither derives one. The test can only establish the first
/// half; a reader who wants the second has the line numbers.
#[test]
fn both_committed_sources_state_the_floors_and_neither_derives_them() {
    let statute = include_str!("../fixtures/revised-code.txt");
    let greenbooks = include_str!("../fixtures/lsc-education-greenbooks.txt");

    // The statute, as bare integers inside the division that applies each one.
    for phrase in [
        "is greater than 6, the special teacher cost",
        "division (E)(1)(a) of this section and 1)",
        "division (E)(3)(a) of this section and 5)",
        "division (F)(3)(b) of this section and 2)",
        "The maximum of the quotient obtained under division (F)(4)(a) of this section and 2",
        "division (F)(5)(a) of this section and 1)",
        "division (F)(6)(b) of this section and 1)",
    ] {
        assert!(
            statute.contains(phrase),
            "R.C. 3317.011 no longer says {phrase:?}"
        );
    }

    // LSC's greenbook for the act that enacted it, restating them in a parameter table.
    for phrase in [
        "Minimum of six special teachers per district",
        "Minimum of one counselor funded",
        "Student to staff ratio of 250 to 1. Minimum of five staff",
        "a student to staff ratio of 750 to 1, with a minimum of two",
        "5,000 to 1 with a minimum of one position funded",
    ] {
        assert!(
            greenbooks.contains(phrase),
            "the H.B. 110 greenbook no longer says {phrase:?}"
        );
    }

    // And the contrast: the Evidence-Based Model wrote a size cut-off the other way round.
    assert!(
        greenbooks
            .contains("If district ADM < 418, then school nurse wellness coordinator factor = 0"),
        "the EBM's own size threshold is what makes the drafting choice a choice"
    );
}
