//! The department's appropriation, FY2014 to FY2027, nominal and in FY2026 dollars.
//!
//! ```text
//! cargo run -p project --example appropriation_history
//! ```

use edfund_core::FiscalYear;
use project::appropriations::{enacted_history, growth, line_history};

fn main() {
    let base = FiscalYear(2026);
    let history = enacted_history(base);

    println!("Department of Education, enacted appropriation, FY{base:?} dollars");
    for year in &history {
        let real = year
            .real
            .map_or_else(|| "no index".to_string(), |real| format!("{real:.0}"));
        println!(
            "  FY{}  {:>15.0}  {real:>15}   {} lines",
            year.fiscal_year, year.nominal, year.items
        );
    }

    // The index reaches FY2026, so the deflated comparison ends there and the last appropriated
    // year is left out of it rather than compared on one deflated endpoint and one nominal.
    let deflated: Vec<_> = history
        .iter()
        .filter(|y| y.real.is_some())
        .copied()
        .collect();
    if let Some((nominal, real, confidence)) = growth(&deflated) {
        println!(
            "\nFY{}-FY{}: {:+.1}% nominal, {:+.1}% real [{confidence:?}]",
            deflated[0].fiscal_year,
            deflated[deflated.len() - 1].fiscal_year,
            nominal * 100.0,
            real * 100.0
        );
    }

    let foundation: Vec<_> = line_history("200550", base)
        .into_iter()
        .filter(|y| y.real.is_some())
        .collect();
    if let Some((nominal, real, _)) = growth(&foundation) {
        println!(
            "Foundation Funding (200550), FY{}-FY{}: {:+.1}% nominal, {:+.1}% real",
            foundation[0].fiscal_year,
            foundation[foundation.len() - 1].fiscal_year,
            nominal * 100.0,
            real * 100.0
        );
    }
}
