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

use bundle::{Bundle, Checkpoint, District, PolicyShape, Statewide, CONTRACT_VERSION};
use dispersion::wealth_neutrality;
use project::panel::{panel, DistrictRecord, MINIMUM_STATE_SHARE, MODEL_YEAR};
use project::policy::{GuaranteeRule, Policy};
use project::report::simulate;

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

fn to_district(record: &DistrictRecord, profile: Option<&&str>) -> District {
    let adm = record.base_cost_adm();
    District {
        irn: record.irn.clone(),
        name: record.name.clone(),
        adm,
        current_year_adm: record.current_year_adm,
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
    let districts: Vec<District> = records
        .iter()
        .map(|record| to_district(record, profile.get(record.irn.as_str())))
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
    };

    let checkpoints: Vec<Checkpoint> = checkpoint_policies()
        .into_iter()
        .map(|(label, policy, shape)| {
            let effect = simulate(&records, &policy);
            Checkpoint {
                label: label.to_string(),
                policy: shape,
                cost: effect.cost(),
                realized_aid: effect.policy.realized_aid,
                gainers: effect.gainers(),
                losers: effect.losers(),
                unmoved: effect.unmoved(),
                on_guarantee: effect.policy.on_guarantee,
            }
        })
        .collect();

    let bundle = Bundle {
        contract_version: CONTRACT_VERSION.to_string(),
        provenance: "Ohio DEW FY27 TRAD State Foundation Funding Calculator (a projection, not \
                     an actual) joined with the FY2024 District Profile Report. Base cost, \
                     guarantee, and formula funding are FY2027; enrolled ADM is FY2024-FY2026, \
                     of which FY2026 is partly departmental estimate; millage is TY2023; \
                     expenditure and demographics are FY2024. See .yidam/catalog/."
            .to_string(),
        fiscal_year: MODEL_YEAR.0,
        statewide,
        checkpoints,
        districts,
    };
    print!("{}", bundle.to_json());
}
