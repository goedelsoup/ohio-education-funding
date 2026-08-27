//! Operating spending by function, FY2025, and the Toledo–Perrysburg comparison.
//!
//! Two jobs, as with the report-card suite. The first is replication: OCG Ground Truth
//! fact-check RL-2026-021 compared Toledo City against Perrysburg Exempted Village on
//! student-support share and total spending per pupil, and both figures are reproduced here from
//! the department's own file.
//!
//! The second is what the fact-check's own framing misses. It normalises special-education
//! spending by *total* pupils in two districts whose disability shares differ by nearly a factor
//! of two, and reads the residual as Toledo spending "somewhat more … consistent with a modestly
//! heavier student-need profile." Normalised by the population actually served, the sign of that
//! residual reverses.
//!
//! Cited by `corpus/education-agency/toledo-city.yml` and
//! `corpus/education-agency/perrysburg-exempted-village.yml`.

use dispersion::functions::{self, Functions};
use dispersion::report_card::{self, ReportCard};
use dispersion::Dispersion;

const TOLEDO: &str = "044909";
const PERRYSBURG: &str = "045583";

/// One district's function row.
fn functions_of(irn: &str) -> Functions {
    functions::district(irn).expect("district present in the function fixture")
}

/// One district's report card.
fn card_of(irn: &str) -> ReportCard {
    report_card::district(irn).expect("district present in the report card")
}

/// One function of one district, per pupil.
fn function(irn: &str, pick: fn(&Functions) -> Option<f64>) -> f64 {
    let row = functions_of(irn);
    pick(&row).expect("the department publishes this function for both districts")
}

/// One function of one district as a share of its operating spending.
fn share(irn: &str, pick: fn(&Functions) -> Option<f64>) -> f64 {
    let row = functions_of(irn);
    row.share(pick(&row)).expect("both figures are published")
}

fn close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{what}: got {actual:.4}, expected {expected:.4}"
    );
}

#[test]
fn the_fixture_covers_every_rated_district() {
    assert_eq!(functions::districts().len(), 607);
}

/// The department's two roll-ups partition operating spending exactly. Checked rather than
/// assumed, because a function fixture whose parts do not sum to its whole invites shares that
/// silently do not add to one.
///
/// Also held by `dispersion::functions`'s own unit test, which is where the reader states the
/// property. It stays here because it is the premise of every share below.
#[test]
fn classroom_and_nonclassroom_partition_operating_spending() {
    for d in functions::districts() {
        let (Some(classroom), Some(other), Some(operating)) =
            (d.classroom_instruction, d.nonclassroom, d.operating)
        else {
            panic!(
                "{}: the department publishes every district's roll-ups",
                d.name
            );
        };
        let sum = classroom + other;
        assert!(
            (sum - operating).abs() < operating * 0.001 + 1.0,
            "{}: {sum:.2} against {operating:.2}",
            d.name
        );
    }
}

// ---------------------------------------------------------------------------------------
// Replicating the fact-check
// ---------------------------------------------------------------------------------------

/// Both headline figures reproduce to the dollar and the tenth of a point.
#[test]
fn the_fact_check_figures_reproduce() {
    close(
        function(TOLEDO, |d| d.operating),
        20_805.0,
        1.0,
        "Toledo operating per pupil",
    );
    close(
        function(PERRYSBURG, |d| d.operating),
        14_632.0,
        1.0,
        "Perrysburg operating per pupil",
    );
    close(
        share(TOLEDO, |d| d.pupil_support),
        0.081,
        0.0005,
        "Toledo student support share",
    );
    close(
        share(PERRYSBURG, |d| d.pupil_support),
        0.085,
        0.0005,
        "Perrysburg student support share",
    );
}

/// The fact-check divides by headcount ADM. The same author's White Paper 013 divides the same
/// district's spending by weighted ADM, and neither publication notes the other's basis.
#[test]
fn the_two_publications_report_toledo_on_different_denominators() {
    let toledo = card_of(TOLEDO);
    let dollars = toledo.operating_expenditure.unwrap();
    let headcount = toledo.unweighted_adm.unwrap();
    let weighted = toledo.weighted_adm.unwrap();
    let published = toledo.per_equivalent_pupil.unwrap();

    close(dollars / headcount, 20_805.0, 1.0, "the fact-check's basis");
    close(dollars / weighted, 14_312.0, 1.0, "White Paper 013's basis");
    close(
        published,
        14_312.0,
        1.0,
        "and that is what the report card prints",
    );

    // A 45% spread on one district in one year, depending only on the divisor.
    assert!(dollars / headcount > 1.45 * (dollars / weighted));
}

// ---------------------------------------------------------------------------------------
// What normalising by total pupils conceals
// ---------------------------------------------------------------------------------------

/// Toledo's disability share is nearly double Perrysburg's, which the fact-check calls a
/// "modestly heavier student-need profile".
#[test]
fn the_disability_shares_differ_by_nearly_a_factor_of_two() {
    let toledo = card_of(TOLEDO).students_with_disabilities.unwrap();
    let perrysburg = card_of(PERRYSBURG).students_with_disabilities.unwrap();
    close(toledo, 21.9, 0.05, "Toledo students with disabilities");
    close(
        perrysburg,
        11.3,
        0.05,
        "Perrysburg students with disabilities",
    );
    assert!(toledo > 1.9 * perrysburg);
}

/// **The correction.** Per *total* pupil Toledo spends 1.17 to 1.36 times what Perrysburg does
/// on special-education instruction; per *student with a disability* it spends about a third
/// less, because it serves nearly twice the share.
///
/// The special-education figures are the fact-check's own, from audited statements this corpus
/// does not hold; the shares are the report card's. Mixing them makes this an order-of-magnitude
/// normalisation rather than an audited per-pupil cost — but a 1.94x population difference
/// against a 1.36x spending difference leaves no room for the sign to come back.
#[test]
fn normalising_by_the_population_served_reverses_the_comparison() {
    let toledo_share = card_of(TOLEDO).students_with_disabilities.unwrap() / 100.0;
    let perrysburg_share = card_of(PERRYSBURG).students_with_disabilities.unwrap() / 100.0;

    // As published by the fact-check, per total pupil.
    let toledo_per_pupil = 2_766.0;
    let (perrysburg_low, perrysburg_high) = (2_032.0, 2_363.0);
    assert!(toledo_per_pupil > perrysburg_high);

    let toledo_per_student = toledo_per_pupil / toledo_share;
    let perrysburg_per_student_low = perrysburg_low / perrysburg_share;
    let perrysburg_per_student_high = perrysburg_high / perrysburg_share;

    close(
        toledo_per_student,
        12_630.0,
        20.0,
        "Toledo per student with a disability",
    );
    close(
        perrysburg_per_student_low,
        17_982.0,
        20.0,
        "Perrysburg, low basis",
    );
    close(
        perrysburg_per_student_high,
        20_912.0,
        20.0,
        "Perrysburg, high basis",
    );

    // The comparison inverts, and not marginally.
    assert!(toledo_per_student < 0.75 * perrysburg_per_student_low);
}

// ---------------------------------------------------------------------------------------
// The comparison the same file supports and the fact-check did not make
// ---------------------------------------------------------------------------------------

/// Toledo spends $6,172 more per pupil and a markedly *smaller* share of it in the classroom.
/// This is a function-level difference between the two districts an order of magnitude larger
/// than the special-education share the claim happened to name.
///
/// # Every figure here is asserted as a value, and two of them used not to be
///
/// `education-agency/toledo-city` prints this table to the tenth of a point and the gap to the
/// dollar. Plant and building administration were checked here as *ratios* — `Toledo > 1.8x
/// Perrysburg` — which is true of the published shares and would also be true of a great many
/// wrong ones, so the two figures the corpus published were not the two figures under test.
///
/// The gap was worse than that in a smaller way. It was asserted at `6_173.0` within two dollars,
/// and the difference of the two published per-pupil figures is `$6,172.41`. Both nodes published
/// `$6,173`, and the tolerance was wide enough that nothing said so for as long as the corpus
/// carried it. See #218 and both nodes' `revisions:`.
#[test]
fn toledo_spends_more_and_a_smaller_share_of_it_on_instruction() {
    close(
        share(TOLEDO, |d| d.instruction),
        0.513,
        0.002,
        "Toledo instruction share",
    );
    close(
        share(PERRYSBURG, |d| d.instruction),
        0.626,
        0.002,
        "Perrysburg instruction share",
    );
    close(
        share(TOLEDO, |d| d.classroom_instruction),
        0.634,
        0.002,
        "Toledo classroom instruction share",
    );
    close(
        share(PERRYSBURG, |d| d.classroom_instruction),
        0.731,
        0.002,
        "Perrysburg classroom instruction share",
    );

    // Where the gap actually opens: plant and building administration, not instruction. Stated
    // to the tenth of a point because that is how the corpus prints them.
    close(
        share(TOLEDO, |d| d.operations_maintenance),
        0.148,
        0.0005,
        "Toledo operations and maintenance share",
    );
    close(
        share(PERRYSBURG, |d| d.operations_maintenance),
        0.077,
        0.0005,
        "Perrysburg operations and maintenance share",
    );
    close(
        share(TOLEDO, |d| d.school_admin),
        0.085,
        0.0005,
        "Toledo building administration share",
    );
    close(
        share(PERRYSBURG, |d| d.school_admin),
        0.048,
        0.0005,
        "Perrysburg building administration share",
    );

    let gap = function(TOLEDO, |d| d.operating) - function(PERRYSBURG, |d| d.operating);
    close(gap, 6_172.41, 0.01, "per-pupil spending gap");
}

/// Neither district is typical, which is the point of holding both. Toledo sits near the top of
/// the state on operating spending per pupil and Perrysburg below the median.
#[test]
fn the_pair_brackets_the_state_distribution() {
    let all: Vec<f64> = functions::districts()
        .iter()
        .filter_map(|d| d.operating)
        .collect();
    let d = Dispersion::of(&all).unwrap();
    assert_eq!(d.n, 607);
    close(
        d.median,
        16_289.0,
        5.0,
        "statewide median operating per pupil",
    );
    assert!(function(TOLEDO, |d| d.operating) > d.median);
    assert!(function(PERRYSBURG, |d| d.operating) < d.median);
    // And Perrysburg's ADM is roughly a quarter of Toledo's.
    assert!(function(TOLEDO, |d| d.adm) > 3.5 * function(PERRYSBURG, |d| d.adm));
}
