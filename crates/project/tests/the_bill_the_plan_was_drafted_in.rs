//! Whether the open-enrolment-only denominator was the plan's choice or the budget's.
//!
//! `formula-component/fsfp-local-capacity-measure` asked three times and answered twice wrongly.
//! First that R.C. 3317.0217(C)(1) might predate the school-choice channels — refuted, its text is
//! H.B. 110's. Then that it was the surviving half of a matched pair with prior law's multiplier
//! netting — refuted, the two reach three channels and one and five are reached by neither.
//!
//! Both were inferences about drafting history read off the shape of the surviving text. An act
//! carries its provisions without their history, so no amount of reading it more carefully was
//! going to produce a third answer that was better than a document.
//!
//! H.B. 1 of the 134th is that document. It **enacts** R.C. 3317.0217 and **amends**
//! R.C. 3317.03, in one bill, and the denominator already reads exactly as it does in force
//! today. See [`project::plan_bill`] for why an un-enacted bill is committed and what it must not
//! be read as.

use project::plan_bill;
use project::statute;

/// The span of R.C. 3317.03(A)(2)'s own lettered list, from its stem to the next division.
///
/// `plan_bill::letters` walks while the letters stay consecutive, and the Revised Code nests
/// lettered lists — unbounded it leaves (A)(2) at `(j)` and counts on into (A)(3), returning
/// sixteen. Bounding it here rather than in the reader keeps the reader honest about what it does.
fn list_of(section: &str) -> &str {
    let start = section
        .find("following entities:")
        .expect("the (A)(2) list is introduced by its stem");
    let rest = &section[start..];
    let end = rest
        .find("(3) The department")
        .expect("and closed by the next division");
    &rest[..end]
}

/// The bill's own title says it enacts the section the question is about.
///
/// The title is the bill's self-description, and checking it first is what distinguishes "the
/// provision was written here" from "the provision is quoted here".
#[test]
fn the_bill_enacts_the_section_and_amends_the_one_it_cites() {
    let title = plan_bill::amending_title();
    assert!(
        title.contains("to enact new sections 3314.085, 3317.017, 3317.0215, 3317.0217"),
        "H.B. 1 enacts R.C. 3317.0217 rather than carrying it: {title}"
    );
    assert!(
        title.contains("3317.03,"),
        "and amends R.C. 3317.03, the section the denominator cites"
    );
}

/// The denominator in the bill is the denominator in force, word for word.
///
/// If these differed, the budget would be where the choice was made and the whole reading would
/// collapse. They do not differ.
#[test]
fn the_denominator_was_already_this_when_the_plan_was_introduced() {
    let drafted = plan_bill::section("3317.0217").expect("the bill enacts it");
    let quotient = "the district's enrolled ADM for that fiscal year - the students described in \
                    division (A)(1)(b) of section 3317.03 of the Revised Code + the students \
                    described in division (A)(2)(d) of section 3317.03 of the Revised Code";
    assert!(
        drafted.contains(quotient),
        "the bill's wealth-per-pupil quotient is not the one in force"
    );

    let in_force = statute::section("3317.0217")
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        in_force.contains(quotient),
        "and the section in force has moved away from it, which would make the bill evidence \
         about a provision that no longer exists"
    );
}

/// And the same bill writes the list that quotient cites, already at **ten** channels.
///
/// The list has not moved either: ten in the bill, the same ten in force. This corpus had been
/// saying nine since #384 — counted off a truncated print — and the figure reached node prose, a
/// test and an issue body before anyone set the two lists side by side. That is what this test
/// does, and it is why the count is asserted on both documents rather than one.
///
/// What the bill settles is the thing neither document says alone: the denominator and the list it
/// cites were written together, so the choice of one channel was the plan's own and not the
/// budget's.
#[test]
fn the_same_bill_enumerates_ten_channels_and_the_denominator_takes_one() {
    let listed = plan_bill::section("3317.03").expect("the bill amends it");
    let channels = plan_bill::letters(list_of(&listed));
    assert_eq!(
        channels,
        ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j'],
        "H.B. 1 enumerates {} channels at (A)(2)",
        channels.len()
    );

    let today = statute::section("3317.03")
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let now = plan_bill::letters(list_of(&today));
    assert_eq!(
        now,
        channels,
        "the bill enumerates {} channels and current law {}. The corpus quoted nine for months \
         from a truncated print of the section; if these two ever genuinely diverge, the claim \
         that the drafter chose one of ten needs the vintage attached to it.",
        channels.len(),
        now.len()
    );

    // And the channel the denominator adjusts for is the same letter in both.
    for (which, text) in [
        ("the bill", list_of(&listed)),
        ("in force", list_of(&today)),
    ] {
        assert!(
            text.contains(
                "(d) An adjacent or other school district under an open enrollment policy"
            ),
            "(d) is not open enrolment {which}, so the citation does not point where it did"
        );
    }
}
