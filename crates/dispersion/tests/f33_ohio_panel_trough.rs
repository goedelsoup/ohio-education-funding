//! Where Ohio's FY2010–FY2014 real spending trough actually fell.
//!
//! The corpus has recorded the trough since the `deflate` calculator first ran, from the Ohio
//! Auditor's statewide series: real operating expenditure per pupil down about 7% from FY2010 to
//! FY2014, back to its FY2006 level, not recovering until FY2018. Statewide it was a single
//! number, and a single number cannot distinguish a broad contraction from a few districts cut
//! hard enough to move an average.
//!
//! This file answers that with the district panel. Two findings, and the second is a negative.
//!
//! **It was broad.** 81% of comparable districts spent less per pupil in real terms in FY2013
//! than in FY2010. The median district fell 5.6%.
//!
//! **Nothing about a district predicts how far it fell.** Not its dependence on state aid, not
//! its federal share, not its local share, not its later poverty or property wealth. The obvious
//! hypothesis — that a contraction coinciding with the withdrawal of federal stimulus and the
//! Bridge formula decade would land hardest on the districts most dependent on the state — is
//! not supported. Every one of those correlations is inside ±0.04.
//!
//! # Two series, two universes, one shape
//!
//! The figures here are Census F-33 `TCURELSC` over `V33` for districts the survey codes as
//! regular and non-charter. The Auditor's series is NCES operating expenditure over a wider
//! entity set that includes educational service centers, community schools and tuition paid out
//! of state. The levels differ by roughly $1,500 per pupil and are not interchangeable. The
//! *shape* agrees: enrollment-weighted, this panel falls 6.8% from FY2010 to FY2013 against the
//! Auditor's 6.9% from FY2010 to FY2014.
//!
//! FY2014 is absent from the NCES archive under every naming the surrounding years use, so the
//! trough is measured to FY2013 here. That understates it: the Auditor's series still falls
//! between FY2012 and FY2014.
//!
//! Cited by `corpus/metric/per-pupil-operating-expenditure.yml`.

use dispersion::{wealth_neutrality, Dispersion};

const PANEL: &str = include_str!("../fixtures/f33-ohio-panel.csv");
const PROFILE: &str = include_str!("../fixtures/cupp-fy24-district-data.csv");
const CPI: &str = include_str!("../../connect/fixtures/cpi-u-june.tsv");

/// Column indices in the panel fixture.
mod col {
    pub const IRN: usize = 2;
    pub const COMPARABLE: usize = 3;
    pub const ENROLLMENT: usize = 4;
    pub const TOTAL_REVENUE: usize = 5;
    pub const FEDERAL_REVENUE: usize = 6;
    pub const STATE_REVENUE: usize = 7;
    pub const LOCAL_REVENUE: usize = 8;
    pub const CURRENT_SPENDING: usize = 10;
}

/// The peak of the trough, and the last year before recovery that the archive holds.
const PEAK: u16 = 2010;
const BOTTOM: u16 = 2013;
/// Everything is restated in the dollars of the panel's final year.
const BASE: u16 = 2022;

#[derive(Debug, Clone)]
struct Row {
    year: u16,
    irn: String,
    enrollment: f64,
    spending_per_pupil: f64,
    state_share: f64,
    federal_share: f64,
    local_share: f64,
}

fn cpi(year: u16) -> f64 {
    CPI.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').map(str::trim).collect();
            (f.len() >= 4 && f[1].parse::<u16>().ok() == Some(year))
                .then(|| f[3].parse::<f64>().ok())
                .flatten()
        })
        .next()
        .unwrap_or_else(|| panic!("no CPI observation for FY{year}"))
}

/// Restate a figure from `year` in [`BASE`] dollars.
fn real(value: f64, year: u16) -> f64 {
    value * cpi(BASE) / cpi(year)
}

fn panel() -> Vec<Row> {
    PANEL
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            let get = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok());
            if f.get(col::COMPARABLE) != Some(&"1") {
                return None;
            }
            let irn = (*f.get(col::IRN)?).to_string();
            if irn.is_empty() {
                return None;
            }
            let (enrollment, spending, revenue) = (
                get(col::ENROLLMENT)?,
                get(col::CURRENT_SPENDING)?,
                get(col::TOTAL_REVENUE)?,
            );
            if enrollment <= 0.0 || revenue <= 0.0 {
                return None;
            }
            Some(Row {
                year: f.first()?.parse().ok()?,
                irn,
                enrollment,
                spending_per_pupil: spending / enrollment,
                state_share: get(col::STATE_REVENUE)? / revenue,
                federal_share: get(col::FEDERAL_REVENUE)? / revenue,
                local_share: get(col::LOCAL_REVENUE)? / revenue,
            })
        })
        .collect()
}

fn year_of(rows: &[Row], year: u16) -> Vec<Row> {
    rows.iter().filter(|r| r.year == year).cloned().collect()
}

/// Districts present in both endpoint years, with the real change between them.
fn changes(rows: &[Row]) -> Vec<(Row, f64)> {
    let bottom = year_of(rows, BOTTOM);
    year_of(rows, PEAK)
        .into_iter()
        .filter_map(|start| {
            let end = bottom.iter().find(|r| r.irn == start.irn)?;
            let change =
                real(end.spending_per_pupil, BOTTOM) / real(start.spending_per_pupil, PEAK) - 1.0;
            Some((start, change))
        })
        .collect()
}

fn correlate(pairs: &[(f64, f64)]) -> f64 {
    let (x, y): (Vec<f64>, Vec<f64>) = pairs.iter().copied().unzip();
    wealth_neutrality(&x, &y)
        .expect("paired series")
        .correlation
}

fn close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{what}: got {actual:.4}, expected {expected:.4}"
    );
}

#[test]
fn the_panel_now_opens_before_the_trough_rather_than_inside_it() {
    let rows = panel();
    let years: std::collections::BTreeSet<u16> = rows.iter().map(|r| r.year).collect();
    assert!(
        years.contains(&2009) && years.contains(&PEAK) && years.contains(&2011),
        "the three added years are present: {years:?}"
    );
    // FY2014 is absent from the NCES archive; the gap is real and is stated rather than papered.
    assert!(!years.contains(&2014));
    assert_eq!(*years.iter().next().expect("a first year"), 2009);
    assert!(year_of(&rows, PEAK).len() > 590);
}

/// **The trough is broad.** Four districts in five spent less per pupil in real terms.
#[test]
fn four_districts_in_five_fell_in_real_terms() {
    let rows = panel();
    let changes = changes(&rows);
    assert!(changes.len() > 600, "{} matched districts", changes.len());

    let falling = changes.iter().filter(|(_, c)| *c < 0.0).count();
    let share = falling as f64 / changes.len() as f64;
    close(share, 0.807, 0.02, "share of districts falling");

    let mut values: Vec<f64> = changes.iter().map(|(_, c)| *c).collect();
    values.sort_by(f64::total_cmp);
    close(
        values[values.len() / 2],
        -0.056,
        0.006,
        "median real change",
    );
    close(
        values[values.len() / 4],
        -0.092,
        0.008,
        "first-quartile real change",
    );
}

/// The enrollment-weighted panel reproduces the Auditor's statewide shape from a different
/// source over a different entity set — which is what licenses reading the two together.
#[test]
fn the_weighted_panel_reproduces_the_auditors_decline() {
    let rows = panel();
    let weighted = |year: u16| {
        let rows = year_of(&rows, year);
        let pupils: f64 = rows.iter().map(|r| r.enrollment).sum();
        rows.iter()
            .map(|r| real(r.spending_per_pupil, year) * r.enrollment)
            .sum::<f64>()
            / pupils
    };
    let peak = weighted(PEAK);
    let bottom = weighted(BOTTOM);
    close(
        peak,
        14_040.0,
        60.0,
        "FY2010 weighted real spending per pupil",
    );
    close(
        bottom,
        13_085.0,
        60.0,
        "FY2013 weighted real spending per pupil",
    );

    let decline = bottom / peak - 1.0;
    close(decline, -0.068, 0.006, "weighted real decline");
    // The Auditor's statewide series falls 6.9% over FY2010-FY2014 on a wider entity set.
    assert!((decline - -0.069).abs() < 0.01);

    // And the recovery does not clear FY2010 until FY2017, two years later than the Auditor's
    // wider series clears it — the two universes recover on slightly different schedules.
    assert!(weighted(2016) < peak);
    assert!(weighted(2017) > peak);
}

/// **The negative result.** Nothing about a district predicts how far it fell — least of all the
/// thing that should have.
///
/// A contraction coinciding with the withdrawal of federal stimulus, inside the Bridge formula
/// decade, ought to land hardest on districts most dependent on state aid. It did not. State,
/// federal and local revenue shares are all uncorrelated with the depth of the fall.
#[test]
fn no_revenue_structure_predicts_the_depth_of_the_fall() {
    let changes = changes(&panel());
    for (label, pick) in [
        ("state share", (|r: &Row| r.state_share) as fn(&Row) -> f64),
        ("federal share", |r: &Row| r.federal_share),
        ("local share", |r: &Row| r.local_share),
    ] {
        let pairs: Vec<(f64, f64)> = changes.iter().map(|(r, c)| (pick(r), *c)).collect();
        let r = correlate(&pairs);
        assert!(
            r.abs() < 0.06,
            "{label} against the depth of the fall is {r:+.3}, which is not nothing"
        );
    }
}

/// Nor does later district character, on the vintage the corpus holds.
///
/// The poverty and valuation figures are FY2023-24, more than a decade after the trough, so this
/// is directional rather than contemporaneous. It comes back as null as the revenue shares do,
/// which is why the corpus states the negative rather than leaving it open.
#[test]
fn later_poverty_and_wealth_do_not_predict_it_either() {
    let changes = changes(&panel());
    let profile: Vec<(String, f64, f64)> = PROFILE
        .lines()
        .skip(1)
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            Some((
                (*f.first()?).to_string(),
                f.get(3)?.parse().ok()?,
                f.get(4)?.parse().ok()?,
            ))
        })
        .collect();

    for (label, pick) in [("disadvantage", 0usize), ("valuation per pupil", 1usize)] {
        let pairs: Vec<(f64, f64)> = changes
            .iter()
            .filter_map(|(row, change)| {
                let entry = profile.iter().find(|(irn, _, _)| *irn == row.irn)?;
                Some((if pick == 0 { entry.1 } else { entry.2 }, *change))
            })
            .collect();
        assert!(pairs.len() > 590, "{label}: only {} matched", pairs.len());
        let r = correlate(&pairs);
        assert!(r.abs() < 0.06, "{label} against the depth is {r:+.3}");
    }
}

/// What little does track the depth is size and prior level, both weak and the second partly
/// mechanical. Recorded so the negative above is not read as "nothing varied".
#[test]
fn size_and_prior_level_track_it_weakly() {
    let changes = changes(&panel());
    let by_size: Vec<(f64, f64)> = changes
        .iter()
        .map(|(r, c)| (r.enrollment.ln(), *c))
        .collect();
    let by_level: Vec<(f64, f64)> = changes
        .iter()
        .map(|(r, c)| (real(r.spending_per_pupil, PEAK), *c))
        .collect();
    close(correlate(&by_size), -0.186, 0.02, "log enrollment");
    close(correlate(&by_level), -0.176, 0.02, "FY2010 spending level");

    // Both are far smaller than the spread they are trying to explain.
    let values: Vec<f64> = changes.iter().map(|(_, c)| *c).collect();
    let spread = Dispersion::of(&values.iter().map(|v| v + 2.0).collect::<Vec<_>>()).unwrap();
    assert!(spread.std_dev > 0.06);
}

/// The worst decile is real and it is not what moved the state.
#[test]
fn the_deepest_decile_is_a_tenth_of_districts_and_an_eighth_of_pupils() {
    let mut changes = changes(&panel());
    changes.sort_by(|a, b| a.1.total_cmp(&b.1));
    let tenth = changes.len() / 10;
    let worst = &changes[..tenth];

    let mean = worst.iter().map(|(_, c)| *c).sum::<f64>() / tenth as f64;
    close(mean, -0.176, 0.02, "mean change in the worst decile");

    let all_pupils: f64 = changes.iter().map(|(r, _)| r.enrollment).sum();
    let their_pupils: f64 = worst.iter().map(|(r, _)| r.enrollment).sum();
    close(
        their_pupils / all_pupils,
        0.118,
        0.02,
        "share of pupils in the worst decile",
    );
    // A tail exists; the median district's 5.6% fall is not an artifact of it.
    assert!(mean < -0.15);
}
