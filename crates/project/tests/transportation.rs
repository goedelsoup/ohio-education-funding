//! Transportation, reproduced — the largest thing outside foundation funding.
//!
//! $726m, with $183m of special education transportation beside it. Transportation alone is
//! larger than special education, which makes it the **second-largest single program in Ohio's
//! school funding** after targeted assistance — and until now it sat entirely inside the remainder
//! this corpus carried between `[H] Foundation Funding` and `[R] Total State Support`.
//!
//! It is the most elaborate component in the calculator and it shares almost nothing with the
//! formula. Four things it does that nothing else here does:
//!
//! 1. **Two competing bases**, per weighted rider and per bus mile, and the district is paid the
//!    greater. The choice flips for more than half the state.
//! 2. **A 50% state minimum share**, against the formula's 10%, on which most districts sit — so
//!    for most of Ohio, local capacity does not determine transportation aid at all.
//! 3. **Two supplements pulling opposite ways**, one rewarding full buses and one compensating for
//!    empty geography.
//! 4. **Its own guarantee**, separate from the one on foundation funding.
//!
//! And one parameter unlike anything else the corpus holds: special education transportation is
//! multiplied by a **proration factor of 0.91746**, which means the appropriation did not cover
//! the computed entitlement and every district was scaled down to fit. A published amount under a
//! proration is not what the formula says a district is owed. It is what was available, divided.

use project::panel::{
    self, Transportation, TRANSPORT_COMMUNITY_WEIGHT, TRANSPORT_DENSITY_PIVOT,
    TRANSPORT_DENSITY_RATE, TRANSPORT_EFFICIENCY_BAND, TRANSPORT_EFFICIENCY_CEILING,
    TRANSPORT_MASS_TRANSIT_RATE, TRANSPORT_MINIMUM_STATE_SHARE, TRANSPORT_NONPUBLIC_WEIGHT,
    TRANSPORT_OTHER_RATE, TRANSPORT_PER_RIDER, TRANSPORT_SPED_PRORATION,
};

fn close(computed: f64, published: f64) -> bool {
    (computed - published).abs() <= 0.02_f64.max(published.abs() * 1e-9)
}

/// Weighted ridership is not ridership: a non-public rider counts twice.
///
/// `public + 2 x non-public + 1.5 x community or STEM`. A district transporting a private-school
/// child is funded at twice the rate of its own pupil, and one transporting a community-school
/// child at one and a half times.
///
/// Whatever the intent — the state requires districts to transport non-public pupils and the
/// routes are less efficient — the effect is that the same bus seat is worth a different amount
/// depending on which school the child is going to, and no published figure says so.
#[test]
fn weighted_ridership_counts_a_non_public_rider_twice() {
    let panel = panel::panel();
    let mut with_nonpublic = 0;

    for record in &panel {
        let t = &record.transportation;
        let expected = ((t.public_riders
            + t.nonpublic_riders * TRANSPORT_NONPUBLIC_WEIGHT
            + t.community_riders * TRANSPORT_COMMUNITY_WEIGHT)
            * 100.0)
            .round()
            / 100.0;
        assert!(
            close(expected, t.weighted_riders),
            "{}: 1/2/1.5 weighting gives {expected:.2} against published {:.2}",
            record.name,
            t.weighted_riders
        );
        if t.nonpublic_riders > 0.0 {
            with_nonpublic += 1;
        }
    }

    assert!(
        with_nonpublic > 300,
        "{with_nonpublic} districts transport non-public riders"
    );

    // The double weighting is material in aggregate, not a rounding detail.
    let raw: f64 = panel.iter().map(|r| r.transportation.riders()).sum();
    let nonpublic: f64 = panel
        .iter()
        .map(|r| r.transportation.nonpublic_riders)
        .sum();
    let weighted: f64 = panel.iter().map(|r| r.transportation.weighted_riders).sum();
    let raw_share = nonpublic / raw;
    let weighted_share = nonpublic * TRANSPORT_NONPUBLIC_WEIGHT / weighted;
    assert!(
        weighted_share > raw_share * 1.5,
        "non-public riders are {raw_share:.3} of riders and {weighted_share:.3} of weighted \
         ridership; the weighting should roughly double their share"
    );
}

/// The greater of two bases, times the greater of the state share and 50%.
///
/// Both `MAX`es matter and both are invisible in the published amount. The base choice flips for
/// more than half the state; the share floor binds for most of it.
#[test]
fn the_school_bus_payment_takes_the_greater_of_two_bases_and_a_floored_state_share() {
    let panel = panel::panel();
    let mut on_miles = 0;
    let mut on_the_floor = 0;

    for record in &panel {
        let t = &record.transportation;
        let Some(share) = record.published_state_share else {
            continue;
        };
        let base = t.per_rider_base().max(t.per_mile_base());
        let effective = share.max(TRANSPORT_MINIMUM_STATE_SHARE);
        // The sheet rounds this one *down* to a tenth of a dollar, which no other line does — and
        // truncation is unforgiving of the last bit. The state share is stored to ten places, so
        // `base * effective` lands either side of a tenth-of-a-dollar boundary on floating point
        // alone, and flooring turns a difference of 1e-9 into one of ten cents. The epsilon is not
        // slack in the check; it is the width of the representation error being truncated.
        let expected = (base * effective * 10.0 + 1e-6).floor() / 10.0;
        assert!(
            close(expected, t.school_bus),
            "{}: computed {expected:.2} against published {:.2}",
            record.name,
            t.school_bus
        );

        if t.paid_on_miles() {
            on_miles += 1;
        }
        if share < TRANSPORT_MINIMUM_STATE_SHARE {
            on_the_floor += 1;
        }
    }

    assert!(
        on_miles > panel.len() / 3,
        "{on_miles} of {} districts are paid on the mile base — the MAX is not a formality",
        panel.len()
    );
    assert!(
        on_the_floor > panel.len() / 2,
        "{on_the_floor} of {} districts are below the 50% floor",
        panel.len()
    );
}

/// Transportation's floor is five times the formula's, and it binds for three times as many.
///
/// The formula's minimum state share is 10% and 138 of 609 districts sit on it. Transportation's
/// is **50%**, and most of the state sits on it. So Ohio equalises getting to school far more
/// aggressively than it equalises what happens there — which is a policy choice of some size,
/// made in a sheet, and recoverable from no published total.
#[test]
fn transportation_is_equalised_five_times_harder_than_the_formula() {
    // Comparing the two constants would prove only that they were transcribed consistently. What
    // is worth asserting is how many districts each floor actually reaches.
    let panel = panel::panel();
    let on_formula_floor = panel.iter().filter(|r| r.at_minimum_state_share()).count();
    let on_transport_floor = panel
        .iter()
        .filter(|r| {
            r.published_state_share
                .is_some_and(|s| s < TRANSPORT_MINIMUM_STATE_SHARE)
        })
        .count();
    assert!(
        on_transport_floor > on_formula_floor * 2,
        "{on_transport_floor} districts are held up by the transportation floor against \
         {on_formula_floor} by the formula's"
    );
}

/// Mass transit and other vehicles, at 35% and 50% of the rider rate.
#[test]
fn the_other_two_vehicle_types_are_paid_at_fractions_of_the_rider_rate() {
    let panel = panel::panel();
    for record in &panel {
        let t = &record.transportation;
        for (label, riders, rate, published) in [
            (
                "mass transit",
                t.mass_transit_riders,
                TRANSPORT_MASS_TRANSIT_RATE,
                t.mass_transit,
            ),
            ("other", t.other_riders, TRANSPORT_OTHER_RATE, t.other),
        ] {
            let expected = riders * TRANSPORT_PER_RIDER * rate;
            assert!(
                close(expected, published),
                "{}: {label} computed {expected:.2} against published {published:.2}",
                record.name
            );
        }
    }
}

/// Two supplements that answer opposite questions, and many districts draw both.
///
/// The **efficiency** supplement pays up to 15% more for filling buses — riders per bus against a
/// capacity target, ramping from an index of 1.0 to 1.5 and flat above. The **density** supplement
/// pays for the absence of the thing efficiency rewards: `(28 - riders per square mile) / 100`
/// times the mile base times 0.55.
///
/// A district can be rewarded for packing its buses and compensated for having few children per
/// square mile at the same time, which is coherent — those are different problems — and is the
/// kind of interaction a single transportation total cannot show.
#[test]
fn the_efficiency_and_density_supplements_reward_opposite_things() {
    let panel = panel::panel();
    let (low, high) = TRANSPORT_EFFICIENCY_BAND;
    let mut efficiency = 0;
    let mut density = 0;
    let mut both = 0;

    for record in &panel {
        let t = &record.transportation;
        let index = t.efficiency_index;
        let expected_efficiency = if index >= high {
            t.school_bus * TRANSPORT_EFFICIENCY_CEILING
        } else if index >= low {
            (index - low) * TRANSPORT_EFFICIENCY_CEILING / (high - low) * t.school_bus
        } else {
            0.0
        };
        assert!(
            close((expected_efficiency * 100.0).round() / 100.0, t.efficiency),
            "{}: efficiency at index {index:.4} computed {expected_efficiency:.2} against \
             published {:.2}",
            record.name,
            t.efficiency
        );

        let supplement_pct = ((TRANSPORT_DENSITY_PIVOT - t.district_density) / 100.0).max(0.0);
        let expected_density = supplement_pct * t.per_mile_base() * TRANSPORT_DENSITY_RATE;
        assert!(
            close(expected_density, t.density),
            "{}: density at {:.4} riders per square mile computed {expected_density:.2} against \
             published {:.2}",
            record.name,
            t.district_density,
            t.density
        );

        if t.efficiency > 0.0 {
            efficiency += 1;
        }
        if t.density > 0.0 {
            density += 1;
        }
        if t.efficiency > 0.0 && t.density > 0.0 {
            both += 1;
        }
    }

    // And the published indices are what the counts say they are — checked separately, so a
    // mismatch shows up as a bad input rather than as a bad supplement.
    for record in &panel {
        let t = &record.transportation;
        assert!(
            (t.efficiency_index - t.efficiency_index_from_inputs()).abs() < 1e-3,
            "{}: published efficiency index {:.4} against {:.4} from its counts",
            record.name,
            t.efficiency_index,
            t.efficiency_index_from_inputs()
        );
        assert!(
            (t.district_density - t.density_from_inputs()).abs() < 1e-3,
            "{}: published density {:.4} against {:.4} from its counts",
            record.name,
            t.district_density,
            t.density_from_inputs()
        );
    }

    assert!(efficiency > 300 && density > 300);
    assert!(
        both > 200,
        "only {both} districts draw both, which is fewer than the two supplements' overlap suggests"
    );
}

/// A second transitional guarantee, which the corpus has no node for.
///
/// `[F]` holds a district at its FY2021 transportation funding where the computed amount has
/// fallen below it. 38 districts, $24.8m. That is a separate mechanism from the guarantee on
/// foundation funding — different base year, different base, different districts — and both are
/// called a temporary transitional aid guarantee.
#[test]
fn transportation_has_its_own_guarantee_on_a_fy2021_base() {
    let panel = panel::panel();
    let mut held = 0;

    for record in &panel {
        let t = &record.transportation;
        let expected = (t.fy21_base - t.before_proration()).max(0.0);
        assert!(
            close(expected, t.guarantee),
            "{}: guarantee computed {expected:.2} against published {:.2}",
            record.name,
            t.guarantee
        );
        if t.guarantee > 0.0 {
            held += 1;
        }
    }

    assert!(
        (10..100).contains(&held),
        "expected a few dozen districts on the transportation guarantee, got {held}"
    );

    // And it reaches different districts from the formula's guarantee, which is the argument for
    // it being a separate mechanism rather than the same one applied twice.
    let formula_only = panel
        .iter()
        .filter(|r| r.on_guarantee() && r.transportation.guarantee <= 0.0)
        .count();
    let transport_only = panel
        .iter()
        .filter(|r| !r.on_guarantee() && r.transportation.guarantee > 0.0)
        .count();
    assert!(
        formula_only > 100 && transport_only > 0,
        "{formula_only} on the formula guarantee alone, {transport_only} on the transportation \
         guarantee alone"
    );
}

/// The whole thing reconciles, and the total is prorated.
#[test]
fn the_five_payments_plus_the_guarantee_are_the_published_total() {
    let panel = panel::panel();
    for record in &panel {
        let t = &record.transportation;
        let expected = t.before_proration() + t.guarantee;
        assert!(
            close(expected, t.total),
            "{}: the parts sum to {expected:.2} against published {:.2}",
            record.name,
            t.total
        );
    }
    let total: f64 = panel.iter().map(|r| r.transportation.total).sum();
    assert!(
        total > 700e6,
        "transportation totals {total:.0}, which is smaller than expected"
    );
}

/// The proration factor, and why it is a different kind of number.
///
/// Special education transportation is `max(state share, 50%) x reported cost x 0.91746`. A factor
/// below one is not a rate or a weight — it is the appropriation divided by the entitlement, so
/// the published amount is not what the formula says a district is owed. It is what there was.
///
/// The corpus holds rates, weights, prices, thresholds and blends. This is the first parameter it
/// has that encodes a **shortfall**, and it cannot be recovered from the amount: a district shown
/// $1.2m has no way to know it was computed $1.31m.
#[test]
fn special_education_transportation_is_prorated_because_the_appropriation_fell_short() {
    let panel = panel::panel();
    let mut shortfall = 0.0;

    for record in &panel {
        let t = &record.transportation;
        let Some(share) = record.published_state_share else {
            continue;
        };
        let expected = share.max(TRANSPORT_MINIMUM_STATE_SHARE)
            * t.reported_sped_cost
            * TRANSPORT_SPED_PRORATION;
        assert!(
            close(expected, t.special_education),
            "{}: computed {expected:.2} against published {:.2}",
            record.name,
            t.special_education
        );
        shortfall += t.special_education_unprorated() - t.special_education;
    }

    // Recover the factor from the published amounts rather than asserting on the constant: for
    // every district, what was paid over what its reported cost and state share entitle it to. If
    // the appropriation had covered the entitlement this would be one everywhere.
    let mut recovered: Vec<f64> = panel
        .iter()
        .filter_map(|r| {
            let t = &r.transportation;
            let share = r.published_state_share?;
            let entitlement = share.max(TRANSPORT_MINIMUM_STATE_SHARE) * t.reported_sped_cost;
            (entitlement > 1000.0).then(|| t.special_education / entitlement)
        })
        .collect();
    assert!(recovered.len() > 400, "sample of {}", recovered.len());
    recovered.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let median = recovered[recovered.len() / 2];
    assert!(
        median < 0.95,
        "the factor recovered from published amounts is {median:.5}; a program whose \
         appropriation covered its entitlement would show one"
    );
    assert!(
        (median - TRANSPORT_SPED_PRORATION).abs() < 1e-6,
        "recovered {median:.9} against the transcribed {TRANSPORT_SPED_PRORATION:.9}"
    );
    assert!(
        shortfall > 10e6,
        "the proration withheld {shortfall:.0} across the panel, which is smaller than expected"
    );
}

/// With transportation read, most of what sits outside foundation funding is accounted for.
#[test]
fn transportation_is_the_bulk_of_what_sits_outside_the_formula() {
    let panel = panel::panel();
    let outside: f64 = panel
        .iter()
        .map(|r| r.total_state_support - r.realized_aid())
        .sum();
    let transportation: f64 = panel
        .iter()
        .map(|r| r.transportation.total + r.transportation.special_education)
        .sum();

    let share = transportation / outside;
    assert!(
        share > 0.6 && share < 1.0,
        "transportation is {share:.3} of what sits outside foundation funding"
    );

    // The scale claim, stated correctly. Transportation *alone* exceeds special education, the
    // second-largest categorical — so it is the second-largest single program in Ohio's school
    // funding, behind only targeted assistance. And with special education transportation beside
    // it, the pair exceeds the four smallest categoricals combined.
    //
    // The first version of this test claimed the pair beat five of the six combined. It does not:
    // $908.7m against $1,392m. The claim was written from the impression the reproduction gave
    // rather than from the arithmetic, and the assertion is what caught it.
    let bus: f64 = panel.iter().map(|r| r.transportation.total).sum();
    let sped: f64 = panel.iter().map(|r| r.categoricals.special_education).sum();
    assert!(
        bus > sped,
        "transportation {bus:.0} against special education {sped:.0}"
    );
    let four_smallest: f64 = panel
        .iter()
        .map(|r| {
            let c = &r.categoricals;
            c.dpia + c.english_learners + c.gifted + c.career_technical
        })
        .sum();
    assert!(
        transportation > four_smallest,
        "transportation and its special education line {transportation:.0} against the four \
         smallest categoricals {four_smallest:.0}"
    );
}

/// Guards a fixture regenerated with the columns shifted.
#[test]
fn the_transportation_struct_is_populated_rather_than_defaulted() {
    let panel = panel::panel();
    assert!(panel
        .iter()
        .any(|r| r.transportation != Transportation::default()));
    assert!(
        panel
            .iter()
            .filter(|r| r.transportation.total <= 0.0)
            .count()
            < 20,
        "almost every district runs buses"
    );
}
