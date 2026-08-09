//! The payments that sit outside foundation funding, reproduced and measured.
//!
//! `[H] Foundation Funding` is base cost plus six categoricals, and the corpus has now decomposed
//! all of it. `[R] Total State Support` is larger, and the difference has been carried as a number
//! with a list of names attached: transportation, preschool special education, special education
//! transportation, the performance supplement. This file opens two of them.
//!
//! Both are structurally unlike anything inside the formula, and in opposite ways:
//!
//! - the **performance supplement** is the only component in Ohio's school funding that pays on a
//!   measured *outcome* rather than an input, and it is distributed **inversely to need**;
//! - the **enrollment growth supplement** is a **cliff**: 3% growth over three years pays $250 on
//!   every pupil, and 2.95% pays nothing.
//!
//! Neither is held by the guarantee, because the guarantee holds `[H]`. So a district can lose
//! either from one year to the next with nothing cushioning the fall — which is the practical
//! reason it matters that they are outside the formula rather than a taxonomic one.

use project::panel::{
    self, DistrictRecord, PerformanceSupplement, Supplements, BASE_FUNDING_SUPPLEMENT_PER_PUPIL,
    ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL, ENROLLMENT_GROWTH_THRESHOLD,
    PERFORMANCE_SUPPLEMENT_PER_POINT,
};

/// Dollar amounts are stored to the cent, so half a cent of rounding is admissible and nothing is.
fn close(computed: f64, published: f64) -> bool {
    (computed - published).abs() <= 0.01_f64.max(published.abs() * 1e-9)
}

// ---------------------------------------------------------------------------------------------
// The performance supplement
// ---------------------------------------------------------------------------------------------

/// $13 a pupil a rating point, on the greater of two ratings, for districts clearing any of three
/// tests.
#[test]
fn the_performance_supplement_reproduces_from_the_ratings_that_gate_it() {
    let panel = panel::panel();
    let mut paid = 0;
    let mut unpaid = 0;

    for record in &panel {
        let p = &record.performance;
        let adm = record.categorical_enrolled_adm;

        if !p.eligible {
            assert_eq!(
                p.amount, 0.0,
                "{}: not eligible and yet paid {:.2}",
                record.name, p.amount
            );
            unpaid += 1;
            continue;
        }
        let rating = p.paid_rating().expect("an eligible district has a rating");
        let expected = rating * PERFORMANCE_SUPPLEMENT_PER_POINT * adm;
        assert!(
            close(expected, p.amount),
            "{}: {rating} x $13 x {adm:.4} gives {expected:.2} against published {:.2}",
            record.name,
            p.amount
        );
        paid += 1;
    }

    assert!(paid > 400 && unpaid > 100, "paid {paid}, unpaid {unpaid}");
}

/// The three routes, and the fact that two of them do not require the district to be good.
///
/// A district qualifies on an overall rating above 3.5 stars, **or** a progress component rating
/// of 3 or more whatever its overall rating, **or** a progress rating merely higher than the year
/// before. The second and third routes carry a fifth of the recipients between them.
///
/// That is a defensible design — rewarding improvement rather than level is the standard answer to
/// the objection that outcome-based funding pays for intake. It does not do enough of the work,
/// which is what the distributional test below measures.
#[test]
fn two_of_the_three_qualifying_routes_do_not_require_a_good_rating() {
    let panel = panel::panel();
    let mut by_route: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for record in &panel {
        if let Some(route) = record.performance.route() {
            *by_route.entry(route).or_default() += 1;
        }
    }
    assert_eq!(by_route.len(), 3, "all three routes should be in use");

    let improvement_only = by_route
        .iter()
        .filter(|(route, _)| !route.contains("overall"))
        .map(|(_, n)| *n)
        .sum::<usize>();
    assert!(
        improvement_only > 100,
        "the two progress routes carry {improvement_only} districts"
    );

    // And a district can be paid while rated poorly: the amount is the greater of the two ratings,
    // so a low overall rating does not cap the payment if progress is higher.
    let paid_above_its_stars = panel
        .iter()
        .filter(|r| {
            let p = &r.performance;
            p.eligible && matches!((p.stars, p.progress), (Some(s), Some(g)) if g > s)
        })
        .count();
    assert!(
        paid_above_its_stars > 0,
        "expected districts paid on progress rather than on their overall rating"
    );
}

/// The finding: a component of an equalising formula, distributed inversely to need.
///
/// Sorted into quintiles by economically disadvantaged share, the mean performance supplement per
/// pupil falls monotonically from the least-poor quintile to the poorest, and the least-poor
/// receive more than twice per pupil what the poorest do.
///
/// **The confound is real and is not a defence.** Ohio's attainment measures track composition —
/// this corpus established that spending per *weighted* pupil against performance is substantially
/// a composition proxy — so any program keyed to those measures will follow composition whatever
/// its intent. That explains the gradient; it does not remove it. $55.7m of a formula built to
/// equalise flows against the grain of everything else in it, and nothing published says so.
///
/// The two progress routes exist precisely to blunt this, and the measurement here is what they
/// achieved: a gradient of 2.3 to one rather than whatever it would otherwise have been.
#[test]
fn the_performance_supplement_pays_least_where_need_is_greatest() {
    let panel = panel::panel();
    let mut sample: Vec<(f64, f64, bool)> = panel
        .iter()
        .filter_map(|r| {
            let adm = r.categorical_enrolled_adm;
            let poverty = r.dpia.percentage;
            (adm > 0.0 && poverty > 0.0)
                .then(|| (poverty, r.performance.amount / adm, r.performance.eligible))
        })
        .collect();
    assert!(sample.len() > 550, "expected the panel: {}", sample.len());
    sample.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("no NaN in the poverty share"));

    let size = sample.len() / 5;
    let quintile = |k: usize| -> (f64, f64) {
        let band = if k == 4 {
            &sample[4 * size..]
        } else {
            &sample[k * size..(k + 1) * size]
        };
        let per_pupil = band.iter().map(|b| b.1).sum::<f64>() / band.len() as f64;
        let qualifying = band.iter().filter(|b| b.2).count() as f64 / band.len() as f64;
        (per_pupil, qualifying)
    };

    let bands: Vec<(f64, f64)> = (0..5).map(quintile).collect();
    for k in 1..5 {
        assert!(
            bands[k].0 < bands[k - 1].0,
            "quintile {k} receives {:.2} against {:.2} for the one below it — the gradient should \
             be monotonic in poverty",
            bands[k].0,
            bands[k - 1].0
        );
    }

    let ratio = bands[0].0 / bands[4].0;
    assert!(
        ratio > 2.0,
        "the least-poor quintile receives {:.2} per pupil against {:.2} for the poorest, a ratio \
         of {ratio:.2}",
        bands[0].0,
        bands[4].0
    );
    assert!(
        bands[0].1 > 0.85 && bands[4].1 < 0.6,
        "qualification rates run {:.0}% to {:.0}%",
        bands[0].1 * 100.0,
        bands[4].1 * 100.0
    );
}

// ---------------------------------------------------------------------------------------------
// The base and enrollment growth supplements
// ---------------------------------------------------------------------------------------------

/// $40 a pupil, every district, no test of any kind.
#[test]
fn the_base_funding_supplement_is_unconditional() {
    let panel = panel::panel();
    for record in &panel {
        let expected = record.categorical_enrolled_adm * BASE_FUNDING_SUPPLEMENT_PER_PUPIL;
        assert!(
            close(expected, record.supplements.base_funding),
            "{}: {expected:.2} against published {:.2}",
            record.name,
            record.supplements.base_funding
        );
        assert!(
            record.supplements.base_funding > 0.0,
            "{}: every district draws this one",
            record.name
        );
    }
}

/// The growth supplement is a cliff, and it pays on the whole roll rather than the increment.
///
/// A district clearing 3% three-year growth receives $250 for **every** pupil, not for the pupils
/// it gained. That is what makes the threshold expensive rather than merely a threshold: the money
/// on either side of it is proportional to the district's size, not to how close it came.
#[test]
fn the_growth_supplement_is_a_cliff_paid_on_every_pupil() {
    let panel = panel::panel();
    let mut drawing = 0;

    for record in &panel {
        let s = &record.supplements;
        let adm = record.categorical_enrolled_adm;

        // The eligibility flag agrees with the arithmetic that produced it.
        let computed_change = if s.adm_fy23 > 0.0 {
            (adm - s.adm_fy23) / s.adm_fy23
        } else {
            0.0
        };
        assert!(
            (computed_change - s.enrollment_change).abs() < 1e-6,
            "{}: change computed {computed_change:.6} against published {:.6}",
            record.name,
            s.enrollment_change
        );
        assert_eq!(
            s.growth_eligible,
            s.enrollment_change >= ENROLLMENT_GROWTH_THRESHOLD,
            "{}: eligibility disagrees with the 3% test at {:.4}%",
            record.name,
            s.enrollment_change * 100.0
        );

        if s.growth_eligible {
            let expected = adm * ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL;
            assert!(
                close(expected, s.growth),
                "{}: {expected:.2} against published {:.2}",
                record.name,
                s.growth
            );
            drawing += 1;
        } else {
            assert_eq!(s.growth, 0.0, "{}: ineligible and yet paid", record.name);
        }
    }

    assert!(
        (20..80).contains(&drawing),
        "expected a few dozen growing districts, got {drawing}"
    );
}

/// What the cliff costs the district nearest it.
///
/// The median Ohio district is shrinking, so growth of any kind is unusual and the threshold sits
/// in a thin part of the distribution — which is exactly where a cliff does the most arbitrary
/// work, because the districts near it are few and their neighbours on either side are otherwise
/// alike.
#[test]
fn the_district_nearest_the_growth_cliff_forgoes_a_real_amount() {
    let panel = panel::panel();

    let mut just_below: Vec<&DistrictRecord> = panel
        .iter()
        .filter(|r| {
            let change = r.supplements.enrollment_change;
            change > 0.0 && change < ENROLLMENT_GROWTH_THRESHOLD
        })
        .collect();
    assert!(
        !just_below.is_empty(),
        "expected growing districts that missed the threshold"
    );
    just_below.sort_by(|a, b| {
        b.supplements
            .enrollment_change
            .partial_cmp(&a.supplements.enrollment_change)
            .expect("no NaN")
    });

    let nearest = just_below[0];
    let missed_by = ENROLLMENT_GROWTH_THRESHOLD - nearest.supplements.enrollment_change;
    let forgone = nearest
        .supplements
        .forgone(nearest.categorical_enrolled_adm)
        .expect("this district did not qualify");

    assert!(
        missed_by < 0.001,
        "{} missed by {:.4} percentage points",
        nearest.name,
        missed_by * 100.0
    );
    assert!(
        forgone > 100_000.0,
        "{} missed by {:.4} percentage points and forgoes {forgone:.0}",
        nearest.name,
        missed_by * 100.0
    );

    // And the median district is going the other way entirely, which is the context the cliff
    // sits in: this is not a threshold most districts are near.
    let mut changes: Vec<f64> = panel
        .iter()
        .map(|r| r.supplements.enrollment_change)
        .collect();
    changes.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let median = changes[changes.len() / 2];
    assert!(
        median < -0.02,
        "the median district's three-year change is {median:.4}, not a decline"
    );
}

/// The two supplements point in opposite directions, and the guarantee holds neither.
///
/// The guarantee pays a district whose enrolment has **fallen**; the growth supplement pays one
/// whose enrolment has **risen**. Both cushion movement, which means the formula responds to
/// enrolment change less than its per-pupil construction suggests — and a district can be drawing
/// both cushions is worth checking rather than assuming.
#[test]
fn the_guarantee_and_the_growth_supplement_reach_different_districts() {
    let panel = panel::panel();
    let both = panel
        .iter()
        .filter(|r| r.on_guarantee() && r.supplements.growth_eligible)
        .count();
    let guaranteed = panel.iter().filter(|r| r.on_guarantee()).count();
    let growing = panel
        .iter()
        .filter(|r| r.supplements.growth_eligible)
        .count();

    assert!(guaranteed > 200 && growing > 20);
    // A growing district can still be on the guarantee — the guarantee tracks aid, not enrolment,
    // and property wealth predicts it three times more strongly than decline does. So this is not
    // expected to be zero, and asserting that it were would be asserting something false.
    assert!(
        both < growing / 2,
        "{both} of {growing} growing districts are also on the guarantee, which is more overlap \
         than the two mechanisms should produce"
    );
}

// ---------------------------------------------------------------------------------------------
// Both, against the whole
// ---------------------------------------------------------------------------------------------

/// These two are real money and they are outside the guarantee's reach.
#[test]
fn the_supplements_are_a_material_share_of_what_sits_outside_foundation_funding() {
    let panel = panel::panel();
    let performance: f64 = panel.iter().map(|r| r.performance.amount).sum();
    let base: f64 = panel.iter().map(|r| r.supplements.base_funding).sum();
    let growth: f64 = panel.iter().map(|r| r.supplements.growth).sum();

    let outside: f64 = panel
        .iter()
        .map(|r| r.total_state_support - r.realized_aid())
        .sum();

    // The department publishes its own statewide total for the performance supplement, on a row
    // whose IRN is `050765` and whose name is `State of Ohio`. The panel reproduces it to three
    // cents across 609 districts, which is per-district rounding and nothing else.
    //
    // That row is why this test exists in the shape it does. The first pass at these figures
    // summed every row on the sheet and reported all three programs at **twice** their size — the
    // statewide row's IRN is all digits, so it looks like a district to any key-based filter. The
    // panel excludes it because `build_fy27_model` tests the name rather than the key, and this
    // assertion is what noticed that the prose did not.
    const PUBLISHED_PERFORMANCE_TOTAL: f64 = 55_676_980.28;
    assert!(
        (performance - PUBLISHED_PERFORMANCE_TOTAL).abs() < 1.0,
        "the panel sums to {performance:.2} against the department's own statewide row \
         {PUBLISHED_PERFORMANCE_TOTAL:.2}"
    );
    assert!(
        base > 50e6 && growth > 30e6,
        "base {base:.0}, growth {growth:.0}"
    );

    let covered = (performance + base + growth) / outside;
    assert!(
        covered > 0.1 && covered < 1.0,
        "the three supplements are {covered:.3} of what sits outside foundation funding; the \
         remainder is transportation and the two preschool and special education transportation \
         lines, which are not yet read"
    );
}

/// A district's supplements move with its rating and its enrolment, and neither is held.
#[test]
fn no_supplement_is_protected_by_the_guarantee() {
    // The guarantee holds `[H] Foundation Funding`. These sit in `[R] Total State Support`, so a
    // district that drops a star or slips below 3% growth loses the money outright. Stated here as
    // a structural check: the supplements must not be inside the figure the guarantee compares.
    let panel = panel::panel();
    for record in &panel {
        let inside = record.core_foundation_funding;
        let supplements =
            record.performance.amount + record.supplements.base_funding + record.supplements.growth;
        if supplements <= 0.0 {
            continue;
        }
        assert!(
            record.total_state_support >= inside + supplements - 1.0,
            "{}: total state support {:.2} is smaller than foundation funding plus supplements \
             {:.2}, so a supplement is being counted inside the formula",
            record.name,
            record.total_state_support,
            inside + supplements
        );
    }
}

/// Guards against a fixture regenerated with the columns shifted.
#[test]
fn the_supplement_structs_are_populated_rather_than_defaulted() {
    let panel = panel::panel();
    assert!(panel
        .iter()
        .any(|r| r.performance != PerformanceSupplement::default()));
    assert!(panel
        .iter()
        .any(|r| r.supplements != Supplements::default()));
    assert!(
        panel
            .iter()
            .filter(|r| r.performance.stars.is_none())
            .count()
            < 5,
        "only the one district rated N/A should be missing a star rating"
    );
}
