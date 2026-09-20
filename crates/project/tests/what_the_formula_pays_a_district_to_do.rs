//! What Ohio's funding formula pays a district to do, at the margin.
//!
//! Every measurement of the hold-harmless system up to here has been backward-looking: which
//! districts are held, why, and what retiring a device would cost. These tests ask what the system
//! pays a district to do next year, and they answer it by perturbation — `policy::apply` already
//! takes the pupil count as an argument, so a margin is the difference between two runs.
//!
//! The finding is the ordering. **The three margins no district chooses are priced at zero, at
//! transportation and at a per-pupil amount; the two a board votes on are priced at the whole
//! statewide average base cost and at half a million dollars for one child.** 144 districts lose
//! exactly nothing when a pupil leaves, 294 lose no foundation aid, and statewide only 56.63% of
//! Ohio's per-pupil funding is actually per pupil. Against that, one open-enrolment FTE is worth
//! $8,241.61 to a guaranteed district — up to 25.80 times what its own child is worth — and the
//! pupil that clears `[M]`'s threshold for New Lexington is worth $516,984.45.
//!
//! `[K]` reverses no sign anywhere. It sets three of the four margins to zero, and the one it
//! cannot reach is the largest.

use project::margin::{self, Response, AVERAGE_YEARS, CENT, SHOCK, VALUATION_WEIGHT};
use project::panel::{panel, DistrictRecord, OPEN_ENROLLMENT_CLAWBACK_PER_FTE};
use project::policy::{Backstop, Policy};

/// The crates' median: the upper middle, never the mean of two.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty());
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Enrolled ADM across the panel, the denominator every share here is taken against.
fn enrolled_adm(panel: &[DistrictRecord]) -> f64 {
    panel.iter().map(|record| record.current_year_adm).sum()
}

/// The three regimes, their populations, and what each pays for one pupil fewer.
#[test]
fn the_marginal_pupil_falls_into_three_regimes() {
    let districts = panel();
    let margins = margin::pupil(&districts, &Policy::current_law());
    let total_adm = enrolled_adm(&districts);

    let of = |regime: Response| -> Vec<&margin::Pupil> {
        margins.iter().filter(|p| p.response == regime).collect()
    };
    let insulated = of(Response::Insulated);
    let held = of(Response::TransportationOnly);
    let formula = of(Response::Full);

    assert_eq!(insulated.len(), 144, "districts drawing [K]");
    assert_eq!(held.len(), 167, "on the guarantee and not drawing [K]");
    assert_eq!(formula.len(), 298, "on formula");
    assert_eq!(
        insulated.len() + held.len() + formula.len(),
        districts.len()
    );

    // The share of Ohio's children in a district whose cheque barely notices one of them.
    let share = |group: &[&margin::Pupil]| group.iter().map(|p| p.adm).sum::<f64>() / total_adm;
    assert!(
        (share(&insulated) - 0.2353).abs() < 0.0001,
        "{}",
        share(&insulated)
    );
    assert!((share(&held) - 0.3169).abs() < 0.0001, "{}", share(&held));
    assert!(
        (share(&formula) - 0.4478).abs() < 0.0001,
        "{}",
        share(&formula)
    );
    assert!(
        share(&insulated) + share(&held) > 0.55,
        "more than half of Ohio's pupils are in a district the floors insulate"
    );

    // The measurement agrees with the classification, which is read off two published columns.
    assert!(
        insulated.iter().all(|p| p.marginal.abs() < CENT),
        "[K] holds the district at a total, so nothing moves at all"
    );
    assert!(
        held.iter()
            .all(|p| p.foundation.abs() < CENT && p.marginal > CENT),
        "the guarantee holds [H] + [I] level and [J] is outside it"
    );
    assert!(
        formula.iter().all(|p| p.foundation > CENT),
        "nothing holds a formula district"
    );

    let median =
        |group: &[&margin::Pupil]| upper_middle(group.iter().map(|p| p.marginal).collect());
    assert!((median(&insulated) - 0.0).abs() < CENT);
    assert!((median(&held) - 503.40).abs() < CENT, "{}", median(&held));
    assert!(
        (median(&formula) - 8_516.08).abs() < CENT,
        "{}",
        median(&formula)
    );

    // 294 districts lose no foundation aid at all: the guaranteed ones, both regimes together.
    let no_foundation = margins.iter().filter(|p| p.foundation.abs() < CENT).count();
    assert_eq!(no_foundation, 294, "the districts the guarantee pays");
}

/// For a guaranteed district the marginal pupil costs its transportation aid and nothing else.
///
/// An identity rather than an approximation — worst residual across all 167 is four billionths of a
/// dollar, which is floating-point noise on a seven-figure quantity — because the guarantee holds
/// `[H] + [I]` level while `[J]` is outside the measure and follows the roll.
#[test]
fn the_guarantee_leaves_exactly_transportation() {
    let districts = panel();
    let margins = margin::pupil(&districts, &Policy::current_law());

    let held: Vec<_> = margins
        .iter()
        .filter(|p| p.response == Response::TransportationOnly)
        .collect();
    assert_eq!(held.len(), 167);

    let worst = held
        .iter()
        .map(|p| (p.marginal - p.transportation).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        worst < 1e-6,
        "not within a cent but within a millionth of one: {worst}"
    );
}

/// For a formula district the marginal pupil is the average pupil, and that is arithmetic.
///
/// Stated as a test so nobody reports it as a finding. R.C. 3317.017(B) multiplies a per-pupil
/// residual by current-year enrolled ADM, so base cost aid is linear in the count by statute, and
/// `policy::apply` scales the categoricals by the same ratio as its documented approximation. A
/// ratio of one carries no information; the zeros in the other two regimes carry all of it.
#[test]
fn the_formula_rows_ratio_is_one_by_construction() {
    let districts = panel();
    let margins = margin::pupil(&districts, &Policy::current_law());

    let formula: Vec<_> = margins
        .iter()
        .filter(|p| p.response == Response::Full)
        .collect();
    let worst = formula
        .iter()
        .map(|p| (p.ratio() - 1.0).abs())
        .fold(0.0_f64, f64::max);
    assert!(worst < 1e-9, "{worst}");
}

/// How much of Ohio's per-pupil funding is per pupil: 56.63% of it.
#[test]
fn two_fifths_of_the_per_pupil_amount_does_not_follow_the_pupil() {
    let districts = panel();
    let aggregate = margin::statewide(&districts, &Policy::current_law(), SHOCK);

    assert!(
        (aggregate.pupils - 14_019.4).abs() < 0.1,
        "{}",
        aggregate.pupils
    );
    assert!(
        (aggregate.saved - 45_704_433.29).abs() < CENT,
        "{}",
        aggregate.saved
    );
    assert!(
        (aggregate.per_pupil() - 3_260.09).abs() < CENT,
        "{}",
        aggregate.per_pupil()
    );
    assert!(
        (aggregate.average - 5_756.93).abs() < CENT,
        "{}",
        aggregate.average
    );
    assert!(
        (aggregate.share() - 0.5663).abs() < 0.0001,
        "{}",
        aggregate.share()
    );
}

/// The boundary is a kink and not a cliff, and in children it is thinly populated.
///
/// Realized aid is `max(formula aid, the FY2020 base)`, which is continuous: no district is better
/// off for sitting a dollar under its base. What changes at the edge is the slope, from one to
/// zero. #381 measured the population near it from inside, in multiples of the base; this measures
/// it from outside, in the unit a district actually loses.
#[test]
fn the_boundary_is_a_kink_and_six_districts_are_within_ten_pupils_of_it() {
    let districts = panel();
    let law = Policy::current_law();
    let headroom = margin::headroom(&districts, &law);

    assert_eq!(
        headroom.len(),
        315,
        "formula districts with a positive margin"
    );
    assert!(
        headroom.iter().all(|h| h.gap > 0.0),
        "a formula district is above its own floor by construction"
    );

    let pupils = upper_middle(headroom.iter().map(|h| h.pupils).collect());
    assert!((pupils - 206.6).abs() < 0.1, "{pupils}");
    assert_eq!(
        headroom.iter().filter(|h| h.pupils < 10.0).count(),
        6,
        "districts within ten pupils of their own floor"
    );

    let closest = headroom
        .iter()
        .min_by(|a, b| a.pupils.total_cmp(&b.pupils))
        .expect("the panel is not empty");
    assert_eq!(closest.name, "Waynesfield-Goshen Local");
    assert!(
        (closest.pupils - 2.3262).abs() < 0.001,
        "{}",
        closest.pupils
    );

    // The kink itself: crossing the boundary changes the slope and not the level. Take the
    // district closest to it, push it past the floor a pupil at a time, and the aid path is
    // continuous while the marginal response collapses to transportation alone.
    let record = districts
        .iter()
        .find(|r| r.irn == closest.irn)
        .expect("the headroom row came from the panel");
    let statewide = project::policy::Statewide::under(&districts, &law);
    let at = |adm: f64| project::policy::apply(record, &law, &statewide, adm);
    let steps: Vec<_> = (0..8)
        .map(|n| at(record.current_year_adm - f64::from(n)))
        .collect();
    for pair in steps.windows(2) {
        let step = pair[0].realized_aid - pair[1].realized_aid;
        assert!(
            (0.0..=closest.gap.max(20_000.0)).contains(&step),
            "no jump at the boundary, only a change of slope: {step}"
        );
    }
    assert!(
        steps.last().expect("eight steps").on_guarantee,
        "eight pupils takes this district onto its floor"
    );
    let before = steps[0].realized_aid - steps[1].realized_aid;
    let after = steps[6].realized_aid - steps[7].realized_aid;
    assert!(before > 1_000.0 && after < CENT, "{before} then {after}");
}

/// In years at each district's own trend, seventeen of them reach the floor inside one.
#[test]
fn seventeen_formula_districts_reach_their_floor_within_a_year() {
    let districts = panel();
    let headroom = margin::headroom(&districts, &Policy::current_law());

    let falling: Vec<f64> = headroom.iter().filter_map(|h| h.years).collect();
    assert_eq!(falling.len(), 238, "formula districts with a falling roll");

    let median = upper_middle(falling.clone());
    assert!((median - 10.1).abs() < 0.1, "{median}");
    assert_eq!(falling.iter().filter(|y| **y <= 1.0).count(), 17);
    assert_eq!(
        falling.iter().filter(|y| **y <= 4.0).count(),
        68,
        "inside two bienniums"
    );

    let soonest = headroom
        .iter()
        .filter(|h| h.years.is_some())
        .min_by(|a, b| {
            a.years
                .unwrap_or(f64::MAX)
                .total_cmp(&b.years.unwrap_or(f64::MAX))
        })
        .expect("238 of them");
    assert_eq!(soonest.name, "Franklin City");
    assert!(
        (soonest.pupils - 11.5228).abs() < 0.001,
        "{}",
        soonest.pupils
    );
}

/// `[M]` tests on the increment and pays on the roll, so the pupil at its edge carries the lot.
#[test]
fn the_pupil_that_clears_the_growth_threshold_is_worth_half_a_million() {
    let districts = panel();
    let mut cliff = margin::growth_cliff(&districts);
    assert_eq!(cliff.len(), 566, "districts short of the threshold");

    cliff.sort_by(|a, b| b.per_pupil().total_cmp(&a.per_pupil()));
    let top = &cliff[0];
    assert_eq!(top.name, "New Lexington School District");
    assert!((top.change - 0.029_502).abs() < 1e-6, "{}", top.change);
    assert!((top.short - 0.8327).abs() < 0.001, "{}", top.short);
    assert!((top.forgone - 430_476.80).abs() < CENT, "{}", top.forgone);
    assert!(
        (top.per_pupil() - 516_984.45).abs() < CENT,
        "{}",
        top.per_pupil()
    );

    assert_eq!(cliff.iter().filter(|c| c.short <= 1.0).count(), 2);
    assert_eq!(cliff.iter().filter(|c| c.short <= 10.0).count(), 15);

    // And it is two thousand times the rate the supplement is written at.
    assert!(
        top.per_pupil() / project::panel::ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL > 2_000.0,
        "a rule written at $250 a pupil prices one pupil at {}",
        top.per_pupil()
    );
}

/// A guaranteed district is paid up to twenty-six times more for another district's child.
///
/// `[I1]` takes the **whole statewide average base cost** off the guarantee for each
/// open-enrolment FTE lost beyond the threshold — not the district's state share of it — while the
/// guarantee is holding the district's own resident margin at transportation alone.
#[test]
fn the_clawback_charges_more_for_another_districts_child_than_the_formula_pays_for_its_own() {
    let districts = panel();
    let margins = margin::open_enrolment(&districts, &Policy::current_law());

    assert_eq!(margins.len(), 43, "districts charged an [I1] adjustment");
    assert_eq!(
        margins.iter().filter(|m| m.on_guarantee).count(),
        21,
        "the clawback only lands where there is a guarantee to take it from"
    );
    assert_eq!(
        margins
            .iter()
            .filter(|m| !m.on_guarantee && m.marginal.abs() > CENT)
            .count(),
        0,
        "[I] clamps at zero, so a district off the guarantee is charged nothing"
    );

    let exposed: Vec<_> = margins.iter().filter(|m| m.marginal.abs() > CENT).collect();
    assert_eq!(exposed.len(), 9, "[K] backstops the other twelve");
    assert!(
        exposed
            .iter()
            .all(|m| (m.marginal - OPEN_ENROLLMENT_CLAWBACK_PER_FTE).abs() < CENT),
        "the charge is the rate itself, undiminished"
    );

    let ratios: Vec<f64> = exposed.iter().filter_map(|m| m.over_resident()).collect();
    assert_eq!(ratios.len(), 9);
    let low = ratios.iter().copied().fold(f64::MAX, f64::min);
    let high = ratios.iter().copied().fold(f64::MIN, f64::max);
    assert!((low - 4.33).abs() < 0.01, "{low}");
    assert!((high - 25.80).abs() < 0.01, "{high}");
}

/// The formula charges a district for its tax base and never for its tax rate.
///
/// So the answer to "is a district paid for not passing a levy" is that it cannot be: the model has
/// 165 columns and not one of them is a rate any district votes on. `capacity_rate` is the sliding
/// scale R.C. 3317.017(A)(4) computes from income and `tax_returns` is a count of filers.
#[test]
fn the_model_carries_no_millage_column() {
    let header = include_str!("../../foundation/fixtures/fy27-department-model.csv")
        .lines()
        .next()
        .expect("the fixture has a header");
    let columns: Vec<&str> = header.split(',').collect();
    assert_eq!(columns.len(), 165);

    let rateish: Vec<&&str> = columns
        .iter()
        .filter(|column| {
            let lower = column.to_ascii_lowercase();
            lower.contains("mill") || lower.contains("levy") || lower.contains("rate")
        })
        .collect();
    assert_eq!(
        rateish,
        vec![&"capacity_rate"],
        "the only rate in the model is the one the statute computes from income"
    );
}

/// A dollar of assessed valuation costs 1.2 cents of state aid, and only for 268 districts.
#[test]
fn the_wealth_charge_is_a_penny_on_the_dollar_and_reaches_fewer_than_half_the_state() {
    let districts = panel();
    let charges = margin::wealth(&districts);

    assert_eq!(
        charges.len(),
        268,
        "the districts no floor already zeroes it for"
    );
    assert_eq!(
        districts.len() - charges.len(),
        341,
        "held, backstopped, or on the minimum state share"
    );

    let steady = upper_middle(charges.iter().map(|w| w.steady).collect());
    assert!((steady - 0.012_320).abs() < 1e-6, "{steady}");
    let low = charges.iter().map(|w| w.steady).fold(f64::MAX, f64::min);
    let high = charges.iter().map(|w| w.steady).fold(f64::MIN, f64::max);
    assert!((low - 0.008_064).abs() < 1e-6, "{low}");
    assert!((high - 0.014_848).abs() < 1e-6, "{high}");

    // It is the valuation weight times the capacity rate, amplified by the categoricals that
    // R.C. 3317.022 multiplies by the state share percentage. Nothing else is in it.
    for charge in &charges {
        let bare = VALUATION_WEIGHT * charge.rate;
        assert!(
            (charge.steady - bare * charge.slope).abs() < 1e-12,
            "{}",
            charge.name
        );
        assert!(charge.slope > 0.9, "{} {}", charge.name, charge.slope);
    }

    // The first year is a third of it, because R.C. 3317.017(A)(1)(a) charges the lesser of the
    // recent year and the three-year average and 602 of 609 districts are on the lagged branch.
    let lagged = districts
        .iter()
        .filter(|record| {
            let value = record.valuation_three_year;
            value[0] > (value[0] + value[1] + value[2]) / AVERAGE_YEARS
        })
        .count();
    assert_eq!(lagged, 602);
    let first = upper_middle(charges.iter().map(|w| w.first_year).collect());
    assert!((first - steady / AVERAGE_YEARS).abs() < 1e-9, "{first}");
}

/// H.B. 920 decides whether the value the formula charged for bought the district anything.
#[test]
fn above_the_twenty_mill_floor_a_reappraisal_is_a_pure_loss() {
    let districts = panel();
    let charges = margin::wealth(&districts);

    let at_floor: Vec<_> = charges
        .iter()
        .filter(|w| w.status.valuation_growth_reaches_revenue())
        .collect();
    let above: Vec<_> = charges
        .iter()
        .filter(|w| !w.status.valuation_growth_reaches_revenue())
        .collect();
    assert_eq!(at_floor.len(), 87);
    assert_eq!(above.len(), 181);

    // Above the floor R.C. 319.301 reduces the rate as the value rises, so existing property
    // yields nothing new — and R.C. 3317.017 charges for the higher value regardless.
    assert!(
        above
            .iter()
            .all(|w| w.local_yield().is_none() && w.recapture().is_none()),
        "no denominator, because there is no revenue"
    );
    let loss = upper_middle(above.iter().map(|w| w.steady).collect());
    assert!((loss - 0.012_063).abs() < 1e-6, "{loss}");

    // At the floor the value does yield revenue, and the state takes back five eighths of it.
    let recapture: Vec<f64> = at_floor.iter().filter_map(|w| w.recapture()).collect();
    assert_eq!(recapture.len(), at_floor.len());
    let median = upper_middle(recapture.clone());
    assert!((median - 0.6326).abs() < 0.0001, "{median}");
    let worst = recapture.iter().copied().fold(f64::MIN, f64::max);
    assert!((worst - 0.7424).abs() < 0.0001, "{worst}");
    assert!(
        worst < 1.0,
        "no district at the floor loses more aid than the value raises"
    );

    // The tolerance the split is taken at, pinned both ways because it is a choice. `millage`
    // defines the floor as a guarantee rather than a clamp, so at or under twenty mills is at it
    // and no tolerance is applied; allowing a hundredth of a mill moves seventeen districts.
    let tolerant = charges.iter().filter(|w| w.mills <= 20.01).count();
    assert_eq!(tolerant, 104);
    assert_eq!(tolerant - at_floor.len(), 17);
}

/// `[K]` reverses no sign. It sets three of the four margins to zero, and misses the largest.
#[test]
fn the_backstop_changes_no_sign_and_zeroes_three_margins_of_four() {
    let districts = panel();
    let law = Policy::current_law();
    let repealed = Policy {
        backstop: Backstop::Repealed,
        ..law
    };

    // No district in the panel is paid more for having fewer pupils, under either rule.
    for policy in [&law, &repealed] {
        assert_eq!(
            margin::pupil(&districts, policy)
                .iter()
                .filter(|p| p.marginal < -CENT)
                .count(),
            0,
            "[K] moves one for one against a total and never overshoots it"
        );
    }

    // What it does instead: 144 zeroes become 2, and those two are a fixture artefact rather than
    // a policy result — five districts publish no transportation components, so their
    // transportation does not follow the roll either.
    let with = margin::pupil(&districts, &law);
    let without = margin::pupil(&districts, &repealed);
    assert_eq!(with.iter().filter(|p| p.marginal.abs() < CENT).count(), 144);
    let left: Vec<&str> = without
        .iter()
        .filter(|p| p.marginal.abs() < CENT)
        .map(|p| p.name.as_str())
        .collect();
    assert_eq!(left, vec!["Grandview Heights Schools", "Lakewood City"]);
    let componentless = districts
        .iter()
        .filter(|r| {
            r.published_state_share.filter(|s| *s > 0.0).is_none()
                || r.transportation.components() <= 0.0
        })
        .count();
    assert_eq!(componentless, 5, "and two of them are on the guarantee");

    // The clawback margin is the one the backstop matters most for.
    assert_eq!(
        margin::open_enrolment(&districts, &law)
            .iter()
            .filter(|m| m.marginal.abs() > CENT)
            .count(),
        9
    );
    assert_eq!(
        margin::open_enrolment(&districts, &repealed)
            .iter()
            .filter(|m| m.marginal.abs() > CENT)
            .count(),
        21,
        "every guaranteed district the clawback reaches, once [K] is gone"
    );

    // And the margin it cannot reach. `[L]`, `[M]` and `[O]` are the only lines outside `[L1]`'s
    // subtrahend, so the largest price in the formula is paid into the hand of a district however
    // deeply the floors hold it.
    let cliff = margin::growth_cliff(&districts);
    let insulated_too: Vec<_> = cliff
        .iter()
        .filter(|c| {
            districts
                .iter()
                .find(|r| r.irn == c.irn)
                .is_some_and(|r| r.transition.transition_supplement > 0.0)
        })
        .collect();
    assert!(
        insulated_too.len() > 100,
        "{} districts drawing [K] are still short of [M], and crossing would pay them in full",
        insulated_too.len()
    );
}
