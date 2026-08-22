//! The appropriation behind a prorated program, and why that program prorates.
//!
//! The appropriation proration factor node recorded: "When each factor was last set, and against
//! which appropriation. Only the preschool one publishes a limit at all, and none publishes a
//! history. Answering it needs the budget bill's line items, which is `lsc-budget` — a connector
//! still blocked on PDFs."
//!
//! It is not blocked. The department's redbook names every appropriation line item and breaks each
//! into its earmarks with amounts, which is exactly the missing half.
//!
//! # As introduced, and the distinction is load-bearing
//!
//! LSC publishes redbooks only for the introduced bill. The **line item numbers** here are the
//! enacted ones — an ALI is not renumbered in conference — and the **amounts** are the executive
//! proposal. Every figure below is labelled accordingly, because the last phase nearly published a
//! House-passed supplement schedule as though it were law.

mod common;

const REDBOOK: &str = include_str!("../fixtures/dew-redbook.txt");

/// **Which line governs.** The two prorated programs sit in named ALIs.
#[test]
fn the_prorated_programs_have_appropriation_line_items() {
    let book = common::flat(REDBOOK);
    assert!(book.contains("GRF 200502 Pupil Transportation"));
    assert!(book.contains("GRF 200540 Special Education Enhancements"));
    assert!(book.contains("GRF 200550 Foundation Funding"));

    // And the proration authority is a condition on the appropriation, not a formula in statute —
    // which is why R.C. 3317.0213 does not mention it.
    assert!(
        book.contains("DEW must prorate the payments so as not to exceed the amount appropriated")
    );
}

/// **Preschool special education is the remainder of a line item, and that is why it prorates.**
///
/// ALI 200540 funds five earmarked purposes and preschool special education takes what is left.
/// A residual claimant is the thing that absorbs an appropriation's shortfall by construction, and
/// nothing in the department's calculator says so — the sheet shows a factor and a limit.
///
/// ```text
///   ALI 200540 earmark                              FY2025 est.   FY2026 intro   FY2027 intro
///   Special education at DD boards and institutions  $38,500,000   $33,945,594   $33,945,594
///   Parent mentoring programs                         $1,350,000    $1,350,000    $1,350,000
///   School psychology interns                         $3,000,000    $3,000,000    $3,000,000
///   Vocational rehabilitation services                $6,500,000            $0            $0
///   Secondary transition services                     $2,000,000    $1,000,000    $1,000,000
///   Remainder - preschool special education         $147,500,000  $153,976,832  $153,976,832
/// ```
#[test]
fn preschool_special_education_is_what_is_left_of_its_line_item() {
    let book = common::flat(REDBOOK);
    assert!(book.contains("Remainder – preschool special education"));
    assert!(book.contains("$147,500,000 $153,976,832 $153,976,832"));
    assert!(book.contains("GRF ALI 200540 total $198,850,000 $193,272,426"));
    // The five earmarks that come first.
    for earmark in [
        "Special education at DD boards and institutions",
        "Parent mentoring programs",
        "School psychology interns",
        "Vocational rehabilitation services",
        "Secondary transition services",
    ] {
        assert!(book.contains(earmark), "{earmark}");
    }
}

/// **The calculator's stated limit is the prior year's appropriation.**
///
/// The FY2027 calculator prints $147,500,000 beside its 0.96854448 preschool proration factor, and
/// the corpus reproduced a program total of $148,408,184 against it — $908,184 over its own limit.
/// The node's reading was "a factor calibrated against an earlier ADM vintage and never
/// recalibrated". The redbook shows something adjacent and sharper: **$147,500,000 is the FY2025
/// estimate.** The FY2026 and FY2027 columns are $153,976,832.
///
/// So the factor and the limit printed beside it are both carried over from the prior biennium. On
/// the introduced FY2027 appropriation the program at $148.4m is **$5.6m under**, and no proration
/// would arise at all. The node hedged that "a recalibration before payment is expected and this
/// may never reach a district"; the appropriation column is why.
///
/// Stated as the introduced figure. Whether the enacted appropriation held at $153,976,832 needs
/// the enacted appropriations document, which is not wired. [the enacted amount is open]
#[test]
fn the_stated_preschool_limit_is_the_prior_years_appropriation() {
    let book = common::flat(REDBOOK);
    // The column header that makes the $147.5m a FY2025 number rather than a FY2027 one.
    assert!(book.contains("FY 2025 Estimate FY 2026 Introduced FY 2027 Introduced"));

    let panel = project::panel::panel();
    let program: f64 = panel
        .iter()
        .map(|r| r.preschool_special_education.total)
        .sum();
    // Over the limit the calculator prints...
    assert!(program > 147_500_000.0);
    // ...and comfortably under the one the budget proposes for the year being modelled.
    assert!(program < 153_976_832.0);
    assert!(
        (153_976_832.0 - program) / 1e6 > 5.0,
        "headroom is ${:.1}m",
        (153_976_832.0 - program) / 1e6
    );
}

/// Transportation's rates are cost-derived, and the redbook says so in as many words.
///
/// `regime_diff`-adjacent finding, arrived at from R.C. 3317.0212 two phases ago by noticing the
/// section contains no dollar figure. The redbook states the same thing in prose, which is a
/// second independent source for a claim that started as an inference from an absence.
#[test]
fn the_redbook_confirms_transportation_is_paid_on_prior_year_costs() {
    assert!(common::flat(REDBOOK)
        .contains("Transportation funds are mostly allocated based on the prior year’s costs"));
}
