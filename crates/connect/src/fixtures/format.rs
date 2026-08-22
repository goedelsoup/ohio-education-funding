//! How a number and a name are written into a fixture.
//!
//! Two functions, and both exist because of a bug they caused. The formatting rule is shared by
//! every builder, so it is defined once rather than restated per file.

/// Format a value the way the fixtures are written: fixed decimals, trailing zeros trimmed,
/// blank for absent.
///
/// Blank rather than `0` for absent is load-bearing. A district with no reported valuation and
/// a district whose valuation is nil are different claims, and the calculators read the
/// difference as `Option<f64>`.
///
/// The rendering itself is [`edfund_core::decimal::trimmed`], which carries the note about what
/// trimming an integer does to a building count. What is this function's own is the `None` arm.
#[must_use]
pub fn format_value(value: Option<f64>, places: usize) -> String {
    value.map_or_else(String::new, |value| {
        edfund_core::decimal::trimmed(value, places)
    })
}

/// District names go into a comma-separated file, so commas are replaced rather than quoted.
#[must_use]
pub fn clean_name(raw: &str) -> String {
    raw.replace(',', " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_trim_only_past_a_decimal_point() {
        assert_eq!(format_value(Some(847.823), 4), "847.823");
        assert_eq!(format_value(Some(20.0), 4), "20");
        assert_eq!(format_value(Some(10.0), 0), "10");
        assert_eq!(format_value(Some(100.0), 0), "100");
        assert_eq!(format_value(None, 2), "");
    }
}
