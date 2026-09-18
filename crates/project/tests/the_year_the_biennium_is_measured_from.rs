//! FY2025, and what the department's final payment report settles that no calculator could.
//!
//! The baseline a "change in funding from FY2025 levels" claim is measured from. It arrives from
//! a payment report rather than a calculator because FY2025's calculator survives only in archive
//! captures truncated at 1 MiB — see `project::baseline` for why that is the better source anyway.
//!
//! Three things are asserted here. That the file agrees with what the corpus already knows from
//! other sources, which is what makes it trustworthy. That it answers an open question the corpus
//! could not close. And that its column letters are not the next year's, which is the trap a
//! reader keyed on the department's tags would fall into.

use project::baseline;
use project::panel::panel;

const DELPHOS: &str = "043885";
const SHAWNEE: &str = "045799";
const MIDDLE_BASS: &str = "048959";
const NORTH_BASS: &str = "048967";

/// Every district the FY2027 panel carries is in the payment report, and two more are.
///
/// The extras are not noise. `Summary_SFPR` holds 611 rows and the committed panel 609, and the
/// corpus recorded the difference without being able to name it — see the `[open — which two]` in
/// `funding-regime/fair-school-funding-plan`. They are the Lake Erie island districts.
#[test]
fn the_payment_report_is_the_panel_plus_the_two_island_districts() {
    let paid: std::collections::BTreeSet<String> =
        baseline::frame().into_iter().map(|row| row.irn).collect();
    let modelled: std::collections::BTreeSet<String> =
        panel().into_iter().map(|record| record.irn).collect();

    assert_eq!(modelled.len(), 609);
    assert_eq!(paid.len(), 611);
    assert!(
        modelled.is_subset(&paid),
        "every modelled district is in the payment report"
    );

    let extra: Vec<&String> = paid.difference(&modelled).collect();
    assert_eq!(extra, vec![MIDDLE_BASS, NORTH_BASS]);
}

/// **The $19,673 the corpus could not attribute is one district, not two.**
///
/// `fair-school-funding-plan` records the department's `Summary_SFPR` guarantee total as
/// $878,974,300 over 611 rows against $878,954,627 across the committed 609, and asks which two
/// districts the $19,673 belongs to. Two rows are missing; the money is North Bass Local's alone,
/// because Middle Bass is not held at its base at all.
///
/// The figure below is FY2025's. The FY2027 workbook's own `Summary_SFPR` states **$19,672.64**
/// for North Bass and **$0.00** for Middle Bass, which is the corpus's gap to the cent — that
/// check lives with the source in `connect`, and what this pins is the structural fact that only
/// one of the two is ever on the guarantee.
#[test]
fn only_one_of_the_two_island_districts_is_held_at_its_base() {
    let middle = baseline::at(MIDDLE_BASS).expect("Middle Bass is in the payment report");
    let north = baseline::at(NORTH_BASS).expect("North Bass is in the payment report");

    assert!(!middle.on_guarantee(), "Middle Bass draws no guarantee");
    assert!(north.on_guarantee(), "North Bass does");
    assert!(
        (north.guarantee - 13_115.75).abs() < 0.01,
        "North Bass FY2025 guarantee: {}",
        north.guarantee
    );
}

/// The repealed tier, as it was last paid.
///
/// The count is the check. `fsfp-targeted-assistance` says the programme paid "$52.5 million to
/// 36 districts" in FY2025, citing the LSC greenbook; the department's final payment report says
/// **$52,272,453** to the same **36**. The districts agree exactly and the dollars differ by
/// 0.4%, which is a greenbook figure against a settled one rather than a disagreement — and it is
/// why the corpus's sentence should keep its own source rather than adopt this one.
#[test]
fn the_repealed_supplemental_tier_is_here_at_its_last_payment() {
    let paid: Vec<f64> = baseline::frame()
        .into_iter()
        .map(|row| row.supplemental_targeted_assistance)
        .filter(|amount| *amount > 0.0)
        .collect();

    assert_eq!(paid.len(), 36, "districts paid the supplemental tier");
    let total: f64 = paid.iter().sum();
    assert!(
        (total - 52_272_453.0).abs() < 1.0,
        "FY2025 supplemental targeted assistance: {total}"
    );
    // The LSC greenbook's $52.5m, which the corpus states. Same programme, different vintage.
    assert!((total / 1e6 - 52.5).abs() < 0.3);
}

/// The two districts whose FSFP-versus-budget bars point opposite ways, at the year they start
/// from — and the fact that decides the direction.
///
/// Delphos City is held at its base in FY2025 and Shawnee Local is not. That is the difference a
/// guarantee-status test run on the FY2027 model cannot see, because by FY2027 both are on it:
/// Shawnee arrives on the guarantee during the biennium, Delphos was already there. A comparison
/// that reads only the terminal year reports them as alike.
#[test]
fn shawnee_reaches_the_guarantee_during_the_biennium_and_delphos_was_already_on_it() {
    let delphos = baseline::at(DELPHOS).expect("Delphos City");
    let shawnee = baseline::at(SHAWNEE).expect("Shawnee Local");

    assert!(
        delphos.on_guarantee(),
        "Delphos is held at its base in FY2025"
    );
    assert!(!shawnee.on_guarantee(), "Shawnee is not");

    // And both are on it by FY2027, which is why the terminal year cannot tell them apart.
    let modelled = panel();
    let on_guarantee = |irn: &str| {
        modelled
            .iter()
            .find(|record| record.irn == irn)
            .is_some_and(|record| record.guarantee_floor() > 0.0)
    };
    assert!(on_guarantee(DELPHOS) && on_guarantee(SHAWNEE));
}
