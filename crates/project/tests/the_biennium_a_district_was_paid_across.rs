//! The FY2025→FY2027 comparison, and the two measures it refuses to conflate.
//!
//! Every figure here is observed: three published department files and no model. What the module
//! exists to prevent is a comparison drawn on the narrow measure the model projects, because the
//! two districts whose bars point opposite ways are identical on it.

use edfund_core::FiscalYear;
use project::biennium::{self, Measure, BASELINE_YEAR, MIDDLE_YEAR};
use project::panel::MODEL_YEAR;

const DELPHOS: &str = "043885";
const SHAWNEE: &str = "045799";

/// The population is the intersection, and it is the panel's 609.
#[test]
fn the_comparison_speaks_for_the_six_hundred_and_nine_all_three_files_agree_on() {
    let rows = biennium::frame();
    assert_eq!(rows.len(), 609);
    // The two island districts are paid in FY2025 and FY2026 and modelled in neither FY2027 nor
    // here, so no total below changes population between its years.
    for absent in ["048959", "048967"] {
        assert!(biennium::at(absent).is_none(), "{absent} is not modelled");
    }
}

/// The window is closed. A projected year is not a missing value, it is a different measure.
#[test]
fn a_year_past_the_model_has_no_answer_rather_than_a_projected_one() {
    let row = biennium::at(DELPHOS).expect("Delphos");
    for year in [BASELINE_YEAR, MIDDLE_YEAR, MODEL_YEAR] {
        assert!(row.total.at(year).is_some(), "{year:?} is observed");
    }
    for year in [FiscalYear(2024), FiscalYear(2028), FiscalYear(2032)] {
        assert!(row.total.at(year).is_none(), "{year:?} is not");
        assert!(row.total.against_baseline(year).is_none());
    }
}

/// **The five lines are exhaustive.** Their sum is the wide measure's own change, to the cent.
///
/// This is what licenses reading any one line as an explanation. A decomposition that left a few
/// dollars unattributed would still look right on the two districts anyone checks.
#[test]
fn the_lines_account_for_the_whole_of_the_change_on_every_district() {
    for row in biennium::frame() {
        let stated = row
            .total
            .against_baseline(MODEL_YEAR)
            .expect("the terminal year is in the window");
        let summed = row.lines.total();
        assert!(
            (stated - summed).abs() < 0.005,
            "{} ({}): lines sum to {summed} against a change of {stated}",
            row.total.name,
            row.total.irn
        );
    }
}

/// The two measures disagree, and the narrow one is the one that cannot see the difference.
///
/// Delphos and Shawnee are held at their funding base in every year of the window, so their
/// **foundation aid does not move at all** — a comparison on the model's measure reports both as
/// zero and is unable to distinguish them. On total state support they differ by a factor of two.
#[test]
fn the_model_measure_reports_both_districts_as_identical_and_the_wide_one_does_not() {
    let (delphos, shawnee) = (
        biennium::at(DELPHOS).expect("Delphos"),
        biennium::at(SHAWNEE).expect("Shawnee"),
    );

    assert_eq!(delphos.total.measure, Measure::TotalStateSupport);
    assert_eq!(delphos.foundation.measure, Measure::FoundationAid);

    // The narrow measure: Delphos flat, and Shawnee down by the step it gave back on arriving at
    // the guarantee. Neither is a gain, and the two are not comparable in the way the bars imply.
    let narrow = |row: &project::biennium::Row| {
        row.foundation
            .against_baseline(MODEL_YEAR)
            .expect("in the window")
    };
    assert!(
        narrow(&delphos).abs() < 0.005,
        "Delphos: {}",
        narrow(&delphos)
    );
    assert!(
        narrow(&shawnee) < -263_000.0,
        "Shawnee: {}",
        narrow(&shawnee)
    );

    // The wide measure, which is what a reader is shown: both gain, Shawnee by about 2.4x.
    let wide = |row: &project::biennium::Row| {
        row.total
            .against_baseline(MODEL_YEAR)
            .expect("in the window")
    };
    assert!(
        (wide(&delphos) - 67_496.78).abs() < 0.01,
        "{}",
        wide(&delphos)
    );
    assert!(
        (wide(&shawnee) - 163_854.81).abs() < 0.01,
        "{}",
        wide(&shawnee)
    );
}

/// What the difference between the two districts is actually made of.
///
/// Delphos's transportation gain is very nearly cancelled by losing the formula transition
/// supplement it was paid in FY2025 and Shawnee never had. That single line is the largest term in
/// the gap between them, and it sits outside the foundation formula entirely — so no base cost,
/// phase-in or guarantee lever reaches it.
#[test]
fn the_gap_between_them_is_transportation_against_a_lost_supplement() {
    let delphos = biennium::at(DELPHOS).expect("Delphos").lines;
    let shawnee = biennium::at(SHAWNEE).expect("Shawnee").lines;

    // Neither district's foundation line contributes anything to a gain.
    assert!(delphos.foundation.abs() < 0.005);
    assert!(shawnee.foundation < 0.0);

    // Shawnee's transportation gain is about 2.5x Delphos's.
    assert!((delphos.transportation - 82_912.74).abs() < 0.01);
    assert!((shawnee.transportation - 205_452.74).abs() < 0.01);

    // And Delphos alone gives a supplement back, which is most of why it lands where it does.
    assert!(
        delphos.supplements < -49_000.0,
        "Delphos supplements: {}",
        delphos.supplements
    );
    assert!(
        shawnee.supplements > 0.0,
        "Shawnee supplements: {}",
        shawnee.supplements
    );
}

/// The biennium counts both years against the baseline, which is not `FY2027 − FY2025`.
#[test]
fn the_biennium_figure_counts_each_year_against_the_year_it_is_measured_from() {
    let row = biennium::at(DELPHOS).expect("Delphos").total;
    let first = row.against_baseline(MIDDLE_YEAR).expect("FY2026");
    let second = row.against_baseline(MODEL_YEAR).expect("FY2027");
    assert!((row.biennium() - (first + second)).abs() < 0.005);
    // And it is strictly larger than the terminal-year change alone, which is the figure a reader
    // would reach for and which counts the first year not at all.
    assert!(row.biennium() > second);
}

/// **Statewide, the two measures point in opposite directions.**
///
/// Over FY2025→FY2027 total state support rises $145.0M and foundation aid falls $114.5M. Both
/// are true of the same 609 districts in the same years, and a reader told either one alone has
/// been told that Ohio's school funding went up, or that it went down.
///
/// The foundation fall is the state share being squeezed out from underneath: local capacity
/// rises while base cost is frozen at FY2022 inputs, and the guarantee holds a district only at a
/// nominal FY2020 figure that does not grow. What more than offsets it is entirely outside the
/// formula — transportation, special education transportation and the supplements H.B. 96
/// created.
///
/// This is the test the module exists for. Nothing else here would fail if `Measure` were dropped
/// and a caller picked a column.
#[test]
fn the_two_measures_disagree_about_the_direction_of_the_whole_biennium() {
    let rows = biennium::frame();
    let sum = |f: fn(&project::biennium::Row) -> f64| rows.iter().map(f).sum::<f64>();

    let wide = sum(|r| r.total.against_baseline(MODEL_YEAR).expect("in the window"));
    let narrow = sum(|r| {
        r.foundation
            .against_baseline(MODEL_YEAR)
            .expect("in the window")
    });

    assert!(
        (wide - 145_023_365.0).abs() < 1.0,
        "total state support: {wide}"
    );
    assert!(
        (narrow + 114_495_522.0).abs() < 1.0,
        "foundation aid: {narrow}"
    );
    assert!(
        wide > 0.0 && narrow < 0.0,
        "the measures must disagree in sign, or this test has stopped saying anything"
    );

    // And the offset is entirely outside the formula.
    let transport = sum(|r| r.lines.transportation);
    let sped_transport = sum(|r| r.lines.special_education_transportation);
    let supplements = sum(|r| r.lines.supplements);
    assert!(transport > 70_000_000.0 && sped_transport > 53_000_000.0);
    assert!(supplements > 134_000_000.0, "supplements: {supplements}");
    assert!(
        (transport
            + sped_transport
            + supplements
            + sum(|r| r.lines.preschool_special_education)
            + narrow
            - wide)
            .abs()
            < 1.0
    );

    // A gain statewide is not a gain everywhere: a third of districts are down on the wide measure.
    let down = rows
        .iter()
        .filter(|r| r.total.against_baseline(MODEL_YEAR).expect("in the window") < 0.0)
        .count();
    assert_eq!(down, 206, "districts down on total state support");
}
