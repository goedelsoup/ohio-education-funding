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
//! **The panel's membership is only as good as one directory.** `LEAID` resolves to an IRN
//! through the FY2022-23 CCD file, so an agency that closed before FY2023 has no IRN here. That
//! is not a defect to hide — [`unnamed_agencies`] reports it per year, and the count going from
//! 124 in FY2012 to 0 in FY2022 *is* the consolidation history, measured.

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

/// Agencies per year the FY2022-23 directory cannot name, which is the consolidation history.
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
    fn the_panel_holds_ten_years_and_names_the_one_it_does_not() {
        let years: Vec<u16> = revenue_mix_by_year().keys().copied().collect();
        assert_eq!(
            years,
            vec![2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022],
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

    /// The finding: the state share fell for a decade while the local share rose.
    #[test]
    fn the_state_share_falls_across_the_decade_and_the_local_share_rises() {
        let mix = revenue_mix_by_year();
        let (first, last) = (mix[&2012], mix[&2022]);

        assert!(
            first.state - last.state > 0.05,
            "the state share fell {:.1} points, not the 7-plus the corpus records",
            100.0 * (first.state - last.state)
        );
        assert!(
            last.local > first.local,
            "the local share no longer rises across the window"
        );
        // Not monotonic and should not be asserted as such: FY2015 interrupts the fall, and
        // FY2022's local share drops because the ESSER-inflated federal share deflates both
        // domestic ones. A test claiming monotonicity would be pinning a stronger fact than the
        // data supports.
        assert!(
            mix.values().all(|m| m.state < 0.45 && m.state > 0.30),
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
        // And ESSER, four times the size of what it replaced at its peak.
        assert!(
            mix[&2022].federal > mix[&2019].federal * 1.8,
            "the ESSER peak is not visible against the pre-pandemic baseline"
        );
    }

    /// The join loss is real, decays with recency, and is reported rather than hidden.
    #[test]
    fn the_directory_names_less_of_the_panel_the_further_back_it_reaches() {
        let unnamed = unnamed_agencies();
        assert_eq!(
            unnamed[&2022], 0,
            "the directory is the FY2022-23 one and should name every FY2022 agency"
        );
        assert!(
            unnamed[&2012] > 100,
            "FY2012 should carry the consolidation gap; it has {}",
            unnamed[&2012]
        );
        assert!(
            unnamed[&2012] > unnamed[&2017] && unnamed[&2017] > unnamed[&2021],
            "the gap should shrink towards the directory's own year"
        );
    }
}
