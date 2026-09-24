//! The October count R.C. 3317.024(E)(2)(d) divides by, 1977 through 2025.
//!
//! The statutory denominator of [`super::nonpublic_support`], and for a long time the thing this
//! corpus did not hold. R.C. 3317.024(E)(2)(d) divides the appropriation for R.C. 3317.06 and
//! 3317.062 by the average daily membership in grades kindergarten through twelve in chartered
//! nonpublic schools "as determined as of the last day of October of each school year". The
//! department publishes that October, every year since 1977, as attachments on one CMS page; the
//! extraction is `connect::fixtures::nonpublic` and the connector is `dew-nonpublic-enrollment`.
//!
//! Two fixtures, and they are two views of one read. [`buildings`] is one row per school per
//! October — 39,440 rows over forty-nine of them. [`octobers`] is the sector: one row per October
//! per publishing file, which is more than one row for some years because two files describe the
//! same October and do not always agree.
//!
//! # Why a count here is three numbers
//!
//! The department masks a small cell rather than publishing it, so most of these counts are not
//! numbers but intervals. [`Bounded`] carries all three parts: the sum of what survived masking,
//! the largest value consistent with it, and how many cells were masked. A reader that wants a
//! single number has to say which end it is standing on, which is the point.
//!
//! The published totals are the exception and the reason [`membership`] exists. From October 2015
//! the department states its own sector total on a separate sheet, computed before the masking, so
//! those are exact. They are also the only figures here that the statute's quotient can use.
//!
//! # In-state only
//!
//! [`October::published_in_state`] and [`October::published_out_of_state`] are stated separately
//! and the statutory denominator is the first. That is not read off the statute — R.C. 3317.024
//! does not say "resident" — but off the arithmetic: FY2025's appropriation net of the College
//! Credit Plus earmark over October 2024's 179,693 in-state pupils is $913.10, and LSC publishes
//! $913. Over the 181,171 including out-of-state pupils it is $905.65, which LSC does not publish.
//! See `super::nonpublic_support::auxiliary_rate_fy2025`.
//!
//! # What it does not hold
//!
//! **The district each building sits in.** No nonpublic file in any era carries a district column,
//! and R.C. 3317.06 flows through the district a school is located in under the first of the two
//! payment routes. So this module can say what a school's membership was and not whose
//! entitlement it generated. The catalog entry records the search; the decision record
//! `nonpublic-enrollment-connector` records why a name match was rejected as a substitute.
//!
//! [`Building::school`] is there to be read, never to be keyed on: October 2023 holds sixteen
//! schools named `St Mary` across 711 buildings with 601 distinct names. [`Building::irn`] is the
//! key.

use edfund_core::csv;

/// The committed sector extract: one row per October per publishing file.
const SECTOR: &str = include_str!("../../fixtures/nonpublic-sector-panel.csv");

/// The header [`octobers`] indexes against.
const SECTOR_HEADER: &str = "school_year,fiscal_year,basis,source,buildings,k12_floor,k12_ceiling,\
    k12_censored,race_floor,race_ceiling,race_censored,out_of_state_floor,out_of_state_ceiling,\
    out_of_state_censored,published_in_state,published_out_of_state,published_total";

/// The committed building extract: one row per school per October.
const BUILDING: &str = include_str!("../../fixtures/nonpublic-building-panel.csv");

/// The header [`buildings`] indexes against.
const BUILDING_HEADER: &str = "school_year,fiscal_year,irn,school,county,school_type,source,\
    k12_floor,k12_ceiling,k12_censored,race_floor,race_ceiling,race_censored,out_of_state_floor,\
    out_of_state_ceiling,out_of_state_censored,preschool,published_total,published_basis";

/// The `basis` of the row the annual file publishes, in the three shapes the department has used.
///
/// A row carrying one of these is the October's own file. Everything else in the sector panel is
/// a later restatement — see [`COMPILATION`] — and the two are kept apart rather than reconciled,
/// because for four Octobers they disagree. [`October::is_annual`] is the test.
pub const ANNUAL: [&str; 3] = ["wide-grade-and-race", "by-grade-and-sex", "building-adm"];

/// The `basis` of the 2014-2019 compilation's rows: a second statement of six Octobers.
///
/// The compilation counts preschool through twelve where the annual files count kindergarten
/// through twelve, and its school counts are two or three lower. It is carried rather than
/// dropped because a disagreement between two of one department's own files is a finding.
pub const COMPILATION: &str = "ps12-compilation";

/// The columns of [`SECTOR_HEADER`] this module reads.
mod sector_column {
    pub const SCHOOL_YEAR: usize = 0;
    pub const FISCAL_YEAR: usize = 1;
    pub const BASIS: usize = 2;
    pub const SOURCE: usize = 3;
    pub const BUILDINGS: usize = 4;
    pub const K12: usize = 5;
    pub const RACE: usize = 8;
    pub const OUT_OF_STATE: usize = 11;
    pub const PUBLISHED_IN_STATE: usize = 14;
    pub const PUBLISHED_OUT_OF_STATE: usize = 15;
    pub const PUBLISHED_TOTAL: usize = 16;
}

/// The columns of [`BUILDING_HEADER`] this module reads.
mod building_column {
    pub const SCHOOL_YEAR: usize = 0;
    pub const FISCAL_YEAR: usize = 1;
    pub const IRN: usize = 2;
    pub const SCHOOL: usize = 3;
    pub const COUNTY: usize = 4;
    pub const SCHOOL_TYPE: usize = 5;
    pub const SOURCE: usize = 6;
    pub const K12: usize = 7;
    pub const OUT_OF_STATE: usize = 13;
    pub const PRESCHOOL: usize = 16;
    pub const PUBLISHED_TOTAL: usize = 17;
}

/// A count the department masked, held as the interval it can lie in.
///
/// The department writes `<10` over a cell it will not publish. The extraction does not guess:
/// it sums what survived, counts what did not, and carries the ceiling that follows from the
/// mask's own meaning — each masked cell is at most nine. So [`Bounded::floor`] is what the file
/// proves and [`Bounded::ceiling`] is what it permits, and the two are equal exactly when nothing
/// was masked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounded {
    /// The sum of the cells the department published: the smallest value consistent with the file.
    pub floor: f64,
    /// [`Bounded::floor`] plus nine for every masked cell: the largest such value.
    pub ceiling: f64,
    /// How many cells were masked.
    pub censored: usize,
}

impl Bounded {
    /// Whether nothing was masked, so the two ends are one number.
    #[must_use]
    pub fn is_exact(&self) -> bool {
        self.censored == 0
    }

    /// How much room the masking leaves: `ceiling - floor`, which is nine a masked cell.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.ceiling - self.floor
    }

    /// Whether `value` is consistent with what the file published.
    ///
    /// The test a published total has to pass before it can be believed to describe the same
    /// pupils as the building sheet under it.
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.floor && value <= self.ceiling
    }
}

/// One October, as one of the department's files states it.
#[derive(Debug, Clone, PartialEq)]
pub struct October {
    /// The school year the department names the sheet for, `1977-78` through `2025-26`.
    pub school_year: String,
    /// The fiscal year that October falls in: the school year's second half.
    pub fiscal_year: u16,
    /// Which shape the file was in — one of [`ANNUAL`], or [`COMPILATION`].
    pub basis: String,
    /// The registry key of the file this row was read from.
    pub source: String,
    /// How many buildings the file listed, where it listed any.
    pub buildings: Option<usize>,
    /// Kindergarten through twelve, bounded by the masking.
    pub k12: Option<Bounded>,
    /// The same pupils counted again by race, where the file published that block.
    pub race: Option<Bounded>,
    /// Pupils the file reports as living outside Ohio, bounded by the masking.
    pub out_of_state: Option<Bounded>,
    /// The department's own total for pupils living in Ohio, computed before the masking.
    pub published_in_state: Option<f64>,
    /// The department's own total for pupils living outside it.
    pub published_out_of_state: Option<f64>,
    /// The two above, as the department adds them.
    pub published_total: Option<f64>,
}

impl October {
    /// Whether this row is the October's own file rather than a later restatement of it.
    #[must_use]
    pub fn is_annual(&self) -> bool {
        ANNUAL.contains(&self.basis.as_str())
    }
}

/// One school in one October.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    /// The school year, as [`October::school_year`].
    pub school_year: String,
    /// The fiscal year that October falls in.
    pub fiscal_year: u16,
    /// The building IRN, and the only safe key — see the module note on `St Mary`.
    pub irn: String,
    /// The school's name as the file prints it, cleaned of punctuation the fixture cannot carry.
    pub school: String,
    /// The county, which the department published only through October 2006.
    pub county: String,
    /// `Elementary`, `High School` and the rest, published over the same years as the county.
    pub school_type: String,
    /// The registry key of the file this row was read from.
    pub source: String,
    /// Kindergarten through twelve at this building, bounded by the masking.
    pub k12: Option<Bounded>,
    /// Pupils at this building living outside Ohio.
    pub out_of_state: Option<Bounded>,
    /// Preschool, where the file counts it separately.
    pub preschool: Option<f64>,
    /// The building total the file printed, where it printed one.
    pub published_total: Option<f64>,
}

/// Every row of the sector extract, in fixture order: by October, then by file.
#[must_use]
pub fn octobers() -> Vec<October> {
    csv::rows(SECTOR, SECTOR_HEADER)
        .map(|row| October {
            school_year: row.str(sector_column::SCHOOL_YEAR).to_string(),
            fiscal_year: fiscal_year(row.str(sector_column::FISCAL_YEAR)),
            basis: row.str(sector_column::BASIS).to_string(),
            source: row.str(sector_column::SOURCE).to_string(),
            buildings: row
                .num(sector_column::BUILDINGS)
                .map(|count| count as usize),
            k12: bounded(&row, sector_column::K12),
            race: bounded(&row, sector_column::RACE),
            out_of_state: bounded(&row, sector_column::OUT_OF_STATE),
            published_in_state: row.num(sector_column::PUBLISHED_IN_STATE),
            published_out_of_state: row.num(sector_column::PUBLISHED_OUT_OF_STATE),
            published_total: row.num(sector_column::PUBLISHED_TOTAL),
        })
        .collect()
}

/// The October of `fiscal_year` as its own file states it.
///
/// The annual file, never the compilation: where the two disagree this returns the one published
/// at the time. [`restatement`] is the other side of the pair.
#[must_use]
pub fn october(fiscal_year: u16) -> Option<October> {
    octobers()
        .into_iter()
        .find(|october| october.fiscal_year == fiscal_year && october.is_annual())
}

/// The same October as the 2014-2019 compilation restates it, where it does.
///
/// Six Octobers, FY2014 through FY2019. The compilation counts preschool through twelve, so a
/// difference from [`october`] is expected in level; what is not expected, and happens anyway, is
/// FY2017 and FY2019 disagreeing on the in-state total.
#[must_use]
pub fn restatement(fiscal_year: u16) -> Option<October> {
    octobers()
        .into_iter()
        .find(|october| october.fiscal_year == fiscal_year && october.basis == COMPILATION)
}

/// The membership the statute's quotient divides by, for `fiscal_year`.
///
/// The department's own in-state total for the October of that fiscal year's school year — so
/// `membership(2025)` is October 2024. `None` before FY2016, where the department published no
/// total of its own and the file leaves only a bound; and for any year it has not yet counted.
///
/// In-state and not the whole, for the reason given in the module note: it is the reading that
/// reproduces LSC's published rate.
///
/// One year's answer should not be used without looking at it. October 2015's totals sheet states
/// 144,512 in-state, which is *below* the floor of its own building sheet — the two blocks of
/// `Revised-FY16-nonpub.xls` do not describe the same pupils, and the 2014-2019 compilation states
/// 172,990 for the same October. This function returns what the department published rather than
/// choosing between them; `the_october_the_statute_divides_by` is where the disagreement is pinned.
#[must_use]
pub fn membership(fiscal_year: u16) -> Option<f64> {
    october(fiscal_year)?.published_in_state
}

/// How many chartered nonpublic schools the department listed in `fiscal_year`'s October.
///
/// 711 in October 2023, 722 in October 2024, 749 in October 2025. `None` for October 2016, whose
/// file publishes a state total over the previous October's building sheet and so states no count
/// of its own.
#[must_use]
pub fn schools(fiscal_year: u16) -> Option<usize> {
    october(fiscal_year)?.buildings
}

/// The first and last October the extract reaches, as fiscal years.
///
/// Computed rather than written down: a fixture's span is the thing that moves when the
/// department posts next October, and a constant here would be a claim that went stale silently.
///
/// # Panics
///
/// If the sector extract is empty, which would mean the fixture stopped being built.
#[must_use]
pub fn span() -> (u16, u16) {
    let years: Vec<u16> = octobers().iter().map(|row| row.fiscal_year).collect();
    let first = years.iter().min().copied();
    let last = years.iter().max().copied();
    (
        first.expect("the sector extract carries at least one October"),
        last.expect("the sector extract carries at least one October"),
    )
}

/// Every school the department listed in `fiscal_year`'s October.
///
/// Reads the whole building extract each call, which is 39,440 rows of compiled-in text and costs
/// a few milliseconds. Call it once per year and hold the result.
#[must_use]
pub fn buildings(fiscal_year: u16) -> Vec<Building> {
    csv::rows(BUILDING, BUILDING_HEADER)
        .filter(|row| fiscal_year_of(row.str(building_column::FISCAL_YEAR)) == Some(fiscal_year))
        .map(|row| Building {
            school_year: row.str(building_column::SCHOOL_YEAR).to_string(),
            fiscal_year,
            irn: row.str(building_column::IRN).to_string(),
            school: row.str(building_column::SCHOOL).to_string(),
            county: row.str(building_column::COUNTY).to_string(),
            school_type: row.str(building_column::SCHOOL_TYPE).to_string(),
            source: row.str(building_column::SOURCE).to_string(),
            k12: bounded(&row, building_column::K12),
            out_of_state: bounded(&row, building_column::OUT_OF_STATE),
            preschool: row.num(building_column::PRESCHOOL),
            published_total: row.num(building_column::PUBLISHED_TOTAL),
        })
        .collect()
}

/// The three cells at `floor` as one [`Bounded`], or `None` where the file published no such block.
///
/// A missing floor and a missing ceiling are the same absence — the extraction writes both or
/// neither — so the floor alone decides.
fn bounded(row: &csv::Row<'_>, floor: usize) -> Option<Bounded> {
    Some(Bounded {
        floor: row.num(floor)?,
        ceiling: row.num(floor + 1)?,
        censored: row.num(floor + 2).unwrap_or(0.0) as usize,
    })
}

/// The fiscal year of a cell that the fixture always fills.
///
/// # Panics
///
/// If the column stopped being a year, which is a shifted fixture rather than a missing value.
fn fiscal_year(cell: &str) -> u16 {
    fiscal_year_of(cell).expect("the sector extract's fiscal year column is a year")
}

/// The same, where the caller is filtering and an unreadable cell should simply not match.
fn fiscal_year_of(cell: &str) -> Option<u16> {
    cell.parse().ok()
}
