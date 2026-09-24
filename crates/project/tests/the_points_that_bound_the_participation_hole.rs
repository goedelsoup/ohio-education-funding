//! The FY2014-FY2023 participation hole is bounded by quoted points, and this is the guard that
//! keeps them from being mistaken for a series.
//!
//! Two fixtures cover the scholarship channel and they do not meet. `scholarship-history.csv` is
//! the department's archive, FY1997 through FY2013, and it publishes two denominators;
//! `scholarship-programs.csv` is the 2024-25 annual report, and it publishes one. Ten fiscal years
//! sit between them with no participation series at all, and
//! `.yidam/decisions/scholarship-reports-connector.yml` recorded the shape of the problem: the hole
//! "is not empty", because LSC's budget analyses quote counts inside it, but those counts "bound
//! the hole and cannot be spliced onto the series".
//!
//! That sentence was the whole of the evidence. Issue #472 asked for it to become a fixture, and
//! [`project::scholarship::bounds`] is that fixture: sixty-four quoted quantities from six
//! committed budget analyses, each carrying its denominator, the publisher's hedge, and the
//! publisher's own words. These tests are what make the fixture an argument rather than a list.
//!
//! # What is asserted here
//!
//! - **The transcription is re-checked, not trusted.** Every quote must appear verbatim in the
//!   document its `source` column names, and must contain the row's own value as the publisher
//!   printed it. A fixture built by reading prose is otherwise correct only on the day it is typed.
//! - **No bound is joinable.** [`Bound::spliceable`] is `None` for all sixty-four, and
//!   [`the_splice_guard_is_not_vacuous`] shows the predicate can say otherwise, so the first
//!   assertion is a measurement of the fixture and not of the code.
//! - **The overlap years are the reason.** Five bounds fall inside the archive's own span, where
//!   both a quote and a measured column exist. They agree once exactly and are out by a factor of
//!   nearly seven elsewhere.
//! - **The FY2024 redbook sits below the 2024-25 report on every programme**, in dollars and in
//!   counts — which is what should happen if both documents are right and the channel grew.
//!
//! What is *not* asserted is any reading of the hole itself. Nothing here interpolates, and nothing
//! here should: the point of the census is that the interpolation cannot be justified.

use project::scholarship::bounds::{self, Bound, Denominator, Measure, Precision};
use project::scholarship::{history, report};

use edfund_core::FiscalYear;

/// The hole, as the two committed fixtures leave it.
const HOLE: std::ops::RangeInclusive<u16> = 2014..=2023;

#[test]
fn the_census_is_a_census_and_reaches_across_the_hole() {
    let all = bounds::census();
    assert_eq!(all.len(), 64);

    // Eight documents — six greenbooks of enacted acts, and both editions of the FY2026-27
    // analysis — under nine source values, because the redbook's Table 5 is addressed separately
    // from its prose. The two editions are kept apart for the reason
    // `project::ledger::budget_analysis` exists: they print the same table under the same heading
    // with different numbers in it.
    let mut sources: Vec<&str> = all.iter().map(|b| b.source.as_str()).collect();
    sources.sort_unstable();
    sources.dedup();
    assert_eq!(
        sources,
        [
            "dew-greenbook",
            "dew-redbook",
            "dew-redbook-table-5",
            "hb166-greenbook",
            "hb33-greenbook",
            "hb49-greenbook",
            "hb59-greenbook",
            "hb64-greenbook",
            "hb66-greenbook",
        ]
    );
    let documents = sources
        .iter()
        .map(|s| s.trim_end_matches("-table-5"))
        .collect::<std::collections::BTreeSet<&str>>();
    assert_eq!(documents.len(), 8);

    // Where the bounds fall relative to the two series. The census is worth holding because of the
    // middle number: twenty-two quoted quantities in ten years that no fixture measures.
    let inside_the_archive = all.iter().filter(|b| b.inside_the_archive()).count();
    let inside_the_hole = all.iter().filter(|b| HOLE.contains(&b.year.0)).count();
    let after = all.iter().filter(|b| b.year.0 > *HOLE.end()).count();
    assert_eq!(inside_the_archive, 8);
    assert_eq!(inside_the_hole, 22);
    assert_eq!(after, 34);
    assert_eq!(inside_the_archive + inside_the_hole + after, all.len());

    // And the hole is still a hole: neither committed series has grown into it.
    let archive = history::span();
    assert_eq!(archive, 1997..=2013);
    assert!(
        *archive.end() < *HOLE.start() && report::FISCAL_YEAR > *HOLE.end(),
        "a committed series now reaches into FY{}-FY{}; this census describes a gap that has \
         moved, and the decision record should say what closed it",
        HOLE.start(),
        HOLE.end()
    );
}

/// The guard the issue asked for: no bound is ever joined to either series.
#[test]
fn no_bound_is_joinable_to_either_committed_series() {
    for bound in bounds::census() {
        assert_eq!(
            bound.spliceable(),
            None,
            "FY{} {} {:?} could be appended to a committed series; a bound that reads as an \
             observation must be argued about before it is extracted, not after",
            bound.year.0,
            bound.program,
            bound.measure
        );
    }
}

/// And the guard can fail, which is what makes the test above worth running.
///
/// Two doctored rows, each one field away from a row the fixture holds. If either of these stops
/// being reported as spliceable, [`no_bound_is_joinable_to_either_committed_series`] has become an
/// assertion about nothing.
#[test]
fn the_splice_guard_is_not_vacuous() {
    let real = bounds::of("cleveland", Measure::Participation)
        .into_iter()
        .find(|b| b.year.0 == 2014)
        .expect("the census holds Cleveland's FY2014 count");

    // The archive's own denominator, one year past its last: the splice a reader extending the
    // series would make.
    let onto_the_archive = Bound {
        denominator: Denominator::Paid,
        ..real.clone()
    };
    assert!(onto_the_archive.spliceable().is_some());

    // And the other end: an unhedged student count in the year before the report's.
    let onto_the_report = Bound {
        year: FiscalYear(report::FISCAL_YEAR - 1),
        denominator: Denominator::Students,
        precision: Precision::Exact,
        ..real
    };
    assert!(onto_the_report.spliceable().is_some());
}

/// The archive's two denominators are in the vocabulary and no bound uses either.
///
/// This is the first of the three reasons the census cannot be spliced, and the one that can be
/// stated row by row: not one of these figures counts applications, and not one counts scholarships
/// with a payment against them, which are the only two things the FY1997-FY2013 series measures.
#[test]
fn no_bound_counts_what_the_archive_counts() {
    let all = bounds::census();
    assert!(
        all.iter().all(|b| !b.denominator.is_the_archives()),
        "a bound now names one of the archive's denominators"
    );

    // Seven populations across sixty-four rows, which is the second reason: the denominator moves
    // from row to row, and for two thirds of them the publisher gave none at all.
    let mut used: Vec<Denominator> = all.iter().map(|b| b.denominator).collect();
    used.sort_unstable();
    used.dedup();
    assert_eq!(
        used,
        [
            Denominator::Students,
            Denominator::Fte,
            Denominator::Scholarships,
            Denominator::Kindergarten,
            Denominator::K5,
            Denominator::Operating,
            Denominator::Unstated,
        ]
    );
    let unstated = all
        .iter()
        .filter(|b| b.denominator == Denominator::Unstated)
        .count();
    assert_eq!(unstated, 42);

    // The FY2026-27 redbook counts two programmes in FTE and three in students, in one sequence of
    // paragraphs about one channel. Nothing in the document reconciles them.
    let redbook_counts: Vec<(&str, Denominator)> = all
        .iter()
        .filter(|b| b.source == "dew-redbook" && b.measure == Measure::Participation)
        .map(|b| (b.program.as_str(), b.denominator))
        .collect();
    assert_eq!(
        redbook_counts,
        [
            ("autism", Denominator::Students),
            ("cleveland", Denominator::Students),
            ("edchoice-expansion", Denominator::Fte),
            ("jon-peterson", Denominator::Students),
            ("traditional-edchoice", Denominator::Fte),
        ]
    );
}

/// Most of the census carries the publisher's own hedge, and the hedge is not about rounding.
#[test]
fn three_quarters_of_the_census_is_qualified_or_estimated() {
    let all = bounds::census();
    let hedged = all.iter().filter(|b| b.precision.is_hedged()).count();
    assert_eq!(hedged, 37);
    assert!(hedged * 2 > all.len());

    // "Over 480 chartered nonpublic schools" and "over 10,800 K-5 students" — the only two
    // one-sided bounds, and the only two figures here that constrain in a single direction.
    let one_sided: Vec<&str> = all
        .iter()
        .filter(|b| b.precision == Precision::Over)
        .map(|b| b.program.as_str())
        .collect();
    assert_eq!(one_sided, ["edchoice-combined", "edchoice-expansion"]);
}

/// Every figure is still checkable against the document it came from.
///
/// The decision record's principle was that "a figure reaching the corpus through prose is a figure
/// nothing recomputes". This is the substitute: the quote must be the publisher's, and the value
/// must be inside the quote, on every `cargo test`.
#[test]
fn every_quote_is_verbatim_and_carries_its_own_figure() {
    for bound in bounds::census() {
        let document = bounds::source_text(&bound.source);
        assert!(
            document.contains(&bound.quote),
            "FY{} {} {:?}: {} does not contain {:?}",
            bound.year.0,
            bound.program,
            bound.measure,
            bound.source,
            bound.quote
        );
        let printed = as_printed(&bound);
        assert!(
            bound.quote.contains(&printed),
            "FY{} {} {:?}: the quote {:?} does not carry {printed}",
            bound.year.0,
            bound.program,
            bound.measure,
            bound.quote
        );
    }
}

/// The third reason, and the one no per-row predicate can express: where both a quote and a
/// measured column exist, they disagree, and not by anything that could be corrected for.
///
/// Five rows fall inside the archive's span on a measure the archive publishes. Cleveland's FY2005
/// count reproduces the payments column exactly; its FY2012 count runs a point above; the
/// traditional programme's FY2012 count runs two points *below*; and autism's FY2012 count is 85%
/// below. A transcription that is exact once and out by a factor of seven elsewhere is not a
/// transcription of the same quantity.
#[test]
fn the_overlap_years_show_the_quotes_tracking_nothing() {
    let mut gaps: Vec<(String, u16, f64)> = Vec::new();
    for bound in bounds::census() {
        if !bound.inside_the_archive() {
            continue;
        }
        let Some(measured) = archived(&bound) else {
            continue;
        };
        gaps.push((
            bound.program.clone(),
            bound.year.0,
            bound.value / measured - 1.0,
        ));
    }
    gaps.sort_by(|a, b| a.2.total_cmp(&b.2));

    assert_eq!(gaps.len(), 5, "{gaps:?}");
    let named: Vec<(&str, u16)> = gaps.iter().map(|(p, y, _)| (p.as_str(), *y)).collect();
    assert_eq!(
        named,
        [
            ("autism", 2012),
            ("traditional-edchoice", 2012),
            ("cleveland", 2005),
            ("cleveland", 2005),
            ("cleveland", 2012),
        ]
    );

    // LSC's autism paragraph for FY2012 says 360 students where the department's workbook says
    // 2,413 — the single largest disagreement in the census, and in the same greenbook whose
    // Cleveland and traditional EdChoice counts are within two points of the same workbook.
    assert!((-0.851..-0.850).contains(&gaps[0].2), "{:?}", gaps[0]);
    assert!((-0.023..-0.022).contains(&gaps[1].2), "{:?}", gaps[1]);

    // Cleveland's FY2005 tutoring grants, a point low; its FY2005 scholarships, exact to the
    // student; its FY2012 scholarships, a point high. The signs do not agree and neither do the
    // magnitudes, so there is no adjustment that would make the hole's quotes comparable to the
    // series on either side of it.
    assert!((-0.011..-0.010).contains(&gaps[2].2), "{:?}", gaps[2]);
    assert_eq!(gaps[3].2, 0.0, "{:?}", gaps[3]);
    assert!((0.009..0.010).contains(&gaps[4].2), "{:?}", gaps[4]);
}

/// What the FY2014 bounds do bound: the interval between the archive's own two denominators.
///
/// Every FY2014 count LSC publishes for a programme the archive covers falls strictly between that
/// programme's FY2013 payments and its FY2013 applications. That is the most that can be said — the
/// quoted figure is consistent with either column having grown, and inconsistent with being either
/// one.
#[test]
fn the_fy2014_counts_land_between_the_archives_last_two_columns() {
    let mut bracketed = Vec::new();
    for bound in bounds::census() {
        if bound.year.0 != 2014 || bound.measure != Measure::Participation {
            continue;
        }
        let last = FiscalYear(*history::span().end());
        let paid = history::series(&bound.program, history::Measure::Used)
            .get(&last)
            .and_then(|c| c.total);
        let applied = history::series(&bound.program, history::Measure::Applications)
            .get(&last)
            .and_then(|c| c.total);
        let (Some(paid), Some(applied)) = (paid, applied) else {
            continue;
        };
        assert!(
            paid < bound.value && bound.value < applied,
            "FY2014 {}: {} is outside FY2013's {paid}-{applied}",
            bound.program,
            bound.value
        );
        bracketed.push(bound.program.clone());
    }
    bracketed.sort();
    assert_eq!(
        bracketed,
        ["cleveland", "jon-peterson", "traditional-edchoice"]
    );

    // Autism is the fourth programme with an FY2014 count and the only one whose archive row has a
    // payments column and no applications column, so there is no interval to fall inside. Its
    // 2,623 is one student above FY2013's 2,622 — which reads as a series continuing, and is the
    // most tempting splice in the census.
    let autism = bounds::of("autism", Measure::Participation)
        .into_iter()
        .find(|b| b.year.0 == 2014)
        .expect("the census holds autism's FY2014 count");
    let last_measured = history::series("autism", history::Measure::Used)[&FiscalYear(2013)]
        .total
        .expect("the archive publishes autism's FY2013 payments");
    assert_eq!(autism.value - last_measured, 1.0);
    assert!(
        autism.spliceable().is_none(),
        "the most plausible-looking bound in the census must still not be joinable"
    );
}

/// The issue's third ask: the FY2024 redbook figures sit below the 2024-25 annual report's, which
/// is what should happen if both documents are right and the channel grew.
///
/// Both halves of that — dollars and counts — because the two documents do not measure
/// participation the same way, and the ordering holds anyway.
#[test]
fn the_fy2024_redbook_sits_below_the_annual_report_on_every_programme() {
    let published = report::programmes();
    let mut compared_dollars = Vec::new();
    let mut compared_counts = Vec::new();

    for bound in bounds::census() {
        if bound.source != "dew-redbook" {
            continue;
        }
        assert_eq!(
            bound.year.0, 2024,
            "the redbook's prose is FY2024 throughout"
        );
        let Some(programme) = published.get(&bound.program) else {
            continue;
        };
        match bound.measure {
            Measure::Payments => {
                let Some(spent) = programme.expenditure else {
                    continue;
                };
                assert!(
                    bound.value < spent,
                    "{}: FY2024's ${:.1}m is not below 2024-25's ${:.1}m",
                    bound.program,
                    bound.value / 1e6,
                    spent / 1e6
                );
                compared_dollars.push(bound.program.clone());
            }
            Measure::Participation => {
                assert!(
                    bound.value < programme.students,
                    "{}: FY2024's {} is not below 2024-25's {}",
                    bound.program,
                    bound.value,
                    programme.students
                );
                compared_counts.push(bound.program.clone());
            }
            _ => {}
        }
    }

    compared_dollars.sort();
    compared_counts.sort();

    // Four of five on dollars: Jon Peterson publishes no statewide total in the annual report, and
    // the fixture leaves that column empty rather than summing the six disability categories the
    // report charts.
    assert_eq!(
        compared_dollars,
        [
            "autism",
            "cleveland",
            "edchoice-expansion",
            "traditional-edchoice"
        ]
    );
    assert_eq!(compared_counts.len(), 5);

    // The ordering is not close, and that is worth pinning on its own: the narrowest of the five
    // gaps is the traditional programme's 5.7%, and the widest is autism's 31%. Anything nearer
    // would make the comparison a rounding question rather than a check — and the spread is a
    // reminder that two of these five counts are FTE and three are students, so the gaps are not
    // five measurements of one year of growth.
    let growth: Vec<(&str, f64)> = compared_counts
        .iter()
        .map(|program| {
            let bound = bounds::of(program, Measure::Participation)
                .into_iter()
                .find(|b| b.source == "dew-redbook")
                .expect("each programme has one redbook count");
            (
                program.as_str(),
                published[program].students / bound.value - 1.0,
            )
        })
        .collect();
    let narrowest = growth
        .iter()
        .copied()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("five programmes were compared");
    let widest = growth
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("five programmes were compared");
    assert_eq!(narrowest.0, "traditional-edchoice");
    assert!((0.057..0.058).contains(&narrowest.1), "{growth:?}");
    assert_eq!(widest.0, "autism");
    assert!((0.307..0.308).contains(&widest.1), "{growth:?}");
}

/// Table 5 is an estimate of payments, and it is internally consistent in two of its three years.
///
/// The redbook's own text says the executive proposal "does not allocate specific amounts to each
/// scholarship program", so this table is LSC's estimate and not an appropriation by programme.
/// That is the reason `program/edchoice-expansion`'s open item on appropriation by fiscal period is
/// narrowed by this census rather than closed by it: these figures bind the estimate, and no
/// committed document states the appropriation programme by programme.
#[test]
fn table_five_estimates_the_channel_and_adds_up_in_two_years_of_three() {
    let table: Vec<Bound> = bounds::census()
        .into_iter()
        .filter(|b| b.source == "dew-redbook-table-5")
        .collect();
    assert_eq!(
        table.len(),
        18,
        "five programmes and a total, over three years"
    );
    assert!(
        table
            .iter()
            .all(|b| b.measure == Measure::Payments && b.precision == Precision::Estimated),
        "every cell of Table 5 is an estimated dollar figure"
    );

    let mut residuals = Vec::new();
    for year in 2025..=2027u16 {
        let cells: Vec<&Bound> = table.iter().filter(|b| b.year.0 == year).collect();
        assert_eq!(cells.len(), 6);
        let printed = cells
            .iter()
            .find(|b| b.program == "channel")
            .expect("each year prints a total");
        let summed: f64 = cells
            .iter()
            .filter(|b| b.program != "channel")
            .map(|b| b.value)
            .sum();
        residuals.push((year, printed.value - summed));
    }

    // FY2026 and FY2027 close exactly. FY2025's printed total is $100,000 above the sum of its own
    // column — a rounding residual in a table of tenths of a million, and the reason nothing here
    // recomputes a channel total from the programme rows.
    assert_eq!(residuals, [(2025, 100_000.0), (2026, 0.0), (2027, 0.0)]);
}

/// And the one place Table 5's first year does not sit below the annual report.
///
/// The FY2025 column is an estimate of the year the annual report went on to measure, so four of
/// the five programmes agree with the direction of travel — the estimate below the actual. The
/// traditional programme inverts: $292.7 million estimated against $283.1 million reported. Pinned
/// because it is the one cell in the census that cannot be reconciled with the channel simply
/// having been underestimated.
#[test]
fn table_fives_estimate_of_the_report_year_inverts_for_one_programme() {
    let published = report::programmes();
    let mut below = Vec::new();
    let mut above = Vec::new();

    for bound in bounds::census() {
        if bound.source != "dew-redbook-table-5" || bound.year.0 != report::FISCAL_YEAR {
            continue;
        }
        let Some(spent) = published.get(&bound.program).and_then(|p| p.expenditure) else {
            continue;
        };
        if bound.value < spent {
            below.push(bound.program.clone());
        } else {
            above.push(bound.program.clone());
        }
    }
    below.sort();

    assert_eq!(below, ["autism", "cleveland", "edchoice-expansion"]);
    assert_eq!(above, ["traditional-edchoice"]);
}

/// The archive's measured figure a bound can be held against, where one exists.
///
/// `None` for every measure the archive does not publish — the averages, the school and provider
/// counts, the dollar totals — which is most of the census.
fn archived(bound: &Bound) -> Option<f64> {
    let measure = match bound.measure {
        Measure::Participation => history::Measure::Used,
        Measure::Tutoring => history::Measure::Tutoring,
        _ => return None,
    };
    history::series(&bound.program, measure)
        .get(&bound.year)
        .and_then(|counts| counts.total)
}

/// The bound's value as its publisher printed it, for checking it against the quote.
///
/// Dollar totals are printed in millions to a tenth and everything else in whole units, both with
/// thousands separators — which is LSC's convention throughout, not a choice made here.
fn as_printed(bound: &Bound) -> String {
    match bound.measure {
        Measure::Payments => separated(bound.value / 1e6, 1),
        _ => separated(bound.value, 0),
    }
}

/// A number with thousands separators, to `places` decimals.
fn separated(value: f64, places: usize) -> String {
    let text = format!("{value:.places$}");
    let (whole, rest) = text.split_once('.').unwrap_or((text.as_str(), ""));
    let mut grouped = String::new();
    for (at, digit) in whole.chars().enumerate() {
        if at > 0 && (whole.len() - at) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if places > 0 {
        grouped.push('.');
        grouped.push_str(rest);
    }
    grouped
}
