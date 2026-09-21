//! The half of the guarantee's size curve R.C. 3317.011 does not explain, and the term that does.
//!
//! # The question, and the premise in it that was too narrow
//!
//! `what_the_staffing_minimums_are_worth_and_where_they_stop.rs` settled half of a shape and named
//! the residual rather than leaving it implied. The shape is the statewide guarantee rate by ADM
//! sextile, non-monotone, lowest among the smallest districts and highest in the third. The
//! staffing floors account for half of it and issue #409 asked what the rest is.
//!
//! It also ruled out the obvious answer, correctly: base cost's only size-dependent terms are the
//! floors and the two banded salaries, and the bands ramp continuously between 500 and 4,000 ADM,
//! so nothing in base cost can produce a peak at 1,192. The conclusion drawn from that — "whatever
//! produces the residual is not size" — does not follow, because **base cost is not the only place
//! the plan measures size**.
//!
//! # Targeted assistance's capacity tier
//!
//! R.C. 3317.0217 credits a district with eight mills of however far its **total** weighted wealth
//! falls below the **statewide median district's total** weighted wealth. Both sides are
//! whole-district dollars and neither is per pupil, so a district with few pupils has a small tax
//! base, scores far below the median, and is paid for it however wealthy each of its pupils is.
//!
//! Across all 609 districts, `ln(total weighted wealth)` on `ln(base cost enrolled ADM)` has a
//! slope of **0.9855** and an r² of **0.8126**. The same districts' weighted wealth **per resident
//! pupil** on the same predictor has a slope of **-0.0429** and an r² of **0.0085**. The tier's
//! index is very nearly a measurement of enrolment, and the quantity it says it measures has no
//! size gradient at all.
//!
//! It is worth **$334,036,712.23** — 24.5% of targeted assistance — reaches 299 districts at a
//! median 876 ADM, and reaches **no district above 3,259.83 ADM**, which is the first district of
//! the largest sextile.
//!
//! # The two terms are the whole of the shape
//!
//! | ordered on base cost enrolled ADM | 1 | 2 | 3 | 4 | 5 | 6 | gap |
//! |---|--:|--:|--:|--:|--:|--:|--:|
//! | as the plan stands | **18** | 39 | 71 | 60 | 49 | 57 | **53** |
//! | staffing floors struck out | 48 | 51 | 74 | 61 | 49 | 57 | 26 |
//! | capacity tier struck out | 52 | 72 | 88 | 63 | 49 | 57 | 36 |
//! | **both struck out** | **89** | 82 | **89** | 64 | 49 | 57 | **0** |
//!
//! The first-to-third gap the floors leave behind closes to **zero**.
//!
//! # The size provision that was already recorded is the half that pushes the other way
//!
//! `the_remaining_categoricals.rs` recorded the tier's explicit size cliff — nothing below 200
//! ADM, 5% from 200 to 400, full only at 600 — and it is the opposite of the mechanism here. The
//! ramp **withholds** $91,859,433 from the 73 districts it touches, all of them small. Pay them in
//! full and the smallest band's guarantee count falls from 48 to **20**, two short of the 18 the
//! plan itself produces: **28 of the 30 districts the floors' removal added to that band go
//! straight back off the guarantee**. The drafters bounded the tier's size dependence with the one
//! provision visibly about size and left the rest of it in the index, where nothing names it.
//!
//! # It is the denominator and not the rate
//!
//! Take the shortfall against the statewide median wealth **per resident pupil** — $276,708.97,
//! the figure the tier's own sibling `[F]` already indexes against — times enrolled ADM. Same
//! section, same eight mills, same published median, **$24.2m less** in aggregate, and the row
//! reads `86 / 74 / 86 / 56 / 45 / 56` with a first-to-third gap of **0**.
//!
//! No lever was added and no shape is proposed. Holding the rate and very nearly the money fixed
//! and changing only what the shortfall is measured against is an attribution instrument.
//!
//! # Why the curve turns over above the third sextile
//!
//! The anchor's own shape, which is the candidate #409 named first. `[H2]` per FY2020 pupil
//! declines monotonically across the six bands — $6,478 to $3,176, the smallest at **2.04x** the
//! largest — and because it is monotone it cannot produce a peak, which is why it is not the
//! answer to the first question. It is the whole of the downward force. Flatten its sextile
//! profile and the curve rises with size; flatten formula aid's and it falls.
//!
//! # And the third candidate is refuted
//!
//! Local capacity **growth** does not correlate with size. Median published `[b1]` growth FY2026
//! to FY2027, by band: 10.01%, 9.09%, 9.76%, 9.01%, 9.54%, 8.84%.
//!
//! # What is not established here
//!
//! Why the tier is written against a total. Nothing committed derives it: R.C. 3317.0217 states
//! the comparison and no LSC analysis of any act that has carried it explains the choice, which is
//! the same negative result the staffing floors' thresholds produced and is asserted the same way
//! below — by reading both documents rather than by silence.

mod common;

use project::panel::categoricals::{
    TA_CAPACITY_MINIMUM_ADM, TA_MEDIAN_WEALTH_PER_PUPIL, TA_MEDIAN_WEIGHTED_WEALTH,
};
use project::panel::{panel, DistrictRecord, TargetedAssistance};
use project::size_terms::{
    anchor_per_pupil, capacity_growth_by_sextile, capacity_tier_per_pupil, capacity_tier_unramped,
    first_to_third_gap, guaranteed_by_sextile, profile, sextiles, striking_out,
    what_the_index_measures, with_a_side_flattened, Change, Side,
};
use project::statute;

/// A cent, the tolerance every dollar figure here is asserted at.
const CENT: f64 = 0.01;

/// The plan as it stands, which is the row every counterfactual is read against.
const AS_IT_STANDS: [usize; 6] = [18, 39, 71, 60, 49, 57];

/// **The bands, and the row the shape is stated on.**
///
/// Ordered on base cost enrolled ADM — the count every threshold in R.C. 3317.011 is against and
/// the one `staffing_minimums` orders on, so that this file's rows and that one's are comparable.
#[test]
fn the_guarantee_rate_by_sextile_is_non_monotone_and_peaks_in_the_third() {
    let districts = panel();
    assert_eq!(districts.len(), 609);

    let bands = sextiles(&districts);
    let sizes: Vec<usize> = bands.iter().map(Vec::len).collect();
    assert_eq!(
        sizes,
        vec![101, 101, 101, 101, 101, 104],
        "the remainder goes to the last band"
    );

    // The boundaries, so that a reader can place a district without rerunning anything.
    let edges: Vec<(f64, f64)> = bands
        .iter()
        .map(|band| {
            (
                band[0].base_cost_adm(),
                band[band.len() - 1].base_cost_adm(),
            )
        })
        .collect();
    for (low, high) in &edges {
        assert!(low <= high);
    }
    assert!(
        (edges[0].1 - 698.55).abs() < CENT,
        "first band ends at {}",
        edges[0].1
    );
    assert!(
        (edges[5].0 - 3_259.83).abs() < CENT,
        "last band starts at {}",
        edges[5].0
    );

    assert_eq!(guaranteed_by_sextile(&districts, &[]), AS_IT_STANDS);
    assert_eq!(first_to_third_gap(AS_IT_STANDS), 53);

    // Non-monotone is the whole premise, so it is asserted rather than described.
    assert!(
        AS_IT_STANDS[0] < AS_IT_STANDS[2] && AS_IT_STANDS[2] > AS_IT_STANDS[3],
        "the curve no longer peaks in the third band and this file needs redoing"
    );
}

/// **The section compares two whole-district totals, and says so.**
///
/// The premise of everything below, read off the Revised Code rather than inferred from the
/// department's columns.
#[test]
fn the_capacity_tier_compares_totals_and_the_wealth_tier_beside_it_compares_per_pupil_amounts() {
    let section = statute::section("3317.0217");
    let code = section
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // The capacity tier at (B)(4)(b)(i): the median district's weighted wealth against this
    // district's, both whole-district totals, with no pupil count anywhere in the comparison.
    assert!(
        code.contains(
            "(The median weighted wealth of all school districts in this state for that fiscal \
             year X 0.008) - (the district's weighted wealth for that fiscal year X 0.008)"
        ),
        "the capacity tier is not stated as this reading has it"
    );
    // The wealth tier one division later, which is the same shape with three words added.
    assert!(
        code.contains("X 0.014) - (the district's weighted wealth per pupil"),
        "the per-pupil sibling is not stated as this reading has it"
    );

    // And the two medians the section indexes against are the same quantity a pupil count apart,
    // which is the whole distinction: divide one by the other and what comes out is pupils.
    let districts = panel();
    let implied = TA_MEDIAN_WEIGHTED_WEALTH / TA_MEDIAN_WEALTH_PER_PUPIL;
    assert!(
        (implied - 1_417.2).abs() < 0.1,
        "the two medians are {implied:.1} pupils apart"
    );
    let median_adm = {
        let mut counts: Vec<f64> = districts
            .iter()
            .map(DistrictRecord::base_cost_adm)
            .collect();
        counts.sort_by(f64::total_cmp);
        counts[counts.len() / 2]
    };
    assert!(
        (implied / median_adm - 1.0).abs() < 0.15,
        "and that is the size of a middling Ohio district, {median_adm:.1} pupils — which is why \
         the tier's comparison reads enrolment and its sibling's does not"
    );
}

/// **The tier's index measures enrolment, and the quantity it names does not vary with size.**
#[test]
fn the_capacity_tiers_index_is_a_measurement_of_enrolment() {
    let districts = panel();
    let fits = what_the_index_measures(&districts);

    assert_eq!(fits.total.n, 609);
    assert_eq!(fits.per_pupil.n, 609);

    let total_slope = fits.total.coefficients[1];
    let per_pupil_slope = fits.per_pupil.coefficients[1];
    assert!(
        (total_slope - 0.985_481).abs() < 1e-5,
        "ln(total weighted wealth) on ln(ADM) has slope {total_slope}"
    );
    assert!(
        (fits.total.r_squared - 0.812_600).abs() < 1e-5,
        "and r2 {}",
        fits.total.r_squared
    );
    assert!(
        (per_pupil_slope - -0.042_899).abs() < 1e-5,
        "ln(wealth per resident pupil) on ln(ADM) has slope {per_pupil_slope}"
    );
    assert!(
        (fits.per_pupil.r_squared - 0.008_526).abs() < 1e-5,
        "and r2 {}",
        fits.per_pupil.r_squared
    );

    // The pair is the claim: a total that scales with enrolment is unremarkable, and it is
    // unremarkable because the per-pupil quantity beside it does not.
    assert!(
        fits.total.r_squared > 0.8 && fits.per_pupil.r_squared < 0.01,
        "four fifths of one and under a hundredth of the other"
    );
    assert!(
        (total_slope - 1.0).abs() < 0.02,
        "a slope of one is what makes it proportional rather than merely correlated"
    );
}

/// **What the tier is worth, and how far up the size distribution it reaches.**
#[test]
fn the_tier_is_a_third_of_a_billion_and_stops_at_three_thousand_two_hundred_pupils() {
    let districts = panel();

    let tier: f64 = districts
        .iter()
        .map(|r| r.targeted_assistance.capacity_amount)
        .sum();
    let targeted: f64 = districts
        .iter()
        .map(|r| r.targeted_assistance.total())
        .sum();
    assert!(
        (tier - 334_036_712.23).abs() < CENT,
        "the capacity tier is worth {tier:.2}"
    );
    assert!((targeted - 1_364_333_154.38).abs() < CENT);
    assert!((tier / targeted - 0.244_835).abs() < 1e-5);

    let reached: Vec<&DistrictRecord> = districts
        .iter()
        .filter(|r| r.targeted_assistance.capacity_amount > 0.0)
        .collect();
    assert_eq!(reached.len(), 299);

    let largest = reached
        .iter()
        .map(|r| r.base_cost_adm())
        .fold(0.0_f64, f64::max);
    assert!(
        (largest - 3_259.83).abs() < CENT,
        "the largest district it reaches is {largest:.2} ADM"
    );
    // Which is the first district of the largest band, so the tier and the top sextile are
    // disjoint but for one district.
    let bands = sextiles(&districts);
    assert!((bands[5][0].base_cost_adm() - largest).abs() < CENT);
    let counts: Vec<usize> = bands
        .iter()
        .map(|band| {
            band.iter()
                .filter(|r| r.targeted_assistance.capacity_amount > 0.0)
                .count()
        })
        .collect();
    assert_eq!(counts, vec![95, 93, 66, 39, 5, 1]);
}

/// **Strike both size terms out and the first-to-third gap is zero.**
///
/// The result the whole file is for.
#[test]
fn the_two_size_terms_are_the_whole_of_the_non_monotonicity() {
    let districts = panel();

    let floors = guaranteed_by_sextile(&districts, &[Change::StaffingFloorsStruckOut]);
    let tier = guaranteed_by_sextile(&districts, &[Change::CapacityTierStruckOut]);
    let both = guaranteed_by_sextile(
        &districts,
        &[
            Change::StaffingFloorsStruckOut,
            Change::CapacityTierStruckOut,
        ],
    );

    assert_eq!(floors, [48, 51, 74, 61, 49, 57]);
    assert_eq!(tier, [52, 72, 88, 63, 49, 57]);
    assert_eq!(both, [89, 82, 89, 64, 49, 57]);

    assert_eq!(first_to_third_gap(floors), 26, "the residual #409 named");
    assert_eq!(first_to_third_gap(tier), 36);
    assert_eq!(first_to_third_gap(both), 0, "and it closes");

    // Neither term reaches above the third band, which is why the turn-over needs another
    // account entirely.
    for band in 4..6 {
        assert_eq!(
            both[band],
            AS_IT_STANDS[band],
            "band {} moves under the counterfactual",
            band + 1
        );
    }

    // The counts are a threshold on a continuous quantity, so the dollars are asserted too: the
    // tier's own aggregate is exactly what leaves formula aid.
    let rows = striking_out(&districts, &[Change::CapacityTierStruckOut]);
    let moved: f64 = rows.iter().map(|r| r.aid_delta).sum();
    let tier_total: f64 = districts
        .iter()
        .map(|r| r.targeted_assistance.capacity_amount)
        .sum();
    assert!(
        (moved + tier_total).abs() < CENT,
        "a dollar off [C] is a dollar off [H] at a 100% phase-in"
    );
    assert_eq!(rows.iter().filter(|r| r.newly_guaranteed()).count(), 87);
    assert_eq!(rows.iter().filter(|r| r.no_longer_guaranteed()).count(), 0);
}

/// **The explicit size cliff is the part of the tier that works against the shape.**
#[test]
fn removing_the_tiers_size_ramp_puts_the_smallest_districts_further_off_the_guarantee() {
    let districts = panel();

    let touched = districts
        .iter()
        .filter(|r| TargetedAssistance::capacity_size_share(r.current_year_adm) < 1.0)
        .count();
    assert_eq!(touched, 73);

    let withheld: f64 = districts
        .iter()
        .map(|r| capacity_tier_unramped(r) - r.targeted_assistance.capacity_amount)
        .sum();
    assert!(
        (withheld - 91_859_433.46).abs() < CENT,
        "the ramp withholds {withheld:.2}"
    );
    assert!(withheld > 0.0, "the ramp can only withhold");

    let unramped = guaranteed_by_sextile(
        &districts,
        &[
            Change::StaffingFloorsStruckOut,
            Change::CapacityTierRampRemoved,
        ],
    );
    let floors = guaranteed_by_sextile(&districts, &[Change::StaffingFloorsStruckOut]);
    assert_eq!(unramped, [20, 51, 74, 61, 49, 57]);
    assert_eq!(floors[0], 48);
    assert_eq!(
        floors[0] - unramped[0],
        28,
        "paying the ramped districts in full takes 28 of the 30 the floors' removal added to the \
         smallest band straight back off the guarantee, which is the opposite of the direction a \
         size cliff is read in"
    );
    assert!(
        unramped[0] - AS_IT_STANDS[0] == 2,
        "two districts short of where the plan itself leaves that band"
    );

    // And it is a provision about size and nothing else: two districts equally far below the
    // median are paid different fractions of the same shortfall.
    assert!(TargetedAssistance::capacity_size_share(TA_CAPACITY_MINIMUM_ADM - 1.0) == 0.0);
    assert!(TargetedAssistance::capacity_size_share(700.0) == 1.0);
}

/// **Change the denominator alone and the residual goes.**
#[test]
fn measuring_the_tiers_shortfall_per_pupil_removes_the_residual_and_costs_less() {
    let districts = panel();

    let restated: f64 = districts
        .iter()
        .map(|r| capacity_tier_per_pupil(r) - r.targeted_assistance.capacity_amount)
        .sum();
    assert!(
        (restated - -24_182_202.67).abs() < CENT,
        "the restatement moves {restated:.2}"
    );
    assert!(restated < 0.0, "and it is cheaper, not dearer");

    let row = guaranteed_by_sextile(
        &districts,
        &[
            Change::StaffingFloorsStruckOut,
            Change::CapacityTierMeasuredPerPupil,
        ],
    );
    assert_eq!(row, [86, 74, 86, 56, 45, 56]);
    assert_eq!(first_to_third_gap(row), 0);

    // Same rate, same published median, same two pupil counts the section already uses.
    let one = districts
        .iter()
        .find(|r| r.targeted_assistance.wealth_per_pupil > TA_MEDIAN_WEALTH_PER_PUPIL)
        .expect("some district is above the median per pupil");
    assert!(
        capacity_tier_per_pupil(one) == 0.0,
        "a district above the median per pupil draws nothing under the restatement"
    );
}

/// **The anchor's own per-pupil gradient, which is why the curve comes back down.**
#[test]
fn the_anchor_falls_with_size_monotonically_and_so_cannot_be_the_peak() {
    let districts = panel();

    let anchor = anchor_per_pupil(&districts);
    let expected = [6_478.01, 5_876.69, 4_960.05, 4_588.65, 3_812.03, 3_176.14];
    for (got, want) in anchor.iter().zip(expected) {
        assert!((got - want).abs() < CENT, "{got:.2} against {want:.2}");
    }
    for pair in anchor.windows(2) {
        assert!(
            pair[0] > pair[1],
            "the anchor is not monotone across the bands and the argument below depends on it"
        );
    }
    assert!((anchor[0] / anchor[5] - 2.039_6).abs() < 1e-4);

    // Both size terms out, so what is left is the two sides against each other.
    let changes = [
        Change::StaffingFloorsStruckOut,
        Change::CapacityTierStruckOut,
    ];
    let shape = profile(&districts, &changes);
    assert_eq!(
        shape.districts, 607,
        "Richmond Heights and Buckeye Local are absent"
    );

    // The enrolment term is flat, which is what rules out enrolment loss as the residual.
    let widest = shape
        .enrollment
        .iter()
        .map(|x| x.abs())
        .fold(0.0_f64, f64::max);
    assert!(
        widest < 0.015,
        "the enrolment term spans {widest:.4} in logs"
    );
    let anchor_span = shape.anchor[0] - shape.anchor[5];
    assert!(
        anchor_span > 0.7 && widest * 40.0 < anchor_span,
        "the anchor spans {anchor_span:.4}, which is the comparison that makes 'flat' mean \
         something"
    );

    // Flatten one side's size profile and the curve resolves into its two forces.
    assert_eq!(
        with_a_side_flattened(&districts, &changes, Side::Anchor),
        [56, 59, 88, 70, 82, 90]
    );
    assert_eq!(
        with_a_side_flattened(&districts, &changes, Side::FormulaAid),
        [97, 90, 58, 71, 45, 16]
    );
}

/// **Capacity growth does not correlate with size, which refutes the third candidate.**
#[test]
fn local_capacity_grows_at_the_same_rate_in_every_band() {
    let districts = panel();
    let growth = capacity_growth_by_sextile(&districts);

    let expected = [0.1001, 0.0909, 0.0976, 0.0901, 0.0954, 0.0884];
    for (got, want) in growth.iter().zip(expected) {
        assert!((got - want).abs() < 5e-5, "{got:.4} against {want:.4}");
    }

    let (low, high) = growth
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), x| (lo.min(*x), hi.max(*x)));
    assert!(
        high - low < 0.013,
        "the widest gap between two bands is {:.4}, against the +9.34% the growth itself is",
        high - low
    );
    // Not ordered by size in either direction, which a gradient would be.
    assert!(
        growth[0] > growth[1] && growth[2] > growth[3] && growth[4] > growth[5],
        "it alternates rather than trends"
    );
}

/// **Nothing committed says why the tier is written against a total.**
///
/// The same negative result the staffing floors' thresholds produced, asserted the same way: by
/// reading both documents rather than by not having looked.
#[test]
fn neither_the_section_nor_the_greenbook_derives_the_comparison() {
    let section = statute::section("3317.0217");
    let code = section
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // The section states the comparison and offers no account of it — no purpose clause, no
    // recital, and no mention of district size anywhere but the ramp's own bracket.
    assert!(code.contains("X 0.008"));
    assert!(
        !code.to_lowercase().contains("because"),
        "the section explains itself after all and this reading needs redoing"
    );

    let lsc = project::greenbook::greenbook("hb110").flat();
    assert!(
        lsc.contains("credited with a capacity amount equal to 8 mills multiplied by"),
        "the greenbook restates the tier"
    );
    // It restates the arithmetic and derives nothing: the sentence that would explain the choice
    // of a total over a rate is the one that is not there.
    assert!(
        !lsc.contains(
            "8 mills multiplied by the amount by which the district\u{2019}s weighted \
                       wealth per pupil"
        ),
        "the greenbook states the capacity tier per pupil after all"
    );
}
