//! Whether the General Assembly was told that a district's categoricals are discounted by a count
//! excluding the children it has lost, while the schools those children went to pay no local share.
//!
//! `formula-component/fsfp-local-capacity-measure` left that `[open]` on the ground that "nothing
//! in R.C. 3317.022 says either way, and the two treatments are one division apart". Both halves
//! of that are wrong, and the documents that refute them were already committed.
//!
//! # The two treatments are not symmetric unknowns
//!
//! The **exemption** is written out nine times. R.C. 3317.022 does not state it once in a general
//! provision; it repeats "if the funding unit is a city, local, or exempted village school
//! district, the district's state share percentage" inside each of the six special education
//! categories and each of the three English learner ones. That is the most explicit form drafting
//! offers, and LSC restated it to legislators in three consecutive bienniums with a reason
//! attached — community and STEM schools have no taxing authority.
//!
//! The **denominator** is stated once, as a definition, and never connected to the percentage it
//! sets. H.B. 110's analysis says the budget "replaces 'Formula ADM' with 'Enrolled ADM' which
//! counts the number of students being educated in a district or school", against a prior-law
//! count that began with students *residing* in the district. Ten pages later the same document
//! says the state share percentage "is used in the calculation of various categorical components".
//! Nothing in any of the fourteen analyses joins the two sentences.
//!
//! # And the General Assembly built an instrument against it, then repealed it
//!
//! H.B. 110 created supplemental targeted assistance for 36 districts whose enrolled ADM was under
//! 88% of their total residential ADM, $56.4m in FY2022. H.B. 96 eliminated it, and stated the
//! corpus's own hypothesis as the thing being eliminated: those districts "might appear relatively
//! wealthier on a per-pupil basis because of changes in how the formula in place since FY 2022
//! counts students relative to the previous formula and thus receive less state aid".
//!
//! So the question is answered in the direction opposite to the one the node supposed. This was
//! not unnoticed. It was noticed, priced at $52.5m, and the price was struck in the biennium this
//! corpus models — see [`project::capacity_denominator::against_the_repealed_supplement`] for what
//! a denominator correction confined to the categoricals would give the same 36 districts.
//!
//! # The adjustment H.B. 110 kept was half of a pair
//!
//! A later reading of this node asked why H.B. 110 reconstructed one of the ten channels of
//! R.C. 3317.03(A)(2) and not the other eight. That supposes a partial residence count, and prior
//! law had a complete one: **formula ADM was total ADM less 80% of JVSD ADM**, where total ADM is
//! *"the number of all students who reside in the district"*, and the single carve-out is there
//! because JVSDs are paid by a separate formula rather than by deduction.
//!
//! The choice channels were corrected on the **multiplier** — *"an adjustment is made to the
//! formula ADM of each district so as to not credit the district with targeted assistance for
//! students educated through these programs"* — and open enrolment is the one channel that list
//! omits. It is the one R.C. 3317.0217(C)(1) adjusts for. So the two corrections sat on opposite
//! sides of the same product and H.B. 110 kept the half that had no home on the payment side,
//! at the point the deduct architecture hosting the other was abolished.
//!
//! Whether that is construction or coincidence is #399. What these tests settle is only that
//! "incomplete reconstruction" is not available as a reading.
//!
//! # A hazard, because the name was reused
//!
//! "Supplemental targeted assistance" under H.B. 64 was an **agricultural** payment: a district's
//! agricultural real property percentage less 10%, times 40% of the formula amount. H.B. 110's
//! payment of the same name is keyed on residence against attendance and reaches, in LSC's words,
//! "primarily lower wealth, urban districts". Tracing the name back through the greenbooks finds a
//! rural programme and invites exactly the wrong conclusion. The same trap the corpus already
//! records for R.C. 3317.0217, which was parity aid in 2001.

use project::greenbook::greenbook;
use project::ledger::budget_analysis::{Edition, GREENBOOK};
use project::statute;

/// LSC's FY2026-27 analysis, flattened. Committed separately from the twelve, so `greenbook`
/// cannot find it — see `ledger::budget_analysis`.
fn hb96() -> String {
    GREENBOOK.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The enacting analysis states both halves of the asymmetry, ten pages apart.
///
/// Page 11 attaches the percentage to the categoricals; page 21 says the same categoricals are
/// computed for community and STEM schools "except that a state share percentage is not applied".
/// A legislator reading the document straight through was shown the differential twice.
#[test]
fn the_enacting_analysis_states_the_exemption_in_as_many_words() {
    let hb110 = greenbook("hb110").flat();

    assert!(
        hb110.contains(
            "This percentage is used in the calculation of various categorical components"
        ),
        "H.B. 110's greenbook attaches the state share percentage to the categoricals"
    );
    assert!(
        hb110.contains(
            "the calculations are the same as those for traditional districts except that a state \
             share percentage is not applied"
        ),
        "and exempts the community and STEM school unit from it in the same document"
    );
}

/// Three consecutive bienniums restate the exemption, and two give the reason.
///
/// The reason is about the recipient, not about the district: these schools have no taxing
/// authority, so there is no local share to take. It answers "why is the community school charged
/// nothing" completely and says nothing whatever about which children the district's percentage is
/// divided by.
#[test]
fn every_biennium_since_gives_the_same_reason_and_it_is_about_taxing_authority() {
    let hb33 = greenbook("hb33").flat();
    assert!(
        hb33.contains(
            "since these schools do not have taxing authority, the state provides all of the base \
             cost and other applicable formula components to them"
        ),
        "H.B. 33 restates it with the reason"
    );
    assert!(
        hb96().contains(
            "since these schools do not have taxing authority, state funding for them does not \
             account for any local support. That is, the state share for community and STEM \
             schools is effectively 100%"
        ),
        "and so does H.B. 96, for the biennium this corpus models"
    );
}

/// The section repeats the conditional nine times rather than stating it once.
///
/// Six special education categories under R.C. 3317.022(A)(3)(a) and three English learner ones
/// under (A)(5)(a). A drafter who writes a qualifier nine times has not left a question open.
#[test]
fn the_statute_writes_the_exemption_out_nine_times() {
    let flat = statute::section("3317.022")
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    let conditional = "if the funding unit is a city, local, or exempted village school district, \
                       the district's state share percentage";
    let occurrences = flat.matches(&conditional.to_lowercase()).count();
    assert_eq!(
        occurrences, 9,
        "the conditional appears {occurrences} times. Nine is six special education categories \
         plus three English learner ones, each writing the exemption out in full; a different \
         count means the section has been restructured and the claim that it is deliberate rests \
         on a shape that is no longer there."
    );
}

/// What the budget replaced, in LSC's own sentence: a residence count became an attendance count.
///
/// This is the whole of what any analysis says about the denominator, and it is said in the ADM
/// section rather than beside the percentage the count now sets. The base cost enrolled ADM
/// definition LSC does give explains only the *vintage* — a three-year maximum, "to smooth out the
/// base cost calculation for districts with declining enrolled ADM" — and not which children.
#[test]
fn the_denominator_change_was_stated_as_a_definition_and_never_connected() {
    let hb110 = greenbook("hb110").flat();

    assert!(
        hb110.contains(
            "The budget replaces \u{201c}Formula ADM\u{201d} with \u{201c}Enrolled ADM\u{201d} \
             which counts the number of students being educated in a district or school"
        ),
        "the switch from residence to attendance is stated once, as a definitional replacement"
    );
    assert!(
        hb110.contains("of students residing in a district minus 80%"),
        "against a prior-law count that began with residence"
    );
    assert!(
        hb110.contains(
            "This smooths out the base cost calculation for districts with declining enrolled ADM"
        ),
        "and the only reason LSC gives for the denominator is the three-year maximum, which is \
         about the vintage of the count and not about which children are in it"
    );
}

/// H.B. 110 built a payment against the effect; H.B. 96 struck it and said what it was for.
///
/// The repeal paragraph is the closest any document comes to the node's `[open]`: it names the
/// mechanism — the formula counts students differently — names the population, and prices it.
#[test]
fn the_general_assembly_priced_this_effect_and_then_repealed_the_price() {
    let hb110 = greenbook("hb110").flat();
    assert!(
        hb110.contains(
            "Supplemental targeted assistance is provided to 36 districts whose enrolled ADM is \
             less than 88% of their total residential ADM"
        ),
        "H.B. 110 created it, gated on the residence-against-attendance gap"
    );
    assert!(
        hb110.contains("estimated to provide $56.4 million to these 36 districts in FY 2022"),
        "and priced it"
    );

    let hb96 = hb96();
    assert!(
        hb96.contains("The budget eliminates supplemental targeted assistance"),
        "H.B. 96 repealed it"
    );
    assert!(
        hb96.contains(
            "might appear relatively wealthier on a per-pupil basis because of changes in how the \
             formula in place since FY 2022 counts students relative to the previous formula and \
             thus receive less state aid"
        ),
        "and stated, as the thing being repealed, the effect this corpus left open"
    );
    assert!(
        hb96.contains(
            "Supplemental targeted assistance payments totaled to an estimated $52.5 million to \
             36 districts in FY 2025"
        ),
        "with the last year's amount"
    );
}

/// Prior law put the choice children in the denominator and took them out of the multiplier.
///
/// This is what rules out reading R.C. 3317.0217(C)(1) as an incomplete reconstruction. There was
/// no partial residence count to rebuild: prior law's **formula ADM** was total ADM — *"the number
/// of all students who reside in the district"* — less 80% of JVSD ADM, and that single carve-out
/// exists only because JVSDs are paid by a separate formula rather than by deduction.
///
/// The choice channels were handled on the **multiplier** instead, and LSC gives the reason. So
/// the two corrections were a matched pair sitting on opposite sides of the same product, and
/// H.B. 110 kept one of them.
#[test]
fn prior_laws_denominator_counted_residents_and_its_multiplier_did_not() {
    let hb59 = greenbook("hb59").flat();

    assert!(
        hb59.contains("Total ADM is the number of all students who reside in the district"),
        "prior law's total ADM is a residence count"
    );
    assert!(
        hb59.contains("Formula ADM = Total ADM - 80% x JVSD ADM"),
        "and formula ADM carves out one channel, not nine"
    );
    assert!(
        hb59.contains("So that these students are not double counted"),
        "for a reason that is about the JVSD's separate formula rather than about school choice"
    );

    assert!(
        hb59.contains(
            "an adjustment is made to the formula ADM of each district so as to not credit the \
             district with targeted assistance for students educated through these programs"
        ),
        "and the choice channels are corrected on the multiplier, with the reason stated"
    );
    assert!(
        hb59.contains(
            "(Formula ADM - e-school ADM - EdChoice ADM - Jon Peterson Special Needs ADM - 75% of \
             non-e-school community school ADM)"
        ),
        "in the tier one formula itself"
    );
}

/// And open enrolment is the one channel that netting list leaves out.
///
/// Which is the channel R.C. 3317.0217(C)(1) adjusts for. The complementarity is what makes
/// "H.B. 110 took one of ten arbitrarily" the wrong reading and
/// <https://github.com/goedelsoup/ohio-education-funding/issues/399> the right question: the
/// surviving correction is the one that had no home on the payment side.
///
/// Asserted narrowly. That the two lists are complements **over all of R.C. 3317.03(A)(2)** is a
/// stronger claim than this checks, and it is (2) on that issue rather than a fact yet.
#[test]
fn open_enrolment_is_the_channel_prior_laws_multiplier_did_not_reach() {
    let hb59 = greenbook("hb59").flat();
    let netting = hb59
        .split_once("Tier one targeted assistance = Tier one targeted assistance per pupil x ")
        .expect("the tier one formula is in the analysis")
        .1
        .split_once(')')
        .expect("the netting list is parenthesised")
        .0;

    for channel in [
        "e-school ADM",
        "EdChoice ADM",
        "Jon Peterson Special Needs ADM",
        "community school ADM",
    ] {
        assert!(
            netting.contains(channel),
            "the netting list omits {channel}, so the pair this reading rests on is not the pair \
             prior law wrote: {netting}"
        );
    }
    assert!(
        !netting.to_lowercase().contains("open enrollment")
            && !netting.to_lowercase().contains("open enrolment"),
        "open enrolment is absent from prior law's multiplier correction, which is what makes \
         R.C. 3317.0217(C)(1)'s choice of that one channel worth a question: {netting}"
    );
}

/// The two corrections are not complements, and five channels are reached by neither.
///
/// #399 asked whether prior law's multiplier netting and R.C. 3317.0217(C)(1)'s denominator
/// adjustment are complementary by construction. They are not. Prior law's netting list names
/// **three** of the ten channels R.C. 3317.03(A)(2) lists — community school (e-schools
/// called out separately, the rest at 75%), the EdChoice nonpublic scholarship, and the Jon
/// Peterson provider scholarship. H.B. 110's adjustment reaches **one**, open enrolment. Together
/// they reach four, and alternative schools, College Credit Plus, educational service centers,
/// compact districts, STEM schools and college-preparatory boarding schools are reached by
/// neither.
///
/// So the neat story — two halves of one design, one surviving the deduct's abolition — is wrong.
/// What survives is narrower: the one channel prior law let a district keep targeted assistance
/// for is the one channel the new denominator adds back.
#[test]
fn prior_laws_netting_reaches_three_channels_and_the_new_adjustment_reaches_one() {
    let hb59 = greenbook("hb59").flat();
    let netting = hb59
        .split_once("Tier one targeted assistance = Tier one targeted assistance per pupil x ")
        .expect("the tier one formula is in the analysis")
        .1
        .split_once(')')
        .expect("the netting list is parenthesised")
        .0
        .to_lowercase();

    // The channels of R.C. 3317.03(A)(2), and the word prior law's netting list would name each
    // by. `None` means the list has no term for it at all.
    let channels: [(&str, Option<&str>); 10] = [
        ("(a) community school", Some("community school")),
        ("(b) alternative school", None),
        ("(c) college, under College Credit Plus", None),
        ("(d) open enrolment", None),
        ("(e) educational service center", None),
        ("(f) another district by compact", None),
        (
            "(g) chartered nonpublic with a scholarship",
            Some("edchoice"),
        ),
        ("(h) provider with a scholarship", Some("jon peterson")),
        ("(i) STEM school", None),
        ("(j) college-preparatory boarding school", None),
    ];

    let reached: Vec<&str> = channels
        .iter()
        .filter(|(_, term)| term.is_some_and(|t| netting.contains(t)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        reached.len(),
        3,
        "prior law's netting list reaches {reached:?} of the ten channels, in: {netting}"
    );

    assert!(
        !netting.contains("open enrollment") && !netting.contains("open enrolment"),
        "open enrolment is the channel R.C. 3317.0217(C)(1) adds back, and prior law did not net \
         it out — which is the whole of the continuity between the two regimes: {netting}"
    );

    let unreached = channels.len() - reached.len() - 1; // less open enrolment
    assert_eq!(
        unreached, 6,
        "five channels are corrected by neither regime, so neither the old formula nor the new \
         one ever had a principled denominator — it is not that H.B. 110 broke one"
    );
}

/// The same name covered a different programme two regimes earlier.
///
/// H.B. 64's supplemental targeted assistance was agricultural and rural. H.B. 110's is urban and
/// keyed on school choice. Reading the series by name gives a continuous programme that does not
/// exist.
#[test]
fn the_supplements_name_was_reused_for_an_unrelated_programme() {
    let hb64 = greenbook("hb64").flat();
    assert!(
        hb64.contains(
            "Supplemental targeted assistance is calculated by subtracting 10% from each \
             district's agricultural percentage"
        ),
        "H.B. 64's payment of that name is an agricultural one"
    );
    assert!(
        hb64.contains("only districts with more than 10% agricultural real property qualify"),
        "and gated on agricultural property"
    );

    let hb110 = greenbook("hb110").flat();
    assert!(
        !hb110.contains("agricultural percentage"),
        "H.B. 110's payment of the same name has no agricultural test at all"
    );
    assert!(
        hb96().contains(
            "primarily lower wealth, urban districts with relatively high proportions of resident \
             students who attended a school other than one operated by their home district"
        ),
        "and LSC describes its population as the opposite one"
    );

    // The redbook carries the same repeal, which is what rules out reading the amount as a
    // proposal that the enacted act changed.
    assert_eq!(
        Edition::Enacted.text(),
        GREENBOOK,
        "the quotations above are from the act as enacted"
    );
}
