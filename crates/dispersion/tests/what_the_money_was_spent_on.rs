//! The largest of the omissions `metric/progress-value-added` recorded, entered into the model.
//!
//! The node's `[open]` reads: "What remains unmodelled is not a list of minor omissions: district
//! typology, regional labour cost, and — the largest — **what the money was actually spent on**,
//! which no column in this workspace measures. A district that spends more on instruction and one
//! that spends more on transportation are the same observation here."
//!
//! `crates/dispersion/fixtures/expenditure-functions-fy25.csv` has measured it since it was
//! committed. It carries the same operating total the report card does, split into eleven
//! functions, per pupil, for 607 districts — and it had been wired to compare two named districts
//! and never entered into a regression.
//!
//! # What separating the money does
//!
//! The +0.209 the corpus publishes is two coefficients averaged under a constraint. Released, the
//! classroom half is +0.243 and everything else is −0.032. Composition also carries information
//! beyond level: holding total spending fixed, a point more classroom share associates with more
//! growth and with higher attainment — and on attainment the two run in opposite directions.
//!
//! The example the node chose turns out to be the wrong one. Pupil transportation is −0.035 and
//! indistinguishable from zero; plant operations and maintenance is the reliably negative
//! function.

use dispersion::composition::{self, District};

/// The join, and the fact that makes every coefficient below comparable to the published ones.
///
/// The function file's operating total and the report card's own per-enrolled-pupil column are
/// the same money over the same denominator. If they were not, this file would be fitting a
/// different model and reporting it as a decomposition of the corpus's.
#[test]
fn the_function_file_carries_the_same_total_the_published_model_uses() {
    let frame = composition::frame();
    assert_eq!(frame.len(), 606, "districts on all three panels");

    let cards: std::collections::BTreeMap<String, f64> = dispersion::report_card::report_cards()
        .into_iter()
        .filter_map(|c| Some((c.irn.clone(), c.per_enrolled_pupil()?)))
        .collect();
    let mut worst: f64 = 0.0;
    for district in &frame {
        let published = cards[&district.irn];
        worst = worst.max((district.operating - published).abs());
    }
    assert!(
        worst < 1.0,
        "the two operating totals differ by up to ${worst:.2} per pupil"
    );

    // And the roll-ups partition it, which is what licenses reading a share.
    for district in &frame {
        assert!(
            (district.classroom + district.nonclassroom - district.operating).abs() < 0.02,
            "{} does not partition: {} + {} against {}",
            district.name,
            district.classroom,
            district.nonclassroom,
            district.operating
        );
    }
}

/// **The finding.** The growth relationship is the classroom half of the money and nothing else.
#[test]
fn the_relationship_is_carried_entirely_by_classroom_spending() {
    let frame = composition::frame();
    let split = composition::split_on_growth(&frame);

    assert!(
        (split.standardized[0] - 0.243).abs() < 0.006,
        "classroom on growth is {:+.4}",
        split.standardized[0]
    );
    assert!(
        (split.standardized[1] + 0.032).abs() < 0.006,
        "non-classroom on growth is {:+.4}",
        split.standardized[1]
    );

    // One is detectable and the other is not, which is the whole of the claim.
    assert!(
        split.t_statistics[1] > 4.0,
        "classroom t = {:+.2}",
        split.t_statistics[1]
    );
    assert!(
        split.t_statistics[2].abs() < 2.0,
        "non-classroom t = {:+.2}, which would be a relationship",
        split.t_statistics[2]
    );

    // The published total sits between them, which is what a constrained average looks like.
    let total = composition::fit(&frame, &[("operating per pupil", |d| d.operating)], |d| {
        d.growth
    });
    assert!(
        (total.standardized[0] - 0.209).abs() < 0.006,
        "the published coefficient does not replicate here: {:+.4}",
        total.standardized[0]
    );
    assert!(
        split.standardized[1] < total.standardized[0]
            && total.standardized[0] < split.standardized[0],
        "the total should lie between the two halves"
    );
}

/// And how a district spends predicts something that how much does not.
#[test]
fn composition_carries_information_beyond_the_level_of_spending() {
    let frame = composition::frame();

    let growth = composition::composition_on(&frame, |d| d.growth);
    assert!(
        (growth.standardized[0] - 0.231).abs() < 0.006
            && (growth.standardized[1] - 0.122).abs() < 0.006,
        "growth: total {:+.4}, share {:+.4}",
        growth.standardized[0],
        growth.standardized[1]
    );
    assert!(
        growth.t_statistics[2] > 2.0,
        "share t = {:+.2}",
        growth.t_statistics[2]
    );

    // **The reversal.** On attainment level the two signs differ, in the same model.
    let level = composition::composition_on(&frame, |d| d.level);
    assert!(
        (level.standardized[0] + 0.056).abs() < 0.006
            && (level.standardized[1] - 0.095).abs() < 0.006,
        "level: total {:+.4}, share {:+.4}",
        level.standardized[0],
        level.standardized[1]
    );
    assert!(
        level.standardized[0] < 0.0 && level.standardized[1] > 0.0,
        "the level model no longer separates how much from how"
    );
    assert!(
        level.t_statistics[2] > level.t_statistics[1].abs(),
        "composition should be the firmer of the two on level"
    );
}

/// The three checks the composition result has to survive, and the one it does not.
#[test]
fn the_composition_result_holds_except_on_the_halved_sample() {
    let frame = composition::frame();

    // Year-matched: the published Progress measure averages three years and the spending is one.
    let one_year = composition::composition_on(&frame, |d| d.growth_one_year);
    assert!(
        one_year.standardized[1] > 0.0 && one_year.t_statistics[2] > 2.0,
        "one-year growth: share {:+.4}, t {:+.2}",
        one_year.standardized[1],
        one_year.t_statistics[2]
    );

    // Food service is largely federally reimbursed and scales with poverty, so it could put need
    // into the share mechanically. Out of the denominator, the coefficient barely moves.
    let ex_food = composition::fit(
        &frame,
        &[
            ("operating per pupil", |d| d.operating),
            (
                "share excluding food",
                District::classroom_share_excluding_food,
            ),
        ],
        |d| d.growth,
    );
    assert!(
        (ex_food.standardized[1] - 0.103).abs() < 0.006 && ex_food.t_statistics[2] > 2.0,
        "excluding food service: share {:+.4}, t {:+.2}",
        ex_food.standardized[1],
        ex_food.t_statistics[2]
    );

    // Rurality is the obvious confounder — a district that buses children far cannot spend that
    // money twice. Entered directly, transportation explains nothing and the share survives.
    let with_transport = composition::on_growth_with_transport(&frame);
    assert!(
        (with_transport.standardized[1] - 0.112).abs() < 0.006
            && with_transport.t_statistics[2] > 2.0,
        "with transport: share {:+.4}, t {:+.2}",
        with_transport.standardized[1],
        with_transport.t_statistics[2]
    );
    assert!(
        with_transport.t_statistics[3].abs() < 2.0,
        "transportation spending t = {:+.2}",
        with_transport.t_statistics[3]
    );

    // **And the check it does not pass.** On the districts whose English-learner share the
    // department did not suppress, the estimate barely moves and the sample halves, so the
    // statistic falls under the conventional bar. Asserted rather than omitted.
    let complete: Vec<District> = frame
        .iter()
        .filter(|d| !d.english_learner_imputed)
        .cloned()
        .collect();
    assert!(
        complete.len() < frame.len() / 2 + 10,
        "the suppression is extensive"
    );
    let restricted = composition::composition_on(&complete, |d| d.growth);
    assert!(
        restricted.standardized[1] > 0.05 && restricted.t_statistics[2] < 2.0,
        "complete cases: share {:+.4}, t {:+.2} — if this now clears 2.0 the caveat in \
         `progress-value-added` is stale",
        restricted.standardized[1],
        restricted.t_statistics[2]
    );
    // The level of spending goes the other way on the same subsample, which is why this reads as
    // power rather than as the composition result failing.
    assert!(
        restricted.standardized[0] > growth_total(&frame),
        "spending: {:+.4} restricted against {:+.4} full",
        restricted.standardized[0],
        growth_total(&frame)
    );
}

/// Function by function, and the example the corpus chose was the wrong one.
#[test]
fn transportation_is_not_what_separates_them_and_building_maintenance_is() {
    let frame = composition::frame();
    let fit = composition::every_function_on_growth(&frame);
    assert_eq!(composition::FUNCTIONS.len(), 9);

    let of = |name: &str| {
        let index = composition::FUNCTIONS
            .iter()
            .position(|(label, _)| *label == name)
            .expect("a named function");
        (fit.standardized[index], fit.t_statistics[index + 1])
    };

    let (instruction, instruction_t) = of("instruction");
    assert!(
        (instruction - 0.194).abs() < 0.006 && instruction_t > 4.0,
        "instruction {instruction:+.4}, t {instruction_t:+.2}"
    );

    // The node's own example. It is not distinguishable from zero.
    let (transport, transport_t) = of("pupil transportation");
    assert!(
        transport.abs() < 0.06 && transport_t.abs() < 2.0,
        "transportation {transport:+.4}, t {transport_t:+.2}"
    );

    // And the function that is.
    let (plant, plant_t) = of("operations and maintenance");
    assert!(
        (plant + 0.126).abs() < 0.006 && plant_t < -2.0,
        "plant {plant:+.4}, t {plant_t:+.2}"
    );
    assert!(
        plant < transport,
        "plant should be the more negative of the two"
    );
}

/// What the classroom share is, before anything is read off a coefficient on it.
///
/// It varies about a third as much as the level of spending does, and it is not a wealth
/// variable — which is the pairing with `wealth_predicts_attainment_but_not_growth` worth having.
#[test]
fn the_share_varies_far_less_than_the_level_and_is_not_a_wealth_measure() {
    let frame = composition::frame();
    let shares: Vec<f64> = frame.iter().map(District::classroom_share).collect();
    let totals: Vec<f64> = frame.iter().map(|d| d.operating).collect();

    let share = dispersion::Dispersion::of(&shares).expect("districts report a share");
    let total = dispersion::Dispersion::of(&totals).expect("districts report a total");
    assert!(
        (share.median - 67.0).abs() < 0.1,
        "the median classroom share is {:.2} points",
        share.median
    );
    assert!(
        share.coefficient_of_variation < total.coefficient_of_variation / 2.0,
        "composition spreads {:.4} against the level's {:.4}",
        share.coefficient_of_variation,
        total.coefficient_of_variation
    );

    let correlate = |pick: fn(&District) -> f64| {
        let other: Vec<f64> = frame.iter().map(pick).collect();
        dispersion::wealth_neutrality(&shares, &other)
            .expect("paired")
            .correlation
    };

    // Property wealth does not predict how a district splits its money at all.
    assert!(
        correlate(|d| d.valuation).abs() < 0.05,
        "valuation against share is {:+.4}",
        correlate(|d| d.valuation)
    );
    // Spending more buys a *smaller* classroom fraction, which is why the total understates the
    // classroom coefficient rather than overstating it.
    assert!(
        correlate(|d| d.operating) < -0.25,
        "spending against share is {:+.4}",
        correlate(|d| d.operating)
    );
    // And size and busing are what it does track.
    assert!(correlate(|d| d.log_enrolment) > 0.3);
    assert!(correlate(|d| d.pupil_transportation) < -0.3);
}

/// The published total-spending coefficient on growth, for the subsample comparison above.
fn growth_total(frame: &[District]) -> f64 {
    composition::composition_on(frame, |d| d.growth).standardized[0]
}
