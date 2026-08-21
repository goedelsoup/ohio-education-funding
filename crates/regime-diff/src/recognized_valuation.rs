//! Recognized valuation — the base the charge-off was actually applied to.
//!
//! # What this corpus had wrong, and it was not a detail
//!
//! Both [`charge_off`](crate::charge_off) and the parameter node described recognized valuation as
//! *"an H.B. 920-adjusted figure, deliberately lower than total taxable value for districts whose
//! reduction factors have bitten."* That is wrong, and wrong in a way that named the wrong
//! districts.
//!
//! The Legislative Service Commission defines it plainly, and the definition is arithmetic rather
//! than adjectival:
//!
//! > To prevent a school district's state base cost funding from fluctuating significantly from
//! > one year to another because of reappraisals and updates, valuation used in calculating a
//! > district's local share of the base cost "recognizes" the district's inflationary increase in
//! > carryover real property (property that was taxed in the year before) in the reappraisal or
//! > update year evenly over three years instead of all at once.
//!
//! ```text
//! Recognized valuation, reappraisal or update year = Actual - 2/3 x inflationary increase
//! Recognized valuation, second year                = Actual - 1/3 x inflationary increase
//! Recognized valuation, third year                 = Actual
//! ```
//!
//! So it is a **three-year phase-in of reappraisal growth**, not a reduction-factor adjustment.
//! H.B. 920 holds a district's *revenue* flat by cutting the rate; recognized valuation holds its
//! *state aid* flat by deferring the base. Two mechanisms answering the same event — a reappraisal
//! — from opposite ends of the same multiplication. LSC discusses them under one heading, which is
//! the likeliest way the conflation happened, and the heading is
//! "Provisions that Soften the Effect of H.B. 920 Tax Reduction Factors."
//!
//! **The correction changes which districts are affected, not just by how much.** Under the
//! recorded reading, the districts whose charge-off was overstated were the ones with long-bitten
//! reduction factors — a wealth-and-tax-history story. Under the real rule they are the ones whose
//! county happened to reappraise recently. Those are close to unrelated sets, and the second one
//! is an administrative accident: a district's deemed local share moved by a tenth depending on
//! which year its county auditor was on the Department of Taxation's calendar.
//!
//! # The calendar is the whole mechanism, and it is published
//!
//! Ohio's 88 counties reappraise every six years and update at the three-year midpoint, on a
//! staggered schedule the Department of Taxation publishes. [`CYCLE`] carries it. The consequence
//! that makes this reconstructible: **every county has exactly one valuation event in TY2022
//! through TY2024**, so a four-year window of Table SD-1 gives each district one event year and
//! two quiet ones.
//!
//! # Estimating the inflationary increase, and being honest that it is estimated
//!
//! LSC's "inflationary increase in carryover real property" excludes new construction by
//! definition — carryover is property that was taxed the year before. Table SD-1 does not publish
//! carryover value, so the increase cannot be read off; it has to be separated from everything
//! else that moved a district's real property in the same year.
//!
//! The staggered calendar does that separation. A district's two non-event years measure its own
//! ordinary growth — new construction, board-of-revision changes, abatements expiring — and the
//! event year's excess over that rate is the reappraisal. Per district, not against a statewide
//! average, which matters because a fast-growing suburb and a shrinking rural district have very
//! different ordinary rates.
//!
//! The separation is wide enough to be reliable. Across the 88 counties the median event-year jump
//! in real property is **28.6%** and the median non-event year is **1.5%**; per district the
//! background rate has a median of 0.96% and a 10th-to-90th range of 0.12% to 3.28%.
//!
//! **This is an estimate and the site says so.** It assumes a district's ordinary growth runs at a
//! similar rate in the event year as in its quiet years. What it cannot see is a district that
//! annexed a subdivision in the same year its county reappraised; that district's estimated
//! inflationary increase is too large and its recognized valuation too low.
//!
//! # What is not modelled
//!
//! LSC records a separate **exempt property adjustment** to recognized valuation for about 13
//! districts holding large amounts of state-owned exempt property — $836.4 million in FY2008,
//! worth $19.2 million of local share. It needs the department's own list of those districts,
//! which this corpus does not hold. It is roughly 0.3% of statewide valuation and it is not here.

use std::collections::HashMap;

use edfund_core::Dollars;

/// What a county did to its property values in a given tax year.
///
/// Carried because they are different acts, though recognized valuation treats them identically:
/// a reappraisal is a full revaluation of every parcel, an update is a trend adjustment from
/// recent sales. The distinction predicts the *size* of the jump, not its treatment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValuationEvent {
    /// A sexennial reappraisal — every parcel revalued.
    Reappraisal,
    /// A triennial update — values trended from sales data.
    Update,
}

/// One county's valuation event inside the TY2022–TY2024 window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountyCycle {
    /// County name, as Table SD-1 spells it, upper-cased.
    pub county: &'static str,
    /// The tax year of the event.
    pub tax_year: u16,
    /// Which kind of event it was.
    pub event: ValuationEvent,
}

/// The first tax year of the window every county has exactly one event in.
pub const WINDOW_FIRST: u16 = 2022;
/// The last tax year of that window.
pub const WINDOW_LAST: u16 = 2024;

/// Every Ohio county's reappraisal or update year in TY2022–TY2024.
///
/// # Where this comes from, and why it is constants rather than a fixture
///
/// The Department of Taxation publishes *Year of Sexennial Reappraisal and Triennial Update for
/// Ohio's 88 Counties, 2024–2029* as a one-page PDF. This workspace has no PDF text extractor and
/// should not grow one for 88 rows; the precedent is the charge-off rate series, which is carried
/// as constants beside its citation for the same reason.
///
/// The published table covers 2024 onward. TY2022 and TY2023 are derived from it rather than from
/// a second document, and the derivation is exact: the cycle is reappraisal, update three years
/// later, reappraisal three years after that. A county listed for reappraisal in 2026 updated in
/// 2023; one listed for update in 2025 was reappraised in 2022. The published table validates the
/// rule on its own face — its 2024 reappraisal counties are precisely its 2027 update counties.
///
/// # It is checked against data that would expose a mistake
///
/// A hand-transcribed table invites a hand-transcription error, so the tests do not merely count
/// rows. `every_county_jumps_in_the_year_the_calendar_says` takes each county's real property
/// value from Table SD-1 across four tax years and asserts that its largest year-over-year
/// increase falls in the year listed here. **All 88 pass.** A county typed into the wrong column
/// would fail it, because a 28% reappraisal jump is not something a misassignment can hide.
pub const CYCLE: &[CountyCycle] = &[
    // TY2022 reappraisal — the counties the 2025 table lists for update.
    reappraisal("ADAMS", 2022),
    reappraisal("COLUMBIANA", 2022),
    reappraisal("HANCOCK", 2022),
    reappraisal("HOCKING", 2022),
    reappraisal("HOLMES", 2022),
    reappraisal("LAWRENCE", 2022),
    reappraisal("MEIGS", 2022),
    reappraisal("MONROE", 2022),
    reappraisal("PAULDING", 2022),
    reappraisal("SCIOTO", 2022),
    reappraisal("TUSCARAWAS", 2022),
    reappraisal("WASHINGTON", 2022),
    // TY2022 update — the counties the 2025 table lists for reappraisal.
    update("CARROLL", 2022),
    update("CHAMPAIGN", 2022),
    update("CLARK", 2022),
    update("FAIRFIELD", 2022),
    update("LOGAN", 2022),
    update("MARION", 2022),
    update("MEDINA", 2022),
    update("MIAMI", 2022),
    update("ROSS", 2022),
    update("UNION", 2022),
    update("WYANDOT", 2022),
    // TY2023 reappraisal — the counties the 2026 table lists for update.
    reappraisal("AUGLAIZE", 2023),
    reappraisal("CLINTON", 2023),
    reappraisal("DARKE", 2023),
    reappraisal("DEFIANCE", 2023),
    reappraisal("DELAWARE", 2023),
    reappraisal("FRANKLIN", 2023),
    reappraisal("GALLIA", 2023),
    reappraisal("GEAUGA", 2023),
    reappraisal("HAMILTON", 2023),
    reappraisal("HARDIN", 2023),
    reappraisal("HARRISON", 2023),
    reappraisal("HENRY", 2023),
    reappraisal("JACKSON", 2023),
    reappraisal("LICKING", 2023),
    reappraisal("MAHONING", 2023),
    reappraisal("MERCER", 2023),
    reappraisal("MORROW", 2023),
    reappraisal("PERRY", 2023),
    reappraisal("PICKAWAY", 2023),
    reappraisal("PIKE", 2023),
    reappraisal("PREBLE", 2023),
    reappraisal("PUTNAM", 2023),
    reappraisal("RICHLAND", 2023),
    reappraisal("SENECA", 2023),
    reappraisal("SHELBY", 2023),
    reappraisal("TRUMBULL", 2023),
    reappraisal("VAN WERT", 2023),
    reappraisal("WOOD", 2023),
    // TY2023 update — the counties the 2026 table lists for reappraisal.
    update("ASHLAND", 2023),
    update("ASHTABULA", 2023),
    update("ATHENS", 2023),
    update("BUTLER", 2023),
    update("CLERMONT", 2023),
    update("FULTON", 2023),
    update("GREENE", 2023),
    update("KNOX", 2023),
    update("MADISON", 2023),
    update("MONTGOMERY", 2023),
    update("NOBLE", 2023),
    update("SUMMIT", 2023),
    update("WAYNE", 2023),
    // TY2024 reappraisal — listed directly on the published table.
    reappraisal("BELMONT", 2024),
    reappraisal("BROWN", 2024),
    reappraisal("CRAWFORD", 2024),
    reappraisal("CUYAHOGA", 2024),
    reappraisal("ERIE", 2024),
    reappraisal("FAYETTE", 2024),
    reappraisal("HIGHLAND", 2024),
    reappraisal("HURON", 2024),
    reappraisal("JEFFERSON", 2024),
    reappraisal("LAKE", 2024),
    reappraisal("LORAIN", 2024),
    reappraisal("LUCAS", 2024),
    reappraisal("MORGAN", 2024),
    reappraisal("MUSKINGUM", 2024),
    reappraisal("OTTAWA", 2024),
    reappraisal("PORTAGE", 2024),
    reappraisal("STARK", 2024),
    reappraisal("WARREN", 2024),
    reappraisal("WILLIAMS", 2024),
    // TY2024 update — listed directly on the published table.
    update("ALLEN", 2024),
    update("COSHOCTON", 2024),
    update("GUERNSEY", 2024),
    update("SANDUSKY", 2024),
    update("VINTON", 2024),
];

const fn reappraisal(county: &'static str, tax_year: u16) -> CountyCycle {
    CountyCycle {
        county,
        tax_year,
        event: ValuationEvent::Reappraisal,
    }
}

const fn update(county: &'static str, tax_year: u16) -> CountyCycle {
    CountyCycle {
        county,
        tax_year,
        event: ValuationEvent::Update,
    }
}

/// The event year for a county, matched case-insensitively on the name Table SD-1 uses.
#[must_use]
pub fn cycle_for(county: &str) -> Option<CountyCycle> {
    CYCLE
        .iter()
        .copied()
        .find(|c| c.county.eq_ignore_ascii_case(county.trim()))
}

/// Where a tax year sits relative to a county's valuation event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseIn {
    /// The reappraisal or update year itself. Two thirds of the increase is deferred.
    First,
    /// The year after. One third is still deferred.
    Second,
    /// The third year and beyond — recognized valuation has caught up with actual.
    Third,
}

impl PhaseIn {
    /// The share of the inflationary increase not yet recognized.
    #[must_use]
    pub fn deferred_fraction(self) -> f64 {
        match self {
            Self::First => 2.0 / 3.0,
            Self::Second => 1.0 / 3.0,
            Self::Third => 0.0,
        }
    }

    /// Where `tax_year` sits given an event in `event_year`.
    ///
    /// A tax year *before* the event is `Third`: the previous cycle's phase-in has finished by
    /// then, because events are three years apart and the phase-in takes exactly three.
    #[must_use]
    pub fn at(event_year: u16, tax_year: u16) -> Self {
        match tax_year.checked_sub(event_year) {
            Some(0) => Self::First,
            Some(1) => Self::Second,
            _ => Self::Third,
        }
    }
}

/// One district's recognized valuation, with the parts it was built from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recognition {
    /// The tax year this describes.
    pub tax_year: u16,
    /// Where the district's county sits in the cycle.
    pub phase: PhaseIn,
    /// The district's own ordinary growth rate, from its non-event years.
    pub background_growth: f64,
    /// The reappraisal-driven increase in real property, net of ordinary growth.
    pub inflationary_increase: Dollars,
    /// The part of that increase not yet recognized.
    pub deferred: Dollars,
    /// Total taxable value as published.
    pub actual: Dollars,
    /// Total taxable value less the deferred increase.
    pub recognized: Dollars,
}

impl Recognition {
    /// Recognized over actual. One where the phase-in has finished, below one where it has not.
    ///
    /// The ratio rather than the level is what should be carried across sources. Table SD-1's
    /// total taxable value and the profile report's assessed valuation per pupil are different
    /// vintages on different denominators, and multiplying the second by a ratio derived from the
    /// first is sound where transplanting a dollar amount would not be.
    #[must_use]
    pub fn ratio(&self) -> f64 {
        if self.actual > 0.0 {
            self.recognized / self.actual
        } else {
            1.0
        }
    }
}

/// Reconstruct a district's recognized valuation for one tax year.
///
/// `real_property` is the district's real property value in consecutive tax years, oldest first;
/// it needs the event year and at least one other to estimate background growth. `actual` is total
/// taxable value in `tax_year` — real property plus public utility — because that is what the
/// charge-off multiplied, while the phase-in applies only to the real property part of it.
///
/// Returns `None` when the county is unknown or the series is too short or degenerate to separate
/// a reappraisal from ordinary growth.
#[must_use]
pub fn recognize(
    county: &str,
    real_property: &[(u16, Dollars)],
    actual: Dollars,
    tax_year: u16,
) -> Option<Recognition> {
    let cycle = cycle_for(county)?;
    let phase = PhaseIn::at(cycle.tax_year, tax_year);

    // Consecutive year-over-year growth rates, tagged with the later year of each pair.
    let changes: Vec<(u16, f64)> = real_property
        .windows(2)
        .filter(|w| w[0].0 + 1 == w[1].0 && w[0].1 > 0.0)
        .map(|w| (w[1].0, w[1].1 / w[0].1 - 1.0))
        .collect();

    // The event's own year, and the quiet years that say what ordinary growth looks like here.
    let event_change = changes.iter().find(|(y, _)| *y == cycle.tax_year)?;
    let quiet: Vec<f64> = changes
        .iter()
        .filter(|(y, _)| *y != cycle.tax_year)
        .map(|(_, g)| *g)
        .collect();
    if quiet.is_empty() {
        return None;
    }
    let background_growth = quiet.iter().sum::<f64>() / quiet.len() as f64;

    // The value the event acted on, and the part of the jump ordinary growth does not explain.
    let before = real_property
        .iter()
        .find(|(y, _)| *y + 1 == cycle.tax_year)
        .map(|(_, v)| *v)?;
    let inflationary_increase = (before * (event_change.1 - background_growth)).max(0.0);
    let deferred = phase.deferred_fraction() * inflationary_increase;

    Some(Recognition {
        tax_year,
        phase,
        background_growth,
        inflationary_increase,
        deferred,
        actual,
        recognized: (actual - deferred).max(0.0),
    })
}

/// A district's real property series and total value, as the fixture supplies them.
#[derive(Debug, Clone)]
pub struct DistrictValuation {
    /// County the district is attributed to.
    pub county: String,
    /// Real property value by tax year, oldest first.
    pub real_property: Vec<(u16, Dollars)>,
    /// Total taxable value in the target tax year.
    pub actual: Dollars,
}

/// Recognize a whole panel at one tax year, keyed by IRN.
#[must_use]
pub fn recognize_panel(
    districts: &HashMap<String, DistrictValuation>,
    tax_year: u16,
) -> HashMap<String, Recognition> {
    districts
        .iter()
        .filter_map(|(irn, d)| {
            recognize(&d.county, &d.real_property, d.actual, tax_year).map(|r| (irn.clone(), r))
        })
        .collect()
}

/// Table SD-1 reduced to one row per district per tax year, as `connect` commits it.
const ABSTRACT: &str = include_str!("../../dispersion/fixtures/sd1-district-taxes.csv");

/// Recognize every district in the committed abstract at one tax year, keyed by IRN.
///
/// The convenience path, so that the bundle and the example do not each grow their own parser and
/// then disagree about a column. Districts whose series is too short are absent from the map
/// rather than present at a default, and [`crate::ChargeOffBase`] treats absence as full value.
///
/// # Panics
///
/// If the committed fixture has lost a column this depends on. That is a build-time fact about
/// the repository rather than a runtime condition, and it should stop the bundle rather than
/// quietly produce a page of unrecognized valuations.
#[must_use]
pub fn from_abstract(tax_year: u16) -> HashMap<String, Recognition> {
    let mut lines = ABSTRACT.lines();
    let head: Vec<&str> = lines
        .next()
        .expect("the abstract has a header")
        .split(',')
        .collect();
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

    let mut districts: HashMap<String, DistrictValuation> = HashMap::new();
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let p: Vec<&str> = line.split(',').collect();
        let Ok(row_year) = p[year].parse::<u16>() else {
            continue;
        };
        let entry = districts
            .entry(p[irn].to_string())
            .or_insert_with(|| DistrictValuation {
                county: p[county].to_string(),
                real_property: Vec::new(),
                actual: 0.0,
            });
        // `conventions::number` rather than `str::parse`: it is the one place that knows what the
        // department writes where there is no value. A raw parse turns `<10`, `#N/A` and a
        // thousands-separated figure alike into `None`, and `unwrap_or(0.0)` then reports every
        // one of them as zero. The zero is kept here because the surrounding type has no way to
        // carry an absence, but it is now a stated substitution rather than a parse failure.
        entry.real_property.push((
            row_year,
            edfund_core::conventions::number(p[real]).unwrap_or(0.0),
        ));
        if row_year == tax_year {
            entry.actual = edfund_core::conventions::number(p[total]).unwrap_or(0.0);
        }
    }
    for district in districts.values_mut() {
        district.real_property.sort_by_key(|(y, _)| *y);
    }
    recognize_panel(&districts, tax_year)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_calendar_covers_every_county_exactly_once() {
        assert_eq!(CYCLE.len(), 88, "Ohio has 88 counties");
        let mut names: Vec<&str> = CYCLE.iter().map(|c| c.county).collect();
        names.sort_unstable();
        let unique = {
            let mut n = names.clone();
            n.dedup();
            n.len()
        };
        assert_eq!(unique, 88, "a county is listed twice");
    }

    #[test]
    fn every_county_has_its_event_inside_the_window() {
        for c in CYCLE {
            assert!(
                (WINDOW_FIRST..=WINDOW_LAST).contains(&c.tax_year),
                "{} sits outside the window at {}",
                c.county,
                c.tax_year
            );
        }
    }

    /// The three event years partition the state, which is what makes the window sufficient.
    ///
    /// Not a restatement of the count above: a calendar could name all 88 counties once and still
    /// pile them into two years, and then a third of the state would have no measurable event.
    #[test]
    fn the_three_event_years_partition_the_eighty_eight() {
        let counts = (WINDOW_FIRST..=WINDOW_LAST)
            .map(|y| CYCLE.iter().filter(|c| c.tax_year == y).count())
            .collect::<Vec<_>>();
        assert_eq!(counts, vec![23, 41, 24]);
        assert_eq!(counts.iter().sum::<usize>(), 88);
    }

    #[test]
    fn the_phase_in_defers_two_thirds_then_one_third_then_nothing() {
        assert_eq!(PhaseIn::at(2024, 2024), PhaseIn::First);
        assert_eq!(PhaseIn::at(2024, 2025), PhaseIn::Second);
        assert_eq!(PhaseIn::at(2024, 2026), PhaseIn::Third);
        // Before the event the previous cycle has already finished phasing in.
        assert_eq!(PhaseIn::at(2024, 2023), PhaseIn::Third);
        assert!((PhaseIn::First.deferred_fraction() - 2.0 / 3.0).abs() < 1e-12);
        assert!((PhaseIn::Second.deferred_fraction() - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(PhaseIn::Third.deferred_fraction(), 0.0);
    }

    /// The Legislative Service Commission's own worked example, reproduced.
    ///
    /// District A goes from $112.5m to $120.0m after a reappraisal, of which $6.0m is inflationary
    /// increase in real property. LSC states the recognized value as $116.0m — actual less two
    /// thirds of the increase. Anchoring on a published example rather than on the formula
    /// restated in code is the point: the formula is what is being checked.
    ///
    /// LSC's example runs TY2005 to TY2006 and the years here are relabelled to TY2023–TY2024, on
    /// a county [`CYCLE`] puts in the right position. The committed calendar covers the window the
    /// fixture covers, and the arithmetic under test does not depend on which years they are — but
    /// the relabelling is stated rather than left for a reader to notice the dates do not match
    /// the source.
    #[test]
    fn the_commissions_worked_example_comes_out_at_its_published_figure() {
        // $1.5m of the $7.5m rise is ordinary growth, leaving $6.0m inflationary — expressed as a
        // rate against the prior year so the estimator has to recover the split itself.
        let background = 1.5 / 112.5;
        let series = [
            (2022, 112_500_000.0 / (1.0 + background)),
            (2023, 112_500_000.0),
            (2024, 120_000_000.0),
        ];
        let got = recognize("CUYAHOGA", &series, 120_000_000.0, 2024)
            .expect("a reappraisal county with a full series");
        assert_eq!(got.phase, PhaseIn::First);
        assert!(
            (got.inflationary_increase - 6_000_000.0).abs() < 1_000.0,
            "inflationary increase {}",
            got.inflationary_increase
        );
        assert!(
            (got.recognized - 116_000_000.0).abs() < 1_000.0,
            "recognized {}",
            got.recognized
        );
    }

    #[test]
    fn a_district_past_the_phase_in_is_recognized_at_its_full_value() {
        let series = [
            (2021, 100_000_000.0),
            (2022, 130_000_000.0),
            (2023, 131_000_000.0),
            (2024, 132_000_000.0),
        ];
        // ADAMS had its event in TY2022, so by TY2024 the phase-in has finished.
        let got = recognize("ADAMS", &series, 132_000_000.0, 2024).expect("known county");
        assert_eq!(got.phase, PhaseIn::Third);
        assert_eq!(got.deferred, 0.0);
        assert!((got.ratio() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn an_unknown_county_yields_nothing_rather_than_a_default() {
        let series = [(2023, 1.0), (2024, 2.0)];
        assert!(recognize("NOT A COUNTY", &series, 2.0, 2024).is_none());
    }

    #[test]
    fn a_district_that_only_shrank_has_no_inflationary_increase_to_defer() {
        // Floored at zero: a negative "increase" would raise recognized valuation above actual,
        // charging a district for value it does not have.
        let series = [
            (2021, 100_000_000.0),
            (2022, 99_000_000.0),
            (2023, 98_000_000.0),
            (2024, 97_000_000.0),
        ];
        let got = recognize("CUYAHOGA", &series, 97_000_000.0, 2024).expect("known county");
        assert_eq!(got.phase, PhaseIn::First);
        assert_eq!(got.inflationary_increase, 0.0);
        assert_eq!(got.recognized, 97_000_000.0);
    }
}
