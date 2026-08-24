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
//! **The amounts did not.** Foundation aid was enacted $82,062,286 above the proposal, and the
//! movement is not spread evenly: the lottery line rose $97,638,202 while everything else in the
//! table fell $15,575,916. A corpus quoting the introduced column would have understated the
//! lottery's share of foundation aid and missed the substitution entirely.
//!
//! These tests hold both halves, because a future edition could move either.
//!
//! # Where the table is read
//!
//! [`project::budget_analysis`], which is the crate's one reader of these two fixtures. It used to
//! be three private ones, and two of them disagreed about how to anchor a section.

use project::budget_analysis::{
    enactment_movement, enactment_movement_off_the_lottery_line, foundation_aid, Edition,
    LOTTERY_LINE, TOTAL_FOUNDATION_AID,
};

#[test]
fn both_documents_name_the_same_five_lines_for_foundation_aid() {
    /*
     * The claim five programme nodes rest on. If the legislature had added or dropped a line at
     * enactment, every one of those nodes would be wrong about where its programme is paid from —
     * and the redbook would still say what it always said.
     */
    for edition in [Edition::Introduced, Edition::Enacted] {
        let doc = edition.text();
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
    let introduced = foundation_aid(Edition::Introduced, TOTAL_FOUNDATION_AID);
    let enacted = foundation_aid(Edition::Enacted, TOTAL_FOUNDATION_AID);

    assert_eq!(introduced.first, 11_147_995_271.0);
    assert_eq!(enacted.first, 11_230_057_557.0);
    assert_eq!(enacted.second, 11_525_218_197.0);
    assert_eq!(enactment_movement(TOTAL_FOUNDATION_AID), 82_062_286.0);
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
    assert_eq!(
        foundation_aid(Edition::Introduced, LOTTERY_LINE).first,
        1_338_945_000.0
    );
    assert_eq!(
        foundation_aid(Edition::Enacted, LOTTERY_LINE).first,
        1_436_583_202.0
    );
    assert_eq!(enactment_movement(LOTTERY_LINE), 97_638_202.0);
    assert_eq!(enactment_movement_off_the_lottery_line(), -15_575_916.0);

    // The whole of the argument in one comparison: the lottery moved more than the total did.
    assert!(enactment_movement(LOTTERY_LINE) > enactment_movement(TOTAL_FOUNDATION_AID));

    // And the lottery's share of foundation aid is higher as enacted than as proposed.
    let share = |edition| {
        foundation_aid(edition, LOTTERY_LINE).first
            / foundation_aid(edition, TOTAL_FOUNDATION_AID).first
    };
    assert!(share(Edition::Enacted) > share(Edition::Introduced));
}

#[test]
fn the_two_documents_are_easy_to_confuse_which_is_why_this_file_exists() {
    /*
     * Not a behavioural test — a statement of the hazard, held so it cannot quietly stop being
     * true. The documents differ in their column headings and almost nowhere else, so a figure
     * copied out of one is indistinguishable from a figure copied out of the other.
     */
    for edition in [Edition::Introduced, Edition::Enacted] {
        assert!(
            edition.text().contains(edition.column_heading()),
            "{edition:?} lost the label that says which document it is"
        );
    }
    assert!(
        !Edition::Enacted.text().contains("FY 2026        FY 2027\n                 Fund/ALI\n                                                  Estimate            Introduced"),
        "the greenbook now carries the redbook's column headings"
    );
}
