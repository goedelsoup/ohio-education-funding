//! What every counterfactual on this site holds fixed, and what that costs.
//!
//! The department's FY2027 calculator carries a note on its own simulation sheet:
//!
//! > Note: The statewide average values (such as economically disadvantaged percentage, median
//! > weighted wealth per-pupil or federal median income) are NOT recalculated based on the data
//! > changes specific to your district.
//!
//! That is a caveat on the department's tool, where a user changes one district. It is a **larger**
//! caveat on this repository, where `crates/project::policy` changes every district at once — and
//! the corpus had not recorded it anywhere.
//!
//! # The exposure was a quarter of the lever, and it is now mostly closed
//!
//! `Policy::base_cost_scale` expresses an input-year refresh: cost inputs restated from FY2018 to
//! FY2022 prices raised statewide base cost by about 3.1%. The model used to scale aggregate base
//! cost and stop.
//!
//! But **$858m of categorical funding is denominated in the statewide average base cost per
//! pupil** — special education, English learners, career-technical and the weighted half of
//! preschool special education are all `weight x $8,241.61 x count x state share`. If base cost per
//! pupil rises, that figure rises with it, and every one of those programs rises too.
//!
//! **The decision was taken: the site models Ohio rather than mirroring the department's tool.**
//! `base_cost_scale` now moves the denominated categoricals, and the divergence from the
//! department's simulator is deliberate and stated on the page. See
//! `.yidam/decisions/scenario-models-ohio.yml`.
//!
//! # What this file measures now
//!
//! Not an omission — a boundary. Three things are still held fixed and each for a different
//! reason, and the point of keeping this file is that the reasons are not interchangeable:
//!
//!   - **$45.7m of preschool special education** is denominated the same way and is *outside*
//!     `[H] Foundation Funding`, so it is outside `categorical_funding` and outside everything the
//!     scenario computes. Not held fixed by choice; simply not modelled.
//!   - **$54.4m of gifted** is exposed through salary prices rather than through base cost.
//!     Scaling it by this factor would be an assumption dressed as an identity.
//!   - **DPIA and targeted assistance**, $1.89bn, genuinely do largely cancel, because they are
//!     indices whose numerator and denominator move together.
//!
//! So $812m of the recorded $858m now moves, and the 5% that does not is a program the scenario
//! has never reached rather than a constant it declines to recompute.

use project::panel::{self, DistrictRecord};

/// The statewide constants a scenario cannot move, and the money that depends on each.
///
/// Every one of these is a single number that the department computes once over all districts and
/// then holds still. Under a change that moves every district — which is what a policy lever does —
/// each of them would move in reality.
#[test]
fn the_frozen_statewide_constants_are_named_and_their_exposure_measured() {
    let panel = panel::panel();
    let total = |pick: fn(&DistrictRecord) -> f64| panel.iter().map(pick).sum::<f64>();

    // The average base cost per pupil, $8,241.61. Four programs multiply it.
    let denominated = total(|r| r.special_education.total())
        + total(|r| r.english_learners.total())
        + total(|r| r.career_technical.total())
        + total(|r| r.preschool_special_education.total)
        - total(|r| r.preschool_special_education.flat_component());
    let base_cost_share = total(|r| r.base_cost_state_share);

    assert!(
        denominated > 800e6,
        "expected the base-cost-denominated categoricals to be around $858m, got {denominated:.0}"
    );

    // The exposure: what a refresh moves on top of base cost, as a fraction of base cost aid.
    let exposure = denominated / base_cost_share;
    assert!(
        (0.20..0.30).contains(&exposure),
        "a base cost refresh moves {exposure:.3} on top of itself; the recorded figure is 0.24 and \
         a materially different one means the mix has shifted and the note needs rewriting"
    );

    // **And the scenario now moves it.** The part inside `[H] Foundation Funding` is what
    // `Policy::base_cost_scale` reaches: $812m of the $858m, or 95%.
    let reachable: f64 = panel
        .iter()
        .map(DistrictRecord::base_cost_denominated_categoricals)
        .sum();
    assert!(
        (reachable / 1e6 - 812.5).abs() < 1.0,
        "reachable ${:.1}m",
        reachable / 1e6
    );
    assert!(
        reachable / denominated > 0.94,
        "the scenario should reach the great majority of the exposure, not {:.3}",
        reachable / denominated
    );

    // The residue is preschool special education's weighted half, and it is unreached because it
    // sits outside foundation funding rather than because a constant is being held still.
    let unreached = denominated - reachable;
    assert!(
        (unreached / 1e6 - 45.7).abs() < 1.0,
        "unreached ${:.1}m",
        unreached / 1e6
    );
    let categoricals: f64 = panel.iter().map(DistrictRecord::categorical_funding).sum();
    assert!(
        reachable < categoricals && unreached > 0.0,
        "the reachable part is inside categorical funding and the residue is not"
    );

    // The statewide poverty share DPIA indexes against, and the median wealth targeted assistance
    // divides by, are frozen the same way — but those are *indices*, so a uniform change moves
    // numerator and denominator together and largely cancels. That is why the exposure above is
    // concentrated in the weighted programs and not spread across all six.
    let index_driven =
        total(|r| r.categoricals.dpia) + total(|r| r.categoricals.targeted_assistance);
    assert!(
        index_driven > denominated,
        "the index-driven programs are larger than the denominated ones, which is why it matters \
         that they are the ones that cancel"
    );
}

/// Gifted is exposed too, and differently, which is worth separating.
///
/// Its unit prices — $85,776 a coordinator, $89,378 and $80,974 for the two specialist bands — are
/// salary assumptions rather than multiples of base cost per pupil. A cost-input refresh that
/// raised teacher salaries would raise these as well, but not by the same factor and not
/// automatically: nothing in the calculator links them.
///
/// So gifted belongs in the exposure with a caveat rather than in the headline figure, and the
/// distinction is exactly the one the `formula-component` nodes were split to preserve.
#[test]
fn gifted_is_exposed_through_prices_rather_than_through_a_base_cost() {
    let panel = panel::panel();
    let gifted: f64 = panel.iter().map(|r| r.gifted.total()).sum();
    let units: f64 = panel.iter().map(|r| r.gifted.unit_funding()).sum();

    assert!(
        units / gifted > 0.8,
        "gifted is {:.2} unit funding; if that fell the exposure would be through the per-pupil \
         amounts instead and this note would need rewriting",
        units / gifted
    );
    assert!(gifted > 40e6 && gifted < 70e6);
}

/// The check that the caveat is about a real dependency and not a theoretical one.
///
/// If the weighted categoricals were *not* denominated in the statewide average, this whole note
/// would be hypothetical. They are: dividing a district's published category aid by its count, its
/// weight and its state share recovers $8,241.61 — the same figure for every district and every
/// program, which is what makes it a shared constant rather than a coincidence.
#[test]
fn the_weighted_programs_really_do_share_one_statewide_multiplicand() {
    let panel = panel::panel();
    let weights = panel::SPECIAL_EDUCATION_WEIGHTS;
    let mut recovered: Vec<f64> = Vec::new();

    for record in &panel {
        let Some(share) = record.published_state_share else {
            continue;
        };
        // Special education Category 2 and English learner Category 1 are the two largest
        // populations, so they give the cleanest recovery.
        if record.special_education.adm[1] > 50.0 {
            recovered.push(
                record.special_education.aid[1]
                    / (record.special_education.adm[1] * weights[1] * share),
            );
        }
        if record.english_learners.adm[0] > 20.0 {
            recovered.push(
                record.english_learners.aid[0]
                    / (record.english_learners.adm[0] * panel::ENGLISH_LEARNER_WEIGHTS[0] * share),
            );
        }
    }

    assert!(recovered.len() > 300, "sample of {}", recovered.len());
    let low = recovered.iter().copied().fold(f64::INFINITY, f64::min);
    let high = recovered.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (high - low) < 1.0,
        "the recovered multiplicand ranges {low:.2} to {high:.2}; it should be one number"
    );
    assert!(
        (low - panel::AVERAGE_BASE_COST_PER_PUPIL).abs() < 1.0,
        "recovered {low:.2} against the transcribed {:.2}",
        panel::AVERAGE_BASE_COST_PER_PUPIL
    );
}
