//! The one input to the EdChoice designation this repository does not derive, and what it is
//! worth.
//!
//! `the_designation_rebuilt_from_its_inputs` reproduces every flag in the 2026-2027 designated
//! list from the raw counts in all 2,877 rows, with one exception it states in its own doc
//! comment: `academic_distress_school`, the Option A route, is taken from the workbook on trust.
//! It is not an arithmetic on any sheet in the file. R.C. 3310.03(C) makes a student eligible "if
//! the student's resident district is subject to section 3302.10", so the column is a fact about
//! whether a commission exists, and no count can produce it.
//!
//! # The column is maintained, and it is dated
//!
//! Three districts have ever held a commission under the current R.C. 3302.10. Two are flagged
//! here and one is not, and the one that is not is the control:
//!
//! - **Lorain City**, IRN `044263`, fifteen buildings, flag `no`. H.B. 33 dissolved its commission
//!   on 4 July 2023 and the column dropped it. So the department does maintain this flag.
//! - **East Cleveland City**, IRN `043901`, five buildings, flag `yes`.
//! - **Youngstown City**, IRN `045161`, thirteen buildings, flag `yes`.
//!
//! The workbook's own `docProps/core.xml` dates it to [`designated::AUTHORED`] — 13 November 2025,
//! created and last modified thirty-four minutes apart. On that date all three flags were right.
//!
//! **Six weeks later East Cleveland's stopped being right.** In late December 2025 the director of
//! education and workforce wrote to the district's superintendent that it "has been released from
//! Academic Distress Commission oversight", having met 16 of 20 benchmarks under its Revitalization
//! Plan with an overall three-star rating. R.C. 3302.10(N)(1) attaches the consequence: "Upon
//! completion of the transition period, the chief executive officer shall relinquish all
//! operational, managerial, and instructional control of the district ... and the academic distress
//! commission shall cease to exist." Release is not a departmental courtesy that leaves the
//! commission standing — the statute ends it with the transition. By January 2026 the department
//! was describing Youngstown as the sole district in Ohio under academic distress.
//!
//! R.C. 3310.03(E)(2) then says the department "shall cease awarding first-time scholarships
//! pursuant to division (C) ... when the academic distress commission established for the district
//! ceases to exist", with (E)(3) preserving prior-year holders. So the file asserts an Option A
//! eligibility for 2026-2027 that the statute had already withdrawn before that school year opened
//! — and the file at the department's URL is still byte-identical to the 13 November 2025 edition.
//!
//! # What that is worth, which is the reason to measure it rather than assert it
//!
//! Almost nothing, and the point of this file is to say so precisely instead of leaving Option A
//! described as merely "read rather than derived".
//!
//! **No designation moves.** All five East Cleveland buildings also satisfy Option B — bottom
//! twenty per cent in two of three rankings, in a district whose three-year Title I average is
//! 48.9% — so they are designated on the derived route whatever the read one says. Dropping the
//! stale flag changes zero rows of `designated`.
//!
//! **The whole load-bearing surface of the unreproduced column is two buildings**, both
//! Youngstown's: Rayen Early College High School and Rayen Early College Middle School, neither of
//! which is in the bottom fifth in two of three years. They rest on Option A alone, and
//! Youngstown's commission is the one that still exists.
//!
//! So the flag is a current fact carrying a stale case, and the stale case is inert.
//!
//! # Why reproducing the trigger would not have closed this
//!
//! R.C. 3302.10(A)(1) establishes a commission after three consecutive years of an overall rating
//! below two stars. That is a rule about *establishment*, and this column is about *existence* —
//! two different predicates, separated by divisions (N) and (O) and by an act of the General
//! Assembly. East Cleveland is the proof already in hand: it was released on an overall rating of
//! **three stars**, far above the trigger, and was flagged `yes` all the same. Lorain left by
//! legislation rather than by rating at all. A district can satisfy the trigger and have no
//! commission, or hold one under division (A)(2)'s pre-2015 clause and never have satisfied the
//! trigger — which is Cleveland's case, and is why Cleveland is on no commission list.
//!
//! Computing the trigger would therefore have produced a set that is not this column and could not
//! audit it. It also cannot be computed from what is held: it needs the **overall** district rating
//! for three consecutive years, and the corpus holds one year of the *Achievement component* star
//! and no overall rating in any of the six cached report card downloads.

use dispersion::designated::{
    self, ACADEMIC_DISTRESS_DISTRICT_IRNS, EAST_CLEVELAND_IRN, LORAIN_IRN, TITLE1_THRESHOLD,
    YOUNGSTOWN_IRN,
};
use std::collections::BTreeSet;

/// Exactly two districts carry the flag, and they are the two whose commissions were alive when
/// the workbook was written.
#[test]
fn the_flag_names_two_districts_and_they_are_the_two_with_commissions_at_authorship() {
    let flagged: BTreeSet<String> = designated::buildings()
        .into_iter()
        .filter(|b| b.academic_distress)
        .map(|b| b.district_irn)
        .collect();

    assert_eq!(
        flagged.iter().map(String::as_str).collect::<Vec<&str>>(),
        ACADEMIC_DISTRESS_DISTRICT_IRNS,
    );
    assert_eq!(designated::AUTHORED, "2025-11-13");
}

/// Lorain is the control: the department dropped this flag once, so the column is maintained.
///
/// Fifteen buildings, every one designated and every one on Option B alone. If the flag were
/// carried forward from edition to edition rather than recomputed, Lorain would still be in it —
/// its commission was dissolved by H.B. 33 on 4 July 2023, two editions back.
#[test]
fn lorain_lost_the_flag_when_its_commission_was_dissolved() {
    let lorain: Vec<_> = designated::buildings()
        .into_iter()
        .filter(|b| b.district_irn == LORAIN_IRN)
        .collect();

    assert_eq!(lorain.len(), 15);
    assert!(lorain.iter().all(|b| b.district_name == "Lorain City"));
    assert!(lorain.iter().all(|b| !b.academic_distress && !b.option_a));
    assert!(lorain.iter().all(|b| b.designated && b.option_b));
}

/// The IRN guard. `047076` is Pettisville Local, and reading it as Lorain inverts every fact above.
///
/// Two buildings rather than fifteen, a Title I average of 6.8% rather than 35.0%, and nothing
/// designated. District names are not unique in Ohio and IRNs are, which is the only reason this
/// is checkable at all.
#[test]
fn the_irn_that_is_not_lorain() {
    let pettisville: Vec<_> = designated::buildings()
        .into_iter()
        .filter(|b| b.district_irn == "047076")
        .collect();

    assert_eq!(pettisville.len(), 2);
    assert!(pettisville
        .iter()
        .all(|b| b.district_name == "Pettisville Local"));
    assert!(pettisville.iter().all(|b| !b.designated));
    assert_ne!("047076", LORAIN_IRN);
}

/// East Cleveland's flag is the stale one, and it moves no designation.
///
/// Five buildings, all flagged, all also on Option B, all designated either way. This is the
/// measurement that bounds what the release in late December 2025 costs the file: nothing in the
/// `designated` column.
#[test]
fn east_clevelands_stale_flag_changes_no_designation() {
    let east_cleveland: Vec<_> = designated::buildings()
        .into_iter()
        .filter(|b| b.district_irn == EAST_CLEVELAND_IRN)
        .collect();

    assert_eq!(east_cleveland.len(), 5);
    assert!(east_cleveland.iter().all(|b| b.academic_distress));

    for building in &east_cleveland {
        assert!(
            building.option_b,
            "{} rests on the released commission alone",
            building.building_name
        );
        assert_eq!(
            building.designated, building.option_b,
            "{} would change if Option A were dropped",
            building.building_name
        );
        assert!(building.bottom_20_two_of_three);
        assert!(building.title1_average >= TITLE1_THRESHOLD);
    }
}

/// The whole exposure of the unreproduced column: two buildings, both in the district whose
/// commission still exists.
#[test]
fn the_two_buildings_that_rest_on_the_read_column_alone_are_youngstowns() {
    let alone: Vec<_> = designated::buildings()
        .into_iter()
        .filter(|b| b.option_a && !b.option_b)
        .collect();

    assert_eq!(alone.len(), 2);
    assert!(alone.iter().all(|b| b.district_irn == YOUNGSTOWN_IRN));
    assert!(alone
        .iter()
        .all(|b| b.designated && !b.bottom_20_two_of_three));
    assert_eq!(
        alone
            .iter()
            .map(|b| b.building_irn.as_str())
            .collect::<Vec<&str>>(),
        ["000520", "011493"],
    );
}

/// Dropping the released district's flag entirely leaves the designated count where it was.
///
/// The one-line statement of the two tests above: recompute the designation with Option A honoured
/// only for the commission that still exists, and 513 buildings are still designated.
#[test]
fn the_designated_count_survives_honouring_only_the_surviving_commission() {
    let all = designated::buildings();
    let published = all.iter().filter(|b| b.designated).count();
    let corrected = all
        .iter()
        .filter(|b| (b.option_a && b.district_irn == YOUNGSTOWN_IRN) || b.option_b)
        .count();

    assert_eq!(published, 513);
    assert_eq!(corrected, published);
}
