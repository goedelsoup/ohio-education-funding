//! Chartered nonpublic enrolment by building, October 1977 through October 2025.
//!
//! The department publishes this as twenty-three attachments on one page: eighteen annual files,
//! one an October, and five compilations that carry the years before the annual series began. It
//! is the only place the denominator of R.C. 3317.024(E)(2)(d) — the membership in chartered
//! nonpublic schools "as determined as of the last day of October of each school year" — can be
//! read per building, and the only long series of the sector at all.
//!
//! # Every count under ten is masked, and the masking is not a small effect
//!
//! A cell printed [`SUPPRESSED`] stands for some number from one
//! to nine. In a file laid out one column per grade per sex, a school of forty pupils has most of
//! its cells masked, and Ohio's nonpublic sector is mostly small schools: in October 2023, 5,113
//! of the building-level cells are masked against a published in-state total of 171,830, and the
//! unmasked cells sum to 146,078. So the building rows bound the sector at **[146,078, 192,095]**
//! and no arithmetic over them recovers the published figure.
//!
//! Every quantity here is therefore a floor, a ceiling and a count of the cells that were
//! censored, never a point. Where a publisher states a true total the row carries it separately,
//! under a name that says which publisher said it.
//!
//! # The two totals a reader will reach for are different quantities
//!
//! **The historical compilations' `State Total` row is a post-masking sum.** It is an Excel
//! `SUM` over cells that had already been overwritten with the mask text, and `SUM` ignores text.
//! It equals the sum of the unmasked building cells **exactly, on all twenty-eight sheets that
//! carry one** — measured over the published files, and its consequence for the reader pinned by
//! `the_state_total_row_is_a_sum_of_what_survived_masking`: a sheet in that shape floors at
//! precisely that row, and its ceiling lies above it by nine a masked cell. The last three
//! sheets of the 2000-2008 compilation carry no such row at all.
//! It is a floor wearing the label of a total, and it is the single most dangerous cell in the
//! series: read as Ohio's nonpublic enrolment, 1977-78 comes out as 243,489 when the sector was
//! somewhere between 260,867 and 266,736.
//!
//! **The annual files' state-totals sheet is a true pre-masking total**, computed before the
//! building rows were censored. October 2023's says 171,830 where its own building rows floor at
//! 146,078. From FY2016 every annual file carries one; before that none does.
//!
//! # Two blocks measure the same pupils, and they do not agree
//!
//! The historical layout carries enrolment twice: once across twenty-six grade-and-sex columns
//! and once across five race columns. They are the same population, so each bounds it — and the
//! race block, being five columns wide rather than twenty-six, masks a fifth as many cells and
//! bounds it far better. For 1977-78 the grade block gives [243,489, 266,736] and the race block
//! [260,867, 270,209].
//!
//! They are not merely differently precise. In the years to 1982 a number of schools reported
//! race and left every grade cell zero, so the grade block is *absent* for them rather than
//! masked; from 1983-84 the two blocks agree on every building with no masked cell in either.
//! Both are therefore carried side by side and neither is reconciled into the other here. A
//! caller wanting one number per building per year should prefer the race block where it exists.
//!
//! # `nonpub_fy17.xls` is October 2015's building data under October 2016's name
//!
//! Its building sheet is `Revised-FY16-nonpub.xls`'s building sheet: 5,883 of 5,887 keyed cells
//! are present in both and **none of the shared cells differs**. Matched against the 2014-2019
//! compilation, which states a per-school total for each of FY2014 through FY2019, the shared
//! sheet agrees exactly with FY2016 on 86 unmasked schools and with no other year on more than
//! six. So the rows are FY2016's, the state-totals sheet beside them is genuinely FY2017's, and
//! **FY2017 has no building-level data of its own**. [`BuildingRows::RepostedFrom`] is how the
//! file table says so; taking the sheet at its filename publishes 2015's schools as 2016's.
//!
//! `Revised-FY16-nonpub.xls`'s own state-totals sheet is separately wrong: it reads 144,512,
//! which is below its own building rows' floor of 147,309 and so cannot be a total of them. The
//! compilation's 171,395 is the figure for that October.
//!
//! # The `_ADM` in the column names is not ADM
//!
//! Every annual file's notes sheet says the counts are a headcount taken during the first full
//! week of October and are **not** equivalent to average daily membership, while the columns
//! those counts sit in are named `BOYS_ADM`, `MALE_ADM` and so on. The notes sheet is right.

use super::format::clean_name;
use crate::conventions::{number, SUPPRESSED};

/// Columns of the building panel.
///
/// One row per school year and building IRN. A blank is "this era does not publish that", never
/// zero — the eras differ in what they carry, and [`super::paths::NONPUBLIC_BUILDING_FIXTURE`]
/// spans forty-nine of them.
pub const NONPUBLIC_BUILDING_HEADER: &[&str] = &[
    "school_year",
    "fiscal_year",
    "irn",
    "school",
    "county",
    "school_type",
    "source",
    "k12_floor",
    "k12_ceiling",
    "k12_censored",
    "race_floor",
    "race_ceiling",
    "race_censored",
    "out_of_state_floor",
    "out_of_state_ceiling",
    "out_of_state_censored",
    "preschool",
    "published_total",
    "published_basis",
];

/// Columns of the sector panel.
///
/// Keyed on school year **and basis**, not on school year alone: for FY2014 through FY2019 two
/// publications describe the same October and they do not agree, so each gets a row and the
/// disagreement is data rather than a resolution taken here.
pub const NONPUBLIC_SECTOR_HEADER: &[&str] = &[
    "school_year",
    "fiscal_year",
    "basis",
    "source",
    "buildings",
    "k12_floor",
    "k12_ceiling",
    "k12_censored",
    "race_floor",
    "race_ceiling",
    "race_censored",
    "out_of_state_floor",
    "out_of_state_ceiling",
    "out_of_state_censored",
    "published_in_state",
    "published_out_of_state",
    "published_total",
];

/// Whether a file's building rows are its own year's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingRows {
    /// The building sheet holds the October the file is named for.
    Own,
    /// The building sheet is a repost of an earlier October's, which the named year therefore
    /// has no building-level data for. Only the file's other sheets are read.
    ///
    /// One file is in this state; the module documentation says how it was identified.
    RepostedFrom(u16),
}

/// One sheet of a published workbook.
#[derive(Debug, Clone, Copy)]
pub struct NonpublicSheet<'a> {
    /// The sheet's own name, which is how the compilations say which October a sheet holds.
    pub name: &'a str,
    /// Its rows, as the workbook reader returned them.
    pub rows: &'a [Vec<String>],
}

/// One published file.
#[derive(Debug, Clone)]
pub struct NonpublicBook<'a> {
    /// The [`crate::registry::Source`] key, which every row it produces carries.
    pub key: &'a str,
    /// The fiscal year an annual file holds, or `None` for a compilation whose sheets each say
    /// their own.
    ///
    /// Declared rather than parsed because the filenames and the sheet names both lie: October
    /// 2023's file is `nonpub_fy24.xls` and its state-totals sheet inside is named
    /// `fy23_nonpub_state`.
    pub fiscal_year: Option<u16>,
    /// Whether to trust the building sheet's year.
    pub buildings: BuildingRows,
    /// Every sheet in the file.
    pub sheets: &'a [NonpublicSheet<'a>],
}

/// A quantity known only up to the cells that were masked out of it.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Bound {
    floor: f64,
    censored: usize,
}

impl Bound {
    /// Add one published cell.
    ///
    /// A masked cell raises the ceiling by nine and the floor by nothing, which is the whole
    /// content of the mask. A blank or an error marker adds neither: it is not a censored one,
    /// it is a cell the publisher left out.
    fn push(&mut self, cell: &str) {
        if cell.trim() == SUPPRESSED {
            self.censored += 1;
        } else if let Some(value) = number(cell) {
            self.floor += value;
        }
    }

    fn merge(&mut self, other: Self) {
        self.floor += other.floor;
        self.censored += other.censored;
    }

    /// The largest the quantity can be: every masked cell at nine.
    fn ceiling(self) -> f64 {
        self.floor + 9.0 * self.censored as f64
    }

    /// Whether any cell at all was seen.
    fn is_empty(self) -> bool {
        self.floor == 0.0 && self.censored == 0
    }
}

/// The department's label for the school year ending in `fiscal_year`.
///
/// Derived rather than read off the sheet name so that the annual files, whose sheets carry no
/// year at all, are labelled the same way the compilations are. The derivation is checked
/// against the compilations' own sheet names by
/// [`the_derived_label_matches_the_sheet_the_department_named`].
fn school_year(fiscal_year: u16) -> String {
    format!("{}-{:02}", fiscal_year - 1, fiscal_year % 100)
}

/// Where the eras differ, and what identifies each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// One row per school, grades across `Boys`/`Girls` column pairs, and in most years a race
    /// block beside them. Every sheet of the four historical compilations but the last.
    Wide,
    /// One row per school **and grade**. The 2007-08 sheet of the 2000-2008 compilation, and
    /// every annual file from October 2008 but October 2009's.
    ByGrade,
    /// One row per school carrying a single number. October 2009 is published only this way.
    BuildingTotals,
    /// The 2014-2019 compilation's per-school sheet.
    CompilationSchools,
    /// An annual file's state-totals sheet: one data row, grades across the columns.
    StateTotals,
    /// The 2014-2019 compilation's state-totals sheet, which carries two grade spans a year.
    CompilationTotals,
}

/// A header row, lowercased, with the index it was found at.
struct Header {
    at: usize,
    cells: Vec<String>,
}

impl Header {
    /// The index of the first column named any of `names`.
    fn any(&self, names: &[&str]) -> Option<usize> {
        self.cells.iter().position(|c| names.contains(&c.as_str()))
    }

    fn has(&self, names: &[&str]) -> bool {
        self.any(names).is_some()
    }
}

/// Find the header row and say what the sheet is.
///
/// Every column is resolved by name, because the names move: the grade column is
/// `SCHOOL_GRADE_DESCR` to October 2013 and `GRADE_LEVEL_CODE` after, the sex columns are
/// `BOYS_ADM`/`GIRLS_ADM` to October 2018 and `MALE_ADM`/`FEMALE_ADM` after, the case changes
/// from file to file, and three files spell the second one `FAMALE`. Positional reads splice
/// them silently.
fn classify(rows: &[Vec<String>]) -> Option<(Layout, Header)> {
    for (at, row) in rows.iter().take(6).enumerate() {
        let cells: Vec<String> = row.iter().map(|c| c.trim().to_lowercase()).collect();
        let header = Header { at, cells };
        let layout = if header.has(&["school_grade_descr", "grade_level_code"]) {
            Layout::ByGrade
        } else if header.has(&["school name"]) && header.has(&["boys"]) {
            Layout::Wide
        } else if header.has(&["school_irn"]) && header.has(&["adm"]) {
            Layout::BuildingTotals
        } else if header.has(&["fy"]) && header.has(&["grades"]) {
            Layout::CompilationTotals
        } else if header.has(&["fy"]) && header.has(&["irn"]) {
            Layout::CompilationSchools
        } else if header.has(&["male", "famale"]) && header.has(&["kindergarten"]) {
            Layout::StateTotals
        } else {
            continue;
        };
        return Some((layout, header));
    }
    None
}

/// The out-of-state pair, under either generation of names.
const OUT_OF_STATE: (&[&str], &[&str]) = (
    &["out_of_state_boys_count", "out_of_state_male_count"],
    &["out_of_state_girls_count", "out_of_state_female_count"],
);

/// One building's year, before it is rendered.
#[derive(Debug, Default, Clone)]
struct Building {
    school: String,
    county: String,
    school_type: String,
    source: String,
    k12: Bound,
    race: Bound,
    out_of_state: Bound,
    preschool: Option<f64>,
    published: Option<f64>,
    published_basis: &'static str,
}

/// What one source said about one October.
#[derive(Debug, Default, Clone)]
struct Sector {
    basis: &'static str,
    source: String,
    buildings: usize,
    k12: Bound,
    race: Bound,
    out_of_state: Bound,
    published_in_state: Option<f64>,
    published_out_of_state: Option<f64>,
    published_total: Option<f64>,
}

/// Every building the sheets describe, keyed on the October and the IRN.
///
/// A map rather than a list because two publications describe the same building in the same
/// October: the annual file bounds its K-12 count and the 2014-2019 compilation states its PS-12
/// total outright, and they belong on one row.
type Buildings = std::collections::BTreeMap<(u16, String), Building>;

/// Every October each publication describes, keyed on the October and the basis.
type Sectors = std::collections::BTreeMap<(u16, &'static str), Sector>;

/// The two panels: buildings, then the sector.
pub type Panels = (Vec<Vec<String>>, Vec<Vec<String>>);

/// The fiscal year a sheet holds.
///
/// An annual file declares it, because neither its filename nor its sheet names can be trusted
/// to. A compilation's sheet is named for the school year it holds, so `2005-06 ethnic data
/// unavailable` yields 2006.
fn fiscal_year_of(book: &NonpublicBook<'_>, sheet: &NonpublicSheet<'_>) -> Result<u16, String> {
    if let Some(declared) = book.fiscal_year {
        return Ok(declared);
    }
    let digits: String = sheet
        .name
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let start: u16 = digits
        .parse()
        .map_err(|_| format!("sheet `{}` of `{}` names no year", sheet.name, book.key))?;
    Ok(start + 1)
}

/// Read one sheet of a historical compilation, laid out one row per school.
fn read_wide(
    book: &NonpublicBook<'_>,
    sheet: &NonpublicSheet<'_>,
    header: &Header,
    buildings: &mut Buildings,
    sectors: &mut Sectors,
) -> Result<(), String> {
    let irn_at = header
        .any(&["irn"])
        .ok_or_else(|| format!("sheet `{}` has no IRN column", sheet.name))?;
    let name_at = header.any(&["school name"]).unwrap_or(0);
    let county_at = header.any(&["county"]);
    // Absent in 1999-00 and 2000-01, whose fifth column is labelled `School Name` a second time.
    let type_at = header.any(&["school type"]);
    let grades: Vec<usize> = positions(header, &["boys", "girls"]);
    let races: Vec<usize> = positions(header, RACES);
    let total_at = header.any(&["total"]);
    let fiscal_year = fiscal_year_of(book, sheet)?;
    let mut sector = Sector {
        basis: "wide-grade-and-race",
        source: book.key.to_string(),
        ..Sector::default()
    };

    for row in &sheet.rows[header.at + 1..] {
        let irn = cell(row, irn_at).trim();
        if irn.is_empty() {
            continue;
        }
        // The `State Total` row is a post-masking sum and carries nothing the floor does not.
        if cell(row, name_at).to_lowercase().contains("total") {
            continue;
        }
        let mut building = Building {
            school: clean_name(cell(row, name_at)),
            county: county_at
                .map(|at| clean_name(cell(row, at)))
                .unwrap_or_default(),
            school_type: type_at
                .map(|at| clean_name(cell(row, at)))
                .unwrap_or_default(),
            source: book.key.to_string(),
            ..Building::default()
        };
        for &at in &grades {
            building.k12.push(cell(row, at));
        }
        for &at in &races {
            building.race.push(cell(row, at));
        }
        // Only 2001-02 states a per-school total, and it is the sum of the grade block.
        building.published = total_at.and_then(|at| number(cell(row, at)));
        if building.published.is_some() {
            building.published_basis = "grade-block-total-column";
        }
        sector.k12.merge(building.k12);
        sector.race.merge(building.race);
        sector.buildings += 1;
        buildings.insert((fiscal_year, irn.to_string()), building);
    }

    sectors.insert((fiscal_year, sector.basis), sector);
    Ok(())
}

/// The five race columns, which are named this way in every year that carries them.
const RACES: &[&str] = &["indian", "asian", "black", "white", "hispanic"];

/// Every column named any of `names`.
fn positions(header: &Header, names: &[&str]) -> Vec<usize> {
    header
        .cells
        .iter()
        .enumerate()
        .filter(|(_, c)| names.contains(&c.as_str()))
        .map(|(at, _)| at)
        .collect()
}

/// A cell by index, trimmed, or `""`.
fn cell(row: &[String], at: usize) -> &str {
    row.get(at).map_or("", |c| c.trim())
}

/// Read a sheet laid out one row per school and grade.
fn read_by_grade(
    book: &NonpublicBook<'_>,
    sheet: &NonpublicSheet<'_>,
    header: &Header,
    buildings: &mut Buildings,
    sectors: &mut Sectors,
) -> Result<(), String> {
    // A reposted sheet holds an October that already has its own file. Reading it would publish
    // one year's schools under the next year's label.
    if book.buildings != BuildingRows::Own {
        return Ok(());
    }
    let irn_at = header
        .any(&["irn"])
        .ok_or_else(|| format!("sheet `{}` has no IRN column", sheet.name))?;
    let name_at = header.any(&["org_nm"]);
    let male_at = header.any(&["boys_adm", "male_adm"]);
    let female_at = header.any(&["girls_adm", "female_adm", "famale_adm"]);
    let (Some(male_at), Some(female_at)) = (male_at, female_at) else {
        return Err(format!("sheet `{}` has no sex columns", sheet.name));
    };
    let out_of_state = [header.any(OUT_OF_STATE.0), header.any(OUT_OF_STATE.1)];
    // School-level, and repeated identically on every one of the school's grade rows. Summing
    // them multiplies a school's preschool by its number of grades.
    let preschool = [
        header.any(&["preschool_boys_count", "preschool_male_count"]),
        header.any(&["preschool_girls_count", "preschool_female_count"]),
    ];
    let fiscal_year = fiscal_year_of(book, sheet)?;
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for row in &sheet.rows[header.at + 1..] {
        let irn = cell(row, irn_at);
        if irn.is_empty() {
            continue;
        }
        let entry = buildings.entry((fiscal_year, irn.to_string())).or_default();
        if seen.insert(irn.to_string()) {
            entry.school = name_at
                .map(|at| clean_name(cell(row, at)))
                .unwrap_or_default();
            entry.source = book.key.to_string();
            entry.preschool = preschool
                .iter()
                .flatten()
                .map(|&at| number(cell(row, at)))
                .try_fold(0.0, |total, value| value.map(|value| total + value));
        }
        entry.k12.push(cell(row, male_at));
        entry.k12.push(cell(row, female_at));
        for &at in out_of_state.iter().flatten() {
            entry.out_of_state.push(cell(row, at));
        }
    }

    // The state-totals sheet may already have made this October's row: from October 2020 the
    // annual files put it *before* the building sheet, and inserting over it would drop the
    // published total — which is the one number in the file that is not a bound.
    let entry = sectors
        .entry((fiscal_year, "by-grade-and-sex"))
        .or_insert_with(|| Sector {
            basis: "by-grade-and-sex",
            ..Sector::default()
        });
    entry.source = book.key.to_string();
    entry.buildings = seen.len();
    for irn in &seen {
        if let Some(building) = buildings.get(&(fiscal_year, irn.clone())) {
            entry.k12.merge(building.k12);
            entry.out_of_state.merge(building.out_of_state);
        }
    }
    Ok(())
}

/// Read October 2009, which is published as one number a school and nothing else.
fn read_building_totals(
    book: &NonpublicBook<'_>,
    sheet: &NonpublicSheet<'_>,
    header: &Header,
    buildings: &mut Buildings,
    sectors: &mut Sectors,
) -> Result<(), String> {
    let irn_at = header
        .any(&["school_irn"])
        .ok_or_else(|| format!("sheet `{}` has no IRN column", sheet.name))?;
    let name_at = header.any(&["school_name"]);
    let adm_at = header
        .any(&["adm"])
        .ok_or_else(|| format!("sheet `{}` has no ADM column", sheet.name))?;
    let fiscal_year = fiscal_year_of(book, sheet)?;
    let mut sector = Sector {
        basis: "building-adm",
        source: book.key.to_string(),
        ..Sector::default()
    };

    for row in &sheet.rows[header.at + 1..] {
        let irn = cell(row, irn_at);
        if irn.is_empty() {
            continue;
        }
        let mut building = Building {
            school: name_at
                .map(|at| clean_name(cell(row, at)))
                .unwrap_or_default(),
            source: book.key.to_string(),
            ..Building::default()
        };
        building.k12.push(cell(row, adm_at));
        sector.k12.merge(building.k12);
        sector.buildings += 1;
        buildings.insert((fiscal_year, irn.to_string()), building);
    }

    sector.published_in_state = Some(sector.k12.floor);
    sectors.insert((fiscal_year, sector.basis), sector);
    Ok(())
}

/// Read the 2014-2019 compilation's per-school sheet.
///
/// Its totals are **PS-12 and unmasked** where the annual files of the same Octobers are K-12 and
/// masked, so they are carried as a stated total beside the bounds rather than merged into them.
/// Thirty of its 4,286 rows are masked; the rest are points.
fn read_compilation_schools(
    book: &NonpublicBook<'_>,
    sheet: &NonpublicSheet<'_>,
    header: &Header,
    buildings: &mut Buildings,
) -> Result<(), String> {
    let year_at = header
        .any(&["fy"])
        .ok_or_else(|| format!("sheet `{}` has no FY column", sheet.name))?;
    let irn_at = header
        .any(&["irn"])
        .ok_or_else(|| format!("sheet `{}` has no IRN column", sheet.name))?;
    let name_at = header.any(&["school"]);
    let total_at = header
        .any(&["ps-12 total"])
        .ok_or_else(|| format!("sheet `{}` has no total column", sheet.name))?;

    for row in &sheet.rows[header.at + 1..] {
        let irn = cell(row, irn_at);
        let Some(fiscal_year) = number(cell(row, year_at)) else {
            continue;
        };
        if irn.is_empty() {
            continue;
        }
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let fiscal_year = fiscal_year as u16;
        let entry = buildings.entry((fiscal_year, irn.to_string())).or_default();
        entry.published = number(cell(row, total_at));
        entry.published_basis = "ps12-compilation";
        if entry.school.is_empty() {
            entry.school = name_at
                .map(|at| clean_name(cell(row, at)))
                .unwrap_or_default();
        }
        if entry.source.is_empty() {
            entry.source = book.key.to_string();
        }
    }

    Ok(())
}

/// Read an annual file's state-totals sheet.
///
/// **This is a true total, computed before the building rows were masked.** It is the only place
/// the sector's October count can be read as a number rather than a range, and it exists only
/// from October 2015.
fn read_state_totals(
    book: &NonpublicBook<'_>,
    sheet: &NonpublicSheet<'_>,
    header: &Header,
    sectors: &mut Sectors,
) -> Result<(), String> {
    let Some(row) = sheet.rows.get(header.at + 1) else {
        return Ok(());
    };
    let fiscal_year = fiscal_year_of(book, sheet)?;
    let at = |names: &[&str]| header.any(names).and_then(|at| number(cell(row, at)));
    let male = at(&["male"]);
    let female = at(&["female", "famale"]);
    let in_state = match (male, female) {
        (Some(male), Some(female)) => Some(male + female),
        _ => None,
    };
    let out_of_state = match (at(&["out-of-state male"]), at(&["out-of-state female"])) {
        (Some(male), Some(female)) => Some(male + female),
        _ => None,
    };
    let entry = sectors
        .entry((fiscal_year, "by-grade-and-sex"))
        .or_insert_with(|| Sector {
            basis: "by-grade-and-sex",
            source: book.key.to_string(),
            ..Sector::default()
        });
    entry.published_in_state = in_state;
    entry.published_out_of_state = out_of_state;
    entry.published_total = match (in_state, out_of_state) {
        (Some(in_state), Some(out_of_state)) => Some(in_state + out_of_state),
        _ => in_state,
    };
    Ok(())
}

/// Read the 2014-2019 compilation's state-totals sheet.
///
/// It states two grade spans an October, `PK-12` and `K-12`. Only the second is comparable with
/// the annual files, whose building rows carry no preschool row in any era.
fn read_compilation_totals(
    book: &NonpublicBook<'_>,
    sheet: &NonpublicSheet<'_>,
    header: &Header,
    sectors: &mut Sectors,
) {
    let (Some(year_at), Some(span_at)) = (header.any(&["fy"]), header.any(&["grades"])) else {
        return;
    };
    let in_state_at = header.any(&["in-state total"]);
    let total_at = header.any(&["total"]);

    for row in &sheet.rows[header.at + 1..] {
        if !cell(row, span_at).eq_ignore_ascii_case("K-12") {
            continue;
        }
        let Some(fiscal_year) = number(cell(row, year_at)) else {
            continue;
        };
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let fiscal_year = fiscal_year as u16;
        let in_state = in_state_at.and_then(|at| number(cell(row, at)));
        let total = total_at.and_then(|at| number(cell(row, at)));
        let entry = sectors
            .entry((fiscal_year, "ps12-compilation"))
            .or_insert_with(|| Sector {
                basis: "ps12-compilation",
                source: book.key.to_string(),
                ..Sector::default()
            });
        entry.published_in_state = in_state;
        entry.published_total = total;
        entry.published_out_of_state = match (in_state, total) {
            (Some(in_state), Some(total)) => Some(total - in_state),
            _ => None,
        };
    }
}

/// Build both panels from every published file.
///
/// Returns the building panel and the sector panel, each sorted: the building panel by school
/// year then IRN, the sector panel by school year then basis.
///
/// # Errors
///
/// If a sheet that classified carries no column it was classified for, or if a compilation
/// sheet names no year.
pub fn build_nonpublic(books: &[NonpublicBook<'_>]) -> Result<Panels, String> {
    let mut buildings = Buildings::new();
    let mut sectors = Sectors::new();

    for book in books {
        for sheet in book.sheets {
            let Some((layout, header)) = classify(sheet.rows) else {
                continue;
            };
            match layout {
                Layout::Wide => read_wide(book, sheet, &header, &mut buildings, &mut sectors)?,
                Layout::ByGrade => {
                    read_by_grade(book, sheet, &header, &mut buildings, &mut sectors)?;
                }
                Layout::BuildingTotals => {
                    read_building_totals(book, sheet, &header, &mut buildings, &mut sectors)?;
                }
                Layout::CompilationSchools => {
                    read_compilation_schools(book, sheet, &header, &mut buildings)?;
                }
                Layout::StateTotals => read_state_totals(book, sheet, &header, &mut sectors)?,
                Layout::CompilationTotals => {
                    read_compilation_totals(book, sheet, &header, &mut sectors);
                }
            }
        }
    }

    // The compilation states a per-school total for Octobers whose annual file states bounds.
    // Its own sector row counts the buildings it names, which the per-school pass has by now
    // merged into the building map.
    for ((fiscal_year, basis), sector) in &mut sectors {
        if *basis == "ps12-compilation" {
            sector.buildings = buildings
                .iter()
                .filter(|((year, _), b)| year == fiscal_year && b.published.is_some())
                .count();
        }
    }

    let building_rows = buildings
        .into_iter()
        .map(|((fiscal_year, irn), building)| render_building(fiscal_year, &irn, &building))
        .collect();
    let sector_rows = sectors
        .into_iter()
        .map(|((fiscal_year, _), sector)| render_sector(fiscal_year, &sector))
        .collect();

    Ok((building_rows, sector_rows))
}

/// An integer-valued count, rendered without a decimal point.
fn count(value: f64) -> String {
    format!("{value:.0}")
}

fn optional(value: Option<f64>) -> String {
    value.map_or_else(String::new, count)
}

/// A bound's three columns, blank where the era published nothing to bound.
fn bound_columns(bound: Bound) -> [String; 3] {
    if bound.is_empty() {
        return [String::new(), String::new(), String::new()];
    }
    [
        count(bound.floor),
        count(bound.ceiling()),
        bound.censored.to_string(),
    ]
}

fn render_building(fiscal_year: u16, irn: &str, building: &Building) -> Vec<String> {
    let mut row = vec![
        school_year(fiscal_year),
        fiscal_year.to_string(),
        irn.to_string(),
        building.school.clone(),
        building.county.clone(),
        building.school_type.clone(),
        building.source.clone(),
    ];
    row.extend(bound_columns(building.k12));
    row.extend(bound_columns(building.race));
    row.extend(bound_columns(building.out_of_state));
    row.push(optional(building.preschool));
    row.push(optional(building.published));
    row.push(building.published_basis.to_string());
    row
}

fn render_sector(fiscal_year: u16, sector: &Sector) -> Vec<String> {
    let mut row = vec![
        school_year(fiscal_year),
        fiscal_year.to_string(),
        sector.basis.to_string(),
        sector.source.clone(),
        // Blank, not zero: October 2016's file publishes a state total over a building sheet
        // that is the previous October's, so it states no buildings of its own.
        if sector.buildings == 0 {
            String::new()
        } else {
            sector.buildings.to_string()
        },
    ];
    row.extend(bound_columns(sector.k12));
    row.extend(bound_columns(sector.race));
    row.extend(bound_columns(sector.out_of_state));
    row.push(optional(sector.published_in_state));
    row.push(optional(sector.published_out_of_state));
    row.push(optional(sector.published_total));
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|c| (*c).to_string()).collect()
    }

    /// The historical layout, narrowed to two grades and the race block.
    ///
    /// The three header rows are the department's: a banner, a grade row spanning `Boys`/`Girls`
    /// pairs, and the row that names the columns. `State Total` carries no IRN, which is what
    /// keeps it out of the building rows.
    fn wide_sheet() -> Vec<Vec<String>> {
        vec![
            row(&["", "", "", "", "", "Enrollment by Grade"]),
            row(&["", "", "", "", "", "Kindergarten", "", "1st Grade"]),
            row(&[
                "School Name",
                "Year",
                "County",
                "IRN",
                "School Type",
                "Boys",
                "Girls",
                "Boys",
                "Girls",
                "",
                "Indian",
                "Asian",
                "Black",
                "White",
                "Hispanic",
            ]),
            row(&[
                "Lima Central Catholic High Sch",
                "1978",
                "Allen",
                "053165",
                "High School",
                "20",
                "30",
                "40",
                "50",
                "",
                "0",
                "0",
                "49",
                "91",
                "0",
            ]),
            row(&[
                "St Mary Elementary School",
                "1978",
                "Wyandot",
                "059188",
                "Elementary",
                "<10",
                "<10",
                "11",
                "<10",
                "",
                "0",
                "0",
                "0",
                "44",
                "0",
            ]),
            // What the department's sheet carries: an Excel SUM over cells that already held
            // the mask text, which SUM ignores — so the three `<10` cells contribute nothing.
            row(&[
                "State Total",
                "",
                "",
                "",
                "",
                "20",
                "30",
                "51",
                "50",
                "",
                "0",
                "0",
                "49",
                "135",
                "0",
            ]),
        ]
    }

    /// The `State Total` row is the floor, not the total.
    ///
    /// Measured over the published files — it holds exactly on all twenty-seven sheets that
    /// carry such a row — and pinned here in the reader: the bound built from the building rows
    /// floors at precisely that row's arithmetic, and its ceiling is above it by nine a masked
    /// cell. A caller reading the row as the sector's enrolment reads the bottom of the range.
    #[test]
    fn the_state_total_row_is_a_sum_of_what_survived_masking() {
        let rows = wide_sheet();
        let sheets = [NonpublicSheet {
            name: "1977-78",
            rows: &rows,
        }];
        let books = [NonpublicBook {
            key: "test",
            fiscal_year: None,
            buildings: BuildingRows::Own,
            sheets: &sheets,
        }];
        let (buildings, sectors) = build_nonpublic(&books).expect("the sheet reads");

        // The State Total row carries no IRN and is not a building.
        assert_eq!(buildings.len(), 2);

        let sector = &sectors[0];
        let state_total: f64 = [20.0, 30.0, 51.0, 50.0].iter().sum();
        assert_eq!(
            sector[5],
            count(state_total),
            "k12_floor is the State Total"
        );
        assert_eq!(sector[7], "3", "three cells were masked");
        assert_eq!(
            sector[6],
            count(state_total + 27.0),
            "the ceiling is nine a masked cell above it"
        );
    }

    /// The label this module derives is the label the department typed.
    ///
    /// The annual files name no school year anywhere — their sheets are `fy24_nonpub` and their
    /// filenames lie — so the label has to be derived from the fiscal year. The compilations do
    /// name it, on every sheet, which makes them the check: these are the department's own sheet
    /// names, and [`school_year`] must reproduce each one from the year [`fiscal_year_of`] reads
    /// out of it.
    #[test]
    fn the_derived_label_matches_the_sheet_the_department_named() {
        const PUBLISHED: &[&str] = &[
            "1977-78",
            "1978-79",
            "1979-80",
            "1988-89",
            "1989-90",
            "1998-99",
            "1999-00",
            "2000-01",
            "2001-02",
            "2002-03 ethnic data unavailable",
            "2006-07 ethnic data unavailable",
            "2007-08 ethnic data unavailable",
        ];
        let empty: Vec<Vec<String>> = Vec::new();
        for name in PUBLISHED {
            let sheet = NonpublicSheet { name, rows: &empty };
            let book = NonpublicBook {
                key: "test",
                fiscal_year: None,
                buildings: BuildingRows::Own,
                sheets: std::slice::from_ref(&sheet),
            };
            let fiscal_year = fiscal_year_of(&book, &sheet).expect("the sheet names a year");
            assert_eq!(
                school_year(fiscal_year),
                name[..7],
                "the derived label for `{name}`"
            );
        }
    }
}
