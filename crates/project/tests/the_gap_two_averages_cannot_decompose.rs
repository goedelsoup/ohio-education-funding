//! What the difference between two published averages can settle, and what it cannot.
//!
//! Traditional EdChoice averaged **$6,808.75** in 2024-25 and EdChoice Expansion **$4,958.41** — a
//! gap of $1,850.34, 27.2%. The two programmes pay out of the same schedule, so the gap looks like
//! a clean measurement of what R.C. 3310.08's income decay does. It is not, and these tests are
//! mostly about why.
//!
//! # The one thing that makes the comparison exact
//!
//! R.C. 3310.08(A)(2) defines the expansion's "base amount" as *the maximum educational choice
//! scholarship amount for the student's grade level under division (A)(10)(a)(ii)(I) of section
//! 3317.022* — the same $5,500 and $7,500 the traditional programme pays. One schedule, two
//! programmes, and the expansion is that schedule multiplied by a factor at most one.
//!
//! # And the thing that makes it exact *in this year*
//!
//! Those amounts are not fixed: the section directs them to rise with the statewide average base
//! cost per pupil. In FY2025 they did not rise at all, because H.B. 33 froze that average at its
//! FY2024 level. **The voucher ceiling is indexed to a number that was held still**, so the
//! enacted figures are the operative ones and no indexing is needed to compare them against a
//! 2024-25 report. See `the_ceiling_is_indexed_to_a_number_that_did_not_move`, and
//! `project::indexation` for the general shape.
//!
//! # Three unknowns, two equations
//!
//! An average award moves with the grade mix, with the decay, and with tuition below the ceiling.
//! Two published averages cannot separate three unknowns, and nothing here pretends otherwise.
//! What the arithmetic *does* support is a pair of one-sided bounds, and they are the findings:
//!
//! - Expansion's average is **below the lowest figure any grade mix can produce**. Something other
//!   than the mix is reducing awards, necessarily, by at least $541.59 a student.
//! - Traditional's average is **above** it, which bounds its K-8 share at 34.6% and is surprising
//!   enough to be worth stating as a question rather than a result.
//!
//! # The populations are not the same, and they push opposite ways
//!
//! Traditional EdChoice is triggered by the performance of the building a student is assigned to;
//! the expansion is universal. So expansion families are on average richer — which lowers the
//! expansion average through the decay — while traditional families plausibly attend cheaper
//! schools, which lowers the *traditional* average through the tuition ceiling. The two effects
//! work in opposite directions on the same gap, and neither is measured here. That is the reason
//! a point estimate is not offered.

use project::base_cost;
use project::scholarship::{
    self, report, Average, EDCHOICE_BASE_9_12, EDCHOICE_BASE_K8, FULL_AWARD_CEILING, MINIMUM_SHARE,
};

/// The report's slug for the universal programme.
const EXPANSION: &str = "edchoice-expansion";

/// And for the building-triggered one.
const TRADITIONAL: &str = "traditional-edchoice";

fn average(programme: &str, basis: Average) -> f64 {
    let programmes = report::programmes();
    basis
        .of(programmes.get(programme).expect("in the report"))
        .expect("the programme publishes both averages")
}

/// The ceiling rises with a number the same act froze, so in FY2025 it did not rise.
///
/// This is what licenses reading the enacted $5,500 and $7,500 against a 2024-25 report without
/// indexing them first, and it is worth a test rather than a remark because the statute's
/// direction and its effect point opposite ways: the section says the amount increases, and the
/// amount did not.
#[test]
fn the_ceiling_is_indexed_to_a_number_that_did_not_move() {
    let enacted = base_cost::nominal(2024).expect("FY2024 is in the series");
    let report_year = base_cost::nominal(2025).expect("FY2025 is in the series");
    assert!(
        (enacted - report_year).abs() < 0.005,
        "FY2025 base cost is {report_year} against FY2024's {enacted}"
    );

    let (nominal, real) = base_cost::growth(2024, 2025).expect("both years");
    assert!(nominal.abs() < 1e-9, "the index target moved {nominal}");
    assert!(real < 0.0, "and it lost ground in real terms: {real}");

    // FY2025 is frozen by act rather than flat by coincidence, and the series says which.
    let year = base_cost::year(2025).expect("FY2025");
    assert_eq!(
        year.frozen.map(|freeze| freeze.anchor),
        Some(2024),
        "FY2025 should be a frozen year anchored to FY2024"
    );
}

/// **The finding.** No grade mix can produce the expansion's average.
///
/// Every award is the lesser of a tuition and a per-grade ceiling, and the smaller ceiling is
/// $5,500. A programme paying every student the full ceiling therefore cannot average below
/// $5,500 whatever its grade mix — and the expansion averages $4,958.41.
///
/// So at least **$541.59 a student** is removed by something that is not the mix: the income
/// decay, a tuition below the ceiling, or both. Across 100,939 students that is **$54.7 million**
/// the schedule directs and the programme did not pay.
#[test]
fn the_expansion_average_is_below_anything_the_grade_mix_alone_could_produce() {
    let students = report::programmes()[EXPANSION].students;

    for (basis, expected_shortfall, expected_aggregate) in [
        (Average::Published, 541.59, 54_667_553.0),
        (Average::Implied, 617.18, 62_297_264.0),
    ] {
        let observed = average(EXPANSION, basis);
        assert!(
            observed < EDCHOICE_BASE_K8,
            "{observed} clears the floor on {basis:?} and the finding would not hold"
        );

        let shortfall =
            scholarship::shortfall_no_mix_explains(observed).expect("below the K-8 ceiling");
        assert!(
            (shortfall - expected_shortfall).abs() < 0.005,
            "{basis:?} shortfall is {shortfall}"
        );
        assert!(
            (shortfall * students - expected_aggregate).abs() < 1.0,
            "{basis:?} aggregate is {}",
            shortfall * students
        );

        // And no mix is recoverable, which is the same fact stated as the function's refusal.
        assert_eq!(scholarship::greatest_k8_share(observed), None);
    }
}

/// Traditional's average is above that floor, which bounds its K-8 share — implausibly low.
///
/// $6,808.75 requires a mix at most **34.6% K-8**, and that is before any tuition binds; every
/// student paying less than the ceiling forces the share lower still. Grades K-8 are nine of
/// thirteen grade levels, so a share near a third is far from what an even spread would give.
///
/// **This is recorded as a question rather than a result.** Either traditional EdChoice is
/// overwhelmingly a high-school programme, or the published average is on a basis that does not
/// support this arithmetic — and the catalog already records two other respects in which the
/// report's published averages do not reconcile. The implied average moves the bound to 42.7% and
/// does not rescue it.
#[test]
fn the_traditional_average_bounds_its_own_grade_mix_at_an_implausible_place() {
    for (basis, expected) in [
        (Average::Published, 0.345_625),
        (Average::Implied, 0.427_19),
    ] {
        let observed = average(TRADITIONAL, basis);
        assert!(
            (EDCHOICE_BASE_K8..=EDCHOICE_BASE_9_12).contains(&observed),
            "{observed} is outside the schedule on {basis:?}"
        );
        let share = scholarship::greatest_k8_share(observed).expect("inside the schedule");
        assert!(
            (share - expected).abs() < 0.0005,
            "{basis:?} bounds the K-8 share at {share}"
        );
        assert!(
            share < 0.5,
            "a bound above a half would not be worth remarking on"
        );
    }

    // The bound is a ceiling: a lower average forces a lower share, never a higher one.
    let steeper = scholarship::greatest_k8_share(6_000.0).expect("inside the schedule");
    let shallower = scholarship::greatest_k8_share(7_000.0).expect("inside the schedule");
    assert!(
        steeper > shallower,
        "the bound should fall as the average rises"
    );
}

/// What the decay would have to be if it explained the whole gap, and why that is a ceiling.
///
/// Holding the grade mix equal and assuming no tuition binds anywhere attributes every dollar of
/// the gap to R.C. 3310.08. That gives a mean factor of **0.7282** — the average expansion student
/// receiving 72.8% of what the schedule would pay them without the decay.
///
/// Inverting the decay puts that factor at **4.96 times the federal poverty guidelines**. Because
/// the decay is convex in income, Jensen's inequality makes the mean *income* at least that, so
/// under this reading the average expansion family sits above 495% of the guidelines. Every
/// dollar of the gap that is really a tuition ceiling makes the decay smaller and this reading
/// weaker, which is why it is the largest the decay can be rather than an estimate of it.
#[test]
fn the_decay_explaining_the_whole_gap_would_put_the_average_family_near_five_times_poverty() {
    for (basis, expected_factor, expected_income) in [
        (Average::Published, 0.728_241, 4.957_5),
        (Average::Implied, 0.734_743, 4.944_7),
    ] {
        let factor = scholarship::decay_explaining_the_whole_gap(
            average(EXPANSION, basis),
            average(TRADITIONAL, basis),
        )
        .expect("both averages are positive");
        assert!(
            (factor - expected_factor).abs() < 0.0005,
            "{basis:?} factor is {factor}"
        );

        let income = scholarship::income_paying(factor).expect("between the floor and one");
        assert!(
            (income - expected_income).abs() < 0.005,
            "{basis:?} income is {income}"
        );
        assert!(
            income > FULL_AWARD_CEILING,
            "below 450% nothing decays at all"
        );
    }
}

/// The inverse is the statute's own curve, and it stops where the statute stops distinguishing.
#[test]
fn the_income_a_factor_implies_is_the_inverse_of_the_award_the_statute_computes() {
    for base in [EDCHOICE_BASE_K8, EDCHOICE_BASE_9_12] {
        for tenths in 1..=10 {
            let factor = f64::from(tenths) / 10.0;
            let income = scholarship::income_paying(factor).expect("a valid factor");
            let paid = scholarship::expansion_award(base, income);
            assert!(
                (paid / base - factor).abs() < 1e-9,
                "{factor} round-trips to {}",
                paid / base
            );
        }
    }

    // Past the floor the award no longer varies with income, so no income is recoverable.
    assert_eq!(scholarship::income_paying(MINIMUM_SHARE / 2.0), None);
    assert_eq!(scholarship::income_paying(1.01), None);

    // At the two ends the inverse gives the statute's own boundaries.
    let full = scholarship::income_paying(1.0).expect("the full award");
    assert!((full - FULL_AWARD_CEILING).abs() < 1e-12);
    let floor = scholarship::income_paying(MINIMUM_SHARE).expect("the floor");
    assert!((floor - scholarship::minimum_award_threshold()).abs() < 1e-12);
}

/// And the finding survives the fact that some expansion students are not on the decayed schedule.
///
/// R.C. 3317.022(A)(10)(a)(ii)(I) pays the **flat** ceiling to a student who first received a
/// 3310.032 scholarship before 3 October 2023 and whose parent does not elect the new calculation.
/// Those students are inside the report's expansion count, receiving an undecayed award.
///
/// That makes the sub-$5,500 average harder to produce rather than easier: part of the programme
/// is paying the full schedule, so whatever reduces the rest must reduce it further to bring the
/// whole average below the floor. Nothing here measures the split — the report does not publish it
/// — and the direction is what matters.
#[test]
fn part_of_the_expansion_is_paid_on_the_undecayed_schedule_and_the_finding_still_holds() {
    let observed = average(EXPANSION, Average::Published);
    assert!(observed < EDCHOICE_BASE_K8);

    // If a share `s` of the programme is paid the flat K-8 ceiling, the rest must average this,
    // and it falls as that share rises.
    let rest_must_average =
        |flat_share: f64| (observed - flat_share * EDCHOICE_BASE_K8) / (1.0 - flat_share);
    let none = rest_must_average(0.0);
    let tenth = rest_must_average(0.10);
    let quarter = rest_must_average(0.25);

    assert!((none - observed).abs() < 1e-9);
    assert!(
        tenth < none,
        "a flat share makes the remainder lower, not higher"
    );
    assert!(quarter < tenth);
    assert!(
        quarter < 4_800.0,
        "a quarter of the programme on the flat schedule puts the rest at {quarter}"
    );
}

/// What would actually identify the decomposition, stated so a later reader does not re-derive it.
///
/// Three unknowns and two equations. Any **one** of these closes it, and none is held: the
/// report's participation split by grade band, its published maximum award per programme, or the
/// income distribution of expansion recipients. The report links a per-district breakdown and the
/// route returns 404, which the `deduction` skill already records.
#[test]
fn the_report_carries_neither_of_the_two_columns_that_would_close_this() {
    let programmes = report::programmes();
    let expansion = &programmes[EXPANSION];

    // Participation is one number, not a grade split, and expenditure is one number too.
    assert!((expansion.students - 100_939.0).abs() < 0.5);
    assert!(expansion.expenditure.is_some());
    assert!(expansion.published_average.is_some());

    // Five programmes, five rows, and no column beyond these.
    assert_eq!(programmes.len(), 5);
    assert!(
        programmes.values().all(|p| p.students > 0.0),
        "a grade split would arrive as more rows or more columns, and neither is here"
    );
}
