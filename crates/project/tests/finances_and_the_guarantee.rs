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
fn the_guarantee_base_is_narrower_than_a_district_s_booked_state_receipts() {
    // The correction this phase exists for.
    //
    // Two phases reported that the median guaranteed district receives 89.9% of what it booked
    // in FY2020, and called the shortfall a finding. It was a construction mismatch. Realized aid
    // is core foundation funding plus the guarantee — the base the guarantee holds a district at
    // — and it excludes transportation, preschool special education, special education
    // transportation, and the performance supplement. Booked unrestricted grants-in-aid includes
    // them. A narrow FY2027 numerator over a broad FY2020 denominator manufactures a shortfall.
    //
    // With the numerator widened to what the district is actually paid, the ratio crosses one:
    // the observed FY2020 figure sits *between* the two FY2027 constructions, which is what a
    // definitional gap looks like rather than a funding one.
    let money = finances();
    let mut held = Vec::new();
    let mut paid = Vec::new();
    for record in panel() {
        if record.guarantee <= 0.0 {
            continue;
        }
        let Some(district) = for_district(&money, &record.irn) else {
            continue;
        };
        let Some(observed) = district.guarantee_baseline_aid().filter(|b| *b > 0.0) else {
            continue;
        };
        held.push(record.realized_aid() / observed);
        paid.push(record.total_state_support / observed);
    }
    assert_eq!(held.len(), 294);

    let narrow = median(held);
    let wide = median(paid);
    assert!(
        (narrow - 0.899).abs() < 0.01,
        "core + guarantee {narrow:.3}"
    );
    assert!((wide - 1.107).abs() < 0.01, "total state support {wide:.3}");
    assert!(
        narrow < 1.0 && wide > 1.0,
        "the observation must fall between the two constructions, or the gap is not definitional"
    );
}

#[test]
fn the_guarantee_gap_against_formula_districts_survives_the_correction() {
    // What the construction mismatch did *not* invalidate. The level was wrong on both
    // constructions; the comparison between two groups measured the same way was not, and it is
    // the same size either way — about 25 points.
    //
    // This is the argument for always stating a ratio against a control rather than alone. The
    // control absorbed an error in the numerator that the level could not.
    let money = finances();
    let mut guaranteed = Vec::new();
    let mut on_formula = Vec::new();
    for record in panel() {
        let Some(district) = for_district(&money, &record.irn) else {
            continue;
        };
        let Some(observed) = district.guarantee_baseline_aid().filter(|b| *b > 0.0) else {
            continue;
        };
        let ratio = record.total_state_support / observed;
        if record.guarantee > 0.0 {
            guaranteed.push(ratio);
        } else {
            on_formula.push(ratio);
        }
    }
    let held = median(guaranteed);
    let other = median(on_formula);
    assert!((held - 1.107).abs() < 0.01, "guaranteed {held:.3}");
    assert!((other - 1.354).abs() < 0.01, "formula {other:.3}");
    assert!(
        other - held > 0.20,
        "the gap between the groups is {:.3}, and it was 0.287 on the narrow construction",
        other - held
    );
}

#[test]
fn the_panel_spans_the_sharpest_price_change_in_forty_years() {
    // The fact that makes every nominal statement above provisional. A district whose state aid
    // rose 5% across this span lost a fifth of its purchasing power, and a reader given only the
    // nominal figure would read a gain.
    let cpi = deflate::CpiSeries::cpi_u_june();
    let growth = cpi
        .index_growth(FiscalYear(2020), FiscalYear(2025))
        .expect("both endpoints are in the committed series");
    assert!(
        (growth - 0.2512).abs() < 0.001,
        "CPI-U June growth {growth:.4}"
    );
    assert!(cpi.label().contains("CPI-U"), "the index must name itself");
}

#[test]
fn statewide_cash_did_not_rise_in_real_terms_and_ended_a_fifth_below_where_it_started() {
    // The headline of this phase, corrected. Nominally the balance rose from $8.37bn to a peak
    // of $11.20bn and fell to $9.14bn — a story about a build-up and a drawdown that still ends
    // above where it began. In FY2020 dollars it ends **below**.
    let cpi = deflate::CpiSeries::cpi_u_june();
    let real = |year: u16| {
        cpi.convert(
            statewide(year, |y| y.ending_cash),
            FiscalYear(year),
            FiscalYear(2020),
        )
        .expect("covered")
        .value
            / 1e9
    };
    let start = real(2020);
    let peak = real(2024);
    let end = real(2025);

    assert!((start - 8.37).abs() < 0.02, "FY2020 ${start:.2}bn");
    assert!(peak > start, "the build-up is real as well as nominal");
    assert!(
        end < start,
        "FY2025 real cash ${end:.2}bn against FY2020 ${start:.2}bn — nominally it is higher"
    );
    let fall = 1.0 - end / start;
    assert!(
        fall > 0.10 && fall < 0.20,
        "real cash ended {fall:.3} below where it started"
    );
}

#[test]
fn real_state_aid_fell_for_most_districts_over_the_observed_span() {
    // Fully observed: FY2020 and FY2025 are both actuals and both endpoints are in the price
    // series, so this needs no assumption about a future index. Nominal state aid rose for most
    // districts across these six years. Real state aid did not.
    let cpi = deflate::CpiSeries::cpi_u_june();
    let money = finances();

    let mut nominal_up = 0;
    let mut real_up = 0;
    let mut changes = Vec::new();
    for district in &money {
        let (Some(first), Some(last)) = (
            district.year(FiscalYear(2020)),
            district.year(FiscalYear(2025)),
        ) else {
            continue;
        };
        if first.unrestricted_aid <= 0.0 {
            continue;
        }
        if last.unrestricted_aid > first.unrestricted_aid {
            nominal_up += 1;
        }
        let real = district
            .real_change(&cpi, |y| y.unrestricted_aid)
            .expect("covered")
            .expect("positive base");
        if real > 0.0 {
            real_up += 1;
        }
        changes.push(real);
    }

    let typical = median(changes.clone());
    assert!(
        nominal_up > changes.len() / 2,
        "{nominal_up} of {} rose nominally",
        changes.len()
    );
    assert!(
        real_up < nominal_up,
        "deflating cannot turn a fall into a rise: {real_up} against {nominal_up}"
    );
    assert!(
        typical < 0.0,
        "the median district's real state aid changed by {typical:.3}"
    );
}

#[test]
fn the_guarantee_erodes_because_it_is_a_nominal_floor() {
    // What "held at FY2020" means once a price index is applied to it. The guarantee is written
    // in dollars and dollars do not hold their value, so a district it protects loses ground
    // every year by construction — not through any decision, and without anything in the
    // formula recording that it happened.
    //
    // Stated over the observed span so it rests on no assumption about a future index: the
    // FY2027 comparison in the sibling test cannot be deflated, because June 2027 has not
    // happened.
    let cpi = deflate::CpiSeries::cpi_u_june();
    let money = finances();
    let mut guaranteed = Vec::new();
    let mut on_formula = Vec::new();
    for record in panel() {
        let Some(district) = for_district(&money, &record.irn) else {
            continue;
        };
        let Ok(Some(real)) = district.real_change(&cpi, |y| y.unrestricted_aid) else {
            continue;
        };
        if record.guarantee > 0.0 {
            guaranteed.push(real);
        } else {
            on_formula.push(real);
        }
    }

    let held = median(guaranteed.clone());
    let other = median(on_formula.clone());
    assert!(
        held < other,
        "guaranteed districts fared {held:.3} against {other:.3} on formula"
    );
    assert!(
        held < 0.0,
        "the median guaranteed district's real state aid changed {held:.3}"
    );
    let losing = guaranteed.iter().filter(|r| **r < 0.0).count();
    assert!(
        losing * 4 > guaranteed.len() * 3,
        "only {losing} of {} guaranteed districts lost ground in real terms",
        guaranteed.len()
    );
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

/// Local capacity, published rather than recovered — and the two agree.
///
/// [`DistrictRecord::implied_local_capacity_per_pupil`] recovers the measure by subtracting state
/// aid from base cost. That is exact where it works and impossible where the minimum state share
/// binds, which is precisely where capacity is highest and the subtraction would understate it.
/// The corpus returned `None` for those 138 districts rather than a plausible wrong number, and
/// recorded the gap as the reason `local-capacity` could not be exposed.
///
/// It never needed recovering. `Detail_SFPR` carries `[b1] Per Pupil Capacity Amount` for every
/// district, and this repository did not read that sheet for eleven phases. This test is both the
/// cross-check — the two methods agree to half a percent where both exist — and the record that
/// the censored 138 are censored no longer.
#[test]
fn the_published_capacity_agrees_with_the_recovered_one_and_covers_the_rest() {
    let panel = panel();

    let mut both = 0;
    let mut uncensored = 0;
    let mut missing = 0;
    let mut worst = 0.0_f64;

    for record in &panel {
        match (
            record.implied_local_capacity_per_pupil(),
            record.published_capacity_per_pupil,
        ) {
            (Some(implied), Some(published)) => {
                both += 1;
                worst = worst.max((implied - published).abs() / published);
            }
            (None, Some(_)) => uncensored += 1,
            _ => missing += 1,
        }
    }

    assert_eq!(
        missing, 0,
        "every district should now carry a capacity figure"
    );
    assert!(
        both > 450,
        "the subtraction should still work where it worked: {both}"
    );
    assert!(
        uncensored > 100,
        "and the published figure should cover the districts it could not reach: {uncensored}"
    );
    assert!(
        worst < 0.01,
        "the two methods disagree by {:.3}% at worst, which is too much to call them the same \
         quantity",
        worst * 100.0
    );

    // The state share the department publishes should reconcile with the one the corpus derives
    // from base cost, on the districts where both are meaningful.
    for record in &panel {
        let (Some(share), Some(capacity)) = (
            record.published_state_share,
            record.published_capacity_per_pupil,
        ) else {
            continue;
        };
        assert!(
            (0.0..=1.0).contains(&share),
            "{}: share {share}",
            record.name
        );
        // Capacity plus the state's share of base cost is base cost, by construction.
        let implied_share = 1.0 - capacity / record.base_cost_per_pupil;
        if implied_share > project::panel::MINIMUM_STATE_SHARE + 0.01 {
            assert!(
                (implied_share - share).abs() < 0.01,
                "{}: published share {share:.4} against {implied_share:.4} implied by capacity",
                record.name
            );
        }
    }
}
