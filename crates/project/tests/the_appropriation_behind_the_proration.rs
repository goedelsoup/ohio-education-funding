//! Whether the appropriation a proration divides by survived the legislature.
//!
//! [`the_supplements_outside_the_formula`](the_supplements_outside_the_formula.rs) established
//! that the FY2027 calculator states a preschool special education proration factor of 0.96854448
//! beside a limit of $147,500,000, and that at that factor the program totals $148,408,184 — over
//! its own stated limit by $908,184.
//!
//! The corpus's reading was that the limit is stale rather than the factor wrong: $147,500,000 is
//! the FY2025 figure, carried into an FY2027 sheet. Checking it needed the enacted appropriation,
//! and the corpus held only LSC's **redbook**, which analyses the bill *as introduced* — the
//! executive proposal, not the law.
//!
//! It now holds the **greenbook**, the same analysis as enacted, which sat at a sibling URL to the
//! redbook for four phases. These tests hold what it says.
//!
//! # Why two documents are checked and not one
//!
//! The greenbook gives the earmark split; the Catalog of Budget Line Items gives the line item
//! total from an entirely different publication, parsed by a different extractor. Either alone
//! would be a transcription. Together they are a reconciliation, and the line only reconciles if
//! both were read correctly.

/// The greenbook: the department's appropriations as enacted.
const GREENBOOK: &str = include_str!("../fixtures/dew-greenbook.txt");

/// The Catalog extract, which carries the same line item from a different publication.
const CATALOG: &str = include_str!("../fixtures/catalog-line-items.csv");

/// What the calculator states as the preschool limit — the FY2025 figure, carried into an FY2027
/// sheet. Pinned in `the_supplements_outside_the_formula`, restated here as the thing to compare
/// against rather than re-derived.
const CALCULATOR_LIMIT: f64 = 147_500_000.0;

/// What the program totals at the calculator's own stated factor. Same provenance.
const PROGRAM_AT_FACTOR: f64 = 148_408_184.0;

/// The dollar amounts on a greenbook row, left to right, searched only inside one ALI's section.
///
/// # Why the section matters
///
/// The greenbook has six rows beginning `Remainder –`, one per line item that has a residual
/// earmark: foundation aid twice, preschool special education, auxiliary services, assessments and
/// education management. A search for `Remainder` alone finds foundation aid's — an eight-billion
/// figure where a hundred-and-fifty-million one belongs — and the first version of this file did
/// exactly that. It failed loudly because the two years of an appropriation are equal and those
/// two were not, which is the only reason it did not pass with the wrong row.
///
/// So the anchor is the section heading, and the row is the first match after it.
fn greenbook_row(section: &str, label: &str) -> Vec<f64> {
    // `rfind`, not `find`: the heading appears twice, once in the table of contents with dot
    // leaders and once over the table. The contents entry carries no figures, so anchoring on it
    // walks forward into whatever line item happens to come next — which is how this file first
    // read foundation aid's eight-billion remainder as preschool special education's.
    let at = GREENBOOK
        .rfind(section)
        .unwrap_or_else(|| panic!("the greenbook has no section {section:?}"));
    GREENBOOK[at..]
        .lines()
        .find(|l| l.trim_start().starts_with(label))
        .unwrap_or_else(|| panic!("no row {label:?} under {section:?}"))
        .split_whitespace()
        .filter_map(|token| {
            token
                .strip_prefix('$')?
                .replace(',', "")
                .parse::<f64>()
                .ok()
        })
        .collect()
}

/// The ALI this file is about.
const SECTION: &str = "Special Education Enhancements (ALI 200540)";

fn catalog_200540(fiscal_year: u16, kind: &str) -> Option<f64> {
    CATALOG
        .lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split(',').map(str::trim).collect();
            (f.get(2)? == &"200540"
                && f.get(4)?.parse::<u16>().ok()? == fiscal_year
                && f.get(5)? == &kind)
                .then(|| f.get(6)?.parse::<f64>().ok())?
        })
        .last()
}

#[test]
fn the_greenbook_is_the_enacted_analysis_and_says_so() {
    // The distinction the corpus spent four phases quoting the wrong side of. A greenbook whose
    // columns said `Introduced` would be a redbook under another name.
    assert!(GREENBOOK.contains("Special Education Enhancements (ALI 200540)"));
    assert!(
        GREENBOOK.contains("Appropriation"),
        "the greenbook does not label an appropriation column"
    );
}

#[test]
fn the_remainder_held_from_introduction_to_enactment() {
    /*
     * The answer to the node's open question. The executive proposed $153,976,832 for the
     * preschool remainder and the enacted act carries the same figure — so the corpus's quoted
     * number was right, and it was right by luck rather than by evidence until now.
     */
    let remainder = greenbook_row(SECTION, "Remainder");
    assert_eq!(remainder.len(), 3, "the earmark table changed shape");
    // FY2025 actual, then the two enacted years, which are equal to each other.
    assert_eq!(remainder[1], remainder[2], "the two years diverged");
    assert_eq!(remainder[1], 153_976_832.0);

    let total = greenbook_row(SECTION, "GRF ALI 200540 total");
    assert_eq!(total[1], 193_272_426.0);
    assert_eq!(total[2], 193_272_426.0);
}

#[test]
fn the_catalog_confirms_the_line_from_a_different_publication() {
    /*
     * The reconciliation. The greenbook is a PDF narrative parsed for its earmark table; the
     * Catalog is a different PDF series parsed by column count. If either extractor were reading
     * the wrong column these would not agree, and they agree to the dollar on three figures.
     */
    let total = greenbook_row(SECTION, "GRF ALI 200540 total");
    assert_eq!(catalog_200540(2026, "appropriation"), Some(total[1]));
    assert_eq!(catalog_200540(2027, "appropriation"), Some(total[2]));
    // And the year the calculator's stale limit came from, as an actual rather than an estimate.
    assert_eq!(catalog_200540(2025, "actual"), Some(total[0]));
}

#[test]
fn the_proration_does_not_arise_against_the_appropriation_that_governs_it() {
    /*
     * The finding, as arithmetic. Against the calculator's own stale cell the program is over its
     * limit and a proration would bite; against the appropriation actually enacted for the year
     * being modelled it is five and a half million under, and none arises.
     *
     * Both comparisons are made because the first is what a reader of the calculator would
     * compute and the second is what is true.
     */
    let enacted = greenbook_row(SECTION, "Remainder")[2];

    let over = PROGRAM_AT_FACTOR - CALCULATOR_LIMIT;
    let under = enacted - PROGRAM_AT_FACTOR;
    assert!(over > 0.0, "the stale reading no longer shows an overrun");
    assert!(
        under > 0.0,
        "the program now exceeds its real appropriation"
    );
    assert!((908_000.0..909_000.0).contains(&over), "{over}");
    assert!((5_568_000.0..5_569_000.0).contains(&under), "{under}");
}

#[test]
fn the_residual_claimant_grew_on_a_line_that_was_cut() {
    /*
     * What "residual claimant" means, in one subtraction — and a comparison that has to be made
     * between two figures of the same kind.
     *
     * The two documents print FY2025 differently. The redbook, written before the year closed,
     * gives an **estimate**: $198,850,000 with a $147,500,000 remainder. The greenbook, written
     * after, gives the **actual**: $195,160,040 with $147,499,999. Only the first is a budget
     * figure comparable to an appropriation, and the first version of this test used the actual,
     * got $8.36m for a movement that is $12.05m, and failed. An actual against an appropriation is
     * the same error `the_catalog_line_item_series` refuses at the source.
     */
    const FY2025_ESTIMATE_LINE: f64 = 198_850_000.0;
    const FY2025_ESTIMATE_REMAINDER: f64 = 147_500_000.0;

    let line_fell = FY2025_ESTIMATE_LINE - greenbook_row(SECTION, "GRF ALI 200540 total")[1];
    let remainder_rose = greenbook_row(SECTION, "Remainder")[1] - FY2025_ESTIMATE_REMAINDER;
    // The earmarks ahead of preschool absorbed both movements: the line's fall and the
    // remainder's rise are both paid for out of them.
    let earmarks_fell = line_fell + remainder_rose;

    assert!(line_fell > 0.0, "the line did not fall");
    assert!(remainder_rose > 0.0, "the remainder did not rise");
    assert!(
        (5_577_000.0..5_578_000.0).contains(&line_fell),
        "{line_fell}"
    );
    assert!(
        (6_476_000.0..6_477_000.0).contains(&remainder_rose),
        "{remainder_rose}"
    );
    assert!(
        (12_054_000.0..12_055_000.0).contains(&earmarks_fell),
        "the earmarks ahead of preschool moved by {earmarks_fell}"
    );
}
