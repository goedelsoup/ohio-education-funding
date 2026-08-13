//! Ohio in the school finance survey, across every year the archive publishes.
//!
//! # What this is for, and how it differs from its sibling
//!
//! [`crate::national_peers`] places one Ohio district among America's in a single year. It
//! answers "is this unusual", and it is the only thing in this repository that can, because every
//! other source is Ohio describing itself.
//!
//! This module answers a different question — **how Ohio changed** — and needs the opposite
//! shape: only Ohio, but ten years of it. The two are separate fixtures for that reason rather
//! than one fixture with a filter.
//!
//! # Why it matters that the definitions are federal
//!
//! Ohio replaced its funding regime twice inside this window and renamed most of its own
//! categories doing it. A series assembled from Ohio's own reporting would be measuring the
//! reporting as much as the funding. The Bureau counted the same way in FY2012 as in FY2022, so a
//! change here is a change in the thing rather than in the definition — which is exactly why the
//! Auditor of State used NCES for the Longitudinal School Finance Study, stating that "a
//! comparable state source of public school financial data was not available across the desired
//! time period."
//!
//! # Three caveats, each capable of producing a wrong reading
//!
//! **FY2014 is missing from the archive**, under every naming the neighbouring years use. Nine
//! intervals across ten years, and one of them is two years wide. Nothing here interpolates it.
//!
//! **The denominator is the Bureau's** — `V33`, fall membership — and not Ohio's enrolled ADM. A
//! per-pupil figure from this module must never be shown beside one from the funding model.
//!
//! **The panel's membership changes, and naming it took sixteen directories.** `LEAID` resolves
//! to an IRN through the CCD directory *for the panel's own year*, so an agency that closed in
//! FY2015 is named by the file written in FY2015. [`unnamed_agencies`] is now zero everywhere and
//! is kept as the assertion that it stays so.
//!
//! It used to resolve through the FY2022-23 file alone, and 124 FY2012 agencies had no IRN. This
//! docstring called that count "the consolidation history, measured". It was not: 121 of the 124
//! are community schools. Ohio's traditional districts barely move across this window — 612
//! comparable agencies in FY2009 against 611 in FY2022 — and the whole district-consolidation
//! history in it is **three districts**. See [`crate::lea_directory`], which names them, and
//! which also records that no rule over the federal directory can say why any of them went.

use std::collections::BTreeMap;

/// The committed panel.
const FIXTURE: &str = include_str!("../fixtures/f33-ohio-panel.csv");

/// The header this loader was written against.
const EXPECTED_HEADER: &str = "fiscal_year,leaid,irn,comparable,enrollment,total_revenue,\
federal_revenue,state_revenue,local_revenue,property_tax,current_spending";

/// One Ohio agency in one year of the survey.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelRow {
    /// The fiscal year the survey reports.
    pub fiscal_year: u16,
    /// The NCES agency identifier, which is the only key present in every year.
    pub leaid: String,
    /// Ohio's identifier, where the FY2022-23 directory still carries the agency. Empty otherwise.
    pub irn: String,
    /// Whether the agency belongs in a district comparison — `AGCHRT != 1` and `SCHLEV == 03`.
    pub comparable: bool,
    /// `V33`, fall membership on the Bureau's count.
    pub enrollment: f64,
    /// All revenue, from every source, in thousands of dollars.
    pub total_revenue: f64,
    /// The federal share of it.
    pub federal_revenue: f64,
    /// The state share.
    pub state_revenue: f64,
    /// And the local share.
    pub local_revenue: f64,
}

/// Every row of the panel.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, which means the extractor
/// changed and this reader did not.
#[must_use]
pub fn panel() -> Vec<PanelRow> {
    let mut lines = FIXTURE.lines();
    assert_eq!(
        lines.next().unwrap_or_default().trim(),
        EXPECTED_HEADER,
        "the Ohio panel fixture header changed; update dispersion::ohio_panel"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            let num = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok());
            Some(PanelRow {
                fiscal_year: f.first()?.parse().ok()?,
                leaid: f.get(1)?.to_string(),
                irn: f.get(2)?.to_string(),
                comparable: f.get(3)? == &"1",
                enrollment: num(4)?,
                total_revenue: num(5)?,
                federal_revenue: num(6)?,
                state_revenue: num(7)?,
                local_revenue: num(8)?,
            })
        })
        .collect()
}

/// Where a year's school money came from, as shares of total revenue.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RevenueMix {
    /// How many comparable districts the shares are computed over.
    pub districts: usize,
    /// Local revenue as a share of total.
    pub local: f64,
    /// State revenue as a share of total.
    pub state: f64,
    /// Federal revenue as a share of total.
    pub federal: f64,
}

/// The revenue mix for every year in the panel, over comparable districts only.
///
/// Comparable-only because that is the population the corpus's existing FY2022 figures are
/// computed over, and a series whose first point is not comparable to the number already on the
/// page is worse than no series.
#[must_use]
pub fn revenue_mix_by_year() -> BTreeMap<u16, RevenueMix> {
    let mut out: BTreeMap<u16, RevenueMix> = BTreeMap::new();
    let mut totals: BTreeMap<u16, (f64, f64, f64, f64)> = BTreeMap::new();
    for row in panel().iter().filter(|r| r.comparable) {
        let t = totals.entry(row.fiscal_year).or_default();
        t.0 += row.total_revenue;
        t.1 += row.local_revenue;
        t.2 += row.state_revenue;
        t.3 += row.federal_revenue;
        out.entry(row.fiscal_year).or_default().districts += 1;
    }
    for (year, (total, local, state, federal)) in totals {
        if let Some(mix) = out.get_mut(&year) {
            if total > 0.0 {
                mix.local = local / total;
                mix.state = state / total;
                mix.federal = federal / total;
            }
        }
    }
    out
}

/// How much of the local gap each level of government closed, in one year.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Equalization {
    /// Comparable districts the quartiles are cut over.
    pub districts: usize,
    /// Mean local revenue per pupil in the poorest quartile.
    pub poorest_local: f64,
    /// And in the richest.
    pub richest_local: f64,
    /// The gap between them, which is what the other levels are measured against.
    pub gap: f64,
    /// Dollars per pupil of that gap closed by state aid.
    pub state_closes: f64,
    /// And by federal aid.
    pub federal_closes: f64,
}

impl Equalization {
    /// What neither level closes — the part a district actually experiences.
    #[must_use]
    pub fn residual(&self) -> f64 {
        self.gap - self.state_closes - self.federal_closes
    }

    /// State aid's share of the gap, as a fraction.
    #[must_use]
    pub fn state_share(&self) -> f64 {
        if self.gap > 0.0 {
            self.state_closes / self.gap
        } else {
            0.0
        }
    }
}

/// The quartile equalization measure, for every year in the panel.
///
/// # The question this answers, and the one it does not
///
/// [`crate::national_peers::ohio_by_local_wealth`] computes this for FY2022 alone, and the corpus
/// recorded that state aid closes 46% of the local gap and federal aid 9.5%. A single year cannot
/// say whether that is Ohio getting better or worse at equalizing, and `doctrine/equity` carried
/// the question as open.
///
/// It answers it in a way neither "better" nor "worse" captures. **The rate holds and the gap
/// grows.** The state's share of the gap it closes stays in a narrow band across ten years while
/// the gap itself grows by two thirds, so the residual — the part no level closes — grows with it.
/// A formula doing the same proportional job against a larger problem leaves districts further
/// apart every year, and the percentage is the thing that looks stable.
///
/// # Both endpoints flatter the federal column
///
/// FY2012 still carries the ARRA tail and FY2022 is the ESSER peak, so the federal contribution
/// is roughly double its ordinary size at each end of the window. Read the middle years for the
/// federal figure and the whole window for the state one.
#[must_use]
pub fn equalization_by_year() -> BTreeMap<u16, Equalization> {
    let mut by_year: BTreeMap<u16, Vec<(f64, f64, f64)>> = BTreeMap::new();
    for row in panel()
        .iter()
        .filter(|r| r.comparable && r.enrollment > 0.0)
    {
        by_year.entry(row.fiscal_year).or_default().push((
            row.local_revenue / row.enrollment,
            row.state_revenue / row.enrollment,
            row.federal_revenue / row.enrollment,
        ));
    }

    let mut out = BTreeMap::new();
    for (year, mut rows) in by_year {
        rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let n = rows.len();
        if n < 4 {
            continue;
        }
        // Cut by position, as the single-year measure does, so the two are comparable.
        let mut sums = [[0.0f64; 3]; 4];
        let mut counts = [0usize; 4];
        for (i, (local, state, federal)) in rows.iter().enumerate() {
            let q = ((i * 4) / n).min(3);
            sums[q][0] += local;
            sums[q][1] += state;
            sums[q][2] += federal;
            counts[q] += 1;
        }
        let mean = |q: usize, k: usize| sums[q][k] / counts[q] as f64;
        let (poorest_local, richest_local) = (mean(0, 0), mean(3, 0));
        out.insert(
            year,
            Equalization {
                districts: n,
                poorest_local,
                richest_local,
                gap: richest_local - poorest_local,
                state_closes: mean(0, 1) - mean(3, 1),
                federal_closes: mean(0, 2) - mean(3, 2),
            },
        );
    }
    out
}

/// Agencies per year no directory names, which should be none.
///
/// Kept after the thing it measured went to zero, because zero is the claim. The extractor
/// resolves each agency against the directory for its own year and falls forward to the nearest
/// later one; a non-zero count here means an agency reported finance in a year no directory
/// carries it, which is a hole in the retrieval rather than a fact about Ohio.
#[must_use]
pub fn unnamed_agencies() -> BTreeMap<u16, usize> {
    let mut out = BTreeMap::new();
    for row in panel() {
        let counter = out.entry(row.fiscal_year).or_insert(0);
        if row.irn.is_empty() {
            *counter += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_holds_thirteen_years_and_names_the_one_it_does_not() {
        let years: Vec<u16> = revenue_mix_by_year().keys().copied().collect();
        assert_eq!(
            years,
            vec![2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022],
            "FY2014 is absent from the archive and nothing should have invented it"
        );
    }

    /// The new series reproduces the figure the corpus already held, by a different route.
    ///
    /// The 51.8% local share came from the Bureau's state-level table and then, independently,
    /// from the national district file. This is a third assembly — Ohio-only, ten archives, a
    /// different builder — and it has to land in the same place or one of the three is wrong.
    #[test]
    fn the_panel_reproduces_the_local_share_the_corpus_already_published() {
        let mix = revenue_mix_by_year();
        let fy2022 = mix[&2022];
        assert!(
            (fy2022.local - 0.518).abs() < 0.005,
            "the panel gives FY2022 a {:.4} local share against the 0.518 already published",
            fy2022.local
        );
        assert_eq!(fy2022.districts, 611, "the comparable panel changed size");
    }

    /// The finding: the state share fell across the window while the local share rose — and it
    /// fell by half again as much as the FY2012 start suggested.
    ///
    /// # Where the window starts changes the size of the finding
    ///
    /// This panel opened at FY2012 until the FY2009-FY2011 archives were added, and FY2012 is
    /// already three years into the decline. From FY2012 the state share falls 7.7 points; from
    /// FY2009 it falls **11.7**, from 45.9% to 34.2%. Both are correct about their own window,
    /// which is the periodic point `metric/per-pupil-operating-expenditure` records: an endpoint
    /// pair reports the window it was given and says nothing about the window it was not.
    #[test]
    fn the_state_share_falls_across_the_window_and_the_local_share_rises() {
        let mix = revenue_mix_by_year();
        let (opening, first, last) = (mix[&2009], mix[&2012], mix[&2022]);

        assert!(
            first.state - last.state > 0.05,
            "the state share fell {:.1} points from FY2012",
            100.0 * (first.state - last.state)
        );
        // The three added years carry four of the eleven-and-a-half points.
        let full = opening.state - last.state;
        assert!(
            (full - 0.117).abs() < 0.005,
            "from FY2009 the state share fell {:.1} points",
            100.0 * full
        );
        assert!(
            full > (first.state - last.state) * 1.4,
            "opening at FY2012 understates the decline by less than the corpus records"
        );
        assert!(
            last.local > first.local && last.local > opening.local,
            "the local share no longer rises across the window"
        );
        // Not monotonic and should not be asserted as such: FY2015 interrupts the fall, and
        // FY2022's local share drops because the ESSER-inflated federal share deflates both
        // domestic ones. A test claiming monotonicity would be pinning a stronger fact than the
        // data supports.
        assert!(
            mix.values().all(|m| m.state < 0.47 && m.state > 0.30),
            "a year's state share left the range the series has ever occupied"
        );
    }

    /// Both federal spikes are in the window, and they are the same shape.
    #[test]
    fn the_panel_carries_two_federal_cliffs_rather_than_one() {
        let mix = revenue_mix_by_year();
        // The ARRA tail: FY2012 still carries stimulus money and FY2013 does not.
        assert!(
            mix[&2012].federal - mix[&2013].federal > 0.02,
            "the ARRA drop between FY2012 and FY2013 is not in the panel"
        );
        // With FY2009-FY2011 added the whole ARRA arc is visible rather than only its tail:
        // 6.7% federal in FY2009, 11.0% at the FY2011 peak, 5.2% by FY2013. The rise and the
        // fall are both larger than the tail the panel used to open on.
        assert!(
            mix[&2011].federal > mix[&2009].federal * 1.5,
            "the ARRA build-up is not in the panel"
        );
        assert!(
            mix[&2011].federal - mix[&2013].federal > 0.05,
            "the full ARRA withdrawal is {:.3}",
            mix[&2011].federal - mix[&2013].federal
        );
        // And ESSER, four times the size of what it replaced at its peak.
        assert!(
            mix[&2022].federal > mix[&2019].federal * 1.8,
            "the ESSER peak is not visible against the pre-pandemic baseline"
        );
    }

    /// The finding: the rate holds, the gap grows, and the residual grows with it.
    #[test]
    fn the_equalization_rate_holds_while_the_gap_it_closes_grows() {
        let by_year = equalization_by_year();
        let (first, last) = (by_year[&2012], by_year[&2022]);

        // The gap grew by two thirds.
        assert!(
            last.gap > first.gap * 1.5,
            "the local gap went from {:.0} to {:.0}, which is not the growth recorded",
            first.gap,
            last.gap
        );

        // And the state's share of it did not move much in either direction. Asserted as a band
        // rather than a trend, because there is no trend — that is the finding.
        for (year, e) in by_year.iter().filter(|(y, _)| **y >= 2012) {
            assert!(
                e.state_share() > 0.38 && e.state_share() < 0.49,
                "FY{year} state share is {:.3}, outside the band the corpus records",
                e.state_share()
            );
        }

        // So the part nobody closes grows with the gap.
        assert!(
            last.residual() > first.residual() * 1.4,
            "the residual went from {:.0} to {:.0}",
            first.residual(),
            last.residual()
        );
    }

    /// **What the three added archives found: during ARRA the state closed a third of the local
    /// gap, not the ~45% it closes in every other year on record.**
    ///
    /// FY2010 and FY2011 sit at 0.349 and 0.330 while federal gap-closing roughly doubles to
    /// 0.122 and 0.128. FY2009, before the money arrived, is an ordinary 0.472. The panel used to
    /// open at FY2012 and could not see this at all.
    ///
    /// Read with care about direction. This is a share of a gap, not a dollar amount, and the gap
    /// itself moves; the state can close a smaller fraction while spending more. What it does
    /// establish is that the two levels moved in opposite directions exactly when the stimulus
    /// landed and again when it left, which is the substitution question the corpus has recorded
    /// as open for the [`crate`]-level equalization series. It does not establish that ARRA
    /// displaced state money — that needs the appropriation series, which is not held here.
    #[test]
    fn the_stimulus_years_are_the_only_ones_where_the_state_closes_a_third() {
        let by_year = equalization_by_year();
        for year in [2010u16, 2011] {
            let e = by_year[&year];
            assert!(
                e.state_share() < 0.36,
                "FY{year} state share is {:.3}, not the third the archives show",
                e.state_share()
            );
            assert!(
                e.federal_closes / e.gap > 0.11,
                "FY{year} federal share of the gap is {:.3}",
                e.federal_closes / e.gap
            );
        }
        // FY2009 brackets them from before, FY2013 from after; both are ordinary.
        for year in [2009u16, 2013] {
            let e = by_year[&year];
            assert!(e.state_share() > 0.44, "FY{year} is {:.3}", e.state_share());
            assert!(e.federal_closes / e.gap < 0.07);
        }
    }

    /// The single-year measure and the panel measure agree where they overlap.
    ///
    /// `national_peers::ohio_by_local_wealth` computes FY2022 from the national district file and
    /// this computes it from the Ohio panel — different fixtures, different builders, same
    /// quartile rule. If they disagreed, one of the two figures the corpus publishes would be
    /// wrong and nothing else would say so.
    #[test]
    fn the_panel_reproduces_the_single_year_equalization_measure() {
        let panel_2022 = equalization_by_year()[&2022];
        let quartiles = crate::national_peers::ohio_by_local_wealth();
        let gap = quartiles[3].local_per_pupil - quartiles[0].local_per_pupil;
        let state = quartiles[0].state_per_pupil - quartiles[3].state_per_pupil;

        assert!(
            (panel_2022.gap - gap).abs() < 1.0,
            "gap: panel {:.0}, cross-section {gap:.0}",
            panel_2022.gap
        );
        assert!(
            (panel_2022.state_closes - state).abs() < 1.0,
            "state: panel {:.0}, cross-section {state:.0}",
            panel_2022.state_closes
        );
    }

    /// Every agency in every year is named, which took sixteen directories rather than one.
    ///
    /// This test used to assert the opposite: that FY2012 carried more than a hundred unnamed
    /// agencies and that the count shrank towards FY2022. Both were true and the reading laid
    /// over them was not — the shrinking count was community schools closing, and the panel was
    /// asking a 2023 file about agencies that had gone in 2015.
    #[test]
    fn the_directory_names_every_agency_in_every_year() {
        for (year, unnamed) in unnamed_agencies() {
            assert_eq!(
                unnamed, 0,
                "FY{year} has {unnamed} agencies no directory year names"
            );
        }
    }

    /// And the identifiers it resolves are the contemporaneous ones, not the survivors'.
    #[test]
    fn an_agency_that_closed_is_named_by_a_file_written_while_it_existed() {
        // Ledgemont Local left the directory after 2016-17 and is absent from the 2022-23 file
        // this panel used to resolve every row through. Its finance rows carry its IRN because
        // the panel now asks the year's own directory.
        let ledgemont: Vec<PanelRow> = panel()
            .into_iter()
            .filter(|r| r.leaid == "3904720")
            .collect();
        assert!(!ledgemont.is_empty(), "Ledgemont Local is not in the panel");
        for row in &ledgemont {
            assert_eq!(
                row.irn, "047209",
                "FY{} names Ledgemont {:?}",
                row.fiscal_year, row.irn
            );
        }
    }
}
