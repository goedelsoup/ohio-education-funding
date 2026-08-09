//! The charge-off against local capacity, at FY2027 inputs.
//!
//! Run with `cargo run --example charge_off_counterfactual -p regime-diff`. The runner is the
//! result: deterministic and dependency-free, so re-running reproduces these figures exactly.
//!
//! Everything printed is asserted in `tests/the_charge_off_against_local_capacity.rs`.

use project::panel::{panel, DistrictRecord};
use regime_diff::recognized_valuation::{self, PhaseIn};
use regime_diff::{
    at_fy2027, panel_at_fy2027, ChargeOffBase, RegimeDiff, ALIGNED, RATES, TERMINAL_MILLS,
};

fn main() {
    let districts = panel();
    let recognized = recognized_valuation::from_abstract(2024);
    let base = ChargeOffBase::Recognized(&recognized);
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS, base);

    println!("THE CHARGE-OFF RATE, AS STATUTE SET IT\n");
    for rate in RATES {
        println!(
            "  {:>4.1} mills  {:<40.40} {}",
            rate.mills,
            rate.effective,
            match rate.base {
                regime_diff::charge_off::ValuationBase::TotalTaxable => "total taxable value",
                regime_diff::charge_off::ValuationBase::Recognized => "recognized valuation",
            }
        );
        println!("               {}", rate.authority);
    }

    println!("\nTHE BASE, RECONSTRUCTED — AND WHAT THIS CORPUS HAD WRONG\n");
    println!("  Recognized valuation is not an H.B. 920 adjustment. It phases a reappraisal's");
    println!("  inflationary increase in over three years, on the Department of Taxation's own");
    println!("  staggered county calendar.\n");
    let mut by_phase: std::collections::BTreeMap<&str, (usize, f64, f64)> = Default::default();
    for r in recognized.values() {
        let label = match r.phase {
            PhaseIn::First => "reappraised this year   (2/3 deferred)",
            PhaseIn::Second => "reappraised last year   (1/3 deferred)",
            PhaseIn::Third => "phase-in complete       (  0 deferred)",
        };
        let entry = by_phase.entry(label).or_insert((0, 0.0, 0.0));
        entry.0 += 1;
        entry.1 += r.actual;
        entry.2 += r.deferred;
    }
    let (mut actual_all, mut deferred_all) = (0.0, 0.0);
    for (label, (n, actual, deferred)) in &by_phase {
        actual_all += actual;
        deferred_all += deferred;
        println!(
            "  {label}  n={n:3}   {:>6.2}% of value deferred   ${:>7.1}m of charge-off",
            100.0 * deferred / actual,
            deferred * TERMINAL_MILLS / 1_000.0 / 1e6
        );
    }
    println!(
        "  {:<40}         {:>6.2}% statewide           ${:>7.1}m",
        "all districts",
        100.0 * deferred_all / actual_all,
        deferred_all * TERMINAL_MILLS / 1_000.0 / 1e6
    );
    println!(
        "\n  Which relief a district gets is decided by its county's place on that calendar,\n  \
         and by nothing about the district."
    );

    println!("\nTHE ONE PAIR THE CORPUS ALIGNS\n");
    println!(
        "  {} replaces {}",
        ALIGNED[0].successor, ALIGNED[0].predecessor
    );
    println!("  role: {}\n", ALIGNED[0].role);

    let complete = diffs.iter().filter(|d| d.is_complete()).count();
    let censored = diffs
        .iter()
        .filter(|d| d.components[0].successor.is_none())
        .count();
    let totals = diffs
        .iter()
        .filter(|d| d.total_difference().is_some())
        .count();
    println!(
        "  {} districts; {complete} fully valued, {censored} censored by the minimum state share",
        diffs.len()
    );
    println!("  totals computable for {totals}; attributions for {complete}");

    let exact = diffs
        .iter()
        .filter(|d| d.residual().is_some_and(|r| r.abs() <= 0.005))
        .count();
    println!("  residual exactly zero for {exact} of {complete}\n");

    for name in [
        "Ottawa Hills Local",
        "Jefferson Township Local",
        "Shaker Heights City",
    ] {
        let record = districts
            .iter()
            .find(|r| r.name == name)
            .expect("in the model");
        show(record, &at_fy2027(record, TERMINAL_MILLS, base));
    }

    // The correction, stated as the difference it makes rather than as a rate.
    let old = panel_at_fy2027(&districts, TERMINAL_MILLS, ChargeOffBase::TotalTaxable);
    let mut overstatement: Vec<f64> = old
        .iter()
        .zip(&diffs)
        .filter_map(|(a, b)| Some(a.components[0].predecessor? - b.components[0].predecessor?))
        .filter(|d| *d > 0.01)
        .collect();
    overstatement.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let zeroed_before = old
        .iter()
        .filter(|d| d.predecessor_total == Some(0.0))
        .count();
    println!("AGAINST THE BASE THIS CORPUS USED TO ASSUME\n");
    println!(
        "  {} of {} districts had their charge-off overstated",
        overstatement.len(),
        diffs.len()
    );
    println!(
        "  by a median of ${:.0} per pupil and as much as ${:.0}",
        overstatement[overstatement.len() / 2],
        overstatement[overstatement.len() - 1]
    );
    println!(
        "  and {zeroed_before} districts were shown losing all base cost aid where {} do",
        diffs
            .iter()
            .filter(|d| d.predecessor_total == Some(0.0))
            .count()
    );

    let favours_plan = diffs
        .iter()
        .filter(|d| d.total_difference().is_some_and(|t| t > 0.01))
        .count();
    let favours_charge_off = diffs
        .iter()
        .filter(|d| d.total_difference().is_some_and(|t| t < -0.01))
        .count();
    let zeroed = diffs
        .iter()
        .filter(|d| d.predecessor_total == Some(0.0))
        .count();
    println!("\nDIRECTION ACROSS THE STATE\n");
    println!("  {favours_plan} districts do better under local capacity");
    println!("  {favours_charge_off} would have done better under the charge-off");
    println!("  {zeroed} would receive no base cost aid at all — the charge-off had no floor");

    let mut rows: Vec<(f64, f64)> = districts
        .iter()
        .zip(&diffs)
        .filter_map(|(r, d)| Some((r.valuation_per_pupil?, d.total_difference()?)))
        .collect();
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("no NaN"));
    println!("\n  mean gain per pupil against the charge-off, by valuation quintile");
    for q in 0..5 {
        let band = &rows[q * rows.len() / 5..(q + 1) * rows.len() / 5];
        let mean: f64 = band.iter().map(|(_, d)| d).sum::<f64>() / band.len() as f64;
        println!(
            "    Q{}  n={:3}  {:>9.0} to {:>9.0}   {:>+9.2}",
            q + 1,
            band.len(),
            band[0].0,
            band[band.len() - 1].0,
            mean
        );
    }
}

fn show(record: &DistrictRecord, diff: &RegimeDiff) {
    println!("{}", diff.name);
    println!(
        "  valuation/pupil {:>10.0}   base cost/pupil {:>9.2}   at the floor: {}",
        record.valuation_per_pupil.unwrap_or(0.0),
        record.base_cost_per_pupil,
        record.at_minimum_state_share()
    );
    let component = diff.components[0];
    print!(
        "  local share/pupil: charge-off {:>9.2}   local capacity ",
        component.predecessor.unwrap_or(0.0)
    );
    match component.successor {
        Some(value) => println!("{value:>9.2}"),
        None => println!("  censored"),
    }
    println!(
        "  base cost aid/pupil: {:>9.2} -> {:>9.2}   difference {:>+9.2}",
        diff.predecessor_total.unwrap_or(0.0),
        diff.successor_total.unwrap_or(0.0),
        diff.total_difference().unwrap_or(0.0)
    );
    match diff.residual() {
        Some(residual) => println!("  residual {residual:>+9.2}\n"),
        None => println!("  residual: unattributable — the component row is censored\n"),
    }
}
