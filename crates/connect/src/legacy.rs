//! The pre-2007 `.xls` format, which this workspace does not read natively.
//!
//! The department still publishes October enrollment headcount as an OLE2 compound file. That
//! is not a zip and shares nothing with XLSX beyond the extension's family resemblance, so
//! [`spreadsheet`] cannot open it. Reading it properly means an OLE2 sector walker and a BIFF8
//! record parser — perfectly tractable, and the honest completion of this connector.
//!
//! Until then, conversion is delegated to headless LibreOffice. This is called out rather than
//! buried: it is the one source in the registry whose extraction is not reproducible from a
//! checkout alone, and the fixture built from it —
//! `crates/foundation/fixtures/fy24-district-grade-bands.csv` — is correspondingly the one
//! fixture `rebuild` does not regenerate.
//!
//! Converting to `.xlsx` rather than straight to CSV is deliberate: LibreOffice's CSV filter
//! exports only the active sheet, and the enrollment workbook keeps district data on the third
//! of seven.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cache::FetchError;

/// Convert an OLE2 `.xls` to `.xlsx`, returning a path [`spreadsheet`] can open.
///
/// The result is cached beside the input, so a second run is free.
///
/// # Errors
///
/// Returns [`FetchError::Transfer`] if LibreOffice is absent or the conversion produces
/// nothing, with a message naming where a hand-converted file could be dropped instead.
pub fn to_xlsx(path: &Path) -> Result<PathBuf, FetchError> {
    let directory = path.parent().unwrap_or(Path::new(".")).join("converted");
    let converted = directory.join(format!(
        "{}.xlsx",
        path.file_stem().unwrap_or_default().to_string_lossy()
    ));
    if converted.exists() {
        return Ok(converted);
    }
    std::fs::create_dir_all(&directory)?;

    let program = ["soffice", "libreoffice"]
        .into_iter()
        .find(|name| {
            Command::new(name)
                .arg("--version")
                .output()
                .is_ok_and(|out| out.status.success())
        })
        .ok_or_else(|| FetchError::Transfer {
            key: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            detail: format!(
                "this is a pre-2007 .xls and needs LibreOffice to convert. Install it, or \
                 convert by hand and put the result at {}",
                converted.display()
            ),
        })?;

    let output = Command::new(program)
        .args(["--headless", "--convert-to", "xlsx", "--outdir"])
        .arg(&directory)
        .arg(path)
        .output()?;
    if !converted.exists() {
        return Err(FetchError::Transfer {
            key: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            detail: format!(
                "LibreOffice produced no {}: {} {}",
                converted.display(),
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    Ok(converted)
}

/// Whether a file is an OLE2 compound document.
///
/// The magic number is fixed by the format, so this distinguishes a genuine legacy workbook
/// from an `.xls` that is really an XLSX or an HTML table — both of which the department has
/// published under that extension at various times.
#[must_use]
pub fn is_ole2(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_the_ole2_magic_number() {
        assert!(is_ole2(&[
            0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0x00
        ]));
    }

    #[test]
    fn a_zip_is_not_an_ole2_document() {
        // An .xls that is really an XLSX; the extension is not the format.
        assert!(!is_ole2(b"PK\x03\x04rest of a zip"));
        assert!(!is_ole2(b""));
    }

    #[test]
    fn an_html_table_saved_as_xls_is_not_an_ole2_document() {
        assert!(!is_ole2(b"<html><table><tr><td>"));
    }
}
