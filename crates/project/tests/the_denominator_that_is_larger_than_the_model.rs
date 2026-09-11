//! What a transportation proration factor divides by, and why FY2027's is below one.
//!
//! `parameter/appropriation-proration-factor` closes on two `[open]`s of the same shape. The
//! first: *"No factor publishes a history, so 'when each was last set' is answered for preschool
//! and open for the other two."* The second, in `statutory_basis`: *"Which budget line governs
//! each, and whether the department's discretion in setting the factor is bounded."*
//!
//! Both are answerable, and the instrument is one the corpus already built for a proration two
//! decades older. `the_dial_that_was_an_appropriation` established that a transportation
//! proration is not a dial but a quotient, and quoted LSC stating the formula in as many words —
//! *"Adjustment percentage = (Earmarked appropriation)/(Total statewide allocation)"*. What was
//! missing for this biennium was the two halves: the numerator, which is in the enacted budget
//! analysis, and the denominator, which is in the calculator. Both were in the repository.
//!
//! # The answer is that the denominator is bigger than the model
//!
//! Divide the earmark by the factor and the statewide allocation comes to **$212.3m**. The 611
//! districts in the department's own workbook account for **$199.1m** of it. The $13.3m left over
//! is not a rounding and it is not a mystery — the greenbook's description of the earmark says
//! who it belongs to, and they are not school districts.
//!
//! So a proration factor is the one number in the calculator that measures a population the
//! calculator does not contain. A reader who divided the earmark by the districts it can see
//! would get **0.9787**, six points too high, and would have no way of knowing.
//!
//! # And it prorates because the floor stepped, not because the line was cut
//!
//! The earmark **rose 10.13%**. The allocation rose 20.56%, and the three things that moved do
//! not move together: the prior year's reported costs added $26.8m, the state share percentages
//! fell and took $4.2m back off, and the statutory floor under the state share — which H.B. 96
//! steps by a twenty-fourth a year — added **$11.3m** on its own. At the floor it stepped from,
//! the districts fit inside the earmark with $7.1m to spare.

use project::budget_analysis::{
    pupil_transportation, pupil_transportation_earmarks, Edition, ALI_200502_EARMARK_TOTAL,
    FOUNDATION_AID_REMAINDER, SPECIAL_EDUCATION_TRANSPORTATION,
};
use project::transport::{self, YEARS};

/// Half a cent, which is the width of the panel fixture's rounding and nothing else.
const CENT: f64 = 0.005;

/// A dollar, for comparisons between figures the documents publish whole.
const DOLLAR: f64 = 1.0;

/// The factor is the payment over the allocation, on every district and in both years.
#[test]
fn every_district_is_paid_the_floor_times_its_cost_times_the_year_s_factor() {
    for fiscal_year in [YEARS.0, YEARS.1] {
        let rates = transport::rates_for(fiscal_year).expect("a committed year");
        let costs = transport::sped_costs(fiscal_year).expect("a modelled year");
        for district in &costs {
            let owed = district.state_share.max(rates.minimum_state_share)
                * district.reported_cost
                * rates.sped_proration;
            assert!(
                (owed - district.paid).abs() < CENT,
                "FY{fiscal_year} {}: {owed:.4} against a published {:.4}",
                district.irn,
                district.paid
            );
        }

        // And therefore statewide, which is the half of the quotient the calculator holds.
        let allocation = transport::special_education(fiscal_year).expect("a modelled year");
        assert!(
            (allocation.paid / allocation.districts - rates.sped_proration).abs() < 1e-9,
            "FY{fiscal_year}: {} over {} is not {}",
            allocation.paid,
            allocation.districts,
            rates.sped_proration
        );
    }
}

/// **The finding.** The factor's denominator reaches past every district the model contains.
#[test]
fn the_denominator_reaches_past_the_districts_the_calculator_holds() {
    let late = transport::special_education(YEARS.1).expect("FY2027 is modelled");

    // The numerator is the enacted earmark, and the denominator solved for from the factor.
    assert!((late.appropriation - 194_820_866.0).abs() < DOLLAR);
    assert!((late.implied_statewide() - 212_348_136.16).abs() < 0.01);
    assert!((late.districts - 199_061_038.65).abs() < 0.01);
    assert!((late.outside_the_model() - 13_287_097.51).abs() < 0.01);

    /*
     * What a reader who divided the earmark by the model would have got. Six points is not a
     * rounding disagreement, and the direction matters: it is *higher*, so the mistake would be
     * to under-report how far the appropriation fell short.
     */
    assert!((late.districts_only_factor() - 0.978_699_133).abs() < 1e-8);
    assert!(late.districts_only_factor() - late.stated_factor > 0.06);

    /*
     * FY2026 bounds the same quantity from the other side, and this is why the year that does not
     * prorate had to be extracted. A factor of 1.0 says the whole statewide allocation fitted
     * inside the earmark; the districts took $165.1m of a $176.9m line, so everything else in the
     * allocation was at most $11.8m that year, against the $13.3m FY2027 measures.
     */
    let early = transport::special_education(YEARS.0).expect("FY2026 is modelled");
    assert!((early.stated_factor - 1.0).abs() < 1e-12);
    assert!((early.districts - 165_119_518.32).abs() < 0.01);
    assert!((early.headroom() - 11_778_159.68).abs() < 0.01);
    assert!(
        early.headroom() < late.outside_the_model(),
        "the FY2026 ceiling no longer sits under the FY2027 measurement"
    );
}

/// And the greenbook names the population, in the paragraph that describes the earmark.
#[test]
fn the_greenbook_says_who_else_is_paid_out_of_the_earmark() {
    let enacted = Edition::Enacted.text().replace(['\n', '\r'], " ");
    let flat = enacted.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(
        flat.contains(
            "County DD boards and ESCs are funded through a nearly identical special education \
             transportation formula, except that the state share percentage for these entities \
             is a uniform amount equal to the minimums for traditional districts."
        ),
        "the earmark's description no longer names the entities outside the calculator"
    );

    // Which also fixes their rate, so the $13.3m is a cost of about twice that at the FY2027
    // floor. Stated as an implication rather than asserted: nothing here publishes their cost.
    let late = transport::special_education(YEARS.1).expect("FY2027 is modelled");
    let implied_cost = late.outside_the_model() / 0.5;
    assert!(
        implied_cost > 26_000_000.0 && implied_cost < 27_000_000.0,
        "{implied_cost:.2}"
    );
}

/// **Why it prorates.** The appropriation rose; the floor rose further.
#[test]
fn the_floor_step_is_what_pushed_the_allocation_past_the_line() {
    let movement = transport::movement().expect("both years are modelled");

    // The three deltas are a path, so they sum to the movement exactly.
    let total = movement.to - movement.from;
    assert!((movement.cost() + movement.state_share() + movement.floor() - total).abs() < 0.01);
    assert!((total - 33_941_520.33).abs() < 0.01, "{total:.2}");

    assert!((movement.cost() - 26_808_906.80).abs() < 0.01);
    assert!(
        movement.state_share() < 0.0,
        "the state share percentages no longer fall"
    );
    assert!((movement.state_share() + 4_207_654.26).abs() < 0.01);
    assert!((movement.floor() - 11_340_267.79).abs() < 0.01);

    /*
     * And the counterfactual the decomposition exists for. `after_state_share` is FY2027's costs
     * and FY2027's state shares at FY2026's floor — the year as it would have been if H.B. 96 had
     * not stepped the minimum — and at that level the districts sit $7.1m *inside* the earmark
     * rather than $4.2m outside it.
     */
    let late = transport::special_education(YEARS.1).expect("FY2027 is modelled");
    assert!(late.headroom() < 0.0, "the districts now fit on their own");
    assert!((late.headroom() + 4_240_172.65).abs() < 0.01);

    let held = late.appropriation - movement.after_state_share;
    assert!(held > 0.0, "the old floor no longer fits inside the line");
    assert!((held - 7_100_095.14).abs() < 0.01, "{held:.2}");

    // The floor itself, stated where the counterfactual is, so the two cannot drift apart.
    let (early, late) = (
        transport::rates_for(YEARS.0).expect("FY2026"),
        transport::rates_for(YEARS.1).expect("FY2027"),
    );
    assert!((early.minimum_state_share - 0.4583).abs() < 1e-4);
    assert!((late.minimum_state_share - 0.50).abs() < 1e-12);
}

/// A factor of 1.0 is a measurement, and the general line's says how close it came.
#[test]
fn the_general_transportation_factor_held_at_one_by_less_than_a_percent_in_fy2026() {
    let (early_paid, early_line) = transport::general_headroom(YEARS.0).expect("FY2026");
    let (late_paid, late_line) = transport::general_headroom(YEARS.1).expect("FY2027");

    // The remainder is the document's own row, not this crate's subtraction of the earmarks.
    assert!((early_line - 703_348_806.0).abs() < DOLLAR);
    assert!((late_line - 762_819_905.0).abs() < DOLLAR);
    assert!(
        (early_line + transport::earmarked(YEARS.0).expect("FY2026") - 882_035_414.0).abs()
            < DOLLAR,
        "the split no longer adds to the FY2026 line"
    );

    let (early, late) = (early_line - early_paid, late_line - late_paid);
    assert!((early - 5_594_475.23).abs() < 0.01, "{early:.2}");
    assert!((late - 36_760_324.87).abs() < 0.01, "{late:.2}");

    /*
     * Six and a half times as much room a year later, on a line that grew 8.5%. The general
     * formula's 1.0 in FY2026 is therefore a *near* miss and FY2027's is not, which is the thing
     * a factor of 1.0 cannot say about itself — and it is the same direction the special
     * education earmark moved in, on the same appropriation, with the opposite outcome.
     */
    assert!(early / early_line < 0.01, "{:.4}", early / early_line);
    assert!(late / late_line > 0.045, "{:.4}", late / late_line);
}

/// The earmark that binds is the one line of ALI 200502 the legislature left alone.
#[test]
fn the_special_education_earmark_survived_the_legislature_untouched() {
    let earmark =
        |edition| pupil_transportation_earmarks(edition, SPECIAL_EDUCATION_TRANSPORTATION);
    let (proposed, enacted) = (earmark(Edition::Introduced), earmark(Edition::Enacted));

    assert_eq!(proposed.first, enacted.first);
    assert_eq!(proposed.second, enacted.second);
    assert_eq!(enacted.first, 176_897_678.0);
    assert_eq!(enacted.second, 194_820_866.0);

    /*
     * Which is only a finding beside the rows that did move. Bus driver training lost $3.9m of
     * its FY2027 proposal, two new earmarks arrived, and the unearmarked remainder — what the
     * general transportation formula is paid from — was raised in both years. So the conference
     * committee was in this line item, and the half of it that prorates is the half it did not
     * open.
     */
    let training = |edition| pupil_transportation_earmarks(edition, "Bus driver training");
    assert_eq!(training(Edition::Introduced).second, 4_988_930.0);
    assert_eq!(training(Edition::Enacted).second, 1_088_930.0);

    let total = |edition| pupil_transportation_earmarks(edition, ALI_200502_EARMARK_TOTAL);
    assert_eq!(total(Edition::Introduced).second, 199_809_796.0);
    assert_eq!(total(Edition::Enacted).second, 196_609_796.0);

    let remainder = |edition| pupil_transportation(edition, FOUNDATION_AID_REMAINDER);
    assert_eq!(remainder(Edition::Introduced).second, 755_819_905.0);
    assert_eq!(remainder(Edition::Enacted).second, 762_819_905.0);
    assert!(remainder(Edition::Enacted).first > remainder(Edition::Introduced).first);
}

/// The floor binds on more districts' payments than it did, and on more than it has to.
#[test]
fn the_floor_pays_four_hundred_and_eleven_districts_in_fy2027() {
    let (early, late) = (
        transport::special_education(YEARS.0).expect("FY2026"),
        transport::special_education(YEARS.1).expect("FY2027"),
    );
    assert_eq!(early.at_the_floor, 332);
    assert_eq!(late.at_the_floor, 411);

    // Both counts are of districts that reported a cost, which is what makes them comparable —
    // 52 of the department's 611 reported nothing in FY2026 and 46 of the panel's 609 in FY2027,
    // and a district with no cost is under the floor and paid nothing either way.
    let reporting = |fiscal_year| {
        transport::sped_costs(fiscal_year)
            .expect("a modelled year")
            .iter()
            .filter(|c| c.reported_cost > 0.0)
            .count()
    };
    assert_eq!(reporting(YEARS.0), 559);
    assert_eq!(reporting(YEARS.1), 563);
}

/// The preschool factor moved alone: every other cell of its block holds across the two years.
///
/// Not this file's subject, and it is here because it *sharpens an `[open]`* rather than closing
/// one. `parameter/appropriation-proration-factor` records that the FY2027 preschool factor is
/// six millionths off FY2026's and that its own year's arithmetic produces no proration at all.
/// What the scalar fixture adds is that the factor is the one cell in that header block which
/// differs — so it was typed into a sheet otherwise carried forward whole, rather than
/// recomputed along with everything beside it.
#[test]
fn the_preschool_factor_is_the_only_cell_of_its_block_that_moved() {
    let (moved, held) = project::prior_model::scalars_that_moved();

    for cell in [
        "preschool_base_amount",
        "preschool_appropriation_cell",
        "average_base_cost_per_pupil",
        "special_education_weight_1",
        "special_education_weight_2",
        "special_education_weight_3",
        "special_education_weight_4",
        "special_education_weight_5",
        "special_education_weight_6",
    ] {
        assert!(held.iter().any(|name| name == cell), "{cell} moved");
    }
    assert!(moved
        .iter()
        .any(|name| name == "preschool_proration_factor"));
}
