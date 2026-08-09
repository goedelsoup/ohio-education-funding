//! Five questions the corpus recorded as `[open]` and could have answered from data it held.
//!
//! Each of these sat in a node as an open item while the panel already carried everything needed
//! to settle it. That is a different kind of gap from the ones waiting on a connector, and worth
//! naming as such: the corpus was not blocked, it had simply not asked.
//!
//! The five, and what came back:
//!
//!   1. **Does gifted identification track district wealth?** Yes, strongly, and the better
//!      predictor is poverty rather than property. Recorded as `[open]` on `fsfp-gifted-units`.
//!   2. **Is the performance supplement large enough to matter distributionally?** Its gradient is
//!      real and its size is not. Recorded as `[open]` on `fsfp-performance-supplement`.
//!   3. **Does any district draw both enrolment supplements?** The question had a hidden premise:
//!      one of the two is universal. Recorded as worth checking on `fsfp-enrolment-supplements`.
//!   4. **Edgerton Local's DPIA blend.** Explained — a cap, and a data problem behind it.
//!      Recorded as "not understood" on `fsfp-disadvantaged-pupil-impact-aid`.
//!   5. **The twenty-mill floor's last untested candidate.** Partly answered, and the limit of
//!      what this data can say is stated rather than papered over.

use project::panel::{self, DistrictRecord};

const SD1: &str = include_str!("../../dispersion/fixtures/sd1-district-taxes.csv");

/// Pearson correlation.
fn correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let sx = x.iter().map(|v| (v - mx).powi(2)).sum::<f64>().sqrt();
    let sy = y.iter().map(|v| (v - my).powi(2)).sum::<f64>().sqrt();
    x.iter()
        .zip(y)
        .map(|(a, b)| (a - mx) * (b - my))
        .sum::<f64>()
        / (sx * sy)
}

/// Mean of `value` within each fifth of the panel ordered by `key`.
fn quintile_means(mut rows: Vec<(f64, f64)>) -> Vec<f64> {
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("no NaN"));
    let n = rows.len();
    (0..5)
        .map(|q| {
            let band = &rows[q * n / 5..(q + 1) * n / 5];
            band.iter().map(|(_, v)| v).sum::<f64>() / band.len() as f64
        })
        .collect()
}

/// Districts big enough for an identification *rate* to mean anything.
fn sizeable(panel: &[DistrictRecord]) -> impl Iterator<Item = &DistrictRecord> {
    panel.iter().filter(|r| r.current_year_adm >= 100.0)
}

fn identified_share(r: &DistrictRecord) -> f64 {
    (r.gifted.fte_k8 + r.gifted.fte_9_12) / r.current_year_adm
}

/// **1. Gifted identification rises with wealth and falls with poverty, steeply.**
///
/// Gifted funding scales off *identified* gifted FTE, so a district's gifted money depends on how
/// many children it identifies — which is a district practice as much as it is a pupil population.
/// The node asked whether identification rates vary systematically with district wealth and said
/// the corpus had not asked. They do.
///
/// Across wealth quintiles the identified share runs **10.1% to 19.0%** — the wealthiest fifth of
/// districts identify nearly twice the share the poorest fifth do. Against economic disadvantage
/// the gradient is steeper still and the correlation much stronger: **25.0% down to 8.6%**, a
/// factor of three, at r = −0.67 against r = +0.35 for property wealth.
///
/// # What this does not establish
///
/// Why. Three explanations fit the same numbers and this data cannot separate them: the underlying
/// distribution of the trait genuinely differs; identification practice differs, because screening
/// costs money and staff; or the assessments used to identify differ in what they measure. The
/// corpus should not pick one. What it can say is that a funding stream keyed to identification
/// pays most where identification is highest, and that is where poverty is lowest.
#[test]
fn gifted_identification_tracks_poverty_more_strongly_than_property() {
    let panel = panel::panel();

    let by_wealth: Vec<(f64, f64)> = sizeable(&panel)
        .filter_map(|r| Some((r.valuation_per_pupil?, identified_share(r))))
        .collect();
    let by_poverty: Vec<(f64, f64)> = sizeable(&panel)
        .map(|r| (r.dpia.percentage, identified_share(r)))
        .collect();
    assert!(by_wealth.len() > 500 && by_poverty.len() > 500);

    let wealth = quintile_means(by_wealth.clone());
    assert!((wealth[0] - 0.1009).abs() < 0.002, "Q1 {:.4}", wealth[0]);
    assert!((wealth[4] - 0.1896).abs() < 0.002, "Q5 {:.4}", wealth[4]);
    assert!(
        wealth.windows(2).all(|w| w[1] > w[0]),
        "identification should rise monotonically with wealth: {wealth:?}"
    );

    let poverty = quintile_means(by_poverty.clone());
    assert!((poverty[0] - 0.2500).abs() < 0.002, "Q1 {:.4}", poverty[0]);
    assert!((poverty[4] - 0.0855).abs() < 0.002, "Q5 {:.4}", poverty[4]);
    assert!(
        poverty.windows(2).all(|w| w[1] < w[0]),
        "identification should fall monotonically with disadvantage: {poverty:?}"
    );
    // Three times, against wealth's two: poverty is the sharper axis.
    assert!(poverty[0] / poverty[4] > wealth[4] / wealth[0]);

    let split = |rows: Vec<(f64, f64)>| {
        let (x, y): (Vec<f64>, Vec<f64>) = rows.into_iter().unzip();
        correlation(&x, &y)
    };
    let r_wealth = split(by_wealth);
    let r_poverty = split(by_poverty);
    assert!((r_wealth - 0.348).abs() < 0.01, "r wealth {r_wealth:.3}");
    assert!((r_poverty + 0.673).abs() < 0.01, "r poverty {r_poverty:.3}");
    assert!(
        r_poverty.abs() > r_wealth.abs() * 1.5,
        "poverty is the stronger predictor by a wide margin, not a narrow one"
    );
}

/// **2. The performance supplement's gradient is real and its size is not.**
///
/// The node asked whether $55.7m is large enough to matter distributionally, having established
/// that the supplement runs *against* the grain of a system that otherwise equalises. Both halves
/// of the answer matter and they point opposite ways.
///
/// The gradient is there: **$55.92 per pupil to the least disadvantaged fifth of districts against
/// $22.06 to the most**, a factor of 2.5. But the supplement is **0.77% of realized state aid**,
/// it reaches at most 7.0% of any single district's aid, and the 90th percentile district gets
/// 2.8%. A mechanism pointed the wrong way, turned down almost to nothing.
///
/// That is worth recording precisely because it is the kind of finding that reads as either
/// alarming or trivial depending on which of the two numbers is quoted alone.
#[test]
fn the_performance_supplement_is_regressive_and_too_small_to_matter() {
    let panel = panel::panel();
    let total: f64 = panel.iter().map(|r| r.performance.amount).sum();
    let paid = panel.iter().filter(|r| r.performance.amount > 0.0).count();
    assert!((total / 1e6 - 55.7).abs() < 0.5, "${:.1}m", total / 1e6);
    assert_eq!(paid, 427);

    let per_pupil = |r: &DistrictRecord| r.performance.amount / r.current_year_adm;
    let poverty = quintile_means(
        panel
            .iter()
            .filter(|r| r.current_year_adm > 0.0)
            .map(|r| (r.dpia.percentage, per_pupil(r)))
            .collect(),
    );
    assert!((poverty[0] - 55.92).abs() < 0.5, "Q1 {:.2}", poverty[0]);
    assert!((poverty[4] - 22.06).abs() < 0.5, "Q5 {:.2}", poverty[4]);
    assert!(
        poverty.windows(2).all(|w| w[1] < w[0]),
        "it should fall monotonically with disadvantage: {poverty:?}"
    );

    // And the size, which is the other half of the answer.
    let aid: f64 = panel.iter().map(DistrictRecord::realized_aid).sum();
    assert!(
        total / aid < 0.01,
        "the supplement is {:.4} of realized aid",
        total / aid
    );
    let mut shares: Vec<f64> = panel
        .iter()
        .filter(|r| r.realized_aid() > 0.0)
        .map(|r| r.performance.amount / r.realized_aid())
        .collect();
    shares.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    assert!(
        shares[shares.len() - 1] < 0.08,
        "the most exposed district gets {:.3} of its aid this way",
        shares[shares.len() - 1]
    );
}

/// **3. The two enrolment supplements are not alternatives — one is universal.**
///
/// The node said "whether any district draws both is worth checking rather than assuming", which
/// carries a premise worth checking too. The base funding supplement is $40 per pupil and **every
/// one of the 609 districts draws it**; the enrolment growth supplement reaches 43. So the growth
/// districts are a strict subset, and "both" is not a coincidence to be counted but a consequence
/// of one arm having no eligibility test at all.
///
/// The two are also nothing like each other in shape despite sharing a sheet: $56.1m spread across
/// everyone at a flat rate, against $39.4m concentrated on 43 districts behind a 3% cliff.
#[test]
fn one_enrolment_supplement_is_universal_and_the_other_is_a_cliff() {
    let panel = panel::panel();
    let base = panel
        .iter()
        .filter(|r| r.supplements.base_funding > 0.0)
        .count();
    let growth = panel.iter().filter(|r| r.supplements.growth > 0.0).count();
    let both = panel
        .iter()
        .filter(|r| r.supplements.base_funding > 0.0 && r.supplements.growth > 0.0)
        .count();

    assert_eq!(
        base,
        panel.len(),
        "the base supplement has no eligibility test"
    );
    assert_eq!(growth, 43);
    assert_eq!(
        both, growth,
        "every growth district draws base, necessarily"
    );
    assert_eq!(
        panel
            .iter()
            .filter(|r| r.supplements.base_funding <= 0.0)
            .count(),
        0
    );

    let base_total: f64 = panel.iter().map(|r| r.supplements.base_funding).sum();
    let growth_total: f64 = panel.iter().map(|r| r.supplements.growth).sum();
    assert!((base_total / 1e6 - 56.1).abs() < 0.5);
    assert!((growth_total / 1e6 - 39.4).abs() < 0.5);
    // Comparable money, wholly different concentration: $92k a district against $916k.
    assert!(growth_total / growth as f64 > (base_total / base as f64) * 8.0);
}

/// **4. Edgerton Local's DPIA blend is a cap, and there is a data problem behind it.**
///
/// The node recorded a published blend of 342.38 against 352.03 computed from the stated 65/35
/// weights, "recorded and not explained ... one district in 609 is the size of a data correction
/// rather than a mechanism, but it is not zero and it is not understood."
///
/// It is understood now, and it is a mechanism rather than a correction. **The weighted ADM is
/// capped at the district's own enrolled ADM.** Edgerton's published figure is 342.38 and its
/// enrolled ADM is 342.3818 — equal to the rounding of the published column. The blend of 352.03
/// would give it more disadvantaged pupils than pupils.
///
/// Three independent selections over the panel return the same single district, which is what
/// makes this an identification rather than a coincidence: the only district whose blend exceeds
/// its enrolment, the only one whose weighted ADM equals its enrolment, and the only one at a
/// disadvantaged percentage of ~100%.
///
/// # And the cause is upstream of the cap
///
/// Edgerton's economically disadvantaged ADM is **1.45 times its enrolled ADM** — 497.21 in a
/// district of 342.38. A district cannot have more disadvantaged pupils than pupils, so the input
/// is on a different basis or is wrong. The cap is what stops that becoming a nonsensical DPIA
/// index, and it is doing real work for exactly one district. What the ED count is counting
/// remains open; it is the department's own input and the calculator does not explain it.
#[test]
fn the_dpia_blend_is_capped_at_a_districts_own_enrolment() {
    let panel = panel::panel();
    let blend = |r: &DistrictRecord| {
        panel::DPIA_BLEND.0 * r.dpia.economically_disadvantaged_adm
            + panel::DPIA_BLEND.1 * r.dpia.directly_certified_adm
    };

    // Exactly one district fails the stated 65/35 identity.
    let failing: Vec<&DistrictRecord> = panel
        .iter()
        .filter(|r| (r.dpia.weighted_adm - blend(r)).abs() > 0.01)
        .collect();
    assert_eq!(failing.len(), 1);
    let edgerton = failing[0];
    assert_eq!(edgerton.name, "Edgerton Local");

    // And it is capped at enrolment, to the two decimals the column is published at.
    assert!(blend(edgerton) > edgerton.current_year_adm);
    assert!(
        (edgerton.dpia.weighted_adm - edgerton.current_year_adm).abs() < 0.01,
        "weighted {:.4} against enrolled {:.4}",
        edgerton.dpia.weighted_adm,
        edgerton.current_year_adm
    );

    // Three selections, one district each time. Any of them alone would be suggestive; together
    // they are the mechanism.
    let exceeds: Vec<&str> = panel
        .iter()
        .filter(|r| blend(r) > r.current_year_adm + 0.005)
        .map(|r| r.name.as_str())
        .collect();
    let equals: Vec<&str> = panel
        .iter()
        .filter(|r| (r.dpia.weighted_adm - r.current_year_adm).abs() < 0.005)
        .map(|r| r.name.as_str())
        .collect();
    let saturated: Vec<&str> = panel
        .iter()
        .filter(|r| r.dpia.percentage > 0.999)
        .map(|r| r.name.as_str())
        .collect();
    assert_eq!(exceeds, vec!["Edgerton Local"]);
    assert_eq!(equals, vec!["Edgerton Local"]);
    assert_eq!(saturated, vec!["Edgerton Local"]);

    // The cause, which the cap conceals: more disadvantaged pupils than pupils.
    let ratio = edgerton.dpia.economically_disadvantaged_adm / edgerton.current_year_adm;
    assert!(
        (ratio - 1.452).abs() < 0.01,
        "ED is {ratio:.3} of enrolment"
    );
}

/// **5. The twenty-mill floor governs the Class I rate cleanly, and the last candidate is not
/// needed to explain it.**
///
/// The node's remaining untested candidate was that emergency and other fixed-sum levies are
/// counted in voted operating millage but excluded from the floor computation. If that were the
/// general rule, districts at the floor would sit at 20 *plus* their fixed-sum mills, and very few
/// would land on 20 exactly.
///
/// **155 of 611 land on 20.0000 within a hundredth of a mill**, and the excess-over-twenty
/// distribution has its median at 0.0008. So for a quarter of the state the floor binds exactly
/// and nothing rides on top of it.
///
/// # What this does not settle, stated rather than glossed
///
/// 68 districts sit between 20.005 and 20.5 mills. Those are consistent with fixed-sum levies on
/// top *and* with districts genuinely a little above the floor that the factors have not finished
/// reducing. Separating the two needs levy-type data — which levies are emergency, which are
/// fixed-sum — and neither Table SD-1 nor the profile report carries it. So the candidate is
/// disconfirmed as a general rule and remains open for those 68.
#[test]
fn the_floor_binds_exactly_for_a_quarter_of_the_state() {
    let mut lines = SD1.lines();
    let head: Vec<&str> = lines.next().expect("a header").split(',').collect();
    let index = |name: &str| head.iter().position(|c| *c == name).expect("a column");
    let (year, rate) = (index("tax_year"), index("class1_rate"));

    let rates: Vec<f64> = SD1
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let p: Vec<&str> = l.split(',').collect();
            (p[year] == "2024").then(|| p[rate].parse().unwrap_or(0.0))
        })
        .collect();
    assert_eq!(rates.len(), 611);

    let at_floor = rates.iter().filter(|r| (**r - 20.0).abs() < 0.005).count();
    let just_above = rates.iter().filter(|r| **r > 20.005 && **r < 20.5).count();
    assert_eq!(at_floor, 155);
    assert_eq!(just_above, 68);

    // The shape of the cluster is the evidence. Under the candidate hypothesis the mass would sit
    // above twenty by the size of the excluded levies; instead it sits on it.
    let mut excess: Vec<f64> = rates
        .iter()
        .map(|r| r - 20.0)
        .filter(|e| (-0.005..3.0).contains(e))
        .collect();
    excess.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let median = excess[excess.len() / 2];
    assert!(
        median < 0.01,
        "the median district near the floor is {median:.4} mills above it"
    );
    assert!(
        at_floor > just_above * 2,
        "exact landings should dominate near-misses: {at_floor} against {just_above}"
    );
}
