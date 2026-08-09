//! Build the web feed from the district panel and the profile-report fixture.
//!
//! ```text
//! cargo run -p bundle > ../web/public/data/bundle.json
//! ```
//!
//! A thin shell: it takes [`project::panel`] as the source of the funding model, joins the
//! District Profile Report for millage, expenditure, and demographics, computes the statewide
//! aggregates, runs a fixed set of policies to produce [`bundle::Checkpoint`]s, and prints JSON.
//! All the logic worth testing lives in the two libraries.

use std::collections::HashMap;

use bundle::{
    BaseCostBuildUp, Bundle, CareerTechnical, Categoricals, Checkpoint, Deflator, District,
    DistrictOutcome, Dpia, EnglishLearners, FinanceYear, ForecastCheckpoint, Gifted,
    HouseDistrictMember, HouseDistrictShare, MillageAnalysis, National, OutcomeStatewide,
    PolicyShape, Projection, PropertyTaxYear, RegimeCounterfactual, SpecialEducation,
    SpendingByFunction, StateFinance, Statewide, TargetedAssistance, CONTRACT_VERSION,
};
use dispersion::{partial_correlation, wealth_neutrality};
use edfund_core::{AgencyType, FiscalYear};
use foundation::{aggregate_base_cost, StatewideFactors};
use project::finances::{finances, for_district, Finances};
use project::house_district::{house_districts, overlaps};
use project::outcomes::{joined, Joined};
use project::panel::{panel, DistrictRecord, HISTORY_YEARS, MINIMUM_STATE_SHARE, MODEL_YEAR};
use project::policy::{GuaranteeRule, Policy};
use project::report::{enrollment_growth_prior, forecast, simulate};
use project::series::{Method, DEFAULT_DAMPING, ONE_SIGMA};
use scenario_delta::ScenarioDelta;

/// The furthest year the page will offer, ten past the last observation.
///
/// Not further: the projection is damped precisely because an undamped trend produces a
/// confident and absurd number at a long horizon, and offering FY2050 would invite one anyway.
const HORIZON: FiscalYear = FiscalYear(2036);

/// The FY2024 District Profile Report: millage, expenditure, demographics.
const PROFILE: &str = include_str!("../../dispersion/fixtures/cupp-fy24-district-data.csv");

/// Table SD-1: taxable value by class and taxes charged, one row per district per tax year.
///
/// The Department of Taxation's table rather than the Department of Education's — the local half
/// of the funding formula, from the half of the state that measures it.
const SD1: &str = include_str!("../../dispersion/fixtures/sd1-district-taxes.csv");

/// The report card's FY2025 operating spending, broken into functions, per pupil.
const FUNCTIONS: &str = include_str!("../../dispersion/fixtures/expenditure-functions-fy25.csv");

/// The Census Bureau's Annual Survey of School System Finances, aggregated to states.
///
/// The only federal source in the feed, and the only one that can say whether Ohio is unusual.
const F33: &str = include_str!("../../dispersion/fixtures/census-f33-states.csv");

fn field(line: &str, index: usize) -> Option<&str> {
    line.split(',').nth(index).map(str::trim)
}

fn parse(line: &str, index: usize) -> Option<f64> {
    field(line, index).and_then(|value| value.parse::<f64>().ok())
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[values.len() / 2]
}

/// The policies the web layer must reproduce before it may compute its own.
///
/// Chosen to exercise every lever and both sides of the guarantee's `max`: removal and a
/// partial phase-out take different paths through it, and a base cost increase moves districts
/// across the threshold in a way no purely linear scenario would.
fn checkpoint_policies() -> Vec<(&'static str, Policy, PolicyShape)> {
    let shape = |guarantee, argument, base_cost_scale, minimum_state_share, base, categorical| {
        PolicyShape {
            guarantee,
            guarantee_argument: argument,
            base_cost_scale,
            minimum_state_share,
            phase_in_base_cost: base,
            phase_in_categorical: categorical,
        }
    };
    vec![
        (
            "current law",
            Policy::current_law(),
            shape("as-enacted", 0.0, 1.0, MINIMUM_STATE_SHARE, 1.0, 1.0),
        ),
        (
            "guarantee removed",
            Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            shape("removed", 0.0, 1.0, MINIMUM_STATE_SHARE, 1.0, 1.0),
        ),
        (
            "guarantee half phased out",
            Policy {
                guarantee: GuaranteeRule::PhasedOut { remaining: 0.5 },
                ..Policy::current_law()
            },
            shape("phase-out", 0.5, 1.0, MINIMUM_STATE_SHARE, 1.0, 1.0),
        ),
        (
            "guarantee rebased to 90%",
            Policy {
                guarantee: GuaranteeRule::Rebased { factor: 0.9 },
                ..Policy::current_law()
            },
            shape("rebase", 0.9, 1.0, MINIMUM_STATE_SHARE, 1.0, 1.0),
        ),
        (
            "base cost +5%",
            Policy {
                base_cost_scale: 1.05,
                ..Policy::current_law()
            },
            shape("as-enacted", 0.0, 1.05, MINIMUM_STATE_SHARE, 1.0, 1.0),
        ),
        (
            "minimum state share 15%",
            Policy {
                minimum_state_share: 0.15,
                ..Policy::current_law()
            },
            shape("as-enacted", 0.0, 1.0, 0.15, 1.0, 1.0),
        ),
        (
            "base cost +10%, guarantee removed",
            Policy {
                base_cost_scale: 1.10,
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            shape("removed", 0.0, 1.10, MINIMUM_STATE_SHARE, 1.0, 1.0),
        ),
        (
            "phase-in 50% base cost, 0% categorical",
            Policy {
                phase_in_base_cost: 0.5,
                phase_in_categorical: 0.0,
                ..Policy::current_law()
            },
            shape("as-enacted", 0.0, 1.0, MINIMUM_STATE_SHARE, 0.5, 0.0),
        ),
    ]
}

/// The (policy, year) pairs the web layer must reproduce before it may draw a band.
///
/// Chosen so that reproducing them is not reproducible by accident. Current law and the
/// guarantee removed at the *same* year is the pair that pins the guarantee's role as a shock
/// absorber — if the page's band is right for one and wrong for the other, it has got the kink
/// in the aid curve wrong rather than the arithmetic. A short horizon checks that the interval
/// compounds with the square root of the years rather than linearly, and a long one at a moved
/// base cost checks that damping is applied per year rather than once.
fn forecast_years() -> Vec<(&'static str, usize, FiscalYear)> {
    vec![
        ("current law, FY2028", 0, FiscalYear(2028)),
        ("current law, FY2032", 0, FiscalYear(2032)),
        ("guarantee removed, FY2032", 1, FiscalYear(2032)),
        ("base cost +5%, FY2036", 4, HORIZON),
    ]
}

/// Pearson correlation over two equal-length series.
fn correlation(xs: &[f64], ys: &[f64]) -> f64 {
    wealth_neutrality(xs, ys).map_or(f64::NAN, |w| w.correlation)
}

/// `xs` against `ys` holding `control` constant.
fn controlling_for(xs: &[f64], ys: &[f64], control: &[f64]) -> f64 {
    partial_correlation(
        correlation(xs, ys),
        correlation(xs, control),
        correlation(ys, control),
    )
    .unwrap_or(f64::NAN)
}

/// Three aligned series over the districts where every one of the three is present.
fn aligned(
    records: &[Joined],
    x: impl Fn(&Joined) -> Option<f64>,
    y: impl Fn(&Joined) -> Option<f64>,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut control = Vec::new();
    for record in records {
        let (Some(a), Some(b), Some(c)) = (x(record), y(record), record.economically_disadvantaged)
        else {
            continue;
        };
        xs.push(a);
        ys.push(b);
        control.push(c);
    }
    (xs, ys, control)
}

/// Where Ohio sits among the states, from the Census survey.
///
/// # Why the property tax rank is taken over a subset and the others are not
///
/// Twelve states fund schools through a parent city or county rather than through a district that
/// levies for itself, so the survey attributes their property tax to the parent and reports the
/// district's own as zero. Ranking all fifty-one on property tax share would put Massachusetts and
/// Virginia at the bottom of a measure they are near the top of. Local revenue includes the parent
/// appropriation and is comparable either way, so that rank is over everyone and the property tax
/// rank is over the thirty-nine that report one.
fn national() -> Option<National> {
    let head = header(F33);
    let states: Vec<StateFinance> = F33
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let p: Vec<&str> = line.split(',').collect();
            let n = |name: &str| number(&head, &p, name);
            StateFinance {
                fips: at(&head, &p, "fips").to_string(),
                name: at(&head, &p, "state").to_string(),
                systems: n("systems") as usize,
                enrollment: n("enrollment"),
                total_revenue: n("total_revenue"),
                federal_revenue: n("federal_revenue"),
                state_revenue: n("state_revenue"),
                local_revenue: n("local_revenue"),
                property_tax_revenue: n("property_tax_revenue"),
                parent_government_revenue: n("parent_government_revenue"),
                current_spending: n("current_spending"),
            }
        })
        .collect();

    if states.len() < 50 {
        return None;
    }

    let rank_of = |key: &dyn Fn(&StateFinance) -> f64, pool: &[&StateFinance]| -> usize {
        let ohio = pool
            .iter()
            .find(|s| s.name == "Ohio")
            .map_or(0.0, |s| key(s));
        pool.iter().filter(|s| key(s) > ohio).count() + 1
    };

    let all: Vec<&StateFinance> = states.iter().collect();
    let independent: Vec<&StateFinance> =
        states.iter().filter(|s| s.fiscally_independent()).collect();

    let total = |pick: &dyn Fn(&StateFinance) -> f64| states.iter().map(pick).sum::<f64>();
    let revenue = total(&|s| s.total_revenue);

    Some(National {
        fiscal_year: 2022,
        ohio_local_rank: rank_of(&StateFinance::local_share, &all),
        ohio_state_rank: rank_of(&StateFinance::state_share, &all),
        ohio_spending_rank: rank_of(&StateFinance::spending_per_pupil, &all),
        ohio_property_tax_rank: rank_of(
            &|s: &StateFinance| {
                if s.total_revenue > 0.0 {
                    s.property_tax_revenue / s.total_revenue
                } else {
                    0.0
                }
            },
            &independent,
        ),
        independent_states: independent.len(),
        national_local_share: total(&|s| s.local_revenue) / revenue,
        national_state_share: total(&|s| s.state_revenue) / revenue,
        national_spending_per_pupil: total(&|s| s.current_spending) * 1_000.0
            / total(&|s| s.enrollment),
        states,
    })
}

/// The statewide outcome block, or `None` if nothing joined.
fn outcome_statewide(records: &[Joined]) -> Option<OutcomeStatewide> {
    if records.is_empty() {
        return None;
    }
    let index = |r: &Joined| r.outcome.performance_index;
    let growth = |r: &Joined| r.outcome.progress_effect_size;
    let guarantee = |r: &Joined| Some(if r.on_guarantee() { 1.0 } else { 0.0 });
    let enrolled = |r: &Joined| r.outcome.per_enrolled_pupil();
    let weighted = |r: &Joined| r.outcome.per_equivalent_pupil;

    let (poverty_series, index_series, _) =
        aligned(records, |r| r.economically_disadvantaged, index);
    let (guarantee_series, guarantee_index, guarantee_poverty) = aligned(records, guarantee, index);
    let (spend_series, growth_series, spend_poverty) = aligned(records, enrolled, growth);
    let (weighted_series, weighted_index, _) = aligned(records, weighted, index);
    let (enrolled_series, enrolled_index, _) = aligned(records, enrolled, index);

    // Federal money is allocated largely by poverty, so the raw association with attainment is
    // mostly the poverty association wearing a different label. Both are reported.
    let federal = |r: &Joined| {
        let (part, whole) = (
            r.outcome.per_equivalent_pupil_federal?,
            r.outcome.per_equivalent_pupil?,
        );
        (whole > 0.0).then_some(part / whole)
    };
    let (federal_series, federal_index, federal_poverty) = aligned(records, federal, index);
    let shares: Vec<f64> = records.iter().filter_map(federal).collect();

    // The two growth measures the department publishes for the same district and year.
    let one_year = |r: &Joined| r.outcome.progress_effect_size_one_year;
    let (three, one, _) = aligned(records, growth, one_year);

    // Published to two decimals, so a printed 0.00 is anything in (-0.005, 0.005) and has no sign.
    // Excluding those is the difference between reporting 44 and reporting 76.
    const DETERMINATE: f64 = 0.005;
    const MATERIAL: f64 = 0.05;
    let pairs: Vec<(f64, f64)> = three.iter().copied().zip(one.iter().copied()).collect();
    let determinate: Vec<(f64, f64)> = pairs
        .iter()
        .copied()
        .filter(|(a, b)| a.abs() >= DETERMINATE && b.abs() >= DETERMINATE)
        .collect();
    let disagree = determinate
        .iter()
        .filter(|(a, b)| (*a > 0.0) != (*b > 0.0))
        .count();
    let materially = determinate
        .iter()
        .filter(|(a, b)| (*a > 0.0) != (*b > 0.0) && a.abs() >= MATERIAL && b.abs() >= MATERIAL)
        .count();

    let median_index = |on: bool| {
        median(
            records
                .iter()
                .filter(|r| r.on_guarantee() == on)
                .filter_map(index)
                .collect(),
        )
    };

    Some(OutcomeStatewide {
        districts: records.len(),
        poverty_vs_performance: correlation(&poverty_series, &index_series),
        guarantee_vs_performance: correlation(&guarantee_series, &guarantee_index),
        guarantee_vs_performance_controlled: controlling_for(
            &guarantee_series,
            &guarantee_index,
            &guarantee_poverty,
        ),
        spending_vs_growth_controlled: controlling_for(
            &spend_series,
            &growth_series,
            &spend_poverty,
        ),
        weighted_spending_vs_performance: correlation(&weighted_series, &weighted_index),
        enrolled_spending_vs_performance: correlation(&enrolled_series, &enrolled_index),
        median_performance_on_guarantee: median_index(true),
        median_performance_on_formula: median_index(false),
        median_federal_share: median(shares.clone()),
        max_federal_share: shares.iter().copied().fold(0.0, f64::max),
        federal_share_above_tenth: shares.iter().filter(|s| **s > 0.10).count(),
        federal_share_vs_performance: controlling_for(
            &federal_series,
            &federal_index,
            &federal_poverty,
        ),
        federal_share_vs_performance_raw: correlation(&federal_series, &federal_index),
        growth_measures_disagree: disagree,
        growth_measures_determinate: determinate.len(),
        growth_measures_disagree_materially: materially,
        growth_measure_agreement: correlation(&three, &one),
    })
}

/// The 99 House districts, apportioned, in the bundle's own shape.
///
/// `project::house_district` does the arithmetic and the verification; this only restates it for
/// the feed, so that the reconciliation test lives beside the apportionment rather than here.
fn house_district_block(records: &[DistrictRecord]) -> Vec<bundle::HouseDistrict> {
    house_districts(records)
        .into_iter()
        .map(|h| bundle::HouseDistrict {
            number: h.number,
            adm: h.adm,
            realized_aid: h.realized_aid,
            base_cost_state_share: h.base_cost_state_share,
            categorical_funding: h.categorical_funding,
            guarantee: h.guarantee,
            districts_on_guarantee: h.districts_on_guarantee,
            districts_at_minimum_state_share: h.districts_at_minimum_state_share,
            districts_wholly_inside: h.districts_wholly_inside,
            members: h
                .members
                .into_iter()
                .map(|m| HouseDistrictMember {
                    irn: m.irn,
                    name: m.name,
                    share: m.share,
                    share_of_house_district: m.share_of_house_district,
                    adm: m.adm,
                    realized_aid: m.realized_aid,
                    wholly_inside: m.wholly_inside,
                })
                .collect(),
        })
        .collect()
}

fn to_district(
    record: &DistrictRecord,
    profile: Option<&&str>,
    outcome: Option<&Joined>,
    money: Option<&Finances>,
    taxes: Option<&Vec<PropertyTaxYear>>,
    functions: Option<&SpendingByFunction>,
    house_districts: &[HouseDistrictShare],
) -> District {
    let adm = record.base_cost_adm();
    District {
        irn: record.irn.clone(),
        name: record.name.clone(),
        county: record.county.clone(),
        transition: {
            let t = &record.transition;
            bundle::Transition {
                funding_base: t.funding_base,
                open_enrollment_prior: t.open_enrollment_prior,
                open_enrollment_current: t.open_enrollment_current,
                open_enrollment_threshold: t.open_enrollment_threshold,
                open_enrollment_adjustment: t.open_enrollment_adjustment,
                fy21_funding_base: t.fy21_funding_base,
                transition_supplement: t.transition_supplement,
            }
        },
        preschool_special_education: {
            let p = &record.preschool_special_education;
            bundle::PreschoolSpecialEducation {
                adm: p.adm,
                aid: p.aid,
                total: p.total,
                flat_component: p.flat_component(),
                unprorated: p.unprorated(),
            }
        },
        transportation: {
            let t = &record.transportation;
            bundle::Transportation {
                public_riders: t.public_riders,
                nonpublic_riders: t.nonpublic_riders,
                community_riders: t.community_riders,
                weighted_riders: t.weighted_riders,
                per_rider_base: t.per_rider_base(),
                per_mile_base: t.per_mile_base(),
                paid_on_miles: t.paid_on_miles(),
                effective_state_share: record
                    .published_state_share
                    .unwrap_or(0.0)
                    .max(project::panel::TRANSPORT_MINIMUM_STATE_SHARE),
                school_bus: t.school_bus,
                mass_transit: t.mass_transit,
                other: t.other,
                efficiency: t.efficiency,
                density: t.density,
                efficiency_index: t.efficiency_index,
                district_density: t.district_density,
                fy21_base: t.fy21_base,
                guarantee: t.guarantee,
                total: t.total,
                special_education: t.special_education,
                special_education_unprorated: t.special_education_unprorated(),
            }
        },
        supplements: bundle::Supplements {
            stars: record.performance.stars,
            progress: record.performance.progress,
            performance_eligible: record.performance.eligible,
            performance: record.performance.amount,
            base_funding: record.supplements.base_funding,
            enrollment_change: record.supplements.enrollment_change,
            growth_eligible: record.supplements.growth_eligible,
            growth: record.supplements.growth,
            growth_forgone: record.supplements.forgone(record.categorical_enrolled_adm),
        },
        house_districts: house_districts.to_vec(),
        adm,
        current_year_adm: record.current_year_adm,
        base_cost_build_up: build_up(record),
        property_tax: taxes.cloned().unwrap_or_default(),
        spending_by_function: functions.copied(),
        base_cost_per_pupil: record.base_cost_per_pupil,
        aggregate_base_cost: record.aggregate_base_cost,
        base_cost_state_share: record.base_cost_state_share,
        categorical_funding: record.categorical_funding(),
        special_education: SpecialEducation {
            adm: record.special_education.adm,
            aid: record.special_education.aid,
        },
        dpia: Dpia {
            economically_disadvantaged_adm: record.dpia.economically_disadvantaged_adm,
            directly_certified_adm: record.dpia.directly_certified_adm,
            weighted_adm: record.dpia.weighted_adm,
            percentage: record.dpia.percentage,
            index: record.dpia.index,
        },
        targeted_assistance: TargetedAssistance {
            property_valuation: record.targeted_assistance.property_valuation,
            federal_gross_income: record.targeted_assistance.federal_gross_income,
            weighted_wealth: record.targeted_assistance.weighted_wealth,
            capacity_index: record.targeted_assistance.capacity_index,
            capacity_amount: record.targeted_assistance.capacity_amount,
            wealth_per_pupil: record.targeted_assistance.wealth_per_pupil,
            wealth_index: record.targeted_assistance.wealth_index,
            wealth_amount: record.targeted_assistance.wealth_amount,
            resident_adm: record
                .targeted_assistance
                .resident_adm(record.categorical_enrolled_adm),
            supplement_eligible: record.targeted_assistance.supplement_eligible,
        },
        career_technical: CareerTechnical {
            fte: record.career_technical.fte,
            aid: record.career_technical.aid,
            associated_services: record.career_technical.associated_services,
        },
        english_learners: EnglishLearners {
            adm: record.english_learners.adm,
            aid: record.english_learners.aid,
        },
        gifted: Gifted {
            identification: record.gifted.identification,
            referral: record.gifted.referral,
            fte_k8: record.gifted.fte_k8,
            fte_9_12: record.gifted.fte_9_12,
            coordinator_units: record.gifted.coordinator_units,
            coordinator_aid: record.gifted.coordinator_aid,
            specialist_k8_units: record.gifted.specialist_k8_units,
            specialist_k8_aid: record.gifted.specialist_k8_aid,
            specialist_9_12_units: record.gifted.specialist_9_12_units,
            specialist_9_12_aid: record.gifted.specialist_9_12_aid,
            entirely_on_the_floor: record.gifted.entirely_on_the_floor(),
        },
        categorical_adm: record.categorical_enrolled_adm,
        categoricals: Categoricals {
            targeted_assistance: record.categoricals.targeted_assistance,
            special_education: record.categoricals.special_education,
            dpia: record.categoricals.dpia,
            english_learners: record.categoricals.english_learners,
            gifted: record.categoricals.gifted,
            career_technical: record.categoricals.career_technical,
        },
        formula_aid_per_pupil: record.core_foundation_funding / adm,
        realized_aid_per_pupil: record.realized_aid() / adm,
        guarantee: record.guarantee,
        at_minimum_state_share: record.at_minimum_state_share(),
        valuation_per_pupil: record
            .valuation_per_pupil
            .or_else(|| profile.and_then(|line| parse(line, 4))),
        effective_class1_millage: profile.and_then(|line| parse(line, 6)),
        voted_operating_millage: profile.and_then(|line| parse(line, 5)),
        millage: millage_analysis(taxes, profile.and_then(|line| parse(line, 5))),
        regime: regime_counterfactual(record, taxes),
        operating_expenditure_per_pupil: profile.and_then(|line| parse(line, 7)),
        economically_disadvantaged: profile.and_then(|line| parse(line, 3)),
        enrollment_change: {
            let [first, _, last] = record.adm_history;
            (first > 0.0).then(|| last / first - 1.0)
        },
        adm_history: record.adm_history,
        finances: money.map_or_else(Vec::new, |f| {
            f.years
                .iter()
                .map(|year| FinanceYear {
                    fiscal_year: year.fiscal_year.0,
                    state_aid: year.unrestricted_aid,
                    // Property and income tax together: a district with an income tax and one
                    // without raise the local share differently, and a page comparing only
                    // property tax would understate the second.
                    local_tax: year.property_tax + year.income_tax,
                    total_revenue: year.total_revenue,
                    total_expenditure: year.total_expenditure,
                    ending_cash: year.ending_cash,
                })
                .collect()
        }),
        outcome: outcome.map(|joined| DistrictOutcome {
            performance_index: joined.outcome.performance_index,
            performance_index_prior: joined.outcome.performance_index_prior,
            performance_index_earliest: joined.outcome.performance_index_earliest,
            progress_effect_size: joined.outcome.progress_effect_size,
            progress_effect_size_one_year: joined.outcome.progress_effect_size_one_year,
            per_enrolled_pupil: joined.outcome.per_enrolled_pupil(),
            per_equivalent_pupil: joined.outcome.per_equivalent_pupil,
            per_equivalent_pupil_federal: joined.outcome.per_equivalent_pupil_federal,
            per_equivalent_pupil_state_local: joined.outcome.per_equivalent_pupil_state_local,
            economically_disadvantaged: joined.outcome.economically_disadvantaged,
            english_learner: joined.outcome.english_learner,
            students_with_disabilities: joined.outcome.students_with_disabilities,
        }),
    }
}

/// Fiscal years the financial panel covers, oldest first.
fn covered_years(districts: &[District]) -> Vec<u16> {
    let mut years: Vec<u16> = districts
        .iter()
        .flat_map(|d| d.finances.iter().map(|y| y.fiscal_year))
        .collect();
    years.sort_unstable();
    years.dedup();
    years
}

/// Sum the per-district actuals into one statewide series.
fn statewide_finances(districts: &[District]) -> Vec<FinanceYear> {
    covered_years(districts)
        .into_iter()
        .map(|fiscal_year| {
            let mut total = FinanceYear {
                fiscal_year,
                state_aid: 0.0,
                local_tax: 0.0,
                total_revenue: 0.0,
                total_expenditure: 0.0,
                ending_cash: 0.0,
            };
            for year in districts
                .iter()
                .flat_map(|d| d.finances.iter())
                .filter(|y| y.fiscal_year == fiscal_year)
            {
                total.state_aid += year.state_aid;
                total.local_tax += year.local_tax;
                total.total_revenue += year.total_revenue;
                total.total_expenditure += year.total_expenditure;
                total.ending_cash += year.ending_cash;
            }
            total
        })
        .collect()
}

/// Recompute the twenty-two elements behind one district's base cost.
///
/// The rest of this feed quotes the department: `project::panel` reads its published model and
/// passes the figures through. This is the one place that does the arithmetic instead, using the
/// district's own grade-band enrollment and the FY2027 statewide factor set — and it reconciles
/// against the published aggregate rather than replacing it, because a computed figure that
/// quietly disagreed with the department's would be worse than no figure at all.
///
/// The reproduction is proved across the whole panel in
/// `crates/foundation/tests/department_model_fy27.rs`: worst deviation $1.09 on $11.77 billion.
fn build_up(record: &DistrictRecord) -> BaseCostBuildUp {
    let computed = aggregate_base_cost(&record.enrollment, &StatewideFactors::fy2027());
    let (a, b, c, d) = (
        computed.teacher,
        computed.student_support,
        computed.district_leadership,
        computed.building_leadership,
    );
    BaseCostBuildUp {
        classroom_teachers: a.classroom,
        special_teachers: a.special,
        substitutes: a.substitute,
        professional_development: a.professional_development,
        teachers: a.total,
        counselors: b.counselors,
        librarians: b.librarians,
        wellness: b.wellness,
        academic_cocurricular: b.academic_cocurricular,
        safety: b.safety,
        supplies: b.supplies,
        technology: b.technology,
        student_support: b.total,
        superintendent: c.superintendent,
        treasurer: c.treasurer,
        other_administrators: c.other_administrators,
        fiscal_support: c.fiscal_support,
        emis: c.emis,
        leadership_support: c.leadership_support,
        itc: c.itc,
        district_leadership: c.total,
        building_leadership_staff: d.leadership,
        building_support: d.support,
        building_operation: d.operation,
        building_leadership: d.total,
        athletic_cocurricular: computed.athletic_cocurricular,
        funded_classroom_teachers: a.funded_classroom_teachers,
        funded_special_teachers: a.funded_special_teachers,
        computed_aggregate: computed.aggregate,
        published_aggregate: record.aggregate_base_cost,
        residual: computed.aggregate - record.aggregate_base_cost,
    }
}

/// Column positions, resolved from the header so a reordered fixture fails loudly.
fn header(csv: &str) -> Vec<&str> {
    csv.lines().next().unwrap_or_default().split(',').collect()
}

fn at<'a>(head: &[&str], parts: &[&'a str], name: &str) -> &'a str {
    let index = head
        .iter()
        .position(|c| *c == name)
        .unwrap_or_else(|| panic!("{name} is not a column of the fixture"));
    parts.get(index).copied().unwrap_or_default()
}

fn number(head: &[&str], parts: &[&str], name: &str) -> f64 {
    at(head, parts, name).trim().parse().unwrap_or(0.0)
}

/// Both tax years of SD-1 for every district, keyed by IRN and ordered oldest first.
///
/// Ordered because the page reads it as a change rather than as two independent years, and a
/// reversed pair would silently invert every direction it reports.
fn property_taxes() -> HashMap<String, Vec<PropertyTaxYear>> {
    let head = header(SD1);
    let mut out: HashMap<String, Vec<PropertyTaxYear>> = HashMap::new();
    for line in SD1.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split(',').collect();
        let n = |name: &str| number(&head, &p, name);
        out.entry(at(&head, &p, "irn").to_string())
            .or_default()
            .push(PropertyTaxYear {
                tax_year: n("tax_year") as u16,
                class1_value: n("class1_value"),
                class2_value: n("class2_value"),
                public_utility_value: n("public_utility_value"),
                total_value: n("total_value"),
                agricultural_value: n("agricultural_value"),
                residential_value: n("residential_value"),
                commercial_value: n("commercial_value"),
                industrial_value: n("industrial_value"),
                mineral_value: n("mineral_value"),
                railroad_value: n("railroad_value"),
                class1_rate: n("class1_rate"),
                class2_rate: n("class2_rate"),
                class1_taxes_charged: n("class1_taxes_charged"),
                class2_taxes_charged: n("class2_taxes_charged"),
                real_property_taxes_charged: n("real_property_taxes_charged"),
                public_utility_taxes_charged: n("public_utility_taxes_charged"),
                value_per_pupil: n("value_per_pupil"),
                adm: n("adm"),
            });
    }
    for years in out.values_mut() {
        years.sort_by_key(|y| y.tax_year);
    }
    out
}

/// Run H.B. 920 against a district, instead of describing it.
///
/// The [`millage`] crate has been in the workspace since the corpus first asserted that the
/// twenty-mill floor is a regime switch rather than a threshold. Nothing called it. This does.
///
/// # Why the prediction runs from last year's effective rate, not from voted millage
///
/// [`millage::effective_millage`] takes the rate *before* this round of reduction. Across a
/// levy's whole life that is the voted rate against the valuation when it passed — but each
/// district's levies passed in different years against different bases, and Table SD-1 does not
/// publish carryover valuation. Year over year the recursion is exact and needs neither: the
/// prior effective rate already embeds every reduction before it, so scaling it by the change in
/// Class I value gives what the factors alone would produce this year.
///
/// The gap between that and the observed rate is the point of the exercise. Reduction factors
/// reach neither new construction nor newly voted millage, so the residual is what they did not
/// touch — and its sign says which.
fn millage_analysis(
    years: Option<&Vec<PropertyTaxYear>>,
    voted: Option<f64>,
) -> Option<MillageAnalysis> {
    let years = years?;
    let (before, after) = (years.first()?, years.last()?);
    if years.len() < 2 || before.class1_value <= 0.0 || after.class1_value <= 0.0 {
        return None;
    }

    let result = millage::effective_millage(
        before.class1_rate,
        before.class1_value,
        after.class1_value,
        AgencyType::City,
    )
    .ok()?;

    Some(MillageAnalysis {
        tax_year: after.tax_year,
        prior_rate: before.class1_rate,
        observed_rate: after.class1_rate,
        predicted_rate: result.effective,
        residual: after.class1_rate - result.effective,
        at_floor: result.status.valuation_growth_reaches_revenue(),
        // Against the voted rate the profile publishes, which is a TY2023 figure — so this is
        // the reduction as of that year, not as of `after`. Stated rather than interpolated.
        cumulative_reduction: voted
            .filter(|v| *v > 0.0)
            .map(|v| 1.0 - (before.class1_rate / v)),
        // One mill against the real property base, per pupil: the local half of the formula
        // reduced to the number that makes its inequality legible.
        //
        // Over Table SD-1's own ADM, recovered from the two figures it publishes, rather than
        // over the formula's base cost ADM. The card puts this beside `value_per_pupil` from the
        // same row, and two per-pupil figures side by side have to share a denominator or the
        // reader is silently comparing a tax-year headcount to a funding-year weighted one.
        yield_per_mill_per_pupil: if after.value_per_pupil > 0.0 && after.total_value > 0.0 {
            let adm = after.total_value / after.value_per_pupil;
            millage::yield_of(1.0, after.class1_value + after.class2_value) / adm
        } else {
            0.0
        },
    })
}

/// The mechanism the Fair School Funding Plan replaced, run at current inputs.
///
/// [`regime_diff::at_fy2027`] holds the plan's own base cost fixed and substitutes the local
/// share: the charge-off's flat statutory millage against valuation, in place of the local
/// capacity measure. Everything but `mills_short_of_charge_off` comes straight from the crate.
///
/// # Which valuation per pupil, and why it has to be the department of education's
///
/// The charge-off was subtracted from a district's computed cost, and that cost is expressed per
/// pupil on the funding formula's enrolled ADM. Table SD-1 publishes the *same* taxable valuation
/// over a different pupil count — its own, which includes children the district does not teach —
/// and the two differ by a factor of 2.2 in Youngstown. Using SD-1's figure here would subtract a
/// local share computed on one denominator from a cost computed on another, and would change the
/// answer for hundreds of districts. So the counterfactual runs on the profile report's basis and
/// only the phantom-revenue comparison, which is rate against rate, touches SD-1.
fn regime_counterfactual(
    record: &DistrictRecord,
    taxes: Option<&Vec<PropertyTaxYear>>,
) -> Option<RegimeCounterfactual> {
    let diff = regime_diff::at_fy2027(record, regime_diff::TERMINAL_MILLS);
    let component = diff.components.first()?;

    // `regime-diff` recovers local capacity by subtraction and censors it where the minimum state
    // share binds — 138 districts. The department publishes the figure for all of them, so the
    // published one is preferred and the recovered one is the fallback. They agree to 0.46% at
    // worst across the 471 where both exist; see `crates/project/tests/finances_and_the_guarantee.rs`.
    let capacity = record.published_capacity_per_pupil.or(component.successor);

    // Rate against rate, so no denominator is involved: what the district's own effective Class I
    // rate is, against the uniform rate the charge-off would deem it able to levy.
    let mills_short = taxes
        .and_then(|years| years.last())
        .map(|y| regime_diff::TERMINAL_MILLS - y.class1_rate)
        .filter(|short| *short > 0.0005);

    Some(RegimeCounterfactual {
        charge_off_mills: regime_diff::TERMINAL_MILLS,
        charge_off_local_share: component.predecessor,
        local_capacity: capacity,
        aid_charge_off: diff.predecessor_total,
        aid_fsfp: diff.successor_total,
        difference: diff.total_difference(),
        residual: diff.residual(),
        exceeds_base_cost: component
            .predecessor
            .is_some_and(|local| local > record.base_cost_per_pupil),
        mills_short_of_charge_off: mills_short,
    })
}

/// FY2025 operating spending by function, per pupil, keyed by IRN.
fn spending_by_function() -> HashMap<String, SpendingByFunction> {
    let head = header(FUNCTIONS);
    FUNCTIONS
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let p: Vec<&str> = line.split(',').collect();
            let n = |name: &str| number(&head, &p, name);
            (
                at(&head, &p, "irn").to_string(),
                SpendingByFunction {
                    adm: n("unweighted_adm_fy25"),
                    operating_per_pupil: n("operating_expenditure_per_pupil_fy25"),
                    classroom_instruction: n("classroom_instruction_per_pupil"),
                    nonclassroom: n("nonclassroom_per_pupil"),
                    instruction: n("instruction_per_pupil"),
                    pupil_support: n("pupil_support_per_pupil"),
                    instructional_staff_support: n("instructional_staff_support_per_pupil"),
                    general_admin: n("general_admin_per_pupil"),
                    school_admin: n("school_admin_per_pupil"),
                    operations_maintenance: n("operations_maintenance_per_pupil"),
                    pupil_transportation: n("pupil_transportation_per_pupil"),
                    other_support: n("other_support_per_pupil"),
                    food_service: n("food_service_per_pupil"),
                },
            )
        })
        .collect()
}

fn main() {
    // Profile columns: 3 economically disadvantaged, 4 valuation/pupil, 6 effective class 1
    // millage, 7 operating expenditure per pupil.
    let profile: HashMap<&str, &str> = PROFILE
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| field(line, 0).map(|irn| (irn, line)))
        .collect();

    let records = panel();
    let outcomes = joined();
    let money = finances();
    let cpi = deflate::CpiSeries::cpi_u_june();
    let taxes = property_taxes();
    let functions = spending_by_function();

    // Which House districts each district lies in, keyed by IRN. Built once: the crosswalk is a
    // 1,085-row fixture and reading it per district would parse it 609 times.
    let mut shares: HashMap<String, Vec<HouseDistrictShare>> = HashMap::new();
    for overlap in overlaps() {
        shares
            .entry(overlap.irn.clone())
            .or_default()
            .push(HouseDistrictShare {
                number: overlap.house_district.clone(),
                share: overlap.share,
            });
    }
    for list in shares.values_mut() {
        list.sort_by(|a, b| {
            b.share
                .partial_cmp(&a.share)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    let districts: Vec<District> = records
        .iter()
        .map(|record| {
            to_district(
                record,
                profile.get(record.irn.as_str()),
                outcomes.iter().find(|j| j.funding.irn == record.irn),
                for_district(&money, &record.irn),
                taxes.get(&record.irn),
                functions.get(&record.irn),
                shares.get(&record.irn).map_or(&[][..], Vec::as_slice),
            )
        })
        .collect();

    let paired: Vec<(f64, f64, f64)> = districts
        .iter()
        .filter_map(|d| {
            Some((
                d.valuation_per_pupil?,
                d.formula_aid_per_pupil,
                d.realized_aid_per_pupil,
            ))
        })
        .collect();
    let wealth: Vec<f64> = paired.iter().map(|t| t.0).collect();
    let formula: Vec<f64> = paired.iter().map(|t| t.1).collect();
    let realized: Vec<f64> = paired.iter().map(|t| t.2).collect();

    // Districts with no SD-1 row have no yield to report; a zero would drag the minimum to it.
    let yields: Vec<f64> = districts
        .iter()
        .filter_map(|d| d.millage.map(|m| m.yield_per_mill_per_pupil))
        .filter(|y| *y > 0.0)
        .collect();

    let statewide = Statewide {
        districts: districts.len(),
        on_guarantee: districts.iter().filter(|d| d.on_guarantee()).count(),
        at_millage_floor: districts.iter().filter(|d| d.at_millage_floor()).count(),
        near_millage_floor: districts.iter().filter(|d| d.near_millage_floor()).count(),
        median_voted_millage: median(
            districts
                .iter()
                .filter_map(|d| d.voted_operating_millage)
                .collect(),
        ),
        median_effective_millage: median(
            districts
                .iter()
                .filter_map(|d| d.effective_class1_millage)
                .collect(),
        ),
        median_millage_reduction: median(
            districts
                .iter()
                .filter_map(|d| d.millage.and_then(|m| m.cumulative_reduction))
                .collect(),
        ),
        median_yield_per_mill: median(yields.clone()),
        min_yield_per_mill: yields.iter().copied().fold(f64::INFINITY, f64::min),
        max_yield_per_mill: yields.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        // On Table SD-1's own denominator, so a district's SD-1 figure is positioned against a
        // median computed the same way. The profile report's median is a different quantity.
        median_sd1_value_per_pupil: median(
            districts
                .iter()
                .filter_map(|d| d.property_tax.last().map(|y| y.value_per_pupil))
                .filter(|v| *v > 0.0)
                .collect(),
        ),
        below_charge_off_rate: districts
            .iter()
            .filter(|d| {
                d.regime
                    .is_some_and(|r| r.mills_short_of_charge_off.is_some())
            })
            .count(),
        charge_off_exceeds_base_cost: districts
            .iter()
            .filter(|d| d.regime.is_some_and(|r| r.exceeds_base_cost))
            .count(),
        median_regime_difference: median(
            districts
                .iter()
                .filter_map(|d| d.regime.and_then(|r| r.difference))
                .collect(),
        ),
        districts_without_targeted_assistance: districts
            .iter()
            .filter(|d| d.categoricals.targeted_assistance <= 0.0)
            .count(),
        at_minimum_state_share: districts
            .iter()
            .filter(|d| d.at_minimum_state_share)
            .count(),
        median_valuation_per_pupil: median(wealth.clone()),
        median_operating_expenditure_per_pupil: median(
            districts
                .iter()
                .filter_map(|d| d.operating_expenditure_per_pupil)
                .collect(),
        ),
        wealth_neutrality_formula: wealth_neutrality(&wealth, &formula)
            .map_or(f64::NAN, |w| w.correlation),
        wealth_neutrality_realized: wealth_neutrality(&wealth, &realized)
            .map_or(f64::NAN, |w| w.correlation),
        guarantee_total: districts.iter().map(|d| d.guarantee).sum(),
        realized_aid_total: records.iter().map(DistrictRecord::realized_aid).sum(),
        minimum_state_share: MINIMUM_STATE_SHARE,
        // Summed over the districts in the feed, not over the 660-body panel behind it. The
        // page cannot then disagree with the feed about which districts are in the total, and
        // the population matches every other statewide figure here.
        finances: statewide_finances(&districts),
        outcomes: outcome_statewide(&outcomes),
    };

    let policies = checkpoint_policies();
    let checkpoints: Vec<Checkpoint> = policies
        .iter()
        .map(|(label, policy, shape)| {
            let effect = simulate(&records, policy);
            // The same comparison a second time, through `scenario-delta`, which counts what
            // happens to the *guarantee population* — held throughout, lifted off, pushed on —
            // and which `simulate` does not. A test asserts the two agree on the counts they
            // share, so two implementations of one comparison cannot drift apart unnoticed.
            let reach = ScenarioDelta::between(&records, &Policy::current_law(), policy)
                .total()
                .reach;
            Checkpoint {
                label: (*label).to_string(),
                policy: *shape,
                cost: effect.cost(),
                realized_aid: effect.policy.realized_aid,
                gainers: effect.gainers(),
                losers: effect.losers(),
                unmoved: effect.unmoved(),
                held_throughout: reach.held_throughout,
                lifted_off: reach.lifted_off,
                pushed_on: reach.pushed_on,
                on_guarantee: effect.policy.on_guarantee,
            }
        })
        .collect();

    // One method for the whole feed. A page that let the reader pick between damped and undamped
    // would be offering a choice whose consequences it has no basis to explain — three
    // observations per district cannot say which is right.
    let method = Method::Damped {
        rate: 0.0,
        damping: DEFAULT_DAMPING,
    };
    let prior = enrollment_growth_prior(&records, ONE_SIGMA);
    let projection = Projection {
        base_year: HISTORY_YEARS[HISTORY_YEARS.len() - 1].0,
        horizon: HORIZON.0,
        method: method.label().to_string(),
        damping: DEFAULT_DAMPING,
        sigma: prior.sigma,
        z: prior.z,
        prior_source: prior.source.to_string(),
        checkpoints: forecast_years()
            .into_iter()
            .map(|(label, index, year)| {
                let (_, policy, shape) = &policies[index];
                let effect = forecast(&records, policy, year, method, prior);
                ForecastCheckpoint {
                    label: label.to_string(),
                    policy: *shape,
                    fiscal_year: year.0,
                    realized_aid: effect.realized_aid,
                    low: effect.low,
                    high: effect.high,
                    adm: effect.adm,
                    on_guarantee: effect.on_guarantee,
                }
            })
            .collect(),
    };

    let bundle = Bundle {
        national: national(),
        house_districts: house_district_block(&records),
        contract_version: CONTRACT_VERSION.to_string(),
        provenance: "Ohio DEW FY27 TRAD State Foundation Funding Calculator (a projection, not \
                     an actual) joined with the FY2024 District Profile Report. Base cost, \
                     guarantee, and formula funding are FY2027; enrolled ADM is FY2024-FY2026, \
                     of which FY2026 is partly departmental estimate; millage is TY2023; \
                     expenditure and demographics are FY2024. Achievement, growth, need, and \
                     FY2025 spending are the 2024-25 Ohio School Report Card, joined on IRN \
                     across the 606 districts every panel covers. See .yidam/catalog/."
            .to_string(),
        fiscal_year: MODEL_YEAR.0,
        statewide,
        checkpoints,
        projection: Some(projection),
        deflator: Some(Deflator {
            label: cpi.label().to_string(),
            points: covered_years(&districts)
                .into_iter()
                .filter_map(|year| cpi.point(FiscalYear(year)).map(|point| (year, point.index)))
                .collect(),
        }),
        districts,
    };
    print!("{}", bundle.to_json());
}
