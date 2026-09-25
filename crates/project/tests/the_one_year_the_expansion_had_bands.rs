//! The year the expansion scholarship was a table, and the three things H.B. 33 left `[open]`.
//!
//! `program/edchoice-expansion` says, flatly, "**There are no income bands.** The award is a
//! continuous function of family income." That is true of the law as it stands and false of the
//! first year of the universal programme. H.B. 33 prescribed a schedule of eight bands for FY2024
//! and switched to the logarithmic function only from FY2025, so the claim needs a year attached
//! to it — and the year it needs is the one the programme's participation tripled in.
//!
//! The same analysis settles the rest of what `legislation/hb-33-2023` recorded as unrecorded: the
//! appropriation totals for the channel, and the act's vetoes, of which there is exactly one and
//! it touches no scholarship.
//!
//! # The greenbook checks the crate twice
//!
//! LSC worked the FY2025 formula independently and published two points on it and the income at
//! which its floor binds. All three reproduce against [`project::scholarship::expansion_award`],
//! which reads R.C. 3310.08 directly — and the third,
//! [`the_income_lsc_publishes_for_the_floor_is_the_one_the_statute_implies`], corroborates a
//! quantity the corpus had derived and marked `[inference]` because "the statute gives the floor
//! as an amount and never says where it binds". LSC says where it binds.

use project::greenbook::greenbook;
use project::scholarship::{expansion_award, minimum_award_threshold, MINIMUM_SHARE};

/// The act's maximum traditional EdChoice awards for FY2024, which are the expansion's base
/// amounts in the same year: the H.B. 110 figures after one year of base-cost indexing.
const FY2024_BASE_K8: f64 = 6_165.0;
const FY2024_BASE_9_12: f64 = 8_407.0;

/// FY2024 was a schedule of eight bands, printed as a table.
///
/// The band edges are the thing the corpus said did not exist. They are at 450, 500, 550, 600,
/// 650, 700 and 750 per cent of the federal poverty guidelines, and above the last of them the
/// award is a flat $650 and $950 rather than a tenth of the base.
#[test]
fn the_expansion_had_eight_income_bands_for_exactly_one_year() {
    let analysis = greenbook("hb33");

    let prescribed = analysis.around("For FY 2024 only, the budget prescribes specific", 3);
    assert!(
        prescribed.contains("which are listed in Table 5 below"),
        "the FY2024 prescription sentence has moved: {prescribed}"
    );

    let table = analysis.around("Table 5. First-Time EdChoice Expansion Scholarship", 14);
    for row in [
        "At or below 450% $6,165 $8,407",
        "451% to 500% $5,200 $7,050",
        "501% to 550% $3,650 $5,000",
        "551% to 600% $2,600 $3,550",
        "601% to 650% $1,850 $2,500",
        "651% to 700% $1,300 $1,750",
        "701% to 750% $900 $1,250",
        "Above 750% $650 $950",
    ] {
        assert!(
            table.contains(row),
            "Table 5 no longer carries {row:?}: {table}"
        );
    }

    let after = analysis.around("Beginning in FY 2025, a logarithmic function formula", 3);
    assert!(
        after.contains("will be used to determine the"),
        "the FY2025 changeover sentence has moved: {after}"
    );
}

/// LSC's two worked examples of the FY2025 formula reproduce against the statute exactly.
///
/// A second publisher arriving at the same two points is what makes the corpus's reading of
/// R.C. 3310.08 a reading rather than an interpretation. They are also the two roundest points on
/// the curve — one halving and two — so an error in the exponent would show at both.
#[test]
fn lscs_worked_examples_reproduce_the_statutes_curve() {
    let analysis = greenbook("hb33");
    let examples = analysis.around("in a student with a family income of 550% FPL", 3);
    assert!(
        examples.contains("receiving 50% of the base amount"),
        "LSC's 550% example has moved: {examples}"
    );
    assert!(
        examples.contains("a family income of 650% FPL receiving 25% of the base amount"),
        "LSC's 650% example has moved: {examples}"
    );

    assert!((expansion_award(FY2024_BASE_K8, 5.5) - FY2024_BASE_K8 * 0.50).abs() < 1e-9);
    assert!((expansion_award(FY2024_BASE_K8, 6.5) - FY2024_BASE_K8 * 0.25).abs() < 1e-9);
    assert!((expansion_award(FY2024_BASE_9_12, 5.5) - FY2024_BASE_9_12 * 0.50).abs() < 1e-9);
    assert!((expansion_award(FY2024_BASE_9_12, 6.5) - FY2024_BASE_9_12 * 0.25).abs() < 1e-9);
}

/// LSC publishes the income at which the floor binds, and the statute implies the same one.
///
/// [`minimum_award_threshold`] solves the decay against the ten per cent floor and gets
/// **782.19%**; LSC writes that the minimum "is triggered at 783% FPL", which is that number
/// rounded up to the whole percentage point at which the floor is actually in force. The corpus
/// carried the derivation as `[inference]` on the ground that nothing stated it. Something does.
#[test]
fn the_income_lsc_publishes_for_the_floor_is_the_one_the_statute_implies() {
    let analysis = greenbook("hb33");
    let floor = analysis.around("a minimum scholarship amount equal to 10% of the base", 2);
    assert!(
        floor.contains("which is triggered at 783% FPL"),
        "LSC's floor sentence has moved: {floor}"
    );

    let derived = minimum_award_threshold();
    assert!((derived - 7.821_928_094_887_363).abs() < 1e-12, "{derived}");

    // LSC's figure is the first whole percentage point at or past the derived one, and the
    // point before it is not yet on the floor. Both halves matter: the first says LSC did not
    // round the other way, the second says 783 is not merely close.
    let published = 7.83;
    assert!(published > derived);
    assert_eq!((derived * 100.0).ceil(), 783.0);
    assert!(expansion_award(FY2024_BASE_K8, 7.82) > FY2024_BASE_K8 * MINIMUM_SHARE);
    assert!(
        (expansion_award(FY2024_BASE_K8, published) - FY2024_BASE_K8 * MINIMUM_SHARE).abs() < 1e-9
    );
}

/// The table and the curve are different schedules, and the table pays more at every band edge.
///
/// A step schedule keyed to a band's range and a continuous decay cannot agree except at crossing
/// points. Evaluated at the top of each of the seven bands above 450% — the income at which a
/// family is about to fall into the next band down — the FY2024 table pays more than the FY2025
/// formula does, in both grade groups, fourteen comparisons out of fourteen.
///
/// So the changeover was not a re-expression of the same schedule. For a first-time recipient
/// near the top of any band it was a cut, and the corpus's "there are no income bands" reads
/// FY2025's shape back over a year that had a different one.
#[test]
fn the_prescribed_table_and_the_formula_are_not_the_same_schedule() {
    // Band ceiling as a multiple of poverty, then the table's K-8 and 9-12 amounts there.
    let bands = [
        (5.00, 5_200.0, 7_050.0),
        (5.50, 3_650.0, 5_000.0),
        (6.00, 2_600.0, 3_550.0),
        (6.50, 1_850.0, 2_500.0),
        (7.00, 1_300.0, 1_750.0),
        (7.50, 900.0, 1_250.0),
        (8.00, 650.0, 950.0),
    ];

    for (ratio, k8, high) in bands {
        assert!(
            k8 > expansion_award(FY2024_BASE_K8, ratio),
            "at {ratio} the K-8 table amount is not above the curve"
        );
        assert!(
            high > expansion_award(FY2024_BASE_9_12, ratio),
            "at {ratio} the 9-12 table amount is not above the curve"
        );
    }

    // The top band is the one where the two schedules are nearest, because the curve is already
    // on its floor there: $650 against $616.50, a difference of $33.50.
    let floor = FY2024_BASE_K8 * MINIMUM_SHARE;
    assert!((floor - 616.50).abs() < 1e-9);
    assert!((650.0 - floor - 33.50).abs() < 1e-9);
}

/// The act's own estimate of what the channel would cost, which the node recorded as unrecorded.
///
/// Table 3 is the enacted figure for all five programmes across three years, and it is the
/// document that dates the expansion's inflection: $121.3m in FY2023, the last income-limited
/// year, against $397.8m in FY2024, the first universal one. The channel as a whole rises 62.1%
/// in that year, and this one programme is three-quarters of the rise.
#[test]
fn the_greenbook_states_the_channels_cost_across_the_change() {
    let analysis = greenbook("hb33");
    let table = analysis.around("Table 3. Estimated Scholarship Payments by Program", 8);

    for row in [
        "EdChoice Expansion $121.3 $397.8 $439.1",
        "Traditional EdChoice $228.2 $276.6 $299.0",
        "Autism $122.5 $135.5 $142.3",
        "Jon Peterson Special Needs $77.9 $100.6 $115.1",
        "Cleveland $45.0 $54.0 $55.4",
        "Total $595.0 $964.5 $1,050.9",
    ] {
        assert!(
            table.contains(row),
            "Table 3 no longer carries {row:?}: {table}"
        );
    }

    // The two appropriation years sum to their printed totals. The estimate year does not, and
    // is out by exactly a tenth of a million.
    for (year, rows, total) in [
        (2024, [397.8, 276.6, 135.5, 100.6, 54.0], 964.5),
        (2025, [439.1, 299.0, 142.3, 115.1, 55.4], 1_050.9),
    ] {
        let summed: f64 = rows.iter().sum();
        assert!(
            (summed - total).abs() < 0.05,
            "FY{year} sums to {summed}, not the printed {total}"
        );
    }

    let fy2023: f64 = [121.3, 228.2, 122.5, 77.9, 45.0].iter().sum();
    assert!((fy2023 - 594.9).abs() < 0.05, "FY2023 sums to {fy2023}");

    // Which of the two is the real column, and how the document answers it. LSC states the FY2024
    // increase as "$369.6 million (62.1%)" in the sentence above the table. That is the rise off
    // **594.9**, not off the 595.0 the table prints — so the printed total is a rounding of a
    // column that carries more precision, and the itemised rows are the ones to take a figure
    // from. The percentage does not discriminate: both readings round to 62.1%.
    let stated_rise: f64 = 369.6;
    assert!((964.5 - fy2023 - stated_rise).abs() < 0.05);
    assert!((964.5 - 595.0 - stated_rise).abs() > 0.05);
    assert!((stated_rise / fy2023 - 0.621).abs() < 0.0005);
    assert!((stated_rise / 595.0 - 0.621).abs() < 0.0005);

    let sentence = analysis.around("scholarship payments are estimated to total", 3);
    assert!(
        sentence.contains("$369.6 million (62.1%)"),
        "LSC's stated increase has moved: {sentence}"
    );

    // And the share of that rise this one programme is.
    assert!(((397.8 - 121.3) / stated_rise - 0.748).abs() < 0.001);
}

/// The act carried exactly one veto, and it is not a scholarship provision.
///
/// The greenbook's contents page lists one "Vetoed provision" section and the body carries one,
/// on JCARR review of the community school full-time-equivalency manual. That is the whole of
/// what `legislation/hb-33-2023` needed to close the veto half of its `[open]` — the answer being
/// that no scholarship provision was struck.
#[test]
fn the_only_vetoed_provision_is_not_about_a_scholarship() {
    let analysis = greenbook("hb33");

    // Trimmed equality rather than a substring match: the contents entry carries dot leaders
    // and a page number, and the body's first sentence says "vetoed provisions" in prose.
    let headings = analysis
        .lines_matching("Vetoed provision")
        .iter()
        .filter(|line| line.trim() == "Vetoed provision")
        .count();
    assert_eq!(
        headings, 1,
        "H.B. 33's greenbook no longer carries one veto section"
    );

    let veto = analysis.around("The Governor vetoed provisions that would have required", 6);
    assert!(
        veto.contains("Joint Committee on Agency"),
        "the veto passage has moved: {veto}"
    );
    assert!(
        veto.contains("full-time equivalency student enrollment reported"),
        "the veto passage has moved: {veto}"
    );
    for programme in [
        "scholarship",
        "Scholarship",
        "EdChoice",
        "Autism",
        "Cleveland",
    ] {
        assert!(
            !veto.contains(programme),
            "the veto passage names {programme:?}, which this test assumed it did not: {veto}"
        );
    }
}
