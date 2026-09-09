//! Ten years the corpus's spending series does not reach, and what they do to its reading.
//!
//! `metric/per-pupil-operating-expenditure` is the corpus's account of what Ohio spends. Its
//! statewide series runs FY2000 to FY2022 off the Auditor's Longitudinal School Finance Study,
//! with the intervening points read off a chart to about $100, and it carries three `[open]`
//! entries about what it cannot see:
//!
//! | the node says | this file answers |
//! |---|---|
//! | real spending "peaked in FY2020, not FY2022"; the FY2020 input is a chart approximation | not on the federal panel: FY2020 and FY2021 are level and FY2024 is the peak |
//! | "the district measurement stops at FY2013 and understates the trough" | it stopped there when it was written; the panel now runs to FY2024 |
//! | per-district series "remain unextracted. They would settle the last of it" | the F-33 panel is a per-district series, and settles most of it |
//!
//! The panel is not the Auditor's series and must not be quoted as one: different survey,
//! narrower entity set — comparable districts only, no community schools, no service centres —
//! and levels about $1,500 apart. What the two share is shape, which is the basis on which
//! `f33_ohio_panel_trough.rs` already reads them together.
//!
//! # The finding the quotient hides
//!
//! Real spending **per pupil** on the panel is 8.9% higher in FY2024 than in FY2010. Real
//! **total** spending is 1.1% *lower*. Enrolment fell 9.2% between them, and that is the whole
//! of the difference — the numerator went slightly the wrong way.
//!
//! The node already warns that per-pupil growth is "partly a numerator effect and partly a
//! denominator one". Over these fifteen years, on this population, it is not partly: it is all
//! denominator. That is a rule the node does not have — name the denominator, not only the
//! window and the index — and it is the same class of error as the weighted-versus-headcount
//! confusion the node was built around.

use dispersion::ohio_panel::{self, trough};

/// Real dollars agree to the cent-per-thousand; these are billions, so name the allowance.
fn near(a: f64, b: f64, allowance: f64) -> bool {
    (a - b).abs() <= allowance
}

/// The panel reaches FY2024, and FY2014 is the only year missing from it.
///
/// When `f33_ohio_panel_trough.rs` was written the panel opened at FY2012 and the archive ended
/// where NCES did. It now runs FY2009 to FY2024 from two publishers on one reconciled
/// definition, with a single gap.
#[test]
fn the_panel_reaches_two_years_the_statewide_series_does_not() {
    let years: Vec<u16> = ohio_panel::spending_by_year().keys().copied().collect();
    assert_eq!(
        years,
        vec![
            2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023,
            2024
        ]
    );
    assert!(
        !years.contains(&2014),
        "FY2014 has arrived and the caveat can go"
    );

    // And the population is one population: the district count barely moves across the window,
    // including across the change of publisher at FY2023.
    for (year, spending) in ohio_panel::spending_by_year() {
        assert!(
            (609..=613).contains(&spending.districts),
            "FY{year} carries {} comparable districts",
            spending.districts
        );
    }
}

/// The real peak is the last year, and the pandemic years are a shelf rather than a summit.
///
/// The node reads a FY2020 real peak off an approximated statewide series and flags the reading
/// `[open]` because the FY2020 input is a chart label. On the panel FY2020 and FY2021 are within
/// $21 of each other, FY2022 is $112 below FY2020, and FY2023 and FY2024 are both above
/// everything before them. The dip the node saw is there; the peak is not.
#[test]
fn the_real_peak_is_the_last_year_and_the_pandemic_years_are_a_shelf() {
    let series = ohio_panel::spending_by_year();
    let peak = series
        .iter()
        .max_by(|a, b| a.1.real_per_pupil.total_cmp(&b.1.real_per_pupil))
        .map(|(year, _)| *year)
        .expect("the panel is not empty");
    assert_eq!(peak, 2024);

    let real = |year: u16| series[&year].real_per_pupil;
    assert!(near(real(2020), 14_681.50, 0.5));
    assert!(near(real(2021), 14_701.86, 0.5));
    assert!(near(real(2022), 14_589.65, 0.5));
    assert!(near(real(2024), 15_284.17, 0.5));

    // FY2021 above FY2020 rather than below it, and by less than a fifth of one per cent.
    assert!(real(2021) > real(2020));
    assert!((real(2021) / real(2020) - 1.0) < 0.002);
    // The FY2022 fall is real and small, and it is the inflation and not the spending.
    assert!((real(2022) / real(2021) - 1.0 + 0.0076).abs() < 0.0005);
    assert!(series[&2022].total > series[&2021].total);
}

/// Every dollar of the per-pupil rise since FY2010 is the denominator.
///
/// Real spending per pupil FY2010 to FY2024: **+8.9%**. Real total spending over the same
/// fifteen years: **−1.1%**. Enrolment: **−9.2%**. The three are the same statement, and only
/// the first of them is in the corpus.
///
/// The population is comparable districts, so part of that enrolment fall is children moving to
/// community schools rather than children not existing — 88,974 pupils outside the comparable
/// set in FY2010 against 134,885 in FY2022, which is about a third of the district loss over the
/// same span. The rest is demographic. Neither reading changes the arithmetic; both change what
/// the arithmetic is *about*.
#[test]
fn every_dollar_of_the_rise_since_the_trough_peak_is_the_denominator() {
    let series = ohio_panel::spending_by_year();
    let (start, end) = (series[&trough::PEAK_YEAR], series[&2024]);

    let per_pupil = end.real_per_pupil / start.real_per_pupil - 1.0;
    let total = end.real_total / start.real_total - 1.0;
    let enrolment = end.enrollment / start.enrollment - 1.0;

    assert!(
        (0.088..0.090).contains(&per_pupil),
        "per pupil {per_pupil:.4}"
    );
    assert!((-0.012..-0.010).contains(&total), "total {total:.4}");
    assert!(
        (-0.093..-0.091).contains(&enrolment),
        "enrolment {enrolment:.4}"
    );

    // The identity, to the fourth decimal: a quotient's growth is its numerator's over its
    // denominator's, so a real total that fell and a per-pupil that rose is arithmetic and not a
    // disagreement between two measurements.
    assert!(((1.0 + total) / (1.0 + enrolment) - (1.0 + per_pupil)).abs() < 1e-4);
}

/// And the pandemic year's rise is a denominator too, more sharply.
///
/// FY2020 to FY2021 the panel's real spending per pupil is flat — up a seventh of one per cent —
/// while its real *total* spending falls 3.3%. Fall membership dropped 54,777 pupils in one
/// year, 3.5%, and did not fully return. A reader taking the per-pupil series alone sees a year
/// in which nothing happened.
#[test]
fn the_pandemic_years_rise_is_a_denominator_too() {
    let series = ohio_panel::spending_by_year();
    let (before, after) = (series[&2020], series[&2021]);

    let per_pupil = after.real_per_pupil / before.real_per_pupil - 1.0;
    let total = after.real_total / before.real_total - 1.0;
    assert!(
        (0.001..0.002).contains(&per_pupil),
        "per pupil {per_pupil:.4}"
    );
    assert!((-0.034..-0.033).contains(&total), "total {total:.4}");

    let lost = before.enrollment - after.enrollment;
    assert!((54_000.0..55_500.0).contains(&lost), "{lost} pupils");
    // Two years later it is still 43,000 short of FY2020.
    assert!(series[&2023].enrollment < before.enrollment - 43_000.0);
}

/// The trough survives ten more years of evidence, and its unobserved year is now bracketed.
///
/// FY2013 is still the minimum of every year the panel holds, and FY2015 — which the earlier
/// reading did not have — is 1.7% above it. So the archive's missing FY2014 no longer sits at
/// the open end of the series; it sits between two observed years. That does not price it, and
/// it is a materially smaller unknown than "the measurement stops here".
#[test]
fn the_trough_survives_ten_more_years_of_evidence() {
    let series = ohio_panel::spending_by_year();
    let bottom = series
        .iter()
        .min_by(|a, b| a.1.real_per_pupil.total_cmp(&b.1.real_per_pupil))
        .map(|(year, _)| *year)
        .expect("the panel is not empty");
    assert_eq!(bottom, trough::BOTTOM_YEAR);

    let recovery = series[&2015].real_per_pupil / series[&trough::BOTTOM_YEAR].real_per_pupil - 1.0;
    assert!(
        (0.017..0.018).contains(&recovery),
        "FY2015 is {recovery:.4} above"
    );

    // And FY2010 is only a local peak: the corpus's `PEAK_YEAR` names the top of the decade the
    // trough sits in, not of the series.
    assert!(series[&2024].real_per_pupil > series[&trough::PEAK_YEAR].real_per_pupil);
}
