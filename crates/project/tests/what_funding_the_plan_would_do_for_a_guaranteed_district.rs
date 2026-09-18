//! The counterfactual that gives 356 districts money, and gives these two none.
//!
//! `draft-legislation/hb-96-with-refreshed-inputs` is this repository's reading of "fund the Fair
//! School Funding Plan as intended": the phase-in exactly as H.B. 96 enacted it, and the base cost
//! reference year refreshed from FY2022 to FY2024 rather than frozen. It prices at **+$220.5M**
//! across **356** districts.
//!
//! Delphos City and Shawnee Local are not among the 356. Both receive **nothing**, and the reason
//! is the whole point: a district whose aid is set by the guarantee is not paid by the formula, so
//! raising what the formula computes raises the amount the guarantee absorbs and not the amount
//! the district is paid. The refresh has to clear a district's entire guarantee headroom before it
//! pays it a cent.
//!
//! This is what makes "the plan as intended would have given district X more" false for a large
//! part of the state rather than merely imprecise. 253 of 609 districts are unmoved by a $220
//! million increase.

use project::panel::panel;
use project::policy::{apply_all, GuaranteeRule, Policy};

const DELPHOS: &str = "043885";
const SHAWNEE: &str = "045799";

/// The scale the corpus node states for a FY2024 refresh, and the reason it is not derived here.
///
/// `hb-96-with-refreshed-inputs` reproduces the department's own model for a classroom-teacher
/// refresh to FY2024 at this multiplier. Re-deriving it from salary tables would be a second,
/// worse estimate of a number the department already published.
const FY2024_REFRESH: f64 = 1.0395;

fn refreshed() -> Policy {
    Policy {
        base_cost_scale: FY2024_REFRESH,
        ..Policy::current_law()
    }
}

/// Neither district gains a cent from funding the plan as intended.
#[test]
fn the_refresh_pays_delphos_and_shawnee_nothing() {
    let outcomes = apply_all(&panel(), &refreshed());
    for irn in [DELPHOS, SHAWNEE] {
        let row = outcomes
            .iter()
            .find(|outcome| outcome.irn == irn)
            .expect("in the panel");
        assert!(
            row.on_guarantee,
            "{} ({irn}) is paid by the guarantee, not the formula",
            row.name
        );
        assert!(
            row.delta().abs() < 0.005,
            "{} ({irn}) moves by {} under a refresh that costs $220.5M",
            row.name,
            row.delta()
        );
    }
}

/// And the refresh is not small — it is simply spent on districts these two are not among.
///
/// The counts are the corpus node's own: 356 up, 0 down, 253 unmoved. Pinned here because the
/// claim above is only interesting if the lever demonstrably works, and a lever that moved nobody
/// would satisfy the test above for the wrong reason.
#[test]
fn the_same_refresh_moves_three_hundred_and_fifty_six_other_districts() {
    let outcomes = apply_all(&panel(), &refreshed());
    let up = outcomes.iter().filter(|row| row.delta() > 0.005).count();
    let down = outcomes.iter().filter(|row| row.delta() < -0.005).count();
    let flat = outcomes
        .iter()
        .filter(|row| row.delta().abs() <= 0.005)
        .count();

    assert_eq!((up, down, flat), (356, 0, 253));
    let total: f64 = outcomes.iter().map(project::policy::Outcome::delta).sum();
    assert!(
        (total - 220_525_319.0).abs() < 1.0,
        "statewide cost of the refresh: {total}"
    );
}

/// How far a refresh has to go before each district is paid anything, which is not the same
/// distance.
///
/// A guaranteed district's headroom is the gap between what the formula computes for it and the
/// base the guarantee holds it at, and a base cost scale has to close that whole gap before the
/// first dollar arrives. Shawnee Local clears at about **1.15** and Delphos City at about
/// **2.65** — a 15% refresh against a 165% one, for two districts a reader would put in the same
/// category because both are "on the guarantee".
///
/// Both are far past the 3.95% the counterfactual proposes, which is the point. The distance
/// between the two is the reason "on the guarantee" is too coarse a fact to reason from.
#[test]
fn the_two_districts_clear_the_guarantee_at_very_different_refreshes() {
    let modelled = panel();
    let first_paid = |irn: &str| {
        let mut scale = 1.0;
        loop {
            let policy = Policy {
                base_cost_scale: scale,
                ..Policy::current_law()
            };
            let moved = apply_all(&modelled, &policy)
                .into_iter()
                .find(|row| row.irn == irn)
                .expect("in the panel")
                .delta()
                > 0.005;
            if moved {
                return scale;
            }
            scale += 0.05;
            assert!(scale < 5.0, "{irn} is not paid at any scale below 5x");
        }
    };

    let (shawnee, delphos) = (first_paid(SHAWNEE), first_paid(DELPHOS));
    assert!(
        (shawnee - 1.15).abs() < 0.051,
        "Shawnee is first paid at {shawnee}"
    );
    assert!(
        (delphos - 2.65).abs() < 0.051,
        "Delphos is first paid at {delphos}"
    );
    // The gap is the finding, not either number on its own.
    assert!(delphos > shawnee * 2.0);

    // And the other direction is why nobody proposes retiring the guarantee instead: Delphos
    // loses more than a third of its aid the moment the floor is taken away.
    let without = Policy {
        guarantee: GuaranteeRule::Removed,
        ..Policy::current_law()
    };
    let removed = apply_all(&modelled, &without)
        .into_iter()
        .find(|row| row.irn == DELPHOS)
        .expect("Delphos is in the panel");
    assert!(
        removed.delta() < -1_000_000.0,
        "retiring the guarantee costs Delphos {}",
        removed.delta()
    );
}
