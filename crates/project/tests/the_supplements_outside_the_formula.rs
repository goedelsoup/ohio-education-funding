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

mod common;

use project::panel::{
    self, DistrictRecord, PerformanceSupplement, Supplements, AVERAGE_BASE_COST_PER_PUPIL,
    BASE_FUNDING_SUPPLEMENT_PER_PUPIL, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL,
    ENROLLMENT_GROWTH_THRESHOLD, OPEN_ENROLLMENT_CLAWBACK_PER_FTE, OPEN_ENROLLMENT_THRESHOLD_FLOOR,
    OPEN_ENROLLMENT_THRESHOLD_FRACTION, PERFORMANCE_SUPPLEMENT_PER_POINT, PREK_SPED_APPROPRIATION,
    PREK_SPED_FLAT_PER_PUPIL, PREK_SPED_PRORATION, PREK_SPED_WEIGHT_FRACTION,
    SPECIAL_EDUCATION_WEIGHTS,
};

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
            common::close(expected, p.amount),
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
/// achieved: a gradient of 2.56 to one rather than whatever it would otherwise have been.
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
            common::close(expected, record.supplements.base_funding),
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
                common::close(expected, s.growth),
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

// ---------------------------------------------------------------------------------------------
// Preschool special education
// ---------------------------------------------------------------------------------------------

/// A flat amount and a half-weight, which nothing else in the formula combines.
///
/// `(ADM x $4,000) + (ADM x weight x average base cost x state share x 0.5)`, prorated. The flat
/// part is 69% of the program and is **not** reduced by the state share, so for most of what this
/// pays the wealthiest district and the poorest are funded identically.
#[test]
fn preschool_special_education_is_a_flat_grant_plus_half_a_weight() {
    let panel = panel::panel();
    let mut checked = 0;

    for record in &panel {
        let Some(share) = record.published_state_share else {
            continue;
        };
        let p = &record.preschool_special_education;
        for (k, weight) in SPECIAL_EDUCATION_WEIGHTS.iter().enumerate() {
            let flat = p.adm[k] * PREK_SPED_FLAT_PER_PUPIL;
            let weighted =
                p.adm[k] * weight * AVERAGE_BASE_COST_PER_PUPIL * share * PREK_SPED_WEIGHT_FRACTION;
            let expected = (flat + weighted) * PREK_SPED_PRORATION;
            assert!(
                common::close(expected, p.aid[k]),
                "{}: preschool category {} computed {expected:.2} against published {:.2}",
                record.name,
                k + 1,
                p.aid[k]
            );
        }
        assert!(
            (p.aid.iter().sum::<f64>() - p.total).abs() < 0.04,
            "{}: the six sum to {:.2} against published total {:.2}",
            record.name,
            p.aid.iter().sum::<f64>(),
            p.total
        );
        checked += 1;
    }
    assert!(checked > 600, "expected the whole panel: {checked}");

    // The flat component dominates, and it is the part the state share does not touch.
    let flat: f64 = panel
        .iter()
        .map(|r| r.preschool_special_education.flat_component())
        .sum();
    let total: f64 = panel
        .iter()
        .map(|r| r.preschool_special_education.total)
        .sum();
    assert!(
        flat / total > 0.6,
        "the flat component is {:.3} of the program",
        flat / total
    );
}

/// Halving the weights against a flat floor compresses the program almost to parity.
///
/// School-age special education is famously top-heavy: Category 6 is 15% of the pupils and 48% of
/// the money. Here the same six weights apply at half, on top of a $4,000 flat amount that is the
/// same for every category — and the result is nearly flat. Category 6's 5,761 pupils draw about
/// what Category 2's 10,842 do.
///
/// So the same weight vector produces a steeply graduated program in one place and an almost
/// undifferentiated one in another, purely because of what sits beside it. A table of Ohio's
/// special education weights describes neither on its own.
#[test]
fn the_same_weights_are_nearly_flat_here_and_steeply_graduated_at_school_age() {
    let panel = panel::panel();
    let share_of = |pick: fn(&panel::PreschoolSpecialEducation, usize) -> f64, k: usize| {
        let part: f64 = panel
            .iter()
            .map(|r| pick(&r.preschool_special_education, k))
            .sum();
        let all: f64 = (0..6)
            .map(|j| {
                panel
                    .iter()
                    .map(|r| pick(&r.preschool_special_education, j))
                    .sum::<f64>()
            })
            .sum();
        part / all
    };
    let money_six = share_of(|p, k| p.aid[k], 5);
    let pupils_six = share_of(|p, k| p.adm[k], 5);

    // At school age, Category 6 is roughly three times its pupil share in money. Here it is close
    // to one, which is the compression the flat grant and the halved weight produce together.
    let concentration = money_six / pupils_six;
    assert!(
        concentration < 2.0,
        "Category 6 takes {money_six:.3} of the money on {pupils_six:.3} of the pupils, a \
         concentration of {concentration:.2} — the flat grant should be flattening this"
    );

    let school_age_money: f64 = panel
        .iter()
        .map(|r| r.special_education.aid[5])
        .sum::<f64>()
        / panel
            .iter()
            .map(|r| r.special_education.total())
            .sum::<f64>();
    let school_age_pupils: f64 = panel
        .iter()
        .map(|r| r.special_education.adm[5])
        .sum::<f64>()
        / panel
            .iter()
            .map(|r| r.special_education.total_adm())
            .sum::<f64>();
    assert!(
        school_age_money / school_age_pupils > concentration * 1.4,
        "school age concentrates at {:.2} against preschool's {concentration:.2}; the same \
         weights should behave very differently in the two programs",
        school_age_money / school_age_pupils
    );
}

/// The proration no longer fits the appropriation it was set against.
///
/// This sheet is the clearest statement in the workbook of what a proration is, because it carries
/// the **appropriation limit** in a cell beside the factor: a budget divided by an entitlement.
///
/// And the division has stopped working. At the stated 0.96854448 the program totals $148,408,184
/// against a $147,500,000 limit — **$908,184 over**. A third cell on the same sheet states
/// $146,708,228.07, which matches neither.
///
/// The likeliest reading is that the factor was calibrated against an earlier ADM vintage and the
/// counts were refreshed without recalibrating. This is a projection published before the fiscal
/// year, so a recalibration before payment is expected. The point of asserting it is that the
/// corpus should notice when a published workbook is internally inconsistent, rather than
/// reproducing the inconsistency silently and calling it verified.
#[test]
fn the_preschool_proration_no_longer_brings_the_program_within_its_appropriation() {
    let panel = panel::panel();
    let total: f64 = panel
        .iter()
        .map(|r| r.preschool_special_education.total)
        .sum();
    let entitlement: f64 = panel
        .iter()
        .map(|r| r.preschool_special_education.unprorated())
        .sum();

    assert!(
        total > PREK_SPED_APPROPRIATION,
        "the program totals {total:.2} against an appropriation of {PREK_SPED_APPROPRIATION:.2}; \
         if this has stopped being true the department has recalibrated and the node describing \
         it needs revisiting"
    );
    let over = total - PREK_SPED_APPROPRIATION;
    assert!(
        (500_000.0..2_000_000.0).contains(&over),
        "over the appropriation by {over:.2}, which is a different size from what was recorded"
    );

    // The factor that would have fitted, which is what a recalibration would produce.
    let fitting = PREK_SPED_APPROPRIATION / entitlement;
    assert!(
        fitting < PREK_SPED_PRORATION,
        "a fitting factor of {fitting:.8} should be below the stated {PREK_SPED_PRORATION:.8}"
    );
}

/// With this read, the gap between foundation funding and total state support is accounted for.
#[test]
fn nothing_material_is_left_unexplained_outside_foundation_funding() {
    let panel = panel::panel();
    let outside: f64 = panel
        .iter()
        .map(|r| r.total_state_support - r.realized_aid())
        .sum();
    let explained: f64 = panel
        .iter()
        .map(|r| {
            r.transportation.total
                + r.transportation.special_education
                + r.preschool_special_education.total
                + r.performance.amount
                + r.supplements.base_funding
                + r.supplements.growth
        })
        .sum();

    let share = explained / outside;
    assert!(
        share > 0.95,
        "{share:.4} of what sits outside foundation funding is now named; the remainder is \
         {:.0} and has no component behind it yet",
        outside - explained
    );
    assert!(
        share < 1.02,
        "{share:.4} — more has been attributed than sits outside the formula, which means \
         something here is also inside it"
    );
}

// ---------------------------------------------------------------------------------------------
// The guarantee's machinery, and the residual it turned out to be
// ---------------------------------------------------------------------------------------------

/// The $63.6m residual has a name: `[K] Formula Transition Supplement`.
///
/// It is a **second** hold-harmless stacked on the guarantee, against a larger base:
/// `max(FY21 funding base − (foundation funding + guarantee + supplemental targeted assistance +
/// transportation), 0)`.
///
/// The `[H2]` base the guarantee compares against covers foundation funding. This `[L1]` base
/// covers foundation funding *plus transportation*, so a district whose transportation has fallen
/// can draw this while drawing nothing from the guarantee — and **17 of the 144 districts on it
/// are not on the guarantee at all**.
#[test]
fn the_transition_supplement_is_a_second_hold_harmless_on_a_larger_base() {
    let panel = panel::panel();
    let mut drawing = 0;
    let mut without_the_guarantee = 0;

    for record in &panel {
        let t = &record.transition;
        let computed = t.fy21_funding_base
            - (record.core_foundation_funding
                + record.guarantee
                + record.categoricals.targeted_assistance * 0.0
                + record.transportation.total);
        // Supplemental targeted assistance is zero for every district, so it drops out of the
        // reproduction — but it is in the department's formula and is named here so that a future
        // biennium funding it does not silently break this.
        let expected = computed.max(0.0);
        assert!(
            (expected - t.transition_supplement).abs() < 0.05,
            "{}: computed {expected:.2} against published {:.2}",
            record.name,
            t.transition_supplement
        );
        if t.transition_supplement > 0.0 {
            drawing += 1;
            if record.guarantee <= 0.0 {
                without_the_guarantee += 1;
            }
        }
    }

    assert!(
        (100..200).contains(&drawing),
        "{drawing} districts draw the transition supplement"
    );
    assert!(
        without_the_guarantee > 0,
        "the supplement should reach districts the guarantee does not; it reaches \
         {without_the_guarantee}"
    );
}

/// The guarantee has an open-enrolment clawback, and the corpus did not know.
///
/// `[I]` is not "hold the district at its old amount". It is `funding base − open enrolment
/// adjustment − foundation funding`, and the middle term reduces the guarantee of a district whose
/// open enrolment FTE has fallen by more than `max(10% of last year, 20 FTE)`.
///
/// **The rate is the full statewide average base cost per pupil, $8,241.61 — not the district's
/// state share of it.** A district at the 10% minimum share was receiving about $824 of state
/// money for that pupil and loses ten times as much guarantee when the pupil leaves.
#[test]
fn the_guarantee_is_clawed_back_for_lost_open_enrolment_at_the_full_per_pupil_rate() {
    let panel = panel::panel();
    let mut clawed = 0;
    let mut withheld = 0.0;

    for record in &panel {
        let t = &record.transition;
        let threshold = (t.open_enrollment_prior * OPEN_ENROLLMENT_THRESHOLD_FRACTION)
            .max(OPEN_ENROLLMENT_THRESHOLD_FLOOR);
        assert!(
            (threshold - t.open_enrollment_threshold).abs() < 0.02,
            "{}: threshold computed {threshold:.2} against published {:.2}",
            record.name,
            t.open_enrollment_threshold
        );

        let expected = if t.open_enrollment_lost() >= t.open_enrollment_threshold {
            t.clawed_back_fte() * OPEN_ENROLLMENT_CLAWBACK_PER_FTE
        } else {
            0.0
        };
        assert!(
            (expected - t.open_enrollment_adjustment).abs() < 0.05,
            "{}: clawback computed {expected:.2} against published {:.2}",
            record.name,
            t.open_enrollment_adjustment
        );

        if t.open_enrollment_adjustment > 0.0 {
            clawed += 1;
            withheld += t.open_enrollment_adjustment;
        }
    }

    assert!(
        (20..80).contains(&clawed),
        "{clawed} districts are clawed back"
    );
    assert!(
        withheld > 4e6,
        "the clawback withholds {withheld:.0}, which is smaller than recorded"
    );
}

/// And the guarantee itself reproduces once the clawback is in the formula.
///
/// This is the check the corpus could not have made before: its guarantee node described a
/// hold-harmless with no clawback in it, and a reproduction from that description would have been
/// wrong for 43 districts and right for the rest — the failure mode that looks like rounding.
#[test]
fn the_guarantee_reproduces_only_with_the_clawback_in_it() {
    let panel = panel::panel();
    let mut wrong_without_it = 0;

    for record in &panel {
        let t = &record.transition;
        let base = t.funding_base;
        let with_clawback = if base > record.core_foundation_funding {
            (base - t.open_enrollment_adjustment - record.core_foundation_funding).max(0.0)
        } else {
            0.0
        };
        assert!(
            (with_clawback - record.guarantee).abs() < 0.05,
            "{}: computed {with_clawback:.2} against published {:.2}",
            record.name,
            record.guarantee
        );

        let naive = (base - record.core_foundation_funding).max(0.0);
        if (naive - record.guarantee).abs() > 1.0 {
            wrong_without_it += 1;
        }
    }

    assert!(
        wrong_without_it > 20,
        "a guarantee described without the clawback is wrong for {wrong_without_it} districts — \
         few enough to look like rounding, which is why it went unnoticed"
    );
}

/// Three mechanisms anchored to FY2021, on three different bases, holding three different sets.
///
/// The corpus has one node for a "temporary transitional aid guarantee". There are three:
///
/// - `[I]` holds a district at its FY2021 **funding base**, less the open-enrolment clawback;
/// - `[K]` holds it at a FY2021 base that **includes transportation**;
/// - transportation's own `[F]` holds it at FY2021 **transportation funding** alone.
///
/// They reach overlapping but different districts, which is what makes them three mechanisms
/// rather than one applied three times.
#[test]
fn three_separate_mechanisms_are_anchored_to_fiscal_2021() {
    let panel = panel::panel();
    let on =
        |pick: fn(&panel::DistrictRecord) -> f64| panel.iter().filter(|r| pick(r) > 0.0).count();
    let formula = on(|r| r.guarantee);
    let transition = on(|r| r.transition.transition_supplement);
    let transport = on(|r| r.transportation.guarantee);

    assert!(formula > 250 && transition > 100 && transport > 20);

    // No two of them hold the same set, and none is a subset of another.
    let only_transition = panel
        .iter()
        .filter(|r| r.transition.transition_supplement > 0.0 && r.guarantee <= 0.0)
        .count();
    let only_transport = panel
        .iter()
        .filter(|r| r.transportation.guarantee > 0.0 && r.guarantee <= 0.0)
        .count();
    assert!(
        only_transition > 0 && only_transport > 0,
        "{only_transition} draw the transition supplement without the guarantee and \
         {only_transport} draw transportation's without it; if either were zero the mechanisms \
         could be nested rather than distinct"
    );

    // And the FY2021 bases differ, which is why they can.
    let differing = panel
        .iter()
        .filter(|r| (r.transition.fy21_funding_base - r.transition.funding_base).abs() > 1.0)
        .count();
    assert!(
        differing > panel.len() / 2,
        "only {differing} districts have different FY2021 bases for the two hold-harmlesses"
    );
}
