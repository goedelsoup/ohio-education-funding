//! Whether a draft's stated baseline is still the law, which nothing checked.
//!
//! `a_draft_cannot_hide_what_it_did_not_price` holds the other half of this: a provision the model
//! cannot run is counted rather than dropped. That gate is about the **proposal**. This one is
//! about the **baseline**, and the two failures are independent — a plank can price cleanly and be
//! measured against law that has since changed. That is the more dangerous of the two, because
//! nothing about the output looks wrong.
//!
//! # Both of this fixture's stale baselines were found by hand
//!
//! **The transportation floor** was stated at 29.17% after H.B. 96 had raised it to 50%. The
//! provision reads as a rise and prices as a $118.1m cut: the baseline moved past the proposal and
//! inverted its sign. See
//! [`the_floor_that_pays_the_wealthy_districts`](the_floor_that_pays_the_wealthy_districts.rs).
//!
//! **The scholarship hold-harmless** is stated against a deduction from the resident district. The
//! Revised Code describes that deduction in the past tense — "as that division existed prior to
//! September 30, 2021" — so the provision proposes to repeal something repealed four years ago.
//!
//! Neither was caught by a gate, and both sat inside provisions already marked unpriced, where
//! nobody was looking.
//!
//! # What this gate does
//!
//! Every provision names the committed text its baseline rests on, in one of three kinds, and
//! these tests resolve it. A `statute` phrase that stops matching is a section amended under a
//! draft that still cites it. A `superseded` phrase must *also* still match, because a repeal is
//! established by text the same way a rule is. `uncodified` is the only kind that quotes nothing,
//! it is counted, and it is one provision.

use std::collections::BTreeSet;

use project::drafts::{self, Anchor};
use project::statute;

/// The statute extract with runs of whitespace flattened, so a quote may wrap in the fixture.
///
/// The same tolerance the corpus's `as_written` bindings apply to prose, applied to law.
fn flattened() -> String {
    statute::FIXTURE
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn flatten(phrase: &str) -> String {
    phrase.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// **The gate.** Every quoted baseline resolves in the committed Revised Code.
///
/// This is the test that would have caught the transportation floor: `29.17 per cent` appears
/// nowhere in R.C. 3317.0212, and the phrase that does appear — `fifty per cent or the district's
/// state share percentage` — is the baseline the provision should have been stated against.
#[test]
fn every_quoted_baseline_resolves_in_the_committed_statute() {
    let text = flattened();
    let mut quoted = 0;
    for draft in drafts::drafts().into_values() {
        for provision in &draft.provisions {
            if !provision.anchor.is_quoted() {
                continue;
            }
            let phrase = flatten(provision.anchor.phrase());
            assert!(
                text.contains(&phrase),
                "{} provision {} anchors on {phrase:?}, which is not in the committed Revised Code",
                draft.slug,
                provision.ordinal
            );
            quoted += 1;
        }
    }
    assert_eq!(quoted, 7, "quoted baselines across every draft");
}

/// Every anchor that quotes nothing says what kind of law it rests on, and they are counted.
///
/// Temporary law is the real case: the guarantee lives in the uncodified sections of an
/// appropriations act, so there is no statutory text to cite and none can be invented. Counted
/// here so the count cannot grow quietly — an `uncodified` anchor is the escape hatch, and an
/// escape hatch nobody counts is how a fixture stops meaning anything.
///
/// # This was `only_the_guarantee_rests_on_law_that_is_not_in_the_revised_code`
///
/// It asserted the count was one, and that was true of a fixture holding one budget-act provision.
/// The two stages of H.B. 96 falsify it honestly rather than by drift: a bill's phase-in
/// percentages are set in `Section 265.220` and its bridge formula in `Section 265.235`, both
/// uncodified, and a draft that *is* a stage of a budget act rests on temporary law almost
/// everywhere. The escape hatch is still counted and every use still has to say why; what is no
/// longer asserted is that the guarantee is the only thing behind it.
#[test]
fn every_uncodified_baseline_names_the_kind_of_law_it_rests_on() {
    let unquoted: Vec<(String, u16, String)> = drafts::drafts()
        .into_values()
        .flat_map(|draft| {
            let slug = draft.slug.clone();
            draft.provisions.into_iter().filter_map(move |provision| {
                matches!(provision.anchor, Anchor::Uncodified(_)).then(|| {
                    (
                        slug.clone(),
                        provision.ordinal,
                        provision.anchor.phrase().to_string(),
                    )
                })
            })
        })
        .collect();

    assert_eq!(unquoted.len(), 9, "uncodified baselines: {unquoted:?}");
    for (slug, ordinal, why) in &unquoted {
        assert!(
            why.contains("temporary law") || why.contains("uncodified"),
            "{slug} provision {ordinal}: the reason should name what kind of law it is, \
             not just that it is absent: {why}"
        );
    }

    // The original case, still asserted individually: it is the only one that is not a stage of a
    // budget act, and the only one whose statute has been in force for two bienniums.
    let guarantee = unquoted
        .iter()
        .find(|(slug, _, _)| slug == "fund-the-plan-and-retire-the-guarantee")
        .expect("the guarantee provision is still here");
    assert_eq!(guarantee.1, 2);
    assert!(guarantee.2.contains("temporary law"));
}

/// The superseded one is superseded, and the Revised Code says so in its own words.
///
/// The quote is not evidence that the provision is wrong by absence — absence is a weak argument
/// over an extract that does not hold every section. It is the statute describing its own repeal
/// in the past tense, which is positive evidence and is why this kind still has to resolve.
#[test]
fn the_scholarship_baseline_is_refuted_by_the_statute_rather_than_by_silence() {
    let provision = drafts::drafts()
        .into_values()
        .find(|draft| draft.slug == "fund-the-plan-and-retire-the-guarantee")
        .expect("the draft is committed")
        .provisions
        .into_iter()
        .find(|provision| provision.ordinal == 5)
        .expect("provision 5");

    assert!(matches!(provision.anchor, Anchor::Superseded(_)));
    assert!(
        !provision.anchor.stands(),
        "a superseded baseline does not stand"
    );
    assert!(provision.authority.contains("3310.41"));

    // The phrase resolves, and it is the past tense that carries the finding.
    let phrase = flatten(provision.anchor.phrase());
    assert!(flattened().contains(&phrase));
    assert!(
        phrase.contains("existed prior to"),
        "the proof of a repeal is the statute's own past tense: {phrase}"
    );

    // And it is the only baseline in the fixture that does not stand.
    let fallen = drafts::drafts()
        .into_values()
        .flat_map(|draft| draft.provisions)
        .filter(|provision| !provision.anchor.stands())
        .count();
    assert_eq!(fallen, 1, "superseded baselines");
}

/// Pricing and standing are independent, which is the reason this gate exists separately.
///
/// Of the fifteen provisions across all five drafts, **four** price and eleven do not; one
/// baseline has fallen and fourteen stand. The two partitions do not line up, and a gate that
/// checked only the first would have passed the fixture in the state that shipped both defects.
///
/// The ratio moved when the two stages of H.B. 96 arrived, and in the direction the class
/// predicts: a real budget act is mostly provisions no lever here expresses. It has now moved
/// once in the other direction — the transportation floor became a lever, so provision 4 of
/// `fund-the-plan-and-retire-the-guarantee` prices where it did not. That is the count moving
/// because the model grew rather than because the fixture changed, which is the only reason it
/// should ever move that way.
#[test]
fn whether_a_provision_prices_says_nothing_about_whether_its_baseline_stands() {
    let provisions: Vec<_> = drafts::drafts()
        .into_values()
        .flat_map(|draft| draft.provisions)
        .collect();
    assert_eq!(provisions.len(), 16);

    let priced = provisions.iter().filter(|p| p.is_priced()).count();
    let standing = provisions.iter().filter(|p| p.anchor.stands()).count();
    assert_eq!((priced, standing), (5, 15));

    // The one that does not stand is one the model could not run either, which is exactly how it
    // stayed invisible: nobody re-reads a provision the tool already declines to cost.
    let fallen_and_unpriced = provisions
        .iter()
        .filter(|p| !p.anchor.stands() && !p.is_priced())
        .count();
    assert_eq!(fallen_and_unpriced, 1);
}

/// Every anchor is distinct where it should be, and shared where the provision is shared.
///
/// Three groups share deliberately. The base-cost reference year is the same clause in three
/// drafts and rests on the same statutory text. The phase-in percentages are the same uncodified
/// section in both stages of H.B. 96. And the five provisions of `Section 265.235` are one section
/// of one substitute, so they rest on one absence of statutory text rather than five.
#[test]
fn the_anchors_are_shared_only_where_the_provision_is() {
    let phrases: Vec<String> = drafts::drafts()
        .into_values()
        .flat_map(|draft| draft.provisions)
        .map(|provision| provision.anchor.phrase().to_string())
        .collect();
    let distinct: BTreeSet<&String> = phrases.iter().collect();
    assert_eq!(phrases.len(), 16);
    assert_eq!(
        distinct.len(),
        9,
        "three anchors are shared by more than one provision"
    );

    let shared = phrases
        .iter()
        .filter(|phrase| phrases.iter().filter(|other| other == phrase).count() > 1)
        .count();
    assert_eq!(
        shared, 10,
        "the base cost reference year in three drafts, the phase-in section in two, \
         and Section 265.235 across five provisions of one"
    );
}
