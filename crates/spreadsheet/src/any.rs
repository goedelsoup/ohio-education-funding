//! Opening a workbook without knowing which format it is.
//!
//! Ohio publishes in both. The FY2027 funding calculator and the report card are XLSX; October
//! enrollment headcount is still the pre-2007 OLE2 format, which shares nothing with XLSX but
//! the family resemblance of the extension. Callers should not have to care, and until this
//! existed the one that did care shelled out to LibreOffice.
//!
//! The format is decided by the file's first bytes rather than by its name, because the
//! extension has been wrong before: the department has published `.xls` files that were XLSX,
//! and `.xls` files that were HTML tables.

use crate::biff::{BiffError, Workbook as LegacyWorkbook};
use crate::xlsx::{Workbook as ModernWorkbook, XlsxError};

/// The two magic numbers worth recognising.
const ZIP: [u8; 2] = [b'P', b'K'];
const OLE2: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// A workbook that could not be opened.
#[derive(Debug, Clone, PartialEq)]
pub enum OpenError {
    /// It is an XLSX and something is wrong with it.
    Xlsx(XlsxError),
    /// It is a legacy workbook and something is wrong with it.
    Biff(BiffError),
    /// The leading bytes match neither format.
    Unrecognised,
}

impl core::fmt::Display for OpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Xlsx(cause) => write!(f, "{cause}"),
            Self::Biff(cause) => write!(f, "{cause}"),
            Self::Unrecognised => write!(
                f,
                "not a workbook: leading bytes are neither a zip nor an OLE2 compound file"
            ),
        }
    }
}

impl std::error::Error for OpenError {}

impl From<XlsxError> for OpenError {
    fn from(cause: XlsxError) -> Self {
        Self::Xlsx(cause)
    }
}

impl From<BiffError> for OpenError {
    fn from(cause: BiffError) -> Self {
        Self::Biff(cause)
    }
}

/// A workbook in either format.
#[derive(Debug, Clone)]
pub enum AnyWorkbook {
    /// Office Open XML, 2007 and later.
    Modern(ModernWorkbook),
    /// BIFF8 in an OLE2 compound file, 1997 to 2003.
    Legacy(LegacyWorkbook),
}

/// Open a workbook, deciding the format from its leading bytes.
///
/// # Errors
///
/// Returns [`OpenError`] if the bytes are neither format, or are that format and malformed.
pub fn open(bytes: Vec<u8>) -> Result<AnyWorkbook, OpenError> {
    if bytes.starts_with(&OLE2) {
        Ok(AnyWorkbook::Legacy(LegacyWorkbook::open(bytes)?))
    } else if bytes.starts_with(&ZIP) {
        Ok(AnyWorkbook::Modern(ModernWorkbook::open(bytes)?))
    } else {
        Err(OpenError::Unrecognised)
    }
}

impl AnyWorkbook {
    /// Sheet names in workbook order.
    #[must_use]
    pub fn sheet_names(&self) -> Vec<&str> {
        match self {
            Self::Modern(book) => book.sheet_names(),
            Self::Legacy(book) => book.sheet_names(),
        }
    }

    /// Every row of a sheet, as strings.
    ///
    /// Both readers pad rows to their widest populated cell and drop empty ones, so the shape is
    /// the same whichever format the department chose that year.
    ///
    /// # Errors
    ///
    /// Returns [`OpenError`] if there is no such sheet or it will not parse.
    pub fn rows(&self, sheet: &str) -> Result<Vec<Vec<String>>, OpenError> {
        match self {
            Self::Modern(book) => Ok(book.rows(sheet)?),
            Self::Legacy(book) => Ok(book.rows(sheet)?),
        }
    }

    /// The first `limit` rows of a sheet, for inspecting a layout without expanding all of it.
    ///
    /// The legacy reader has to shape the whole sheet either way — its cells arrive in no
    /// guaranteed order, so there is no prefix to stop at — and truncating afterwards keeps the
    /// two formats behaving the same from the outside.
    ///
    /// # Errors
    ///
    /// Returns [`OpenError`] if there is no such sheet or it will not parse.
    pub fn rows_limited(&self, sheet: &str, limit: usize) -> Result<Vec<Vec<String>>, OpenError> {
        match self {
            Self::Modern(book) => Ok(book.rows_limited(sheet, limit)?),
            Self::Legacy(book) => {
                let mut rows = book.rows(sheet)?;
                rows.truncate(limit);
                Ok(rows)
            }
        }
    }

    /// Whether this came from the pre-2007 format.
    #[must_use]
    pub const fn is_legacy(&self) -> bool {
        matches!(self, Self::Legacy(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_html_table_saved_as_xls_is_not_a_workbook() {
        // The department has published this. Recognising it by extension would have produced a
        // confusing parse error instead of a clear refusal.
        assert_eq!(
            open(b"<html><table><tr><td>1</td></tr></table></html>".to_vec()).unwrap_err(),
            OpenError::Unrecognised
        );
    }

    #[test]
    fn an_empty_file_is_refused_rather_than_guessed_at() {
        assert_eq!(open(Vec::new()).unwrap_err(), OpenError::Unrecognised);
    }

    #[test]
    fn a_zip_that_is_not_a_workbook_reports_as_an_xlsx_problem() {
        // Dispatch is on the magic number, so a zip gets the zip reader's diagnosis rather than
        // "unrecognised" — which is the more useful message when the file really is an XLSX
        // with something wrong inside it.
        let mut bytes = b"PK\x03\x04".to_vec();
        bytes.resize(200, 0);
        assert!(matches!(open(bytes), Err(OpenError::Xlsx(_))));
    }

    #[test]
    fn an_ole2_file_that_is_not_a_workbook_reports_as_a_biff_problem() {
        let mut bytes = OLE2.to_vec();
        bytes.resize(600, 0);
        assert!(matches!(open(bytes), Err(OpenError::Biff(_))));
    }
}
