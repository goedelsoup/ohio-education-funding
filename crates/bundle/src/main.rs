//! Build the web feed by joining the two committed district fixtures.
//!
//! ```text
//! cargo run -p bundle > ../web/data/bundle.json
//! ```
//!
//! A thin shell: it reads two fixtures, joins them on IRN, computes the statewide aggregates,
//! and prints JSON. All the logic worth testing lives in the library.

use bundle::{Bundle, District, Statewide, CONTRACT_VERSION};
use dispersion::wealth_neutrality;
use std::collections::HashMap;

/// The department's FY2027 funding model: base cost, guarantee, core foundation funding.
const FY27: &str = include_str!("../../foundation/fixtures/fy27-department-model.csv");
/// The FY2024 District Profile Report: millage, expenditure, demographics.
const CUPP: &str = include_str!("../../dispersion/fixtures/cupp-fy24-district-data.csv");

fn field(line: &str, i: usize) -> Option<&str> {
    line.split(',').nth(i).map(str::trim)
}

fn parse(line: &str, i: usize) -> Option<f64> {
    field(line, i).and_then(|v| v.parse::<f64>().ok())
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    // FY27 model columns: 2 base cost ADM, 14 base cost per pupil, 15 guarantee,
    // 16/18 enrolled ADM FY2024 and FY2026, 19 valuation per pupil, 20 core foundation.
    // Cupp columns: 4 valuation/pupil, 6 effective class 1 millage, 7 operating EPP,
    // 3 economically disadvantaged.
    let cupp: HashMap<&str, &str> = CUPP
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| field(l, 0).map(|irn| (irn, l)))
        .collect();

    let mut districts: Vec<District> = Vec::new();
    for line in FY27.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let Some(irn) = field(line, 0) else { continue };
        let (Some(adm), Some(core)) = (parse(line, 2), parse(line, 20)) else {
            continue;
        };
        if adm <= 0.0 {
            continue;
        }
        let guarantee = parse(line, 15).unwrap_or(0.0);
        let c = cupp.get(irn);

        districts.push(District {
            irn: irn.to_string(),
            name: field(line, 1).unwrap_or_default().to_string(),
            adm,
            base_cost_per_pupil: parse(line, 14).unwrap_or(0.0),
            formula_aid_per_pupil: core / adm,
            realized_aid_per_pupil: (core + guarantee) / adm,
            guarantee,
            valuation_per_pupil: parse(line, 19).or_else(|| c.and_then(|l| parse(l, 4))),
            effective_class1_millage: c.and_then(|l| parse(l, 6)),
            operating_expenditure_per_pupil: c.and_then(|l| parse(l, 7)),
            economically_disadvantaged: c.and_then(|l| parse(l, 3)),
            enrollment_change: match (parse(line, 16), parse(line, 18)) {
                (Some(a), Some(b)) if a > 0.0 => Some(b / a - 1.0),
                _ => None,
            },
        });
    }

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
        median_valuation_per_pupil: median(wealth.clone()),
        median_operating_expenditure_per_pupil: median(
            districts
                .iter()
                .filter_map(|d| d.operating_expenditure_per_pupil)
                .collect(),
        ),
        wealth_neutrality_formula: wealth_neutrality(&wealth, &formula)
            .map(|w| w.correlation)
            .unwrap_or(f64::NAN),
        wealth_neutrality_realized: wealth_neutrality(&wealth, &realized)
            .map(|w| w.correlation)
            .unwrap_or(f64::NAN),
        guarantee_total: districts.iter().map(|d| d.guarantee).sum(),
    };

    let bundle = Bundle {
        contract_version: CONTRACT_VERSION.to_string(),
        provenance: "Ohio DEW FY27 TRAD State Foundation Funding Calculator (projection) joined \
                     with the FY2024 District Profile Report. Base cost, guarantee, and core \
                     foundation funding are FY2027 projections; millage is TY2023; expenditure \
                     and demographics are FY2024. See .yidam/catalog/."
            .to_string(),
        statewide,
        districts,
    };
    print!("{}", bundle.to_json());
}
