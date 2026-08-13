//! The same table in two documents, and which half of it moved.
//!
//! LSC publishes its analysis of a budget bill twice: a **redbook** for the bill as introduced,
//! whose amounts are the executive proposal, and a **greenbook** for the bill as enacted. The two
//! are structurally identical — same categories, same headings, same sentences — which is what
//! makes the redbook so easy to quote as though it were the law.
//!
//! This corpus did quote it that way. Six claims across five programme nodes and two revenue-stream
//! documents cited the redbook, and two of them carried figures — $1.34 billion of an $11.15
//! billion foundation aid total — that are the proposal and not the appropriation.
//!
//! # What the sweep found
//!
//! **The structural claims held.** The five line items that collectively fund foundation aid and
//! the scholarship programmes are the same five in both documents, and the sentence naming them
//! survives enactment nearly word for word.
//!
//! **The amounts did not.** Foundation aid was enacted $82 million above the proposal, and the
//! movement is not spread evenly: the lottery line rose $97.6 million while everything else in the
//! table fell $15.6 million. A corpus quoting the introduced column would have understated the
//! lottery's share of foundation aid and missed the substitution entirely.
//!
//! These tests hold both halves, because a future edition could move either.

/// The bill as introduced. Amounts are the executive proposal.
const REDBOOK: &str = include_str!("../fixtures/dew-redbook.txt");

/// The same analysis as enacted.
const GREENBOOK: &str = include_str!("../fixtures/dew-greenbook.txt");

/// The dollar amounts on the first row starting with `label` after `section`, left to right.
fn row(doc: &str, section: &str, label: &str) -> Vec<f64> {
    let at = doc
        .find(section)
        .unwrap_or_else(|| panic!("no section {section:?}"));
    doc[at..]
        .lines()
        .find(|l| l.trim_start().starts_with(label))
        .unwrap_or_else(|| panic!("no row {label:?} under {section:?}"))
        .split_whitespace()
        .filter_map(|t| t.strip_prefix('$')?.replace(',', "").parse::<f64>().ok())
        .collect()
}

const TABLE: &str = "Foundation Aid Appropriations";

#[test]
fn both_documents_name_the_same_five_lines_for_foundation_aid() {
    /*
     * The claim five programme nodes rest on. If the legislature had added or dropped a line at
     * enactment, every one of those nodes would be wrong about where its programme is paid from —
     * and the redbook would still say what it always said.
     */
    for doc in [REDBOOK, GREENBOOK] {
        assert!(doc.contains("ALIs 200550, 200502, 200604, 200491,"));
        assert!(doc.contains("Lottery Fund 7017 ALI 200612"));
        assert!(doc.contains(
            "support state foundation aid payments for\nall public school students in the state \
             and scholarship payments for students enrolled in state\nscholarship programs"
        ));
    }
}

#[test]
fn the_enacted_foundation_aid_total_is_above_the_proposal() {
    /*
     * The figure two corpus documents had wrong. $11.15 billion is the introduced column and
     * $11.23 billion is the appropriation, and nothing in the redbook says which of the two a
     * reader is looking at except a column heading three lines above.
     */
    let introduced = row(REDBOOK, TABLE, "Total foundation aid");
    let enacted = row(GREENBOOK, TABLE, "Total foundation aid");
    assert_eq!(introduced.len(), 3, "the redbook table changed shape");
    assert_eq!(enacted.len(), 3, "the greenbook table changed shape");

    // Column 1 is FY2026 in both — an estimate/actual sits in column 0.
    assert!(
        enacted[1] > introduced[1],
        "FY2026 was enacted at {} against a proposal of {}",
        enacted[1],
        introduced[1]
    );
    let moved = enacted[1] - introduced[1];
    assert!((82_000_000.0..83_000_000.0).contains(&moved), "{moved}");
}

#[test]
fn the_lottery_line_rose_by_more_than_the_total_did() {
    /*
     * The finding the correction produced, and the reason it strengthens the lottery substitution
     * argument rather than weakening it. Between the proposal and the act the lottery's
     * contribution to foundation aid rose and the rest of the table fell — so the General Assembly
     * put in more lottery money and slightly less of everything else, which is what substitution
     * looks like when it is visible at all.
     */
    let intro_total = row(REDBOOK, TABLE, "Total foundation aid")[1];
    let enacted_total = row(GREENBOOK, TABLE, "Total foundation aid")[1];
    let intro_lottery = row(REDBOOK, TABLE, "Fund 7017 ALI 200612")[1];
    let enacted_lottery = row(GREENBOOK, TABLE, "Fund 7017 ALI 200612")[1];

    let lottery_rose = enacted_lottery - intro_lottery;
    let total_rose = enacted_total - intro_total;
    assert!(lottery_rose > total_rose, "the substitution reversed");

    let rest_moved = (enacted_total - enacted_lottery) - (intro_total - intro_lottery);
    assert!(
        rest_moved < 0.0,
        "everything else did not fall: {rest_moved}"
    );
    assert!(
        (-15_600_000.0..-15_500_000.0).contains(&rest_moved),
        "{rest_moved}"
    );

    // And the lottery's share of foundation aid is higher as enacted than as proposed.
    assert!(enacted_lottery / enacted_total > intro_lottery / intro_total);
}

#[test]
fn the_two_documents_are_easy_to_confuse_which_is_why_this_file_exists() {
    /*
     * Not a behavioural test — a statement of the hazard, held so it cannot quietly stop being
     * true. The documents differ in their column headings and almost nowhere else, so a figure
     * copied out of one is indistinguishable from a figure copied out of the other.
     */
    assert!(REDBOOK.contains("Introduced"), "the redbook lost its label");
    assert!(
        GREENBOOK.contains("Appropriation"),
        "the greenbook lost its label"
    );
    assert!(
        !GREENBOOK.contains("FY 2026        FY 2027\n                 Fund/ALI\n                                                  Estimate            Introduced"),
        "the greenbook now carries the redbook's column headings"
    );
}
