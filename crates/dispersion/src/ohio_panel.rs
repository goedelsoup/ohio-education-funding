//! Ohio in the school finance survey, across every year the archive publishes.
//!
//! # What this is for, and how it differs from its sibling
//!
//! [`crate::national_peers`] places one Ohio district among America's in a single year. It
//! answers "is this unusual", and it is the only thing in this repository that can, because every
//! other source is Ohio describing itself.
//!
//! This module answers a different question — **how Ohio changed** — and needs the opposite
//! shape: only Ohio, but fifteen years of it. The two are separate fixtures for that reason rather
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
//! # Four caveats, each capable of producing a wrong reading
//!
//! **FY2014 is missing from the archive**, under every naming the neighbouring years use. Nine
//! intervals across ten years, and one of them is two years wide. Nothing here interpolates it.
//!
//! **FY2023 and FY2024 come from a different publisher.** NCES stops at FY2022 — `sdf23_1a.zip`
//! does not exist — and the Bureau's own individual unit file carries the same collection two
//! years further on. It is the same survey and the same column names, and it differs in three
//! ways that are reconciled in the extractor rather than left in the fixture: money in thousands,
//! `NCESID` for `LEAID`, and `TCURELSC` inclusive of the `V91`/`V92` payments NCES nets out. It
//! also does not carry Ohio's community schools at all, so the Bureau years hold 655 agencies
//! against the NCES years' ~950 while the comparable count is unchanged at 609.
//!
//! **The denominator is the Bureau's** — `V33`, fall membership — and not Ohio's enrolled ADM. A
//! per-pupil figure from this module must never be shown beside one from the funding model.
//!
//! **The state column changes basis at FY2016**, and it is the caveat that has already produced
//! a wrong published finding. From FY2016 the Bureau nets each district's community-school
//! deduct out of its general formula assistance and before FY2016 it does not, so a big-city
//! district appears to lose a fifth to a third of its state money in a single year and did not.
//! [`crate::survey_basis`] holds the correction, the evidence that locates the break, and the
//! list of what it overturned. Nothing that compares this column across FY2015 and FY2016 is
//! sound without it.
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
federal_revenue,state_revenue,local_revenue,property_tax,current_spending,student_transportation,\
charter_payments,general_formula_assistance,state_nonspecified,capital_outlay";

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
    /// All revenue, from every source, in dollars.
    ///
    /// Dollars, not the thousands this said until the Bureau's own file was spliced on. The
    /// survey publishes thousands and NCES's rendering of it does not; a 327-pupil district at
    /// $3,298,000 settles which one this fixture carries. The Bureau's years are multiplied on
    /// the way in so that the column keeps meaning one thing across the seam.
    pub total_revenue: f64,
    /// The federal share of it.
    pub federal_revenue: f64,
    /// The state share.
    pub state_revenue: f64,
    /// And the local share.
    pub local_revenue: f64,
    /// `T06` â school property tax. `None` where the agency reports none, which for a fiscally
    /// dependent agency means the tax belongs to a parent government rather than that none was
    /// levied.
    pub property_tax: Option<f64>,
    /// `TCURELSC` â current spending on elementary and secondary education, in dollars.
    ///
    /// Absent from this record until the real-spending trough was computed from it. That
    /// analysis lived in `tests/f33_ohio_panel_trough.rs` on a second, private parser of this
    /// same fixture â which read the column this reader did not carry, and disagreed with it
    /// about which rows are rows. See issue #157.
    ///
    /// **It is the one column the two publishers define differently**, and the only one the
    /// extractor has to reconcile. The Bureau folds `V91` and `V92` — the instruction lines that
    /// pay a private or a charter school to teach — inside `E13`; NCES nets them out. Across
    /// FY2022, where both publish Ohio, the identity holds on all 710 shared agencies and in one
    /// direction: the Bureau's total is NCES's plus `V91 + V92`. It moves 79 of them. This column
    /// is NCES's definition in every year, the Bureau's years included.
    pub current_spending: Option<f64>,
    /// `V45` — support services, student transportation, in dollars.
    ///
    /// The only fuel-exposed line the survey separates, and the reason this column exists: the
    /// state's two transportation rates are trimmed means of what districts reported spending,
    /// so the cost side of that feedback is measurable here and nowhere else across years.
    /// `None` where the agency reports none, which for a district that buses nobody is a real
    /// zero reported as absent rather than as `0`.
    pub student_transportation: Option<f64>,
    /// `V92` — what the district paid community schools, in dollars.
    ///
    /// Carried because [`Self::state_revenue`] is not one quantity across FY2016 without it. From
    /// FY2016 the Bureau subtracts this figure from Ohio's general formula assistance to remove
    /// the double count Ohio's deduct creates, and before FY2016 it does not. See
    /// [`crate::survey_basis`], which is where the two eras are put on one basis, and never
    /// subtract this by hand — it is already out of the state column in the later years.
    pub charter_payments: Option<f64>,
    /// `C01` — general formula assistance, in dollars.
    ///
    /// About 94% of Ohio's state revenue and the part of it that is not capital, which is what
    /// makes it the operating series [`Self::state_revenue`] is not. It is the column the Bureau's
    /// charter adjustment acts on, so it carries the FY2016 basis break too — see
    /// [`crate::survey_basis`].
    pub general_formula_assistance: Option<f64>,
    /// `C35` — state revenue the reporting unit could not split across the survey's categories.
    ///
    /// The survey calls it "nonspecified" and in Ohio it is the school construction money: see
    /// [`crate::facilities`], which is where the identification is made and its limits stated.
    pub state_nonspecified: Option<f64>,
    /// `TCAPOUT` — total capital outlay spending, in dollars, from every source.
    ///
    /// Beside [`Self::state_nonspecified`] so the state's share of a district's capital spending
    /// is a division rather than an inference. Excluded from [`Self::current_spending`] by
    /// definition, which is why a build year shows state revenue above current spending.
    pub capital_outlay: Option<f64>,
}

impl PanelRow {
    /// Current spending per pupil, on the Bureau's own count. `None` where either is missing.
    ///
    /// Nominal. The whole point of the series is what happens to it deflated, so a caller has to
    /// restate it â `deflator::CpiSeries::cpi_u_june` is the index the corpus uses.
    #[must_use]
    pub fn spending_per_pupil(&self) -> Option<f64> {
        match (self.current_spending, self.enrollment) {
            (Some(spending), pupils) if pupils > 0.0 => Some(spending / pupils),
            _ => None,
        }
    }

    /// Student transportation spending per pupil, on the Bureau's own count.
    ///
    /// Nominal, like [`Self::spending_per_pupil`], and for the same reason.
    #[must_use]
    pub fn transportation_per_pupil(&self) -> Option<f64> {
        match (self.student_transportation, self.enrollment) {
            (Some(spending), pupils) if pupils > 0.0 => Some(spending / pupils),
            _ => None,
        }
    }

    /// Student transportation as a share of current spending. `None` where either is missing,
    /// or where the agency reports no current spending to take a share of.
    #[must_use]
    pub fn transportation_share(&self) -> Option<f64> {
        match (self.student_transportation, self.current_spending) {
            (Some(transport), Some(current)) if current > 0.0 => Some(transport / current),
            _ => None,
        }
    }

    /// One revenue source as a share of total revenue. `None` where the agency reports no
    /// revenue at all.
    #[must_use]
    pub fn share(&self, part: f64) -> Option<f64> {
        (self.total_revenue > 0.0).then(|| part / self.total_revenue)
    }
}

/// Every row of the panel.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against — which means the extractor
/// changed and this reader did not — or if a row's width differs from the header's. Both come
/// from [`edfund_core::csv::rows`], which is what holds the uniform-width invariant these
/// fixtures are written under: the hand-rolled `split(',')` this replaced checked the header
/// and then indexed by position, so a cell that grew a comma shifted every field after it and
/// the row still parsed.
#[must_use]
pub fn panel() -> Vec<PanelRow> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            Some(PanelRow {
                fiscal_year: row.str(0).parse().ok()?,
                leaid: row.str(1).to_string(),
                irn: row.str(2).to_string(),
                comparable: row.str(3) == "1",
                enrollment: row.num(4)?,
                total_revenue: row.num(5)?,
                federal_revenue: row.num(6)?,
                state_revenue: row.num(7)?,
                local_revenue: row.num(8)?,
                property_tax: row.num(9),
                current_spending: row.num(10),
                student_transportation: row.num(11),
                charter_payments: row.num(12),
                general_formula_assistance: row.num(13),
                state_nonspecified: row.num(14),
                capital_outlay: row.num(15),
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

/// One year of the comparable panel's spending, in the two forms that answer different questions.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct YearOfSpending {
    /// Comparable districts reporting spending and enrolment.
    pub districts: usize,
    /// Fall membership, summed.
    pub enrollment: f64,
    /// Current spending for elementary and secondary education, summed, in nominal dollars.
    pub total: f64,
    /// The same total in [`trough::BASE_YEAR`] dollars.
    pub real_total: f64,
    /// `total / enrollment`, nominal.
    pub per_pupil: f64,
    /// The same per-pupil figure in [`trough::BASE_YEAR`] dollars.
    pub real_per_pupil: f64,
}

/// Every year of the comparable panel, summed, nominal and real.
///
/// # Why both the total and the quotient
///
/// `metric/per-pupil-operating-expenditure` warns that per-pupil growth "is partly a numerator
/// effect and partly a denominator one, and attributing it wholly to spending decisions is
/// wrong". A series of quotients cannot show which, and the corpus carried only quotients. These
/// carry the numerator and the denominator beside it, so a caller can say how a per-pupil change
/// was produced rather than only that it happened.
///
/// The population is the same one [`trough::comparable`] uses and the same one the corpus's
/// existing panel figures are computed over: comparable districts, named, reporting both
/// enrolment and spending. Community schools are outside it in every year, which is what makes
/// the fifteen years one population and also what makes an enrolment fall here not the same
/// thing as an enrolment fall in Ohio.
#[must_use]
pub fn spending_by_year() -> BTreeMap<u16, YearOfSpending> {
    let mut out: BTreeMap<u16, YearOfSpending> = BTreeMap::new();
    for row in trough::comparable() {
        let Some(spending) = row.current_spending else {
            continue;
        };
        let year = out.entry(row.fiscal_year).or_default();
        year.districts += 1;
        year.enrollment += row.enrollment;
        year.total += spending;
    }
    for (fiscal_year, year) in &mut out {
        year.real_total = trough::real(year.total, *fiscal_year);
        if year.enrollment > 0.0 {
            year.per_pupil = year.total / year.enrollment;
            year.real_per_pupil = trough::real(year.per_pupil, *fiscal_year);
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

/// The smallest enrolment a per-pupil ratio is taken over.
///
/// # One district, and it was the finding
///
/// Kelleys Island Local SD teaches between one and eighteen children on an island in Lake Erie,
/// against a tax base of resort property. Its local revenue per pupil is **$214,400 in FY2023** —
/// four times the next district in the state and ten times the tenth. In a quartile of 152 that
/// one row adds $1,410 to the richest quartile's mean.
///
/// It is not in the population every year. `comparable` is the survey's own flag, `AGCHRT != 1`
/// and `SCHLEV == 03`, and the Bureau codes this agency differently as its enrolment moves: it is
/// out in FY2012, FY2013, FY2018 and **FY2022**, and in for FY2023 and FY2024. So the population
/// changed at exactly the year the corpus recorded a break in the series, and the break was the
/// population changing. See
/// `the_break_the_corpus_recorded_was_one_district_entering_the_population`.
///
/// # Why a floor and not a robust statistic
///
/// A median would be immune to this and to anything like it, and it was not chosen: it measures
/// something else. The gap between quartile *means* is the distance between what the rich and
/// poor quarters of Ohio actually receive per pupil, which is the quantity `doctrine/equity`
/// states and `national_peers::ohio_by_local_wealth` pins for FY2022. Swapping in medians moves
/// the whole series to 50-61% and makes every published figure incomparable, to fix one row.
///
/// # The number is read off the data, not chosen to fit it
///
/// **No comparable Ohio district has ever had enrolment between 18 and 63 in this panel.** The
/// largest Kelleys Island year is 18; the smallest any other district reaches, across all fifteen
/// years, is 63. Forty sits in the middle of that empty band — 22 pupils above everything it
/// excludes and 23 below everything it keeps — so it removes one district in every year and
/// nothing else in any year. `the_enrolment_floor_sits_in_an_empty_band` fails if that stops
/// being true.
///
/// It is a floor on the *ratio*, not on the panel. Kelleys Island is a real district and its
/// dollars are in every aggregate this module computes; what it cannot carry is a denominator of
/// five.
pub const MIN_ENROLMENT: f64 = 40.0;

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
/// grows.** The state's share of the gap it closes stays between 0.400 and 0.488 across all
/// thirteen years from FY2012 to FY2024, while the gap itself grows from $5,727 to $10,527 per
/// pupil, so the residual — the part no level closes — grows with it from $2,713 to $4,772. A
/// formula doing the same proportional job against a larger problem leaves districts further
/// apart every year, and the percentage is the thing that looks stable.
///
/// # The band used to stop at FY2022, and that was an artefact
///
/// FY2023 and FY2024 read as a break — both closing 0.372 against a gap a fifth wider — and the
/// corpus recorded it as one, with the cause open between Ohio's FY2023 reappraisals and the Fair
/// School Funding Plan's phase-in. It was neither. It was a five-pupil island district entering
/// the population in the first of those years, and [`MIN_ENROLMENT`] is the correction. See
/// `the_break_the_corpus_recorded_was_one_district_entering_the_population`.
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
        .filter(|r| r.comparable && r.enrollment >= MIN_ENROLMENT)
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

/// Long-run enrolment growth rate per district, keyed on Ohio's IRN.
///
/// The endpoint compound rate over every year the panel holds for an agency — fourteen surveyed
/// years between FY2009 and FY2024, FY2014 excepted. Comparable districts only, so it means what
/// every other figure taken from this panel means.
///
/// # Why a rate and not a series
///
/// The consumer is a projection that runs on **enrolled ADM**, and this is `V33` fall membership.
/// The two count nearly the same children — at FY2024 their levels correlate at 0.9997 across 608
/// districts — but they are not the same measure, and handing out levels would invite someone to
/// splice them onto an ADM series. A rate composes with an ADM level and a count does not.
///
/// Their two-year *rates* correlate at only 0.287, which is the finding that makes this function
/// worth having: a rate fitted over two or three points is mostly noise in either measure, and a
/// long-run rate is what a short one should be shrunk toward. The assumption that the two series
/// share a long-run trend while differing in short-run noise is recorded, and cannot be tested
/// from here — only three years of ADM exist.
///
/// Keyed on IRN rather than `LEAID` because that is what the department's model uses. An agency
/// the FY2022-23 directory does not carry has no IRN in this panel and is absent here rather than
/// keyed on an empty string.
#[must_use]
pub fn long_run_enrollment_rates() -> BTreeMap<String, f64> {
    let mut history: BTreeMap<String, BTreeMap<u16, f64>> = BTreeMap::new();
    let mut irn_of: BTreeMap<String, String> = BTreeMap::new();
    let rows = panel();
    for row in rows.iter().filter(|r| r.comparable) {
        if !row.irn.is_empty() {
            irn_of.insert(row.leaid.clone(), row.irn.clone());
        }
    }
    for row in rows.iter().filter(|r| r.comparable) {
        if row.enrollment <= 0.0 {
            continue;
        }
        if let Some(irn) = irn_of.get(&row.leaid) {
            history
                .entry(irn.clone())
                .or_default()
                .insert(row.fiscal_year, row.enrollment);
        }
    }
    history
        .into_iter()
        .filter_map(|(irn, years)| {
            let (first_year, first) = years.iter().next()?;
            let (last_year, last) = years.iter().next_back()?;
            let span = f64::from(last_year.checked_sub(*first_year)?);
            // A single year, or a year that somehow repeated, carries no rate. Absent rather than
            // zero: zero is a district that held steady, and saying that of one we cannot measure
            // would be a claim.
            if span <= 0.0 || *first <= 0.0 || *last <= 0.0 {
                return None;
            }
            Some((irn, (last / first).powf(1.0 / span) - 1.0))
        })
        .collect()
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

    /// Fifteen years and one hole, from two publishers of the same collection.
    ///
    /// FY2014 is absent because NCES never published it. FY2023 and FY2024 are present because
    /// the *Bureau* did: `sdf23_1a.zip` does not exist under any naming the earlier years answer
    /// to, and the panel stopped at FY2022 for that reason rather than because the survey did.
    #[test]
    fn the_panel_holds_fifteen_years_and_names_the_one_it_does_not() {
        let years: Vec<u16> = revenue_mix_by_year().keys().copied().collect();
        assert_eq!(
            years,
            vec![
                2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023,
                2024
            ],
            "FY2014 is absent from the archive and nothing should have invented it"
        );
    }

    /// The change of publisher moved no row of the thirteen years that preceded it.
    ///
    /// The two files disagree in three places and each is reconciled in the builder: the Bureau
    /// states money in thousands, keys on `NCESID`, and folds `V91` and `V92` — the instruction
    /// lines that pay a private or charter school to teach — inside `E13` where NCES nets them
    /// out. Left alone the third would have shifted `current_spending` upward on about one Ohio
    /// district in eight, in one direction, at exactly the year the publisher changed: a break in
    /// the series that looks like a finding.
    ///
    /// What this test can check from the fixture alone is the shape either side of the seam.
    #[test]
    fn the_bureau_years_carry_the_same_population_as_the_years_before_them() {
        let mix = revenue_mix_by_year();
        for year in [2023u16, 2024] {
            assert_eq!(
                mix[&year].districts, 609,
                "FY{year} should hold Ohio's 609 districts"
            );
        }
        // The Bureau does not survey Ohio's community schools at all — they are not governments —
        // so its Ohio file is smaller than NCES's by the 324 agencies NCES flags `AGCHRT == 1`.
        // Comparable count is what has to match; total agency count is not expected to.
        let comparable: Vec<usize> = mix.values().map(|m| m.districts).collect();
        assert!(
            comparable.iter().all(|n| (609..=613).contains(n)),
            "the comparable population moved outside 609-613: {comparable:?}"
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
        //
        // The band ran to FY2022 and stopped, because FY2023 and FY2024 read as a break. They
        // were one district: see
        // [`the_break_the_corpus_recorded_was_one_district_entering_the_population`]. With the
        // enrolment floor the band runs the whole thirteen years, 0.400 to 0.488, and closing it
        // at FY2022 would now be the thing that hides a finding.
        for (year, e) in by_year.iter().filter(|(y, _)| (2012..=2024).contains(*y)) {
            assert!(
                e.state_share() > 0.39 && e.state_share() < 0.50,
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

    /// **What the corpus recorded as a break in the series was one district joining the
    /// population.**
    ///
    /// The finding here used to be that the rate holding since FY2012 stopped holding: FY2023 and
    /// FY2024 both closing 0.372 of a gap 16% and 21% wider than FY2022's, with the residual up
    /// 46% in two years. Every one of those numbers was Kelleys Island Local SD — five pupils, an
    /// island tax base, and $214,400 of local revenue per pupil — entering a quartile mean of 152
    /// in the first year of the series it had been outside of since FY2021.
    ///
    /// | | recorded | with [`MIN_ENROLMENT`] |
    /// |---|---:|---:|
    /// | FY2022 share | 0.464 | 0.464 |
    /// | FY2023 share | **0.372** | 0.449 |
    /// | FY2024 share | **0.372** | 0.439 |
    /// | FY2024 gap | $11,586 | $10,527 |
    /// | FY2024 residual | $6,185 | $4,772 |
    ///
    /// FY2022 does not move because the district was not comparable that year. That is the whole
    /// mechanism: the flag is the Bureau's own and it changes with the agency's coding, so the
    /// population moved and the measure read it as Ohio moving.
    ///
    /// **The corrected reading is the stronger one.** The band does not stop at FY2022 — it runs
    /// 0.400 to 0.488 across all thirteen years from FY2012 to FY2024, and the two Bureau years
    /// sit inside it rather than breaking it. See
    /// [`the_equalization_rate_holds_while_the_gap_it_closes_grows`], which now closes at FY2024.
    ///
    /// This test asserts the artefact is gone rather than that it was there: what has to stay
    /// true is that no single district can move the series this far again.
    #[test]
    fn the_break_the_corpus_recorded_was_one_district_entering_the_population() {
        let by_year = equalization_by_year();

        // The two years the corpus called a break are ordinary years.
        for year in [2023u16, 2024] {
            let e = by_year[&year];
            assert!(
                (0.40..=0.49).contains(&e.state_share()),
                "FY{year} closes {:.4}, outside the band the rest of the series holds",
                e.state_share()
            );
        }

        // And the district that produced it is out of every year, not just the two.
        let island: Vec<PanelRow> = panel()
            .into_iter()
            .filter(|r| r.irn == "046797" && r.comparable)
            .collect();
        assert!(
            island.len() >= 6,
            "Kelleys Island is comparable in {} years, so this is no longer the right district",
            island.len()
        );
        assert!(
            island.iter().all(|r| r.enrollment < MIN_ENROLMENT),
            "Kelleys Island now reaches the enrolment floor and the exclusion needs re-arguing"
        );

        // The reason it could move a mean at all: it is not a small district, it is a different
        // order of magnitude. Asserted so that a fixture revision which fixes the underlying
        // number turns this test red rather than leaving a floor in place for nothing.
        let worst = island
            .iter()
            .map(|r| r.local_revenue / r.enrollment)
            .fold(f64::MIN, f64::max);
        assert!(
            worst > 100_000.0,
            "the island's worst local revenue per pupil is {worst:.0}, not the outlier this excludes"
        );
    }

    /// The floor is read off the data rather than chosen to fit it.
    ///
    /// Across all fifteen years no comparable Ohio district has an enrolment between the largest
    /// Kelleys Island year and the smallest year any other district reaches. [`MIN_ENROLMENT`]
    /// sits inside that empty band, so it excludes one district in every year and nothing else in
    /// any year — which is what makes the corrected series a reading of the data and not of the
    /// threshold.
    #[test]
    fn the_enrolment_floor_sits_in_an_empty_band() {
        let (mut below, mut above) = (f64::MIN, f64::MAX);
        for row in panel()
            .iter()
            .filter(|r| r.comparable && r.enrollment > 0.0)
        {
            if row.enrollment < MIN_ENROLMENT {
                below = below.max(row.enrollment);
            } else {
                above = above.min(row.enrollment);
            }
        }
        assert!(
            below <= 18.0 && above >= 63.0,
            "the band around the floor runs {below} to {above}, and it was 18 to 63"
        );
        assert!(
            MIN_ENROLMENT - below >= 20.0 && above - MIN_ENROLMENT >= 20.0,
            "the floor of {MIN_ENROLMENT} is no longer clear of both sides: {below} and {above}"
        );
    }

    /// **What the three added archives found: during ARRA the state closed less of the local gap
    /// than in any year of the FY2012-FY2024 band, and the federal government closed more.**
    ///
    /// FY2010 and FY2011 sit at 0.386 and 0.375, under the band's 0.400 floor, while federal
    /// gap-closing roughly doubles to 0.133 and 0.142. FY2009, before the money arrived, is
    /// 0.504 — the highest state share in the panel. The panel used to open at FY2012 and could
    /// not see this at all.
    ///
    /// The levels here moved when [`MIN_ENROLMENT`] was applied: these three are years Kelleys
    /// Island was comparable, and they read 0.472, 0.349 and 0.330 before it. The finding is what
    /// survived — the ARRA years are still the low ones and FY2009 is still the high one — and
    /// the margin got thinner, which is why the assertions below are stated against the band's
    /// own floor rather than against a round number.
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
        let band_floor = by_year
            .iter()
            .filter(|(y, _)| (2012..=2024).contains(*y))
            .map(|(_, e)| e.state_share())
            .fold(f64::MAX, f64::min);
        for year in [2010u16, 2011] {
            let e = by_year[&year];
            assert!(
                e.state_share() < band_floor,
                "FY{year} closes {:.4}, which is not below the band's floor of {band_floor:.4}",
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

/// The FY2010-FY2013 contraction, district by district.
///
/// # Why this is here rather than in the test that found it
///
/// `tests/f33_ohio_panel_trough.rs` located the trough and then asserted it in **bands** —
/// `close(share, 0.807, 0.02)`, `close(median, -0.056, 0.006)`. The corpus publishes the same
/// quantities as *values*: `490 — 81%`, `median change -5.6%`, `first quartile -9.2%`. A band and
/// a value can disagree indefinitely and both stay green, which is the failure convention four
/// was written for after it had already happened four times.
///
/// So the analysis is public and pinned in `crates/figures`, and the test keeps its bands — they
/// are the right shape for "this result is robust", which is a different claim from "this is the
/// number the corpus prints".
///
/// # The years, and the hole in them
///
/// [`trough::PEAK_YEAR`] to [`trough::BOTTOM_YEAR`], because **FY2014 is absent from the NCES archive** under
/// every naming the surrounding years use. The Auditor's statewide series is still falling
/// between FY2012 and FY2014, so this measurement may stop short of the bottom and *understate*
/// the contraction. Stated rather than interpolated.
///
/// The gap is no longer at the end of the record. [`spending_by_year`] runs to FY2024, FY2013 is
/// the minimum of all fifteen years, and FY2015 is 1.7% above it — so the unobserved year sits
/// between two observed ones.
pub mod trough {
    use super::{panel, PanelRow};
    use crate::percentile_sorted;
    use deflator::CpiSeries;
    use edfund_core::FiscalYear;

    /// The year the real series peaks *before the trough*, on both entity sets.
    ///
    /// Not the peak of the panel: [`super::spending_by_year`] runs to FY2024 and FY2024 is
    /// higher. This names the top of the decade the contraction sits in, which is what the
    /// contraction is measured from.
    pub const PEAK_YEAR: u16 = 2010;
    /// The bottom of the contraction, and the minimum of every year the panel holds.
    pub const BOTTOM_YEAR: u16 = 2013;
    /// The dollars every figure here is stated in.
    pub const BASE_YEAR: u16 = 2022;

    /// The comparable districts, in the years they report spending.
    ///
    /// The filter is the analysis's rather than the reader's: [`panel`] returns every row the
    /// survey holds, including the community schools and service centres the comparability flag
    /// excludes, and the years an agency reported revenue without spending.
    #[must_use]
    pub fn comparable() -> Vec<PanelRow> {
        panel()
            .into_iter()
            .filter(|r| {
                r.comparable
                    && !r.irn.is_empty()
                    && r.enrollment > 0.0
                    && r.total_revenue > 0.0
                    && r.current_spending.is_some()
            })
            .collect()
    }

    /// A figure restated in [`BASE_YEAR`] dollars.
    ///
    /// # Panics
    ///
    /// If the CPI series has no observation for the year, which the committed series rules out
    /// across this window.
    #[must_use]
    pub fn real(value: f64, year: u16) -> f64 {
        CpiSeries::cpi_u_june()
            .convert(value, FiscalYear(year), FiscalYear(BASE_YEAR))
            .unwrap_or_else(|e| panic!("no CPI observation for FY{year}: {e}"))
            .value
    }

    /// The rows of one year.
    #[must_use]
    pub fn year_of(rows: &[PanelRow], year: u16) -> Vec<PanelRow> {
        rows.iter()
            .filter(|r| r.fiscal_year == year)
            .cloned()
            .collect()
    }

    /// Districts present in **both** endpoint years, with the real change between them.
    ///
    /// The inner join is the population every figure below is computed over, and it is smaller
    /// than either year alone: a district the survey lost or gained between them cannot carry a
    /// change. That is why the count here is not the panel's district count.
    #[must_use]
    pub fn changes(rows: &[PanelRow]) -> Vec<(PanelRow, f64)> {
        let bottom = year_of(rows, BOTTOM_YEAR);
        year_of(rows, PEAK_YEAR)
            .into_iter()
            .filter_map(|start| {
                let end = bottom.iter().find(|r| r.irn == start.irn)?;
                let spend = |row: &PanelRow, year: u16| {
                    real(
                        row.spending_per_pupil()
                            .expect("filtered to rows that report spending"),
                        year,
                    )
                };
                Some((
                    start.clone(),
                    spend(end, BOTTOM_YEAR) / spend(&start, PEAK_YEAR) - 1.0,
                ))
            })
            .collect()
    }

    /// What the contraction looked like across the districts that lived through it.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Contraction {
        /// Districts present in both endpoint years.
        pub districts: usize,
        /// How many of them spent less per pupil in real terms.
        pub falling: usize,
        /// [`Self::falling`] as a fraction of [`Self::districts`].
        pub falling_share: f64,
        /// The median real change, as a **magnitude of decline** — positive, because the key
        /// carries the direction and a numeral reader cannot see a minus sign.
        pub median_decline: f64,
        /// The first-quartile real change, on the same convention.
        pub first_quartile_decline: f64,
        /// The enrollment-weighted decline, which is the figure comparable to the Auditor's
        /// statewide series — a different survey over a different entity set.
        pub weighted_decline: f64,
    }

    /// Measure it.
    ///
    /// # Panics
    ///
    /// If the panel carries no district in both endpoint years, which the committed fixture
    /// rules out.
    #[must_use]
    pub fn contraction() -> Contraction {
        let rows = comparable();
        let changes = changes(&rows);
        assert!(!changes.is_empty(), "no district spans both endpoint years");

        let falling = changes.iter().filter(|(_, c)| *c < 0.0).count();
        let mut values: Vec<f64> = changes.iter().map(|(_, c)| *c).collect();
        values.sort_by(f64::total_cmp);

        let weighted = |year: u16| {
            let rows = year_of(&rows, year);
            let pupils: f64 = rows.iter().map(|r| r.enrollment).sum();
            rows.iter()
                .map(|r| {
                    real(
                        r.spending_per_pupil()
                            .expect("filtered to rows that report spending"),
                        year,
                    ) * r.enrollment
                })
                .sum::<f64>()
                / pupils
        };

        /*
         * `percentile_sorted` and not `values[len / 2]`, which is what the test did and what this
         * function did when it was lifted out of it. On an even-length series the ad-hoc form
         * takes the upper of the two middle observations; this interpolates them. The difference
         * is small and it is not nothing — and `percentile_sorted`'s own docstring already names
         * three places in this workspace that had the ad-hoc form and were corrected. This was
         * the fourth, and it reached the corpus.
         */
        Contraction {
            districts: changes.len(),
            falling,
            falling_share: falling as f64 / changes.len() as f64,
            median_decline: -percentile_sorted(&values, 0.5),
            first_quartile_decline: -percentile_sorted(&values, 0.25),
            weighted_decline: 1.0 - weighted(BOTTOM_YEAR) / weighted(PEAK_YEAR),
        }
    }
    /// What tracks the depth of the fall, and what does not.
    ///
    /// The corpus publishes this as a seven-row table of correlations under `[verified]`, and it
    /// is a **negative** result: the hypothesis it refutes is that a contraction coinciding with
    /// the withdrawal of federal stimulus should land hardest on the districts most dependent on
    /// state aid. It did not. Only size and prior level register at all.
    ///
    /// Each figure is a magnitude with its sign carried in the field name, per the convention a
    /// numeral reader forces: prose writes `+0.029` and `-0.186` and the reader sees neither sign.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Predictors {
        /// FY2010 state revenue share against the depth of the fall.
        pub state_share: f64,
        /// FY2010 federal revenue share.
        pub federal_share: f64,
        /// FY2010 local revenue share.
        pub local_share: f64,
        /// Natural log of enrollment — the strongest of the seven, and still weak.
        pub log_enrollment: f64,
        /// FY2010 real spending per pupil, which is partly mechanical: a district starting higher
        /// has more room to fall.
        pub prior_spending: f64,
    }

    /// Compute them.
    ///
    /// # Panics
    ///
    /// If the paired series cannot be correlated, which a non-empty panel rules out.
    #[must_use]
    pub fn predictors() -> Predictors {
        let changes = changes(&comparable());
        let correlate = |pairs: &[(f64, f64)]| {
            let (x, y): (Vec<f64>, Vec<f64>) = pairs.iter().copied().unzip();
            crate::wealth_neutrality(&x, &y)
                .expect("paired series")
                .correlation
        };
        let against = |pick: &dyn Fn(&PanelRow) -> f64| {
            correlate(
                &changes
                    .iter()
                    .map(|(r, c)| (pick(r), *c))
                    .collect::<Vec<_>>(),
            )
        };
        let share =
            |row: &PanelRow, part: f64| row.share(part).expect("filtered to rows with revenue");
        Predictors {
            state_share: against(&|r| share(r, r.state_revenue)),
            federal_share: against(&|r| share(r, r.federal_revenue)),
            local_share: against(&|r| share(r, r.local_revenue)),
            log_enrollment: against(&|r| r.enrollment.ln()),
            prior_spending: against(&|r| {
                real(
                    r.spending_per_pupil()
                        .expect("filtered to rows that report spending"),
                    PEAK_YEAR,
                )
            }),
        }
    }

    /// The worst tenth of districts — real, and not what moved the state.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct DeepestDecile {
        /// How many districts are in it.
        pub districts: usize,
        /// Their mean decline, as a magnitude.
        pub mean_decline: f64,
        /// The share of the panel's pupils they hold, which is the reason this is a tail rather
        /// than the story.
        pub pupil_share: f64,
    }

    /// Measure it.
    ///
    /// # Panics
    ///
    /// If the panel is empty.
    #[must_use]
    pub fn deepest_decile() -> DeepestDecile {
        let mut changes = changes(&comparable());
        changes.sort_by(|a, b| a.1.total_cmp(&b.1));
        let tenth = changes.len() / 10;
        assert!(tenth > 0, "the panel is too small to have a decile");
        let worst = &changes[..tenth];
        let pupils: f64 = changes.iter().map(|(r, _)| r.enrollment).sum();
        DeepestDecile {
            districts: tenth,
            mean_decline: -worst.iter().map(|(_, c)| *c).sum::<f64>() / tenth as f64,
            pupil_share: worst.iter().map(|(r, _)| r.enrollment).sum::<f64>() / pupils,
        }
    }
}
