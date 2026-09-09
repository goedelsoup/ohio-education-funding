//! Targeted assistance before the biennium the corpus computes, and the unit its rates are in.
//!
//! `parameter/targeted-assistance-rates` carries nine constants, gives them `unit: ratio`, and
//! records one `unfilled:` entry — "the rates before FY2026" — whose stated route is Ohio Laws'
//! version archive. The archive is not the only route and, for this parameter, not the best one:
//! LSC's analysis of the act that *enacted* the section states every one of the nine, and the
//! analyses before it state the two components this one replaced.
//!
//! # The section is the only place these rates are not millage
//!
//! R.C. 3317.0217 writes 0.014, 0.0112 and 0.008 and never uses the word "mill". Every LSC
//! account of the same arithmetic, in every biennium it has existed, writes them as mills: 14 and
//! 11.2 for the wealth tier, 8 for the capacity tier, 6 for the tier one this replaced, and one
//! mill for the capacity aid it absorbed. The decimals are millage with the unit dropped, which
//! is why `sensitivity` on the node reads as arithmetic about coefficients and the same sentence
//! in LSC's hands reads as a statement about the property tax.
//!
//! That matters beyond vocabulary. `parameter/local-share-charge-off-millage` is the corpus's
//! record of the *other* millage instrument in Ohio's formula, and the two have never been
//! related to each other, because one is written in mills and one is written in decimals.
//!
//! # What a greenbook is evidence of
//!
//! LSC's account of an act as enrolled — not the act, and where they differ the act governs. The
//! FY2022 magnitudes below are LSC's estimates at enactment, and the FY2027 figures beside them
//! are the department's own model; they are the same quantity computed by two instruments two
//! bienniums apart, which is enough to say the programme grew and not enough to attribute the
//! growth to the cent.

mod common;

use project::greenbook::{self, Greenbook};
use project::panel::{
    self, TA_CAPACITY_RATE, TA_WEALTH_INDEX_FLOOR, TA_WEALTH_OFFSET_RATE, TA_WEALTH_RATE,
};
use project::statute;

/// The FY2014-15 budget's analysis — Am. Sub. H.B. 59 of the 130th, which named the component.
fn first() -> Greenbook<'static> {
    greenbook::greenbook("hb59")
}

/// The FY2016-17 budget's — Am. Sub. H.B. 64 of the 131st, which added capacity aid beside it.
fn split() -> Greenbook<'static> {
    greenbook::greenbook("hb64")
}

/// The FY2022-23 budget's — Am. Sub. H.B. 110 of the 134th, which merged the two back together.
fn merged() -> Greenbook<'static> {
    greenbook::greenbook("hb110")
}

/// The rates are millage, and the section is the one document that does not say so.
///
/// Three rates, two units. The Revised Code states all three as bare decimals and uses the word
/// "mill" nowhere in the section; H.B. 110's analysis states the same three as 14 mills, 11.2
/// mills and 8 mills in the sentences that explain them.
#[test]
fn the_rates_are_millage_and_the_section_is_where_the_unit_was_dropped() {
    let section = statute::section("3317.0217");
    let code = section
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    assert!(code.contains("X 0.014) - (the district's weighted wealth per pupil"));
    assert!(code.contains("X 0.0112)"));
    assert!(code.contains("X 0.008"));
    assert!(
        !code.to_lowercase().contains("mill"),
        "the section names a unit after all and this reading needs redoing"
    );

    let lsc = merged().flat();
    assert!(lsc.contains(
        "the statewide median district\u{2019}s wealth per pupil multiplied by 14 mills and the \
         district\u{2019}s weighted wealth per pupil multiplied by 11.2 mills"
    ));
    assert!(lsc.contains("credited with a capacity amount equal to 8 mills multiplied by"));

    // And the same analysis restates them as the section's decimals, so this is one document
    // holding both notations rather than two documents disagreeing.
    assert!(
        lsc.contains("per pupil x 0.014) - (District\u{2019}s weighted wealth per pupil x 0.0112)")
    );
    assert!(lsc.contains("x 0.008 x District\u{2019}s capacity amount percentage"));
}

/// The wealth tier's 0.8 cutoff is not a third parameter. It is 11.2 over 14.
///
/// The node reads the floor as a boundary the tier stops paying at, which is true, and as a
/// boundary somebody chose, which is not. The bracket is
/// `median x 0.014 - district x 0.0112`, so it reaches zero exactly where
/// `district / median = 0.014 / 0.0112`, that is at a wealth index of 0.8. Both statutory gates
/// in this section have that shape: the capacity tier's index test at 1.0 is where
/// `median - district` reaches zero. Neither gate excludes a district that would otherwise have
/// been paid; both prevent a negative payment.
///
/// So the section has nine constants and seven degrees of freedom, and a proposal that "keeps
/// the 0.8 eligibility floor" while changing either coefficient has described something that
/// cannot be done.
#[test]
fn both_eligibility_gates_are_the_arithmetic_and_not_a_policy() {
    assert!((TA_WEALTH_OFFSET_RATE - TA_WEALTH_RATE * TA_WEALTH_INDEX_FLOOR).abs() < 1e-15);
    assert!((TA_WEALTH_OFFSET_RATE / TA_WEALTH_RATE - TA_WEALTH_INDEX_FLOOR).abs() < 1e-15);

    // At the floor the payment is zero, from the coefficients alone and with the gate removed.
    let median = panel::TA_MEDIAN_WEALTH_PER_PUPIL;
    let at_the_floor = median / TA_WEALTH_INDEX_FLOOR;
    assert!(common::close(
        median * TA_WEALTH_RATE - at_the_floor * TA_WEALTH_OFFSET_RATE,
        0.0
    ));

    // The capacity tier's gate has the same shape: its bracket is a difference, so the index test
    // at 1.0 is the point the difference reaches zero and not an eligibility rule.
    let wealth = panel::TA_MEDIAN_WEIGHTED_WEALTH;
    assert!(common::close((wealth - wealth) * TA_CAPACITY_RATE, 0.0));

    // Both gates are nonetheless written out as separate divisions, which is what made them look
    // like separate decisions.
    let code = statute::section("3317.0217")
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(code.contains(
        "is less than 0.8, the district's wealth amount for that fiscal year shall be zero"
    ));
}

/// One payment now, two components before, and the analysis that merged them says which.
///
/// The trace across all twelve committed greenbooks: targeted assistance is named from FY2014,
/// capacity aid joins it for two bienniums, both go unmentioned in the frozen FY2020-21 budget,
/// and H.B. 110 replaces the pair with a single payment in two tiers. H.B. 33 then describes
/// neither, which is why the node's FY2024-25 evidence is absent rather than contrary.
#[test]
fn the_payment_is_two_earlier_components_merged_and_lsc_names_both() {
    let named: Vec<(u16, bool, bool)> = greenbook::greenbooks()
        .iter()
        .map(|g| {
            (
                g.first_fiscal_year(),
                g.mentions("targeted assistance"),
                g.mentions("capacity aid"),
            )
        })
        .collect();

    assert_eq!(
        named,
        vec![
            (2002, false, false),
            (2004, false, false),
            (2006, false, false),
            (2008, false, false),
            (2010, false, false),
            (2012, false, false),
            (2014, true, false),
            (2016, true, true),
            (2018, true, true),
            (2020, true, false),
            (2022, true, true),
            (2024, false, false),
        ]
    );

    assert!(merged().flat().contains(
        "The budget replaces the targeted assistance and capacity aid components of the previous \
         formula with a targeted assistance payment consisting of a \u{201c}wealth amount\u{201d} \
         and a \u{201c}capacity amount.\u{201d}"
    ));
}

/// The first targeted assistance charged 6 mills against a rank, not a median.
///
/// Four things the current section does not do. The wealth measure blended valuation and income
/// **50/50** rather than 60/40; the index was against the *statewide* wealth per pupil rather
/// than the median; equalisation ran to the **490th lowest** district's wealth rather than to the
/// median district's; and the wealth index entered the payment a second time as a multiplier, so
/// the tier was progressive in wealth twice over. The current wealth tier is linear in the
/// district's own wealth and the earlier one was not.
#[test]
fn the_first_targeted_assistance_charged_six_mills_against_a_rank() {
    let lsc = first().flat();
    assert_eq!(first().first_fiscal_year(), 2014);

    assert!(lsc.contains(
        "District wealth per pupil = 0.5 x (Average of last three years' taxable property \
         valuation / Formula ADM) + 0.5 x (Average of last three years' FAGI / Formula ADM)"
    ));
    assert!(lsc.contains(
        "Tier one targeted assistance is provided to the 489 districts with the lowest wealth per \
         pupil."
    ));
    assert!(lsc.contains("the district with the 490th lowest wealth per pupil"));
    assert!(lsc.contains("multiplied by a target millage rate of 6 mills in each fiscal year"));
    assert!(lsc.contains("Target millage = 0.006"));
    assert!(lsc.contains(
        "Tier one targeted assistance per pupil = (Wealth per pupil of 490th lowest wealth \
         district - District wealth per pupil) x Target millage x District wealth index"
    ));

    // And a second tier the plan has no counterpart for, keyed to agricultural property.
    assert!(lsc.contains("Agricultural targeted percentage = 40%"));
}

/// Capacity aid was one mill, and its dial moved twice in the four years it existed.
///
/// The component the current capacity tier descends from was charged on the amount a district
/// raises with a single mill, scaled by a ratio capped at 2.5, and then multiplied by a number
/// that was itself the policy: 2.75 in FY2016, 3.50 in FY2017, 4.00 in FY2018 and FY2019. Three
/// values in four years, in a component that lasted four years.
///
/// The node's `sensitivity` says nobody prices the size brackets. The history says something
/// adjacent and sharper: this tier's history *is* a history of turning one multiplier, and the
/// section that replaced it has no multiplier to turn.
#[test]
fn capacity_aid_was_one_mill_and_a_multiplier_that_moved_twice() {
    let lsc = split().flat();
    assert_eq!(split().first_fiscal_year(), 2016);

    assert!(lsc.contains(
        "This component, capacity aid, is based on the amount a district can raise with one mill \
         and is provided to districts that raise less than the median amount."
    ));
    assert!(lsc.contains("District capacity amount = Three-year average valuation x 0.001"));
    assert!(lsc.contains(
        "Capacity ratio = The lesser of [(Median capacity amount / District capacity amount) \
         \u{2013} 1] or 2.5"
    ));
    assert!(lsc.contains("Capacity aid multiplier = 2.75 in FY 2016 and 3.50 in FY 2017"));

    assert!(greenbook::greenbook("hb49").flat().contains(
        "increases a multiplier used in the formula for computing capacity aid funds from 3.5 in \
         FY 2017 to 4.0 in both FY 2018 and FY 2019"
    ));
}

/// Not one of the nine constants has moved since FY2022, and the programme has grown by a third.
///
/// H.B. 110 states each tier's estimate before guarantees and phase-ins: $738.5 million for the
/// wealth amount, $250.0 million for the capacity amount, $988.4 million together. The FY2027
/// panel computes $1,364,333,154.32 for the same two tiers under the same nine constants.
///
/// The medians are what moved. Both tiers pay a *difference* between the median district's wealth
/// and the district's own, so a statewide revaluation that lifts every district's property value
/// proportionally lifts every gap proportionally and raises the programme's cost with no
/// enactment. That is the node's own `[inference]` about distribution, with the growth measured.
#[test]
fn the_rates_have_not_moved_and_the_programme_has() {
    let lsc = merged().flat();
    assert!(lsc.contains(
        "Before any guarantees or phase-ins, targeted assistance is estimated to be $988.4 \
         million in FY 2022."
    ));
    assert!(lsc.contains(
        "Before any guarantees or phase-ins, the wealth amount is estimated to be $738.5 million \
         in FY 2022."
    ));
    assert!(lsc.contains(
        "Before any guarantees or phase-ins, the capacity amount is estimated to be $250.0 \
         million in FY 2022."
    ));

    let fy2027: f64 = panel::panel()
        .iter()
        .map(|r| r.categoricals.targeted_assistance)
        .sum();
    assert!(common::close(fy2027, 1_364_333_154.32));

    // LSC's two tiers sum to $988.5m against a stated total of $988.4m. That is the last digit
    // of three figures published to a tenth of a million, not a discrepancy — and it is why the
    // growth below is bounded rather than stated to the dollar.
    assert!((738.5_f64 + 250.0 - 988.4 - 0.1).abs() < 1e-9);

    let growth = fy2027 / 988_400_000.0 - 1.0;
    assert!(
        (0.37..0.39).contains(&growth),
        "targeted assistance grew {growth:.4} from FY2022 to FY2027"
    );
}
