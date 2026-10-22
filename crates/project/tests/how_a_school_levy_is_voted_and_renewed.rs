//! How a school levy is voted and renewed, read from the sections that say so.
//!
//! The levy modality nodes under `revenue-stream/` sort a district's levies by the section that
//! grants them and the terms on which they end. Three readings in them were hypotheses before
//! this file:
//!
//! - that a school district's current expense and permanent improvement levies are R.C. 5705.19
//!   levies. They are not. R.C. 5705.19's first sentence excludes school districts, and a school
//!   levy is voted under R.C. 5705.21 *for* a purpose 5705.19 lists;
//! - that H.B. 96 struck "increase or" from both of R.C. 5705.21's renewal sentences. It did, and
//!   the version it replaced is committed beside the version in force;
//! - that R.C. 5705.25's "renew and increase ... regardless of the section" survives as a route
//!   around the strike. Its words survive; it is a section about when a renewal reaches the
//!   ballot, for every subdivision, and grants nothing.

use project::statute::{prior, section};

const LEVY_LIMITS: &str = include_str!("../fixtures/enacted-levy-limits.txt");

/// A school levy is not a 5705.19 levy: that section excludes school districts in its first
/// sentence, and 5705.21 is the grant that borrows its purposes.
#[test]
fn a_school_levy_is_granted_by_5705_21_for_a_purpose_5705_19_lists() {
    let general = section("5705.19");
    assert!(general
        .body
        .starts_with("This section does not apply to school districts"));
    assert!(general
        .body
        .contains("(A) For current expenses of the subdivision"));
    assert!(general
        .body
        .contains("(F) For the construction or acquisition of any specific permanent improvement"));
    // H.B. 96 did not amend it.
    assert!(general.legislation.contains("House Bill 110"));

    let school = section("5705.21").body;
    assert!(school.contains(
        "for one of the purposes specified in division (A), (D), (F), (H), or (DD) of section \
         5705.19 of the Revised Code, for general permanent improvements"
    ));
    // Current expense and general permanent improvement levies may run for a continuing period;
    // anything else is five years at most.
    assert!(school.contains(
        "The number of years may be any number not exceeding five or, if the levy is for current \
         expenses of the district or for general permanent improvements, for a continuing period \
         of time."
    ));
    // "General permanent improvements" are defined against 5705.19(F)'s single-bond-issue limit,
    // which is the difference between the two kinds of permanent improvement levy.
    assert!(school.contains(
        "\"general permanent improvements\" means permanent improvements without regard to the \
         limitation of division (F) of section 5705.19"
    ));
}

/// H.B. 96 struck "increase or" from both renewal sentences, effective January 1, 2026; the
/// version before it is H.B. 496's of April 9, 2025.
#[test]
fn hb96_struck_increase_from_both_renewal_sentences_of_5705_21() {
    let before = prior("5705.21", "4-9-2025");
    let now = section("5705.21");
    assert_eq!(before.effective, "April 9, 2025");
    assert!(before.legislation.contains("House Bill 496"));
    assert_eq!(now.effective, "January 1, 2026");
    assert!(now.legislation.contains("House Bill 96"));

    for (was, is) in [
        (
            "or to increase or decrease a single levy imposed under either such division",
            "or to decrease a single levy imposed under either such division",
        ),
        (
            "or to increase or decrease a single such existing levy, for the purpose of general \
             permanent improvements",
            "or to decrease a single such existing levy, for the purpose of general permanent \
             improvements",
        ),
        (
            "and the total rate may be increased, decreased, or remain the same",
            "and the total rate may be decreased or remain the same",
        ),
    ] {
        assert!(before.body.contains(was), "the prior version reads {was:?}");
        assert!(
            !now.body.contains(was),
            "the version in force still reads {was:?}"
        );
        assert!(now.body.contains(is), "the version in force reads {is:?}");
    }
    assert_eq!(project::levy::renew_and_increase_sentences_before_hb96(), 2);
    assert_eq!(project::levy::renew_and_increase_sentences_in_force(), 0);
}

/// The permanent improvement renewal sentence is keyed on the purpose, not on the section that
/// imposed the levy: any existing levy "for the purpose specified in division (F) of section
/// 5705.19" falls under it, and it lost "increase or" with the rest.
#[test]
fn the_permanent_improvement_renewal_reaches_a_levy_by_its_purpose() {
    let now = section("5705.21").body;
    assert!(now.contains(
        "If the board of education imposes one or more existing levies for the purpose \
         specified in division (F) of section 5705.19 of the Revised Code, the resolution may \
         propose to renew one or more of those existing levies, or to decrease a single such \
         existing levy, for the purpose of general permanent improvements."
    ));
    let before = prior("5705.21", "4-9-2025").body;
    assert!(before.contains(
        "If the board of education imposes one or more existing levies for the purpose \
         specified in division (F) of section 5705.19 of the Revised Code, the resolution may \
         propose to renew one or more of those existing levies, or to increase or decrease a \
         single such existing levy"
    ));
}

/// The replacement levy is gone from the reduction-factor section too: R.C. 319.301 as it reads
/// now does not use the word, and the analysis lists 319.301 among the sections the repeal of
/// R.C. 5705.192 touched.
#[test]
fn the_reduction_factor_section_no_longer_names_a_replacement() {
    let text = section("319.301").body.to_lowercase();
    assert!(!text.contains("replacement"));
    assert!(!text.contains("replace"));
    let analysis = LEVY_LIMITS.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(analysis.contains("(R.C. 319.301, 319.302, 523.06,"));
    assert!(analysis.contains(
        "A replacement levy extends the term of an existing levy at the same original millage \
         rate of the levy it is replacing."
    ));
}

/// R.C. 5705.25 kept its renew-and-increase words. They govern when a renewal reaches the
/// ballot and what the ballot says, for every subdivision; they grant no levy.
#[test]
fn the_election_section_kept_its_renew_and_increase_words_and_grants_nothing() {
    let before = prior("5705.25", "4-9-2025").body;
    let now = section("5705.25");
    assert_eq!(now.effective, "January 1, 2026");
    for words in [
        "a resolution to renew or to renew and increase or renew and decrease an existing levy, \
         regardless of the section of the Revised Code under which the tax was imposed, shall \
         not be placed on the ballot unless",
        "\"A renewal of ________ mills and an increase of ______ mills for each $1 of taxable \
         value to constitute a\"",
    ] {
        assert!(before.contains(words));
        assert!(
            now.body.contains(words),
            "5705.25 no longer reads {words:?}"
        );
    }
    // What H.B. 96 did change here: "market value" on the ballot, and one named levy that may
    // not be renewed and increased.
    assert!(before.contains("county auditor's appraised value"));
    assert!(!now.body.contains("county auditor's appraised value"));
    assert!(now.body.contains("county auditor's market value"));
    assert!(now
        .body
        .contains("(E) A tax levied under section 5705.2111 of the Revised Code shall not be renewed and increased."));
    // And it names 5705.21 among the levies whose renewals it times.
    assert!(now
        .body
        .contains("or under section 5705.21 or 5705.217 of the Revised Code"));
}

/// The bond levy: a vote on the securities and a tax outside the ten-mill limitation for as long
/// as they are outstanding, inside R.C. 133.06's limits.
#[test]
fn a_bond_levy_runs_for_the_debt_and_the_debt_has_a_limit() {
    let election = section("133.18").body;
    assert!(election.contains(
        "Declares the necessity of levying a tax outside the tax limitation to pay the debt \
         charges on the bonds and any anticipatory securities."
    ));
    assert!(election.contains(
        "with the levy and collection of a property tax outside the tax limitation during the \
         period the securities are outstanding"
    ));

    let limits = section("133.06");
    assert!(limits.body.contains(
        "(A) A school district shall not incur, without a vote of the electors, net indebtedness \
         that exceeds an amount equal to one-tenth of one per cent of its tax valuation"
    ));
    assert!(limits.body.contains(
        "(B) Except as provided in divisions (E), (F), and (I) of this section, a school \
         district shall not incur net indebtedness that exceeds an amount equal to nine per \
         cent of its tax valuation."
    ));
    assert!(limits.body.contains(
        "(I) A school district may incur net indebtedness by the issuance of securities in \
         accordance with the provisions of this chapter in excess of the limit specified in \
         division (B) or (C) of this section when necessary to raise the school district \
         portion of the basic project cost"
    ));
    assert!((project::levy::unvoted_limit() - 0.001).abs() < 1e-15);
    assert!((project::levy::total_limit() - 0.09).abs() < 1e-15);
}

/// The final analysis says what an effective date cannot: that the General Assembly overrode the
/// veto, and from which elections the limits apply.
#[test]
fn the_analysis_dates_the_limits_by_election_and_records_the_override() {
    let text = LEVY_LIMITS.split_whitespace().collect::<Vec<_>>().join(" ");
    for words in [
        "Additional limitations on school district levies (VETO OVERRIDDEN)",
        "both of which will apply to elections held on or after January 1, 2026",
        "The act prohibits school districts from levying any new emergency levy, substitute \
         levy, or combined school district income tax and current expense property tax levy.",
        "The act also prohibits school districts from proposing to renew and increase an \
         existing levy.",
        "a school district will only be allowed to renew a levy at its current effective rate \
         or at a lesser rate",
        "Replacement property tax levies (VETO OVERRIDDEN)",
        "5705.192, repealed",
        "eliminate the authority of political subdivisions to levy replacement property tax \
         levies, beginning with elections held on or after January 1, 2026",
        "20-mill floor limits (VETOED)",
    ] {
        let words = words.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            text.contains(&words),
            "the analysis no longer reads {words:?}"
        );
    }
}
