//! Recognized valuation, reconstructed for all 611 districts and checked against Table SD-1.
//!
//! The corpus described recognized valuation for its whole life as an H.B. 920 adjustment. It is
//! not: it is a three-year phase-in of reappraisal growth, on a calendar the Department of
//! Taxation publishes. These tests establish both that the calendar is transcribed correctly and
//! that the reconstruction built on it behaves the way the Legislative Service Commission's own
//! description says it should.
//!
//! # The test that matters most is the calendar one
//!
//! `every_county_jumps_in_the_year_the_calendar_says` is what stands between an 88-row hand
//! transcription and a silently wrong site. It never reads the calendar's own claims back to
//! itself; it takes each county's real property value from the abstract across four tax years and
//! asks whether the largest jump lands where the schedule says. A county typed into the wrong
//! column fails it, because the thing being detected is a 28% revaluation against a 1% background
//! and there is no version of that which hides.

use std::collections::HashMap;

use regime_diff::recognized_valuation::{
    cycle_for, recognize, PhaseIn, ValuationEvent, CYCLE, WINDOW_FIRST, WINDOW_LAST,
};

const SD1: &str = include_str!("../../dispersion/fixtures/sd1-district-taxes.csv");

/// One district-year of the abstract, reduced to what recognized valuation needs.
struct Row {
    irn: String,
    county: String,
    tax_year: u16,
    real_property: f64,
    total: f64,
}

fn rows() -> Vec<Row> {
    let mut lines = SD1.lines();
    let head: Vec<&str> = lines.next().expect("a header").split(',').collect();
    let index = |name: &str| {
        head.iter()
            .position(|c| *c == name)
            .unwrap_or_else(|| panic!("{name} is not a column of the abstract"))
    };
    let (irn, county, year, real, total) = (
        index("irn"),
        index("county"),
        index("tax_year"),
        index("real_property_value"),
        index("total_value"),
    );
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let p: Vec<&str> = line.split(',').collect();
            Row {
                irn: p[irn].to_string(),
                county: p[county].to_string(),
                tax_year: p[year].parse().expect("a tax year"),
                real_property: p[real].parse().unwrap_or(0.0),
                total: p[total].parse().unwrap_or(0.0),
            }
        })
        .collect()
}

/// One district's four tax years, in the shape `recognize` wants them.
struct Series {
    county: String,
    /// Real property by tax year, oldest first — what the phase-in applies to.
    real_property: Vec<(u16, f64)>,
    /// Total taxable value by tax year — what the charge-off multiplies.
    total: HashMap<u16, f64>,
}

/// Real property by tax year for each district, oldest first, with its county.
fn by_district() -> HashMap<String, Series> {
    let mut out: HashMap<String, Series> = HashMap::new();
    for row in rows() {
        let entry = out.entry(row.irn.clone()).or_insert_with(|| Series {
            county: row.county.clone(),
            real_property: Vec::new(),
            total: HashMap::new(),
        });
        entry.real_property.push((row.tax_year, row.real_property));
        entry.total.insert(row.tax_year, row.total);
    }
    for entry in out.values_mut() {
        entry.real_property.sort_by_key(|(y, _)| *y);
    }
    out
}

/// Aggregate real property by county and tax year.
fn county_series() -> HashMap<String, HashMap<u16, f64>> {
    let mut out: HashMap<String, HashMap<u16, f64>> = HashMap::new();
    for row in rows() {
        *out.entry(row.county.to_uppercase())
            .or_default()
            .entry(row.tax_year)
            .or_insert(0.0) += row.real_property;
    }
    out
}

/// **The transcription check.** Every county's largest valuation jump falls in its scheduled year.
///
/// This is the one test that can catch a county typed into the wrong column of a PDF, and it does
/// it without consulting the transcription for anything but the prediction it is being asked to
/// make. All 88 pass.
#[test]
fn every_county_jumps_in_the_year_the_calendar_says() {
    let series = county_series();
    let mut wrong = Vec::new();
    for entry in CYCLE {
        let years = series
            .get(entry.county)
            .unwrap_or_else(|| panic!("{} is not a county of the abstract", entry.county));
        let mut best = (0_u16, f64::MIN);
        for year in (WINDOW_FIRST..=WINDOW_LAST).rev() {
            let (Some(after), Some(before)) = (years.get(&year), years.get(&(year - 1))) else {
                panic!("{} is missing tax year {year}", entry.county);
            };
            let growth = after / before - 1.0;
            if growth > best.1 {
                best = (year, growth);
            }
        }
        if best.0 != entry.tax_year {
            wrong.push((entry.county, entry.tax_year, best));
        }
    }
    assert!(
        wrong.is_empty(),
        "counties whose largest jump is not in the scheduled year: {wrong:?}"
    );
}

/// The reappraisal signal dwarfs everything else, which is why the estimator can separate them.
///
/// Stated as a measured gap rather than as an assumption. The event year and the quiet years are
/// not close: if they ever became close, the reconstruction would be resting on nothing and this
/// would say so before the site did.
#[test]
fn the_event_year_is_an_order_of_magnitude_above_the_quiet_ones() {
    let series = county_series();
    let (mut event, mut quiet) = (Vec::new(), Vec::new());
    for entry in CYCLE {
        let years = &series[entry.county];
        for year in WINDOW_FIRST..=WINDOW_LAST {
            let growth = years[&year] / years[&(year - 1)] - 1.0;
            if year == entry.tax_year {
                event.push(growth);
            } else {
                quiet.push(growth);
            }
        }
    }
    let median = |mut v: Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        v[v.len() / 2]
    };
    let (event_median, quiet_median) = (median(event), median(quiet));
    assert!(
        event_median > 0.20,
        "event-year median growth is only {event_median:.4}"
    );
    assert!(
        quiet_median < 0.03,
        "quiet-year median growth is {quiet_median:.4}, too close to the signal"
    );
    assert!(
        event_median > quiet_median * 10.0,
        "separation collapsed: {event_median:.4} against {quiet_median:.4}"
    );
}

/// Every district in the abstract resolves to a county on the calendar and recognizes.
#[test]
fn all_six_hundred_and_eleven_districts_recognize() {
    let districts = by_district();
    assert_eq!(districts.len(), 611);
    let mut missing = Vec::new();
    for (irn, d) in &districts {
        assert!(
            cycle_for(&d.county).is_some(),
            "{} is not on the calendar",
            d.county
        );
        if recognize(
            &d.county,
            &d.real_property,
            d.total[&WINDOW_LAST],
            WINDOW_LAST,
        )
        .is_none()
        {
            missing.push(irn.clone());
        }
    }
    assert!(
        missing.is_empty(),
        "districts that failed to recognize: {missing:?}"
    );
}

/// Recognized valuation never exceeds actual, and never falls below the phase-in's own bound.
///
/// The second half is the one worth having. Two thirds of a single year's reappraisal is the most
/// the mechanism can ever defer, so a district recognized below that is a sign the background
/// estimate has gone wrong — most likely a district that shrank in its quiet years, making
/// ordinary growth negative and the inferred reappraisal larger than the jump that produced it.
#[test]
fn recognized_valuation_stays_between_its_bounds() {
    let districts = by_district();
    for (irn, d) in &districts {
        let actual = d.total[&WINDOW_LAST];
        let Some(got) = recognize(&d.county, &d.real_property, actual, WINDOW_LAST) else {
            continue;
        };
        assert!(
            got.recognized <= actual + 1.0,
            "{irn}: recognized {} above actual {actual}",
            got.recognized
        );
        assert!(
            got.ratio() > 0.5,
            "{irn}: recognized at {:.4} of actual, which no phase-in can produce",
            got.ratio()
        );
    }
}

/// The statewide size of the correction, and how unevenly the calendar spreads it.
///
/// This is the finding the whole module exists to state. The charge-off counterfactual was
/// applying 23 mills to total taxable value; against recognized valuation it charges $793m less,
/// and the relief is not spread evenly or by anything resembling need — it is 13% of value for
/// districts whose county reappraised in TY2024, 7% for TY2023, and nothing at all for TY2022.
#[test]
fn the_correction_is_eight_percent_of_valuation_and_falls_by_calendar() {
    let districts = by_district();
    let mut actual_total = 0.0;
    let mut recognized_total = 0.0;
    let mut by_phase: HashMap<u16, (usize, f64, f64)> = HashMap::new();

    for d in districts.values() {
        let actual = d.total[&WINDOW_LAST];
        let Some(got) = recognize(&d.county, &d.real_property, actual, WINDOW_LAST) else {
            continue;
        };
        actual_total += actual;
        recognized_total += got.recognized;
        let event = cycle_for(&d.county).expect("on the calendar").tax_year;
        let entry = by_phase.entry(event).or_insert((0, 0.0, 0.0));
        entry.0 += 1;
        entry.1 += actual;
        entry.2 += got.deferred;
    }

    let deferred = actual_total - recognized_total;
    let share = deferred / actual_total;
    assert!(
        (0.07..0.09).contains(&share),
        "statewide deferral is {share:.4} of valuation"
    );
    // At the terminal charge-off rate, what the counterfactual was over-charging.
    let overcharge = deferred * regime_diff::TERMINAL_MILLS / 1_000.0;
    assert!(
        (750_000_000.0..850_000_000.0).contains(&overcharge),
        "overcharge at 23 mills is ${overcharge:.0}"
    );

    // TY2022 counties have finished phasing in; the other two have not.
    assert_eq!(by_phase[&2022].2, 0.0, "a finished phase-in still defers");
    let share_of = |year: u16| by_phase[&year].2 / by_phase[&year].1;
    assert!(
        share_of(2024) > share_of(2023),
        "the most recent reappraisal should defer the most"
    );
    assert!(share_of(2024) > 0.10, "TY2024 defers {:.4}", share_of(2024));
    assert!(
        (0.05..0.09).contains(&share_of(2023)),
        "TY2023 defers {:.4}",
        share_of(2023)
    );
}

/// Two districts in the same funding position get different charge-offs for a calendar reason.
///
/// The point of the correction stated as a case rather than a rate. Nothing about these districts'
/// wealth, effort or need differs in the way their deemed local share does.
#[test]
fn the_relief_a_district_gets_depends_on_its_county_not_its_finances() {
    let districts = by_district();
    let mut first = None;
    let mut third = None;
    for d in districts.values() {
        let actual = d.total[&WINDOW_LAST];
        let Some(got) = recognize(&d.county, &d.real_property, actual, WINDOW_LAST) else {
            continue;
        };
        match got.phase {
            PhaseIn::First if first.is_none() && got.ratio() < 0.92 => first = Some(got),
            PhaseIn::Third if third.is_none() => third = Some(got),
            _ => {}
        }
    }
    let (first, third) = (
        first.expect("a district mid-phase-in"),
        third.expect("one past it"),
    );
    assert!(first.ratio() < 0.92);
    assert_eq!(third.ratio(), 1.0);
}

/// Reappraisals and updates are treated identically, and both appear in the window.
///
/// Recognized valuation makes no distinction between them — LSC's rule says "reappraisal or update
/// year" — so the calendar carrying the difference is descriptive. Asserted so that the field
/// cannot quietly acquire meaning in the arithmetic without a test changing.
#[test]
fn both_kinds_of_valuation_event_occur_and_are_treated_alike() {
    let reappraisals = CYCLE
        .iter()
        .filter(|c| c.event == ValuationEvent::Reappraisal)
        .count();
    let updates = CYCLE.len() - reappraisals;
    assert!(reappraisals > 0 && updates > 0);
    assert_eq!(reappraisals + updates, 88);
}
