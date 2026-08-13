//! Which appropriation lines gave up ground while the department shrank in real terms.
//!
//! ```text
//! cargo run -p project --example what_absorbed_it
//! ```

use edfund_core::FiscalYear;
use project::appropriations::changes;

fn main() {
    let mut moved = changes(FiscalYear(2026), 2014, 2026);
    moved.sort_by(|a, b| a.shift().partial_cmp(&b.shift()).expect("no NaN"));
    println!("Largest real-terms falls, FY2014 to FY2026, in FY2026 dollars:");
    for change in moved.iter().take(10) {
        println!(
            "  {:>14.0}  {}  {}",
            change.shift(),
            change.line_item,
            change.title
        );
    }
    println!("\nLargest real-terms gains:");
    for change in moved.iter().rev().take(5) {
        println!(
            "  {:>14.0}  {}  {}",
            change.shift(),
            change.line_item,
            change.title
        );
    }
    let falls: f64 = moved.iter().map(|c| c.shift()).filter(|s| *s < 0.0).sum();
    let gains: f64 = moved.iter().map(|c| c.shift()).filter(|s| *s > 0.0).sum();
    println!(
        "\n{} lines in both years: {falls:.0} lost, {gains:.0} gained",
        moved.len()
    );

    // The lines that did not survive the window, which is where the rest of it went.
    use project::appropriations::{enacted_history, lines, TAX_REIMBURSEMENT};
    use std::collections::BTreeSet;
    let present = |year: u16| -> BTreeSet<String> {
        lines()
            .iter()
            .filter(|l| {
                l.kind == "enacted"
                    && l.fiscal_year == year
                    && !TAX_REIMBURSEMENT.contains(&l.line_item.as_str())
            })
            .map(|l| l.line_item.clone())
            .collect()
    };
    let (start, end) = (present(2014), present(2026));
    let gone: Vec<_> = start.difference(&end).cloned().collect();
    let born: Vec<_> = end.difference(&start).cloned().collect();
    let sum = |year: u16, set: &[String]| -> f64 {
        lines()
            .iter()
            .filter(|l| l.kind == "enacted" && l.fiscal_year == year && set.contains(&l.line_item))
            .map(|l| l.amount)
            .sum()
    };
    println!(
        "\n{} lines existed in FY2014 and not FY2026, worth {:.0} nominal then",
        gone.len(),
        sum(2014, &gone)
    );
    println!(
        "{} lines exist in FY2026 and not FY2014, worth {:.0} nominal now",
        born.len(),
        sum(2026, &born)
    );
    for year in [2014u16, 2026] {
        let h = enacted_history(FiscalYear(2026));
        let y = h.iter().find(|y| y.fiscal_year == year).expect("year");
        println!(
            "  FY{year}: {} lines, {:.0} real",
            y.items,
            y.real.unwrap_or(0.0)
        );
    }
}
