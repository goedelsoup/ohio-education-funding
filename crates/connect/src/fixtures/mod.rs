//! Building the committed CSV fixtures the calculator crates read.
//!
//! Each builder is a pure function from parsed rows to rows of strings, so it tests without a
//! network or a workbook. Only [`write_csv`] and [`write_text`] touch disk.
//!
//! # Why a CSV sits in the middle
//!
//! Retrieval is side-effecting and can fail; calculation must be deterministic and auditable.
//! The fixture is the seam. It is also legible in a diff, which a cached workbook is not — when
//! the department reposts a corrected file, the change shows up as changed numbers in a review
//! rather than as a silently different answer.
//!
//! # One module per publisher's file
//!
//! This was one file of 7,122 lines holding twenty-two unrelated extractors, and the arrangement
//! actively misled. Nine ASCII banners claimed to bound regions they did not: the one reading
//! "Enacted budget acts" covered the enacted-act extractor, four unrelated path constants, the
//! whole legislative-district crosswalk with its Census PL 94-171 readers, the rebuild manifest,
//! and two date parsers. The test module sat in the middle with 4,110 lines of production code
//! after it, and its 663 lines belonged almost entirely to the extractor immediately above.
//!
//! Three modules are shared rather than per-publisher, and they are the ones worth naming:
//!
//! - [`paths`] holds every `*_FIXTURE` constant and the [`REBUILT`] manifest that lists them.
//!   They used to be scattered across 5,500 lines, and `REBUILT` forward-referenced eight
//!   constants defined up to 2,900 lines later, so nothing about it could be checked by reading.
//! - [`delimited`] and [`text`] hold the five readers more than one extractor needs.
//!
//! Everything is re-exported here, so `fixtures::build_fy27_model` and its ninety-odd siblings
//! keep the paths `lib.rs`, `index.rs`, `main.rs`, and `registry.rs` already use.

pub mod appropriations;
pub mod buildings;
pub mod casino;
pub mod catalog;
pub mod ccd;
pub mod cpi;
pub mod crosswalk;
pub mod dates;
pub mod delimited;
pub mod enacted;
pub mod f33;
pub mod finance;
pub mod format;
pub mod fy27;
pub mod grade_bands;
pub mod identified;
pub mod mr81;
pub mod paths;
pub mod report_card;
pub mod scholarship;
pub mod sd1;
pub mod session_laws;
pub mod statute;
pub mod text;
pub mod transfers;
pub mod write;

pub use appropriations::{
    build_appropriations, build_greenbook_appropriations, reconcile, AppropriationBook, Greenbook,
    APPROPRIATION_HEADER,
};
pub use buildings::{build_buildings, BUILDING_HEADER};
pub use casino::{
    build_casino_extract, casino_printed_period, casino_statutory_period, CasinoSheet,
    CASINO_HEADER,
};
pub use catalog::{catalog_items, CatalogItem};
pub use ccd::{build_ccd_directory, DirectoryYear, CCD_DIRECTORY_HEADER};
pub use cpi::build_cpi_extract;
pub use crosswalk::{
    build_legislative_crosswalk, pl_blocks, pl_counts, Crosswalk, CROSSWALK_HEADER,
};
pub use dates::{decided_on, published_on};
pub use enacted::extract_school_funding;
pub use f33::{
    build_f33_districts, build_f33_ohio_panel, build_f33_states, PanelYear, F33_DISTRICTS_HEADER,
    F33_HEADER, F33_OHIO_PANEL_HEADER,
};
pub use finance::{build_finance_extract, FINANCE_HEADER};
pub use format::{clean_name, format_value};
pub use fy27::{build_fy27_model, build_profile_extract, Fy27Sheets, FY27_HEADER, PROFILE_HEADER};
pub use grade_bands::{build_grade_bands, GRADE_BANDS_HEADER};
pub use identified::{build_identified, IDENTIFIED_HEADER};
pub use mr81::{build_mr81, Mr81Layout, Mr81Report, Stream, MR81_HEADER};
pub use paths::{
    APPROPRIATION_FIXTURE, BUILDING_FIXTURE, CASINO_FIXTURE, CATALOG_BASIS_FIXTURE,
    CATALOG_FIXTURE, CCD_DIRECTORY_FIXTURE, CORRECTIONS_FIXTURE, CPI_FIXTURE, CROSSWALK_FIXTURE,
    EDCHOICE_FIXTURE, ENACTED_FIXTURE, F33_DISTRICTS_FIXTURE, F33_FIXTURE, F33_FY2024_FIXTURE,
    F33_OHIO_PANEL_FIXTURE, FINANCE_FIXTURE, FUNCTIONS_FIXTURE, FY27_FIXTURE, GRADE_BANDS_FIXTURE,
    GREENBOOK_FIXTURE, IDENTIFIED_FIXTURE, LSC_GREENBOOK_FIXTURE, MR81_FIXTURE, NOT_REGENERATED,
    OPINIONS_FIXTURE, PROFILE_FIXTURE, REBUILT, REDBOOK_FIXTURE, REPORT_CARD_FIXTURE,
    SCHOLARSHIP_FIXTURE, SD1_FIXTURE, SESSION_LAW_FIXTURE, STATUTE_FIXTURE, TRANSFER_FIXTURE,
};
pub use report_card::{
    build_function_extract, build_report_card_extract, FUNCTIONS_HEADER, REPORT_CARD_HEADER,
};
pub use scholarship::scholarship_programmes;
pub use sd1::{build_sd1_extract, Sd1Year, SD1_HEADER};
pub use session_laws::{build_session_laws, ActText, SESSION_LAW_HEADER};
pub use statute::{build_records, parse_statute, Record, RECORD_MARKER};
pub use transfers::{build_transfers, AuditReport, TRANSFER_HEADER};
pub use write::{write_csv, write_text};
