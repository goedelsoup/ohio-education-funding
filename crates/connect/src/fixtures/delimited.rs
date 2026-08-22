//! Reading a delimited line, and finding a column by the name its header gives it.
//!
//! Eight extractors read a delimited file and every one of them needs the same two things: fields
//! that survive a quoted comma, and a column index taken from the header rather than counted by
//! hand. Both used to be defined in the middle of the Census F-33 reader, 3,000 lines from most
//! of their callers.

/// Split one line of a delimited file, honouring double-quoted fields.
///
/// The CCD directory quotes any agency name containing a comma, and `LEA_NAME` sits at column 4
/// — ahead of the two columns this reads. Splitting on the delimiter alone gives the right answer
/// for the 2022-23 file, because no quoted comma happens to fall before column 8; that is luck
/// rather than a property of the format, and the kind that fails silently the year an agency is
/// renamed to "Dayton, City of".
pub(super) fn delimited_fields(line: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // An escaped quote inside a quoted field is written as two quotes.
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            c if c == delimiter && !quoted => out.push(core::mem::take(&mut field)),
            c => field.push(c),
        }
    }
    out.push(field);
    out
}

/// Find a column by name, so a layout change is an error rather than a silent misread.
///
/// The survey's columns are located by header rather than by position, unlike
/// [`build_f33_states`]. Both files are published once a year and neither promises a stable
/// layout — `census-f33`'s own note records that the column map is per-era — and a header lookup
/// turns next year's reshuffle into a named failure instead of a fixture full of the wrong
/// numbers.
pub(super) fn column(header: &[String], name: &str, file: &str) -> Result<usize, String> {
    header
        .iter()
        .position(|h| h.trim() == name)
        .ok_or_else(|| format!("{file} has no {name} column; its layout has moved"))
}

/// Locate a column whose header matches any of `names`, ignoring case, spacing and punctuation.
pub(super) fn column_named(header: &[String], names: &[&str]) -> Option<usize> {
    let normalize = |text: &str| -> String {
        text.chars()
            .filter(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_uppercase()
    };
    header
        .iter()
        .position(|cell| names.iter().any(|name| normalize(cell) == normalize(name)))
}
