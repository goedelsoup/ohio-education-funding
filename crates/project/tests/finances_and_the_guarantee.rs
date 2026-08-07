//! What the financial actuals say that the funding model could not.
//!
//! Every figure this repository held before this panel was **modelled**: what a formula computes,
//! what a district spent per pupil on the department's definitions, what its pupils achieved.
//! None was a record of money arriving. These tests pin what changed when one did.
//!
//! Two cautions run through all of it, and both are load-bearing rather than decorative.
//!
//! **Booked aid is not formula output.** A district's unrestricted grants-in-aid is state
//! foundation money as its treasurer records it in the general fund, and the FY2027 calculator's
//! "total state support" is a different construction. Any single ratio between them carries that
//! gap. What survives it is a *comparison between groups measured the same way*, which is why
//! every claim here about the guarantee is stated against a formula-funded control.
//!
//! **FY2021 to FY2024 are the pandemic relief years.** Federal money was booked in the general
//! fund by some districts and separately by others, so a balance rising across them is not
//! evidence about a district's own position.

use edfund_core::FiscalYear;
use project::finances::{finances, for_district, Finances};
use project::panel::panel;

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[values.len() / 2]
}

fn statewide(year: u16, pick: impl Fn(&project::finances::YearRecord) -> f64) -> f64 {
    finances()
        .iter()
        .filter_map(|f| f.year(FiscalYear(year)))
        .map(&pick)
        .sum()
}

#[test]
fn the_financial_panel_reaches_every_district_the_funding_model_pays() {
    // Not "most". A statewide financial figure computed over a subset of the funding panel would
    // be quietly answering a different question, and there would be nothing to notice.
    let money = finances();
    let funding = panel();
    let missing: Vec<&str> = funding
        .iter()
        .filter(|record| for_district(&money, &record.irn).is_none())
        .map(|record| record.name.as_str())
        .collect();
    assert!(missing.is_empty(), "no filing for {missing:?}");
    assert_eq!(funding.len(), 609);
}

#[test]
fn statewide_cash_rose_through_the_relief_years_and_then_fell() {
    // The shape of the whole panel in one line. Balances built up while federal money was
    // arriving and fell in the first year after it stopped.
    let cash = |year| statewide(year, |y| y.ending_cash) / 1e9;
    let fy2020 = cash(2020);
    let peak = cash(2024);
    let fy2025 = cash(2025);

    assert!((fy2020 - 8.37).abs() < 0.02, "FY2020 ${fy2020:.2}bn");
    assert!((peak - 11.20).abs() < 0.02, "FY2024 ${peak:.2}bn");
    assert!((fy2025 - 9.14).abs() < 0.02, "FY2025 ${fy2025:.2}bn");

    // Monotone up to the peak, then the only fall in the series.
    for pair in [2020, 2021, 2022, 2023, 2024].windows(2) {
        assert!(
            cash(pair[1]) > cash(pair[0]),
            "FY{} did not exceed FY{}",
            pair[1],
            pair[0]
        );
    }
    assert!(
        peak - fy2025 > 2.0,
        "the FY2025 drawdown was ${:.2}bn",
        peak - fy2025
    );
}

#[test]
fn fy2025_is_the_year_the_spending_caught_up() {
    // Revenue rose 5%; spending rose 16%. The gap came out of the balances above, and it is not
    // one or two large districts — the median district raised spending by more than a tenth.
    let revenue = |year| statewide(year, |y| y.total_revenue);
    let spending = |year| statewide(year, |y| y.total_expenditure);

    let revenue_growth = revenue(2025) / revenue(2024) - 1.0;
    let spending_growth = spending(2025) / spending(2024) - 1.0;
    assert!(
        (revenue_growth - 0.049).abs() < 0.005,
        "revenue grew {revenue_growth:.3}"
    );
    assert!(
        (spending_growth - 0.165).abs() < 0.005,
        "spending grew {spending_growth:.3}"
    );

    let ratios: Vec<f64> = finances()
        .iter()
        .filter_map(|f| {
            let before = f.year(FiscalYear(2024))?;
            let after = f.year(FiscalYear(2025))?;
            (before.total_expenditure > 1e6)
                .then(|| after.total_expenditure / before.total_expenditure)
        })
        .collect();
    let typical = median(ratios);
    assert!(
        (typical - 1.137).abs() < 0.01,
        "median district spending ratio {typical:.3} — a statewide jump driven by a handful of \
         districts would leave this near 1.0"
    );
}

#[test]
fn most_districts_spent_more_than_they_received_in_the_most_recent_closed_year() {
    let deficits = finances()
        .iter()
        .filter_map(|f| f.year(FiscalYear(2025)))
        .filter(|y| y.operating_result() < 0.0)
        .count();
    let total = finances()
        .iter()
        .filter(|f| f.year(FiscalYear(2025)).is_some())
        .count();
    assert_eq!(deficits, 460);
    assert_eq!(total, 659);
    // Against the year before, when it was a minority.
    let prior = finances()
        .iter()
        .filter_map(|f| f.year(FiscalYear(2024)))
        .filter(|y| y.operating_result() < 0.0)
        .count();
    assert!(
        prior < total / 2,
        "{prior} of {total} ran a deficit in FY2024 too, so FY2025 is not the change it looks like"
    );
}

#[test]
fn the_guarantee_is_not_producing_districts_that_sit_on_cash() {
    // A standing political claim about the guarantee, and the panel does not support it. Cash is
    // measured against the scale of the operation, because the dollar figure is meaningless
    // without it: the same $10M is a year of reserve for a small district and three weeks for a
    // large one.
    let money = finances();
    let mut guaranteed = Vec::new();
    let mut on_formula = Vec::new();
    for record in panel() {
        let Some(finances) = for_district(&money, &record.irn) else {
            continue;
        };
        let Some(cash) = finances
            .year(FiscalYear(2025))
            .and_then(project::finances::YearRecord::cash_as_years_of_spending)
        else {
            continue;
        };
        if record.guarantee > 0.0 {
            guaranteed.push(cash);
        } else {
            on_formula.push(cash);
        }
    }
    assert_eq!(guaranteed.len(), 294);
    assert_eq!(on_formula.len(), 314);

    let held = median(guaranteed);
    let other = median(on_formula);
    assert!((held - 0.319).abs() < 0.005, "guaranteed {held:.3}");
    assert!((other - 0.338).abs() < 0.005, "formula {other:.3}");
    // The difference is about a week of spending, in the direction opposite to the claim.
    assert!(
        held < other,
        "guaranteed districts hold {held:.3} years of spending against {other:.3} on formula"
    );
    assert!(
        (other - held) < 0.05,
        "and the gap is small: {:.3}",
        other - held
    );
}

#[test]
fn guaranteed_districts_are_below_their_own_fy2020_receipts_and_formula_districts_are_above() {
    // The guarantee holds a district at FY2020. FY2020 is now an observation rather than an
    // inference, so the claim is checkable for the first time — with the caveat in the module
    // note that booked aid and formula output are differently constructed.
    //
    // That caveat is exactly why the control matters. A definitional gap between the two
    // measures would move both groups together. It does not: the median guaranteed district
    // receives less in FY2027 than it booked in FY2020, in nominal dollars, while the median
    // formula district receives a fifth more.
    let money = finances();
    let mut guaranteed = Vec::new();
    let mut on_formula = Vec::new();
    for record in panel() {
        let Some(finances) = for_district(&money, &record.irn) else {
            continue;
        };
        let Some(baseline) = finances.guarantee_baseline_aid().filter(|b| *b > 0.0) else {
            continue;
        };
        let ratio = record.realized_aid() / baseline;
        if record.guarantee > 0.0 {
            guaranteed.push(ratio);
        } else {
            on_formula.push(ratio);
        }
    }

    let held = median(guaranteed.clone());
    let other = median(on_formula.clone());
    assert!((held - 0.899).abs() < 0.01, "guaranteed {held:.3}");
    assert!((other - 1.186).abs() < 0.01, "formula {other:.3}");
    assert!(
        held < 1.0 && other > 1.0,
        "the groups fall on opposite sides of parity"
    );

    let below = |v: &[f64]| v.iter().filter(|r| **r < 1.0).count();
    assert_eq!(below(&guaranteed), 244, "of {}", guaranteed.len());
    assert_eq!(below(&on_formula), 65, "of {}", on_formula.len());

    // And this is before inflation. In real terms the guaranteed group is far further down —
    // which is what "held at FY2020" means once a price index is applied to it.
}

#[test]
fn the_pandemic_years_are_declared_so_a_reader_cannot_miss_them() {
    // Every claim above about FY2020 to FY2024 sits inside this window, and the module says so
    // rather than leaving it to a footnote nobody reads.
    let window = Finances::pandemic_years();
    assert!(window.contains(&FiscalYear(2021)));
    assert!(window.contains(&FiscalYear(2024)));
    assert!(!window.contains(&FiscalYear(2025)));
}
