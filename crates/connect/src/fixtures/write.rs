//! The only two functions here that touch disk.
//!
//! Everything else in [`super`] is a pure function from parsed rows to rows of strings. Keeping
//! the writes in one small module is what makes that claim checkable rather than asserted: a
//! builder that grew a side effect would have to import from here to do it.

use std::fs;
use std::io;
use std::path::Path;

/// Write a fixture with LF endings, so git sees no spurious churn on a rebuild.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] if the directory cannot be created or the file written.
pub fn write_csv(path: &Path, header: &[&str], rows: &[Vec<String>]) -> io::Result<usize> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = String::with_capacity(rows.len() * 160 + 256);
    out.push_str(&header.join(","));
    out.push('\n');
    for row in rows {
        // This writer does not quote, and every consumer of these fixtures splits on the comma.
        // A field carrying one shifts that row's remaining columns and nothing downstream can
        // tell — the MR-81 panel shipped 99 such rows on its first build, because Ohio has
        // sponsors called "Holy Trinity, Swanton Ele Sch" and "Edge Academy, The". Failing here
        // is the only place the problem is visible; by the time a calculator reads the file, the
        // corruption looks like data.
        assert!(
            !row.iter().any(|field| field.contains(',')),
            "a field contains a comma and this writer does not quote: {row:?}"
        );
        // The other half of the same invariant: every reader indexes by position, so a row
        // that does not have the header's number of cells is one whose columns have moved.
        // `edfund_core::csv::rows` asserts this on the way in as well — this one names the
        // builder that produced the bad row, which the reader cannot.
        assert_eq!(
            row.len(),
            header.len(),
            "a row has {} cells where the header names {}: {row:?}",
            row.len(),
            header.len()
        );
        out.push_str(&row.join(","));
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(rows.len())
}

/// Write a text fixture, creating its directory.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] if the file cannot be written.
pub fn write_text(path: &Path, contents: &str) -> io::Result<usize> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(contents.lines().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_lf_endings_and_returns_a_row_count() {
        let dir = std::env::temp_dir().join(format!("edfund-fixture-{}", std::process::id()));
        let path = dir.join("nested/out.csv");
        let written = write_csv(
            &path,
            &["a", "b"],
            &[vec!["1".into(), "2".into()], vec!["3".into(), "4".into()]],
        )
        .unwrap();
        assert_eq!(written, 2);
        let raw = fs::read(&path).unwrap();
        assert!(
            !raw.windows(2).any(|w| w == b"\r\n"),
            "CRLF would churn the diff on every rebuild"
        );
        assert!(String::from_utf8_lossy(&raw).starts_with("a,b\n1,2\n"));
        let _ = fs::remove_dir_all(&dir);
    }

    /// The two ways a row can leave here unreadable, both refused at the point the builder
    /// that produced them is still nameable.
    #[test]
    #[should_panic(expected = "does not quote")]
    fn a_field_containing_a_comma_is_refused() {
        let dir = std::env::temp_dir().join(format!("edfund-comma-{}", std::process::id()));
        let _ = write_csv(
            &dir.join("out.csv"),
            &["a", "b"],
            &[vec!["1".into(), "Holy Trinity, Swanton Ele Sch".into()]],
        );
    }

    #[test]
    #[should_panic(expected = "has 3 cells where the header names 2")]
    fn a_row_wider_than_the_header_is_refused() {
        let dir = std::env::temp_dir().join(format!("edfund-width-{}", std::process::id()));
        let _ = write_csv(
            &dir.join("out.csv"),
            &["a", "b"],
            &[vec!["1".into(), "2".into(), "3".into()]],
        );
    }
}
