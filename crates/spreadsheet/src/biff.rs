//! BIFF8 — the record format inside a pre-2007 `.xls` workbook stream.
//!
//! A stream of `(id, length, payload)` records, with every sheet's cells in one substream
//! delimited by `BOF`/`EOF` and located by a `BOUNDSHEET` record near the front. Cells are
//! typed at the record level rather than by an attribute, so `RK`, `NUMBER`, `LABELSST` and
//! `FORMULA` are four different records that all mean "a value at row and column".
//!
//! # Cached formula results, again
//!
//! As in XLSX, a formula cell carries the value of the last recalculation, and that is what is
//! read. A numeric result is in the `FORMULA` record itself; a string result is in a `STRING`
//! record immediately after it. That second case is the one an implementation forgets, and it
//! shows up as blanks in exactly the columns a spreadsheet computes text into.
//!
//! # Two things this format does that nothing else does
//!
//! **`RK` numbers are compressed into 30 bits.** Two flag bits say whether the remainder is an
//! integer or the top half of an IEEE double, and whether to divide by 100. Getting the flags
//! backwards produces numbers a hundred times too large, which looks like a unit error rather
//! than a parse error.
//!
//! **Shared strings straddle record boundaries mid-character.** The string table is longer than
//! one record can hold, so it continues into `CONTINUE` records — and a string cut in half
//! resumes with a *fresh* encoding flag, which may differ from the one it started with. A
//! reader that treats the continuation as a plain byte concatenation gets Chinese characters
//! out of English district names. See the `SharedStrings` decoder below.

use crate::ole2::{Compound, Ole2Error};

// Record identifiers, in the order the specification lists them.
const BOF: u16 = 0x0809;
const EOF_RECORD: u16 = 0x000A;
const BOUNDSHEET: u16 = 0x0085;
const SST: u16 = 0x00FC;
const CONTINUE: u16 = 0x003C;
const LABELSST: u16 = 0x00FD;
const LABEL: u16 = 0x0204;
const RK: u16 = 0x027E;
const MULRK: u16 = 0x00BD;
const NUMBER: u16 = 0x0203;
const FORMULA: u16 = 0x0006;
const STRING: u16 = 0x0207;
const BOOLERR: u16 = 0x0205;

/// A workbook that could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BiffError {
    /// The container is not a compound file, or its workbook stream is missing.
    Container(Ole2Error),
    /// The workbook stream does not begin with a `BOF`.
    NotAWorkbook,
    /// A record declares a length that runs past the end of the stream.
    Truncated,
    /// The file is BIFF5 or older, which stores strings differently throughout.
    UnsupportedVersion {
        /// The version word from the `BOF` record.
        version: u16,
    },
    /// No sheet by that name.
    NoSuchSheet {
        /// The name asked for.
        name: String,
        /// The names the workbook does have.
        available: Vec<String>,
    },
}

impl core::fmt::Display for BiffError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Container(cause) => write!(f, "{cause}"),
            Self::NotAWorkbook => write!(f, "workbook stream does not start with a BOF record"),
            Self::Truncated => write!(f, "a record runs past the end of the stream"),
            Self::UnsupportedVersion { version } => {
                write!(f, "BIFF version {version:#06x}; only BIFF8 is supported")
            }
            Self::NoSuchSheet { name, available } => {
                write!(f, "no sheet {name:?}; have {available:?}")
            }
        }
    }
}

impl std::error::Error for BiffError {}

impl From<Ole2Error> for BiffError {
    fn from(cause: Ole2Error) -> Self {
        Self::Container(cause)
    }
}

/// One record: its identifier and its payload.
struct Record<'a> {
    id: u16,
    body: &'a [u8],
}

/// Walk the records of a stream from a byte offset.
struct Records<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> Iterator for Records<'a> {
    type Item = Record<'a>;

    fn next(&mut self) -> Option<Record<'a>> {
        let id = u16::from_le_bytes(self.data.get(self.at..self.at + 2)?.try_into().ok()?);
        let len =
            u16::from_le_bytes(self.data.get(self.at + 2..self.at + 4)?.try_into().ok()?) as usize;
        let body = self.data.get(self.at + 4..self.at + 4 + len)?;
        self.at += 4 + len;
        Some(Record { id, body })
    }
}

fn records(data: &[u8], from: usize) -> Records<'_> {
    Records { data, at: from }
}

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// Decode an `RK` value.
///
/// Bit 1 says the remaining 30 bits are a signed integer rather than the high half of a double;
/// bit 0 says divide the result by 100. Both are set for a small currency amount, neither for a
/// number that happens to fit the top of a double exactly.
fn rk(value: u32) -> f64 {
    let number = if value & 0x02 != 0 {
        f64::from((value as i32) >> 2)
    } else {
        f64::from_bits(u64::from(value & 0xFFFF_FFFC) << 32)
    };
    if value & 0x01 != 0 {
        number / 100.0
    } else {
        number
    }
}

/// The shared string table, decoded across its `CONTINUE` records.
///
/// The awkward part of the format, and the reason this is a type rather than a function. A
/// string may be split at any byte boundary, including between the two halves of a UTF-16 code
/// unit, and the continuation begins with a **new** encoding flag: a string that started
/// compressed can resume wide. The decoder therefore has to know where each record ended, which
/// is what `boundaries` carries.
struct SharedStrings;

impl SharedStrings {
    /// Read an `SST` record and its continuations into a string table.
    fn read(data: &[u8], sst_start: usize) -> Vec<String> {
        // Concatenate the SST payload and every CONTINUE that follows it, remembering where each
        // joined so the encoding flag can be re-read at the seam.
        let mut body = Vec::new();
        let mut boundaries = Vec::new();
        let mut count = 0u32;
        for (index, record) in records(data, sst_start).enumerate() {
            if index == 0 {
                if record.id != SST {
                    return Vec::new();
                }
                count = u32_at(record.body, 4).unwrap_or(0);
                body.extend_from_slice(&record.body[8.min(record.body.len())..]);
            } else if record.id == CONTINUE {
                boundaries.push(body.len());
                body.extend_from_slice(record.body);
            } else {
                break;
            }
        }

        let mut out = Vec::with_capacity(count as usize);
        let mut at = 0usize;
        for _ in 0..count {
            match Self::string(&body, at, &boundaries) {
                Some((text, next)) => {
                    out.push(text);
                    at = next;
                }
                None => break,
            }
        }
        out
    }

    /// Read one string, honouring the encoding flag reset at every record boundary.
    ///
    /// Returns the text and the offset just past it.
    fn string(body: &[u8], mut at: usize, boundaries: &[usize]) -> Option<(String, usize)> {
        let char_count = u16_at(body, at)? as usize;
        at += 2;
        let flags = *body.get(at)?;
        at += 1;
        let mut wide = flags & 0x01 != 0;
        let rich = flags & 0x08 != 0;
        let extended = flags & 0x04 != 0;

        let run_count = if rich {
            let runs = u16_at(body, at)? as usize;
            at += 2;
            runs
        } else {
            0
        };
        let extended_size = if extended {
            let size = u32_at(body, at)? as usize;
            at += 4;
            size
        } else {
            0
        };

        let mut units: Vec<u16> = Vec::with_capacity(char_count);
        for _ in 0..char_count {
            // A boundary here means the string was cut; the next byte is a fresh flag for the
            // remainder, and it may disagree with the one this string started with.
            if boundaries.contains(&at) {
                wide = *body.get(at)? & 0x01 != 0;
                at += 1;
            }
            if wide {
                units.push(u16_at(body, at)?);
                at += 2;
            } else {
                units.push(u16::from(*body.get(at)?));
                at += 1;
            }
        }
        at += run_count * 4 + extended_size;
        Some((String::from_utf16_lossy(&units), at))
    }
}

/// Read a `XLUnicodeString` with a 16-bit character count, as `LABEL` and `STRING` carry it.
fn unicode_string(body: &[u8], at: usize) -> Option<String> {
    let char_count = u16_at(body, at)? as usize;
    let flags = *body.get(at + 2)?;
    let wide = flags & 0x01 != 0;
    let start = at + 3;
    let units: Vec<u16> = if wide {
        body.get(start..start + char_count * 2)?
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    } else {
        body.get(start..start + char_count)?
            .iter()
            .map(|byte| u16::from(*byte))
            .collect()
    };
    Some(String::from_utf16_lossy(&units))
}

/// Read a `ShortXLUnicodeString`, whose character count is one byte. `BOUNDSHEET` uses it.
fn short_unicode_string(body: &[u8], at: usize) -> Option<String> {
    let char_count = *body.get(at)? as usize;
    let flags = *body.get(at + 1)?;
    let wide = flags & 0x01 != 0;
    let start = at + 2;
    let units: Vec<u16> = if wide {
        body.get(start..start + char_count * 2)?
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect()
    } else {
        body.get(start..start + char_count)?
            .iter()
            .map(|byte| u16::from(*byte))
            .collect()
    };
    Some(String::from_utf16_lossy(&units))
}

/// An opened BIFF8 workbook.
#[derive(Debug, Clone)]
pub struct Workbook {
    stream: Vec<u8>,
    shared: Vec<String>,
    sheets: Vec<(String, usize)>,
}

impl Workbook {
    /// Open a workbook from the bytes of a `.xls` file.
    ///
    /// # Errors
    ///
    /// Returns [`BiffError`] if the file is not a compound file, holds no workbook stream, or is
    /// BIFF5 or older.
    pub fn open(bytes: Vec<u8>) -> Result<Self, BiffError> {
        let compound = Compound::open(bytes)?;
        let stream = compound.read_any(&["Workbook", "Book"])?;

        let first = records(&stream, 0).next().ok_or(BiffError::NotAWorkbook)?;
        if first.id != BOF {
            return Err(BiffError::NotAWorkbook);
        }
        let version = u16_at(first.body, 0).unwrap_or(0);
        if version != 0x0600 {
            return Err(BiffError::UnsupportedVersion { version });
        }

        let mut sheets = Vec::new();
        let mut shared = Vec::new();
        let mut at = 0usize;
        for record in records(&stream, 0) {
            match record.id {
                BOUNDSHEET => {
                    let position = u32_at(record.body, 0).unwrap_or(0) as usize;
                    if let Some(name) = short_unicode_string(record.body, 6) {
                        sheets.push((name, position));
                    }
                }
                SST => {
                    shared = SharedStrings::read(&stream, at);
                }
                // The globals substream ends before the first sheet's; everything wanted from
                // it has been seen by then.
                EOF_RECORD => break,
                _ => {}
            }
            at += 4 + record.body.len();
        }

        Ok(Self {
            stream,
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
    /// Padded to the widest populated cell and with empty rows dropped, matching
    /// [`crate::xlsx::Workbook::rows`] so a caller does not have to know which format it got.
    ///
    /// # Errors
    ///
    /// Returns [`BiffError::NoSuchSheet`] if there is no sheet by that name.
    pub fn rows(&self, sheet: &str) -> Result<Vec<Vec<String>>, BiffError> {
        let start = self
            .sheets
            .iter()
            .find(|(name, _)| name == sheet)
            .map(|(_, position)| *position)
            .ok_or_else(|| BiffError::NoSuchSheet {
                name: sheet.to_string(),
                available: self.sheets.iter().map(|(n, _)| n.clone()).collect(),
            })?;

        // Cells arrive in no guaranteed order, so they are collected by coordinate and shaped
        // afterwards rather than appended.
        let mut cells: Vec<(u16, u16, String)> = Vec::new();
        let mut pending_formula: Option<(u16, u16)> = None;

        for record in records(&self.stream, start).skip(1) {
            match record.id {
                EOF_RECORD => break,
                RK => {
                    if let (Some(row), Some(column), Some(value)) = (
                        u16_at(record.body, 0),
                        u16_at(record.body, 2),
                        u32_at(record.body, 6),
                    ) {
                        cells.push((row, column, format_number(rk(value))));
                    }
                }
                MULRK => {
                    // One record, a run of columns: `(row, first)` then a pair per column, then
                    // the last column index.
                    let (Some(row), Some(first)) = (u16_at(record.body, 0), u16_at(record.body, 2))
                    else {
                        continue;
                    };
                    let mut at = 4usize;
                    let mut column = first;
                    while at + 6 <= record.body.len().saturating_sub(2) {
                        if let Some(value) = u32_at(record.body, at + 2) {
                            cells.push((row, column, format_number(rk(value))));
                        }
                        at += 6;
                        column += 1;
                    }
                }
                NUMBER => {
                    if let (Some(row), Some(column)) =
                        (u16_at(record.body, 0), u16_at(record.body, 2))
                    {
                        if let Some(bytes) = record.body.get(6..14) {
                            let value = f64::from_le_bytes(bytes.try_into().unwrap_or_default());
                            cells.push((row, column, format_number(value)));
                        }
                    }
                }
                LABELSST => {
                    if let (Some(row), Some(column), Some(index)) = (
                        u16_at(record.body, 0),
                        u16_at(record.body, 2),
                        u32_at(record.body, 6),
                    ) {
                        if let Some(text) = self.shared.get(index as usize) {
                            cells.push((row, column, text.clone()));
                        }
                    }
                }
                LABEL => {
                    if let (Some(row), Some(column), Some(text)) = (
                        u16_at(record.body, 0),
                        u16_at(record.body, 2),
                        unicode_string(record.body, 6),
                    ) {
                        cells.push((row, column, text));
                    }
                }
                FORMULA => {
                    let (Some(row), Some(column)) =
                        (u16_at(record.body, 0), u16_at(record.body, 2))
                    else {
                        continue;
                    };
                    // A numeric result sits in the record. The special pattern `xx xx xx xx xx
                    // xx FF FF` means the value is somewhere else — a following STRING record,
                    // an error, or a boolean.
                    let Some(bytes) = record.body.get(6..14) else {
                        continue;
                    };
                    if bytes[6] == 0xFF && bytes[7] == 0xFF {
                        if bytes[0] == 0x00 {
                            pending_formula = Some((row, column));
                        }
                    } else {
                        let value = f64::from_le_bytes(bytes.try_into().unwrap_or_default());
                        cells.push((row, column, format_number(value)));
                    }
                }
                STRING => {
                    // The other half of a text-valued formula. Forgetting this leaves blanks in
                    // exactly the columns a spreadsheet computes text into.
                    if let (Some((row, column)), Some(text)) =
                        (pending_formula.take(), unicode_string(record.body, 0))
                    {
                        cells.push((row, column, text));
                    }
                }
                BOOLERR => {
                    if let (Some(row), Some(column), Some(&value), Some(&is_error)) = (
                        u16_at(record.body, 0),
                        u16_at(record.body, 2),
                        record.body.get(6),
                        record.body.get(7),
                    ) {
                        let text = if is_error == 1 {
                            error_text(value).to_string()
                        } else {
                            u8::from(value != 0).to_string()
                        };
                        cells.push((row, column, text));
                    }
                }
                _ => {}
            }
        }

        Ok(shape(cells))
    }
}

/// The department writes these markers in its published files; a reader that turned them into
/// blanks would lose the distinction between "no data" and "a formula that failed".
fn error_text(code: u8) -> &'static str {
    match code {
        0x00 => "#NULL!",
        0x07 => "#DIV/0!",
        0x0F => "#VALUE!",
        0x17 => "#REF!",
        0x1D => "#NAME?",
        0x24 => "#NUM!",
        0x2A => "#N/A",
        _ => "#ERR",
    }
}

/// Render a number the way a spreadsheet's General format would, so the output matches what a
/// conversion through Excel or LibreOffice produces.
fn format_number(value: f64) -> String {
    if value == value.trunc() && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        let mut text = format!("{value}");
        if text.contains('e') {
            text = format!("{value:.10}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
        }
        text
    }
}

/// Turn scattered `(row, column, value)` triples into padded rows with the empty ones dropped.
fn shape(mut cells: Vec<(u16, u16, String)>) -> Vec<Vec<String>> {
    if cells.is_empty() {
        return Vec::new();
    }
    cells.sort_by_key(|(row, column, _)| (*row, *column));
    let mut out = Vec::new();
    let mut current_row = cells[0].0;
    let mut row: Vec<String> = Vec::new();
    for (index, (row_number, column, value)) in cells.into_iter().enumerate() {
        if index > 0 && row_number != current_row {
            out.push(std::mem::take(&mut row));
            current_row = row_number;
        }
        let column = column as usize;
        if row.len() <= column {
            row.resize(column + 1, String::new());
        }
        row[column] = value;
    }
    out.push(row);
    out.retain(|row| row.iter().any(|cell| !cell.is_empty()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rk_decodes_the_four_flag_combinations() {
        // Integer, no division: 3 << 2 with bit 1 set.
        assert!((rk((3 << 2) | 0x02) - 3.0).abs() < 1e-12);
        // Integer, divided by 100.
        assert!((rk((325 << 2) | 0x03) - 3.25).abs() < 1e-12);
        // The top half of an IEEE double: 1.5 is 0x3FF8000000000000.
        assert!((rk(0x3FF8_0000) - 1.5).abs() < 1e-12);
        // The same, divided by 100.
        assert!((rk(0x3FF8_0001) - 0.015).abs() < 1e-12);
    }

    #[test]
    fn rk_handles_a_negative_integer() {
        assert!((rk(((-7i32 << 2) as u32) | 0x02) - -7.0).abs() < 1e-12);
    }

    #[test]
    fn a_compressed_shared_string_reads_as_latin1() {
        // cch = 5, grbit = 0 (compressed), then five bytes.
        let mut body = vec![5, 0, 0];
        body.extend_from_slice(b"Akron");
        let (text, at) = SharedStrings::string(&body, 0, &[]).unwrap();
        assert_eq!(text, "Akron");
        assert_eq!(at, body.len());
    }

    #[test]
    fn a_wide_shared_string_reads_as_utf16() {
        let mut body = vec![3, 0, 1];
        for unit in "Ada".encode_utf16() {
            body.extend_from_slice(&unit.to_le_bytes());
        }
        assert_eq!(SharedStrings::string(&body, 0, &[]).unwrap().0, "Ada");
    }

    #[test]
    fn a_string_split_across_a_boundary_re_reads_its_encoding_flag() {
        // The case that produces Chinese characters out of English district names when it is
        // missed: "Ada" starts compressed, is cut after one character, and resumes wide.
        let mut body = vec![3, 0, 0];
        body.extend_from_slice(b"A");
        let boundary = body.len();
        body.push(0x01); // the continuation's own flag: wide
        for unit in "da".encode_utf16() {
            body.extend_from_slice(&unit.to_le_bytes());
        }
        let (text, _) = SharedStrings::string(&body, 0, &[boundary]).unwrap();
        assert_eq!(text, "Ada");
    }

    #[test]
    fn a_boundary_that_is_not_inside_a_string_changes_nothing() {
        let mut body = vec![5, 0, 0];
        body.extend_from_slice(b"Akron");
        // A boundary past the end of this string must not be consumed as a flag byte.
        let (text, _) = SharedStrings::string(&body, 0, &[body.len()]).unwrap();
        assert_eq!(text, "Akron");
    }

    #[test]
    fn a_rich_string_skips_its_formatting_runs() {
        // grbit bit 3 set, two runs of four bytes each after the characters.
        let mut body = vec![4, 0, 0x08, 2, 0];
        body.extend_from_slice(b"Ohio");
        body.extend_from_slice(&[0u8; 8]);
        let (text, at) = SharedStrings::string(&body, 0, &[]).unwrap();
        assert_eq!(text, "Ohio");
        assert_eq!(at, body.len(), "the runs must be stepped over, not read");
    }

    #[test]
    fn shaping_pads_rows_and_drops_empty_ones() {
        let cells = vec![
            (0, 0, "a".to_string()),
            (0, 3, "d".to_string()),
            (2, 1, "x".to_string()),
        ];
        let rows = shape(cells);
        assert_eq!(rows.len(), 2, "the empty row 1 is dropped");
        assert_eq!(rows[0], ["a", "", "", "d"]);
        assert_eq!(rows[1], ["", "x"]);
    }

    #[test]
    fn numbers_render_the_way_a_general_format_would() {
        assert_eq!(format_number(3.0), "3");
        assert_eq!(format_number(847.823), "847.823");
        assert_eq!(format_number(-0.5), "-0.5");
    }

    #[test]
    fn error_codes_survive_as_the_markers_the_department_writes() {
        assert_eq!(error_text(0x2A), "#N/A");
        assert_eq!(error_text(0x07), "#DIV/0!");
    }

    #[test]
    fn a_file_that_is_not_compound_is_refused() {
        assert!(matches!(
            Workbook::open(b"not an xls".to_vec()),
            Err(BiffError::Container(_))
        ));
    }
}
