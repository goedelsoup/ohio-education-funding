//! Which school levies a reduction factor reaches, read from the sections that say so.
//!
//! `parameter/tax-reduction-factor` sorts levies by how they meet R.C. 319.301: a levy voted as a
//! rate is reduced as values rise, and a levy voted as a sum of money is not. The sort is only as
//! good as the sections behind it, and the corpus had them half right: the floor node and
//! `the_statute_behind_the_weights.rs` both called R.C. 5705.199 the emergency levy section. It is
//! the substitute levy. The emergency levy is R.C. 5705.194, which the exemption reaches by its
//! general words rather than by name.

use project::statute::section;

/// A factor holds each levy's yield on carryover property, class by class, without touching the
/// rate on the books.
#[test]
fn a_factor_is_computed_per_tax_and_per_class_and_never_moves_the_voted_rate() {
    let s = section("319.301").body;
    assert!(s.contains("Separate determinations shall be made for each of the two classes"));
    assert!(s.contains("section 5713.041"));
    assert!(s.contains(
        "No reduction shall be made under this section in the rate at which any tax is levied"
    ));
    // A first-time levy is fixed against the prior year's values; a renewal is not "levied for
    // the first time", and so carries the reduced rate of the levy it renews.
    assert!(s.contains("levied for the first time that is not a renewal"));
    // The floor applies class by class.
    assert!(s.contains("for either class"));
}

/// The classes exist for nothing but the reduction.
#[test]
fn the_two_classes_exist_for_the_reduction() {
    let s = section("5713.041").body;
    assert!(s.contains("classified as residential/agricultural real property"));
    assert!(s.contains("classified as nonresidential/agricultural real property"));
    assert!(s.contains(
        "solely for the purpose of making the reductions in taxes required by section 319.301"
    ));
}

/// The emergency levy is a sum of money, and new ones are confined to districts in fiscal
/// distress or a declared disaster.
#[test]
fn the_emergency_levy_is_section_5705_194_and_is_voted_as_a_sum() {
    let s = section("5705.194");
    assert!(s.title.to_lowercase().contains("ten-mill"));
    assert!(s
        .body
        .contains("the amount of money it is necessary to raise for the current expenses"));
    assert!(s
        .body
        .contains("fiscal caution, fiscal watch, or fiscal emergency"));
    assert!(s.body.contains("may not be renewed"));
    assert!(s.body.contains("renewed for the same or a lesser sum"));
}

/// The substitute levy is a sum that grows with new construction and not with reappraisal, and
/// no new one may be proposed after H.B. 96.
#[test]
fn the_substitute_levy_is_section_5705_199_and_grows_only_with_new_construction() {
    let s = section("5705.199");
    assert!(s.body.contains("proposed as a substitute for all or a portion of one or more existing levies imposed under section 5705.194"));
    assert!(s
        .body
        .contains("excluding carryover property as defined in section 319.301"));
    assert!(s
        .body
        .contains("At any time before the effective date of this amendment"));
    assert!(s.legislation.contains("96"), "{}", s.legislation);
    // And the exemption names it.
    assert!(section("319.301")
        .body
        .contains("including a tax levied under section 5705.199 or 5748.09"));
}

/// The H.B. 186 credit reduces only the taxes the floor holds up.
#[test]
fn the_inflation_cap_credit_reduces_only_floor_adjusted_taxes() {
    let s = section("319.303");
    assert!(s.legislation.contains("186"), "{}", s.legislation);
    assert!(s.body.contains("shall reduce only the taxes charged and payable of taxes whose effective tax rate is adjusted by operation of division (E) of section 319.301"));
    assert!(s.body.contains("gross domestic product deflator"));
    assert!(s.body.contains("Inflation Cap Credit"));
}
