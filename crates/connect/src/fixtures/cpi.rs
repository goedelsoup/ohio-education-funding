//! One BLS series, one period, out of a national price file.
//!
//! Not a CSV builder — the output is a trimmed TSV, because the input is one and the deflator
//! reads it as one. [`crate::cpi`] is the reader; this is what writes what it reads.

/// Reduce the Bureau's 2.7 MB all-series flat file to the one series and period the deflator
/// uses.
///
/// Committing the extract rather than the whole file is what makes the deflator's verification
/// hermetic: [`deflator`](../../../deflator/) can claim its index points are checked against the
/// agency, and a test proves it without a network. Lines are kept in their published form so
/// [`crate::cpi::parse_series`] reads the extract and the original identically.
#[must_use]
pub fn build_cpi_extract(text: &str, series_id: &str, period: &str) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let mut lines = text.lines();
    if let Some(header) = lines.next() {
        out.push_str(header);
        out.push('\n');
    }
    for line in lines {
        let mut fields = line.split('\t').map(str::trim);
        if fields.next() == Some(series_id) && fields.nth(1) == Some(period) {
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cpi_extract_keeps_the_header_and_one_series() {
        let text = "series_id\tyear\tperiod\tvalue\n\
             CUUR0000SA0     \t2000\tM06\t       172.400\t\n\
             CUUR0000SA0     \t2000\tM05\t       171.300\t\n\
             CUUR0000SAF1    \t2000\tM06\t       167.900\t\n";
        let extract = build_cpi_extract(text, "CUUR0000SA0", "M06");
        let lines: Vec<&str> = extract.lines().collect();
        assert_eq!(lines.len(), 2, "header plus one matching observation");
        assert!(lines[0].starts_with("series_id"));
        assert!(lines[1].contains("172.400"));
    }
}
