//! Restate Ohio's per-pupil operating expenditure series in constant FY2022 dollars.
//!
//! Reads the nominal series recorded on
//! `.yidam/corpus/metric/per-pupil-operating-expenditure.yml` and emits the deflated series
//! for commit back into that node. Run from `crates/`:
//!
//! ```text
//! cargo run --example epp_real_series -p deflator
//! ```
//!
//! Output confidence is inherited from the CPI endpoints, so most rows come out
//! `[inference]` — the intervening CPI observations are transcribed and unverified, and the
//! nominal figures for even years are chart-label approximations. Only the FY2000 and FY2022
//! endpoints are verified on both sides.

use deflator::{Confidence, CpiSeries};
use edfund_core::FiscalYear;

/// Nominal operating expenditure per pupil, all Ohio public school entities.
///
/// Moved to a fixture so the integration test reads the same series this example prints. A
/// series that lives only inside an example binary cannot be asserted on.
const NOMINAL_CSV: &str = include_str!("../fixtures/ohio-epp-nominal.csv");

/// Parse the fixture into (fiscal year, nominal dollars, exact-or-chart-label).
fn nominal() -> Vec<(u16, f64, bool)> {
    NOMINAL_CSV
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let f: Vec<&str> = line.split(',').collect();
            (
                f[0].trim().parse().expect("fiscal year"),
                f[1].trim().parse().expect("dollars"),
                f[2].trim() == "true",
            )
        })
        .collect()
}

/// FY2022 excluding federal COVID relief funds.
const FY2022_EX_RELIEF: f64 = 14_493.0;

fn main() {
    let cpi = CpiSeries::cpi_u_june();
    let base = FiscalYear(2022);

    println!("Ohio public school operating expenditure per pupil");
    println!("Constant FY2022 dollars, deflated by {}", cpi.label());
    println!();
    println!(
        "{:<8} {:>12} {:>14} {:>14}  confidence",
        "FY", "nominal", "real (FY2022)", "vs FY2000"
    );

    let fy2000_real = cpi
        .convert(nominal()[0].1, FiscalYear(2000), base)
        .expect("FY2000 is in the series");

    for &(year, nominal, exact_nominal) in &nominal() {
        let fy = FiscalYear(year);
        let real = cpi.convert(nominal, fy, base).expect("year in series");
        let growth = real.value / fy2000_real.value - 1.0;

        // A row is only as strong as its weakest input: the CPI endpoints and the nominal
        // figure both have to be solid for the result to be quotable as verified.
        let confidence = if real.confidence == Confidence::Verified && exact_nominal {
            "verified"
        } else {
            "inference"
        };

        println!(
            "{:<8} {:>12} {:>14} {:>13.1}%  {}",
            fy,
            format!("${nominal:>10.0}"),
            format!("${:>10.0}", real.value),
            growth * 100.0,
            confidence
        );
    }

    let ex_relief = cpi
        .convert(FY2022_EX_RELIEF, base, base)
        .expect("base year");
    let ex_relief_growth = ex_relief.value / fy2000_real.value - 1.0;
    println!(
        "{:<8} {:>12} {:>14} {:>13.1}%  verified",
        "FY2022*",
        format!("${FY2022_EX_RELIEF:>10.0}"),
        format!("${:>10.0}", ex_relief.value),
        ex_relief_growth * 100.0,
    );
    println!();
    println!("* excluding federal COVID relief funds");
    println!(
        "FY2000 restated in FY2022 dollars: ${:.0}",
        fy2000_real.value
    );
}
