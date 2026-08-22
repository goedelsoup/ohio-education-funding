//! Build the web feed from the district panel and the profile-report fixture.
//!
//! ```text
//! cargo run -p bundle > ../web/public/data/bundle.json
//! ```
//!
//! It takes [`project::panel()`] as the source of the funding model, joins the District Profile
//! Report for millage, expenditure, and demographics, computes the statewide aggregates, and
//! runs a fixed set of policies to produce [`crate::Checkpoint`]s. All the logic worth testing
//! lives in the calculator crates.
//!
//! # Why this is a module and not a `main.rs`
//!
//! It was one. `fn main` was three lines; the other 1,800 were derivation that nothing outside
//! the binary could call. So when `connect` wanted the district count for the repository index,
//! it could not ask — it counted occurrences of a field name in the committed `bundle.json`
//! instead.
//!
//! That discriminator broke twice. `"irn": ` stopped being unique when House districts arrived
//! carrying an IRN of their own, and the count went from 609 to 1,694 with nothing else changed.
//! `label` stopped when the deflator acquired one. Neither failure could announce itself: a
//! wrong count renders as a number, not as an error.
//!
//! [`build`] is an ordinary function now, and `main.rs` is the three lines it always was.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{
    AppropriationLine, AppropriationYear, BaseCostBuildUp, Bundle, CareerTechnical, CasinoYear,
    Categoricals, Checkpoint, Deflator, District, DistrictOutcome, Dpia, Draft, DraftProvision,
    EnglishLearners, FinanceYear, ForecastCheckpoint, Gifted, HistoryYear, HouseDistrictMember,
    HouseDistrictShare, MealProgramYear, MillageAnalysis, National, OutcomeStatewide, PolicyShape,
    Projection, PropertyTaxYear, RegimeCounterfactual, SeriesYear, SpecialEducation,
    SpendingByFunction, StateFinance, Statewide, TargetedAssistance, YearKind, CONTRACT_VERSION,
};
use dispersion::mr81::poverty_share_by_year;
use dispersion::ohio_panel::{equalization_by_year, revenue_mix_by_year};
use dispersion::{partial_correlation, wealth_neutrality};
use edfund_core::{AgencyType, FiscalYear};
use foundation::{aggregate_base_cost, StatewideFactors};
use project::appropriations;
use project::drafts::Lever;
use project::finances::{finances, for_district, Finances};
use project::legislative_district::{legislative_districts, overlaps, Chamber};
use project::line_origins;
use project::outcomes::{joined, Joined};
use project::panel::{panel, DistrictRecord, HISTORY_YEARS, MINIMUM_STATE_SHARE, MODEL_YEAR};
use project::policy::{GuaranteeRule, Policy};
use project::report::{enrollment_growth_prior, forecast, simulate};
use project::series::{Method, DEFAULT_DAMPING, ONE_SIGMA};
use project::session_laws;
use scenario_delta::ScenarioDelta;

/// The furthest year the page will offer, ten past the last observation.
///
/// Not further: the projection is damped precisely because an undamped trend produces a
/// confident and absurd number at a long horizon, and offering FY2050 would invite one anyway.
const HORIZON: FiscalYear = FiscalYear(2036);

/// The FY2024 District Profile Report: millage, expenditure, demographics.
const PROFILE: &str = include_str!("../../dispersion/fixtures/cupp-fy24-district-data.csv");

/// The fiscal year [`PROFILE`] describes.
const PROFILE_YEAR: u16 = 2024;

/// The school year the report card describes, as its publisher writes it.
///
/// A constant and not a derivation, because a school year is the one reckoning with no number to
/// read off a row: the extract carries `performance_index`, not `2024-25`. It is named here, once,
/// rather than typed into an Astro `<meta>` description, which is where it lived and where
/// nothing checked it.
///
/// It is not merely a constant, though. `report_card_fixture_and_year_agree` holds it against the
/// fixture *path* — `report-card-2425-district-data.csv` — so swapping in next year's download
/// without moving the label fails the build. A named constant nothing checks is the same hazard
/// as a literal; the check is what makes the difference.
const REPORT_CARD_YEAR: &str = "2024-25";

/// The report card extract the label above describes, named so the two can be checked against
/// each other. The file itself is read by `project::outcomes`.
///
/// Test-only: it exists to be compared with [`REPORT_CARD_YEAR`], not to be read.
#[cfg(test)]
const REPORT_CARD_FIXTURE: &str = "report-card-2425-district-data.csv";

/// The fiscal year the report card's *spending* columns are on.
///
/// One year, two reckonings. The report card publishes attainment for the 2024-25 school year and
/// operating expenditure for FY2025 in the same download, and a card showing both under one label
/// would be picking one and being wrong about the other half of its own figures.
const REPORT_CARD_SPENDING_YEAR: u16 = 2025;

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

/// The median of an unsorted series, zero where it is empty.
///
/// Sorts, then defers to [`dispersion::median`] so the feed and the equity statistics share one
/// definition. This used to take the upper of the two middle observations, which disagrees with
/// `dispersion` on every even-length series — and two of the panels here are even.
fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    dispersion::median(&values).unwrap_or(0.0)
}

/// Every draft, flattened for the feed.
///
/// The unpriced provisions are exported alongside the priced ones rather than filtered out. A feed
/// carrying only the levers would let the site show a statewide total for two of a bill's five
/// clauses with nothing on the page saying so, which is the failure `project::drafts::Priced`
/// exists to make impossible one layer down.
fn draft_export() -> Vec<Draft> {
    project::drafts::drafts()
        .into_values()
        .map(|draft| Draft {
            slug: draft.slug,
            provisions: draft
                .provisions
                .into_iter()
                .map(|p| DraftProvision {
                    ordinal: p.ordinal,
                    // The key rather than the parsed variant: the query string the scenario page
                    // reads is keyed on these five strings, and `Lever::key` is where they are
                    // written down so this is not a second place they could drift from.
                    lever: p.lever.map(Lever::key).unwrap_or_default().to_string(),
                    title: p.title,
                    authority: p.authority,
                    parameter: p.parameter,
                    proposed: p.proposed,
                    note: p.note,
                })
                .collect(),
        })
        .collect()
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

/// A share the source publishes as 0 to 100, as the fraction this bundle publishes.
///
/// Named rather than written as `/ 100.0` at three call sites, so that the next passthrough field
/// arriving on a percentage scale has something to reach for and `share_fields_are_fractions` has
/// something to point at when it fails.
fn percent_to_fraction(v: f64) -> f64 {
    v / 100.0
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
/// `project::legislative_district` does the arithmetic and the verification; this only restates it for
/// the feed, so that the reconciliation test lives beside the apportionment rather than here.
fn house_district_block(records: &[DistrictRecord], chamber: Chamber) -> Vec<crate::HouseDistrict> {
    legislative_districts(records, chamber)
        .into_iter()
        .map(|h| crate::HouseDistrict {
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

/// Everything joined onto one district, named rather than positional.
///
/// Seven optional references in a row is the mistake the `Fy27Sheets` struct was introduced to
/// prevent on the connector side, arriving here from the other direction: most of these are
/// `Option<&…>` of different types, so a transposition is a compile error more often than not —
/// but `taxes` and `functions` are both `Option<&…>` over collections and would swap silently.
struct Joins<'a> {
    profile: Option<&'a &'a str>,
    outcome: Option<&'a Joined>,
    money: Option<&'a Finances>,
    taxes: Option<&'a Vec<PropertyTaxYear>>,
    functions: Option<&'a SpendingByFunction>,
    house_districts: &'a [HouseDistrictShare],
    national: Option<&'a dispersion::national_peers::NationalPosition>,
    /// Recognized valuation for every district at TY2024, keyed by IRN.
    ///
    /// Shared across the whole panel rather than looked up per district, because it is parsed once
    /// from the committed abstract and every row needs it.
    recognized: &'a HashMap<String, regime_diff::Recognition>,
    /// The casino county student fund by fiscal year, and the county funds the district was last
    /// paid out of. Absent for a district the Department of Taxation's sheets do not name.
    casino: Option<&'a (Vec<CasinoYear>, Option<usize>)>,
}

fn to_district(record: &DistrictRecord, joins: &Joins<'_>) -> District {
    let Joins {
        profile,
        outcome,
        money,
        taxes,
        functions,
        house_districts,
        national,
        recognized,
        casino,
    } = *joins;
    let adm = record.base_cost_adm();
    District {
        irn: record.irn.clone(),
        name: record.name.clone(),
        county: record.county.clone(),
        national: national.map(|n| crate::NationalPosition {
            local_share: n.local_share,
            local_share_percentile: n.local_share_percentile,
            revenue_per_pupil: n.revenue_per_pupil,
            revenue_per_pupil_percentile: n.revenue_per_pupil_percentile,
            spending_per_pupil: n.spending_per_pupil,
            spending_per_pupil_percentile: n.spending_per_pupil_percentile,
        }),
        transition: {
            let t = &record.transition;
            crate::Transition {
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
            crate::PreschoolSpecialEducation {
                adm: p.adm,
                aid: p.aid,
                total: p.total,
                flat_component: p.flat_component(),
                unprorated: p.unprorated(),
            }
        },
        transportation: {
            let t = &record.transportation;
            crate::Transportation {
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
        supplements: crate::Supplements {
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
        base_cost_denominated_categoricals: record.base_cost_denominated_categoricals(),
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
        regime: regime_counterfactual(record, taxes, recognized),
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
        casino: casino.map_or_else(Vec::new, |(years, _)| years.clone()),
        casino_counties: casino.and_then(|(_, counties)| *counties),
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
            // The report card publishes these three as 0 to 100 and every other share this
            // bundle carries is a fraction. Converted here, at the one seam they cross, rather
            // than left for each consumer to remember — see `CONTRACT_VERSION` for what that
            // cost when it was not.
            economically_disadvantaged: joined
                .outcome
                .economically_disadvantaged
                .map(percent_to_fraction),
            english_learner: joined.outcome.english_learner.map(percent_to_fraction),
            students_with_disabilities: joined
                .outcome
                .students_with_disabilities
                .map(percent_to_fraction),
        }),
    }
}

/// The Census survey, year by year, as the historical view needs it.
///
/// Two measures joined on the year, both already computed and tested in
/// [`dispersion::ohio_panel`]: where a year's money came from, and how much of the gap between
/// the poorest and richest quartiles of districts each level of government closed.
///
/// The panel is the only thing in this feed that reaches before FY2020, and it has been sitting
/// in the workspace unexported — computed over by two Rust modules and a test, and invisible to
/// every reader who was not running `cargo`. That is the gap this closes.
///
/// FY2014 is absent because it is absent from the Bureau's archive under every naming the other
/// years use, so the series has a hole in it rather than an interpolation across it.
/// What year each block of this feed is measured in, read off the blocks themselves.
///
/// # Why nothing here is typed
///
/// Because the prose version was wrong. `Bundle::provenance` is a hand-written paragraph naming
/// every year in the feed, and it said **"millage is TY2023"** while all 609 districts carried
/// `tax_year: 2024`. Nothing could have caught it: a sentence and a column cannot disagree in a
/// way a compiler or a test notices, and the sentence is the half a reader sees.
///
/// So every label below is derived — from `MODEL_YEAR`, from the maximum tax year actually
/// present, from the ends of the series. A year that moves in the data moves here, and the
/// stale-label failure is structurally unavailable rather than merely unlikely.
///
/// The report card is the exception that proves the rule, and it is handled at
/// [`REPORT_CARD_YEAR`]: it is a school year, so there is no number in the extract to read it
/// off. It is a named constant with a test against the fixture rather than a string typed into a
/// page.
fn series_years(
    districts: &[District],
    history: &[HistoryYear],
    appropriations: &[AppropriationYear],
    meal_program: &[MealProgramYear],
    casino: &[CasinoYear],
) -> Vec<SeriesYear> {
    let mut out = vec![
        SeriesYear {
            series: "formula".into(),
            kind: YearKind::Fiscal,
            label: format!("FY{}", MODEL_YEAR.0),
            source: "DEW FY27 TRAD State Foundation Funding Calculator".into(),
        },
        SeriesYear {
            series: "outcome.performance".into(),
            kind: YearKind::School,
            label: REPORT_CARD_YEAR.into(),
            source: "Ohio School Report Card".into(),
        },
        SeriesYear {
            series: "outcome.spending".into(),
            kind: YearKind::Fiscal,
            label: format!("FY{REPORT_CARD_SPENDING_YEAR}"),
            source: "Ohio School Report Card, expenditure download".into(),
        },
        SeriesYear {
            series: "profile".into(),
            kind: YearKind::Fiscal,
            label: format!("FY{PROFILE_YEAR}"),
            source: "DEW District Profile Report".into(),
        },
    ];

    // The maximum rather than a constant, and the maximum of what is *in the feed* rather than of
    // what the source file holds. This is the field the provenance paragraph got wrong.
    if let Some(year) = districts
        .iter()
        .filter_map(|d| d.millage.as_ref())
        .map(|m| m.tax_year)
        .max()
    {
        out.push(SeriesYear {
            series: "millage".into(),
            kind: YearKind::Tax,
            label: year.to_string(),
            source: "Department of Taxation, Table SD-1".into(),
        });
    }

    if let Some((first, last)) = span(
        districts
            .iter()
            .flat_map(|d| d.property_tax.iter().map(|p| p.tax_year)),
    ) {
        out.push(SeriesYear {
            series: "property_tax".into(),
            kind: YearKind::Tax,
            label: label_span(first, last, ""),
            source: "Department of Taxation, Table SD-1".into(),
        });
    }

    if let Some((first, last)) = span(
        districts
            .iter()
            .flat_map(|d| d.finances.iter().map(|f| f.fiscal_year)),
    ) {
        out.push(SeriesYear {
            series: "finances".into(),
            kind: YearKind::Fiscal,
            label: label_span(first, last, "FY"),
            source: "District five-year forecast filings, R.C. 5705.391".into(),
        });
    }

    if let Some((first, last)) = span(history.iter().map(|h| h.fiscal_year)) {
        out.push(SeriesYear {
            series: "history".into(),
            kind: YearKind::Fiscal,
            label: label_span(first, last, "FY"),
            source: "Census Bureau, Annual Survey of School System Finances".into(),
        });
        // The cross-state comparison is one year of the same survey, and it is the *last* one.
        // Separate from `history` because a card placing Ohio among the states is showing that
        // year alone, and a chip reading FY2009-FY2022 there would be describing the wrong thing.
        out.push(SeriesYear {
            series: "national".into(),
            kind: YearKind::Fiscal,
            label: format!("FY{last}"),
            source: "Census Bureau, Annual Survey of School System Finances".into(),
        });
    }

    // The enrollment counts the formula runs on, which span three years inside a single FY2027
    // model — and the last of them is partly departmental estimate rather than observation.
    if let Some((first, last)) = span(districts.iter().flat_map(|d| {
        d.adm_history
            .iter()
            .enumerate()
            .map(|(i, _)| MODEL_YEAR.0 - 3 + u16::try_from(i).unwrap_or(0))
    })) {
        out.push(SeriesYear {
            series: "enrollment".into(),
            kind: YearKind::Fiscal,
            label: label_span(first, last, "FY"),
            source: "DEW calculator, ADM Data sheet; the last year is partly estimate".into(),
        });
    }

    if let Some((first, last)) = span(appropriations.iter().map(|a| a.fiscal_year)) {
        out.push(SeriesYear {
            series: "appropriations".into(),
            kind: YearKind::Fiscal,
            label: label_span(first, last, "FY"),
            source: "Legislative Service Commission, enacted appropriations".into(),
        });
    }

    if let Some((first, last)) = span(meal_program.iter().map(|m| m.fiscal_year)) {
        out.push(SeriesYear {
            series: "meal_program".into(),
            kind: YearKind::Fiscal,
            label: label_span(first, last, "FY"),
            source: "Office for Child Nutrition, MR-81".into(),
        });
    }

    // Read off the statewide block rather than off the districts, because the two do not span the
    // same years for every district — a district that took nothing in a year has no row for it —
    // and the chip has to describe the series the card is showing.
    if let Some((first, last)) = span(casino.iter().map(|c| c.fiscal_year)) {
        out.push(SeriesYear {
            series: "casino".into(),
            kind: YearKind::Fiscal,
            label: label_span(first, last, "FY"),
            source: "Department of Taxation, county student distribution".into(),
        });
    }

    out
}

/// The lowest and highest of an iterator of years, or `None` if it is empty.
fn span(years: impl Iterator<Item = u16>) -> Option<(u16, u16)> {
    let mut sorted: Vec<u16> = years.collect();
    sorted.sort_unstable();
    Some((*sorted.first()?, *sorted.last()?))
}

/// `FY2020-FY2025`, or just `FY2025` where the span is one year.
///
/// A range collapsed to its single member rather than printed as `FY2025-FY2025`, because a chip
/// reading the latter tells a reader there is a span to think about when there is not.
fn label_span(first: u16, last: u16, prefix: &str) -> String {
    if first == last {
        format!("{prefix}{last}")
    } else {
        format!("{prefix}{first}-{prefix}{last}")
    }
}

fn history() -> Vec<HistoryYear> {
    let equalization = equalization_by_year();
    revenue_mix_by_year()
        .into_iter()
        .map(|(fiscal_year, mix)| {
            // Every year in the mix has an equalization figure — both are computed over the same
            // comparable rows — but defaulting rather than unwrapping keeps a future year with
            // too few districts to quartile from taking the whole feed down.
            let gap = equalization.get(&fiscal_year).copied().unwrap_or_default();
            HistoryYear {
                fiscal_year,
                districts: mix.districts,
                local_share: mix.local,
                state_share: mix.state,
                federal_share: mix.federal,
                poorest_local_per_pupil: gap.poorest_local,
                richest_local_per_pupil: gap.richest_local,
                gap_per_pupil: gap.gap,
                state_closes_per_pupil: gap.state_closes,
                federal_closes_per_pupil: gap.federal_closes,
            }
        })
        .collect()
}

/// What the General Assembly appropriated, year by year.
///
/// Computed and tested in [`project::appropriations`], which excludes the property tax
/// reimbursement lines and joins two publications so the enacted series is continuous — the
/// Catalog of Budget Line Items answers for FY2006-07 and FY2012-13 and the workbooks for
/// everything else.
///
/// Unlike [`meal_program`] this **is** passed to [`deflator_years`]. It is dollars across
/// twenty-six years, and CPI-U roughly doubles across them: the nominal series grows by half and
/// the real one is close to flat, so a page showing this without the index does not merely lose
/// precision, it reports the opposite of what happened.
fn appropriation_block() -> Vec<AppropriationYear> {
    // The base year is irrelevant here — only `nominal` is carried into the feed, and the web
    // layer deflates against `bundle.deflator` like every other financial view. Passing the model
    // year keeps the call honest rather than inventing a base the feed does not use.
    let by_year: HashMap<u16, usize> = appropriations::enacted_history(MODEL_YEAR)
        .into_iter()
        .map(|y| (y.fiscal_year, y.items))
        .collect();

    let mut totals: BTreeMap<u16, (f64, f64)> = BTreeMap::new();
    let mut from_catalog: BTreeSet<u16> = BTreeSet::new();
    // Which years the workbook series answers for. Everything else came from the Catalog, and the
    // rule is read off the fixture rather than hard-coded, so a later extraction that closes
    // FY2012-13 from the workbooks re-labels these rows without anyone editing a list of years.
    let workbook_years: BTreeSet<u16> = appropriations::lines()
        .into_iter()
        .filter(|l| l.kind == "enacted")
        .map(|l| l.fiscal_year)
        .collect();

    for line in appropriations::enacted_lines() {
        if appropriations::is_tax_reimbursement(&line.line_item, line.fiscal_year) {
            continue;
        }
        if !workbook_years.contains(&line.fiscal_year) {
            from_catalog.insert(line.fiscal_year);
        }
        let entry = totals.entry(line.fiscal_year).or_insert((0.0, 0.0));
        entry.0 += line.amount;
        if FOUNDATION_LINES.contains(&line.line_item.as_str()) {
            entry.1 += line.amount;
        }
    }

    // The four years before the workbook series, from the acts themselves. Kept separate down to
    // here rather than merged into `enacted_lines`, because the acts are a different publisher
    // reading a different document and the row that says so is `source`.
    let from_acts = session_laws::department_total();
    let act_foundation = session_laws::foundation_funding();
    let act_items = session_laws::items_by_year();

    from_acts
        .into_iter()
        .map(|(fiscal_year, enacted)| AppropriationYear {
            fiscal_year,
            enacted,
            foundation_funding: act_foundation
                .get(&fiscal_year)
                .copied()
                .unwrap_or_default(),
            items: act_items.get(&fiscal_year).copied().unwrap_or_default(),
            source: "act".to_string(),
        })
        .chain(
            totals
                .into_iter()
                .map(
                    |(fiscal_year, (enacted, foundation_funding))| AppropriationYear {
                        fiscal_year,
                        enacted,
                        foundation_funding,
                        items: by_year.get(&fiscal_year).copied().unwrap_or_default(),
                        source: if from_catalog.contains(&fiscal_year) {
                            "catalog"
                        } else {
                            "workbook"
                        }
                        .to_string(),
                    },
                ),
        )
        .collect()
}

/// The lines the formula itself is paid from, across the renumbering in the middle of the series.
///
/// `200550` and `200612` are both titled `Foundation Funding` and are the GRF and Lottery Profits
/// halves of it today. `200501` is the same GRF money before FY2006, when it was titled `Base Cost
/// Funding` — the Catalog records `200550` as "originally established by Am. Sub. H.B. 66 of the
/// 126th G.A.", the FY2006-07 act, which is exactly where the number changes.
///
/// **Summing all three is safe and was checked rather than assumed.** The two GRF lines appear
/// together in FY2006-FY2011, which looks like double counting and is not: `200501` is carried at
/// exactly $0.00 in every one of those years, a discontinued line the document still lists. Had it
/// held a residual the sum would have been wrong by that residual and nothing would have shown it.
///
/// This is why the pair is a constant with a note rather than a filter written inline. An
/// appropriation line item is **not** a stable identifier across this period — `200604` names
/// three different programmes across three funds — so any series built by line number needs its
/// succession established before it means anything.
const FOUNDATION_LINES: [&str; 3] = ["200501", "200550", "200612"];

/// The department's appropriation lines, with the act that created each.
///
/// Computed in [`project::line_origins`] from the Catalog's legal-basis clause. The current
/// edition only — a line's origin does not change, so restating it once per edition would be
/// eighteen chances for two editions' wording to disagree and no way to adjudicate.
fn appropriation_lines() -> Vec<AppropriationLine> {
    line_origins::current()
        .into_iter()
        .map(|line| AppropriationLine {
            fund: line.fund,
            ali: line.ali,
            name: line.name,
            established_by: line.established_by,
            convened: line.general_assembly.map(line_origins::convened),
            general_assembly: line.general_assembly,
            discontinued: line.discontinued,
        })
        .collect()
}

/// The meal-program poverty share, year by year.
///
/// Computed and tested in [`dispersion::mr81`], which excludes non-public sponsors and the
/// sponsor-years whose published enrollment cannot be right. It reaches back to FY1998, eleven
/// years before anything else in this feed, and forward to FY2014, where the archive stops.
///
/// Deliberately not passed to [`deflator_years`]: every field here is a count or a share, so
/// there is nothing to deflate, and adding FY1998-FY2008 to the deflator would extend a price
/// index across years no dollar figure in the feed covers.
/// The casino county student fund by fiscal year, statewide.
///
/// **Every district the Department of Taxation pays**, which is around a thousand — not the 609
/// this feed carries. `dispersion::casino::by_fiscal_year` drops any fiscal year missing one of
/// its two payments, so a half-year at either end of the series is absent rather than reported as
/// a year that fell by half.
fn casino_statewide() -> Vec<CasinoYear> {
    dispersion::casino::by_fiscal_year()
        .into_iter()
        .map(|(fiscal_year, amount)| CasinoYear {
            fiscal_year,
            amount,
        })
        .collect()
}

/// The same fund per district, plus the county funds it was last paid out of.
///
/// Keyed on the IRN the tax department writes, which is the IRN the funding calculator writes for
/// every traditional district — a join checked in `crates/dispersion/tests/casino_distributions.rs`
/// rather than assumed here.
///
/// The county count comes from the **last** distribution in the panel rather than the last fiscal
/// year, because it is a fact about a district's catchment and the most recent statement of it is
/// the best one. A district absent from that distribution gets `None` rather than a stale count.
fn casino_by_district() -> HashMap<String, (Vec<CasinoYear>, Option<usize>)> {
    let complete: Vec<u16> = dispersion::casino::by_fiscal_year().into_keys().collect();
    let rows = dispersion::casino::panel();
    let last = rows
        .iter()
        .map(|row| row.month.as_str())
        .max()
        .unwrap_or_default()
        .to_string();

    let complete: BTreeSet<u16> = complete.into_iter().collect();
    let mut totals: BTreeMap<(String, u16), f64> = BTreeMap::new();
    let mut counties: HashMap<String, Option<usize>> = HashMap::new();
    for row in &rows {
        if complete.contains(&row.fiscal_year()) {
            *totals
                .entry((row.irn.clone(), row.fiscal_year()))
                .or_default() += row.amount;
        }
        if row.month == last {
            counties.insert(row.irn.clone(), row.counties);
        }
    }

    let mut out: HashMap<String, (Vec<CasinoYear>, Option<usize>)> = HashMap::new();
    for ((irn, fiscal_year), amount) in totals {
        if amount <= 0.0 {
            continue;
        }
        let entry = out.entry(irn).or_default();
        entry.0.push(CasinoYear {
            fiscal_year,
            amount,
        });
    }
    for (irn, span) in counties {
        out.entry(irn).or_default().1 = span;
    }
    out
}

fn meal_program() -> Vec<MealProgramYear> {
    poverty_share_by_year()
        .into_iter()
        .map(|(fiscal_year, year)| MealProgramYear {
            fiscal_year,
            sponsors: year.sponsors,
            enrollment: year.enrollment,
            approved: year.approved,
            identified: year.identified,
            share: year.share,
            floor: year.floor,
            ceiling: year.ceiling,
            without_applications: year.without_applications,
            streams: year.streams,
            // The upstream type carries the basis as a bool because it only has two states. The
            // feed writes the name, because a consumer reading `"basis": false` would have to
            // come back here to learn which count that was.
            basis: if year.basis_is_ce { "ce" } else { "adm" }.to_string(),
        })
        .collect()
}

/// Every year either axis of the feed carries, oldest first.
fn deflator_years(
    districts: &[District],
    history: &[HistoryYear],
    appropriations: &[AppropriationYear],
) -> Vec<u16> {
    let mut years = covered_years(districts);
    years.extend(history.iter().map(|year| year.fiscal_year));
    years.extend(appropriations.iter().map(|year| year.fiscal_year));
    years.sort_unstable();
    years.dedup();
    years
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

/// A numeric column by header name, zero where the department wrote no value.
///
/// `conventions::number` rather than `str::parse`: it is the one place that knows what the
/// department writes where there is no value. A raw parse turns `<10`, `#N/A` and a
/// thousands-separated figure alike into `None`, and `unwrap_or(0.0)` then reports every
/// one of them as zero. The zero is kept here because the surrounding type has no way to
/// carry an absence, but it is now a stated substitution rather than a parse failure.
fn number(head: &[&str], parts: &[&str], name: &str) -> f64 {
    edfund_core::conventions::number(at(head, parts, name)).unwrap_or(0.0)
}

/// Every tax year of SD-1 for a district, keyed by IRN and ordered oldest first.
///
/// Ordered because the page reads it as a change rather than as independent years, and a
/// reversed sequence would silently invert every direction it reports. Consumers that want a
/// change should take the **last two**, not the ends — see [`millage_analysis`].
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
///
/// # The last two years, not the first and last
///
/// "Year over year the recursion is exact" is a claim about *consecutive* years, and it is the
/// only reason this can skip carryover valuation. SD-1 carried two tax years when this was
/// written, so `first()` and `last()` were consecutive and the distinction did not exist. It
/// carries four now — added for `regime_diff::recognized_valuation` — and `first()` would quietly
/// become TY2021, turning a one-year recursion into a three-year one that spans a reappraisal.
/// The rate would still be a rate and the page would still render; it would simply be wrong.
fn millage_analysis(
    years: Option<&Vec<PropertyTaxYear>>,
    voted: Option<f64>,
) -> Option<MillageAnalysis> {
    let years = years?;
    let [before, after] = match years.as_slice() {
        [.., before, after] => [before, after],
        _ => return None,
    };
    debug_assert_eq!(
        after.tax_year,
        before.tax_year + 1,
        "the recursion is only exact between consecutive years"
    );
    if before.class1_value <= 0.0 || after.class1_value <= 0.0 {
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
///
/// # And it runs on recognized valuation, which is a correction
///
/// The charge-off was applied to recognized valuation — total taxable value with a reappraisal's
/// inflationary increase phased in over three years. This corpus recorded a wrong definition of
/// that term and so computed on the full value, overstating the charge-off by a median $493 per
/// pupil. `recognized_share` carries the per-district ratio and `overstated_by` carries what the
/// old base was adding, because the page should be able to show the correction rather than only
/// its result.
fn regime_counterfactual(
    record: &DistrictRecord,
    taxes: Option<&Vec<PropertyTaxYear>>,
    recognized: &HashMap<String, regime_diff::Recognition>,
) -> Option<RegimeCounterfactual> {
    let base = regime_diff::ChargeOffBase::Recognized(recognized);
    let diff = regime_diff::at_fy2027(record, regime_diff::TERMINAL_MILLS, base);
    let component = diff.components.first()?;
    let recognized_share = base.ratio_for(&record.irn);

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
        recognized_share,
        reappraisal_year: regime_diff::recognized_valuation::cycle_for(&record.county)
            .map_or(0, |c| c.tax_year),
        // What the discarded base was adding, in the same per-pupil unit as the rest of the row.
        overstated_by: record.valuation_per_pupil.map(|valuation| {
            valuation * (1.0 - recognized_share) * regime_diff::TERMINAL_MILLS / 1_000.0
        }),
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

/// Assemble the bundle from the committed fixtures.
///
/// Split from `main` so the tests can assert on the document this crate actually publishes rather
/// than on its inputs. `main` was 273 lines ending in a `print!`, and a rule about what the bundle
/// carries — that every share in it is a fraction — could only be checked by a consumer parsing
/// the JSON back, which is the layer that had the bug.
pub fn build() -> Bundle {
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
    let cpi = deflator::CpiSeries::cpi_u_june();
    let taxes = property_taxes();
    let functions = spending_by_function();

    // Which House districts each district lies in, keyed by IRN. Built once: the crosswalk is a
    // 1,085-row fixture and reading it per district would parse it 609 times.
    let mut shares: HashMap<String, Vec<HouseDistrictShare>> = HashMap::new();
    for overlap in overlaps(Chamber::House) {
        shares
            .entry(overlap.irn.clone())
            .or_default()
            .push(HouseDistrictShare {
                number: overlap.district.clone(),
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

    let (national_positions, _national_medians) = dispersion::national_peers::positions();

    // Recognized valuation at TY2024, parsed once for the whole panel. The charge-off
    // counterfactual runs on this rather than on total taxable value; see `regime_counterfactual`.
    let recognized = regime_diff::recognized_valuation::from_abstract(2024);

    let casino_by_district = casino_by_district();

    let districts: Vec<District> = records
        .iter()
        .map(|record| {
            to_district(
                record,
                &Joins {
                    profile: profile.get(record.irn.as_str()),
                    outcome: outcomes.iter().find(|j| j.funding.irn == record.irn),
                    money: for_district(&money, &record.irn),
                    taxes: taxes.get(&record.irn),
                    functions: functions.get(&record.irn),
                    house_districts: shares.get(&record.irn).map_or(&[][..], Vec::as_slice),
                    national: national_positions.get(&record.irn),
                    recognized: &recognized,
                    casino: casino_by_district.get(&record.irn),
                },
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

    let history = history();
    let appropriations = appropriation_block();
    // Bound rather than called twice apiece: each reparses the MR-81 archive from scratch, and
    // both are wanted again below in `series_years`.
    let meal_program = meal_program();
    let casino = casino_statewide();

    Bundle {
        national: national(),
        history: history.clone(),
        appropriation_lines: appropriation_lines(),
        appropriations: appropriations.clone(),
        meal_program: meal_program.clone(),
        casino: casino.clone(),
        house_districts: house_district_block(&records, Chamber::House),
        senate_districts: house_district_block(&records, Chamber::Senate),
        contract_version: CONTRACT_VERSION.to_string(),
        // Deliberately no longer restates the years. It used to name every one of them, and it
        // said "millage is TY2023" while all 609 districts carried `tax_year: 2024` — a sentence
        // and a column cannot disagree in a way anything notices, and the sentence is the half a
        // reader sees. Years are in `series_years` now, derived, and stated once.
        // Plain prose, no backticks: this string is rendered into the site footer as text, so a
        // code span would print its own punctuation to the reader.
        provenance: "Ohio DEW TRAD State Foundation Funding Calculator (a projection, not an \
                     actual) joined with the District Profile Report, the Department of \
                     Taxation's Table SD-1, and the Ohio School Report Card on IRN across the \
                     606 districts every panel covers. Enrolled ADM spans three years, of which \
                     the last is partly departmental estimate. Each block states the year it is \
                     measured in beside its own figures; see .yidam/catalog/ for what each \
                     source can be trusted for."
            .to_string(),
        fiscal_year: MODEL_YEAR.0,
        series_years: series_years(
            &districts,
            &history,
            &appropriations,
            &meal_program,
            &casino,
        ),
        statewide,
        checkpoints,
        drafts: draft_export(),
        projection: Some(projection),
        deflator: Some(Deflator {
            label: cpi.label().to_string(),
            // The union of both axes rather than the financial panel alone. The history block
            // spans FY2009-FY2022 and CPI-U rose 37% across it, so a page showing that gap in
            // nominal dollars would report a widening that is partly just money getting smaller.
            // A deflator that covers only part of what the feed carries is the failure mode where
            // the page silently falls back to nominal for the years it cannot convert.
            points: deflator_years(&districts, &history, &appropriations)
                .into_iter()
                .filter_map(|year| cpi.point(FiscalYear(year)).map(|point| (year, point.index)))
                .collect(),
        }),
        districts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_card_fixture_and_year_agree() {
        /*
         * The label and the file it labels, held together.
         *
         * `2024-25` used to be a string literal in an Astro `<meta>` description, and the year
         * before that it was a different string literal in the same place. Nothing connected
         * either to the download it described, so replacing the fixture would have left the page
         * confidently naming the wrong school year — and a regenerated constant produces no diff,
         * which is the failure `connect::index`'s node count had when it was the literal `58`.
         *
         * `2024-25` -> `2425` is the department's own filename convention.
         */
        let compact: String = REPORT_CARD_YEAR.split('-').collect();
        assert_eq!(compact.len(), 6, "a school year is `YYYY-YY`");
        let short = format!("{}{}", &compact[2..4], &compact[4..6]);
        assert!(
            REPORT_CARD_FIXTURE.contains(&short),
            "REPORT_CARD_YEAR is {REPORT_CARD_YEAR}, so the fixture should carry `{short}`, and \
             it is {REPORT_CARD_FIXTURE}"
        );
    }

    #[test]
    fn the_spending_year_is_the_later_half_of_the_report_card_year() {
        /*
         * One download, two reckonings: attainment for the 2024-25 school year and operating
         * expenditure for FY2025. They are the same period and they are not the same label, and
         * the relationship between them is the thing worth pinning — if the report card moves on,
         * both constants move together or the pair is wrong.
         */
        let ends_in: u16 = REPORT_CARD_YEAR[..4]
            .parse::<u16>()
            .expect("a leading year")
            + 1;
        assert_eq!(REPORT_CARD_SPENDING_YEAR, ends_in);
    }

    #[test]
    fn a_one_year_span_is_not_written_as_a_range() {
        // `FY2025-FY2025` tells a reader there is a span to think about when there is not.
        assert_eq!(label_span(2025, 2025, "FY"), "FY2025");
        assert_eq!(label_span(2020, 2025, "FY"), "FY2020-FY2025");
        assert_eq!(label_span(2024, 2024, ""), "2024");
    }

    /// Every share this bundle publishes is a fraction.
    ///
    /// The rule was true of all but three fields and enforced by nothing, so the three that broke
    /// it broke it silently: the report card publishes 0 to 100, `main` passed those straight
    /// through, and `outcome.economically_disadvantaged` sat in the same document as
    /// `District::economically_disadvantaged` 100× apart under the same name. A consumer reading
    /// the wrong one got a plausible number, and a percentage through a helper expecting a
    /// fraction rendered `10000%` rather than failing.
    ///
    /// So the rule is a test rather than a convention. A share arriving from a new source on a
    /// percentage scale now fails here, at the point it is added, naming the field.
    #[test]
    fn share_fields_are_fractions() {
        let bundle = super::build();

        let mut offenders: Vec<String> = Vec::new();
        let mut check = |name: &str, irn: &str, value: Option<f64>| {
            if let Some(v) = value {
                if !(0.0..=1.0).contains(&v) {
                    offenders.push(format!("{name} = {v} for IRN {irn}"));
                }
            }
        };

        for d in &bundle.districts {
            check(
                "economically_disadvantaged",
                &d.irn,
                d.economically_disadvantaged,
            );
            check("dpia.percentage", &d.irn, Some(d.dpia.percentage));
            if let Some(r) = &d.regime {
                check("regime.recognized_share", &d.irn, Some(r.recognized_share));
            }
            check(
                "transportation.effective_state_share",
                &d.irn,
                Some(d.transportation.effective_state_share),
            );
            if let Some(o) = &d.outcome {
                check(
                    "outcome.economically_disadvantaged",
                    &d.irn,
                    o.economically_disadvantaged,
                );
                check("outcome.english_learner", &d.irn, o.english_learner);
                check(
                    "outcome.students_with_disabilities",
                    &d.irn,
                    o.students_with_disabilities,
                );
            }
            if let Some(n) = &d.national {
                check("national.local_share", &d.irn, Some(n.local_share));
                check(
                    "national.local_share_percentile",
                    &d.irn,
                    Some(n.local_share_percentile),
                );
                check(
                    "national.revenue_per_pupil_percentile",
                    &d.irn,
                    Some(n.revenue_per_pupil_percentile),
                );
                check(
                    "national.spending_per_pupil_percentile",
                    &d.irn,
                    Some(n.spending_per_pupil_percentile),
                );
            }
        }

        assert!(
            offenders.is_empty(),
            "{} share field(s) outside 0..=1, so the bundle publishes two scales under one \
             convention. Divide at the seam in `to_district`, as the report-card shares are, \
             and bump CONTRACT_VERSION:\n  {}",
            offenders.len(),
            offenders
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }

    /// The three the rule was written for, at the value that exposed it.
    ///
    /// 31 districts publish exactly 100% economically disadvantaged in the profile report, which
    /// is the value that made the mismatch visible on the site: as a fraction that is `1.0`, and
    /// as the report card's percentage it was `100.0`. Both are in this bundle under the same
    /// name, and this pins which is which.
    #[test]
    fn report_card_shares_are_converted() {
        let bundle = super::build();
        let with_outcome: Vec<_> = bundle
            .districts
            .iter()
            .filter_map(|d| d.outcome.as_ref())
            .collect();
        assert!(
            with_outcome.len() > 500,
            "the report card covers most districts"
        );

        let disadvantaged: Vec<f64> = with_outcome
            .iter()
            .filter_map(|o| o.economically_disadvantaged)
            .collect();
        let max = disadvantaged.iter().copied().fold(f64::MIN, f64::max);
        // Districts genuinely reach the ceiling, so the top of the range is 1.0 and not merely
        // "below 1.0" — a conversion that quietly clamped would pass a laxer assertion.
        assert!((max - 1.0).abs() < 1e-9, "the ceiling is 1.0, not {max}");
        assert!(
            disadvantaged.iter().any(|v| *v > 0.5),
            "the scale is not divided twice"
        );
    }
}
