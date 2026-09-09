//! What happened to the charge-off rate after FY2009, read out of LSC's own budget analyses.
//!
//! `parameter/local-share-charge-off-millage` holds three of four eras and says so deliberately:
//!
//! > The Evidence-Based Model years and the Bridge formula decade are not established. Secondary
//! > reporting gives 22 mills for FY2010-11 under the Evidence-Based Model and 20 mills applied to
//! > total valuation later, and neither is sourced well enough to commit here. `[open]` The corpus
//! > deliberately holds three of the four eras rather than a complete series with one guessed
//! > entry.
//!
//! The node's `statutory_basis` records why: Ohio Laws' version archive for R.C. 3317.022 begins
//! on 1 July 2014, after the mechanism was retired, so "the operative text for the charge-off era
//! is not retrievable from `codes.ohio.gov` and must come from the opinions or from session law."
//!
//! There is a third route, and it has been committed in this repository the whole time.
//! [`project::greenbook`] holds LSC's education analysis of every enacted budget act from the
//! 124th General Assembly through the 135th — twelve documents, 2.6 MB, with no reader until now.
//! LSC states each formula's rate and base in prose, biennium by biennium, which is the shape a
//! parameter series needs and neither an opinion nor an appropriation table gives.
//!
//! # The answer, and why the question was hard
//!
//! **There is no fourth entry to hold.** The Evidence-Based Model replaced one rate on one base
//! with one rate on *two* bases, split by whether a district sits at the twenty-mill floor. The
//! Bridge formula then replaced the rate with an index that yields a different implied rate for
//! every district — LSC says so in as many words — and the "20 mills" of secondary reporting is a
//! statewide *average* of that spread, not a rate anybody legislated.
//!
//! So the parameter did not run for thirty years and stop. It stopped being a parameter in FY2010,
//! and the corpus's caution about a "complete series with one guessed entry" was right for a
//! reason better than the one it gave.

use project::greenbook::{self, Greenbook};

/// The Evidence-Based Model kept the mechanism and split its base in two.
///
/// LSC's H.B. 1 analysis states both the prior rule and the new one in one paragraph, and three
/// things in it are new to this corpus.
///
/// **The rate fell to 22 mills, and the base forked.** A district at or very near the twenty-mill
/// floor — LSC's threshold is a class 1 effective current expense rate of 20.1 or lower — is
/// charged 22 mills of *total taxable valuation*; every other district is charged 22 mills of
/// *recognized valuation*. The node records the recognized-valuation base as a single fact about
/// FY2008 and could not have anticipated a rule that assigns bases by floor status, which is the
/// same status `parameter/twenty-mill-floor` turns on.
///
/// **There was a second charge-off all along.** Under prior law districts contributed 23 mills
/// toward base cost *and up to 3.3 mills toward special education, career-technical education and
/// transportation*. The node's series table carries only the first.
///
/// **The exempt-property adjustment has a stated rule.** The node records it as `[open]` for want
/// of the department's list of affected districts. The rule is not open: above 25% of potential
/// taxable value exempted, the excess over 25% comes off the base before the charge-off.
#[test]
fn the_evidence_based_model_charged_twenty_two_mills_against_two_different_bases() {
    let book = greenbook::greenbook("hb1");
    assert_eq!(book.general_assembly, 128);
    assert_eq!(book.first_fiscal_year(), 2010);

    let passage = book.around("Under prior law, school districts contributed", 9);

    // The rate, and the fork in the base.
    assert!(passage.contains("is 22 mills (2.2%) of total taxable valuation"));
    assert!(passage.contains("(those above the floor) is 22 mills (2.2%) of recognized valuation"));
    assert!(
        passage.contains("class 1 effective current expense millage rate is 20.1 or lower"),
        "the threshold that assigns the base is what makes this two rules and not one"
    );

    // Prior law, as LSC states it — and the second charge-off the corpus does not hold.
    assert!(passage.contains("contributed 23 mills (2.3%) of their recognized valuation"));
    assert!(
        passage.contains("up to 3.3 mills (0.33%) of their recognized"),
        "the special education, career-technical and transportation charge-off is absent from the \
         node's series and is a second rate on the same base"
    );

    // The exempt-property rule, which the node records as needing a district list it does not.
    assert!(passage.contains("more than 25% of potential taxable value exempted from taxation"));
}

/// The next biennium's analysis states the same 22 mills, which is what makes it a fact and not a
/// reading.
///
/// LSC footnotes it in the H.B. 153 greenbook while explaining the state education aid offset:
/// *"its local share of foundation funding, which was 22 mills (2.2%) of its taxable property
/// value in FY 2011"*. Two independent statements of the rate, in two documents, by the agency
/// that wrote the analysis of the act that set it.
#[test]
fn the_twenty_two_mills_is_stated_twice_in_two_bienniums() {
    let later = greenbook::greenbook("hb153");
    assert_eq!(later.general_assembly, 129);
    assert!(
        later
            .around("which was 22 mills", 2)
            .contains("22 mills (2.2%) of its taxable property value in FY 2011"),
        "the confirming footnote is gone and this series rests on one document again"
    );
}

/// The Bridge formula has no charge-off rate, which is why none could be found.
///
/// LSC, analysing H.B. 59: *"the state share index used for the opportunity grant does not result
/// in a uniform charge-off rate. Rather, if a local share in FY 2014 were to be derived from the
/// state share index and expressed as a charge-off of adjusted valuation, the charge-off rate
/// would vary from 11.3 mills to 22.9 mills (excluding several outlier districts) with the
/// statewide average charge-off being 20.6 mills."*
///
/// Three things follow. The "20 mills applied to total valuation later" of secondary reporting is
/// **20.6 mills and a statewide average**, so entering it in a parameter series would state a
/// distribution's mean as a legislated rate. The spread is wide — a factor of two across
/// districts, before outliers. And the phrase "uniform charge-off rate", which the node's own
/// description calls the point of the parameter, is the thing LSC says the index does not produce.
#[test]
fn the_bridge_formula_replaced_the_rate_with_a_spread() {
    let book = greenbook::greenbook("hb59");
    assert_eq!(book.first_fiscal_year(), 2014);

    let passage = book.around("Prior to FY 2010, the school funding formula", 11);
    assert!(passage
        .contains("In FY 2009 and several years prior to that, the charge-off rate was 23 mills"));
    assert!(
        passage.contains("does not result in a uniform charge-off rate"),
        "LSC's own denial that the Bridge formula has the parameter this node models"
    );
    assert!(passage.contains("would vary from 11.3 mills to 22.9 mills"));
    assert!(passage.contains("statewide average charge-off being 20.6 mills"));
}

/// The word outlives the mechanism by one biennium, attached to a valuation measure.
///
/// Counted across all twelve analyses, "charge-off" runs 35, 25, 3, 3 — then **zero in the
/// Evidence-Based Model's own greenbook**, which is the biennium the rate was 22 mills. It returns
/// eleven times in H.B. 153 and **every one of those eleven is "charge-off valuation"**, the
/// EBM's valuation measure carried forward as the index the Bridge formula's per-pupil adjustment
/// is scaled by. Not one is a rate.
///
/// H.B. 59 is the last analysis to use "charge-off rate" at all, and it uses it to say the formula
/// no longer has one. After a single JVSD mention each in H.B. 64 and H.B. 49, the word does not
/// appear again in any of the last three budgets.
///
/// A word count is weak evidence of a mechanism and strong evidence about a **search**: the reason
/// the corpus could not source the era is that the documents which would carry it stopped
/// discussing the subject, and a reader looking for a number was looking for the wrong shape of
/// thing.
#[test]
fn the_phrase_leaves_the_analyses_a_biennium_before_the_mechanism_does() {
    let count =
        |book: &Greenbook<'_>, phrase: &str| book.body.to_lowercase().matches(phrase).count();

    let mentions: Vec<(u16, usize)> = greenbook::greenbooks()
        .iter()
        .map(|book| (book.first_fiscal_year(), count(book, "charge-off")))
        .collect();
    assert_eq!(
        mentions,
        vec![
            (2002, 35),
            (2004, 25),
            (2006, 3),
            (2008, 3),
            (2010, 0),
            (2012, 11),
            (2014, 9),
            (2016, 1),
            (2018, 1),
            (2020, 0),
            (2022, 0),
            (2024, 0),
        ]
    );

    // The eleven in the Bridge formula's first biennium are all the valuation measure.
    let bridge = greenbook::greenbook("hb153");
    assert_eq!(
        count(&bridge, "charge-off"),
        count(&bridge, "charge-off valuation")
    );
    assert_eq!(count(&bridge, "charge-off rate"), 0);

    // And H.B. 59 is the last analysis that says "charge-off rate" at all.
    for book in greenbook::greenbooks() {
        if book.first_fiscal_year() > 2014 {
            assert_eq!(
                count(&book, "charge-off rate"),
                0,
                "FY{} still discusses a charge-off rate",
                book.first_fiscal_year()
            );
        }
    }
}

/// For joint vocational districts the charge-off was never retired.
///
/// The node opens by calling the parameter retired — *"the Fair School Funding Plan replaced the
/// charge-off with a local capacity measure"* — and that is true of city, local and exempted
/// village districts only. LSC's H.B. 110 analysis of the plan itself: *"Instead of a per-pupil
/// local capacity amount, the local share of the base cost for JVSDs is calculated by multiplying
/// a 1/2 mill by the lesser of the district's three-year average valuation or most recent
/// valuation. This is consistent with the prior formula for JVSDs."*
///
/// So the mechanism the node describes in the past tense is how a whole class of district is
/// funded under the plan, and the node's `series` line — "JVSDs: 0.5 mills throughout the observed
/// range" — reaches further than "observed" suggests. What did change is the base: H.B. 59 gives
/// the three-year average, and the plan takes the **lesser** of that and the most recent
/// valuation, which is a floor against a district's valuation falling.
#[test]
fn the_half_mill_survived_the_plan_that_retired_the_rest_of_the_mechanism() {
    let bridge = greenbook::greenbook("hb59");
    assert!(
        bridge
            .around("The local share of this cost is calculated", 3)
            .contains(
                "a uniform charge-off rate of 0.5 mill by the JVSD's three-year average valuation"
            ),
        "the Bridge-era JVSD rule is the baseline the plan is 'consistent with'"
    );

    let plan = greenbook::greenbook("hb110");
    assert_eq!(plan.first_fiscal_year(), 2022);
    let passage = plan.around("Instead of a per-pupil local capacity", 4);
    assert!(passage.contains(
        "the local share of the base cost for JVSDs is calculated by multiplying a 1\u{2044}2 mill"
    ));
    assert!(
        passage.contains("the lesser of the district\u{2019}s three-year average valuation or most recent valuation"),
        "the base changed even though the rate did not, which is this node's recurring shape"
    );
    assert!(passage.contains("This is consistent with the prior formula for JVSDs"));
}

/// Every biennium from FY2002 is covered, which is what makes an absence readable.
///
/// The counts above are only evidence because there is no gap in the series: a biennium with no
/// analysis committed would be indistinguishable from a biennium that stopped discussing the
/// subject. Twelve consecutive General Assemblies, twelve consecutive bienniums, FY2002 through
/// FY2025.
#[test]
fn the_extract_covers_every_biennium_without_a_gap() {
    let books = greenbook::greenbooks();
    assert_eq!(books.len(), 12);
    assert_eq!(books.first().expect("twelve").first_fiscal_year(), 2002);
    assert_eq!(books.last().expect("twelve").first_fiscal_year(), 2024);

    for book in &books {
        assert!(
            book.body.len() > 10_000,
            "{} holds {} bytes, which is too little to be a budget analysis",
            book.id,
            book.body.len()
        );
        assert!(
            book.mentions("Department of Education"),
            "{} does not read like an education greenbook",
            book.id
        );
    }
}
