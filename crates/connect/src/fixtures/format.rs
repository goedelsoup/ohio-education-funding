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
///
/// The replacement is what makes the collapse necessary, and that is the third bug this file
/// exists because of. A publisher writes `EDGE ACADEMY, THE ELEMENTARY`; substituting a space
/// for the comma leaves `EDGE ACADEMY  THE ELEMENTARY`, because the comma already had a space
/// after it. So the function manufactured the doubled space it then preserved, in five
/// committed fixtures — the CCD directory, the casino distributions, both EdChoice extracts and
/// the community school model.
///
/// #439 is what made it worth fixing rather than noting: the department's revised FY2024
/// profile report restates 25 district names with a doubled space of the publisher's own, and
/// a rule that collapses one kind and not the other would have to explain which.
#[must_use]
pub fn clean_name(raw: &str) -> String {
    raw.replace(',', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_substituted_comma_does_not_leave_a_doubled_space() {
        // The publisher's spelling, and the one `replace` alone produced from it.
        assert_eq!(
            clean_name("EDGE ACADEMY, THE ELEMENTARY"),
            "EDGE ACADEMY THE ELEMENTARY"
        );
        // A doubled space the publisher wrote, which the revised FY2024 profile report has in
        // 25 district names. Same rule, so there is nothing to tell apart.
        assert_eq!(
            clean_name("Arcanum-Butler Local  (046631) - Darke County"),
            "Arcanum-Butler Local (046631) - Darke County"
        );
        assert_eq!(
            clean_name("  Ada Exempted Village  "),
            "Ada Exempted Village"
        );
    }

    #[test]
    fn formats_trim_only_past_a_decimal_point() {
        assert_eq!(format_value(Some(847.823), 4), "847.823");
        assert_eq!(format_value(Some(20.0), 4), "20");
        assert_eq!(format_value(Some(10.0), 0), "10");
        assert_eq!(format_value(Some(100.0), 0), "100");
        assert_eq!(format_value(None, 2), "");
    }
}
