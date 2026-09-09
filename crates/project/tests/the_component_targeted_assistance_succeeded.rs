//! What targeted assistance replaced, which `formula-component/fsfp-targeted-assistance` asks and
//! answers three ways at once.
//!
//! The node's `[open]`: *"The charge-off era had its own equalisation and its own supplement for
//! districts the charge-off overcharged — gap aid, $73.5m across 145 districts in FY2008 — and
//! whether targeted assistance is the successor to that, to something else, or to nothing is not
//! established here."*
//!
//! It is the successor to **parity aid**, and gap aid could not have had a successor.
//!
//! # Two components, and LSC's own headings separate them
//!
//! Gap aid's heading in every analysis that carries it is **"Charge-off Supplement (Gap Aid)"**,
//! and its definition is the phantom-revenue fix: *"Gap aid is provided to districts whose actual
//! operating revenue is lower than the local share assumed by the formula."* It supplemented an
//! assumption. When [`crate::greenbook`]'s trace shows the charge-off gone, the assumption is
//! gone, and there is nothing left to supplement — `parameter/local-share-charge-off-millage`
//! records the charge-off's own end and this is the same event seen from the other side.
//!
//! Parity aid is the equalisation, and every structural feature of targeted assistance descends
//! from it: a millage instrument, a blend of property wealth with income, and a rank threshold.
//!
//! # The lineage, in the three things that carry it
//!
//! | | millage | property/income blend | equalised to |
//! |---|---|---|---|
//! | FY2002 parity aid | 9.5 | 2/3 : 1/3 | the 80th percentile district |
//! | FY2008 parity aid | 8.0 | — | the 80th percentile district |
//! | FY2009 parity aid | 8.5 | 75 : 25 as H.B. 1 states it | the 80th percentile district |
//! | FY2014 targeted assistance | 6 | 1/2 : 1/2 | the 490th lowest of ~612 |
//! | FY2022 targeted assistance | 14 and 11.2 | 60 : 40 | the **median** |
//!
//! The 490th lowest of 612 *is* the 80th percentile. H.B. 59 renamed the component, changed the
//! blend and cut the rate, and left parity aid's threshold where it was. The Fair School Funding
//! Plan is the act that moved it — from the wealth of a fairly rich district to the wealth of the
//! middle one — and raised the rate to pay for the shorter distance. That is a larger change than
//! any of the rate changes and it is the one none of the analyses announces as a change.

mod common;

use project::greenbook::{self, Greenbook};

/// The FY2002-03 budget's analysis — Am. Sub. H.B. 94 of the 124th, which created both.
fn created() -> Greenbook<'static> {
    greenbook::greenbook("hb94")
}

/// The FY2008-09 budget's — Am. Sub. H.B. 119 of the 127th, the last to carry either.
fn last() -> Greenbook<'static> {
    greenbook::greenbook("hb119")
}

/// The FY2010-11 budget's — Am. Sub. H.B. 1 of the 128th, which ended parity aid.
fn ended() -> Greenbook<'static> {
    greenbook::greenbook("hb1")
}

/// Parity aid and gap aid are two components, and only one of them has a successor.
///
/// The trace across all twelve committed analyses: both begin in FY2002, gap aid stops after
/// FY2009, parity aid after FY2011, nothing carries either through FY2012-13, and targeted
/// assistance appears in FY2014. Two components out, one in, and a biennium of neither between.
#[test]
fn parity_aid_and_gap_aid_are_two_components_and_one_successor() {
    let trace: Vec<(u16, bool, bool, bool)> = greenbook::greenbooks()
        .iter()
        .map(|book| {
            (
                book.first_fiscal_year(),
                book.mentions("parity aid"),
                book.mentions("gap aid"),
                book.mentions("targeted assistance"),
            )
        })
        .collect();

    assert_eq!(
        trace,
        vec![
            (2002, true, true, false),
            (2004, true, true, false),
            (2006, true, true, false),
            (2008, true, true, false),
            (2010, true, false, false),
            (2012, false, false, false),
            (2014, false, false, true),
            (2016, false, false, true),
            (2018, false, false, true),
            (2020, false, false, true),
            (2022, false, false, true),
            (2024, false, false, false),
        ]
    );
}

/// Gap aid supplemented the charge-off, which is why it could not outlive it.
///
/// LSC files it under the heading "Charge-off Supplement (Gap Aid)" and defines it as the payment
/// that closes the gap between what the formula *assumes* a district raises locally and what it
/// actually raises. The corpus's `formula-component/charge-off-local-share` already records the
/// mechanism from the other direction: a local share truncated at zero.
#[test]
fn gap_aid_supplemented_an_assumption_and_died_with_it() {
    let lsc = last().flat();
    assert!(lsc.contains("Charge-off Supplement (Gap Aid)."));
    assert!(lsc.contains(
        "Gap aid is provided to districts whose actual operating revenue is lower than the local \
         share assumed by the formula."
    ));

    // And the claim its creators made for it, which is the phantom-revenue argument in one line.
    assert!(created()
        .flat()
        .contains("With gap aid, formula phantom revenue has been completely eliminated."));
    assert!(created().flat().contains(
        "gap aid calculations will include the local share of the base cost funding at 23 mill \
         charge-off"
    ));
}

/// Parity aid equalised millage to the eightieth percentile, on a blend of property and income.
///
/// Every structural feature targeted assistance has. The one it does not have is parity aid's
/// stated purpose: parity aid equalised revenue *above* the adequacy level, so it was explicitly
/// a payment for what districts spend beyond what the state calls adequate.
#[test]
fn parity_aid_equalised_millage_to_the_eightieth_percentile() {
    let lsc = created().flat();
    assert_eq!(created().first_fiscal_year(), 2002);

    assert!(lsc.contains(
        "Parity aid equalizes an additional 9.5 mills (above the adequacy level) to the 80th \
         percentile district\u{2019}s wealth level."
    ));
    assert!(lsc.contains(
        "The parity aid wealth is a weighted average property wealth (2/3) and income wealth (1/3)."
    ));
    assert!(lsc.contains(
        "Overall, about 492 school districts are eligible for parity aid with no additional local \
         effort requirement."
    ));
}

/// The rate rose while eligibility narrowed, and the threshold did not move.
///
/// FY2008 equalises 8.0 mills to 410 districts and FY2009 8.5 mills to 367, both still to the
/// eightieth-percentile district's wealth. So the two dials the General Assembly turned on this
/// component were the rate and the eligible count, in opposite directions, against a fixed target.
#[test]
fn the_rate_rose_while_eligibility_narrowed() {
    let lsc = last().flat();
    assert_eq!(last().first_fiscal_year(), 2008);
    assert!(lsc.contains(
        "The budget lowers the number of qualifying districts to the 410 lowest wealth districts \
         in FY 2008 and the 367 lowest wealth districts in FY 2009."
    ));
    assert!(lsc.contains(
        "The budget also changes the calculation so that it equalizes 8.0 mills in FY 2008 and \
         8.5 mills in FY 2009. These mills are still equalized to the wealth level of the \
         district at the 80th percentile."
    ));
}

/// The 490th lowest district of about 612 is the eightieth percentile under another name.
///
/// H.B. 59 replaced parity aid with targeted assistance and kept its threshold: 489 districts
/// eligible, equalised to the wealth of the 490th lowest, out of the 612 the panel of that era
/// carries. That is the 80.1st percentile, against parity aid's 80th, and against the 492
/// districts parity aid reached in FY2002.
#[test]
fn the_four_hundred_and_ninetieth_district_is_the_eightieth_percentile() {
    let lsc = greenbook::greenbook("hb59").flat();
    assert!(lsc.contains(
        "Tier one targeted assistance is provided to the 489 districts with the lowest wealth per \
         pupil."
    ));
    assert!(lsc.contains("the district with the 490th lowest wealth per pupil"));

    // The panel of the era, from the federal survey rather than from a number typed here.
    let districts = dispersion::ohio_panel::spending_by_year()[&2013].districts;
    assert_eq!(districts, 612);

    let percentile = 490.0 / f64::from(u32::try_from(districts).expect("a district count fits"));
    assert!(
        common::agrees_within(0.005, percentile, 0.80),
        "the 490th of {districts} is the {:.1}th percentile",
        percentile * 100.0
    );
}

/// And the plan moved the target from the eightieth percentile to the median.
///
/// Every earlier version equalises to a district well up the wealth distribution. H.B. 110
/// equalises to the *median* district and raises the rate from 6 mills to 14 — a shorter distance
/// charged harder. No analysis presents that as the change it is; the FY2022 account states the
/// new formula and does not say what the threshold used to be.
#[test]
fn the_plan_moved_the_target_from_the_eightieth_percentile_to_the_median() {
    let plan = greenbook::greenbook("hb110").flat();
    assert!(plan.contains(
        "A wealth index is calculated for each district by dividing the statewide median \
         district\u{2019}s weighted wealth per pupil by the district\u{2019}s weighted wealth per \
         pupil"
    ));
    assert!(
        !plan.contains("80th percentile"),
        "the analysis names the old threshold after all and this reading needs redoing"
    );
    assert!(
        !plan.contains("percentile"),
        "the plan's account uses no percentile at all"
    );
}

/// Parity aid ended as a district payment, and its wealth measure did not end.
///
/// H.B. 1 dropped it from the district formula, froze it as a per-pupil transfer to community
/// schools at each district's FY2009 rate, and then used its wealth measure twice over — once in
/// the Evidence-Based Model's own wealth per pupil and once in a new supplemental transportation
/// payment. So the component the corpus could not find a successor for left three descendants,
/// and only one of them is called targeted assistance.
#[test]
fn parity_aid_ended_as_a_payment_and_survived_as_a_measure() {
    let lsc = ended().flat();
    assert_eq!(ended().first_fiscal_year(), 2010);

    assert!(lsc.contains(
        "Each district\u{02B9}s wealth per pupil is measured as 75% of its recognized taxable \
         property valuation plus 25% of the sum of the incomes of its residents, divided by its \
         ADM. This is the same wealth measure that was used in the parity aid component of the \
         previous school funding formula."
    ));
    assert!(lsc.contains(
        "Wealth per pupil is measured using the same wealth measure previously used in the \
         calculation of parity aid, which incorporates both income and property value."
    ));
    assert!(lsc.contains(
        "the per pupil amount to be deducted from a school district in FY 2010 and FY 2011 and \
         transferred to a community school for parity aid and poverty\u{2010}based assistance is \
         equal to the per pupil amount paid to the respective school district for FY 2009"
    ));
}
