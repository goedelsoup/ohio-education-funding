//! Reading XLSX workbooks.
//!
//! # Cached formula results are the point
//!
//! A cell holding a formula stores both the formula in `<f>` and the value the last
//! recalculation produced in `<v>`. This reader takes `<v>` and ignores `<f>`, which means the
//! department's own funding calculator — a workbook that is almost entirely formulas — reads as
//! data without a spreadsheet engine anywhere in the pipeline. That is what makes the FY2027
//! model a *source* rather than a document.
//!
//! The trade is that the values are whatever the department last recalculated and saved. They
//! are the published figures, which is what the corpus wants, but they are not recomputed here
//! and a workbook saved with stale calculation would read stale.

use crate::xml::{Event, Reader};
use crate::zip::{Archive, ZipError};

const SHARED_STRINGS: &str = "xl/sharedStrings.xml";
const WORKBOOK: &str = "xl/workbook.xml";
const WORKBOOK_RELS: &str = "xl/_rels/workbook.xml.rels";

/// A workbook that could not be read.
#[derive(Debug, Clone, PartialEq)]
pub enum XlsxError {
    /// The container is not a readable zip archive.
    Container(ZipError),
    /// A required part is missing — the file is a zip, but not a workbook.
    MissingPart {
        /// The part that was expected.
        part: String,
    },
    /// No sheet by that name.
    NoSuchSheet {
        /// The name that was asked for.
        name: String,
        /// The names the workbook does have, so the caller can see the spelling.
        available: Vec<String>,
    },
}

impl core::fmt::Display for XlsxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Container(cause) => write!(f, "{cause}"),
            Self::MissingPart { part } => write!(f, "not a workbook: no {part}"),
            Self::NoSuchSheet { name, available } => {
                write!(f, "no sheet {name:?}; have {available:?}")
            }
        }
    }
}

impl std::error::Error for XlsxError {}

impl From<ZipError> for XlsxError {
    fn from(cause: ZipError) -> Self {
        Self::Container(cause)
    }
}

/// Zero-based column index from a cell reference such as `BA12`.
///
/// Returns `None` if the reference does not begin with column letters.
#[must_use]
pub fn column_index(reference: &str) -> Option<usize> {
    let mut index = 0usize;
    let mut letters = 0;
    for byte in reference.bytes() {
        if byte.is_ascii_uppercase() {
            index = index * 26 + (byte - b'A' + 1) as usize;
            letters += 1;
        } else {
            break;
        }
    }
    (letters > 0).then(|| index - 1)
}

/// An opened workbook, with its shared-string table and sheet list resolved.
#[derive(Debug, Clone)]
pub struct Workbook {
    archive: Archive,
    shared: Vec<String>,
    sheets: Vec<(String, String)>,
}

impl Workbook {
    /// Open a workbook from its bytes.
    ///
    /// # Errors
    ///
    /// Returns [`XlsxError`] if the bytes are not a zip archive or the archive has no workbook
    /// part.
    pub fn open(bytes: Vec<u8>) -> Result<Self, XlsxError> {
        let archive = Archive::open(bytes)?;
        if !archive.contains(WORKBOOK) {
            return Err(XlsxError::MissingPart {
                part: WORKBOOK.into(),
            });
        }

        let shared = if archive.contains(SHARED_STRINGS) {
            read_shared_strings(&archive.read_to_string(SHARED_STRINGS)?)
        } else {
            Vec::new()
        };

        let relationships = if archive.contains(WORKBOOK_RELS) {
            read_relationships(&archive.read_to_string(WORKBOOK_RELS)?)
        } else {
            Vec::new()
        };
        let sheets = read_sheet_list(&archive.read_to_string(WORKBOOK)?, &relationships);

        Ok(Self {
            archive,
            shared,
            sheets,
        })
    }

    /// Sheet names in workbook order.
    #[must_use]
    pub fn sheet_names(&self) -> Vec<&str> {
        self.sheets.iter().map(|(name, _)| name.as_str()).collect()
    }

    /// Every row of a sheet, as strings.
    ///
    /// Rows are padded to their widest populated cell and empty rows are dropped — the
    /// department's workbooks use blank rows as visual spacing, and carrying them would make
    /// every consumer filter them out again.
    ///
    /// # Errors
    ///
    /// Returns [`XlsxError`] if there is no such sheet or its part will not decompress.
    pub fn rows(&self, sheet: &str) -> Result<Vec<Vec<String>>, XlsxError> {
        self.rows_limited(sheet, usize::MAX)
    }

    /// The first `limit` rows of a sheet. Useful for inspecting a layout without expanding a
    /// 40 MB part.
    ///
    /// # Errors
    ///
    /// Returns [`XlsxError`] as [`Workbook::rows`] does.
    pub fn rows_limited(&self, sheet: &str, limit: usize) -> Result<Vec<Vec<String>>, XlsxError> {
        let part = self
            .sheets
            .iter()
            .find(|(name, _)| name == sheet)
            .map(|(_, part)| part.as_str())
            .ok_or_else(|| XlsxError::NoSuchSheet {
                name: sheet.to_string(),
                available: self.sheets.iter().map(|(n, _)| n.clone()).collect(),
            })?;
        let xml = self.archive.read_to_string(part)?;
        Ok(read_rows(&xml, &self.shared, limit))
    }
}

fn read_shared_strings(xml: &str) -> Vec<String> {
    let mut reader = Reader::new(xml);
    let mut out = Vec::new();
    while let Some(event) = reader.next_event() {
        match event {
            Event::Open(tag) if tag.name == "si" => out.push(reader.text_until_close("si")),
            Event::Empty(tag) if tag.name == "si" => out.push(String::new()),
            _ => {}
        }
    }
    out
}

fn read_relationships(xml: &str) -> Vec<(String, String)> {
    let mut reader = Reader::new(xml);
    let mut out = Vec::new();
    while let Some(event) = reader.next_event() {
        let (Event::Open(tag) | Event::Empty(tag)) = event else {
            continue;
        };
        if tag.name != "Relationship" {
            continue;
        }
        if let (Some(id), Some(target)) = (tag.attr("Id"), tag.attr("Target")) {
            out.push((id.into_owned(), target.into_owned()));
        }
    }
    out
}

fn read_sheet_list(xml: &str, relationships: &[(String, String)]) -> Vec<(String, String)> {
    let mut reader = Reader::new(xml);
    let mut out = Vec::new();
    while let Some(event) = reader.next_event() {
        let (Event::Open(tag) | Event::Empty(tag)) = event else {
            continue;
        };
        if tag.name != "sheet" {
            continue;
        }
        let (Some(name), Some(id)) = (tag.attr("name"), tag.attr("id")) else {
            continue;
        };
        let Some((_, target)) = relationships.iter().find(|(rid, _)| *rid == id) else {
            continue;
        };
        // A relationship target may be written relative to `xl/` or absolute from the archive
        // root; both name the same part.
        let trimmed = target.trim_start_matches('/');
        let part = format!("xl/{}", trimmed.strip_prefix("xl/").unwrap_or(trimmed));
        out.push((name.into_owned(), part));
    }
    out
}

fn read_rows(xml: &str, shared: &[String], limit: usize) -> Vec<Vec<String>> {
    let mut reader = Reader::new(xml);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut cells: Vec<(usize, String)> = Vec::new();
    let mut next_column = 0usize;

    while let Some(event) = reader.next_event() {
        match event {
            Event::Open(tag) if tag.name == "row" => {
                cells.clear();
                next_column = 0;
            }
            Event::Close("row") => {
                if let Some(&(widest, _)) = cells.iter().max_by_key(|(column, _)| *column) {
                    let mut row = vec![String::new(); widest + 1];
                    for (column, value) in cells.drain(..) {
                        row[column] = value;
                    }
                    rows.push(row);
                    if rows.len() >= limit {
                        return rows;
                    }
                }
                cells.clear();
            }
            // A self-closing `<c/>` is an empty cell. It has to be matched before the opening
            // form, because scanning it for a value would run past its (absent) close tag and
            // swallow the rest of the row.
            Event::Empty(tag) if tag.name == "c" => {
                let column = tag
                    .attr("r")
                    .and_then(|reference| column_index(&reference))
                    .unwrap_or(next_column);
                next_column = column + 1;
            }
            Event::Open(tag) if tag.name == "c" => {
                let column = tag
                    .attr("r")
                    .and_then(|reference| column_index(&reference))
                    .unwrap_or(next_column);
                next_column = column + 1;
                let kind = tag.attr("t").unwrap_or_default().into_owned();
                let raw = read_cell_value(&mut reader);
                if let Some(value) = resolve(raw, &kind, shared) {
                    cells.push((column, value));
                }
            }
            _ => {}
        }
    }
    rows
}

/// Read the value out of a `<c>` element, stopping at its close.
///
/// `<f>` is skipped: the formula is not the datum, the cached result is.
fn read_cell_value(reader: &mut Reader<'_>) -> Option<String> {
    let mut value = None;
    while let Some(event) = reader.next_event() {
        match event {
            Event::Open(tag) if tag.name == "v" => value = Some(reader.text_until_close("v")),
            Event::Open(tag) if tag.name == "is" => value = Some(reader.text_until_close("is")),
            Event::Close("c") => break,
            // Only reachable on malformed input: a well-formed `<c>` closes before its row
            // does, and a self-closing `<c/>` is handled without ever calling this.
            Event::Close("row") => break,
            _ => {}
        }
    }
    value
}

/// Turn a raw cell body into its displayed value.
///
/// A shared-string index that does not resolve is treated as no value rather than as an empty
/// one, because the two are different claims and a broken index means a broken file.
fn resolve(raw: Option<String>, kind: &str, shared: &[String]) -> Option<String> {
    let raw = raw?;
    if kind == "s" {
        return raw
            .trim()
            .parse::<usize>()
            .ok()
            .and_then(|index| shared.get(index).cloned());
    }
    Some(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zip::crc32;

    const RELS: &str = r#"<?xml version="1.0"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Target="/xl/worksheets/sheet2.xml"/>
</Relationships>"#;

    const BOOK: &str = r#"<?xml version="1.0"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
 xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>
<sheet name="Notes" sheetId="1" r:id="rId1"/>
<sheet name="District Data" sheetId="2" r:id="rId2"/>
</sheets></workbook>"#;

    const SHARED: &str = r#"<?xml version="1.0"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<si><t>IRN</t></si><si><t>District</t></si><si><t>State of Ohio</t></si>
</sst>"#;

    const NOTES: &str = r#"<worksheet><sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>a note</t></is></c></row>
</sheetData></worksheet>"#;

    // Row 2 is an empty spacer. Row 3's D cell holds a cached formula result. Row 5 is the
    // statewide aggregate the department ships in its summary sheets.
    const DATA: &str = r#"<worksheet><sheetData>
<row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row>
<row r="2"/>
<row r="3"><c r="A3"><v>043786</v></c><c r="B3" t="inlineStr"><is><t>Cleveland</t></is></c>
  <c r="C3"/><c r="D3"><f>SUM(X1:X9)</f><v>184903.95</v></c></row>
<row r="4"><c r="A4"><v>049056</v></c><c r="B4" t="inlineStr"><is><t>Northern</t></is></c>
  <c r="D4" t="str"><v>#N/A</v></c></row>
<row r="5"><c r="A5"><v>999999</v></c><c r="B5" t="s"><v>2</v></c></row>
</sheetData></worksheet>"#;

    fn stored_archive(members: &[(&str, &str)]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut directory = Vec::new();
        for (name, body) in members {
            let offset = out.len() as u32;
            let body = body.as_bytes();
            let crc = crc32(body);
            out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
            out.extend_from_slice(&[20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&(body.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(body);

            directory.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
            directory.extend_from_slice(&[20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            directory.extend_from_slice(&crc.to_le_bytes());
            directory.extend_from_slice(&(body.len() as u32).to_le_bytes());
            directory.extend_from_slice(&(body.len() as u32).to_le_bytes());
            directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
            directory.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            directory.extend_from_slice(&offset.to_le_bytes());
            directory.extend_from_slice(name.as_bytes());
        }
        let directory_offset = out.len() as u32;
        let directory_size = directory.len() as u32;
        out.extend_from_slice(&directory);
        out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(members.len() as u16).to_le_bytes());
        out.extend_from_slice(&(members.len() as u16).to_le_bytes());
        out.extend_from_slice(&directory_size.to_le_bytes());
        out.extend_from_slice(&directory_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    fn workbook() -> Workbook {
        Workbook::open(stored_archive(&[
            (WORKBOOK_RELS, RELS),
            (WORKBOOK, BOOK),
            (SHARED_STRINGS, SHARED),
            ("xl/worksheets/sheet1.xml", NOTES),
            ("xl/worksheets/sheet2.xml", DATA),
        ]))
        .unwrap()
    }

    #[test]
    fn single_and_double_letter_columns() {
        assert_eq!(column_index("A1"), Some(0));
        assert_eq!(column_index("D3"), Some(3));
        assert_eq!(column_index("Z9"), Some(25));
        assert_eq!(column_index("AA1"), Some(26));
        assert_eq!(column_index("BA12"), Some(52));
        assert_eq!(column_index("123"), None);
    }

    #[test]
    fn lists_sheets_in_workbook_order() {
        assert_eq!(workbook().sheet_names(), vec!["Notes", "District Data"]);
    }

    #[test]
    fn resolves_shared_strings_and_inline_strings() {
        let book = workbook();
        let rows = book.rows("District Data").unwrap();
        assert_eq!(rows[0][0], "IRN");
        assert_eq!(rows[0][1], "District");
        assert_eq!(rows[1][1], "Cleveland");
    }

    #[test]
    fn takes_the_cached_result_of_a_formula_cell() {
        // The point of the whole reader: <f> is skipped, <v> is the datum.
        let book = workbook();
        assert_eq!(book.rows("District Data").unwrap()[1][3], "184903.95");
    }

    #[test]
    fn skips_empty_spacer_rows() {
        // Header, two districts, one aggregate — the blank row 2 is dropped.
        assert_eq!(workbook().rows("District Data").unwrap().len(), 4);
    }

    #[test]
    fn pads_short_rows_to_their_widest_populated_cell() {
        let book = workbook();
        let row = &book.rows("District Data").unwrap()[1];
        assert_eq!(row.len(), 4);
        assert_eq!(row[2], "", "an empty <c/> holds no value");
    }

    #[test]
    fn a_relationship_target_may_be_relative_or_absolute() {
        // sheet1 is "worksheets/sheet1.xml"; sheet2 is "/xl/worksheets/sheet2.xml".
        let book = workbook();
        assert!(book.rows("Notes").is_ok());
        assert!(book.rows("District Data").is_ok());
    }

    #[test]
    fn a_limit_stops_before_expanding_the_rest() {
        assert_eq!(
            workbook().rows_limited("District Data", 2).unwrap().len(),
            2
        );
    }

    #[test]
    fn an_unknown_sheet_names_the_available_ones() {
        let err = workbook().rows("Nope").unwrap_err();
        let XlsxError::NoSuchSheet { available, .. } = &err else {
            panic!("expected NoSuchSheet, got {err}")
        };
        assert!(available.contains(&"District Data".to_string()));
    }

    #[test]
    fn error_text_carries_the_department_error_marker_through_unchanged() {
        // "#N/A" is a value the department writes; interpreting it is the caller's job.
        let book = workbook();
        assert_eq!(book.rows("District Data").unwrap()[2][3], "#N/A");
    }

    #[test]
    fn a_zip_that_is_not_a_workbook_says_so() {
        let bytes = stored_archive(&[("hello.txt", "hi")]);
        assert!(matches!(
            Workbook::open(bytes).unwrap_err(),
            XlsxError::MissingPart { .. }
        ));
    }

    #[test]
    fn an_unresolvable_shared_string_index_is_missing_not_empty() {
        assert_eq!(resolve(Some("7".into()), "s", &["a".to_string()]), None);
        assert_eq!(
            resolve(Some("0".into()), "s", &["a".to_string()]),
            Some("a".to_string())
        );
    }
}
