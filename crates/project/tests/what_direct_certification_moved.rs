//! What H.B. 96's direct-certification blend actually did to the distribution of DPIA.
//!
//! # The question, and where it came from
//!
//! A Fordham Institute commentary on the House-passed budget called the DPIA count "a mess" —
//! economically disadvantaged rates inflated by community eligibility until they no longer
//! distinguish a poor district from one that feeds everybody — and named **direct certification**
//! as the fix that would "significantly improve the allocation of funds". Three months later the
//! enacted act adopted it, 75/25 then 65/35. See
//! `.yidam/catalog/fordham-house-bridge-commentary.md`.
//!
//! So a recommendation and its enactment are both on the record, and the department's own FY2027
//! model publishes **both counts per district**, which makes the claim checkable rather than
//! arguable. This file checks it.
//!
//! # The answer is two numbers, and reporting one of them alone is the trap
//!
//! The provision does two things at once and they point opposite ways:
//!
//! - **It redistributes toward the intended beneficiaries.** Holding the statewide total fixed so
//!   that only the distribution can move, the blend sends the Ohio Eight **+$16.1m** against the
//!   pure economically-disadvantaged count. The districts paying for it are the ones the
//!   commentary described — Pickerington Local, an affluent Columbus suburb reporting 97.9%
//!   economically disadvantaged and 25.5% directly certified, is the largest single loser in the
//!   state.
//! - **It is also a cut**, and a larger one. Statewide DPIA for traditional districts falls
//!   $84.5m then $31.8m, $649.2m to $532.8m before phase-in, per the greenbook. The Ohio Eight
//!   hold 30.1% of enacted DPIA, so their share of that is roughly −$35m against +$16m of better
//!   targeting.
//!
//! A better measure of a smaller pot. Either half quoted alone is a different policy than the one
//! that passed.
//!
//! # Why the counterfactual rescales the index instead of recomputing it
//!
//! R.C. 3317.02(I)(1)(a)(i) defines the statewide economically disadvantaged percentage as a
//! *computation*, not a constant — so under a different count it would take a different value,
//! and holding it fixed while the counts rise 17.6% overstates the level effect badly. This
//! corpus cannot recompute it: the denominator implied by the published 0.5334 is 1,354,592,
//! which is **smaller than traditional enrolled ADM**, so whatever population it divides is not
//! one the FY2027 model carries. Recorded `[open]` on the component node.
//!
//! Rather than invent a denominator, the counterfactual here rescales the index to hold the
//! statewide dollar total *exactly* fixed. That isolates redistribution and declines to price the
//! level, which the greenbook prices instead. `scenario-models-ohio` is the decision that says a
//! provision moving all 609 districts at once may not hold a statewide average fixed and call the
//! result a cost.

use project::panel::{
    panel, DistrictRecord, DPIA_BLEND, DPIA_PER_PUPIL, DPIA_STATEWIDE_PERCENTAGE,
};

mod common;

/// The eight urban districts the commentary calls the Ohio Eight, by the model's own names.
const OHIO_EIGHT: [&str; 8] = [
    "Akron City",
    "Canton City",
    "Cincinnati Public Schools",
    "Cleveland Municipal",
    "Columbus City School District",
    "Dayton City",
    "Toledo City",
    "Youngstown City",
];

/// DPIA from a blended count, at a stated statewide index denominator.
///
/// `aid = d1 x $422 x (d1 / enrolled ADM / statewide)^2`, with `d1` capped at the district's own
/// enrolled ADM — the cap binds in exactly one district and is the reason Edgerton Local's
/// published blend is its enrolment rather than the weighted sum.
fn dpia(weighted_adm: f64, enrolled_adm: f64, statewide: f64) -> f64 {
    let d1 = weighted_adm.min(enrolled_adm);
    let d2 = d1 / enrolled_adm;
    d1 * DPIA_PER_PUPIL * (d2 / statewide).powi(2)
}

/// The enacted count: 65% of the FY2025 disadvantaged ADM, 35% of the directly certified one.
fn enacted_count(r: &DistrictRecord) -> f64 {
    DPIA_BLEND.0 * r.dpia.economically_disadvantaged_adm
        + DPIA_BLEND.1 * r.dpia.directly_certified_adm
}

/// The count the formula used before H.B. 96 rewrote it: the disadvantaged ADM alone.
///
/// This holds the *vintage* fixed at FY2025 and moves only the blend, which is what makes it a
/// single-provision counterfactual. The prior formula would also have used a current-year count,
/// and that second difference is not priced here because the model does not carry the year.
fn prior_count(r: &DistrictRecord) -> f64 {
    r.dpia.economically_disadvantaged_adm
}

/// **The mechanism reproduces the department's published DPIA column**, which is what licenses
/// every counterfactual below it.
///
/// Statewide to within 0.02%, and 607 of 609 districts inside half a percent. The two outside it
/// are Akron City at +0.53%, which is unestablished, and Kelleys Island Local at −0.59% on a $33
/// payment, which is rounding on the smallest DPIA figure in the state. Every other residual is a
/// rounding artefact and the median is four hundred-thousandths of a percent.
#[test]
fn the_squared_index_reproduces_the_published_column() {
    let panel = panel();

    let mut computed_total = 0.0;
    let mut published_total = 0.0;
    let mut outside_half_a_percent = Vec::new();

    for r in &panel {
        let computed = dpia(
            enacted_count(r),
            r.current_year_adm,
            DPIA_STATEWIDE_PERCENTAGE,
        );
        computed_total += computed;
        published_total += r.categoricals.dpia;

        if r.categoricals.dpia > 0.0 && ((computed / r.categoricals.dpia) - 1.0).abs() >= 0.005 {
            outside_half_a_percent.push(r.name.clone());
        }
    }

    let relative = (computed_total / published_total) - 1.0;
    assert!(
        relative.abs() < 0.0005,
        "statewide DPIA reproduces to {relative:+.5}, computed {computed_total:.0} against \
         published {published_total:.0}"
    );
    assert_eq!(
        outside_half_a_percent,
        vec!["Akron City".to_string(), "Kelleys Island Local".to_string()],
        "607 of 609 districts should sit inside half a percent"
    );
}

/// **Holding the statewide total fixed, the blend moves $16.1m to the Ohio Eight** — the
/// redistribution the commentary predicted, in the direction it predicted.
///
/// +11.3% and +$96 per pupil against the pure disadvantaged count. 423 of 609 districts gain.
#[test]
fn the_blend_redistributes_toward_the_ohio_eight() {
    let panel = panel();

    let enacted_total: f64 = panel
        .iter()
        .map(|r| {
            dpia(
                enacted_count(r),
                r.current_year_adm,
                DPIA_STATEWIDE_PERCENTAGE,
            )
        })
        .sum();
    let prior_total_at_published_index: f64 = panel
        .iter()
        .map(|r| {
            dpia(
                prior_count(r),
                r.current_year_adm,
                DPIA_STATEWIDE_PERCENTAGE,
            )
        })
        .sum();

    // Aid scales as the inverse square of the index denominator, so this is the exact rescale
    // that holds the statewide total equal. No search, and no tolerance to choose.
    let neutral =
        DPIA_STATEWIDE_PERCENTAGE * (prior_total_at_published_index / enacted_total).sqrt();

    let mut eight_enacted = 0.0;
    let mut eight_prior = 0.0;
    let mut eight_adm = 0.0;
    let mut gainers = 0;
    let mut neutral_total = 0.0;

    for r in &panel {
        let enacted = dpia(
            enacted_count(r),
            r.current_year_adm,
            DPIA_STATEWIDE_PERCENTAGE,
        );
        let prior = dpia(prior_count(r), r.current_year_adm, neutral);
        neutral_total += prior;

        if enacted > prior {
            gainers += 1;
        }
        if OHIO_EIGHT.contains(&r.name.as_str()) {
            eight_enacted += enacted;
            eight_prior += prior;
            eight_adm += r.current_year_adm;
        }
    }

    assert!(
        common::agrees_within(1.0, neutral_total, enacted_total),
        "the rescale must be level-neutral by construction, not approximately"
    );

    let moved = eight_enacted - eight_prior;
    assert!(
        (moved - 16_068_712.0).abs() < 100_000.0,
        "the Ohio Eight gain should be $16.07m, got {moved:.0}"
    );
    assert!(
        ((moved / eight_prior) - 0.113).abs() < 0.002,
        "which is +11.3% of what the prior count would have paid them"
    );
    assert!(
        ((moved / eight_adm) - 96.0).abs() < 2.0,
        "and +$96 per pupil across their {eight_adm:.0} enrolled ADM"
    );
    assert_eq!(gainers, 423, "districts better off under the blend");
}

/// **The districts that pay for it are the community-eligibility suburbs, exactly as described.**
///
/// Pickerington Local is the largest single loser in the state: 97.9% economically disadvantaged
/// against 25.5% directly certified, and −$3.08m. Cleveland Municipal is the largest gainer at
/// +$5.82m on a capture ratio of 0.65.
///
/// This is the finding that makes the commentary's diagnosis right rather than merely directional
/// — it names the mechanism (a district-wide free-meals election, not district-wide poverty) and
/// the mechanism identifies the districts.
#[test]
fn the_largest_loser_is_a_suburb_reporting_near_universal_disadvantage() {
    let panel = panel();

    let enacted_total: f64 = panel
        .iter()
        .map(|r| {
            dpia(
                enacted_count(r),
                r.current_year_adm,
                DPIA_STATEWIDE_PERCENTAGE,
            )
        })
        .sum();
    let prior_total: f64 = panel
        .iter()
        .map(|r| {
            dpia(
                prior_count(r),
                r.current_year_adm,
                DPIA_STATEWIDE_PERCENTAGE,
            )
        })
        .sum();
    let neutral = DPIA_STATEWIDE_PERCENTAGE * (prior_total / enacted_total).sqrt();

    let mut swings: Vec<(f64, &DistrictRecord)> = panel
        .iter()
        .map(|r| {
            let enacted = dpia(
                enacted_count(r),
                r.current_year_adm,
                DPIA_STATEWIDE_PERCENTAGE,
            );
            (
                enacted - dpia(prior_count(r), r.current_year_adm, neutral),
                r,
            )
        })
        .collect();
    swings.sort_by(|a, b| a.0.total_cmp(&b.0));

    let (worst, loser) = &swings[0];
    assert_eq!(loser.name, "Pickerington Local");
    assert!(
        (worst + 3_084_984.0).abs() < 50_000.0,
        "Pickerington should lose $3.08m, got {worst:.0}"
    );

    // The profile that produces it: nearly everybody counted, barely anybody certified.
    let reported = loser.dpia.economically_disadvantaged_adm / loser.current_year_adm;
    let certified = loser.dpia.directly_certified_adm / loser.dpia.economically_disadvantaged_adm;
    assert!(
        (reported - 0.979).abs() < 0.005,
        "reported share {reported:.3}"
    );
    assert!(
        (certified - 0.255).abs() < 0.005,
        "capture ratio {certified:.3}"
    );

    let (best, gainer) = &swings[swings.len() - 1];
    assert_eq!(gainer.name, "Cleveland Municipal");
    assert!(
        (best - 5_821_562.0).abs() < 100_000.0,
        "Cleveland should gain $5.82m, got {best:.0}"
    );
}

/// **The level effect is the larger term and points the other way.**
///
/// The Ohio Eight hold 30.1% of enacted DPIA. Against the greenbook's $116.4m statewide reduction
/// that is about −$35m, which more than cancels the +$16.1m of improved targeting.
///
/// The apportionment is an apportionment and not a department figure — the cut does not fall
/// pro-rata — so this test asserts the share and the sign of the comparison, which is what the
/// claim rests on, and not a district-level incidence nobody has computed.
#[test]
fn the_targeting_gain_is_smaller_than_their_share_of_the_cut() {
    let panel = panel();

    let statewide: f64 = panel.iter().map(|r| r.categoricals.dpia).sum();
    let eight: f64 = panel
        .iter()
        .filter(|r| OHIO_EIGHT.contains(&r.name.as_str()))
        .map(|r| r.categoricals.dpia)
        .sum();

    let share = eight / statewide;
    assert!(
        (share - 0.301).abs() < 0.005,
        "the Ohio Eight hold {share:.3} of enacted DPIA"
    );

    // $649.2m to $532.8m before phase-in, per the greenbook: $84.5m then $31.8m.
    const GREENBOOK_REDUCTION: f64 = 116_400_000.0;
    let their_share_of_the_cut = share * GREENBOOK_REDUCTION;
    const TARGETING_GAIN: f64 = 16_068_712.0;

    assert!(
        their_share_of_the_cut > TARGETING_GAIN,
        "a claim that direct certification helped the Ohio Eight has to clear their share of the \
         cut it arrived with: {their_share_of_the_cut:.0} against {TARGETING_GAIN:.0}"
    );
    assert!(
        (their_share_of_the_cut - TARGETING_GAIN - 18_985_836.0).abs() < 1_000_000.0,
        "net of both, about $19m worse off"
    );
}
