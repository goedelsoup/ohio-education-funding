//! Where every committed fixture is written, and the manifest of the ones the rebuild produces.
//!
//! Thirty-two path constants and [`REBUILT`], which lists the subset a full rebuild regenerates.
//!
//! # Why they are together
//!
//! They used to be scattered across 5,500 lines of parsing code — `FY27_FIXTURE` at line 1,711,
//! `CPI_FIXTURE` at 2,415, `F33_FIXTURE` at 3,112, `MR81_FIXTURE` at 5,280 — and `REBUILT`
//! forward-referenced eight constants defined up to 2,900 lines *later*. Nothing about the
//! manifest could be checked by reading it, which is the one thing a manifest is for.
//!
//! A new fixture registers in one place now, and `registry`'s test that every rebuilt path is
//! claimed by a connector has a list it can be read against.

/// Where each fixture is written, relative to the repository root.
pub const FY27_FIXTURE: &str = "crates/foundation/fixtures/fy27-department-model.csv";
/// Where the district-profile fixture is written, relative to the repository root.
pub const PROFILE_FIXTURE: &str = "crates/dispersion/fixtures/cupp-fy24-district-data.csv";
/// Where the grade-band headcount fixture is written, relative to the repository root.
pub const GRADE_BANDS_FIXTURE: &str = "crates/foundation/fixtures/fy24-district-grade-bands.csv";

/// Where the per-district financial panel is written, relative to the repository root.
pub const FINANCE_FIXTURE: &str = "crates/project/fixtures/district-finances.csv";

/// Where the SD-1 taxable value and taxes charged panel is written, relative to the root.
pub const SD1_FIXTURE: &str = "crates/dispersion/fixtures/sd1-district-taxes.csv";

/// Where the per-district casino distribution panel is written, relative to the root.
pub const CASINO_FIXTURE: &str = "crates/dispersion/fixtures/casino-district-distributions.csv";

/// LSC's education analysis of every enacted budget act the corpus describes from one.
///
/// One file rather than one per act, on the same ground [`OPINIONS_FIXTURE`] holds four opinions:
/// they are the same kind of document about the same question, read together or not at all. A file
/// that reads as a series and quietly omits the biennium whose PDF did not download is worse than
/// no file, because a gap in the record and a gap in the cache look identical from the fixture.
///
/// **Named for what it holds rather than for when.** It was `bridge-era-greenbooks` for exactly one
/// commit, which was long enough to be wrong: H.B. 95 and H.B. 119 are the two budgets before the
/// Evidence-Based Model and are not Bridge era at all. A fixture named after the first thing put
/// in it has to be renamed or lied about the second time, and this repository has a README's worth
/// of evidence about which of those actually happens.
///
/// **As enrolled, which is the enacted document.** Not a redbook. That distinction has cost this
/// repository a `[verified]` before: a redbook figure is the executive's proposal, and marking one
/// verified reads as though the act had been checked. Every analysis here is LSC's account of what
/// passed.
///
/// **Twelve records and about 2.6 MB, which is a deliberate choice and the largest fixture here.**
/// It is one document per General Assembly from the 124th to the 135th, unbroken — every act that
/// set Ohio's school funding formula from FY2002 to FY2025. Committed whole rather than sliced for
/// the reason the H.B. 96 greenbook is: a greenbook is about one agency and there is nothing in it
/// to cut away. The alternative to the size is that no claim about any of these acts can be
/// `[verified]` at all, which is what the class looked like three phases ago.
///
/// The **136th is absent on purpose**. H.B. 96's greenbook is committed on its own at
/// [`GREENBOOK_FIXTURE`], where corpus nodes and tests already cite it; duplicating a quarter of a
/// megabyte to make one file look complete would be paying in bytes for a tidier heading.
pub const LSC_GREENBOOK_FIXTURE: &str = "crates/project/fixtures/lsc-education-greenbooks.txt";

/// Where the deflator's check fixture is written, relative to the repository root.
pub const CPI_FIXTURE: &str = "crates/connect/fixtures/cpi-u-june.tsv";
/// Where the 2024-25 report card fixture is written, relative to the repository root.
pub const REPORT_CARD_FIXTURE: &str =
    "crates/dispersion/fixtures/report-card-2425-district-data.csv";
/// Where the FY2025 expenditure-function fixture is written, relative to the repository root.
pub const FUNCTIONS_FIXTURE: &str = "crates/dispersion/fixtures/expenditure-functions-fy25.csv";

/// Where the Census F-33 state comparison is written, relative to the root.
pub const F33_FIXTURE: &str = "crates/dispersion/fixtures/census-f33-states.csv";

/// Where the FY2024 state cross-section is written, relative to the repository root.
///
/// A second fixture rather than a year column on the first, because the FY2022 file is what the
/// corpus's published interstate figures were computed from and a test pins them. Two years side
/// by side is also the only way to see that the Bureau's own per-pupil figures are not
/// reconstructible from these files — see the WP015 catalog entry.
pub const F33_FY2024_FIXTURE: &str = "crates/dispersion/fixtures/census-f33-states-fy2024.csv";

/// Where the per-district F-33 extract is written, relative to the repository root.
pub const F33_DISTRICTS_FIXTURE: &str = "crates/dispersion/fixtures/f33-districts-fy2022.csv";

/// Where the multi-year Ohio F-33 panel is written, relative to the repository root.
pub const F33_OHIO_PANEL_FIXTURE: &str = "crates/dispersion/fixtures/f33-ohio-panel.csv";

/// Where the statute extract is written, relative to the repository root.
pub const STATUTE_FIXTURE: &str = "crates/project/fixtures/revised-code.txt";

/// Where the enacted-act extract is written, relative to the repository root.
pub const ENACTED_FIXTURE: &str = "crates/project/fixtures/enacted-school-funding.txt";

/// Where the department's redbook extract is written, relative to the repository root.
pub const REDBOOK_FIXTURE: &str = "crates/project/fixtures/dew-redbook.txt";

/// The same analysis as enacted. See [`REDBOOK_FIXTURE`], which is the introduced one.
pub const GREENBOOK_FIXTURE: &str = "crates/project/fixtures/dew-greenbook.txt";

/// Where the court opinion extract is written, relative to the repository root.
pub const OPINIONS_FIXTURE: &str = "crates/regime-diff/fixtures/derolph-opinions.txt";

/// Where the EdChoice challenge's appellate record is written, relative to the repository root.
///
/// A second file rather than a fifth record in [`OPINIONS_FIXTURE`], because that file's own first
/// line says *"DeRolph v. State, the four opinions this corpus cites"* and this is neither. The
/// rebuild now selects a connector's sources by the fixture each one declares it feeds, rather
/// than assuming every source of a connector feeds the same file — an assumption that was true
/// while `ohio-courts` held one case and would have quietly stopped being true here.
pub const EDCHOICE_FIXTURE: &str = "crates/regime-diff/fixtures/edchoice-appellate-record.txt";

/// Where the legislative-district crosswalk is written, relative to the repository root.
pub const CROSSWALK_FIXTURE: &str = "crates/project/fixtures/legislative-district-crosswalk.csv";

/// Where the recited transfer orders are written, relative to the repository root.
pub const TRANSFER_FIXTURE: &str = "crates/dispersion/fixtures/territory-transfers.tsv";

/// Where the pre-FY2002 enacted appropriation lines are written, relative to the repository root.
pub const SESSION_LAW_FIXTURE: &str = "crates/project/fixtures/session-law-lines.csv";

/// Where the corrective act's own text is written, relative to the repository root.
///
/// # Why this is a second file and not eight more rows in [`SESSION_LAW_FIXTURE`]
///
/// That fixture is a table, built by finding a department heading and reading the money columns
/// under it. Sub. H.B. 583 of the 134th appropriates almost nothing and amends a great deal, so
/// there is no such table in it to read. What it carries is its own amending title — every
/// Revised Code section it changed, and every uncodified section of H.B. 110 it reopened — and
/// that is prose, which is the form the question about it was asked in.
///
/// Committed whole, for the reason [`GREENBOOK_FIXTURE`] is: it is an education act by its own
/// title, and the retirement, nursing and tax-credit sections sitting beside the school funding
/// ones are a few pages against ninety-four.
pub const CORRECTIONS_FIXTURE: &str = "crates/project/fixtures/hb583-corrections.txt";

/// Where the Ohio slice of the CCD agency directory is written, relative to the repository root.
pub const CCD_DIRECTORY_FIXTURE: &str = "crates/dispersion/fixtures/ccd-lea-directory.csv";

/// Where the MR-81 sponsor panel is written, relative to the repository root.
pub const MR81_FIXTURE: &str = "crates/dispersion/fixtures/mr81-sponsor-panel.csv";

/// Where the building-level report card extract is written, relative to the repository root.
pub const BUILDING_FIXTURE: &str = "crates/dispersion/fixtures/report-card-2425-buildings.csv";

/// Where the identified-schools extract is written, relative to the repository root.
pub const IDENTIFIED_FIXTURE: &str = "crates/dispersion/fixtures/identified-schools-2026.csv";

/// The appropriation-line series: every Department of Education line item, by fiscal year.
pub const APPROPRIATION_FIXTURE: &str = "crates/project/fixtures/appropriation-lines.csv";

/// Where the scholarship programme summary is written, relative to the repository root.
pub const SCHOLARSHIP_FIXTURE: &str = "crates/project/fixtures/scholarship-programs.csv";

/// Where the per-line-item money series is written, relative to the repository root.
pub const CATALOG_FIXTURE: &str = "crates/project/fixtures/catalog-line-items.csv";

/// Where the per-line-item legal basis is written.
pub const CATALOG_BASIS_FIXTURE: &str = "crates/project/fixtures/catalog-line-item-basis.tsv";

/// Every fixture [`crate::rebuild`] writes.
///
/// The list exists so that "a source declares a fixture" and "something regenerates it" can be
/// checked against each other without a populated cache, which CI does not have. It caught the
/// F-33 district panel: 754 KB committed, read by a calculator, declared by a `Source`, and
/// produced by nothing — the digest manifest could not see it, because a digest pins the input
/// and says nothing about whether the derivation from it still exists.
pub const REBUILT: &[&str] = &[
    FY27_FIXTURE,
    PROFILE_FIXTURE,
    GRADE_BANDS_FIXTURE,
    REPORT_CARD_FIXTURE,
    FUNCTIONS_FIXTURE,
    FINANCE_FIXTURE,
    SD1_FIXTURE,
    CASINO_FIXTURE,
    LSC_GREENBOOK_FIXTURE,
    CPI_FIXTURE,
    F33_FIXTURE,
    F33_FY2024_FIXTURE,
    F33_DISTRICTS_FIXTURE,
    F33_OHIO_PANEL_FIXTURE,
    MR81_FIXTURE,
    CCD_DIRECTORY_FIXTURE,
    SESSION_LAW_FIXTURE,
    CORRECTIONS_FIXTURE,
    TRANSFER_FIXTURE,
    BUILDING_FIXTURE,
    IDENTIFIED_FIXTURE,
    CROSSWALK_FIXTURE,
    STATUTE_FIXTURE,
    ENACTED_FIXTURE,
    REDBOOK_FIXTURE,
    GREENBOOK_FIXTURE,
    OPINIONS_FIXTURE,
    EDCHOICE_FIXTURE,
    APPROPRIATION_FIXTURE,
    SCHOLARSHIP_FIXTURE,
    CATALOG_FIXTURE,
    CATALOG_BASIS_FIXTURE,
];

/// Fixtures a [`crate::registry::Source`] declares that [`crate::rebuild`] does not produce.
///
/// **An entry here is a debt with a name on it, not a category of fixture.** The pairing is the
/// path and what would have to be written to retire it.
///
/// The list is empty, and the two entries it held are the reason it exists: the F-33 district
/// panel and the legislative-district crosswalk were both committed, both read by a calculator,
/// both declared by a `Source`, and both produced by nothing. Neither was discoverable — a digest
/// pins the bytes that went in and cannot notice that the derivation from them was never written.
///
/// The test holds this list to both directions — an unlisted gap fails, and so does an entry that
/// has quietly been fixed — so it cannot grow silently or rot after the work is done.
pub const NOT_REGENERATED: &[(&str, &str)] = &[];
