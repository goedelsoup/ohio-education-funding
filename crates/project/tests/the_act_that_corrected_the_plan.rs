//! What Sub. H.B. 583 of the 134th General Assembly actually changed, read from the act.
//!
//! # The claim this replaces
//!
//! The corpus recorded, for four phases, that this act's contents were out of reach: *"a separate
//! PDF that no connector fetches, so the corpus can say that corrections were made … and cannot
//! say which provisions moved."* Every other act here comes through the legislature's own version
//! API, and so does this one — `ohio-session-laws`, version code `08_EN`. Nothing was blocking it
//! except the sentence saying it was.
//!
//! # Reading an amending act from `pdftotext` output, and the trap in it
//!
//! An enrolled act prints amended language as strike-and-insert, and the strike is typographic.
//! Flattened to text the two readings run together: this act's Section 265.210 prints *"the
//! section of this act H.B. 110 of the 134th General Assembly entitled"*, where "this act" is
//! struck and the bill reference replaces it. A reader who takes the run of words as operative
//! gets a sentence the legislature did not enact.
//!
//! `ohio-session-laws` already records that hazard for H.B. 650 of the 122nd, which is why that
//! act is deliberately unheld. The same hazard is in this fixture, so it is pinned here rather
//! than left for somebody to meet by surprise: the assertions below quote the *printed* run,
//! and any claim drawn from this file has to say which half of a strike-and-insert it read.

mod common;

use project::act::{self, HB583 as ACT};

/// The sections of the school funding chapter this act reopened.
///
/// Thirteen, and the list is the answer to "which provisions moved" printed on the act's own
/// first page — amended by SECTION 1 and repealed in their prior form by SECTION 2, which is how
/// an Ohio act replaces a section.
const AMENDED_3317: &[&str] = &[
    "3317.011",
    "3317.0110",
    "3317.014",
    "3317.016",
    "3317.017",
    "3317.019",
    "3317.02",
    "3317.0212",
    "3317.0215",
    "3317.024",
    "3317.051",
    "3317.064",
    "3317.25",
];

/// **Thirteen sections of R.C. 3317 are reprinted, and `3317.022` is not one of them.**
///
/// That is the shape of the correction: the act reworked the sections that *feed* core foundation
/// funding and left the section that *assembles* it alone. `3317.022` is cited by ten of the act's
/// own sections and is a heading in none of them.
///
/// # The count that was a band, and the noun that was wrong
///
/// This asserted `matches("section 3317.022 of the Revised Code").count() > 5` while the corpus
/// published *"it appears ten times as a cross-reference"*. Neither half survived measurement. The
/// canonical phrasing occurs **13** times and the number occurs **23** times in all, so "ten
/// times" was true of no occurrence count — what is ten is the number of the act's **sections**
/// that cite it. A lower bound of five stood behind a published figure and could not have said so;
/// that is the fourth convention in #158, and #226 is where it reached this file.
#[test]
fn it_reopened_thirteen_funding_sections_and_left_the_assembly_section_alone() {
    let headings = act::headings(ACT);
    for section in AMENDED_3317 {
        assert!(
            headings.contains(section),
            "{section} should be reprinted as a section heading"
        );
    }
    assert_eq!(
        headings.iter().filter(|h| h.starts_with("3317.")).count(),
        AMENDED_3317.len(),
        "the act reprints these sections of R.C. 3317 and no others"
    );
    // Four uncodified sections of H.B. 110, which the corpus states beside the thirteen.
    assert_eq!(headings.iter().filter(|h| h.starts_with("265.")).count(), 4);

    assert!(
        !act::reprints(ACT, "3317.022"),
        "3317.022 is cited by this act and not amended by it"
    );
    assert_eq!(
        act::sections_citing(ACT, "3317.022").len(),
        10,
        "and ten of the act's own sections cite it, which is what makes the absence a choice"
    );
}

/// **The first input-year freeze in the Fair School Funding Plan, one biennium in.**
///
/// The plan was enacted in 2021 and this act — passed the following June — already holds two of
/// its transportation inputs at FY2020 data for both years of the biennium. The corpus's whole
/// input-refresh-versus-freeze question is usually asked of base cost across biennia; it starts
/// here, on a smaller quantity, before the first biennium closed.
#[test]
fn it_froze_the_transportation_cost_inputs_at_fy2020_data() {
    assert!(common::flat(ACT).contains(
        "the statewide average cost per rider and statewide average cost per mile used to \
         calculate funding under section 3317.0212 of the Revised Code and payment in lieu of \
         transportation payment under section 3327.02 shall be based on data from fiscal year 2020"
    ));
}

/// **It ended the proration of EdChoice expansion scholarships, retroactively.**
///
/// A student whose award had been cut under division (E) of R.C. 3310.032 receives the full
/// amount from the act's effective date. This is a scholarship the corpus records as a deduction
/// against districts, so an act described as "technical" moved money in the channel the districts
/// are currently litigating over.
#[test]
fn it_made_prorated_expansion_scholarships_whole() {
    assert!(common::flat(ACT).contains(
        "whose scholarship amount was prorated under division (E) of that section as it existed \
         prior to that date shall, on and after that date, receive the full scholarship amount"
    ));
}

/// **Its whole appropriation is one federal line worth $2.4m in one year.**
///
/// Which is why it is invisible in an appropriation series: SECTION 9's table has one row, in the
/// federal fund group, and the fund-group total equals it. Everything this act did to state money
/// it did by amending H.B. 110's uncodified sections — it names appropriation items 200502 and
/// 200550 repeatedly and appropriates for neither.
///
/// The row itself is split by a page break — `Relief` lands after the two money columns — so the
/// assertion is on the pieces rather than on a phrase that only looks contiguous.
#[test]
fn its_only_appropriation_is_one_federal_line() {
    let act = common::flat(ACT);
    let start = act
        .find("SECTION 9. All items")
        .expect("the appropriating section");
    let end = act.find("SECTION 10.").expect("the section after it");
    let table = &act[start..end];

    assert!(table.contains("3HS0 200640 Federal Coronavirus School $2,415,201 $0"));
    assert!(table.contains("E TOTAL ALL BUDGET FUND GROUPS $2,415,201 $0"));
    assert!(
        !table.contains("General Revenue Fund"),
        "the table has one fund group and it is the federal one"
    );
    assert!(
        act.contains("appropriation item 200550, Foundation Funding - All Students"),
        "which it amends the earmarks of, without appropriating to it"
    );
}

/// **The strike-and-insert is legible as adjacent words, and reading it wrong is easy.**
///
/// Pinned so that the hazard is a test failure if the extraction ever changes shape, and so that
/// no node quotes the run of words as if the legislature had written it.
#[test]
fn the_struck_and_the_inserted_words_run_together_in_the_extract() {
    assert!(common::flat(ACT).contains(
        "the section of this act H.B. 110 of the 134th General Assembly entitled \
         \"FORMULA TRANSITION SUPPLEMENT.\""
    ));
}
