//! The Energy Information Administration's Midwest diesel series, out of a 2004-vintage workbook.
//!
//! Two columns and a date, which is the whole file. It is here rather than in a reader beside
//! [`crate::cpi`] because the Administration publishes a workbook where the Bureau publishes a
//! flat file, and a workbook has to be walked rather than filtered.
//!
//! # Why the fixture stays monthly
//!
//! Ohio's fiscal year is July to June and every question this series will be asked is a
//! fiscal-year one, so the temptation is to average here and commit twelve numbers. This does
//! not, for the reason the whole workspace is split the way it is: `connect` retrieves and
//! parses, and an average is a computation. A committed monthly series can be re-aggregated by
//! whoever needs a different window — fuel is bought all year, so a July-to-June *mean* is the
//! natural alignment where the deflator's June *point* observation is right for an index — and a
//! committed annual one cannot be taken back apart.

/// Columns of the diesel fixture.
pub const DIESEL_HEADER: &[&str] = &["calendar_year", "month", "dollars_per_gallon"];

/// The workbook's data sheet. The other sheet is a table of contents.
pub const DIESEL_SHEET: &str = "Data 1";

/// Excel's day zero, as a count of days before the Unix epoch.
///
/// Serial 1 is 1 January 1900 and the format carries the 1900-leap-year bug, so the naive
/// origin is 30 December 1899. Every date in this series is 1994 or later — serial 34,408 and
/// up — so the bug's window is nowhere near it and the plain offset is exact.
const EXCEL_EPOCH_DAYS_BEFORE_UNIX: i64 = 25_569;

/// Civil year and month from an Excel date serial.
///
/// Hinnant's `civil_from_days`, which is exact over the whole range and has no branch for leap
/// years. Only the year and month are returned: every observation in this series is the first of
/// a month and the day carries nothing.
#[must_use]
pub fn year_month_from_serial(serial: i64) -> (i64, u32) {
    let days = serial - EXCEL_EPOCH_DAYS_BEFORE_UNIX;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, u32::try_from(month).unwrap_or(1))
}

/// One observation per month, oldest first.
///
/// # What is dropped, and why it is not an error
///
/// The Administration writes a row for every month from the series' start and leaves the value
/// empty where it published none — March 1994 is the first row and has no price, because the
/// weekly survey began part-way through it. A row with no parsable value is skipped rather than
/// rejected: an absent observation is the publisher saying nothing, and turning that into a
/// build failure would mean the series could not be retrieved at all.
///
/// # Errors
///
/// Returns a message if the sheet's header is not the one this was written against, which is
/// the case that must fail loudly: a moved column would otherwise read date serials as prices.
pub fn build_diesel_series(rows: &[Vec<String>]) -> Result<Vec<Vec<String>>, String> {
    let header = rows
        .iter()
        .position(|row| row.first().is_some_and(|cell| cell.trim() == "Date"))
        .ok_or("the diesel sheet has no `Date` header row")?;

    let label = rows[header]
        .get(1)
        .map(|cell| cell.trim())
        .unwrap_or_default();
    // Every part of the name is load-bearing. The Administration publishes gasoline for the same
    // region in the same units — "Midwest All Grades Gasoline (Dollars per Gallon)" — so a check
    // for the region and the units alone passes on the wrong fuel and reads plausible prices
    // that are not diesel.
    if !label.contains("Midwest")
        || !label.contains("Diesel")
        || !label.contains("Dollars per Gallon")
    {
        return Err(format!(
            "the diesel sheet's value column is {label:?}, not the Midwest diesel \
             dollars-per-gallon series this was written against"
        ));
    }

    let mut out = Vec::new();
    for row in rows.iter().skip(header + 1) {
        let Some(serial) = row.first().and_then(|cell| cell.trim().parse::<i64>().ok()) else {
            continue;
        };
        let Some(price) = row.get(1).and_then(|cell| cell.trim().parse::<f64>().ok()) else {
            continue;
        };
        let (year, month) = year_month_from_serial(serial);
        out.push(vec![
            year.to_string(),
            month.to_string(),
            format!("{price:.3}"),
        ]);
    }

    if out.len() < 300 {
        return Err(format!(
            "the diesel series yielded {} observations; it has run monthly since 1994 and \
             should carry over 300",
            out.len()
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed series, for the one claim about it that was published wrong.
    const COMMITTED: &str = include_str!("../../fixtures/midwest-diesel-monthly.csv");

    /// Ohio's motor fuel tax on diesel, R.C. 5735.05(E)(2): "forty-seven cents on each gallon of
    /// motor fuel other than gasoline". School districts claim it back.
    const OHIO_DIESEL_TAX: f64 = 0.47;

    /// A fiscal-year mean, July to June, from the committed monthly series.
    fn fiscal_year_mean(fy: u16) -> f64 {
        let mut sum = 0.0;
        let mut n = 0.0;
        for line in COMMITTED.lines().skip(1) {
            let mut f = line.split(',');
            let (Some(y), Some(m), Some(v)) = (f.next(), f.next(), f.next()) else {
                continue;
            };
            let (Ok(y), Ok(m), Ok(v)) = (y.parse::<u16>(), m.parse::<u32>(), v.parse::<f64>())
            else {
                continue;
            };
            let in_year = (y == fy - 1 && m >= 7) || (y == fy && m <= 6);
            if in_year {
                sum += v;
                n += 1.0;
            }
        }
        sum / n
    }

    /// The tax wedge **widens** a proportional rise. This entry once said it narrowed one.
    ///
    /// The claim matters because it decides which way an elasticity fitted to retail prices is
    /// biased. Subtracting a fixed per-gallon amount from both ends of a rise always increases
    /// the ratio — for `p1 > p0 > w`, `(p1 − w)/(p0 − w) > p1/p0` — so the retail series
    /// *understates* what a district's fuel bill did, and a coefficient taken from it without
    /// the adjustment is too small rather than too large.
    #[test]
    fn the_refundable_tax_widens_the_rise_rather_than_narrowing_it() {
        let (fy25, fy26) = (fiscal_year_mean(2025), fiscal_year_mean(2026));
        assert!(
            (3.55..=3.57).contains(&fy25) && (4.17..=4.19).contains(&fy26),
            "FY2025 and FY2026 should average about $3.558 and $4.181; {fy25:.3} and {fy26:.3}"
        );

        let retail = fy26 / fy25 - 1.0;
        let net = (fy26 - OHIO_DIESEL_TAX) / (fy25 - OHIO_DIESEL_TAX) - 1.0;

        assert!(
            (0.174..=0.176).contains(&retail),
            "retail rises about 17.5%; {:.3}",
            retail
        );
        assert!(
            (0.201..=0.203).contains(&net),
            "net of the refundable tax it rises about 20.2%; {:.3}",
            net
        );
        assert!(
            net > retail,
            "the wedge must widen the rise, not narrow it: net {net:.4} against retail              {retail:.4}"
        );
    }

    #[test]
    fn the_epoch_lands_on_the_dates_the_workbook_means() {
        // The first two rows of the published sheet, and one modern anchor.
        assert_eq!(year_month_from_serial(34_408), (1994, 3));
        assert_eq!(year_month_from_serial(34_439), (1994, 4));
        assert_eq!(year_month_from_serial(44_713), (2022, 6));
    }

    #[test]
    fn a_month_the_agency_left_empty_is_skipped_rather_than_failing() {
        let mut rows = vec![
            vec![
                "Date".into(),
                "Midwest No 2 Diesel Retail Prices (Dollars per Gallon)".into(),
            ],
            vec!["34408".into()],
            vec!["34439".into(), "1.087".into()],
        ];
        // Pad to the floor the builder insists on, so this test measures the skip and not it.
        for n in 0..320 {
            rows.push(vec![(34_469 + n * 31).to_string(), "2.000".into()]);
        }
        let built = build_diesel_series(&rows).expect("the sheet is well formed");
        assert_eq!(
            built[0],
            vec!["1994", "4", "1.087"],
            "March carries no value and is dropped"
        );
    }

    #[test]
    fn a_moved_value_column_fails_rather_than_reading_dates_as_prices() {
        let rows = vec![
            vec![
                "Date".into(),
                "Midwest All Grades Gasoline (Dollars per Gallon)".into(),
            ],
            vec!["34439".into(), "1.087".into()],
        ];
        let error = build_diesel_series(&rows).expect_err("the series is not the diesel one");
        assert!(
            error.contains("not the Midwest diesel dollars-per-gallon series"),
            "{error}"
        );
    }
}
