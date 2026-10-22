//! The school district income tax, as Taxation paid it and as the districts booked it.
//!
//! Table SD-2-Q4 (`dispersion::income_tax`) is Taxation's record of what it distributed. Line
//! 1.030 of each district's five-year forecast (`project::finances`) is the district's record of
//! what it received. The two are kept separately, so where they disagree is something to find out
//! rather than assume. See
//! [`school-district-income-tax`](../../../.yidam/corpus/revenue-stream/school-district-income-tax.yml).

use std::collections::BTreeSet;

use dispersion::income_tax::{panel, Status};
use edfund_core::FiscalYear;
use project::finances::finances;

/// Districts whose FY2025 forecast books income tax above zero.
fn booked(fiscal_year: u16) -> BTreeSet<String> {
    finances()
        .into_iter()
        .filter(|district| {
            district
                .year(FiscalYear(fiscal_year))
                .and_then(|year| year.income_tax)
                .is_some_and(|amount| amount > 0.0)
        })
        .map(|district| district.irn)
        .collect()
}

#[test]
fn two_districts_book_an_income_tax_that_taxation_did_not_distribute() {
    let paid: BTreeSet<String> = panel()
        .iter()
        .filter(|row| row.fiscal_year == 2025)
        .map(|row| row.irn.clone())
        .collect();
    let booked = booked(2025);
    assert_eq!(booked.len(), 217);
    let unpaid: Vec<&str> = booked
        .iter()
        .filter(|irn| !paid.contains(*irn))
        .map(String::as_str)
        .collect();
    // Euclid City and Orrville City. Neither is on SD-2 in either year, so what they book on line
    // 1.030 is not a Chapter 5748 tax distributed by Taxation, whatever it is.
    assert_eq!(unpaid, ["043950", "044610"]);
}

#[test]
fn three_districts_with_a_tax_in_force_book_none() {
    let booked = booked(2025);
    let missing: Vec<&str> = panel()
        .iter()
        .filter(|row| row.fiscal_year == 2025 && matches!(row.status, Status::InForce(_)))
        .filter(|row| !booked.contains(&row.irn))
        .map(|row| row.district.as_str())
        .collect();
    // Bluffton EVSD, Northridge LSD and Triway LSD. The expired and merged rows are excluded
    // above, so these three are taxes in force that the forecast does not show on line 1.030.
    assert_eq!(missing, ["Bluffton EVSD", "Northridge LSD", "Triway LSD"]);
}
