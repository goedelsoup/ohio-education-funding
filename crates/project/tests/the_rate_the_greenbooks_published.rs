//! Seven years of the auxiliary services rate LSC published, and which of them the fixtures close.
//!
//! `the_october_the_statute_divides_by` closed one year: FY2025's $913, which took three readings
//! held together — the line net of its College Credit Plus earmark, the October of the school
//! year, and the pupils living in Ohio. Issue #758 asked whether the readings hold in every other
//! year the fixtures reach, and whether a year that does not close says why.
//!
//! The fixtures reach further than the issue supposed in one direction and less far in another.
//! Every greenbook since H.B. 119 prints the line's earmark table, so the numerator is read rather
//! than subtracted from FY2008 on — and the earmark is not H.B. 49's: H.B. 66 earmarked $2,000,000
//! for postsecondary enrollment options inside the line, and College Credit Plus replaced that
//! programme in the same place. LSC states a rate for seven fiscal years, not twenty, all odd: the
//! year before each budget it analyses.
//!
//! Five close. FY2017, FY2021, FY2023 and FY2025 each refute the three alternative readings on
//! their own; FY2019's "roughly" is too coarse to refute the October. The two that do not close,
//! FY2013 and FY2015, are the two with no exact October, and the published figure sits inside the
//! interval the masked building sheet leaves for both.

use edfund_core::FiscalYear;
use project::auxiliary_rate::{self as aux, Rate};
use project::ledger::nonpublic_enrolment as enrolment;
use project::ledger::nonpublic_support as nonpublic;

/// A dollar figure is compared to the cent.
const CENT: f64 = 0.005;

/// The quotient for one year.
fn rate(fiscal_year: u16) -> Rate {
    aux::rates()
        .into_iter()
        .find(|rate| rate.fiscal_year == fiscal_year)
        .unwrap_or_else(|| panic!("FY{fiscal_year} has a remainder and an October"))
}

/// What LSC printed for one year, from the newest analysis that printed it.
fn published(fiscal_year: u16) -> aux::PublishedRate {
    aux::published_rates()
        .into_iter()
        .rfind(|rate| rate.fiscal_year == fiscal_year)
        .unwrap_or_else(|| panic!("LSC printed no rate for FY{fiscal_year}"))
}

#[test]
fn lsc_printed_a_rate_for_seven_years_and_every_one_is_odd() {
    /*
     * Each greenbook states the rate for the year before the biennium it analyses, which is the
     * last year with a number in it when the analysis is written. So the published series has a
     * year every two, and it starts with H.B. 59 because no earlier greenbook states one.
     *
     * The redbook states FY2025's too, as "is $913" against the greenbook's "was $913" — the
     * same figure from the proposal and the enactment, so it was not a forecast.
     */
    let printed: Vec<(&str, u16, f64, bool)> = aux::published_rates()
        .iter()
        .map(|rate| (rate.edition, rate.fiscal_year, rate.rate, rate.roughly))
        .collect();
    assert_eq!(
        printed,
        [
            ("hb59-greenbook", 2013, 710.0, true),
            ("hb64-greenbook", 2015, 787.0, false),
            ("hb49-greenbook", 2017, 867.0, false),
            ("hb166-greenbook", 2019, 900.0, true),
            ("hb110-greenbook", 2021, 927.0, false),
            ("hb33-greenbook", 2023, 927.0, false),
            ("hb96-redbook", 2025, 913.0, false),
            ("hb96-greenbook", 2025, 913.0, false),
        ]
    );
}

#[test]
fn every_greenbook_since_hb_119_prints_what_the_quotient_divides() {
    /*
     * The numerator R.C. 3317.024(E)(2)(d) names is the appropriation "for the implementation of
     * sections 3317.06 and 3317.062", and every enacted budget in the fixture earmarks part of
     * ALI 200511 for something else. LSC prints what is left as its own row, so the reader takes
     * it from there.
     *
     * The issue expected the earmark to begin with H.B. 49. It is older: H.B. 66 earmarked
     * $2,000,000 a year for postsecondary enrollment options, and the first printed table is
     * H.B. 119's. College Credit Plus took the same place in H.B. 64.
     */
    let years: Vec<u16> = aux::rates().iter().map(|rate| rate.fiscal_year).collect();
    assert_eq!(years, (2008..=2026).collect::<Vec<_>>());
    assert_eq!(
        aux::remainders().first().map(|row| row.edition),
        Some("hb119-greenbook")
    );
    assert!(project::greenbook::greenbook("hb66")
        .flat()
        .contains("earmarks $2,000,000 in each fiscal year of this appropriation item"));

    // A remainder is always less than the line it is the remainder of, on the same basis: the
    // catalog's first appropriation for an appropriated column, its actual for a closed one. On
    // any other pairing FY2009 fails — the enacted remainder is above what was spent once the
    // year was cut.
    let claims = nonpublic::claims(nonpublic::AUXILIARY_SERVICES);
    for row in aux::remainders() {
        let line = claims
            .iter()
            .find(|claim| claim.fiscal_year == row.fiscal_year && claim.kind == row.kind)
            .and_then(|claim| claim.amount)
            .expect("the catalog carries the line on the same basis");
        assert!(row.amount < line, "{row:?} is not below {line}");
    }

    // And where both are closed, the two publishers agree to the dollar: the catalog's FY2025
    // actual less the greenbook's earmark is the greenbook's remainder.
    let fy2025 = aux::remainder(2025).expect("FY2025");
    assert_eq!((fy2025.edition, fy2025.kind), ("hb96-greenbook", "actual"));
    let line = nonpublic::latest(nonpublic::AUXILIARY_SERVICES, 2025)
        .and_then(|claim| claim.amount)
        .expect("FY2025");
    assert!((line - nonpublic::COLLEGE_CREDIT_PLUS_FY2025 - fy2025.amount).abs() < CENT);

    // ALI 200492, the separate College Credit Plus line, did not take the earmark's place: it
    // spent $904,000 in FY2025 while the earmark inside 200511 spent $2,739,015.
    let separate = nonpublic::latest("200492", 2025)
        .and_then(|claim| claim.amount)
        .expect("FY2025");
    assert!((separate - 904_000.0).abs() < CENT);
}

#[test]
fn five_of_the_seven_close_on_the_three_readings() {
    /*
     * The pairing #758 asked for. The numerator is the newest greenbook's remainder; the
     * denominator is the in-state count of the school year's own October, taken from the
     * 2014-2019 compilation where it restates one — see `Rate::statutory` for why.
     *
     * A rate printed to the dollar closes when the quotient rounds to it. "Roughly" and "about"
     * are read to the ten dollars.
     */
    let closes = [
        (2017, 867.029_105_184_784),
        (2019, 897.440_290_977_638_9),
        (2021, 927.176_198_782_660_3),
        (2023, 926.523_112_705_296),
        (2025, 913.100_421_274_061_9),
    ];
    for (fiscal_year, quotient) in closes {
        let statutory = rate(fiscal_year).statutory().expect("an exact October");
        assert!(
            (statutory - quotient).abs() < 1e-9,
            "FY{fiscal_year}: {statutory}"
        );
        assert!(
            published(fiscal_year).reproduced_by(statutory),
            "FY{fiscal_year} stopped closing: {statutory}"
        );
    }

    // FY2019, FY2021, FY2023 and FY2025 are closed years in the greenbook that states the rate;
    // FY2017 is not, because H.B. 49's table prints no closed column, and it closes on H.B. 64's
    // appropriation instead. LSC states the rate on what it has.
    assert_eq!(rate(2017).remainder.kind, "appropriation");
    for fiscal_year in [2019, 2021, 2023, 2025] {
        assert_eq!(
            rate(fiscal_year).remainder.kind,
            "actual",
            "FY{fiscal_year}"
        );
    }
}

#[test]
fn each_closing_year_refutes_the_other_three_readings_on_its_own() {
    /*
     * FY2025 ruled in three readings by ruling out their alternatives. Four of the other closing
     * years do the same independently, so the readings are the department's method and not a
     * coincidence of one year:
     *
     *   - over the line *gross* of the earmark, every one misses high, by ten to sixteen
     *     dollars;
     *   - over the October's *whole* total, out-of-state pupils included, every one misses low;
     *   - over the *next* October, the fiscal year's name rather than the school year's, every
     *     one misses.
     *
     * FY2019 cannot join them. LSC printed "roughly $900", and the next October's quotient,
     * $901.45, rounds there too.
     */
    for fiscal_year in [2017, 2021, 2023, 2025] {
        let rate = rate(fiscal_year);
        let printed = published(fiscal_year);
        let statutory = rate.statutory().expect("an exact October");
        let pupils = rate.remainder.amount / statutory;

        let line = nonpublic::latest(nonpublic::AUXILIARY_SERVICES, fiscal_year)
            .and_then(|claim| claim.amount)
            .expect("the catalog carries the line");
        let gross = line / pupils;
        assert!(
            !printed.reproduced_by(gross) && gross > statutory + 10.0,
            "FY{fiscal_year} gross of the earmark: {gross}"
        );

        let october = if fiscal_year == 2017 {
            enrolment::restatement(fiscal_year)
        } else {
            enrolment::october(fiscal_year)
        }
        .expect("the October");
        let whole = rate.remainder.amount / october.published_total.expect("a total");
        assert!(
            !printed.reproduced_by(whole) && whole < statutory,
            "FY{fiscal_year} on the whole total: {whole}"
        );

        let next = rate.remainder.amount
            / enrolment::membership(fiscal_year + 1).expect("the next October");
        assert!(
            !printed.reproduced_by(next),
            "FY{fiscal_year} on the next October: {next}"
        );
    }

    let fy2019 = rate(2019);
    let next = fy2019.remainder.amount / enrolment::membership(2020).expect("October 2019");
    assert!(published(2019).reproduced_by(next), "{next}");
}

#[test]
fn fy2017_closes_on_the_compilation_and_not_on_its_own_file() {
    /*
     * The October 2016 file states 171,426 pupils in Ohio and 1,537 outside it. The 2014-2019
     * compilation states 169,901 in Ohio and 171,426 in all — the annual file's in-state figure
     * as its *total*. `two_of_the_departments_own_files_describe_six_octobers_and_disagree`
     * pinned that coincidence without deciding it.
     *
     * The published rate decides it. Over the compilation's 169,901 the quotient is $867.03 and
     * LSC printed $867; over the annual file's 171,426 it is $859.32, which nobody printed. So
     * the annual file's in-state column is the whole, mislabeled, and the compilation is right.
     */
    let fy2017 = rate(2017);
    let annual = fy2017.on_the_annual_file.expect("the annual file");
    let compilation = fy2017.on_the_compilation.expect("the compilation");
    assert!((annual - 859.316_043_073_979_5).abs() < 1e-9, "{annual}");
    assert!(!published(2017).reproduced_by(annual));
    assert!(published(2017).reproduced_by(compilation));
    assert_eq!(fy2017.statutory(), Some(compilation));

    // October 2015's annual total is the one below its own building sheet's floor. On it the
    // FY2016 quotient would be $980.23; on the compilation it is $826.48, inside the bound.
    let fy2016 = rate(2016);
    let (low, high) = fy2016.bound.expect("a building sheet");
    let on_the_file = fy2016.on_the_annual_file.expect("the annual file");
    assert!(
        (on_the_file - 980.225_462_245_349_8).abs() < 1e-9,
        "{on_the_file}"
    );
    assert!(on_the_file > high, "{on_the_file} against ({low}, {high})");
    let statutory = fy2016.statutory().expect("the compilation");
    assert!(statutory > low && statutory < high, "{statutory}");
}

#[test]
fn the_two_that_do_not_close_fall_inside_their_interval() {
    /*
     * FY2013 and FY2015 are the two published rates with no exact October beneath them. The
     * department published no sector total before October 2015, so all the annual file allows is
     * the interval its masked building sheet leaves — the remainder over the sheet's ceiling and
     * over its floor — and LSC's figure has to sit inside it. Both do.
     *
     * FY2015 has a second witness and it misses by fifty-five cents past the rounding edge: over
     * the compilation's 173,030 the quotient is $787.55, which rounds to $788. LSC called the
     * figure an "average per-pupil amount", in both FY2013 and FY2015, which may be a different
     * statistic from the quotient; the fixtures cannot say.
     */
    let fy2013 = rate(2013);
    let (low, high) = fy2013.bound.expect("a building sheet");
    assert!((low - 636.413_266_010_835_1).abs() < 1e-9, "{low}");
    assert!((high - 825.596_320_777_255_8).abs() < 1e-9, "{high}");
    assert!(fy2013.statutory().is_none(), "no exact October 2012");
    assert!(published(2013).rate > low && published(2013).rate < high);

    let fy2015 = rate(2015);
    let (low, high) = fy2015.bound.expect("a building sheet");
    assert!(published(2015).rate > low && published(2015).rate < high);
    let restated = fy2015.on_the_compilation.expect("the compilation");
    assert!(
        (restated - 787.547_968_560_365_2).abs() < 1e-9,
        "{restated}"
    );
    assert!(
        !published(2015).reproduced_by(restated),
        "FY2015 closed, which would make this test's premise wrong"
    );
}

#[test]
fn per_pupil_the_real_fall_is_deeper_than_the_lines() {
    /*
     * The question the issue ended on. Over FY2017 to FY2025, the first and last years that close
     * on an exact October, the line rose 11.87% in cash and fell 15.04% in constant FY2025
     * dollars. Divided among pupils, the cash rise shrinks to 5.31% and the real fall deepens to
     * 20.02%: the in-state count the quotient divides by grew 5.76% over the same years, from
     * 169,901 to 179,693.
     */
    let (nominal, real) = aux::rate_growth(2017, 2025).expect("both years close");
    assert!((nominal - 0.053_137).abs() < 1e-6, "{nominal}");
    assert!((real - -0.200_241).abs() < 1e-6, "{real}");

    let line = nonpublic::series(nonpublic::AUXILIARY_SERVICES, FiscalYear(2025));
    let window: Vec<_> = [2017, 2025]
        .iter()
        .filter_map(|year| nonpublic::at(&line, *year))
        .collect();
    let (line_nominal, line_real) = nonpublic::growth(&window).expect("both years");
    assert!((line_nominal - 0.118_741).abs() < 1e-6, "{line_nominal}");
    assert!((line_real - -0.150_420).abs() < 1e-6, "{line_real}");
    assert!(real < line_real, "per pupil fell further");

    let pupils = |fiscal_year: u16| {
        let rate = rate(fiscal_year);
        rate.remainder.amount / rate.statutory().expect("an exact October")
    };
    assert!((pupils(2017) - 169_901.0).abs() < 0.5);
    assert!((pupils(2025) - 179_693.0).abs() < 0.5);
}
