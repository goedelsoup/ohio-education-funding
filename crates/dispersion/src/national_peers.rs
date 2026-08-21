//! Where one Ohio district sits among America's, on federal definitions.
//!
//! # The comparison the corpus could not make
//!
//! `census-f33` settled the *DeRolph* claim at state level: Ohio raises 51.8% of school revenue
//! locally against a national 43.4%, seventh of fifty-one. That is a fact about Ohio and it is not
//! a fact about any district in Ohio — and the district is the unit every other page here works in.
//!
//! The obstacle was an identifier. The Census F-33 keys on `IDCENSUS`; the funding model keys on
//! IRN; and the corpus recorded the join as unavailable for as long as `census-f33` has been wired.
//! It was available: NCES publishes the same survey keyed on `LEAID`, and the CCD directory's
//! `ST_LEAID` is the IRN. **All 609 districts in the funding panel join, with no losses.**
//!
//! # What a percentile here means, and what it does not
//!
//! The distribution is **16,872 school districts in every state**, all reported on the Census
//! Bureau's own definitions rather than on each state's. That is the whole value: Ohio describing
//! itself cannot say whether Ohio is unusual, and every other source in this corpus is Ohio
//! describing itself.
//!
//! Three caveats travel with every figure this module produces.
//!
//! **The year is FY2022.** The funding model is FY2027. A district's national position is where it
//! stood five years before the model it sits beside on the page, and nothing here bridges that —
//! the F-33 is published years in arrears and this is the most recent complete file.
//!
//! **The denominator is the federal one.** `V33` is fall membership as the Bureau counts it, not
//! Ohio's enrolled ADM and not the report card's unweighted count. A per-pupil figure from this
//! module must never be shown beside one from the funding model without saying so; see
//! `web/src/lib/denominators.ts`, which exists because this site shipped exactly that error twice.
//!
//! **A district's peers are not its neighbours.** A national percentile answers "is this unusual
//! in America", which is a different question from "is this unusual in Ohio" — and for a state
//! that is itself unusual, the two can point opposite ways. A district can be ordinary for Ohio
//! and extreme nationally, and that is the finding rather than a defect.

use std::collections::BTreeMap;

/// The national panel: every district the survey covers with usable enrolment and revenue.
const FIXTURE: &str = include_str!("../fixtures/f33-districts-fy2022.csv");

/// The header this loader was written against.
const EXPECTED_HEADER: &str = "leaid,irn,state,comparable,enrollment,total_revenue,\
federal_revenue,state_revenue,local_revenue,property_tax,current_spending";

/// One district in the national survey.
#[derive(Debug, Clone, PartialEq)]
pub struct NationalDistrict {
    /// The NCES agency identifier.
    pub leaid: String,
    /// Ohio's own identifier, where this is an Ohio district. Empty otherwise.
    pub irn: String,
    /// Two-letter state code.
    pub state: String,
    /// Whether this agency belongs in the comparison at all — see [`national_panel`].
    pub comparable: bool,
    /// `V33` — fall membership on the Bureau's definition, not Ohio's ADM.
    pub enrollment: f64,
    /// All revenue, from every source.
    pub total_revenue: f64,
    /// The federal share of it.
    pub federal_revenue: f64,
    /// The state share.
    pub state_revenue: f64,
    /// And the local share, which is the measure DeRolph is about.
    pub local_revenue: f64,
    /// `T06`. `None` where the district reports none — which for a fiscally dependent district
    /// means the tax belongs to a parent government rather than that no tax was levied.
    pub property_tax: Option<f64>,
    /// `TCURELSC` — current spending on elementary and secondary education.
    pub current_spending: Option<f64>,
}

impl NationalDistrict {
    /// Local revenue as a share of total. The measure *DeRolph* is a claim about.
    #[must_use]
    pub fn local_share(&self) -> f64 {
        if self.total_revenue > 0.0 {
            self.local_revenue / self.total_revenue
        } else {
            0.0
        }
    }

    /// Total revenue per pupil, on the federal count.
    #[must_use]
    pub fn revenue_per_pupil(&self) -> f64 {
        if self.enrollment > 0.0 {
            self.total_revenue / self.enrollment
        } else {
            0.0
        }
    }

    /// Current spending per pupil, on the federal count. `None` where unreported.
    #[must_use]
    pub fn spending_per_pupil(&self) -> Option<f64> {
        (self.enrollment > 0.0)
            .then(|| self.current_spending.map(|s| s / self.enrollment))
            .flatten()
    }
}

/// Every agency in the survey that this comparison can use, plus every Ohio agency.
///
/// # What "comparable" excludes, and why it had to
///
/// The survey's unit is a **local education agency**, not a school district as Ohio means it. Of
/// Ohio's 968 rows, **357 are community schools, joint vocational districts and educational
/// service centres** — and a community school receives almost no local tax revenue by
/// construction, so leaving them in drags the distribution somewhere no traditional district
/// lives. The first pass did leave them in, and put Ohio's 200 smallest agencies at an **8% local
/// share**, which is a fact about charter finance and not about school districts.
///
/// The filter is the survey's own: `AGCHRT != 1` — no associated charter schools — and
/// `SCHLEV == 03`, a unified elementary-and-secondary agency. That leaves **10,382 agencies
/// nationally**, and it is the set every percentile here is computed against.
///
/// It costs one Ohio district. A single K-8 district reports `SCHLEV = 01`, so it is carried with
/// its figures and without a national position: a K-8 agency's spending per pupil is not
/// comparable with a unified one's, and inventing a percentile for it would be worse than
/// admitting there isn't one.
///
/// # Panics
///
/// If the fixture's header is not the one this loader expects.
#[must_use]
pub fn national_panel() -> Vec<NationalDistrict> {
    let mut lines = FIXTURE.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the F-33 district fixture header changed; update dispersion::national_peers"
    );

    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            let opt = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok());
            Some(NationalDistrict {
                leaid: f.first()?.to_string(),
                irn: f.get(1)?.to_string(),
                state: f.get(2)?.to_string(),
                comparable: f.get(3)? == &"1",
                enrollment: opt(4)?,
                total_revenue: opt(5)?,
                federal_revenue: opt(6)?,
                state_revenue: opt(7)?,
                local_revenue: opt(8)?,
                property_tax: opt(9),
                current_spending: opt(10),
            })
        })
        .collect()
}

/// Where one district sits in the national distribution, on the measures that matter here.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NationalPosition {
    /// Local share of revenue, and its percentile among all reporting districts.
    pub local_share: f64,
    /// Where that share sits among all comparable districts.
    pub local_share_percentile: f64,
    /// Revenue per pupil on the federal count, and its percentile.
    pub revenue_per_pupil: f64,
    /// Where that sits.
    pub revenue_per_pupil_percentile: f64,
    /// Current spending per pupil, and its percentile. Zero where unreported.
    pub spending_per_pupil: f64,
    /// And that.
    pub spending_per_pupil_percentile: f64,
}

/// The national medians the positions are read against.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NationalMedians {
    /// How many agencies the distribution is built over.
    pub districts: usize,
    /// The median local share among them.
    pub local_share: f64,
    /// The median revenue per pupil.
    pub revenue_per_pupil: f64,
    /// And the median current spending per pupil.
    pub spending_per_pupil: f64,
}

/// One quartile of Ohio districts, ordered by how much local revenue they raise per pupil.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WealthQuartile {
    /// How many districts fall in this quartile.
    pub districts: usize,
    /// Mean local revenue per pupil — the variable the quartiles are cut on.
    pub local_per_pupil: f64,
    /// Mean state revenue per pupil.
    pub state_per_pupil: f64,
    /// Mean federal revenue per pupil.
    pub federal_per_pupil: f64,
}

/// How far each higher level of government closes the gap the local one opens.
///
/// # Why this belongs in the corpus rather than in a chart
///
/// Ohio's distinctive feature is not how much it spends but who pays: 51.8% local against a
/// national 43.4%. That is a statement about the state. It says nothing about whether the
/// resulting disparity is corrected, and "the state equalizes" is the claim every defence of the
/// formula rests on.
///
/// Quartiling the 611 comparable districts on local revenue per pupil and reading the other two
/// levels against it turns the claim into a number. **State aid closes just under half the local
/// gap and federal aid closes a tenth of it**, which is a different and more useful thing to know
/// than either "the formula equalizes" or "the formula fails".
///
/// The federal column also answers a question the corpus has left `[open]` in prose: federal money
/// is compensatory in Ohio, monotonically so across all four quartiles. It is simply small.
///
/// # The year flatters it
///
/// FY2022 is the peak of ESSER. The federal offset computed here is the **largest it has ever
/// been**, and the funds behind it expired in September 2024 — so this is an upper bound on a
/// channel that has since shrunk, not a steady state. Any use of it must say so.
#[must_use]
pub fn ohio_by_local_wealth() -> [WealthQuartile; 4] {
    let panel = national_panel();
    let mut ohio: Vec<&NationalDistrict> = panel
        .iter()
        .filter(|d| d.state == "OH" && d.comparable && d.enrollment > 0.0)
        .collect();
    ohio.sort_by(|a, b| {
        (a.local_revenue / a.enrollment)
            .partial_cmp(&(b.local_revenue / b.enrollment))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let n = ohio.len();
    let mut out = [WealthQuartile::default(); 4];
    for (i, district) in ohio.iter().enumerate() {
        // Cut by position rather than by a value threshold: the quartiles have to partition the
        // panel exactly, and a threshold cut ties districts on the boundary into whichever side a
        // comparison happens to land.
        let q = ((i * 4) / n).min(3);
        out[q].districts += 1;
        out[q].local_per_pupil += district.local_revenue / district.enrollment;
        out[q].state_per_pupil += district.state_revenue / district.enrollment;
        out[q].federal_per_pupil += district.federal_revenue / district.enrollment;
    }
    for quartile in &mut out {
        if quartile.districts > 0 {
            let count = quartile.districts as f64;
            quartile.local_per_pupil /= count;
            quartile.state_per_pupil /= count;
            quartile.federal_per_pupil /= count;
        }
    }
    out
}

fn percentile_of(sorted: &[f64], value: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let below = sorted.partition_point(|x| *x < value);
    below as f64 / sorted.len() as f64
}

/// Place every Ohio district in the national distribution, keyed by IRN.
///
/// The distribution is built once over all 16,872 districts and each Ohio district is located in
/// it — so a percentile here is against America, not against the 968 Ohio rows the survey carries.
#[must_use]
pub fn positions() -> (BTreeMap<String, NationalPosition>, NationalMedians) {
    let panel = national_panel();

    let sorted = |mut v: Vec<f64>| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        v
    };
    // The distribution is the comparable set only; Ohio's non-comparable agencies are carried in
    // the fixture so their figures exist, and are not part of anything anyone is ranked against.
    let pool: Vec<&NationalDistrict> = panel.iter().filter(|d| d.comparable).collect();
    let shares = sorted(pool.iter().map(|d| d.local_share()).collect());
    let revenue = sorted(pool.iter().map(|d| d.revenue_per_pupil()).collect());
    let spending = sorted(pool.iter().filter_map(|d| d.spending_per_pupil()).collect());

    let medians = NationalMedians {
        districts: pool.len(),
        local_share: crate::median(&shares).unwrap_or(0.0),
        revenue_per_pupil: crate::median(&revenue).unwrap_or(0.0),
        spending_per_pupil: crate::median(&spending).unwrap_or(0.0),
    };

    let mut out = BTreeMap::new();
    for d in panel.iter().filter(|d| !d.irn.is_empty() && d.comparable) {
        let spend = d.spending_per_pupil().unwrap_or(0.0);
        out.insert(
            d.irn.clone(),
            NationalPosition {
                local_share: d.local_share(),
                local_share_percentile: percentile_of(&shares, d.local_share()),
                revenue_per_pupil: d.revenue_per_pupil(),
                revenue_per_pupil_percentile: percentile_of(&revenue, d.revenue_per_pupil()),
                spending_per_pupil: spend,
                spending_per_pupil_percentile: if spend > 0.0 {
                    percentile_of(&spending, spend)
                } else {
                    0.0
                },
            },
        );
    }
    (out, medians)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_survey_covers_every_state_and_most_of_its_districts() {
        let panel = national_panel();
        // 10,382 comparable agencies plus Ohio's 357 non-comparable ones, carried so their
        // figures exist without being part of any distribution.
        assert!(panel.len() > 10_000, "{} agencies", panel.len());
        let comparable = panel.iter().filter(|d| d.comparable).count();
        assert!(comparable > 10_000 && comparable < panel.len());
        let states: std::collections::BTreeSet<&str> =
            panel.iter().map(|d| d.state.as_str()).collect();
        assert!(states.len() >= 50, "{} states", states.len());
    }

    #[test]
    fn every_ohio_district_in_the_survey_carries_an_irn() {
        // The join this whole module was blocked on. `census-f33` recorded it as unavailable for
        // as long as it has been wired; it was one column in a file the corpus had already named.
        let panel = national_panel();
        let ohio: Vec<&NationalDistrict> = panel.iter().filter(|d| d.state == "OH").collect();
        assert!(ohio.len() > 900, "{} Ohio rows", ohio.len());
        assert!(
            ohio.iter().all(|d| !d.irn.is_empty()),
            "{} Ohio rows without an IRN",
            ohio.iter().filter(|d| d.irn.is_empty()).count()
        );
        // And no non-Ohio district carries one, which would mean a bad join rather than a gap.
        assert!(panel.iter().all(|d| d.state == "OH" || d.irn.is_empty()));
    }

    #[test]
    fn a_percentile_is_against_america_and_not_against_ohio() {
        /*
         * The distinction this module exists for. Ohio is seventh of fifty-one on local share, so
         * its districts sit high in the national distribution — and a percentile computed over
         * the 968 Ohio rows instead would put half of them below the median by construction.
         *
         * The check: Ohio's median district must sit visibly above the national median, which it
         * cannot do if the distribution were Ohio's own.
         */
        let (positions, medians) = positions();
        assert!(positions.len() > 600);

        let mut ohio_shares: Vec<f64> = positions.values().map(|p| p.local_share).collect();
        ohio_shares.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
        let ohio_median = ohio_shares[ohio_shares.len() / 2];
        assert!(
            ohio_median > medians.local_share,
            "Ohio's median local share is {ohio_median:.4} against a national {:.4}; the state \
             ranks seventh of fifty-one and its districts should sit above the middle",
            medians.local_share
        );

        let mut pcts: Vec<f64> = positions
            .values()
            .map(|p| p.local_share_percentile)
            .collect();
        pcts.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
        let median_pct = pcts[pcts.len() / 2];
        assert!(
            median_pct > 0.55,
            "the median Ohio district sits at the {median_pct:.3} percentile nationally; if this \
             were near 0.5 the distribution would be Ohio's rather than America's"
        );
    }

    #[test]
    fn the_national_medians_are_plausible_and_the_shares_are_fractions() {
        let (positions, medians) = positions();
        assert!((0.3..0.6).contains(&medians.local_share), "{:?}", medians);
        assert!((10_000.0..25_000.0).contains(&medians.revenue_per_pupil));
        assert!((10_000.0..25_000.0).contains(&medians.spending_per_pupil));
        for (irn, p) in &positions {
            assert!((0.0..=1.0).contains(&p.local_share), "{irn}: {p:?}");
            assert!((0.0..=1.0).contains(&p.local_share_percentile), "{irn}");
        }
    }

    /// An ordinary Ohio district is well above average for America, and the shift is largest in
    /// the middle of the distribution.
    ///
    /// This is why a national percentile is worth publishing beside an Ohio one rather than instead
    /// of it. The two answer different questions, and for a state this reliant on local revenue the
    /// answers are far apart in a way that is not uniform:
    ///
    ///     Ohio percentile   local share   national percentile
    ///           10%            0.251             24%
    ///           25%            0.365             48%
    ///           50%            0.463             66%
    ///           75%            0.596             82%
    ///           90%            0.713             91%
    ///
    /// The tails roughly agree and the middle does not. Ohio's *median* district is at the 66th
    /// national percentile; its quarter-poorest is at the national median. So the districts whose
    /// position changes most between the two views are the unremarkable ones — which is the
    /// opposite of the intuition that a national comparison mainly relocates the extremes.
    #[test]
    fn an_ordinary_ohio_district_is_well_above_the_national_middle() {
        let (positions, _) = positions();
        let mut pcts: Vec<f64> = positions
            .values()
            .map(|p| p.local_share_percentile)
            .collect();
        pcts.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));

        let at = |q: f64| pcts[((pcts.len() as f64) * q) as usize];
        let (p25, p50, p75) = (at(0.25), at(0.5), at(0.75));
        assert!(
            p50 > 0.6,
            "Ohio's median district sits at the {p50:.3} national percentile; the recorded figure \
             is 0.66 and a materially lower one means the comparable set has changed"
        );

        // The shift is bigger in the middle than at the top: Ohio's median moves 16 points of
        // national percentile against its own, and its 75th only 7.
        let middle_shift = p50 - 0.5;
        let upper_shift = p75 - 0.75;
        assert!(
            middle_shift > upper_shift,
            "the median shifts {middle_shift:.3} and the 75th {upper_shift:.3}; the national view \
             is expected to relocate ordinary districts more than extreme ones"
        );
        assert!(
            p25 > 0.4,
            "Ohio's quarter-poorest sits at {p25:.3} nationally"
        );

        // And more Ohio districts sit in the national top fifth than a fifth of them would.
        let top = positions
            .values()
            .filter(|p| p.local_share_percentile > 0.8)
            .count();
        assert!(
            top as f64 / positions.len() as f64 > 0.25,
            "{top} of {} Ohio districts are in the national top fifth",
            positions.len()
        );
    }

    /// The state figure this corpus already held, reproduced from a different file.
    ///
    /// `census-f33` recorded Ohio at **51.8%** local share, computed from the Bureau's state-level
    /// table. This module builds the same quantity from the district-level file, over agencies
    /// selected by a filter that table never applied — and gets 51.7%.
    ///
    /// Two independently assembled sources agreeing to a tenth of a point is the check that the
    /// comparability filter is selecting the right agencies. A filter that let Ohio's community
    /// schools in would not reproduce it.
    #[test]
    fn the_district_file_reproduces_the_state_level_local_share() {
        let panel = national_panel();
        let ohio: Vec<&NationalDistrict> = panel
            .iter()
            .filter(|d| d.state == "OH" && d.comparable)
            .collect();
        let local: f64 = ohio.iter().map(|d| d.local_revenue).sum();
        let total: f64 = ohio.iter().map(|d| d.total_revenue).sum();
        let share = local / total;
        assert!(
            (share - 0.518).abs() < 0.005,
            "the district file gives Ohio a {share:.4} local share against the 0.518 the \
             state-level table gives; the two should agree"
        );
    }

    /// The figures the corpus quotes, pinned so a source revision fails here rather than passing
    /// silently into a node.
    #[test]
    fn the_wealth_quartiles_are_the_ones_the_equity_node_carries() {
        let q = ohio_by_local_wealth();
        assert_eq!(
            q.iter().map(|x| x.districts).sum::<usize>(),
            611,
            "the comparable Ohio panel changed size"
        );

        for (i, (local, state, federal)) in [
            (4342.0, 8970.0, 2738.0),
            (6678.0, 7199.0, 2231.0),
            (8541.0, 6232.0, 2181.0),
            (13932.0, 4522.0, 1825.0),
        ]
        .iter()
        .enumerate()
        {
            assert!(
                (q[i].local_per_pupil - local).abs() < 1.0,
                "Q{} local is {:.0}, not {local:.0}",
                i + 1,
                q[i].local_per_pupil
            );
            assert!(
                (q[i].state_per_pupil - state).abs() < 1.0,
                "Q{} state is {:.0}, not {state:.0}",
                i + 1,
                q[i].state_per_pupil
            );
            assert!(
                (q[i].federal_per_pupil - federal).abs() < 1.0,
                "Q{} federal is {:.0}, not {federal:.0}",
                i + 1,
                q[i].federal_per_pupil
            );
        }
    }

    /// The claim itself, stated as an inequality rather than as four numbers.
    ///
    /// Pinning the figures catches a source revision. This catches a *reversal* — the case where
    /// the numbers all move and the finding the corpus rests on quietly stops being true.
    #[test]
    fn both_higher_levels_are_compensatory_and_neither_closes_the_gap() {
        let q = ohio_by_local_wealth();

        for i in 1..4 {
            assert!(
                q[i].local_per_pupil > q[i - 1].local_per_pupil,
                "the quartiles are not ordered on the variable they are cut by"
            );
            assert!(
                q[i].state_per_pupil < q[i - 1].state_per_pupil,
                "state aid is not monotonically compensatory at Q{}",
                i + 1
            );
            assert!(
                q[i].federal_per_pupil < q[i - 1].federal_per_pupil,
                "federal aid is not monotonically compensatory at Q{}",
                i + 1
            );
        }

        let gap = q[3].local_per_pupil - q[0].local_per_pupil;
        let state = q[0].state_per_pupil - q[3].state_per_pupil;
        let federal = q[0].federal_per_pupil - q[3].federal_per_pupil;
        assert!(
            state + federal < gap,
            "the two together now close the whole local gap, which would reverse the finding"
        );
        assert!(
            state > federal * 4.0,
            "state equalization is no longer the dominant offset"
        );
    }
}
