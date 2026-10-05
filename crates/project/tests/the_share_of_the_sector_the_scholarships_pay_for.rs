//! What share of the chartered nonpublic sector the scholarships pay for, FY2014 through FY2025.
//!
//! The October count of chartered nonpublic pupils is the auxiliary services rate's denominator,
//! and the scholarship channel grew roughly sixfold over the same years. No node said how much of
//! that sector the channel had become. These tests hold the answer as an interval a year, from
//! the 2025 report's participation charts read off their drawing, over the October the rate
//! divides by.
//!
//! # Why the charts are believed
//!
//! The first three tests are the calibration. A drawn value is trusted to a tenth of a point of
//! drawing because that is the tolerance under which every value the charts share with a printed
//! source reproduces: the archive through FY2013, the consolidated report for FY2025, and the
//! three Jon Peterson editions. Doing that also shows what two of the charts drew before FY2014,
//! which is not what their titles say.
//!
//! # What the ratio is not
//!
//! A year's participants against one October day. The ratio overstates the October share by the
//! pupils who held a scholarship that year but were not on the October roster, and no source
//! publishes how many those were. See [`project::scholarship::participation`].

use edfund_core::FiscalYear;
use project::ledger::nonpublic_enrolment;
use project::scholarship::history::{self, Measure};
use project::scholarship::participation::{self, least_school_bound_growth, series, share, shares};
use project::scholarship::{jpsn, report};

/// The programme, and the archive measure its chart reproduces through FY2013.
const ARCHIVE_SPLICE: [(&str, Measure); 3] = [
    ("traditional-edchoice", Measure::Applications),
    ("cleveland", Measure::Applications),
    ("autism", Measure::Used),
];

#[test]
fn two_participant_charts_draw_the_archives_applications_and_autism_draws_its_used() {
    let mut checked = 0;
    for (program, measure) in ARCHIVE_SPLICE {
        let drawn = series(program);
        let archive = history::series(program, measure);
        assert!(
            !archive.is_empty(),
            "{program}: no archive rows on {measure:?}"
        );
        for (year, counts) in archive {
            let printed = counts.total.expect("the archive prints a total");
            let point = &drawn[&year];
            assert!(
                point.admits(printed),
                "{program} FY{}: drawn {} ± {:.1}, archive {measure:?} {printed}",
                year.0,
                point.value,
                point.tolerance()
            );
            checked += 1;
        }
    }
    // Seven Traditional years, seventeen Cleveland, ten Autism.
    assert_eq!(checked, 34);

    // And the titles' own word is not what was drawn: the used column misses every year, by at
    // least two and a half points of drawing for Traditional and eleven for Cleveland.
    for program in ["traditional-edchoice", "cleveland"] {
        let drawn = series(program);
        for (year, counts) in history::series(program, Measure::Used) {
            let used = counts.total.expect("the archive prints a total");
            let point = &drawn[&year];
            assert!(
                (point.value - used) / point.pupils_per_point > 2.5,
                "{program} FY{}: drawn {} is within 2.5 points of used {used}",
                year.0,
                point.value
            );
        }
    }
}

#[test]
fn every_charts_last_year_is_the_participation_the_report_prints() {
    let year = FiscalYear(report::FISCAL_YEAR);
    let programmes = report::programmes();
    assert_eq!(
        programmes.len(),
        participation::points()
            .iter()
            .filter(|p| p.year == year)
            .count()
    );
    for (program, printed) in programmes {
        let point = &series(&program)[&year];
        assert!(
            point.admits(printed.students),
            "{program}: drawn {} ± {:.1}, report {}",
            point.value,
            point.tolerance(),
            printed.students
        );
    }
}

#[test]
fn the_jon_peterson_applications_chart_draws_its_editions_students() {
    let drawn = series(jpsn::PROGRAM);
    assert!(drawn
        .values()
        .all(|p| p.measure == participation::Measure::Applications));
    let editions = jpsn::editions();
    assert_eq!(editions.len(), 3);
    for (year, edition) in editions {
        let point = &drawn[&year];
        assert!(
            point.admits(edition.students),
            "FY{}: drawn {} ± {:.1}, edition {}",
            year.0,
            point.value,
            point.tolerance(),
            edition.students
        );
    }
}

#[test]
fn the_share_is_an_interval_a_year_from_fy2014() {
    // (fiscal year, floor, ceiling), each to a thousandth and rounded outward.
    const SHARES: [(u16, f64, f64); 12] = [
        (2014, 0.140, 0.174),
        (2015, 0.169, 0.209),
        (2016, 0.197, 0.243),
        (2017, 0.218, 0.270),
        (2018, 0.237, 0.294),
        (2019, 0.251, 0.315),
        (2020, 0.302, 0.371),
        (2021, 0.358, 0.432),
        (2022, 0.393, 0.468),
        (2023, 0.416, 0.495),
        (2024, 0.798, 0.880),
        (2025, 0.844, 0.928),
    ];
    let computed = shares();
    let years: Vec<u16> = computed.iter().map(|s| s.year.0).collect();
    assert_eq!(years, SHARES.map(|(year, _, _)| year));
    for (row, (year, floor, ceiling)) in computed.iter().zip(SHARES) {
        assert!(
            row.share.0 >= floor && row.share.1 <= ceiling,
            "FY{year}: {:?} outside [{floor}, {ceiling}]",
            row.share
        );
        assert!(
            row.share.0 - floor < 0.001 && ceiling - row.share.1 < 0.001,
            "FY{year}: {:?} is not [{floor}, {ceiling}] to a thousandth",
            row.share
        );
    }
}

#[test]
fn the_denominator_is_the_october_the_auxiliary_rate_divides_by() {
    // The compilation where it restates the October. On FY2016's annual total, which sits below
    // its own building sheet, the ceiling would be 28.8% rather than 24.2%; FY2017's annual file
    // states the whole sector as in-state.
    for (year, compilation, annual) in [
        (2016, 171_395.0, 144_512.0),
        (2017, 169_901.0, 171_426.0),
        (2019, 165_511.0, 165_346.0),
    ] {
        let row = share(FiscalYear(year)).expect("the year is computable");
        assert_eq!(row.sector, compilation, "FY{year}");
        assert_eq!(
            nonpublic_enrolment::membership(year),
            Some(annual),
            "FY{year}"
        );
    }
    // FY2014 and FY2015 have only the compilation, and FY2020 onward only the annual file.
    assert_eq!(share(FiscalYear(2014)).expect("FY2014").sector, 174_108.0);
    assert_eq!(share(FiscalYear(2025)).expect("FY2025").sector, 179_693.0);
}

#[test]
fn the_universal_expansion_year_takes_the_floor_past_three_quarters() {
    let fy2023 = share(FiscalYear(2023)).expect("FY2023 is computable");
    let fy2024 = share(FiscalYear(2024)).expect("FY2024 is computable");
    assert!(fy2023.share.1 < 0.5, "{:?}", fy2023.share);
    assert!(fy2024.share.0 > 0.75, "{:?}", fy2024.share);
    // Even the ceiling of FY2023 is below FY2024's floor by more than thirty points.
    assert!(fy2024.share.0 - fy2023.share.1 > 0.30);
}

#[test]
fn the_scholarships_grew_eleven_times_as_fast_as_the_october_count_they_sit_in() {
    // FY2017 is the first year both the scholarship charts and the rate's October are exact enough
    // to compare; the corpus's rate series starts there for the same reason.
    let (from, to) = (FiscalYear(2017), FiscalYear(2025));
    let sector_growth = nonpublic_enrolment::statutory_membership(to.0).expect("FY2025 is counted")
        - nonpublic_enrolment::statutory_membership(from.0).expect("FY2017 is restated");
    assert_eq!(sector_growth, 9_792.0);

    let least_growth = least_school_bound_growth(from, to).expect("both years are drawn");
    assert!(least_growth > 114_000.0, "{least_growth}");
    assert!(
        least_growth / sector_growth > 11.6,
        "{}",
        least_growth / sector_growth
    );

    // So at least 104,000 of the school-bound scholarships added in those eight years are not
    // matched by any rise in the October count: they went to pupils inside a count that barely
    // moved. The auxiliary rate divides by that count, so the channel's growth reached the
    // denominator at most as the sector's 9,792, under an eleventh of it.
    assert!(least_growth - sector_growth > 104_000.0);
}

#[test]
fn the_pupils_no_scholarship_pays_for_fell_from_over_124_000_to_under_28_000() {
    let fy2017 = share(FiscalYear(2017))
        .expect("FY2017 is computable")
        .unfunded();
    let fy2025 = share(FiscalYear(2025))
        .expect("FY2025 is computable")
        .unfunded();
    assert!(fy2017.0 > 124_000.0, "{fy2017:?}");
    assert!(fy2025.1 < 28_000.0, "{fy2025:?}");
    assert!(fy2025.0 > 12_900.0, "{fy2025:?}");
}
