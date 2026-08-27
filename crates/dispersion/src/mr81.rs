//! Seventeen Octobers of the free and reduced-price lunch report.
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
//! # Four things that will produce a wrong reading
//!
//! **The denominator changes definition in FY2010.** `AdmCount` becomes `CECount` — "the highest
//! daily number of students with access to the program" — which is neither ADM nor the count that
//! preceded it. [`Sponsor::basis`] carries which, and [`poverty_share_by_year`] reports it, so a
//! series crossing FY2009 is spliced knowingly or not at all.
//!
//! **The report becomes three reports in FY2012, and they do not count the same thing.** The
//! Traditional file collects applications and says in its own header that it excludes the other
//! two. Provision 2 reports applications frozen at a base year. Community eligibility collects no
//! applications at all: its approval columns are zero by construction. Adding the three gives a
//! poverty rate that falls thirteen points in three years because the poorest sponsors stopped
//! filling in forms — see [`PovertyYear::applications_share`], which is that number, kept so a
//! test can hold how wrong it is. [`PovertyYear::floor`] and [`PovertyYear::ceiling`] are what
//! the source can actually support.
//!
//! **Sponsors are not districts.** "Public" covers county boards of developmental disabilities
//! and community schools as well as traditional districts, and the report also carries non-public
//! schools, residential child care institutions and camps. The sponsor count rising from 718 to
//! 973 across the window is mostly community schools opening, not districts appearing. It is 973
//! and not the 1,001 rows FY2014 carries, because from FY2012 a sponsor may file in more than one
//! stream — see [`PovertyYear::sponsors`], which counts bodies, and [`PovertyYear::filings`],
//! which counts returns.
//!
//! **The first three Octobers do not state a type at all.** FY1998-FY2000 are one row per school
//! with the district on it and no sponsor-type column anywhere, so the extract writes `Unknown`
//! and [`resolved_kind`] supplies the type from FY2001. Some thirty-five sponsors a year are not
//! in FY2001 and stay `Unknown` — children's services boards, camps, parish schools and one
//! renamed career centre, carrying eight thousand pupils between them against 1.8 million.
//!
//! # One published cell is wrong by two orders of magnitude, and three others were ours
//!
//! See [`implausible_sponsors`]: the FY2005 file gives Wilson Elementary School an `AdmCount` of
//! 342,332, which puts Portsmouth City at 344,048 students across nine sites. The extractor
//! transcribes it, because an extractor that quietly repairs its source is worse than one that
//! ships a visible error — but nothing should aggregate FY2005 without excluding it.
//!
//! That list used to hold four sponsor-years, and the other three were this repository's doing.
//! FY2001 is the only comma-delimited file; nine of its rows carry a comma inside a school name;
//! and split positionally those rows put a site IRN into the enrollment column. Two were Cleveland
//! City SD, which reached the aggregate as 192,147 students against its real 73,562 and held the
//! published FY2001 poverty share 1.8 points below what the file says.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/mr81-sponsor-panel.csv");

const EXPECTED_HEADER: &str = "fiscal_year,sponsor_irn,sponsor_name,county,sponsor_type,stream,\
sites,enrollment,enrollment_basis,free_lunch,reduced_lunch,identified,claimable,censored";

/// The October the report was last one file.
pub const LAST_SINGLE_STREAM: u16 = 2011;

/// The Octobers whose population is not the state, because most sponsors did not file.
///
/// Under USDA's nationwide free-meal waivers a sponsor could serve every student free without
/// collecting one application, and almost every one of them did. These two Octobers carry
/// **296 and 261 sponsors against about 850**, and **425,154 and 398,209 pupils against 1.7
/// million** — so what they report is not Ohio, it is the sponsors who still had a reason to
/// file. Their floor and ceiling read twenty points above the years on either side, and every
/// point of that is about the waiver.
///
/// Carried rather than dropped: the rows are true about the sponsors in them, and a gap in the
/// panel would be indistinguishable from a year nobody retrieved. [`PovertyYear::comparable`] is
/// how a consumer tells them apart, and `web/src/lib/mealProgram.ts` breaks its line on it.
pub const WAIVER_OCTOBERS: [u16; 2] = [2020, 2021];

/// The October whose sponsor types are borrowed by the three school-centric years before it.
const TYPE_SOURCE: u16 = 2001;

/// One sponsor in one October, in one of the report's streams.
#[derive(Debug, Clone, PartialEq)]
pub struct Sponsor {
    /// The October counted.
    pub year: u16,
    /// Six-digit IRN, zero-padded — the published files disagree about padding.
    pub irn: String,
    /// The sponsor's name as published.
    pub name: String,
    /// What kind of sponsor: `Public`, `Non-Public`, two smaller kinds, and `Unknown` for the
    /// three Octobers whose files carry no type column.
    pub kind: String,
    /// Which publication this row came from: `single` through FY2011, then `traditional`,
    /// `provision2` or `community`.
    pub stream: String,
    /// How many school sites the sponsor reported.
    pub sites: usize,
    /// The meal-program denominator, whose definition changes in FY2010.
    pub enrollment: f64,
    /// `adm` through FY2009, `ce` from FY2010.
    pub basis: String,
    /// Free lunch applications. Zero for every community-eligibility row, because those sponsors
    /// collect none.
    pub free: f64,
    /// Reduced-price lunch applications.
    pub reduced: f64,
    /// Directly certified children, published only by the community-eligibility stream.
    pub identified: f64,
    /// What the sponsor may claim for: its approvals, or for community eligibility the
    /// directly-certified count multiplied by USDA's 1.6 and capped at enrollment site by site.
    pub claimable: f64,
    /// How many of this sponsor-year's application cells the publisher printed as `<10`.
    ///
    /// Zero for every October through FY2014, which masks nothing. From FY2015 a count is masked
    /// when either half of the free/reduced pair falls under ten — **not** when the cell itself
    /// does, which is why it is not a bound. The extractor recovers the number from the
    /// percentage the workbook prints beside it, on an identity that reproduces all 42,983
    /// unmasked cells exactly, so the counts here are the publisher's own. This column says how
    /// many of them arrived by the second route.
    pub censored: f64,
}

/// Every row of the panel.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a row's width differs
/// from the header's — both by way of [`edfund_core::csv::rows`], which is what holds the
/// uniform-width invariant these fixtures are written under. A hand-rolled `split(',')` here
/// checked the header and then indexed by position, so a cell that grew a comma shifted every
/// field after it and the row still parsed.
#[must_use]
pub fn panel() -> Vec<Sponsor> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            Some(Sponsor {
                year: row.str(0).parse().ok()?,
                irn: row.str(1).to_string(),
                name: row.str(2).to_string(),
                kind: row.str(4).to_string(),
                stream: row.str(5).to_string(),
                sites: row.str(6).parse().ok()?,
                enrollment: row.num(7)?,
                basis: row.str(8).to_string(),
                free: row.num(9)?,
                reduced: row.num(10)?,
                identified: row.num(11)?,
                claimable: row.num(12)?,
                censored: row.num(13)?,
            })
        })
        .collect()
}

/// Sponsor type by IRN, as FY2001 states it.
///
/// The three school-centric Octobers before it are one row per school with the district named and
/// numbered on the row and **no sponsor-type column at all**. Ohio's meal-program sponsors are not
/// all districts — county boards of developmental disabilities, residential institutions, camps
/// and every non-public school in the state are in the same file — so a poverty series that does
/// not distinguish them is not a series of anything.
///
/// FY2001 rather than the whole panel, and rather than a guess from the IRN's range. It is the
/// adjacent October, it numbers districts the same way, and asking one year avoids inheriting an
/// answer from a decade later when an IRN has been reused. What it costs is about twenty sponsors
/// a year that had gone by 2001 — three thousand pupils between them, against 1.8 million — and
/// those stay `Unknown` rather than being assigned a type nothing states.
#[must_use]
pub fn types_in_2001() -> BTreeMap<String, String> {
    panel()
        .into_iter()
        .filter(|s| s.year == TYPE_SOURCE && s.kind != "Unknown")
        .map(|s| (s.irn, s.kind))
        .collect()
}

/// The sponsor's type, from its own row where the file states one and from FY2001 where it does
/// not.
#[must_use]
pub fn resolved_kind<'a>(sponsor: &'a Sponsor, types: &'a BTreeMap<String, String>) -> &'a str {
    if sponsor.kind == "Unknown" {
        types.get(&sponsor.irn).map_or("Unknown", String::as_str)
    } else {
        &sponsor.kind
    }
}

/// Sponsor-years whose enrollment cannot be right, so an aggregate can exclude them by name.
///
/// A site averaging over twenty thousand students is not a school. The threshold is deliberately
/// far above anything real — Ohio's largest district averages a few hundred per site — so this
/// catches published corruption rather than unusual districts.
///
/// It catches one sponsor-year and used to catch four. The other three were a parse defect and
/// not a published one, which is worth remembering when reading a filter like this: it caught the
/// three whose spurious value happened to be enormous next to a small site count, and missed the
/// two Cleveland rows whose spurious value was merely large next to a hundred and eighteen.
#[must_use]
pub fn implausible_sponsors() -> Vec<Sponsor> {
    panel()
        .into_iter()
        .filter(|s| s.sites > 0 && s.enrollment / s.sites as f64 > 20_000.0)
        .collect()
}

/// The share of the meal-program denominator eligible for free or reduced-price lunch, by year.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PovertyYear {
    /// Public sponsors counted — **distinct entities**, not rows.
    ///
    /// From FY2012 the report is three files and a sponsor may file in more than one of them, so
    /// the rows are filings and counting them counts the same body twice. This was that count
    /// until it was corrected: it published 1,001 public sponsors for FY2014 where 973 exist, and
    /// turned the FY2011-FY2014 rise of 2.5% into one of 5.5%, which is the growth of *filings*
    /// wearing the label of the growth of *sponsors*. See [`Self::filings`].
    pub sponsors: usize,
    /// Rows the year is summed over: one per sponsor per stream.
    ///
    /// Equal to [`Self::sponsors`] through FY2011 and above it after. Carried rather than
    /// dropped, because it is the right denominator for a question about the report — how many
    /// returns were filed — and the wrong one for every question about Ohio, and the two were
    /// indistinguishable while only one number was published.
    pub filings: usize,
    /// The denominator, summed over those sponsors.
    pub enrollment: f64,
    /// Free and reduced approvals, summed over those sponsors.
    pub approved: f64,
    /// Directly certified children under community eligibility, where the sponsor collects no
    /// applications. Zero before FY2012.
    pub identified: f64,
    /// Whether this October's filings are a reading of the state at all.
    ///
    /// False for the two [`WAIVER_OCTOBERS`], and true everywhere else. A consumer drawing a
    /// series has to break on it: those two years are twenty points above their neighbours
    /// because two thirds of the sponsors stopped filing, not because Ohio got poorer and then
    /// better again.
    pub comparable: bool,
    /// Approvals, or for community eligibility what the sponsor may claim for.
    pub claimable: f64,
    /// Approvals as a share of enrollment.
    ///
    /// **This is the poverty share only while the report is one file.** From FY2012 it divides
    /// applications by an enrollment that includes sponsors who collect none, so it falls as
    /// community eligibility spreads and would read as poverty falling. Kept because it is what
    /// a naive extension of the series produces, and a test holds how far wrong it goes.
    pub applications_share: f64,
    /// The lowest share the source supports: approvals plus directly certified children.
    ///
    /// A floor because direct certification reaches children whose families are already on SNAP,
    /// TANF, foster care or a homeless roll and nobody else — every child who would have
    /// qualified by application and did not appear on one of those lists is missing from it.
    pub floor: f64,
    /// The highest: what every sponsor may claim for.
    ///
    /// A ceiling because the 1.6 multiplier behind it is a programme-wide reimbursement rule
    /// rather than a measurement of any particular school, and because it is capped at enrollment,
    /// so a school where four in five children are directly certified counts as though all of
    /// them were.
    pub ceiling: f64,
    /// The poverty share, where the source supports one number.
    ///
    /// `None` from FY2012, and that is the finding rather than a gap. The three publications count
    /// three different things, so an October published as three files has a band and does not have
    /// a share. Every consumer that wants one line has to decide what to do about that, which is
    /// the decision this field exists to force rather than to make.
    pub share: Option<f64>,
    /// The share of the October's enrollment under sponsors that collect no applications.
    ///
    /// Zero before FY2012 and a sixth by FY2014. This is the size of the hole in
    /// [`Self::applications_share`], and the reason the hole grows is that community eligibility
    /// is open to schools whose poverty is already high — so the population leaving the
    /// applications-based measure is not a random sample of it.
    pub without_applications: f64,
    /// Whether the denominator that year is `adm` or `ce`.
    pub basis_is_ce: bool,
    /// How many publications the October was split across: one through FY2011, three after.
    pub streams: usize,
}

/// Public sponsors only, excluding the sponsor-years [`implausible_sponsors`] names.
///
/// Types for the three Octobers that state none come from [`types_in_2001`]; a sponsor it cannot
/// name is left out, on the same footing as one that is not a public body.
#[must_use]
pub fn poverty_share_by_year() -> BTreeMap<u16, PovertyYear> {
    let bad: BTreeSet<(u16, String)> = implausible_sponsors()
        .into_iter()
        .map(|s| (s.year, s.irn))
        .collect();
    let types = types_in_2001();

    // The fifth slot is the set of sponsor IRNs. A sponsor that files in two of the three
    // post-FY2011 streams is two rows and one body: the sums want both rows, and the count wants
    // one entry, and keeping the set is the only way to have both.
    type Totals = (PovertyYear, BTreeSet<String>, bool, f64, BTreeSet<String>);
    let mut totals: BTreeMap<u16, Totals> = BTreeMap::new();
    for s in panel() {
        if resolved_kind(&s, &types) != "Public" || bad.contains(&(s.year, s.irn.clone())) {
            continue;
        }
        let entry = totals.entry(s.year).or_default();
        entry.4.insert(s.irn.clone());
        entry.0.filings += 1;
        entry.0.enrollment += s.enrollment;
        entry.0.approved += s.free + s.reduced;
        entry.0.identified += s.identified;
        entry.0.claimable += s.claimable;
        if s.stream == "community" {
            entry.3 += s.enrollment;
        }
        entry.1.insert(s.stream);
        entry.2 = s.basis == "ce";
    }

    totals
        .into_iter()
        .map(|(year, (mut totals, streams, ce, community, irns))| {
            totals.sponsors = irns.len();
            if totals.enrollment > 0.0 {
                totals.applications_share = totals.approved / totals.enrollment;
                totals.floor = (totals.approved + totals.identified) / totals.enrollment;
                totals.ceiling = totals.claimable / totals.enrollment;
                totals.without_applications = community / totals.enrollment;
            }
            totals.basis_is_ce = ce;
            totals.streams = streams.len();
            totals.comparable = !WAIVER_OCTOBERS.contains(&year);
            // One file, one share. The three readings coincide there because `identified` is
            // zero and `claimable` is the approvals, so which of them is written is arbitrary —
            // what is not arbitrary is that from FY2012 there is nothing to write.
            totals.share = (totals.streams == 1).then_some(totals.ceiling);
            (year, totals)
        })
        .collect()
}

/// One stream of one October, for the years the report is split.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StreamYear {
    /// Public sponsors filing in this stream.
    pub sponsors: usize,
    /// Their enrollment.
    pub enrollment: f64,
    /// Their approvals, which are zero by construction under community eligibility.
    pub approved: f64,
    /// Their directly certified children, which are published only under community eligibility.
    pub identified: f64,
    /// Their enrollment as a share of the October's.
    pub of_enrollment: f64,
}

/// Public sponsors by stream, for the three Octobers the report is published as three files.
///
/// The number this exists for is `community.of_enrollment`: a fourteenth of Ohio's public
/// meal-program enrollment in FY2012 and a sixth by FY2014. That is the population that left the
/// applications-based measure, and it left because its poverty was high enough to qualify for
/// feeding every child without asking.
#[must_use]
pub fn streams_by_year() -> BTreeMap<u16, BTreeMap<String, StreamYear>> {
    let bad: BTreeSet<(u16, String)> = implausible_sponsors()
        .into_iter()
        .map(|s| (s.year, s.irn))
        .collect();
    let types = types_in_2001();

    let mut totals: BTreeMap<u16, BTreeMap<String, StreamYear>> = BTreeMap::new();
    for s in panel() {
        if s.stream == "single"
            || resolved_kind(&s, &types) != "Public"
            || bad.contains(&(s.year, s.irn.clone()))
        {
            continue;
        }
        let entry = totals
            .entry(s.year)
            .or_default()
            .entry(s.stream.clone())
            .or_default();
        entry.sponsors += 1;
        entry.enrollment += s.enrollment;
        entry.approved += s.free + s.reduced;
        entry.identified += s.identified;
    }

    for streams in totals.values_mut() {
        let year: f64 = streams.values().map(|s| s.enrollment).sum();
        if year <= 0.0 {
            continue;
        }
        for stream in streams.values_mut() {
            stream.of_enrollment = stream.enrollment / year;
        }
    }
    totals
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Twenty-eight Octobers without a gap, which used to be seventeen.
    ///
    /// The open directory at `public.education.ohio.gov/MR81/` really does end at October 2014,
    /// and the Internet Archive holds nothing later either — 101 URLs captured and the newest
    /// directory is `MR81_October_2014`. What ended was the directory. The report moved to the
    /// department's own page and has been published every year since, which is #16.
    #[test]
    fn the_panel_holds_every_october_the_archive_publishes() {
        let years: Vec<u16> = poverty_share_by_year().keys().copied().collect();
        assert_eq!(years, (1998..=2025).collect::<Vec<u16>>());
    }

    /// The one published defect, pinned by name so a new one cannot hide behind it.
    ///
    /// This test used to assert four, and the other three were a parse defect of this
    /// repository's own — FY2001 rows whose columns had shifted across an unquoted comma, putting
    /// site IRNs in the enrollment column. They are gone because the file is now read correctly,
    /// not because the filter changed.
    #[test]
    fn the_published_defect_is_the_one_known_one() {
        let bad = implausible_sponsors();
        assert_eq!(
            bad.len(),
            1,
            "found {} implausible sponsor-years",
            bad.len()
        );
        assert_eq!(bad[0].year, 2005);
        assert_eq!(bad[0].name, "Portsmouth City SD");
        assert_eq!(bad[0].kind, "Public");
        assert!(
            bad[0].enrollment > 300_000.0,
            "the defect is a 342,332 in one site's AdmCount; it now reads {}",
            bad[0].enrollment
        );
    }

    /// The repair, held from the outside: no row's enrollment may be one of the file's own IRNs.
    ///
    /// The nine shifted FY2001 rows all failed the same way, by reading an eight-digit identifier
    /// as a count. A site IRN is six or eight digits and a sponsor of that many pupils does not
    /// exist, so an enrollment that matches a live IRN is the signature rather than the size.
    #[test]
    fn no_sponsor_reports_an_identifier_as_its_enrollment() {
        let irns: BTreeSet<String> = panel().into_iter().map(|s| s.irn).collect();
        for s in panel() {
            let as_irn = format!("{:0>6}", s.enrollment as i64);
            assert!(
                !irns.contains(&as_irn) || s.enrollment < 100_000.0,
                "FY{} {} reports {} enrolled, which is the IRN {as_irn}",
                s.year,
                s.name,
                s.enrollment
            );
        }
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

    /// The finding: measured poverty rises by seventeen points in fourteen years.
    #[test]
    fn the_poverty_share_rises_across_the_single_stream_window() {
        let by_year = poverty_share_by_year();
        let first = by_year[&1998].share.expect("FY1998 is one file");
        let last = by_year[&LAST_SINGLE_STREAM]
            .share
            .expect("FY2011 is one file");
        assert!(
            (first - 0.289).abs() < 0.005,
            "FY1998 share is {first:.4}, not 0.289"
        );
        assert!(
            (last - 0.463).abs() < 0.005,
            "FY2011 share is {last:.4}, not 0.463"
        );
        assert!(
            last - first > 0.15,
            "the rise is {:.1} points, not the seventeen-plus recorded",
            100.0 * (last - first)
        );
    }

    /// FY2001 is the year the comma repair moves, and it moves it up.
    ///
    /// The published series had FY2001 at 27.7%, below FY2000 and below FY1998, which read as
    /// poverty falling for three years and then turning. It was two Cleveland rows.
    #[test]
    fn fy2001_is_no_longer_a_trough() {
        let by_year = poverty_share_by_year();
        let share = |year: u16| by_year[&year].share.expect("one file");
        assert!(
            (share(2001) - 0.2946).abs() < 0.002,
            "FY2001 share is {:.4}, not the 0.2946 the file supports",
            share(2001)
        );
        assert!(share(2001) > share(2000), "FY2001 is below FY2000 again");
        assert!(
            by_year[&2001].enrollment < 1_850_000.0,
            "FY2001 enrollment is {:.0}, which is the inflated figure back",
            by_year[&2001].enrollment
        );
    }

    /// The denominator changes in FY2010 and the panel says so rather than smoothing it.
    #[test]
    fn the_denominator_changes_definition_in_2010_and_is_labelled() {
        let by_year = poverty_share_by_year();
        assert!(
            (1998..=2009).all(|y| !by_year[&y].basis_is_ce),
            "a pre-2010 year is labelled CE"
        );
        assert!(
            (2010..=2014).all(|y| by_year[&y].basis_is_ce),
            "a year from FY2010 is not labelled CE"
        );
    }

    /// The trap this extension exists not to fall into.
    ///
    /// Summing the three streams' applications gives a share that falls thirteen points in three
    /// years, at the end of a decade in which it rose eighteen. Nothing about poverty happened;
    /// the sponsors most likely to raise the share stopped collecting the forms it is counted
    /// from. The honest reading is a band, and the band does not fall through FY2011's figure.
    #[test]
    fn adding_the_three_streams_would_read_as_poverty_collapsing() {
        let by_year = poverty_share_by_year();
        let fy2011 = by_year[&LAST_SINGLE_STREAM]
            .share
            .expect("FY2011 is one file");
        let naive = by_year[&2014].applications_share;
        assert!(
            fy2011 - naive > 0.10,
            "the naive FY2014 share is {naive:.4} against FY2011's {fy2011:.4}, and the collapse \
             this test is about has stopped happening"
        );
        for year in 2012..=2014 {
            let y = &by_year[&year];
            assert_eq!(
                y.share, None,
                "FY{year} is three files and claims one share"
            );
            assert!(
                y.floor <= y.ceiling,
                "FY{year}'s floor {:.4} is above its ceiling {:.4}",
                y.floor,
                y.ceiling
            );
            assert!(
                y.applications_share < y.floor,
                "FY{year}'s applications share is no longer the lowest of the three"
            );
            assert!(
                y.floor < fy2011 && fy2011 < y.ceiling,
                "FY{year}'s band [{:.4}, {:.4}] no longer brackets FY2011's {fy2011:.4}, which is \
                 the whole reason the source cannot settle the direction",
                y.floor,
                y.ceiling
            );
        }
    }

    /// Which years may be drawn as one line, and which may not.
    #[test]
    fn only_the_years_published_as_one_file_carry_one_share() {
        let by_year = poverty_share_by_year();
        for (year, y) in &by_year {
            let single = *year <= LAST_SINGLE_STREAM;
            assert_eq!(y.share.is_some(), single, "FY{year}");
            /*
             * Three streams from FY2012 — except October 2021, which has **two**.
             *
             * Under USDA's nationwide free-meal waivers a sponsor could serve every student free
             * without collecting an application, and that October **not one Provision 2 sponsor
             * filed at all**. An absent filing is not an empty one: it is not written, so the
             * year carries two streams rather than three with a zero in it.
             */
            let expected = if single {
                1
            } else if *year == 2021 {
                2
            } else {
                3
            };
            assert_eq!(y.streams, expected, "FY{year}");
            if single {
                assert_eq!(
                    (y.applications_share, y.floor),
                    (y.ceiling, y.ceiling),
                    "FY{year} is one file and its three readings differ"
                );
            }
        }
    }

    /// A sponsor filing in two streams is one sponsor, and the count says so.
    #[test]
    fn the_sponsor_count_counts_bodies_and_the_filing_count_counts_returns() {
        let years = poverty_share_by_year();

        // Through FY2011 the report is one file, so every sponsor files once and the two counts
        // are the same number. That is what made the error invisible for the fourteen years
        // before it started being wrong.
        for year in 1998..=LAST_SINGLE_STREAM {
            assert_eq!(
                years[&year].sponsors, years[&year].filings,
                "FY{year} is one file and cannot have a sponsor filing twice"
            );
        }

        // And from FY2012 they separate. 1,001 was published as a sponsor count for FY2014; it is
        // the number of returns filed, and 28 of them are a second return from a body already
        // counted.
        assert_eq!(years[&2014].filings, 1_001);
        assert_eq!(years[&2014].sponsors, 973);
        assert!(
            (2012..=2014).all(|y| years[&y].sponsors < years[&y].filings),
            "the three-file Octobers have no sponsor filing in two streams"
        );

        // What the mistake was worth. Growth stated on filings is more than twice growth stated
        // on sponsors, over exactly the window the card draws.
        let growth = |pick: fn(&PovertyYear) -> usize| {
            pick(&years[&2014]) as f64 / pick(&years[&LAST_SINGLE_STREAM]) as f64 - 1.0
        };
        let (bodies, returns) = (growth(|y| y.sponsors), growth(|y| y.filings));
        assert!(
            (bodies - 0.0253).abs() < 0.0005 && (returns - 0.0548).abs() < 0.0005,
            "FY2011-FY2014: {bodies:.4} of sponsors against {returns:.4} of filings"
        );
        assert!(returns > bodies * 2.0);

        // The endpoints the module note and the card's prose both name.
        assert_eq!((years[&1998].sponsors, years[&2014].sponsors), (718, 973));
    }

    /// The measured size of the population that left the applications-based count.
    #[test]
    fn community_eligibility_takes_a_sixth_of_the_enrollment_in_three_years() {
        let streams = streams_by_year();
        assert_eq!(
            streams.keys().copied().collect::<Vec<u16>>(),
            (2012..=2025).collect::<Vec<u16>>()
        );
        let by_year = poverty_share_by_year();
        let share = |year: u16| streams[&year]["community"].of_enrollment;
        assert!(
            (share(2012) - 0.070).abs() < 0.005,
            "FY2012 community share is {:.4}, not 0.070",
            share(2012)
        );
        assert!(
            (share(2014) - 0.166).abs() < 0.005,
            "FY2014 community share is {:.4}, not 0.166",
            share(2014)
        );
        assert!(share(2012) < share(2013) && share(2013) < share(2014));
        // The same figure reaches the feed off `PovertyYear`, and the two are computed from
        // different aggregates — one over the streams and one over the year — so they have to be
        // asserted equal or they will drift.
        for year in 2012..=2014 {
            assert!(
                (by_year[&year].without_applications - share(year)).abs() < 1e-9,
                "FY{year}: the year says {:.6} and the streams say {:.6}",
                by_year[&year].without_applications,
                share(year)
            );
        }
        assert!(
            (1998..=LAST_SINGLE_STREAM).all(|y| by_year[&y].without_applications == 0.0),
            "a single-file October has sponsors collecting no applications"
        );

        for (year, by_stream) in &streams {
            // Three, except the waiver October when no Provision 2 sponsor filed. See
            // `only_the_years_published_as_one_file_carry_one_share`.
            assert_eq!(
                by_stream.len(),
                if *year == 2021 { 2 } else { 3 },
                "FY{year} does not carry the streams it should"
            );
            assert_eq!(
                by_stream["community"].approved, 0.0,
                "FY{year}'s community sponsors report approvals, which they cannot collect"
            );
            assert_eq!(
                by_stream["traditional"].identified, 0.0,
                "FY{year}'s traditional file reports directly certified children"
            );
        }
    }

    /// The two Octobers that are not a reading of Ohio, and how far out they read.
    ///
    /// The whole reason `comparable` exists. Drawn as part of the series these are a twenty-point
    /// poverty spike and a twenty-point recovery, and both are about who filed.
    #[test]
    fn the_waiver_octobers_are_not_the_state() {
        let by_year = poverty_share_by_year();
        for year in WAIVER_OCTOBERS {
            let y = &by_year[&year];
            assert!(
                !y.comparable,
                "FY{year} should not be drawn with its neighbours"
            );
            // A third of the sponsors and a quarter of the pupils the years either side carry.
            assert!(
                y.sponsors < by_year[&2019].sponsors / 2,
                "FY{year} has {} sponsors against FY2019's {}",
                y.sponsors,
                by_year[&2019].sponsors
            );
            assert!(y.enrollment < 0.3 * by_year[&2019].enrollment);
            // And the floor reads far above the years on either side, which is the defect.
            assert!(
                y.floor > by_year[&2019].floor + 0.15 && y.floor > by_year[&2022].floor + 0.15,
                "FY{year}'s floor of {:.4} is not the outlier this test is about",
                y.floor
            );
        }
        // Everything else is a reading of the state, including the eleven new Octobers.
        assert!(
            (1998..=2025)
                .filter(|y| !WAIVER_OCTOBERS.contains(y))
                .all(|y| by_year[&y].comparable),
            "a year outside the waivers is marked incomparable"
        );
    }

    /// The three Octobers with no type column of their own, and what borrowing FY2001 costs.
    #[test]
    fn the_school_centric_years_borrow_their_sponsor_types() {
        let types = types_in_2001();
        assert!(types.len() > 1_000, "FY2001 names {} sponsors", types.len());
        for year in 1998..=2000 {
            let rows: Vec<Sponsor> = panel().into_iter().filter(|s| s.year == year).collect();
            assert!(
                rows.iter().all(|s| s.kind == "Unknown"),
                "FY{year} states a sponsor type, so it should not be borrowing one"
            );
            let unresolved: Vec<&Sponsor> = rows
                .iter()
                .filter(|s| resolved_kind(s, &types) == "Unknown")
                .collect();
            assert!(
                unresolved.len() < 50,
                "FY{year} leaves {} sponsors untyped",
                unresolved.len()
            );
            // Half a per cent of the October. Bounded rather than pinned, and bounded tightly:
            // the point of the borrow is that what it cannot reach is too small to move a share,
            // and a threshold loose enough to hide a district would not say that.
            let lost: f64 = unresolved.iter().map(|s| s.enrollment).sum();
            assert!(
                lost < 10_000.0,
                "FY{year} leaves {lost:.0} pupils under an untyped sponsor"
            );
        }
    }
}
