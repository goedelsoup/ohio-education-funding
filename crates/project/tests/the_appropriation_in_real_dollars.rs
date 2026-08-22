//! What the appropriation series says once the dollars are made comparable.
//!
//! Ohio's school funding argument is conducted in nominal totals, and nearly every biennium in
//! this period can truthfully be called a record investment. These tests pin what the same series
//! says in constant dollars, because the two statements point opposite ways and both are correct.

use edfund_core::FiscalYear;
use project::appropriations::{
    enacted_history, growth, is_tax_reimbursement, line_history, lines, reimbursements,
    TAX_REIMBURSEMENT,
};

/// The base year every figure here is denominated in: the last year the price index covers.
const BASE: FiscalYear = FiscalYear(2026);

/// The years that can be deflated — FY2027 is appropriated and the index does not reach it.
fn deflated(history: Vec<project::appropriations::Year>) -> Vec<project::appropriations::Year> {
    history.into_iter().filter(|y| y.real.is_some()).collect()
}

/// A named window of the series.
///
/// Every growth figure here states the years it is over, rather than taking the first and last of
/// whatever the series happens to hold. That was how these tests were written when the series
/// began at FY2014, and extending it back to FY2002 silently changed what they measured: the
/// formula line grows 3.9% in real terms over FY2014-FY2026 and falls 3.8% over FY2008-FY2026,
/// and both are true.
fn window(
    history: Vec<project::appropriations::Year>,
    from: u16,
    to: u16,
) -> Vec<project::appropriations::Year> {
    deflated(history)
        .into_iter()
        .filter(|y| y.fiscal_year >= from && y.fiscal_year <= to)
        .collect()
}

#[test]
fn the_department_grew_in_both_bases_once_the_reimbursements_are_out_of_both_ends() {
    /*
     * This test asserted a real-terms *fall* of about 8% and was wrong, because
     * `TAX_REIMBURSEMENT` was written from the FY2026-27 greenbook, where the class has two
     * members, and applied to a window whose other end had five. `200901 Property Tax Allocation
     * - Education` alone was $1.14 billion in FY2014, so the start of the window carried roughly
     * $1.65 billion the end did not.
     *
     * Corrected, the department's enacted appropriation rises 50.6% in cash and **7.5% in
     * constant dollars** between FY2014 and FY2026. The sign is the opposite of what this
     * repository reported for two commits.
     */
    let history = window(enacted_history(BASE), 2014, 2026);
    let (nominal, real, _) = growth(&history).expect("both endpoints deflate");
    assert!(nominal > 0.45, "nominal growth is {nominal}");
    assert!(
        real > 0.0,
        "the real-terms series falls again at {real}; the exclusion list may have lost a member \
         at one end of the window"
    );
    assert!(
        real > 0.04 && real < 0.11,
        "real growth is {real}, outside the band this finding holds"
    );
}

#[test]
fn every_reimbursement_line_is_excluded_at_both_ends_of_any_window() {
    // The guard for the defect above. The class is identified by what its titles say, and every
    // member must be in the exclusion list — otherwise a window spanning a renumbering compares a
    // total that includes them against one that does not.
    let named: std::collections::BTreeSet<String> = lines()
        .iter()
        .filter(|line| {
            let title = line.title.to_lowercase();
            title.contains("tax allocation")
                || title.contains("tax replacement")
                || title.contains("tax reimbursement")
                // `200906 Tangible Tax Exemption - Education` is a reimbursement and says
                // "exemption". This vocabulary had the same blind spot the exclusion list did, so
                // the test that was meant to guard the class could not see the member it was
                // missing. It was found by reading the session laws, where 200906 sits beside
                // 200901 in every table from FY1998.
                || title.contains("tax exemption")
        })
        .map(|line| line.line_item.clone())
        .collect();
    for item in &named {
        assert!(
            TAX_REIMBURSEMENT.contains(&item.as_str()),
            "line item {item} reimburses tax and is not excluded; a window crossing the years it \
             exists in would compare unlike totals"
        );
    }
    assert_eq!(named.len(), TAX_REIMBURSEMENT.len());
}

#[test]
fn the_formula_line_moved_the_other_way_from_the_department_around_it() {
    // Foundation Funding is the formula itself, and it is the one large line that gained ground:
    // +3.9% real while the department it sits in lost 7.9%. Whatever else the Fair School Funding
    // Plan did, it did not come out of a growing budget — it came out of a shrinking one.
    let foundation = window(line_history("200550", BASE), 2014, 2026);
    let (nominal, real, _) = growth(&foundation).expect("both endpoints deflate");
    assert!(
        nominal > 0.40,
        "the formula line's nominal growth is {nominal}"
    );
    assert!(
        real > 0.0,
        "Foundation Funding no longer grows in real terms; it is {real}"
    );

    let department = window(enacted_history(BASE), 2014, 2026);
    let (_, department_real, _) = growth(&department).expect("both endpoints deflate");
    // Both rise in real terms once the reimbursements are out of both ends; the formula's line
    // still rises less than the department around it, which is the opposite of what this test
    // asserted when the department appeared to be shrinking.
    assert!(
        department_real > 0.0 && real > 0.0,
        "one of them now falls: formula {real}, department {department_real}"
    );
}

#[test]
fn the_last_appropriated_year_has_no_real_figure_rather_than_a_nominal_one() {
    // FY2027 is enacted and the index stops at FY2026. Reporting its cash amount in a column
    // headed "constant dollars" is the failure `deflator` exists to prevent, so the year is
    // carried with `real: None` and every caller here filters on it.
    let history = enacted_history(BASE);
    let last = history.last().expect("the series is not empty");
    assert_eq!(last.fiscal_year, 2027);
    assert!(last.real.is_none(), "FY2027 acquired a real figure");
    assert!(last.nominal > 0.0, "FY2027 lost its nominal figure");
}

#[test]
fn the_tax_reimbursement_lines_are_excluded_and_are_not_small() {
    // They are numbered as the department's and are not its budget, on the publisher's own
    // statement. This asserts both halves: that they are out of the total, and that leaving them
    // in would have mattered.
    let total: f64 = lines()
        .iter()
        .filter(|line| line.kind == "enacted" && line.fiscal_year == 2026)
        .map(|line| line.amount)
        .sum();
    let department = enacted_history(BASE)
        .into_iter()
        .find(|year| year.fiscal_year == 2026)
        .expect("FY2026 is in the series");
    let excluded = total - department.nominal;
    assert!(
        excluded > 1_000_000_000.0,
        "the excluded reimbursement lines are only {excluded}; either they have shrunk or the \
         exclusion has stopped working"
    );
    assert_eq!(TAX_REIMBURSEMENT.len(), 7);
}

#[test]
fn the_long_view_says_something_different_from_the_short_one() {
    // What the extension back to FY2002 bought, and the reason every figure here names its years.
    // Over FY2014-FY2026 the formula's line gains 3.9% in real terms; over FY2008-FY2026 — the
    // whole span the same line item exists for — it loses ground. The Fair School Funding Plan
    // recovered something that had been given up earlier, which is a different claim from growth.
    let short = window(line_history("200550", BASE), 2014, 2026);
    let long = window(line_history("200550", BASE), 2008, 2026);
    let (_, short_real, _) = growth(&short).expect("both endpoints deflate");
    let (_, long_real, _) = growth(&long).expect("both endpoints deflate");
    assert!(
        short_real > 0.0,
        "the short window no longer gains: {short_real}"
    );
    assert!(
        long_real < 0.0,
        "the long window no longer loses: {long_real}"
    );
    assert!(
        long[0].fiscal_year == 2008,
        "200550 now reaches before FY2008 and this window has changed meaning"
    );
}

#[test]
fn the_real_terms_fall_is_in_lines_that_stopped_being_listed() {
    /*
     * The question `state-foundation-aid` carried open: which categorical lines absorbed the
     * department's 7.9% real-terms fall between FY2014 and FY2026.
     *
     * **None of them did.** Of the 61 lines appropriated in both years, the gains outweigh the
     * losses — roughly $1.08 billion gained against $0.55 billion lost, in FY2026 dollars. The
     * fall is not in surviving lines shrinking. It is in the department going from **112 lines to
     * 80**: 51 that existed in FY2014 do not exist in FY2026, and only 19 new ones appeared.
     *
     * **And this series cannot say whether that is a cut or a consolidation.** A line ceasing to
     * be listed may have been abolished or folded into another, and the two largest gains here —
     * `200550` and `200612`, both Foundation Funding - All Students, up about $320 million and
     * $350 million real — are exactly what a consolidation into the formula would look like.
     * Distinguishing them needs the acts' own language, which is the analysis half of
     * `lsc-budget` and is not held.
     */
    use project::appropriations::{changes, enacted_history, lines, TAX_REIMBURSEMENT};
    use std::collections::BTreeSet;

    let moved = changes(BASE, 2014, 2026);
    let lost: f64 = moved.iter().map(|c| c.shift()).filter(|s| *s < 0.0).sum();
    let gained: f64 = moved.iter().map(|c| c.shift()).filter(|s| *s > 0.0).sum();
    assert!(
        gained > lost.abs(),
        "surviving lines now lose more than they gain ({gained} against {lost}); the fall has \
         moved into them and the finding beside this test is stale"
    );

    let present = |year: u16| -> BTreeSet<String> {
        lines()
            .iter()
            .filter(|l| {
                l.kind == "enacted"
                    && l.fiscal_year == year
                    && !TAX_REIMBURSEMENT.contains(&l.line_item.as_str())
            })
            .map(|l| l.line_item.clone())
            .collect()
    };
    let (start, end) = (present(2014), present(2026));
    assert!(
        start.difference(&end).count() > 40,
        "the count of lines that stopped being listed has changed; it was 48"
    );

    let history = enacted_history(BASE);
    let count = |year: u16| {
        history
            .iter()
            .find(|y| y.fiscal_year == year)
            .expect("year")
            .items
    };
    assert!(
        count(2014) > count(2026) + 25,
        "the department no longer has far fewer appropriation lines than it did: {} against {}",
        count(2026),
        count(2014)
    );
}

/// The reimbursement lines are out of the department's total and reachable on their own.
///
/// Kept and tested rather than deleted: `reimbursements` had no caller, but the module doc
/// states the class is "excluded from every total here and available through [`reimbursements`]".
/// Half of that sentence is an exclusion the other tests lean on; the other half is this
/// function, and nothing checked that the two halves agree.
#[test]
fn the_reimbursements_are_excluded_from_the_total_and_retrievable_beside_it() {
    let carved_out = reimbursements(BASE);
    assert!(
        !carved_out.is_empty(),
        "the reimbursement class is non-empty in this series, so an empty result means the \
         line-item match stopped matching rather than that the money stopped flowing"
    );

    // Every year it reports is a year whose lines really are of the class.
    for year in &carved_out {
        assert!(
            year.nominal > 0.0,
            "FY{}: a reported year carries money",
            year.fiscal_year
        );
        assert!(
            year.items > 0,
            "FY{}: and the lines it is over",
            year.fiscal_year
        );
    }

    // The exclusion half. For a year the two series share, the department's enacted total and
    // the reimbursement total are drawn from disjoint sets of lines, so the enacted total plus
    // the reimbursements is strictly larger than the enacted total alone.
    let enacted = enacted_history(BASE);
    let shared = carved_out
        .iter()
        .find(|r| enacted.iter().any(|e| e.fiscal_year == r.fiscal_year))
        .expect("the two series overlap");
    let total = enacted
        .iter()
        .find(|e| e.fiscal_year == shared.fiscal_year)
        .expect("the year is in both");

    let counted_twice = lines()
        .into_iter()
        .filter(|l| {
            l.kind == "enacted"
                && l.fiscal_year == shared.fiscal_year
                && is_tax_reimbursement(&l.line_item, l.fiscal_year)
        })
        .count();
    assert_eq!(
        counted_twice, shared.items,
        "FY{}: the lines `reimbursements` reports are exactly the class",
        shared.fiscal_year
    );
    assert!(
        total.nominal > 0.0 && shared.nominal > 0.0,
        "FY{}: both series carry money in the year they share",
        shared.fiscal_year
    );
}
