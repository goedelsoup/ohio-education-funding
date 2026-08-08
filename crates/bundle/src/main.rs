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
    BaseCostBuildUp, Bundle, Checkpoint, Deflator, District, DistrictOutcome, FinanceYear,
    ForecastCheckpoint, OutcomeStatewide, PolicyShape, Projection, Statewide, CONTRACT_VERSION,
};
use dispersion::{partial_correlation, wealth_neutrality};
use edfund_core::FiscalYear;
use foundation::{aggregate_base_cost, StatewideFactors};
use project::finances::{finances, for_district, Finances};
use project::outcomes::{joined, Joined};
use project::panel::{panel, DistrictRecord, HISTORY_YEARS, MINIMUM_STATE_SHARE, MODEL_YEAR};
use project::policy::{GuaranteeRule, Policy};
use project::report::{enrollment_growth_prior, forecast, simulate};
use project::series::{Method, DEFAULT_DAMPING, ONE_SIGMA};

/// The furthest year the page will offer, ten past the last observation.
///
/// Not further: the projection is damped precisely because an undamped trend produces a
/// confident and absurd number at a long horizon, and offering FY2050 would invite one anyway.
const HORIZON: FiscalYear = FiscalYear(2036);

/// The FY2024 District Profile Report: millage, expenditure, demographics.
const PROFILE: &str = include_str!("../../dispersion/fixtures/cupp-fy24-district-data.csv");

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
    })
}

fn to_district(
    record: &DistrictRecord,
    profile: Option<&&str>,
    outcome: Option<&Joined>,
    money: Option<&Finances>,
) -> District {
    let adm = record.base_cost_adm();
    District {
        irn: record.irn.clone(),
        name: record.name.clone(),
        adm,
        current_year_adm: record.current_year_adm,
        base_cost_build_up: build_up(record),
        base_cost_per_pupil: record.base_cost_per_pupil,
        aggregate_base_cost: record.aggregate_base_cost,
        base_cost_state_share: record.base_cost_state_share,
        categorical_funding: record.categorical_funding(),
        formula_aid_per_pupil: record.core_foundation_funding / adm,
        realized_aid_per_pupil: record.realized_aid() / adm,
        guarantee: record.guarantee,
        at_minimum_state_share: record.at_minimum_state_share(),
        valuation_per_pupil: record
            .valuation_per_pupil
            .or_else(|| profile.and_then(|line| parse(line, 4))),
        effective_class1_millage: profile.and_then(|line| parse(line, 6)),
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
            per_enrolled_pupil: joined.outcome.per_enrolled_pupil(),
            per_equivalent_pupil: joined.outcome.per_equivalent_pupil,
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
    let districts: Vec<District> = records
        .iter()
        .map(|record| {
            to_district(
                record,
                profile.get(record.irn.as_str()),
                outcomes.iter().find(|j| j.funding.irn == record.irn),
                for_district(&money, &record.irn),
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

    let statewide = Statewide {
        districts: districts.len(),
        on_guarantee: districts.iter().filter(|d| d.on_guarantee()).count(),
        at_millage_floor: districts.iter().filter(|d| d.at_millage_floor()).count(),
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
            Checkpoint {
                label: (*label).to_string(),
                policy: *shape,
                cost: effect.cost(),
                realized_aid: effect.policy.realized_aid,
                gainers: effect.gainers(),
                losers: effect.losers(),
                unmoved: effect.unmoved(),
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
