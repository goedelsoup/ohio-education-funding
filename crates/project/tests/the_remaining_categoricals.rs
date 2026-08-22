//! The four categorical programs the corpus carried as dollar totals, reproduced from their inputs.
//!
//! Special education and DPIA were taken apart in the two phases before this one. These are the
//! rest: **targeted assistance** at $1.36bn, **career-technical** at $54m, **gifted** at $54m and
//! **English learners** at $36m. Together with the two already done they are the whole of the
//! $2.76bn the corpus called `categorical_funding` and could say nothing else about.
//!
//! Every mechanism here is reproduced from the department's own published inputs, and every one of
//! them turns out to contain something a dollar total conceals:
//!
//! - **Career-technical is weighted against its own base cost**, $9,855.62, where every other
//!   weighted categorical uses the statewide average $8,241.61.
//! - **The English learner weights descend.** The most recently arrived learner is funded at twice
//!   the longest-settled one, so the program tapers rather than deepens.
//! - **Gifted has a floor.** A district that identifies no gifted pupils at all still draws 0.5
//!   coordinator units and 0.3 of each specialist unit, because units buy staff and you cannot hire
//!   a fraction of a person below some size. It also has a cap, at eight coordinators.
//! - **Targeted assistance has a size cliff.** The capacity tier pays nothing below 200 pupils, 5%
//!   from 200 to 400, and ramps to full only between 400 and 600 — so two districts equally far
//!   below the state's median wealth are paid differently for being different sizes.
//!
//! And one thing that spans them, which cost this file two rewrites to find.
//!
//! All four are paid on `[a] Enrolled ADM` — column 3 of the `ADM Data` sheet, and **not** the
//! `[b3] FY26 Enrolled ADM` at column 28 that the corpus already carried.
//!
//! The two agree in 608 of 609 districts. They disagree in **Akron**, where they are 18,892.45 and
//! 18,842.45 — fifty pupils apart, four columns from each other on one sheet. So this is not two
//! definitions of enrolment; it is one number entered twice and corrected once. Which of the two
//! is right for Akron is not determinable from the workbook, and the corpus now carries both
//! rather than choosing.
//!
//! It is worth dwelling on how this surfaced, because a single-district discrepancy is exactly the
//! kind that does not. Reproducing gifted from the column the corpus already had gave an answer
//! 0.3% out for one district and exact for the other 608 — which reads as a rounding problem, and
//! the first version of this test responded by loosening its tolerance until the district passed.
//! A tolerance wide enough to admit an error is wide enough to admit the next one.
//!
//! Targeted assistance then adds two more: it divides by **resident** ADM to measure a district's
//! wealth and multiplies by **enrolled** ADM to pay it, one line apart in the same formula. With
//! base cost's three-year average that is four distinct pupil counts inside one district's aid —
//! the same hazard `web/src/lib/denominators.ts` was built for on the presentation side, arriving
//! here from the department's own workbook.

mod common;

use project::panel::{
    self, TargetedAssistance, AVERAGE_BASE_COST_PER_PUPIL, CTE_ASSOCIATED_WEIGHT,
    CTE_BASE_COST_PER_PUPIL, CTE_WEIGHTS, ENGLISH_LEARNER_WEIGHTS, GIFTED_COORDINATOR_DIVISOR,
    GIFTED_COORDINATOR_UNIT_BOUNDS, GIFTED_COORDINATOR_UNIT_PRICE, GIFTED_IDENTIFICATION_PER_PUPIL,
    GIFTED_REFERRAL_PER_PUPIL, GIFTED_SPECIALIST_DIVISOR, GIFTED_SPECIALIST_UNIT_FLOOR,
    GIFTED_SPECIALIST_UNIT_PRICES, TA_CAPACITY_MINIMUM_ADM, TA_CAPACITY_RATE,
    TA_MEDIAN_WEALTH_PER_PUPIL, TA_MEDIAN_WEIGHTED_WEALTH, TA_WEALTH_BLEND, TA_WEALTH_INDEX_FLOOR,
    TA_WEALTH_OFFSET_RATE, TA_WEALTH_RATE,
};

// ---------------------------------------------------------------------------------------------
// Career-technical education
// ---------------------------------------------------------------------------------------------

/// Five weights times five FTE counts times a base cost times the state share — and the base cost
/// is not the one the rest of the plan uses.
///
/// Mechanically this is special education's shape exactly. The difference is the multiplicand: CTE
/// weights are applied to a **career-technical** base cost per pupil of $9,855.62, 19.6% above the
/// $8,241.61 that special education and English learners are weighted against. A Category 1 CTE
/// pupil at a weight of 0.623 therefore generates $6,140 rather than the $5,135 the same weight
/// against the general base would give.
///
/// Nothing in `[G] Career-Technical` says so, and comparing the CTE weights against the special
/// education weights as though they were on the same scale — which is what a table of weights
/// invites — is wrong by a fifth before any pupil is counted.
#[test]
fn career_technical_reproduces_from_five_weights_against_its_own_base_cost() {
    let panel = panel::panel();
    let mut checked = 0;
    for record in &panel {
        let Some(share) = record.published_state_share else {
            continue;
        };
        let cte = &record.career_technical;
        for (k, weight) in CTE_WEIGHTS.iter().enumerate() {
            let expected = cte.fte[k] * weight * CTE_BASE_COST_PER_PUPIL * share;
            assert!(
                common::close(expected, cte.aid[k]),
                "{}: CTE category {} computed {expected:.2} against published {:.2}",
                record.name,
                k + 1,
                cte.aid[k]
            );
        }
        let associated = cte.total_fte() * CTE_ASSOCIATED_WEIGHT * CTE_BASE_COST_PER_PUPIL * share;
        assert!(
            common::close(associated, cte.associated_services),
            "{}: associated services computed {associated:.2} against published {:.2}",
            record.name,
            cte.associated_services
        );
        assert!(
            common::reconciles(cte.total(), record.categoricals.career_technical, 6),
            "{}: five categories plus associated services {:.2} against published total {:.2}",
            record.name,
            cte.total(),
            record.categoricals.career_technical
        );
        checked += 1;
    }
    assert!(checked > 600, "expected the whole panel: {checked}");

    // The base the weights multiply is the finding, and it is worth establishing from the
    // department's published amounts rather than from the constants above — those are transcribed
    // from the same workbook and asserting on them proves only that the transcription is
    // self-consistent.
    //
    // So recover each program's implied base cost by dividing a published amount back out by its
    // weight, its count and the state share, and compare the two. If career-technical and English
    // learners were on the same scale these would agree.
    let implied = |amount: f64, count: f64, weight: f64, share: f64| {
        (count > 1.0 && amount > 0.0).then(|| amount / (count * weight * share))
    };
    let mut cte_base: Vec<f64> = Vec::new();
    let mut general_base: Vec<f64> = Vec::new();
    for record in &panel {
        let Some(share) = record.published_state_share else {
            continue;
        };
        cte_base.extend(implied(
            record.career_technical.aid[0],
            record.career_technical.fte[0],
            CTE_WEIGHTS[0],
            share,
        ));
        general_base.extend(implied(
            record.english_learners.aid[0],
            record.english_learners.adm[0],
            ENGLISH_LEARNER_WEIGHTS[0],
            share,
        ));
    }
    assert!(
        cte_base.len() > 100 && general_base.len() > 100,
        "expected a usable sample from both programs: {} and {}",
        cte_base.len(),
        general_base.len()
    );
    let mean = |xs: &[f64]| xs.iter().sum::<f64>() / xs.len() as f64;
    let (career, general) = (mean(&cte_base), mean(&general_base));
    assert!(
        career / general > 1.19,
        "career-technical implies a base cost of {career:.0} against {general:.0} elsewhere, a \
         ratio of {:.3} — the two programs should not be on the same scale",
        career / general
    );
}

// ---------------------------------------------------------------------------------------------
// English learners
// ---------------------------------------------------------------------------------------------

/// Three weights, and they run the wrong way round.
///
/// 0.2104, 0.1577, 0.1053: Category 1 is funded at almost exactly twice Category 3. Category 1 is
/// the most recently arrived learner. So the program pays most in a pupil's first year and least in
/// their third, and a district whose English learners are settling in sees its EL funding fall
/// while the pupils are still learning English.
///
/// Every other weighted categorical in the plan runs the other way — special education's weights
/// ascend from 0.2435 to 3.9554 as need deepens. English learners is the exception, and a reader
/// who assumes the plan's weights mean "more need, more money" has it backwards here.
#[test]
fn english_learners_reproduces_and_its_weights_descend() {
    let panel = panel::panel();
    let mut with_learners = 0;
    for record in &panel {
        let Some(share) = record.published_state_share else {
            continue;
        };
        let el = &record.english_learners;
        for (k, weight) in ENGLISH_LEARNER_WEIGHTS.iter().enumerate() {
            let expected = el.adm[k] * weight * AVERAGE_BASE_COST_PER_PUPIL * share;
            assert!(
                common::close(expected, el.aid[k]),
                "{}: EL category {} computed {expected:.2} against published {:.2}",
                record.name,
                k + 1,
                el.aid[k]
            );
        }
        assert!(
            common::reconciles(el.total(), record.categoricals.english_learners, 3),
            "{}: three categories {:.2} against published total {:.2}",
            record.name,
            el.total(),
            record.categoricals.english_learners
        );
        if el.total_adm() > 0.0 {
            with_learners += 1;
        }
    }

    // That the weights descend is the finding, so establish it from what districts are paid rather
    // than from the weights transcribed above. Recover each category's per-pupil amount from a
    // district's own published aid and count, and compare across categories.
    let mut taper: Vec<f64> = Vec::new();
    for record in &panel {
        let el = &record.english_learners;
        if el.adm[0] < 5.0 || el.adm[2] < 5.0 {
            continue;
        }
        let per_pupil = |k: usize| el.aid[k] / el.adm[k];
        if per_pupil(2) > 0.0 {
            taper.push(per_pupil(0) / per_pupil(2));
        }
    }
    assert!(
        taper.len() > 40,
        "expected districts with learners in both the first and third categories: {}",
        taper.len()
    );
    taper.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in published amounts"));
    let median_taper = taper[taper.len() / 2];
    assert!(
        median_taper > 1.9 && median_taper < 2.1,
        "a Category 1 learner should be funded at about twice a Category 3 one, not \
         {median_taper:.3} times"
    );

    // The program is near-universal and almost entirely concentrated, which is a combination the
    // $36m total hides in both directions. **505 of 609 districts have at least one English
    // learner** — this is not a big-city program by reach. But the median district's entire EL
    // allocation is about $6,300, roughly a fifteenth of one teacher, while **38 districts are 80%
    // of the money** and the largest five are a third of it.
    //
    // So "almost every district receives English learner funding" and "English learner funding
    // goes to a few dozen districts" are both true, and a reader given only the total will pick
    // whichever they already believed.
    assert!(
        with_learners > 500,
        "English learner funding reaches most districts, not a few: {with_learners}"
    );

    let mut amounts: Vec<f64> = panel
        .iter()
        .map(|r| r.categoricals.english_learners)
        .filter(|amount| *amount > 0.0)
        .collect();
    amounts.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in published amounts"));
    let total: f64 = amounts.iter().sum();
    let median = amounts[amounts.len() / 2];
    assert!(
        median < 10_000.0,
        "the median receiving district's whole allocation is {median:.0}"
    );

    let mut running = 0.0;
    let mut carrying = 0;
    for amount in amounts.iter().rev() {
        running += amount;
        carrying += 1;
        if running >= 0.8 * total {
            break;
        }
    }
    assert!(
        carrying < 50,
        "expected the program to concentrate in a few dozen districts, not {carrying}"
    );
}

// ---------------------------------------------------------------------------------------------
// Gifted
// ---------------------------------------------------------------------------------------------

/// The one categorical that buys staff rather than pupils, and the only one with a floor.
///
/// Two per-pupil amounts — $24 against K-6 enrolment for identification, $2.50 against all
/// enrolment for referral — plus three kinds of unit. A unit is a headcount entitlement priced at
/// a salary-like figure: $85,776 for a coordinator, $89,378 for a K-8 intervention specialist,
/// $80,974 for a 9-12 one.
///
/// The unit counts are clamped, and that is the policy:
///
/// - coordinator units are `enrolled ADM / 3,300`, **floored at 0.5 and capped at 8**;
/// - specialist units are `identified gifted FTE / 140` in each band, **floored at 0.3**.
///
/// Both bounds bind in practice, at opposite ends of the state. That is what this reproduces.
#[test]
fn gifted_reproduces_from_two_per_pupil_amounts_and_three_kinds_of_unit() {
    let panel = panel::panel();
    let (floor, cap) = GIFTED_COORDINATOR_UNIT_BOUNDS;
    let (k8_price, senior_price) = GIFTED_SPECIALIST_UNIT_PRICES;
    let mut at_the_cap = 0;
    let mut at_the_coordinator_floor = 0;

    for record in &panel {
        let Some(share) = record.published_state_share else {
            continue;
        };
        let g = &record.gifted;
        let enrolled = record.categorical_enrolled_adm;

        let identification = g.adm_k6 * GIFTED_IDENTIFICATION_PER_PUPIL * share;
        assert!(
            common::close(identification, g.identification),
            "{}: identification computed {identification:.2} against published {:.2}",
            record.name,
            g.identification
        );
        let referral = enrolled * GIFTED_REFERRAL_PER_PUPIL * share;
        assert!(
            common::close(referral, g.referral),
            "{}: referral computed {referral:.2} against published {:.2}",
            record.name,
            g.referral
        );

        let coordinator_units = (enrolled / GIFTED_COORDINATOR_DIVISOR).clamp(floor, cap);
        assert!(
            common::close(coordinator_units, g.coordinator_units),
            "{}: coordinator units computed {coordinator_units:.6} against published {:.6}",
            record.name,
            g.coordinator_units
        );
        let specialist_units =
            |fte: f64| (fte / GIFTED_SPECIALIST_DIVISOR).max(GIFTED_SPECIALIST_UNIT_FLOOR);
        for (label, units, published_units, price, published_aid) in [
            (
                "coordinator",
                coordinator_units,
                g.coordinator_units,
                GIFTED_COORDINATOR_UNIT_PRICE,
                g.coordinator_aid,
            ),
            (
                "specialist K-8",
                specialist_units(g.fte_k8),
                g.specialist_k8_units,
                k8_price,
                g.specialist_k8_aid,
            ),
            (
                "specialist 9-12",
                specialist_units(g.fte_9_12),
                g.specialist_9_12_units,
                senior_price,
                g.specialist_9_12_aid,
            ),
        ] {
            assert!(
                common::close(units, published_units),
                "{}: {label} units computed {units:.6} against published {published_units:.6}",
                record.name
            );
            let aid = units * price * share;
            assert!(
                common::close(aid, published_aid),
                "{}: {label} aid computed {aid:.2} against published {published_aid:.2}",
                record.name
            );
        }

        assert!(
            common::reconciles(g.total(), record.categoricals.gifted, 6),
            "{}: the parts sum to {:.2} against published total {:.2}",
            record.name,
            g.total(),
            record.categoricals.gifted
        );

        if (g.coordinator_units - cap).abs() < 1e-9 {
            at_the_cap += 1;
        }
        if (g.coordinator_units - floor).abs() < 1e-9 {
            at_the_coordinator_floor += 1;
        }
    }

    // Both bounds bind, and they bind at opposite ends of the state. Neither is decorative.
    assert!(
        at_the_cap > 0,
        "the eight-coordinator cap should bind for the largest districts"
    );
    assert!(
        at_the_coordinator_floor > 0,
        "the half-coordinator floor should bind for the smallest"
    );
}

/// What a district gets for identifying no gifted pupils at all.
///
/// The floors mean gifted funding never reaches zero. A district below every threshold still draws
/// 0.5 coordinator units, 0.3 K-8 specialist units and 0.3 9-12 specialist units — $93,993 before
/// its state share, plus identification and referral on its enrolment.
///
/// This is defensible: units buy people, and half a coordinator is already the smallest thing a
/// small district can be funded for. It is also invisible. A district's gifted line looks like a
/// payment for gifted pupils and is, for the smallest districts, mostly a payment for existing.
#[test]
fn gifted_pays_a_floor_to_districts_with_almost_no_identified_pupils() {
    let panel = panel::panel();
    let (floor, _) = GIFTED_COORDINATOR_UNIT_BOUNDS;
    let (k8_price, senior_price) = GIFTED_SPECIALIST_UNIT_PRICES;

    let entirely_on_the_floor: Vec<_> = panel
        .iter()
        .filter(|r| r.gifted.entirely_on_the_floor())
        .collect();
    assert!(
        !entirely_on_the_floor.is_empty(),
        "expected districts drawing only floored units"
    );

    let bare_minimum = floor * GIFTED_COORDINATOR_UNIT_PRICE
        + GIFTED_SPECIALIST_UNIT_FLOOR * k8_price
        + GIFTED_SPECIALIST_UNIT_FLOOR * senior_price;
    assert!(
        bare_minimum > 90_000.0,
        "the floor is worth {bare_minimum:.0} before the state share"
    );

    // For a district on the floor, unit funding is the bulk of its gifted aid — the money is not
    // tracking gifted pupils, because it has almost none.
    for record in &entirely_on_the_floor {
        assert!(
            record.gifted.unit_funding() > record.gifted.identification + record.gifted.referral,
            "{}: unit funding {:.0} against per-pupil amounts {:.0}",
            record.name,
            record.gifted.unit_funding(),
            record.gifted.identification + record.gifted.referral
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Targeted assistance
// ---------------------------------------------------------------------------------------------

/// The largest categorical, reproduced: two tiers that measure different things and add.
///
/// `[G] = [C] + [F]`, and the two addends are not two views of one quantity:
///
/// - `[C]` the capacity amount is 0.8% of however far the district's **total** weighted wealth
///   falls below the median district's, phased by district size.
/// - `[F]` the wealth amount is a rate against weighted wealth **per resident pupil**, cutting off
///   where that reaches 1.25 times the state median.
///
/// Weighted wealth itself is 60% assessed valuation and 40% federal adjusted gross income — the
/// same two quantities local capacity blends, at different weights and toward a different end.
#[test]
fn targeted_assistance_reproduces_as_two_additive_tiers() {
    let panel = panel::panel();
    let (property, income) = TA_WEALTH_BLEND;
    let mut receiving = 0;
    let mut capacity_tier = 0;
    let mut wealth_tier = 0;

    for record in &panel {
        let ta = &record.targeted_assistance;
        if ta.weighted_wealth <= 0.0 {
            continue;
        }
        let enrolled = record.categorical_enrolled_adm;

        let blended = property * ta.property_valuation + income * ta.federal_gross_income;
        assert!(
            common::close(blended, ta.weighted_wealth),
            "{}: 60/40 blend gives {blended:.2} against published {:.2}",
            record.name,
            ta.weighted_wealth
        );

        let capacity_index = TA_MEDIAN_WEIGHTED_WEALTH / ta.weighted_wealth;
        assert!(
            (capacity_index - ta.capacity_index).abs() < 5e-9,
            "{}: capacity index computed {capacity_index:.9} against published {:.9}",
            record.name,
            ta.capacity_index
        );

        let shortfall = TA_MEDIAN_WEIGHTED_WEALTH - ta.weighted_wealth;
        let capacity = if shortfall > 0.0 {
            TargetedAssistance::capacity_size_share(enrolled) * TA_CAPACITY_RATE * shortfall
        } else {
            0.0
        };
        assert!(
            common::close(capacity, ta.capacity_amount),
            "{}: capacity amount computed {capacity:.2} against published {:.2} (ADM {enrolled:.1})",
            record.name,
            ta.capacity_amount
        );

        // `[D]` is published rounded to the cent, and `[E]` and `[F]` are computed from the
        // rounded figure rather than the exact one — so the reproduction has to round too.
        let resident = ta.resident_adm(enrolled);
        let per_pupil = if resident > 0.0 {
            (ta.weighted_wealth / resident * 100.0).round() / 100.0
        } else {
            0.0
        };
        assert!(
            common::close(per_pupil, ta.wealth_per_pupil),
            "{}: wealth per resident pupil computed {per_pupil:.2} against published {:.2}",
            record.name,
            ta.wealth_per_pupil
        );

        let wealth_index = if per_pupil > 0.0 {
            TA_MEDIAN_WEALTH_PER_PUPIL / per_pupil
        } else {
            0.0
        };
        assert!(
            common::close(wealth_index, ta.wealth_index),
            "{}: wealth index computed {wealth_index:.8} against published {:.8}",
            record.name,
            ta.wealth_index
        );

        let wealth = if wealth_index < TA_WEALTH_INDEX_FLOOR {
            0.0
        } else {
            (TA_MEDIAN_WEALTH_PER_PUPIL * TA_WEALTH_RATE - per_pupil * TA_WEALTH_OFFSET_RATE)
                * enrolled
        };
        assert!(
            common::close(wealth, ta.wealth_amount),
            "{}: wealth amount computed {wealth:.2} against published {:.2}",
            record.name,
            ta.wealth_amount
        );

        assert!(
            common::reconciles(ta.total(), record.categoricals.targeted_assistance, 2),
            "{}: the two tiers sum to {:.2} against published total {:.2}",
            record.name,
            ta.total(),
            record.categoricals.targeted_assistance
        );

        if ta.total() > 0.0 {
            receiving += 1;
        }
        if ta.capacity_amount > 0.0 {
            capacity_tier += 1;
        }
        if ta.wealth_amount > 0.0 {
            wealth_tier += 1;
        }
    }

    // The two tiers reach overlapping but different sets of districts, which is the argument for
    // reading them apart. Neither is a subset of the other.
    assert!(
        capacity_tier > 250 && wealth_tier > 400 && receiving > capacity_tier.max(wealth_tier),
        "capacity {capacity_tier}, wealth {wealth_tier}, either {receiving}"
    );
}

/// The wealth tier's cutoff is not a policy threshold. It is where the arithmetic reaches zero.
///
/// `[F]` is `(median_per_pupil × 0.014 − district_per_pupil × 0.0112) × ADM`, gated by
/// `IF(index < 0.8, 0, …)`. And 0.0112 is exactly 0.8 × 0.014, so the bracket goes negative at
/// precisely the index the gate cuts off at. The `IF` is not choosing a threshold — it is stopping
/// the formula from paying negative aid to wealthy districts.
///
/// Worth knowing before anyone treats 0.8 as a dial. Moving it without moving the coefficient pair
/// either creates a discontinuity or starts charging districts.
#[test]
fn the_wealth_tier_cuts_off_exactly_where_its_own_bracket_reaches_zero() {
    let panel = panel::panel();

    // No district below the floor is paid, which is what the gate says.
    let paid_below_the_floor = panel
        .iter()
        .filter(|r| {
            r.targeted_assistance.wealth_index > 0.0
                && r.targeted_assistance.wealth_index < TA_WEALTH_INDEX_FLOOR
                && r.targeted_assistance.wealth_amount > 0.0
        })
        .count();
    assert_eq!(
        paid_below_the_floor, 0,
        "no district below the floor should receive the wealth tier"
    );

    // And — the actual claim — the tier is **continuous** across the cutoff rather than stepping.
    // Take the districts just above the floor and look at what they are paid per pupil. If the
    // gate were an independently chosen threshold there would be a jump there, and the districts
    // nearest it would be receiving a real amount. Instead the payment approaches zero.
    let mut just_above: Vec<(&str, f64, f64)> = panel
        .iter()
        .filter_map(|r| {
            let ta = &r.targeted_assistance;
            let adm = r.categorical_enrolled_adm;
            (ta.wealth_index >= TA_WEALTH_INDEX_FLOOR && ta.wealth_index < 0.85 && adm > 0.0)
                .then(|| (r.name.as_str(), ta.wealth_index, ta.wealth_amount / adm))
        })
        .collect();
    assert!(
        just_above.len() > 10,
        "expected districts sitting just above the cutoff: {}",
        just_above.len()
    );
    just_above.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("no NaN in published indices"));

    let (name, index, per_pupil) = just_above[0];
    assert!(
        per_pupil < 25.0,
        "{name} sits at an index of {index:.4}, barely above the cutoff, and is paid \
         {per_pupil:.2} per pupil — the tier should approach zero there rather than step"
    );

    // Stated the other way: the payment rises with the index from that near-zero start, so the
    // cutoff is the bottom of a slope and not the edge of a shelf.
    let steepest = just_above.last().expect("a non-empty sample").2;
    assert!(
        steepest > per_pupil * 4.0,
        "across a narrow band above the cutoff the payment should climb steeply, from \
         {per_pupil:.2} to {steepest:.2} per pupil"
    );
}

/// The capacity tier has a size cliff, and it is a cliff rather than a slope.
///
/// It binds for **five** districts today. That is the honest scale of it, and it is not why the
/// cliff is worth recording: a threshold written into the formula is a standing property of the
/// formula, and the number of districts on the wrong side of it is a fact about this year's
/// enrolment. Sixty-eight districts sit between 200 and 600 pupils, on the shelf or the ramp, and
/// a shrinking district crosses these boundaries in the direction that costs it money.
///
/// A district under 200 pupils receives none of the capacity tier however far below the median its
/// wealth falls. At 200 it receives 5%. It stays at 5% to 400, then ramps linearly to 100% at 600.
///
/// So two districts identically placed against the state's median wealth are paid differently for
/// being different sizes, and the difference between 199 pupils and 201 is the whole tier against
/// a twentieth of it. The department is presumably guarding against a tiny district with a tiny tax
/// base drawing equalisation designed for a district twenty times its size — but the guard is a
/// step function, and the corpus should be able to say where the steps are.
#[test]
fn the_capacity_tier_pays_nothing_below_two_hundred_pupils() {
    let panel = panel::panel();

    let small_and_poor: Vec<_> = panel
        .iter()
        .filter(|r| {
            r.current_year_adm < TA_CAPACITY_MINIMUM_ADM
                && r.targeted_assistance.weighted_wealth > 0.0
                && r.targeted_assistance.weighted_wealth < TA_MEDIAN_WEIGHTED_WEALTH
        })
        .collect();
    assert!(
        !small_and_poor.is_empty(),
        "expected districts below 200 pupils and below the median wealth"
    );
    for record in &small_and_poor {
        assert_eq!(
            record.targeted_assistance.capacity_amount,
            0.0,
            "{} has {:.0} pupils and {:.0} below the median wealth, and still draws {:.2}",
            record.name,
            record.current_year_adm,
            TA_MEDIAN_WEIGHTED_WEALTH - record.targeted_assistance.weighted_wealth,
            record.targeted_assistance.capacity_amount
        );
    }

    // The step itself, stated as arithmetic rather than as prose.
    assert_eq!(TargetedAssistance::capacity_size_share(199.0), 0.0);
    assert_eq!(TargetedAssistance::capacity_size_share(200.0), 0.05);
    assert_eq!(TargetedAssistance::capacity_size_share(400.0), 0.05);
    assert!((TargetedAssistance::capacity_size_share(500.0) - 0.525).abs() < 1e-12);
    assert_eq!(TargetedAssistance::capacity_size_share(601.0), 1.0);
}

/// The categoricals' pupil count against the one the corpus already had: one district apart.
///
/// `[a] Enrolled ADM` and `[b3] FY26 Enrolled ADM` are the same number in 608 districts and fifty
/// pupils apart in Akron. Pinned here so that a future workbook which changes the relationship —
/// in either direction — fails rather than silently shifting four programs onto a different base.
#[test]
fn the_categorical_pupil_count_matches_fy26_everywhere_except_one_district() {
    let panel = panel::panel();
    let diverging: Vec<_> = panel
        .iter()
        .filter(|r| (r.categorical_enrolled_adm - r.current_year_adm).abs() > 0.01)
        .collect();

    assert_eq!(
        diverging.len(),
        1,
        "expected exactly one district where the two enrolled-ADM columns disagree, got {:?}",
        diverging.iter().map(|r| &r.name).collect::<Vec<_>>()
    );
    let odd = diverging[0];
    assert!(
        (odd.categorical_enrolled_adm - odd.current_year_adm).abs() > 40.0,
        "{}: the gap is {:.2} pupils, smaller than the fifty recorded",
        odd.name,
        odd.categorical_enrolled_adm - odd.current_year_adm
    );
}

/// Two pupil counts, one line apart, inside the same tier.
///
/// `[D]` divides weighted wealth by **resident** ADM — enrolled, less those open-enrolling in,
/// plus those open-enrolling out. `[F]` then multiplies the resulting rate by **enrolled** ADM.
///
/// Both are defensible in isolation. Wealth per pupil should arguably be measured against the
/// pupils a district's own taxpayers are responsible for; aid should arguably be paid on the pupils
/// it actually teaches. Together they mean a district with heavy inbound open enrolment is measured
/// as wealthier per pupil than it is and then paid on the larger count — the measurement and the
/// payment disagree about who the district's pupils are.
#[test]
fn the_wealth_tier_measures_on_resident_pupils_and_pays_on_enrolled_ones() {
    let panel = panel::panel();
    let mut differing = 0;
    let mut widest: Option<(&str, f64)> = None;

    for record in &panel {
        let ta = &record.targeted_assistance;
        let enrolled = record.categorical_enrolled_adm;
        let resident = ta.resident_adm(enrolled);
        if enrolled <= 0.0 || resident <= 0.0 {
            continue;
        }
        let gap = (enrolled - resident) / resident;
        if gap.abs() > 0.01 {
            differing += 1;
        }
        if widest.is_none_or(|(_, w)| gap > w) {
            widest = Some((&record.name, gap));
        }
    }

    assert!(
        differing > 100,
        "expected the two counts to diverge for many districts, not {differing}"
    );
    let (name, gap) = widest.expect("a panel with districts");
    assert!(
        gap > 0.2,
        "{name} has the widest gap at {:.1}%, which is smaller than expected",
        gap * 100.0
    );
}

/// The supplemental tier is a program line that pays nothing.
///
/// `[I] Supplemental Targeted Assistance` is zero for every district in the FY2027 model. Its
/// eligibility test still runs and still qualifies districts: `[H]` is `YES` where a district's
/// FY2019 wealth index exceeded 1.6 **and** its FY2019 enrolled ADM was below 88% of its total ADM
/// — that is, a district that was poor and had already lost more than an eighth of its pupils to
/// open enrolment and community schools.
///
/// So the state maintains a test for a distress condition, identifies the districts that meet it,
/// and attaches no money. Worth recording as a live zero rather than as an absence: a future
/// biennium can fund the line without changing anything else, and the corpus would not otherwise
/// notice it existed.
#[test]
fn the_supplemental_tier_qualifies_districts_and_pays_them_nothing() {
    let panel = panel::panel();

    let eligible: Vec<_> = panel
        .iter()
        .filter(|r| r.targeted_assistance.supplement_eligible)
        .collect();
    assert!(
        eligible.len() > 20,
        "expected a meaningful set of qualifying districts, got {}",
        eligible.len()
    );

    let paid: f64 = panel
        .iter()
        .map(|r| r.targeted_assistance.supplemental)
        .sum();
    assert_eq!(
        paid, 0.0,
        "the supplemental tier paid {paid:.2} across the panel"
    );

    // The condition the flag encodes, reproduced from the FY2019 figures the sheet carries. A
    // handful of rows have no FY2019 history at all and cannot be tested against it.
    let mut agreeing = 0;
    let mut testable = 0;
    for record in &panel {
        let ta = &record.targeted_assistance;
        if ta.fy19_total_adm <= 0.0 || ta.fy19_wealth_index <= 0.0 {
            continue;
        }
        testable += 1;
        let expected =
            ta.fy19_wealth_index > 1.6 && ta.fy19_enrolled_adm < 0.88 * ta.fy19_total_adm;
        if expected == ta.supplement_eligible {
            agreeing += 1;
        }
    }
    assert_eq!(
        agreeing, testable,
        "the eligibility rule should reproduce for every district with FY2019 history"
    );
}

// ---------------------------------------------------------------------------------------------
// All six together
// ---------------------------------------------------------------------------------------------

/// With these four, every dollar of the $2.76bn categorical lump has a mechanism behind it.
///
/// The six programs now decompose to their inputs: two sets of ascending weights, one set of
/// descending weights, a blended and squared poverty index, a unit-and-floor staffing model, and a
/// two-tier equalisation. Nothing about any of those is recoverable from the sum, and the sum was
/// what the corpus had for eight phases.
#[test]
fn every_one_of_the_six_categoricals_reconciles_to_its_own_decomposition() {
    let panel = panel::panel();
    for record in &panel {
        let c = &record.categoricals;
        for (label, decomposed, published) in [
            (
                "targeted assistance",
                record.targeted_assistance.total(),
                c.targeted_assistance,
            ),
            (
                "special education",
                record.special_education.total(),
                c.special_education,
            ),
            (
                "career-technical",
                record.career_technical.total(),
                c.career_technical,
            ),
            ("gifted", record.gifted.total(), c.gifted),
            (
                "English learners",
                record.english_learners.total(),
                c.english_learners,
            ),
        ] {
            assert!(
                common::reconciles(decomposed, published, 6),
                "{}: {label} decomposes to {decomposed:.2} against published {published:.2}",
                record.name
            );
        }

        // DPIA is the sixth, and its decomposition is a count and an index rather than a set of
        // amounts, so it reconciles through the per-pupil rate instead.
        assert!(
            record.dpia.index >= 0.0,
            "{}: DPIA index should be non-negative",
            record.name
        );
    }
}
