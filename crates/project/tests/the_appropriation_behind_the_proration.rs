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

use project::budget_analysis::{
    special_education_enhancements, Edition, ALI_200540_TOTAL, PRESCHOOL_REMAINDER,
};

/// The Catalog extract, which carries the same line item from a different publication.
const CATALOG: &str = include_str!("../fixtures/catalog-line-items.csv");

/// What the calculator states as the preschool limit — the FY2025 figure, carried into an FY2027
/// sheet. Pinned in `the_supplements_outside_the_formula`, restated here as the thing to compare
/// against rather than re-derived.
const CALCULATOR_LIMIT: f64 = 147_500_000.0;

/// The enacted earmark table for the ALI this file is about.
///
/// The section anchoring — six greenbook rows begin `Remainder`, and the heading appears in the
/// table of contents as well as over the table — lives on [`project::budget_analysis::row`], where
/// the two other files that read these fixtures now get it too. It used to live here, in one of
/// three private parsers that did not agree.
fn enacted(label: &str) -> project::budget_analysis::Row {
    special_education_enhancements(Edition::Enacted, label)
}

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
    let greenbook = Edition::Enacted.text();
    assert!(greenbook.contains("Special Education Enhancements (ALI 200540)"));
    assert!(
        greenbook.contains(Edition::Enacted.column_heading()),
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
    let remainder = enacted(PRESCHOOL_REMAINDER);
    // FY2025 actual, then the two enacted years, which are equal to each other.
    assert_eq!(remainder.first, remainder.second, "the two years diverged");
    assert_eq!(remainder.first, 153_976_832.0);

    let total = enacted(ALI_200540_TOTAL);
    assert_eq!(total.first, 193_272_426.0);
    assert_eq!(total.second, 193_272_426.0);
}

#[test]
fn the_catalog_confirms_the_line_from_a_different_publication() {
    /*
     * The reconciliation. The greenbook is a PDF narrative parsed for its earmark table; the
     * Catalog is a different PDF series parsed by column count. If either extractor were reading
     * the wrong column these would not agree, and they agree to the dollar on three figures.
     */
    let total = enacted(ALI_200540_TOTAL);
    assert_eq!(catalog_200540(2026, "appropriation"), Some(total.first));
    assert_eq!(catalog_200540(2027, "appropriation"), Some(total.second));
    // And the year the calculator's stale limit came from, as an actual rather than an estimate.
    assert_eq!(catalog_200540(2025, "actual"), Some(total.prior));
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
     *
     * # The program total is computed here rather than pinned
     *
     * It used to be a `const PROGRAM_AT_FACTOR: f64 = 148_408_184.0` copied from the sibling file
     * that derives it, and both differences below were checked against a thousand-dollar band. A
     * band is the wrong instrument for a figure the corpus publishes to the dollar: the two
     * numbers this file exists to state are $908,184 and $5,568,648, and a test that admits
     * anything from $908,000 to $908,999 is not standing behind either of them. See #158.
     */
    let program: f64 = project::panel::panel()
        .iter()
        .map(|record| record.preschool_special_education.total)
        .sum();
    let appropriation = enacted(PRESCHOOL_REMAINDER).second;

    let over = program - CALCULATOR_LIMIT;
    let under = appropriation - program;
    assert!(over > 0.0, "the stale reading no longer shows an overrun");
    assert!(
        under > 0.0,
        "the program now exceeds its real appropriation"
    );
    assert!((over - 908_183.76).abs() < 0.01, "{over:.2}");
    assert!((under - 5_568_648.24).abs() < 0.01, "{under:.2}");
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
     *
     * Which column belongs to which edition is now `Edition::prior_year_heading`, so the two
     * cannot be picked up interchangeably by reaching for `[0]`.
     */
    let estimate = |label| special_education_enhancements(Edition::Introduced, label).prior;
    assert_eq!(estimate(ALI_200540_TOTAL), 198_850_000.0);
    assert_eq!(estimate(PRESCHOOL_REMAINDER), 147_500_000.0);

    let line_fell = estimate(ALI_200540_TOTAL) - enacted(ALI_200540_TOTAL).first;
    let remainder_rose = enacted(PRESCHOOL_REMAINDER).first - estimate(PRESCHOOL_REMAINDER);
    // The earmarks ahead of preschool absorbed both movements: the line's fall and the
    // remainder's rise are both paid for out of them.
    let earmarks_fell = line_fell + remainder_rose;

    assert_eq!(
        line_fell, 5_577_574.0,
        "the line did not fall by what it did"
    );
    assert_eq!(remainder_rose, 6_476_832.0, "the remainder moved");
    assert_eq!(earmarks_fell, 12_054_406.0);
}
