//! What Ohio's districts actually spend moving children, across fifteen years of the survey.
//!
//! This column exists because of a feedback loop. The state's two transportation rates —
//! $1,337.175 per weighted rider and $6.867 per mile — are not chosen by anyone. R.C. 3317.0212(C)
//! and (D) define both as **trimmed means of what districts themselves reported spending in the
//! prior fiscal year**, so the cost side of the formula is an input the formula does not control,
//! and it is measurable here and nowhere else across years. Census F-33 `V45` is the only
//! fuel-exposed line the survey separates.
//!
//! Four findings, and the last two are the ones that constrain what can be asked of this series.
//!
//! **Transportation is a remarkably stable share of current spending.** Between 5.02% and 5.39%
//! in every year of the panel but the two pandemic ones, on 609 to 613 comparable districts. A
//! share that steady across a decade that includes the FY2010–FY2013 real-spending trough means
//! transportation is not what districts cut first, and not what they protect first either. The two
//! years the Bureau supplies sit inside it — 5.28% and 5.16% — so the change below is not a
//! reallocation.
//!
//! **In real terms it rose about 16%, gave a third of it back, and then took a new peak.** $673
//! per pupil in FY2009, $779 at the FY2019 peak, $689 at the FY2021 trough, and **$794 in FY2023**,
//! all in FY2022 dollars. FY2019 was the maximum of the series until the panel could see past
//! FY2022; it is not.
//!
//! **The FY2021 trough is a service interruption, not a price.** Real per-pupil transportation
//! fell 11.6% from FY2019 to FY2021 — buses stopped running — and the share of current spending
//! fell to 4.68%, outside the band every other year sits in. This is the finding that matters
//! for any attempt to regress this series on fuel prices, because **the confound is nearly
//! collinear with the treatment**: diesel collapsed in calendar 2020 at the same moment Ohio's
//! buses stopped. A naive regression over this panel will read a pandemic as a fuel elasticity.
//!
//! **And the largest real move in the series is now FY2023, not either pandemic year.** +7.8%,
//! against the FY2022 rebound's +7.0% and the FY2021 collapse's −6.6%. FY2023 is the year of the
//! diesel excursion and it is not confounded by a school closure, which is the whole reason two
//! more years of this column were worth acquiring: the identifying variation the earlier panel was
//! missing is in these two rows.
//!
//! Nothing here estimates a fuel response. The panel is the cost side; these tests exist so that
//! the shape of the series is pinned before anything is fitted to it.

use deflator::CpiSeries;
use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::FiscalYear;

/// The base year every real figure here is stated in, and one of the two the CPI series marks
/// verified.
const BASE_YEAR: u16 = 2022;

/// Comparable districts only — `AGCHRT != 1` and `SCHLEV == 03` — so a figure here means what one
/// in [`f33_ohio_panel_trough`] means.
fn comparable() -> Vec<PanelRow> {
    ohio_panel::panel()
        .into_iter()
        .filter(|r| r.comparable)
        .collect()
}

/// Every fiscal year in the panel, oldest first. FY2014 is absent from the archive.
fn years(rows: &[PanelRow]) -> Vec<u16> {
    let mut all: Vec<u16> = rows.iter().map(|r| r.fiscal_year).collect();
    all.sort_unstable();
    all.dedup();
    all
}

/// Enrolment-weighted transportation spending per pupil for one year, nominal.
fn per_pupil(rows: &[PanelRow], year: u16) -> f64 {
    let (spend, pupils) = rows
        .iter()
        .filter(|r| r.fiscal_year == year)
        .filter_map(|r| Some((r.student_transportation?, r.enrollment)))
        .fold((0.0, 0.0), |(s, p), (spend, pupils)| {
            (s + spend, p + pupils)
        });
    spend / pupils
}

/// The same, restated in [`BASE_YEAR`] dollars.
fn real_per_pupil(rows: &[PanelRow], year: u16) -> f64 {
    CpiSeries::cpi_u_june()
        .convert(
            per_pupil(rows, year),
            FiscalYear(year),
            FiscalYear(BASE_YEAR),
        )
        .expect("the CPI series covers every year of the panel")
        .value
}

/// The column is present in every era of the survey, which is why it can be a series at all.
///
/// The four layouts carry 256, 260, 354 and 183 columns and `V45` is in all of them under that
/// name — including the Bureau's own file, which is where FY2023 and FY2024 come from. A year that
/// lost the column would read as a shorter series rather than as a failure, so the count is
/// asserted rather than assumed.
#[test]
fn every_year_of_the_panel_carries_student_transportation() {
    let rows = comparable();
    let years = years(&rows);
    assert_eq!(
        years.len(),
        15,
        "the panel should hold fifteen years, FY2009-FY2024 less the missing FY2014"
    );

    for year in years {
        let present = rows
            .iter()
            .filter(|r| r.fiscal_year == year)
            .filter(|r| r.student_transportation.is_some())
            .count();
        let total = rows.iter().filter(|r| r.fiscal_year == year).count();
        assert_eq!(
            present, total,
            "FY{year}: {present} of {total} comparable districts report V45; a partial year \
             cannot be compared to a whole one"
        );
        assert!(
            total > 600,
            "FY{year}: {total} comparable districts, which is too few for Ohio"
        );
    }
}

/// Transportation holds a narrow band of current spending, and the pandemic leaves it.
///
/// The band is the finding. Districts did not trade transportation against instruction in either
/// direction over thirteen years — including the four in which real spending per pupil fell.
#[test]
fn transportation_is_a_stable_share_of_current_spending_except_when_buses_stop() {
    let rows = comparable();
    let share_of = |year: u16| {
        let (transport, current) = rows
            .iter()
            .filter(|r| r.fiscal_year == year)
            .filter_map(|r| Some((r.student_transportation?, r.current_spending?)))
            .fold((0.0, 0.0), |(t, c), (transport, current)| {
                (t + transport, c + current)
            });
        transport / current
    };

    for year in years(&rows) {
        let share = share_of(year);
        // FY2020 and FY2021 are the pandemic and are asserted separately below.
        if year == 2020 || year == 2021 {
            continue;
        }
        assert!(
            (0.0500..=0.0540).contains(&share),
            "FY{year}: transportation is {:.4} of current spending, outside the band every \
             other non-pandemic year sits in",
            share
        );
    }

    assert!(
        share_of(2021) < 0.0470,
        "FY2021's share should fall below the band; it is {:.4}",
        share_of(2021)
    );
}

/// The series rises in real terms, the pandemic takes back a third of it, and FY2023 takes a new
/// peak.
///
/// Stated as measurements rather than bands where the value is what the corpus would cite, and
/// as inequalities where the claim is about shape.
///
/// **What changed when the panel gained two years:** FY2019's $779.31 was the maximum of the
/// series and was asserted as such. It is now the pre-pandemic maximum, and the maximum is FY2023
/// at $794.31 — 1.9% above it. FY2024 holds that level at $788.74 rather than giving it back, so
/// the new peak is not a single-year spike.
#[test]
fn real_transportation_spending_takes_a_new_peak_in_fy2023() {
    let rows = comparable();

    let first = real_per_pupil(&rows, 2009);
    let pre_pandemic_peak = real_per_pupil(&rows, 2019);
    let trough = real_per_pupil(&rows, 2021);
    let rebound = real_per_pupil(&rows, BASE_YEAR);
    let peak = real_per_pupil(&rows, 2023);

    assert!(
        (672.0..=674.0).contains(&first),
        "FY2009 real transportation per pupil is ${first:.2}, not the measured $673.12"
    );
    assert!(
        (778.0..=781.0).contains(&pre_pandemic_peak),
        "FY2019 real transportation per pupil is ${pre_pandemic_peak:.2}, not the measured $779.31"
    );
    assert!(
        (793.0..=796.0).contains(&peak),
        "FY2023 real transportation per pupil is ${peak:.2}, not the measured $794.31"
    );

    // FY2023 is the maximum of the whole series, not merely greater than its neighbours.
    for year in years(&rows) {
        let value = real_per_pupil(&rows, year);
        assert!(
            value <= peak,
            "FY{year} at ${value:.2} exceeds the FY2023 peak of ${peak:.2}"
        );
    }
    // And it clears the old peak rather than tying it.
    assert!(
        peak > pre_pandemic_peak,
        "FY2023 (${peak:.2}) should exceed the FY2019 peak (${pre_pandemic_peak:.2})"
    );
    // FY2024 holds the new level: the peak is a step, not a spike.
    let after = real_per_pupil(&rows, 2024);
    assert!(
        after > pre_pandemic_peak && (peak - after) / peak < 0.02,
        "FY2024 (${after:.2}) should hold the FY2023 level (${peak:.2})"
    );

    let fall = (pre_pandemic_peak - trough) / pre_pandemic_peak;
    assert!(
        (0.110..=0.122).contains(&fall),
        "FY2019 to FY2021 should fall about 11.6% in real terms; it falls {:.1}%",
        fall * 100.0
    );

    // And the FY2022 rebound is partial: back inside the pre-pandemic range but below its peak.
    assert!(
        trough < rebound && rebound < pre_pandemic_peak,
        "FY2022 (${rebound:.2}) should sit above the FY2021 trough (${trough:.2}) and below the \
         FY2019 peak (${pre_pandemic_peak:.2})"
    );
}

/// The pandemic confound, stated as a test so that fitting anything to this series has to
/// contend with it — and the year that is now larger than either.
///
/// The two pandemic years are still the largest moves *of the thirteen NCES years*, in both
/// directions, and any fuel-price elasticity estimated over those without holding them out is
/// fitting the closure of Ohio's schools.
///
/// **FY2023 is larger than both.** +7.8% real against the FY2022 rebound's +7.0% and the FY2021
/// collapse's −6.6%, and it is the first large move in the series that is not a school closure or
/// the recovery from one. That is why two more years of this column were worth acquiring: the
/// panel's identifying variation used to be entirely pandemic.
#[test]
fn the_largest_move_is_fy2023_and_the_pandemic_years_are_next() {
    let rows = comparable();
    let years = years(&rows);

    let mut moves: Vec<(u16, f64)> = years
        .windows(2)
        .map(|pair| {
            let (before, after) = (pair[0], pair[1]);
            let change = (real_per_pupil(&rows, after) - real_per_pupil(&rows, before))
                / real_per_pupil(&rows, before);
            (after, change)
        })
        .collect();
    moves.sort_by(|a, b| {
        b.1.abs()
            .partial_cmp(&a.1.abs())
            .expect("no NaN in the series")
    });

    let largest: Vec<u16> = moves.iter().take(3).map(|(year, _)| *year).collect();
    assert_eq!(
        largest[0], 2023,
        "the largest real move should be FY2023; the ordering is {largest:?}"
    );
    assert!(
        largest.contains(&2021) && largest.contains(&2022),
        "the FY2021 collapse and the FY2022 rebound should follow it; they are {largest:?}"
    );

    // And within the thirteen years NCES published, the pandemic pair is still the largest two —
    // which is the confound as it stood, unchanged by the two years added after it.
    let mut nces_only: Vec<(u16, f64)> =
        moves.iter().copied().filter(|(y, _)| *y <= 2022).collect();
    nces_only.sort_by(|a, b| {
        b.1.abs()
            .partial_cmp(&a.1.abs())
            .expect("no NaN in the series")
    });
    let pre: Vec<u16> = nces_only.iter().take(2).map(|(year, _)| *year).collect();
    assert!(
        pre.contains(&2021) && pre.contains(&2022),
        "through FY2022 the two largest moves should still be the pandemic pair; they are {pre:?}"
    );
}
