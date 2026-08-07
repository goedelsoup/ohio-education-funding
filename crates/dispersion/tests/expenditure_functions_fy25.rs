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

use dispersion::Dispersion;

const FUNCTIONS: &str = include_str!("../fixtures/expenditure-functions-fy25.csv");
const REPORT_CARD: &str = include_str!("../fixtures/report-card-2425-district-data.csv");

/// Column indices in the function fixture.
mod col {
    pub const IRN: usize = 0;
    pub const ADM: usize = 2;
    pub const OPERATING: usize = 3;
    pub const INSTRUCTION: usize = 4;
    pub const PUPIL_SUPPORT: usize = 5;
    pub const CLASSROOM_INSTRUCTION: usize = 7;
    pub const SCHOOL_ADMIN: usize = 9;
    pub const OPERATIONS_MAINTENANCE: usize = 10;
    pub const NONCLASSROOM: usize = 14;
}

/// Column indices in the report-card fixture.
mod card {
    pub const IRN: usize = 0;
    pub const UNWEIGHTED_ADM: usize = 5;
    pub const WEIGHTED_ADM: usize = 6;
    pub const OPERATING_EXPENDITURES: usize = 7;
    pub const EQUIVALENT_PUPIL: usize = 8;
    pub const SWD_SHARE: usize = 16;
}

const TOLEDO: &str = "044909";
const PERRYSBURG: &str = "045583";

fn field(csv: &str, irn: &str, key: usize, column: usize) -> Option<f64> {
    csv.lines().skip(1).find_map(|line| {
        let parts: Vec<&str> = line.split(',').collect();
        (parts.get(key).map(|s| s.trim()) == Some(irn))
            .then(|| parts.get(column).and_then(|v| v.trim().parse::<f64>().ok()))
            .flatten()
    })
}

fn function(irn: &str, column: usize) -> f64 {
    field(FUNCTIONS, irn, col::IRN, column).expect("district present in the function fixture")
}

fn share(irn: &str, column: usize) -> f64 {
    function(irn, column) / function(irn, col::OPERATING)
}

fn close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{what}: got {actual:.4}, expected {expected:.4}"
    );
}

#[test]
fn the_fixture_covers_every_rated_district() {
    assert_eq!(FUNCTIONS.lines().count() - 1, 607);
}

/// The department's two roll-ups partition operating spending exactly. Checked rather than
/// assumed, because a function fixture whose parts do not sum to its whole invites shares that
/// silently do not add to one.
#[test]
fn classroom_and_nonclassroom_partition_operating_spending() {
    for line in FUNCTIONS.lines().skip(1) {
        let parts: Vec<&str> = line.split(',').collect();
        let get = |i: usize| parts[i].trim().parse::<f64>().unwrap_or(0.0);
        let sum = get(col::CLASSROOM_INSTRUCTION) + get(col::NONCLASSROOM);
        let operating = get(col::OPERATING);
        assert!(
            (sum - operating).abs() < operating * 0.001 + 1.0,
            "{}: {sum:.2} against {operating:.2}",
            parts[1]
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
        function(TOLEDO, col::OPERATING),
        20_805.0,
        1.0,
        "Toledo operating per pupil",
    );
    close(
        function(PERRYSBURG, col::OPERATING),
        14_632.0,
        1.0,
        "Perrysburg operating per pupil",
    );
    close(
        share(TOLEDO, col::PUPIL_SUPPORT),
        0.081,
        0.0005,
        "Toledo student support share",
    );
    close(
        share(PERRYSBURG, col::PUPIL_SUPPORT),
        0.085,
        0.0005,
        "Perrysburg student support share",
    );
}

/// The fact-check divides by headcount ADM. The same author's White Paper 013 divides the same
/// district's spending by weighted ADM, and neither publication notes the other's basis.
#[test]
fn the_two_publications_report_toledo_on_different_denominators() {
    let dollars = field(REPORT_CARD, TOLEDO, card::IRN, card::OPERATING_EXPENDITURES).unwrap();
    let headcount = field(REPORT_CARD, TOLEDO, card::IRN, card::UNWEIGHTED_ADM).unwrap();
    let weighted = field(REPORT_CARD, TOLEDO, card::IRN, card::WEIGHTED_ADM).unwrap();
    let published = field(REPORT_CARD, TOLEDO, card::IRN, card::EQUIVALENT_PUPIL).unwrap();

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
    let toledo = field(REPORT_CARD, TOLEDO, card::IRN, card::SWD_SHARE).unwrap();
    let perrysburg = field(REPORT_CARD, PERRYSBURG, card::IRN, card::SWD_SHARE).unwrap();
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
    let toledo_share = field(REPORT_CARD, TOLEDO, card::IRN, card::SWD_SHARE).unwrap() / 100.0;
    let perrysburg_share =
        field(REPORT_CARD, PERRYSBURG, card::IRN, card::SWD_SHARE).unwrap() / 100.0;

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

/// Toledo spends $6,173 more per pupil and a markedly *smaller* share of it in the classroom.
/// This is a function-level difference between the two districts an order of magnitude larger
/// than the special-education share the claim happened to name.
#[test]
fn toledo_spends_more_and_a_smaller_share_of_it_on_instruction() {
    close(
        share(TOLEDO, col::INSTRUCTION),
        0.513,
        0.002,
        "Toledo instruction share",
    );
    close(
        share(PERRYSBURG, col::INSTRUCTION),
        0.626,
        0.002,
        "Perrysburg instruction share",
    );
    close(
        share(TOLEDO, col::CLASSROOM_INSTRUCTION),
        0.634,
        0.002,
        "Toledo classroom instruction share",
    );
    close(
        share(PERRYSBURG, col::CLASSROOM_INSTRUCTION),
        0.731,
        0.002,
        "Perrysburg classroom instruction share",
    );

    // Where the gap actually opens: plant and building administration, not instruction.
    assert!(
        share(TOLEDO, col::OPERATIONS_MAINTENANCE)
            > 1.8 * share(PERRYSBURG, col::OPERATIONS_MAINTENANCE)
    );
    assert!(share(TOLEDO, col::SCHOOL_ADMIN) > 1.7 * share(PERRYSBURG, col::SCHOOL_ADMIN));

    let gap = function(TOLEDO, col::OPERATING) - function(PERRYSBURG, col::OPERATING);
    close(gap, 6_173.0, 2.0, "per-pupil spending gap");
}

/// Neither district is typical, which is the point of holding both. Toledo sits near the top of
/// the state on operating spending per pupil and Perrysburg below the median.
#[test]
fn the_pair_brackets_the_state_distribution() {
    let all: Vec<f64> = FUNCTIONS
        .lines()
        .skip(1)
        .filter_map(|l| l.split(',').nth(col::OPERATING)?.trim().parse::<f64>().ok())
        .collect();
    let d = Dispersion::of(&all).unwrap();
    assert_eq!(d.n, 607);
    close(
        d.median,
        16_289.0,
        5.0,
        "statewide median operating per pupil",
    );
    assert!(function(TOLEDO, col::OPERATING) > d.median);
    assert!(function(PERRYSBURG, col::OPERATING) < d.median);
    // And Perrysburg's ADM is roughly a quarter of Toledo's.
    assert!(function(TOLEDO, col::ADM) > 3.5 * function(PERRYSBURG, col::ADM));
}
