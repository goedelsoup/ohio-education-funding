//! What a nominal freeze costs, per parameter and per district.
//!
//! Two corpus requests and one test comment ask for the same thing in the same words.
//! `parameter/gifted-funding-rates`: *"The corpus's `deflator` is the tool for that and no
//! constant-dollar companion series exists here yet."* `the_parameters_before_this_biennium`, on
//! the preschool flat grant: *"it is the part a constant-dollar series would say most about — a
//! nominal $4,000 unmoved since at least FY2014."*
//!
//! [`project::base_cost`] answered it for the one parameter with its own module. These are the
//! parameters beside it.
//!
//! # The three kinds of check here
//!
//! **That the table is sourced.** Every amount carries a sentence from a committed LSC analysis
//! that dates it, and the sentence has to be in the greenbook and to contain the amount. A
//! codified section states only its current numbers, so the act's own analysis is the only thing
//! in this workspace that can say *when* a figure stopped moving — which is the fact the whole
//! module rests on.
//!
//! **That the deflation is right.** Against `deflator`, which is checked against BLS elsewhere.
//!
//! **That the incidence is the department's own arithmetic.** Each parameter enters its formula
//! linearly, so scaling a rate scales its aid by the same fraction; the only non-linearity is the
//! guarantee, and it is applied here exactly as `DistrictRecord` carries it.

use edfund_core::FiscalYear;
use project::greenbook;
use project::indexation::{
    self, Component, Maintenance, Standing, BASE_YEAR, PARAMETERS, TRANSPORTATION_MINIMUM_SHARE,
};
use project::panel::{panel, DistrictRecord};

/// `4000.0` as `$4,000`, which is how LSC prints an amount.
fn printed(dollars: f64) -> String {
    let whole = dollars.round() as i64;
    let digits = whole.abs().to_string();
    let mut out = String::from("$");
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

fn districts() -> Vec<DistrictRecord> {
    panel()
}

/// Every amount in the table is dated by a sentence a committed greenbook actually contains.
///
/// The sentence is the evidence for `since`, and `since` is what the deflation runs from, so an
/// unsourced date would silently move every figure in this file.
#[test]
fn every_parameter_is_dated_by_a_sentence_in_a_committed_analysis() {
    for parameter in PARAMETERS {
        let carrying: Vec<&str> = greenbook::greenbooks()
            .iter()
            .filter(|book| book.flat().contains(parameter.dated_by))
            .map(|book| book.bill)
            .collect();
        assert!(
            !carrying.is_empty(),
            "no committed analysis contains the sentence dating {}",
            parameter.name
        );
    }
}

/// And the sentence prints the amount, so the quote is evidence of *this* figure.
///
/// Two of the seven are dated by a sentence that states a ratio rather than a price — the three
/// unit salaries are set by R.C. 3317.051 and the analysis states the staffing ratio beside them
/// — so those are exempted explicitly rather than by a loose rule.
#[test]
fn the_dating_sentence_prints_the_amount_except_where_it_states_a_ratio() {
    let ratio_dated = ["one gifted intervention specialist unit to 140 gifted students"];
    for parameter in PARAMETERS {
        if ratio_dated.contains(&parameter.dated_by) {
            continue;
        }
        let amount = printed(parameter.nominal);
        assert!(
            parameter.dated_by.contains(&amount)
                || parameter
                    .dated_by
                    .contains(&format!("${}", parameter.nominal)),
            "{} is dated by a sentence that does not print {amount}",
            parameter.name
        );
    }
}

/// The base year is the last one with a price index, and it is a year behind the panel.
///
/// Stated as a test because it is the caveat that makes every figure here a lower bound: the
/// district amounts are the department's FY2027 model and the index stops at June 2026.
#[test]
fn the_base_year_is_the_last_june_the_index_reaches() {
    assert_eq!(BASE_YEAR, FiscalYear(2026));
    let cpi = deflator::CpiSeries::cpi_u_june();
    assert!(cpi.point(FiscalYear(2026)).is_some());
    assert!(
        cpi.point(FiscalYear(2027)).is_none(),
        "June 2027 has an index now, so this module can state the erosion against the year it is \
         actually paid in and the lower-bound caveat should go"
    );
}

/// What each frozen amount is worth in FY2026 dollars.
///
/// The preschool grant is the outlier and it is the outlier for one reason: it has been frozen
/// three times as long as the others.
#[test]
fn the_frozen_amounts_in_constant_dollars() {
    let real = |name: &str| {
        PARAMETERS
            .iter()
            .find(|p| p.name == name)
            .expect("the parameter is in the table")
            .real()
            .expect("a frozen parameter deflates")
    };
    let close = |got: f64, want: f64| {
        assert!((got - want).abs() < 0.01, "{got} is not {want}");
    };

    close(real("preschool flat grant"), 5_604.56);
    close(real("DPIA per pupil"), 475.61);
    close(real("gifted identification"), 27.05);
    close(real("gifted referral"), 2.82);
    close(real("gifted coordinator unit"), 96_672.30);
    close(real("gifted K-8 specialist unit"), 100_731.87);
    close(real("gifted 9-12 specialist unit"), 91_260.29);
}

/// Two freeze vintages, two erosions, and every parameter of a vintage shares one.
///
/// Worth asserting as a group rather than one by one: the erosion is a property of *when* the
/// amount was set and of nothing else, which is what makes the preschool grant's 28.6% a
/// statement about legislative attention rather than about preschool.
#[test]
fn the_erosion_is_a_property_of_the_vintage_alone() {
    let mut by_vintage: std::collections::BTreeMap<u16, Vec<f64>> =
        std::collections::BTreeMap::new();
    for parameter in PARAMETERS {
        let Maintenance::Frozen { since } = parameter.maintenance else {
            continue;
        };
        by_vintage
            .entry(since)
            .or_default()
            .push(parameter.erosion().expect("a frozen parameter erodes"));
    }
    assert_eq!(
        by_vintage.keys().copied().collect::<Vec<_>>(),
        vec![2014, 2022]
    );

    for (vintage, erosions) in &by_vintage {
        let first = erosions[0];
        for erosion in erosions {
            assert!(
                (erosion - first).abs() < 1e-12,
                "FY{vintage} amounts do not share an erosion"
            );
        }
    }
    assert!((by_vintage[&2014][0] - 0.2863).abs() < 0.0001);
    assert!((by_vintage[&2022][0] - 0.1127).abs() < 0.0001);
}

/// The one parameter that rises does so on a rule, not by amendment each time.
///
/// Eight to twelve twenty-fourths. This is the contrast the rest of the file is measured against:
/// the plan is perfectly capable of indexing a parameter, and does, for exactly one of them.
#[test]
fn the_transportation_share_rises_on_a_schedule_of_twenty_fourths() {
    for (step, (year, stated)) in (8..=12).zip(TRANSPORTATION_MINIMUM_SHARE) {
        let exact = f64::from(step) / 24.0 * 100.0;
        assert!(
            (stated - (exact * 100.0).round() / 100.0).abs() < 1e-9,
            "FY{year}'s {stated:.2}% is not {step}/24"
        );
    }
    assert_eq!(TRANSPORTATION_MINIMUM_SHARE[0].0, 2023);
    assert_eq!(TRANSPORTATION_MINIMUM_SHARE[4].0, 2027);
}

/// What the freeze costs statewide, and what the guarantee absorbs of it.
#[test]
fn the_statewide_shortfall_and_what_the_guarantee_keeps_from_reaching_districts() {
    let statewide = indexation::statewide(&districts());
    let close = |got: f64, want: f64| assert!((got - want).abs() < 1.0, "{got} is not {want}");

    close(statewide.computed, 133_152_626.93);
    close(statewide.delivered, 104_298_429.45);
    close(statewide.computed - statewide.delivered, 28_854_197.47);
    assert_eq!(statewide.held, 294);

    // 78.3% of it would reach districts. The rest is absorbed by guarantee floors, which is the
    // same mechanism `who_a_change_reaches` measures for a base cost increase.
    let reaching = statewide.delivered / statewide.computed;
    assert!((0.78..0.79).contains(&reaching), "{reaching}");
}

/// Every district is reached, and for the held ones that is entirely the component outside `[H]`.
///
/// This is the sharpest fact in the module. Of 294 districts the guarantee holds, the in-formula
/// shortfall is absorbed **in full** for 282 and in part for 12 — and all 294 are still reached,
/// because preschool special education sits outside `[H] Foundation Funding` and so outside the
/// `max` the guarantee applies.
///
/// So the parameter that has lost the most value is also the one no floor cushions.
#[test]
fn the_guarantee_absorbs_the_formula_components_and_cannot_touch_the_third() {
    let records = districts();
    let rows = indexation::incidence(&records);

    assert_eq!(rows.len(), 609);
    assert!(
        rows.iter().all(|row| row.delivered > 0.0),
        "a district receives none of it"
    );

    let (mut absorbed_whole, mut absorbed_part) = (0, 0);
    for row in rows.iter().filter(|row| row.on_guarantee) {
        let beside: f64 = row
            .by_component
            .iter()
            .filter(|(component, _)| component.standing() == Standing::BesideFormula)
            .map(|(_, shortfall)| shortfall)
            .sum();
        if (row.delivered - beside).abs() < 0.01 {
            absorbed_whole += 1;
        } else {
            absorbed_part += 1;
        }
    }
    assert_eq!((absorbed_whole, absorbed_part), (282, 12));

    // And the standing is a property of the component rather than of the district.
    assert_eq!(Component::Dpia.standing(), Standing::InFormula);
    assert_eq!(Component::Gifted.standing(), Standing::InFormula);
    assert_eq!(
        Component::PreschoolSpecialEducation.standing(),
        Standing::BesideFormula
    );
}

/// Which components carry it.
///
/// DPIA is half of it on the strength of its size rather than its erosion — it is frozen the same
/// six years as gifted and is ten times the money. Preschool is 45% of the total on $59.5m of aid,
/// which is the 28.6% erosion doing the work.
#[test]
fn the_shortfall_by_component() {
    let by_component = indexation::by_component(&districts());
    let close = |got: f64, want: f64| assert!((got - want).abs() < 1.0, "{got} is not {want}");

    close(by_component[&Component::Dpia], 66_703_817.98);
    close(
        by_component[&Component::PreschoolSpecialEducation],
        59_532_514.24,
    );
    close(by_component[&Component::Gifted], 6_916_294.71);
}

/// Who pays for it: the shortfall per pupil rises monotonically with district poverty.
///
/// Quintiles of economically disadvantaged percentage, and the gradient is **four to one** from
/// the wealthiest fifth to the poorest. Nothing in the statute chose that. It falls out of DPIA
/// being the largest of the three frozen components and DPIA being paid on a poverty index — so
/// letting a nominal amount decay is a distributive act whether or not it was meant as one.
///
/// The direction matters as much as the size. A reader who knows only that three rates are frozen
/// might reasonably guess the loss is spread evenly per pupil; it is not, and the districts least
/// able to replace state money from local property are the ones losing most of it.
#[test]
fn the_shortfall_per_pupil_rises_with_district_poverty() {
    let records = districts();
    let rows = indexation::incidence(&records);
    let mut paired: Vec<(f64, f64)> = records
        .iter()
        .zip(&rows)
        .map(|(record, row)| (record.dpia.percentage, row.per_pupil))
        .collect();
    paired.sort_by(|a, b| a.0.total_cmp(&b.0));

    let quintile_mean = |index: usize| {
        let n = paired.len();
        let slice = &paired[index * n / 5..(index + 1) * n / 5];
        slice.iter().map(|pair| pair.1).sum::<f64>() / slice.len() as f64
    };
    let means: Vec<f64> = (0..5).map(quintile_mean).collect();

    for pair in means.windows(2) {
        assert!(pair[1] > pair[0], "the gradient is not monotone: {means:?}");
    }
    let close = |got: f64, want: f64| assert!((got - want).abs() < 0.01, "{got} is not {want}");
    close(means[0], 34.69);
    close(means[1], 43.17);
    close(means[2], 53.03);
    close(means[3], 94.95);
    close(means[4], 141.66);

    assert!(
        means[4] / means[0] > 4.0,
        "the poorest fifth loses {:.2}x what the wealthiest does",
        means[4] / means[0]
    );
}

/// The incidence is the department's own arithmetic plus one `max`, and this shows it is linear.
///
/// Each of these parameters enters as `rate × count × share`, so a district's shortfall is its own
/// aid times a factor that depends only on the freeze vintage. Asserted directly against the
/// panel's aid columns so that a change in how the shortfall is computed has to disagree with the
/// definition rather than only with a pinned total.
#[test]
fn each_districts_shortfall_is_its_own_aid_times_the_vintage_factor() {
    let records = districts();
    let rows = indexation::incidence(&records);
    let factor = |name: &str| {
        PARAMETERS
            .iter()
            .find(|p| p.name == name)
            .expect("in the table")
            .shortfall_factor()
            .expect("frozen")
    };

    for (record, row) in records.iter().zip(&rows) {
        let expected = record.categoricals.dpia * factor("DPIA per pupil")
            + record.categoricals.gifted * factor("gifted identification")
            + record.preschool_special_education.total * factor("preschool flat grant");
        assert!(
            (row.computed - expected).abs() < 0.01,
            "{} computes {} against {expected}",
            record.name,
            row.computed
        );
    }
}
