//! Retrieval and extraction for the Ohio education funding corpus.
//!
//! The side-effecting half of the domain computer. It fetches the department's publications,
//! reads them with [`spreadsheet`], and writes the CSV fixtures the calculator crates compute
//! against.
//!
//! # The connector/calculator split
//!
//! [PHASES.md](../../.yidam/.vendor/prelude/PHASES.md) draws the line: connectors are
//! side-effecting and may fail, calculators are pure and deterministic. Everything here can
//! fail — a URL moves, a workbook changes shape, the network is not there. Nothing here
//! computes a funding figure. That is why an extraction commit and an assessment commit have
//! different meanings in the log, and why they are not mixed.
//!
//! # Refresh and verification check each other
//!
//! Extraction can be re-run freely *because* the findings are pinned downstream. A source
//! revision that moves the guarantee count, the millage floor count, or a scenario result fails
//! a test in [`foundation`](../foundation/) or [`dispersion`](../dispersion/) rather than
//! passing silently into the corpus. The digest manifest in [`cache`] closes the other
//! direction: it records exactly which published file a fixture was built from, so a changed
//! answer can always be traced to a changed source.
//!
//! ```text
//! edfund-connect list                 what is retrievable, and how far each connector got
//! edfund-connect fetch <source>       download one source into .cache/sources
//! edfund-connect rebuild              regenerate the committed fixtures
//! edfund-connect verify               check cached sources against the committed digests
//! edfund-connect cpi                  check the deflator series against the Bureau's file
//! ```

#![forbid(unsafe_code)]

pub mod cache;
pub mod conventions;
pub mod cpi;
pub mod fixtures;
pub mod legacy;
pub mod registry;
pub mod sha256;

use std::path::Path;

use cache::FetchError;
use registry::Source;
use spreadsheet::Workbook;

pub use fixtures::{CPI_FIXTURE, FY27_FIXTURE, PROFILE_FIXTURE};

pub use registry::{connector, source, Connector, Format, Status, CONNECTORS};

/// What happened to one fixture during a rebuild.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rebuilt {
    /// Regenerated from a cached source.
    Written {
        /// Path relative to the repository root.
        path: String,
        /// How many data rows were written.
        rows: usize,
    },
    /// Left alone, because its source is not available.
    ///
    /// Reported rather than silently omitted: a rebuild that quietly regenerates two of three
    /// fixtures reads as a full refresh in a commit message, and the third then carries an
    /// older vintage than the others without saying so.
    Skipped {
        /// Path relative to the repository root.
        path: String,
        /// Why it was not rebuilt.
        reason: String,
    },
}

/// Something went wrong rebuilding a fixture.
#[derive(Debug)]
pub enum RebuildError {
    /// A source could not be read.
    Source(FetchError),
    /// A workbook could not be parsed.
    Workbook(spreadsheet::XlsxError),
    /// A fixture could not be written.
    Io(std::io::Error),
}

impl core::fmt::Display for RebuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Source(cause) => write!(f, "{cause}"),
            Self::Workbook(cause) => write!(f, "{cause}"),
            Self::Io(cause) => write!(f, "{cause}"),
        }
    }
}

impl std::error::Error for RebuildError {}

impl From<FetchError> for RebuildError {
    fn from(cause: FetchError) -> Self {
        Self::Source(cause)
    }
}

impl From<spreadsheet::XlsxError> for RebuildError {
    fn from(cause: spreadsheet::XlsxError) -> Self {
        Self::Workbook(cause)
    }
}

impl From<std::io::Error> for RebuildError {
    fn from(cause: std::io::Error) -> Self {
        Self::Io(cause)
    }
}

/// Open a cached source as a workbook, converting a legacy `.xls` first if need be.
///
/// # Errors
///
/// Returns [`RebuildError`] if the source is not cached, cannot be converted, or is not a
/// readable workbook.
pub fn open_workbook(root: &Path, source: &Source) -> Result<Workbook, RebuildError> {
    let path = cache::cached_path(root, source);
    if !path.exists() {
        return Err(RebuildError::Source(FetchError::NotCached {
            key: source.key.to_string(),
            path,
        }));
    }
    let path = if source.format == Format::LegacyXls {
        legacy::to_xlsx(&path)?
    } else {
        path
    };
    Ok(Workbook::open(std::fs::read(path)?)?)
}

/// Rebuild every committed fixture from the cached sources.
///
/// Reads only what is already in the cache — refreshing is a separate, explicit act, so that
/// regenerating a fixture never silently pulls a revised publication.
///
/// # Errors
///
/// Returns [`RebuildError`] if a source is missing from the cache or will not parse.
pub fn rebuild(root: &Path) -> Result<Vec<Rebuilt>, RebuildError> {
    let fy27 = source("fy27-calculator").expect("registered").1;
    let profile = source("cupp-fy24").expect("registered").1;

    let fy27_book = open_workbook(root, fy27)?;
    let profile_book = open_workbook(root, profile)?;
    let profile_rows = profile_book.rows("District Data")?;

    let model = fixtures::build_fy27_model(
        &fy27_book.rows("Base_Cost")?,
        &fy27_book.rows("Summary_SFPR")?,
        &fy27_book.rows("ADM Data")?,
        &profile_rows,
    );
    let extract = fixtures::build_profile_extract(&profile_rows);

    let mut out = vec![
        Rebuilt::Written {
            path: fixtures::FY27_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::FY27_FIXTURE),
                fixtures::FY27_HEADER,
                &model,
            )?,
        },
        Rebuilt::Written {
            path: fixtures::PROFILE_FIXTURE.to_string(),
            rows: fixtures::write_csv(
                &root.join(fixtures::PROFILE_FIXTURE),
                fixtures::PROFILE_HEADER,
                &extract,
            )?,
        },
    ];

    let cpi_source = source("cpi-u-all-items").expect("registered").1;
    out.push(match cache::read_cached(root, cpi_source) {
        Ok(bytes) => {
            let extract = fixtures::build_cpi_extract(
                &String::from_utf8_lossy(&bytes),
                cpi::ALL_ITEMS_NSA,
                cpi::JUNE,
            );
            let path = root.join(fixtures::CPI_FIXTURE);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let rows = extract.lines().count().saturating_sub(1);
            std::fs::write(&path, extract)?;
            Rebuilt::Written {
                path: fixtures::CPI_FIXTURE.to_string(),
                rows,
            }
        }
        Err(cause) => Rebuilt::Skipped {
            path: fixtures::CPI_FIXTURE.to_string(),
            reason: cause.to_string(),
        },
    });

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixtures_rebuild_names_point_at_files_that_exist() {
        let root = cache::repository_root();
        for path in [fixtures::FY27_FIXTURE, fixtures::PROFILE_FIXTURE] {
            assert!(root.join(path).exists(), "{path} is not committed");
        }
    }

    #[test]
    fn every_wired_connector_has_a_committed_digest() {
        // A source the fixtures were built from but that nothing pins is a provenance hole.
        let root = cache::repository_root();
        let manifest = std::fs::read_to_string(root.join(cache::MANIFEST)).unwrap_or_default();
        let pinned = cache::parse_manifest(&manifest);
        for connector in CONNECTORS {
            if connector.status != Status::Wired {
                continue;
            }
            for source in connector.sources {
                assert!(
                    pinned.iter().any(|d| d.key == source.key),
                    "{} is wired but not pinned in {}",
                    source.key,
                    cache::MANIFEST
                );
            }
        }
    }
}
