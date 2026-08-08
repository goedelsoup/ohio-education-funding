//! The H.B. 920 calculator, checked against the two tables the state publishes.
//!
//! Everything in `millage` was written from the statute. R.C. 319.301 says reduction factors hold
//! a levy's dollar yield roughly constant as valuation rises, and R.C. 319.301(D) says they may
//! not carry a rate below twenty mills. The unit tests in the crate prove the arithmetic does
//! what the doc comments claim. None of them prove it describes Ohio.
//!
//! These do. Two independent tables — the Department of Taxation's Table SD-1 and the Department
//! of Education's District Profile Report — carry, between them, every quantity the crate takes
//! and every quantity it returns, for 606 districts across two tax years. So each of the crate's
//! claims becomes a statement that either survives 1,218 district-years of published data or does
//! not:
//!
//! - [`millage::yield_of`] says a mill is a dollar per thousand of valuation. Multiply SD-1's rate
//!   by SD-1's value and the product has to be SD-1's charge.
//! - `effective_millage` clamps with `.min(voted)`, on the grounds that reduction factors never
//!   raise a rate. If any district's effective rate exceeded its voted rate, that clamp would be
//!   hiding a real phenomenon rather than enforcing a real constraint.
//! - [`millage::floor_for`] returns twenty mills. If the floor were doing what the name suggests,
//!   no district would be under it — but six are, and the reason is the second condition in the
//!   crate's `binds` test rather than an exception to the floor.
//!
//! Cited by `catalog/dot-sd1-school-district-taxes.md` and
//! `catalog/ode-district-profile-report.md`.

use edfund_core::AgencyType;
use millage::{effective_millage, floor_for, yield_of, FloorStatus};

const SD1: &str = include_str!("../../dispersion/fixtures/sd1-district-taxes.csv");
const PROFILE: &str = include_str!("../../dispersion/fixtures/cupp-fy24-district-data.csv");

/// Column indices in the SD-1 fixture.
mod sd1 {
    pub const IRN: usize = 0;
    pub const TAX_YEAR: usize = 3;
    pub const CLASS1_VALUE: usize = 6;
    pub const CLASS2_VALUE: usize = 11;
    pub const CLASS1_CHARGED: usize = 15;
    pub const CLASS2_CHARGED: usize = 16;
    pub const CLASS1_RATE: usize = 20;
    pub const CLASS2_RATE: usize = 21;
}

/// Column indices in the District Profile Report fixture.
mod profile {
    pub const IRN: usize = 0;
    /// Total current operating millage — the rate voters approved.
    pub const VOTED: usize = 5;
    /// Effective Class I operating millage — the rate anyone pays.
    pub const EFFECTIVE: usize = 6;
}

fn rows(csv: &str) -> impl Iterator<Item = Vec<&str>> {
    csv.lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split(',').collect())
}

fn number(parts: &[&str], column: usize) -> Option<f64> {
    parts.get(column)?.trim().parse().ok()
}

/// Every SD-1 row as (irn, tax year, class I value, class II value, charges, rates).
struct TaxRow {
    irn: String,
    tax_year: u16,
    class1_value: f64,
    class2_value: f64,
    class1_charged: f64,
    class2_charged: f64,
    class1_rate: f64,
    class2_rate: f64,
}

fn tax_rows() -> Vec<TaxRow> {
    rows(SD1)
        .filter_map(|parts| {
            Some(TaxRow {
                irn: parts.get(sd1::IRN)?.trim().to_string(),
                tax_year: parts.get(sd1::TAX_YEAR)?.trim().parse().ok()?,
                class1_value: number(&parts, sd1::CLASS1_VALUE)?,
                class2_value: number(&parts, sd1::CLASS2_VALUE)?,
                class1_charged: number(&parts, sd1::CLASS1_CHARGED)?,
                class2_charged: number(&parts, sd1::CLASS2_CHARGED)?,
                class1_rate: number(&parts, sd1::CLASS1_RATE)?,
                class2_rate: number(&parts, sd1::CLASS2_RATE)?,
            })
        })
        .collect()
}

/// Voted and effective millage from the profile report, keyed by IRN.
fn voted_and_effective() -> Vec<(String, f64, f64)> {
    rows(PROFILE)
        .filter_map(|parts| {
            Some((
                parts.get(profile::IRN)?.trim().to_string(),
                number(&parts, profile::VOTED)?,
                number(&parts, profile::EFFECTIVE)?,
            ))
        })
        .collect()
}

/// A mill is a dollar per thousand dollars of valuation, and the Department of Taxation agrees.
///
/// `yield_of` is four tokens long and looks too simple to be worth testing. What it is really
/// asserting is that SD-1's rate column and its value column are the two things whose product is
/// its charge column — that the rate is *effective* rather than voted, that the value is the
/// taxable base rather than the market one, and that the charge covers current expenses only.
/// Any of those being false would show up here, and each of them is an assumption the site's
/// property tax pages rest on.
///
/// The tolerance is five hundredths of a percent because SD-1 rounds rates to four decimals and
/// charges to the dollar. On a base of a few hundred million, a rate rounded in the fourth place
/// moves the product by more than a dollar.
#[test]
fn one_mill_raises_a_dollar_per_thousand_in_every_district_and_year() {
    let rows = tax_rows();
    assert!(
        rows.len() >= 1_200,
        "expected both tax years: {}",
        rows.len()
    );

    let mut checked = 0;
    let mut worst: Option<(String, u16, f64)> = None;

    for row in &rows {
        for (value, rate, charged, class) in [
            (row.class1_value, row.class1_rate, row.class1_charged, "I"),
            (row.class2_value, row.class2_rate, row.class2_charged, "II"),
        ] {
            if value <= 0.0 || charged <= 0.0 {
                continue;
            }
            checked += 1;
            let relative = (yield_of(rate, value) - charged).abs() / charged;
            assert!(
                relative < 0.0005,
                "class {class} in {} for TY{}: yield_of gives {:.0} against a published charge \
                 of {charged:.0}, off by {:.4}%",
                row.irn,
                row.tax_year,
                yield_of(rate, value),
                relative * 100.0
            );
            if worst.as_ref().is_none_or(|w| relative > w.2) {
                worst = Some((row.irn.clone(), row.tax_year, relative));
            }
        }
    }

    assert!(checked >= 2_400, "expected both classes: {checked}");
    let (irn, year, off) = worst.expect("at least one row");
    assert!(
        off < 0.0005,
        "worst case {irn} TY{year} at {:.4}%",
        off * 100.0
    );
}

/// Reduction factors never raise a rate, so `.min(voted)` guards a real constraint.
///
/// The clamp inside `effective_millage` would be unreachable defensive code if the statute were
/// self-enforcing. It is worth keeping because it is the invariant that makes a *reduction*
/// factor a reduction, and because the published data offers a way to check that no district is
/// an exception. None is.
#[test]
fn no_district_has_an_effective_rate_above_its_voted_rate() {
    let districts = voted_and_effective();
    assert_eq!(districts.len(), 606, "the profile report covers 606");

    for (irn, voted, effective) in &districts {
        assert!(
            *effective <= voted + 0.001,
            "{irn} has an effective rate of {effective} against a voted rate of {voted}, which \
             reduction factors cannot produce"
        );
    }
}

/// The six districts the floor does not reach, and why that is the statute rather than a gap.
///
/// `effective_millage` binds the floor only when `voted >= floor`. That second condition is easy
/// to read as belt and braces; it is not. Six Ohio districts have never voted twenty mills of
/// current operating levy, and for every one of them the profile report publishes a voted rate
/// and an effective rate that are the same number. There is no reduction to make, and a floor
/// that clamped them up to twenty would invent revenue that nobody levied.
///
/// This is also the bug the bundle's `at_millage_floor` carried: it tested `(m - 20.0).abs() <
/// 0.005`, so a district charging 18.70 mills was reported as above the floor with reduction
/// factors operative.
#[test]
fn districts_that_never_voted_twenty_mills_are_not_pushed_up_to_it() {
    let floor = floor_for(AgencyType::City).expect("school districts have one");
    let below: Vec<_> = voted_and_effective()
        .into_iter()
        .filter(|(_, voted, _)| *voted < floor - 0.005)
        .collect();

    assert_eq!(
        below.len(),
        6,
        "expected the six sub-twenty-mill districts, found {:?}",
        below.iter().map(|d| &d.0).collect::<Vec<_>>()
    );

    for (irn, voted, effective) in &below {
        assert!(
            (voted - effective).abs() < 0.002,
            "{irn} voted {voted} and collects {effective}: with nothing to reduce, they match"
        );

        // The crate agrees, at any valuation growth: there is no reduction to make.
        let result = effective_millage(*voted, 1_000_000.0, 2_000_000.0, AgencyType::City)
            .expect("positive valuations");
        assert!(
            (result.effective - voted).abs() < 1e-9,
            "{irn}: doubling valuation must not change a rate the floor does not govern"
        );
        assert_eq!(result.status, FloorStatus::AtFloor);
        assert!(
            result.status.valuation_growth_reaches_revenue(),
            "{irn}: no reduction is operating, so a reappraisal is a revenue event here too"
        );
    }
}

/// Year over year, the recursion the bundle uses is the crate's own function.
///
/// The bundle predicts a district's TY2024 rate by scaling its TY2023 rate by the change in Class
/// I value. That is `effective_millage` with the prior effective rate standing in for the voted
/// one — legitimate, because the prior effective rate already embeds every reduction before it,
/// but worth pinning so the two cannot drift apart.
///
/// What the assertion below is really about is the residual. Reduction factors reach neither new
/// construction nor newly voted millage, so a prediction built only from them should undershoot
/// far more often than it overshoots. It does: across the districts whose rate moved at all, the
/// observed rate is at or above the prediction roughly nine times in ten.
#[test]
fn reduction_factors_alone_predict_a_rate_no_higher_than_the_one_charged() {
    let rows = tax_rows();
    let mut by_irn: std::collections::BTreeMap<String, Vec<&TaxRow>> =
        std::collections::BTreeMap::new();
    for row in &rows {
        by_irn.entry(row.irn.clone()).or_default().push(row);
    }

    let (mut over, mut under, mut exact) = (0_usize, 0_usize, 0_usize);

    for years in by_irn.values() {
        let [before, after] = years.as_slice() else {
            continue;
        };
        if before.class1_value <= 0.0 || after.class1_value <= 0.0 {
            continue;
        }
        let predicted = effective_millage(
            before.class1_rate,
            before.class1_value,
            after.class1_value,
            AgencyType::City,
        )
        .expect("positive valuations")
        .effective;

        let residual = after.class1_rate - predicted;
        if residual > 0.01 {
            under += 1;
        } else if residual < -0.01 {
            over += 1;
        } else {
            exact += 1;
        }
    }

    let total = over + under + exact;
    assert!(
        total >= 600,
        "expected two years for every district: {total}"
    );
    assert!(
        under + exact >= total * 9 / 10,
        "reduction factors alone should undershoot or land exactly, not overshoot: \
         {under} under, {exact} exact, {over} over, of {total}"
    );
    assert!(
        over > 0,
        "some districts must overshoot — a levy that expires falls faster than the factors alone"
    );
}
