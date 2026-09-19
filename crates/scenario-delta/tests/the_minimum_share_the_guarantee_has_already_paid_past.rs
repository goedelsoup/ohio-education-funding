//! Ohio's 10% minimum state share, and the guarantee that has already paid past it.
//!
//! The Fair School Funding Plan pays `base cost x state share`, where the share is set by a
//! local-capacity measure and floored at a **minimum state share**. The plan was enacted with
//! 5%; the department's FY2027 model states **10%**, and 138 of 609 districts sit on it.
//!
//! A second hold-harmless sits on the same districts. The temporary transitional aid guarantee
//! holds every district at its FY2020 receipt, pays **$879.0M** across **294** districts, and
//! its top 50 recipients hold **51.7%** of it.
//!
//! The two instruments are stacked, and this file asks which one is load-bearing.
//!
//! # The population, and why the boolean is not enough
//!
//! **107 of the 138 districts at the minimum — 77.5% — are also on the guarantee**, and they
//! hold **$410.6M**, *46.7% of the entire guarantee*. Those 107 are not marginally held:
//!
//! | `realized / formula` | districts |
//! |---|--:|
//! | `< 1.05x` | 3 |
//! | `1.05 - 1.25x` | 7 |
//! | `1.25 - 2x` | 26 |
//! | `2 - 5x` | 65 |
//! | `> 5x` | 6 |
//!
//! The median is **2.40x**. The guarantee pays these districts more than twice what the formula
//! computes for them, so the last cent of that formula — which is what the minimum share sets —
//! is nowhere near the `max` that decides their cheque. The other 31 are on the formula and
//! their median multiple is exactly **1.00x**. The 138 are two populations, not one.
//!
//! # The finding: the minimum is dead letter for the 107
//!
//! Lowering the minimum **10% -> 5%** halves the base cost aid of all 138. Naively that is
//! **$166.9M**. Realized, it is **$35.1M — 21.0%** of it.
//!
//! | | 10% -> 5% | 10% -> 0% |
//! |---|--:|--:|
//! | naive cut, the 107 held | $118.8M | $237.5M |
//! | naive cut, the 31 on formula | $48.1M | $96.2M |
//! | **naive total** | **$166.9M** | **$333.7M** |
//! | **realized** | **$35.1M** | **$54.4M** |
//! | realized / naive | 21.0% | 16.3% |
//! | districts moved | 31 | 31 |
//!
//! Two absorptions, not one. The $118.8M aimed at the 107 vanishes entirely — they are
//! [`Standing::HeldThroughout`] and move by **exactly zero**. And of the $48.1M aimed at the
//! 31, a further quarter is absorbed as 12 of them fall through the formula onto their own
//! FY2020 floor ([`Standing::PushedOn`]): those 12 face a naive $18.9M and lose $5.9M, **68.8%
//! absorbed**. At a 0% minimum the second absorption dominates — 28 of the 31 land on the
//! guarantee and only **3** districts in Ohio take the full cut.
//!
//! So the reach of the minimum state share is **31 districts, not 138**, and raising it from
//! the enacted 5% to 10% bought $35.1M rather than the $166.9M its apparent reach implies.
//! **The guarantee is the operative instrument; the minimum share is decorative for 77.5% of
//! the districts it nominally reaches.**
//!
//! # And the containment is exact
//!
//! Cut on [`Axis::state_share`], the whole effect sits in the floor's mass point and every
//! other band is **$0.00**, not approximately zero. Cut on [`Axis::valuation`] the bottom three
//! quintiles are likewise exactly zero, so [`Incidence::gradient`] is `None` — the case its
//! docs name as the one a floor produces.
//!
//! # Which measure, and which denominator
//!
//! Everything here is [`Measure::FoundationAid`] — `core foundation funding + guarantee`, which
//! is what [`DistrictRecord::realized_aid`] returns and the only channel this lever touches.
//! Transportation has its own, separate and much higher, minimum share; see
//! `project/tests/the_floor_that_pays_the_wealthy_districts.rs`. Medians are the upper-middle
//! value, agreeing with [`dispersion::median`] here because every subgroup taken is odd.
//!
//! # What does not reproduce
//!
//! A prior session's quartile table — median base cost per pupil $8,231 / $8,231 / $8,163 /
//! $8,136 against median state share 58.9% / 38.0% / 21.2% / 10.0% — does not reproduce at
//! those figures under any binning tried: equal-count on valuation gives 59.6 / 42.7 / 23.8 /
//! 10.0, equal-count on state share 61.0 / 43.5 / 24.6 / 10.0, equal-*pupil* on valuation 59.2
//! / 39.5 / 20.1 / 10.0. Its shape reproduces on all three and is asserted below; its exact
//! values are not pinned, because the axis and weighting behind them were never stated. Its
//! "multiple held above formula" column reproduces under none of them: a median over every
//! district in a quartile is **1.000x** in the bottom two, since most districts there are not on
//! the guarantee at all.

use dispersion::wealth_neutrality;
use project::biennium::Measure;
use project::panel::{panel, DistrictRecord};
use project::policy::{Policy, Statewide};
use scenario_delta::{across, Axis, Incidence, ScenarioDelta, Standing};

/// The minimum state share the department's FY2027 model is published at.
const IN_FORCE: f64 = 0.10;

/// What the Fair School Funding Plan was enacted with, and the first counterfactual.
const AS_ENACTED: f64 = 0.05;

/// No floor at all, which bounds the lever.
const NONE: f64 = 0.0;

fn under(districts: &[DistrictRecord], minimum: f64) -> ScenarioDelta {
    ScenarioDelta::between(
        districts,
        &Policy::current_law(),
        &Policy {
            minimum_state_share: minimum,
            ..Policy::current_law()
        },
    )
}

/// What halving the floor would take if nothing absorbed it: the censored districts' base cost
/// aid *is* the floor, so a floor at `minimum` leaves `minimum / 0.10` of it.
///
/// Computed from base cost aid rather than from `minimum x base cost`: the two disagree by 1.5%
/// because the department divides state share by current-year enrolled ADM and base cost by the
/// three-year average, so "10% of base cost" is 9.85% of *this* denominator and a naive figure
/// built the other way would carry that error into every ratio below.
fn naive_cut(record: &DistrictRecord, minimum: f64) -> f64 {
    record.base_cost_state_share * (1.0 - minimum / IN_FORCE)
}

fn upper_middle(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a published figure"));
    values[values.len() / 2]
}

/// **The population.** 138 at the floor, and 107 of them paid by the guarantee instead.
#[test]
fn most_districts_at_the_minimum_are_paid_by_the_guarantee_rather_than_the_formula() {
    let districts = panel();
    assert_eq!(districts.len(), 609, "the FY2027 panel");

    let at_minimum: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|r| r.at_minimum_state_share())
        .collect();
    assert_eq!(
        at_minimum.len(),
        138,
        "districts at the minimum state share"
    );

    let held: Vec<&&DistrictRecord> = at_minimum.iter().filter(|r| r.on_guarantee()).collect();
    assert_eq!(held.len(), 107, "of those, also on the guarantee");
    let share = held.len() as f64 / at_minimum.len() as f64;
    assert!(
        (share - 0.775).abs() < 0.001,
        "the overlap is {share} of the floor population"
    );

    // Not a boolean: banded, because "on the guarantee" spans 1.003x to past 5x.
    let mut bands = [0usize; 5];
    for record in &held {
        let multiple = record.realized_aid() / record.core_foundation_funding;
        let band = match multiple {
            m if m < 1.05 => 0,
            m if m < 1.25 => 1,
            m if m < 2.0 => 2,
            m if m < 5.0 => 3,
            _ => 4,
        };
        bands[band] += 1;
    }
    assert_eq!(bands, [3, 7, 26, 65, 6], "the multiple, banded: {bands:?}");

    let median = upper_middle(
        held.iter()
            .map(|r| r.realized_aid() / r.core_foundation_funding)
            .collect(),
    );
    assert!(
        (median - 2.404_040).abs() < 1e-5,
        "the median multiple among the held is {median}"
    );

    // And the 31 the guarantee does not pay are at exactly the formula, which is what makes the
    // 138 two populations rather than a spectrum.
    let on_formula = upper_middle(
        at_minimum
            .iter()
            .filter(|r| !r.on_guarantee())
            .map(|r| r.realized_aid() / r.core_foundation_funding)
            .collect(),
    );
    assert_eq!(
        on_formula, 1.0,
        "the median multiple among the other 31 is {on_formula}"
    );
}

/// The 107 are not a marginal slice of the guarantee — they are nearly half of it.
#[test]
fn the_districts_at_the_floor_hold_almost_half_the_guarantee() {
    let districts = panel();

    let paid: Vec<&DistrictRecord> = districts.iter().filter(|r| r.on_guarantee()).collect();
    assert_eq!(paid.len(), 294, "districts the guarantee pays");
    let total: f64 = paid.iter().map(|r| r.guarantee).sum();
    assert!(
        (total - 878_954_627.38).abs() < 1.0,
        "the guarantee totals {total}"
    );

    let mut amounts: Vec<f64> = paid.iter().map(|r| r.guarantee).collect();
    amounts.sort_by(|a, b| b.partial_cmp(a).expect("no NaN in a published figure"));
    let top_fifty: f64 = amounts.iter().take(50).sum::<f64>() / total;
    assert!(
        (top_fifty - 0.5168).abs() < 0.0005,
        "the top 50 hold {top_fifty} of it"
    );

    let at_floor: f64 = districts
        .iter()
        .filter(|r| r.at_minimum_state_share() && r.on_guarantee())
        .map(|r| r.guarantee)
        .sum();
    assert!(
        (at_floor - 410_617_498.51).abs() < 1.0,
        "the 107 hold {at_floor}"
    );
    let concentration = at_floor / total;
    assert!(
        (concentration - 0.4672).abs() < 0.0005,
        "which is {concentration} of the guarantee"
    );
}

/// **The prediction, and it holds.** The lever reaches 31 districts, not 138.
///
/// At both counterfactual minimums the same 31 move and the same 107 are
/// [`Standing::HeldThroughout`]. Not approximately: a held district's aid comes out of the same
/// `max` against the same fixed baseline in both runs, so it moves by exactly zero.
#[test]
fn lowering_the_minimum_reaches_thirty_one_districts_and_not_the_other_hundred_and_seven() {
    let districts = panel();

    for minimum in [AS_ENACTED, NONE] {
        let delta = under(&districts, minimum);
        let total = delta.total();
        assert_eq!(total.reach.districts, 609);
        assert_eq!(
            total.reach.gainers, 0,
            "nobody gains from a lower floor, but {} do at {minimum}",
            total.reach.gainers
        );
        assert_eq!(
            total.reach.losers, 31,
            "districts moved at a {minimum} minimum: {}",
            total.reach.losers
        );
        assert_eq!(
            total.reach.unmoved, 578,
            "districts untouched at {minimum}: {}",
            total.reach.unmoved
        );

        // Within the floor population: 107 held, and every one of them by exactly zero.
        let mut held = 0;
        for (record, row) in districts.iter().zip(&delta.deltas) {
            assert_eq!(
                record.irn, row.irn,
                "the panel and the table are in lockstep"
            );
            if !record.at_minimum_state_share() {
                assert_eq!(
                    row.dollars(),
                    0.0,
                    "{} is not at the floor and moved by {} at {minimum}",
                    row.irn,
                    row.dollars()
                );
                continue;
            }
            if row.standing == Standing::HeldThroughout {
                held += 1;
                assert_eq!(
                    row.dollars(),
                    0.0,
                    "{} is held and moved by {} at {minimum}",
                    row.irn,
                    row.dollars()
                );
            }
        }
        assert_eq!(held, 107, "held throughout at {minimum}: {held}");
    }
}

/// **And the savings are a fifth of the naive figure, because the guarantee rises to meet the
/// cut — twice.**
#[test]
fn the_guarantee_absorbs_four_fifths_of_what_a_lower_floor_would_take() {
    let districts = panel();

    // (minimum, naive on the held, naive on the movers, realized, pushed-on naive, pushed-on
    //  realized, how many take the full cut, how many are caught partway)
    let expected = [
        (
            AS_ENACTED,
            118_768_813.42,
            48_081_997.20,
            -35_099_477.35,
            18_876_031.23,
            5_893_511.38,
            19,
            12,
        ),
        (
            NONE,
            237_537_626.85,
            96_163_994.40,
            -54_419_493.03,
            75_987_024.72,
            34_242_523.35,
            3,
            28,
        ),
    ];

    for (minimum, naive_held, naive_movers, realized, pushed_naive, pushed_real, full, partway) in
        expected
    {
        let delta = under(&districts, minimum);

        let (mut held, mut movers) = (0.0, 0.0);
        let (mut pushed_n, mut pushed_r) = (0.0, 0.0);
        let (mut took_it_all, mut caught) = (0, 0);
        for (record, row) in districts.iter().zip(&delta.deltas) {
            if !record.at_minimum_state_share() {
                continue;
            }
            let naive = naive_cut(record, minimum);
            if row.standing == Standing::HeldThroughout {
                held += naive;
                continue;
            }
            movers += naive;
            if row.standing == Standing::PushedOn {
                pushed_n += naive;
                pushed_r += -row.dollars();
                caught += 1;
            } else {
                // On the formula in both runs: the cut lands in full.
                assert!(
                    (-row.dollars() - naive).abs() < 1.0,
                    "{} takes {} against a naive {naive} at {minimum}",
                    row.irn,
                    -row.dollars()
                );
                took_it_all += 1;
            }
        }

        assert!(
            (held - naive_held).abs() < 1.0,
            "naive cut on the held at {minimum} is {held}"
        );
        assert!(
            (movers - naive_movers).abs() < 1.0,
            "naive cut on the movers at {minimum} is {movers}"
        );
        assert_eq!(
            took_it_all, full,
            "districts taking the full cut at {minimum}: {took_it_all}"
        );
        assert_eq!(
            caught, partway,
            "districts caught partway at {minimum}: {caught}"
        );

        let total = delta.total().dollars;
        assert!(
            (total - realized).abs() < 1.0,
            "realized at {minimum} is {total}"
        );

        // The first absorption: everything aimed at the held goes nowhere.
        assert!(
            (total.abs() - (movers - (pushed_n - pushed_r))).abs() < 1.0,
            "the realized {total} is not the movers' naive less what the floor caught"
        );

        // The second: the pushed-on lose a fraction of what was aimed at them.
        assert!(
            (pushed_n - pushed_naive).abs() < 1.0,
            "pushed-on naive at {minimum} is {pushed_n}"
        );
        assert!(
            (pushed_r - pushed_real).abs() < 1.0,
            "pushed-on realized at {minimum} is {pushed_r}"
        );
        let absorbed = 1.0 - pushed_r / pushed_n;
        assert!(
            absorbed > 0.5,
            "the floor absorbs only {absorbed} of the pushed-on cut at {minimum}"
        );

        // And the headline: a fifth of the naive figure at the enacted minimum, less at zero.
        let ratio = total.abs() / (held + movers);
        let want = if minimum == AS_ENACTED {
            0.2104
        } else {
            0.1631
        };
        assert!(
            (ratio - want).abs() < 0.0005,
            "realized is {ratio} of naive at {minimum}"
        );
    }
}

/// Lowering the floor further makes the guarantee *more* of the answer, not less.
#[test]
fn removing_the_floor_entirely_leaves_three_districts_taking_the_full_cut() {
    let districts = panel();

    let enacted = under(&districts, AS_ENACTED);
    let none = under(&districts, NONE);

    assert_eq!(enacted.total().reach.pushed_on, 12);
    assert_eq!(
        none.total().reach.pushed_on,
        28,
        "at no floor, {} are pushed onto the guarantee",
        none.total().reach.pushed_on
    );

    // The naive cut doubles; the realized one does not.
    let naive_ratio: f64 = 333_701_621.25 / 166_850_810.62;
    let realized_ratio = none.total().dollars / enacted.total().dollars;
    assert!(
        (naive_ratio - 2.0).abs() < 1e-6,
        "the naive cut scales by {naive_ratio}"
    );
    assert!(
        realized_ratio < 1.6,
        "the realized cut scales by {realized_ratio}, which is not far short of the naive 2.0"
    );
}

/// The effect is contained in the floor's mass point, exactly.
#[test]
fn every_band_that_is_not_the_floor_moves_by_exactly_zero() {
    let districts = panel();
    let delta = under(&districts, AS_ENACTED);

    let by_share: Incidence = across(&delta.deltas, &Axis::state_share(), |d| {
        Some(d.state_share_fraction)
    });
    let first = by_share.strata.first().expect("the axis has bands");
    assert_eq!(
        first.districts, 138,
        "the mass point holds {}",
        first.districts
    );
    assert_eq!(first.moved, 31);
    assert_eq!(first.held_throughout, 107);
    assert!(
        (first.dollars - delta.total().dollars).abs() < 1e-6,
        "the first band holds {} of a statewide {}",
        first.dollars,
        delta.total().dollars
    );
    for band in by_share.strata.iter().skip(1) {
        assert_eq!(
            band.dollars, 0.0,
            "{} [{:.4}..{:.4}] moved by {}",
            band.label, band.lower, band.upper, band.dollars
        );
    }

    // On valuation the bottom three quintiles are likewise exactly nothing, which is the case
    // `Incidence::gradient` documents as the one a floor produces.
    let by_wealth = across(&delta.deltas, &Axis::valuation(), |d| d.valuation_per_pupil);
    for band in by_wealth.strata.iter().take(3) {
        assert_eq!(
            band.dollars, 0.0,
            "{} moved by {}",
            band.label, band.dollars
        );
        assert_eq!(band.moved, 0);
    }
    assert_eq!(
        by_wealth.gradient(),
        None,
        "a bottom band of zero has no ratio to a top one"
    );
    assert_eq!(by_wealth.unclassified.districts, 3, "no assessed valuation");
    assert!(
        by_wealth.unclassified.dollars < 0.0,
        "and they are carried, not dropped: {}",
        by_wealth.unclassified.dollars
    );
}

/// The lever moves no statewide statistic, so one resolution serves both runs.
///
/// Asserted rather than assumed: [`Statewide::under`] moves for the DPIA blend and the
/// supplemental rate, and a lever that quietly moved it would make every figure above a
/// comparison between two different denominators.
#[test]
fn the_minimum_state_share_does_not_move_a_statewide_statistic() {
    let districts = panel();
    let published = Statewide::under(&districts, &Policy::current_law());
    for minimum in [AS_ENACTED, NONE] {
        let moved = Statewide::under(
            &districts,
            &Policy {
                minimum_state_share: minimum,
                ..Policy::current_law()
            },
        );
        assert_eq!(
            moved, published,
            "a {minimum} minimum resolves {moved:?} against a published {published:?}"
        );
    }
}

/// **Why the guarantee is the instrument worth reforming: it runs the wrong way on wealth.**
///
/// The formula is equalizing — aid falls as valuation rises, at `r = -0.662`. What districts are
/// actually paid is *less* equalizing, at `r = -0.603`, because the guarantee tops up exactly
/// the districts the formula has finished with. The minimum share cannot account for the gap:
/// it is $35.1M of a $879.0M instrument and does not reach 107 of the 138 districts it names.
#[test]
fn the_guarantee_makes_the_system_less_equalizing_than_the_formula_it_overrides() {
    let districts = panel();
    let rows: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|r| r.valuation_per_pupil.is_some() && r.current_year_adm > 0.0)
        .collect();
    assert_eq!(rows.len(), 606, "districts with an assessed valuation");

    let wealth: Vec<f64> = rows
        .iter()
        .map(|r| r.valuation_per_pupil.expect("filtered"))
        .collect();
    let realized: Vec<f64> = rows
        .iter()
        .map(|r| r.realized_aid() / r.current_year_adm)
        .collect();
    let formula: Vec<f64> = rows
        .iter()
        .map(|r| r.core_foundation_funding / r.current_year_adm)
        .collect();

    let paid = wealth_neutrality(&wealth, &realized).expect("606 pairs with variation");
    let computed = wealth_neutrality(&wealth, &formula).expect("606 pairs with variation");
    assert!(
        (paid.correlation - -0.602_977).abs() < 1e-5,
        "realized aid against valuation is {}",
        paid.correlation
    );
    assert!(
        (computed.correlation - -0.661_718).abs() < 1e-5,
        "formula aid against valuation is {}",
        computed.correlation
    );
    assert!(
        paid.correlation > computed.correlation,
        "the guarantee should weaken the relationship, not strengthen it: {} against {}",
        paid.correlation,
        computed.correlation
    );

    // The measure both are on, named rather than assumed.
    assert_eq!(Measure::FoundationAid.label(), "foundation aid");
}

/// The shape a prior quartile table claimed, which reproduces even though its figures do not:
/// base cost per pupil is flat across wealth and state share collapses to the floor.
#[test]
fn base_cost_is_flat_across_wealth_while_state_share_collapses_onto_the_floor() {
    let districts = panel();
    let mut rows: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|r| r.valuation_per_pupil.is_some())
        .collect();
    rows.sort_by(|a, b| {
        a.valuation_per_pupil
            .expect("filtered")
            .partial_cmp(&b.valuation_per_pupil.expect("filtered"))
            .expect("no NaN in a published figure")
    });

    let quarter = rows.len() / 4;
    let mut base_cost = Vec::new();
    let mut state_share = Vec::new();
    for q in 0..4 {
        let upper = if q == 3 {
            rows.len()
        } else {
            (q + 1) * quarter
        };
        let slice = &rows[q * quarter..upper];
        base_cost.push(upper_middle(
            slice.iter().map(|r| r.base_cost_per_pupil).collect(),
        ));
        state_share.push(upper_middle(
            slice.iter().map(|r| r.state_share_fraction()).collect(),
        ));
    }

    // Flat: the poorest quartile's base cost per pupil is within 2% of the wealthiest's.
    let spread = base_cost[0] / base_cost[3] - 1.0;
    assert!(
        spread < 0.02,
        "base cost per pupil spans {spread} across the wealth distribution: {base_cost:?}"
    );

    // Collapsing, monotonically, and landing on the floor.
    for pair in state_share.windows(2) {
        assert!(
            pair[1] < pair[0],
            "state share does not fall: {state_share:?}"
        );
    }
    assert!(
        (state_share[3] - IN_FORCE).abs() < 0.0005,
        "the wealthiest quartile's median state share is {}",
        state_share[3]
    );
    assert!(
        state_share[0] > 0.55,
        "the poorest quartile's median state share is {}",
        state_share[0]
    );
}
