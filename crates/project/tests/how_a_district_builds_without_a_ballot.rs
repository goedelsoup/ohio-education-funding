//! How a school district builds without a ballot, read from the sections that let it.
//!
//! `revenue-stream/capital-finance-without-a-vote` sets three unvoted routes to a building beside
//! the bond levy. The research it was drafted from held four readings as hypotheses:
//!
//! - that an energy conservation contract under R.C. 3313.372 needs no vote, is capped at the
//!   guaranteed savings, runs fifteen years, and sits outside R.C. 133.06. The first three hold.
//!   The fourth is half right: division (E) leaves the debt out of the R.C. 133.06 count, and
//!   division (D) still caps all of the district's unvoted debt at one per cent of tax valuation;
//! - that a lease-purchase under R.C. 3313.375 is not debt. The section says its obligations are
//!   not net indebtedness under R.C. 133.06, and runs them in one-year renewable terms of at most
//!   thirty years;
//! - that a certificate of participation has an authorizing section. None names it for a school
//!   district. R.C. 133.01(N) defines it as a fractionalized interest in a public obligation, and
//!   (GG)(2) makes a lease-purchase payment a public obligation;
//! - that "urgent necessity" in R.C. 3313.46 is defined and has a decider. It is neither. R.C.
//!   3311.75 alone, among the school bidding rules, says who decides.

use project::capital;
use project::statute::{prior, section};

/// An energy conservation contract is let and its notes issued without a vote, and paid only
/// out of savings that occur.
#[test]
fn energy_conservation_notes_need_no_vote_and_are_paid_from_savings() {
    let body = section("3313.372").body;
    assert!(body.contains(
        "the board may provide, without the vote of the electors of the district, for annually \
         levying and collecting taxes in amounts sufficient to pay the interest on and retire the \
         notes"
    ));
    assert!(body.contains("shall not be subject to Chapter 133. of the Revised Code"));
    assert!(body.contains(
        "shall not exceed the calculated energy, water, or waste water cost savings, avoided \
         operating costs, and avoided capital costs"
    ));
    assert!(body.contains("Those payments shall be made only to the extent that the savings"));
    assert!(body.contains(
        "The energy services company shall warrant and guarantee that the energy conservation \
         measures shall realize guaranteed savings"
    ));
    // The facilities construction commission must find the savings finding reasonable.
    assert!(body.contains("that finding shall cover the ensuing fifteen years"));
    assert!(body.contains("approves the contract"));
}

/// Fifteen years, one-fifteenth of it within two.
#[test]
fn the_contract_runs_fifteen_years() {
    assert_eq!(capital::energy_contract_term_years(), Some(15));
    let first = capital::energy_contract_first_payment_share().expect("reads");
    assert!((first - 1.0 / 15.0).abs() < 1e-15);
}

/// **Outside 133.06, and not outside its unvoted ceiling.** Division (E) leaves the debt out of
/// the count; division (D) caps all unvoted debt at one per cent; and 133.06(A) names 3313.372(D)
/// among the exceptions to its own one-tenth of one per cent.
#[test]
fn energy_debt_is_left_out_of_net_indebtedness_but_not_out_of_the_one_per_cent_ceiling() {
    assert!(capital::energy_debt_excluded_from_net_indebtedness());
    let ceiling = capital::energy_notes_unvoted_ceiling().expect("reads");
    assert!((ceiling - 0.01).abs() < 1e-15, "{ceiling}");
    assert!(section("133.06")
        .body
        .contains("division (D) of section 3313.372 of the Revised Code"));
}

/// The parallel route: energy securities under 133.06(G), nine-tenths of one per cent of
/// principal, inside the same one per cent.
#[test]
fn energy_securities_are_nine_tenths_inside_one_per_cent() {
    let principal = capital::energy_securities_principal_limit().expect("reads");
    assert!((principal - 0.009).abs() < 1e-15, "{principal}");
    let ceiling = capital::energy_securities_unvoted_ceiling().expect("reads");
    assert!((ceiling - 0.01).abs() < 1e-15, "{ceiling}");
    assert!(principal > project::levy::unvoted_limit());
}

/// H.B. 15 of the 136th General Assembly added solar panels to the measures and the loan fund
/// to the section, effective August 14, 2025.
#[test]
fn hb15_added_solar_panels_and_the_loan_fund() {
    let now = section("3313.372");
    let before = prior("3313.372", "9-29-2017");
    assert!(now.legislation.contains("House Bill 15"));
    assert!(now.effective.contains("August 14, 2025"));
    assert!(before.legislation.contains("House Bill 49"));
    assert_eq!(capital::energy_measures_in_force(), 10);
    assert_eq!(capital::energy_measures_before_hb15(), 9);
    assert!(capital::names_solar_panels(now.body));
    assert!(!capital::names_solar_panels(before.body));
    assert!(now
        .body
        .contains("school energy performance contracting loan fund"));
    assert!(!before.body.contains("loan fund"));
}

/// Lease-purchase: thirty years at most, paid as operating expense, not net indebtedness, and
/// with no vote anywhere in the section.
#[test]
fn a_lease_purchase_is_thirty_years_of_operating_expense_and_no_debt() {
    let lease = section("3313.375");
    assert_eq!(capital::lease_purchase_term_ceiling_years(), Some(30));
    assert!(lease
        .body
        .contains("a series of one-year renewable lease terms"));
    assert!(capital::lease_purchase_paid_as_operating_expense());
    assert!(capital::lease_purchase_is_outside_net_indebtedness());
    assert!(!lease.body.contains("electors"));
    assert!(!lease.body.contains("certificates of participation"));
    assert!(lease.effective.contains("September 29, 2015"));
}

/// A certificate of participation is a fractionalized interest in a public obligation, and a
/// lease-purchase payment is a public obligation. R.C. 9.94 bars a third party from selling one
/// without the obligor's written approval.
#[test]
fn a_certificate_of_participation_is_defined_not_granted() {
    let definitions = section("133.01").body;
    assert!(definitions.contains(
        "(N) \"Fractionalized interests in public obligations\" means participations, \
         certificates of participation"
    ));
    assert!(definitions.contains(
        "(2) Obligations of a public issuer to make payments under installment sale, lease, lease \
         purchase, or similar agreements"
    ));
    let sale = section("9.94").body;
    assert!(sale.contains("as defined in division (GG)(2) of section 133.01"));
    assert!(sale.contains("without the knowledge and express written approval or authorization"));
}

/// "Urgent necessity" is an exception R.C. 3313.46 names and does not define or assign.
#[test]
fn urgent_necessity_is_undefined_and_3313_46_names_no_decider() {
    let bidding = section("3313.46");
    assert!(bidding
        .body
        .contains("except in cases of urgent necessity, or for the security and protection of school property"));
    assert!(!bidding.body.contains("\"urgent necessity\" means"));
    assert_eq!(capital::urgent_necessity_determiner("3313.46"), None);
    // The municipal school district rule, for supplies and services, says who decides.
    assert_eq!(
        capital::urgent_necessity_determiner("3311.75"),
        Some("two-thirds vote of the board")
    );
    assert!(section("3311.75")
        .body
        .contains("supplies or services involving the expenditure of fifty thousand dollars"));
    // The definitions of Chapter 133 do not supply one either.
    assert!(!capital::mentions_urgent_necessity("133.01"));
}

/// H.B. 96 widened the rule from "any school building" to "any building or other property";
/// S.B. 168 had already tied the threshold to R.C. 9.17.
#[test]
fn hb96_widened_3313_46_to_any_building_or_other_property() {
    let now = section("3313.46");
    let before = prior("3313.46", "10-24-2024");
    assert!(now.legislation.contains("House Bill 96"));
    assert!(now.effective.contains("September 30, 2025"));
    assert!(before.legislation.contains("Senate Bill 168"));
    assert!(now.body.contains("demolish any building or other property"));
    assert!(before.body.contains("demolish any school building"));
    assert!(before.body.contains("the amount specified in section 9.17"));
}

/// The threshold 3313.46 borrows: $75,000 through 2024, then three per cent a year.
#[test]
fn the_bidding_threshold_is_seventy_five_thousand_then_three_per_cent() {
    assert_eq!(capital::bidding_threshold_through_2024(), Some(75_000));
    let step = capital::bidding_threshold_escalator().expect("reads");
    assert!((step - 0.03).abs() < 1e-15);
    assert!(section("9.17").legislation.contains("House Bill 33"));
}
