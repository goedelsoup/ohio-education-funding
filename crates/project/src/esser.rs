//! The federal relief cliff, read against the fund the five-year forecast actually reports.
//!
//! # The join the corpus asked for
//!
//! `revenue-stream/esser` records what it cannot say: *"Whether districts spent ESSER on
//! recurring or one-time costs, and which ones are now cutting, needs the FY2025 and FY2026
//! five-year forecasts read against the ESSER allocations — both of which exist, and neither of
//! which is joined here."* Both do exist here. The forecasts are [`crate::finances`], six closed
//! fiscal years of every district's general fund; the relief money is the federal column of
//! [`dispersion::ohio_panel`], where it is unmistakable.
//!
//! # Why the general fund is the right place to look, and the only place
//!
//! ESSER was never general-fund money. Ohio districts received it into special revenue funds and
//! the five-year forecast reports the general fund, so the grant itself is invisible to
//! [`crate::finances`] — [`federal_against_the_general_fund`] shows exactly that, and it is the
//! control rather than a limitation.
//!
//! It is what makes the test possible. A district that spent relief money on one-time costs
//! leaves no general-fund trace at all. A district that spent it on recurring costs — staff,
//! programmes, interventions it meant to keep — has to carry those costs somewhere once the
//! grant ends, and the general fund is where they land. So the recurring-cost hypothesis
//! predicts general-fund spending growth that rises with a district's relief exposure, and
//! [`by_exposure`] is that prediction tested.

use std::collections::BTreeMap;

use edfund_core::FiscalYear;

/// The last pre-relief year, the baseline every exposure is measured from.
pub const BASELINE_YEAR: u16 = 2019;

/// The two years the relief peak falls in; a district's peak is the larger of them.
pub const PEAK_YEARS: [u16; 2] = [2022, 2023];

/// The interval general-fund spending growth is measured over — the first closed year after the
/// grants ended, against the last year they were still being spent.
pub const AFTER: [u16; 2] = [2023, 2025];

/// One district's relief exposure and what its general fund did afterwards.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Ohio's identifier.
    pub irn: String,
    /// Federal revenue per pupil at the peak, less the [`BASELINE_YEAR`] figure. The relief
    /// money a district actually received, as the Census survey saw it.
    pub surge_per_pupil: f64,
    /// General-fund spending growth across [`AFTER`], as a share.
    pub spending_growth: f64,
    /// The change in ending cash measured in years of spending, across the same interval.
    /// Negative is a drawdown.
    pub cash_years_change: f64,
}

/// Every district the two sources both reach in every year the comparison needs.
///
/// The Census panel's comparability filter is applied, so joint vocational districts and the
/// other agencies it excludes are out — see [`dispersion::ohio_panel`].
#[must_use]
pub fn districts() -> Vec<District> {
    let mut by_district: BTreeMap<String, BTreeMap<u16, dispersion::ohio_panel::PanelRow>> =
        BTreeMap::new();
    for row in dispersion::ohio_panel::panel() {
        if row.comparable && !row.irn.is_empty() {
            by_district
                .entry(row.irn.clone())
                .or_default()
                .insert(row.fiscal_year, row);
        }
    }

    let finances = crate::finances::finances();
    let mut out = Vec::new();
    for (irn, years) in by_district {
        let Some(baseline) = years.get(&BASELINE_YEAR) else {
            continue;
        };
        if baseline.enrollment <= 0.0 {
            continue;
        }
        let Some(peak) = PEAK_YEARS
            .iter()
            .filter_map(|y| years.get(y))
            .map(|row| row.federal_revenue)
            .reduce(f64::max)
        else {
            continue;
        };
        if PEAK_YEARS.iter().any(|y| !years.contains_key(y)) {
            continue;
        }

        let Some(district) = crate::finances::for_district(&finances, &irn) else {
            continue;
        };
        let (Some(before), Some(after)) = (
            district.year(FiscalYear(AFTER[0])),
            district.year(FiscalYear(AFTER[1])),
        ) else {
            continue;
        };
        let (Some(spent_before), Some(spent_after), Some(cash_before), Some(cash_after)) = (
            before.total_expenditure,
            after.total_expenditure,
            before.ending_cash,
            after.ending_cash,
        ) else {
            continue;
        };
        if spent_before <= 0.0 || spent_after <= 0.0 {
            continue;
        }

        out.push(District {
            irn,
            surge_per_pupil: (peak - baseline.federal_revenue) / baseline.enrollment,
            spending_growth: spent_after / spent_before - 1.0,
            cash_years_change: cash_after / spent_after - cash_before / spent_before,
        });
    }
    out.sort_by(|a, b| a.surge_per_pupil.total_cmp(&b.surge_per_pupil));
    out
}

/// One quartile of districts, ordered by relief exposure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quartile {
    /// Districts in it.
    pub districts: usize,
    /// The median relief surge per pupil.
    pub surge_per_pupil: f64,
    /// The median general-fund spending growth across [`AFTER`].
    pub spending_growth: f64,
    /// The median change in cash measured in years of spending.
    pub cash_years_change: f64,
}

/// The four exposure quartiles, poorest-relieved first.
///
/// If districts moved recurring costs onto the general fund when the grants ended, spending
/// growth rises across these four and the cash drawdown deepens. Neither does.
#[must_use]
pub fn by_exposure() -> [Quartile; 4] {
    let all = districts();
    let size = all.len() / 4;
    std::array::from_fn(|i| {
        let slice = if i < 3 {
            &all[i * size..(i + 1) * size]
        } else {
            &all[3 * size..]
        };
        Quartile {
            districts: slice.len(),
            surge_per_pupil: median(slice.iter().map(|d| d.surge_per_pupil)),
            spending_growth: median(slice.iter().map(|d| d.spending_growth)),
            cash_years_change: median(slice.iter().map(|d| d.cash_years_change)),
        }
    })
}

/// How closely relief exposure and general-fund spending growth move together.
#[must_use]
pub fn exposure_against_spending_growth() -> f64 {
    let all = districts();
    correlation(
        &all.iter().map(|d| d.surge_per_pupil).collect::<Vec<_>>(),
        &all.iter().map(|d| d.spending_growth).collect::<Vec<_>>(),
    )
}

/// One fiscal year of the two aggregates, over the districts [`districts`] admits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YearOfBoth {
    /// Federal revenue as the Census survey reports it, in dollars.
    pub federal_revenue: Option<f64>,
    /// General fund revenue as the districts file it, in dollars.
    pub general_fund_revenue: Option<f64>,
    /// General fund spending, in dollars.
    pub general_fund_spending: Option<f64>,
}

/// The control: the relief bulge beside the fund the forecast reports.
///
/// Federal revenue more than doubles and the general fund does not move, which is what says the
/// grant never passed through it — and therefore that any general-fund signal after the grants
/// ended is a cost a district chose to keep.
#[must_use]
pub fn federal_against_the_general_fund() -> BTreeMap<u16, YearOfBoth> {
    let admitted: std::collections::BTreeSet<String> =
        districts().into_iter().map(|d| d.irn).collect();

    let mut federal: BTreeMap<u16, f64> = BTreeMap::new();
    for row in dispersion::ohio_panel::panel() {
        if admitted.contains(&row.irn) {
            *federal.entry(row.fiscal_year).or_default() += row.federal_revenue;
        }
    }

    let mut revenue: BTreeMap<u16, f64> = BTreeMap::new();
    let mut spending: BTreeMap<u16, f64> = BTreeMap::new();
    for district in crate::finances::finances() {
        if !admitted.contains(&district.irn) {
            continue;
        }
        for year in &district.years {
            if let Some(amount) = year.total_revenue {
                *revenue.entry(year.fiscal_year.0).or_default() += amount;
            }
            if let Some(amount) = year.total_expenditure {
                *spending.entry(year.fiscal_year.0).or_default() += amount;
            }
        }
    }

    let years: std::collections::BTreeSet<u16> = federal
        .keys()
        .chain(revenue.keys())
        .chain(spending.keys())
        .copied()
        .collect();
    years
        .into_iter()
        .map(|year| {
            (
                year,
                YearOfBoth {
                    federal_revenue: federal.get(&year).copied(),
                    general_fund_revenue: revenue.get(&year).copied(),
                    general_fund_spending: spending.get(&year).copied(),
                },
            )
        })
        .collect()
}

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut sample: Vec<f64> = values.collect();
    sample.sort_by(f64::total_cmp);
    sample.get(sample.len() / 2).copied().unwrap_or(f64::NAN)
}

fn correlation(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n < 2.0 {
        return f64::NAN;
    }
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let cov: f64 = xs.iter().zip(ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let sx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>().sqrt();
    let sy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum::<f64>().sqrt();
    cov / (sx * sy)
}
