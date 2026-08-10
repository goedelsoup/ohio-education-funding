//! Eleven Octobers of the free and reduced-price lunch report.
//!
//! # What this is a series of
//!
//! Not enrollment. MR-81 is the Office for Child Nutrition's meal-program report, and it carries
//! an enrollment column because a lunch claim needs a denominator. Its value here is the **free
//! and reduced-price counts**: R.C. 3317.03(B)(21) hands the definition of "economically
//! disadvantaged" to the department, free-lunch eligibility has been the department's operative
//! test, and this is the longest run of it available. Ohio's
//! [disadvantaged pupil impact aid](../../../.yidam/corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)
//! is paid on that count.
//!
//! # Three things that will produce a wrong reading
//!
//! **The denominator changes definition in FY2010.** `AdmCount` becomes `CECount` — "the highest
//! daily number of students with access to the program" — which is neither ADM nor the count that
//! preceded it. [`Sponsor::basis`] carries which, and [`poverty_share_by_year`] reports it, so a
//! series crossing FY2009 is spliced knowingly or not at all.
//!
//! **Sponsors are not districts.** "Public" covers county boards of developmental disabilities
//! and community schools as well as traditional districts, and the report also carries non-public
//! schools, residential child care institutions and camps. The sponsor count rising from 730 to
//! 944 across the window is mostly community schools opening, not districts appearing.
//!
//! **One published cell is wrong by two orders of magnitude.** See
//! [`implausible_sponsors`]: the FY2005 file gives Wilson Elementary School an `AdmCount` of
//! 342,332, which puts Portsmouth City at 344,048 students across nine sites. The extractor
//! transcribes it, because an extractor that quietly repairs its source is worse than one that
//! ships a visible error — but nothing should aggregate FY2005 without excluding it.

use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../fixtures/mr81-sponsor-panel.csv");

const EXPECTED_HEADER: &str = "fiscal_year,sponsor_irn,sponsor_name,county,sponsor_type,sites,\
enrollment,enrollment_basis,free_lunch,reduced_lunch";

/// One sponsor in one October.
#[derive(Debug, Clone, PartialEq)]
pub struct Sponsor {
    /// The October counted.
    pub year: u16,
    /// Six-digit IRN, zero-padded — the published files disagree about padding.
    pub irn: String,
    /// The sponsor's name as published.
    pub name: String,
    /// What kind of sponsor: `Public`, `Non-Public`, and two smaller kinds.
    pub kind: String,
    /// How many school sites the sponsor reported.
    pub sites: usize,
    /// The meal-program denominator, whose definition changes in FY2010.
    pub enrollment: f64,
    /// `adm` through FY2009, `ce` from FY2010.
    pub basis: String,
    /// Free lunch applications.
    pub free: f64,
    /// Reduced-price lunch applications.
    pub reduced: f64,
}

impl Sponsor {
    /// Free and reduced together, as a share of the denominator in force that year.
    #[must_use]
    pub fn poverty_share(&self) -> Option<f64> {
        (self.enrollment > 0.0).then(|| (self.free + self.reduced) / self.enrollment)
    }
}

/// Every row of the panel.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against.
#[must_use]
pub fn panel() -> Vec<Sponsor> {
    let mut lines = FIXTURE.lines();
    assert_eq!(
        lines.next().unwrap_or_default().trim(),
        EXPECTED_HEADER,
        "the MR-81 fixture header changed; update dispersion::mr81"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            let num = |i: usize| f.get(i).and_then(|v| v.parse::<f64>().ok());
            Some(Sponsor {
                year: f.first()?.parse().ok()?,
                irn: f.get(1)?.to_string(),
                name: f.get(2)?.to_string(),
                kind: f.get(4)?.to_string(),
                sites: f.get(5)?.parse().ok()?,
                enrollment: num(6)?,
                basis: f.get(7)?.to_string(),
                free: num(8)?,
                reduced: num(9)?,
            })
        })
        .collect()
}

/// Sponsor-years whose enrollment cannot be right, so an aggregate can exclude them by name.
///
/// A site averaging over twenty thousand students is not a school. The threshold is deliberately
/// far above anything real — Ohio's largest district averages a few hundred per site — so this
/// catches published corruption rather than unusual districts.
#[must_use]
pub fn implausible_sponsors() -> Vec<Sponsor> {
    panel()
        .into_iter()
        .filter(|s| s.sites > 0 && s.enrollment / s.sites as f64 > 20_000.0)
        .collect()
}

/// The share of the meal-program denominator eligible for free or reduced-price lunch, by year.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PovertyYear {
    /// Public sponsors counted.
    pub sponsors: usize,
    /// Free and reduced as a share of enrollment.
    pub share: f64,
    /// Whether the denominator that year is `adm` or `ce`.
    pub basis_is_ce: bool,
}

/// Public sponsors only, excluding the sponsor-years [`implausible_sponsors`] names.
#[must_use]
pub fn poverty_share_by_year() -> BTreeMap<u16, PovertyYear> {
    let bad: Vec<(u16, String)> = implausible_sponsors()
        .into_iter()
        .map(|s| (s.year, s.irn))
        .collect();
    let mut totals: BTreeMap<u16, (f64, f64, usize, bool)> = BTreeMap::new();
    for s in panel() {
        if s.kind != "Public" || bad.contains(&(s.year, s.irn.clone())) {
            continue;
        }
        let t = totals.entry(s.year).or_default();
        t.0 += s.enrollment;
        t.1 += s.free + s.reduced;
        t.2 += 1;
        t.3 = s.basis == "ce";
    }
    totals
        .into_iter()
        .map(|(year, (enrollment, poor, sponsors, ce))| {
            (
                year,
                PovertyYear {
                    sponsors,
                    share: if enrollment > 0.0 {
                        poor / enrollment
                    } else {
                        0.0
                    },
                    basis_is_ce: ce,
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_holds_the_unbroken_sponsor_centric_era() {
        let years: Vec<u16> = poverty_share_by_year().keys().copied().collect();
        assert_eq!(years, (2001..=2011).collect::<Vec<u16>>());
    }

    /// The published defects, pinned by name so a new one cannot hide behind them.
    ///
    /// Two kinds, and only one of them reaches the poverty series. FY2001 leaks what look like
    /// IRNs into the count column for three residential and juvenile sponsors — 910213, 910132
    /// and 69807 — none of them Public, so the Public-only aggregate never saw them. Portsmouth
    /// 2005 is Public and did.
    #[test]
    fn the_published_defects_are_the_four_known_ones() {
        let bad = implausible_sponsors();
        assert_eq!(
            bad.len(),
            4,
            "found {} implausible sponsor-years",
            bad.len()
        );

        let public: Vec<&Sponsor> = bad.iter().filter(|s| s.kind == "Public").collect();
        assert_eq!(public.len(), 1, "a second Public sponsor became impossible");
        assert_eq!(public[0].year, 2005);
        assert_eq!(public[0].name, "Portsmouth City SD");
        assert!(
            public[0].enrollment > 300_000.0,
            "the defect is a 342,332 in one site's AdmCount; it now reads {}",
            public[0].enrollment
        );
        assert_eq!(
            bad.iter().filter(|s| s.year == 2001).count(),
            3,
            "the three FY2001 leaked-identifier rows are not all present"
        );
    }

    /// The writer refuses a field carrying the delimiter, because this fixture needed it.
    #[test]
    fn no_sponsor_name_carries_a_comma() {
        // Ohio has sponsors called "Holy Trinity, Swanton Ele Sch" and "Edge Academy, The", and
        // the first build of this panel shipped 99 rows whose columns were shifted by one.
        assert!(
            panel().iter().all(|s| !s.name.contains(',')),
            "a sponsor name carries a comma and the fixture is unquoted"
        );
        assert!(
            panel().iter().any(|s| s.name.contains(';')),
            "no name carries the substitute, so the substitution silently stopped happening"
        );
    }

    /// The finding: measured poverty rises by nearly twenty points in eleven years.
    #[test]
    fn the_poverty_share_rises_across_the_whole_window() {
        let by_year = poverty_share_by_year();
        let first = by_year[&2001].share;
        let last = by_year[&2011].share;
        assert!(
            (first - 0.277).abs() < 0.005,
            "FY2001 share is {first:.4}, not 0.277"
        );
        assert!(
            (last - 0.463).abs() < 0.005,
            "FY2011 share is {last:.4}, not 0.463"
        );
        assert!(
            last - first > 0.15,
            "the rise is {:.1} points, not the eighteen-plus recorded",
            100.0 * (last - first)
        );
    }

    /// The denominator changes in FY2010 and the panel says so rather than smoothing it.
    #[test]
    fn the_denominator_changes_definition_in_2010_and_is_labelled() {
        let by_year = poverty_share_by_year();
        assert!(
            (2001..=2009).all(|y| !by_year[&y].basis_is_ce),
            "a pre-2010 year is labelled CE"
        );
        assert!(
            by_year[&2010].basis_is_ce && by_year[&2011].basis_is_ce,
            "FY2010 and FY2011 are CE and should be labelled so"
        );
    }
}
