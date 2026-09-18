//! The department's typology: which of eight kinds of district each district is.
//!
//! Ohio has classified its districts into similar-district groups since 1996, revised in 2007 for
//! the 2000 census and again in 2013 for the 2010 one. The 2013 scheme, amended January 2015, is
//! the one in force: eight codes ordered by *urbanicity*, from rural through small town and
//! suburban to urban, each naming a student-poverty level and a student-population size alongside
//! it — plus a code zero for the districts the department removes from the analysis entirely.
//!
//! # Why this is worth a connector of its own
//!
//! Because the corpus has been asking for it since before it could answer. `education-agency.ont`
//! declares a `typology` property; five agency nodes fill it with prose and then record the
//! department's own code as `unfilled:`; `.yidam/skills/scenario-delta.md` states outright that
//! "typology is not an available axis"; and `fordham-house-bridge-commentary` warns that
//! **assessed valuation per pupil is the nearest proxy the corpus holds and it is not the
//! department's typology**. Four recorded open items, one 80 KB workbook.
//!
//! # The vintage is a fact about the department, not a defect here
//!
//! The assignment is **2013**, and nothing since has revised it. That is twelve years behind the
//! FY2027 model this repository computes, and it matters in a specific way: a district coded
//! `Rural — High Student Poverty` was one in 2013 and the poverty measure has moved since. So the
//! four measures the department classified *on* are carried beside the code, each named with the
//! vintage it was measured in, and a reader comparing `student_poverty_2013` against the report
//! card's current share can see the drift rather than having to assume it away.
//!
//! Carrying the criteria is the same argument [`super::designated`] makes for carrying the
//! EdChoice flags: a classification that cannot be interrogated is a label, and this repository's
//! standing objection is to labels whose derivation nothing records.
//!
//! # Code zero is not a ninth kind of district
//!
//! Five districts are coded `0`, and the department's own word for them is *removed from the
//! analysis*. Three are in this repository's panel — **Kelleys Island Local**, **Put-In-Bay
//! Local** and **College Corner Local** — and two are not, having closed since. They are island
//! and border districts of a few dozen pupils each, and three of their four classifying measures
//! read `N/A` in the workbook because the census tables they come from suppress them.
//!
//! This is worth stating rather than smoothing over: Kelleys Island Local has four pupils, and it
//! is the district whose per-pupil figures the `/reach` view had to fix its axes against. The
//! department reached the same conclusion about the same district from the other direction.

use super::delimited::column;
use super::format::clean_name;

/// Columns of the typology extract.
///
/// The four measures carry `_2013` because that is when they were measured, and a reader who
/// takes `student_poverty_2013` for a current poverty share is making an error this name is the
/// only thing standing in the way of. The code has no suffix: it is the assignment in force.
pub const TYPOLOGY_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "typology",
    "enrollment_2013",
    "median_income_2013",
    "student_poverty_2013",
    "pct_minority_2013",
];

/// The sheet the assignment is on. The workbook's other sheet is five exemplars per code.
pub const TYPOLOGY_SHEET: &str = "Typology By District";

/// The workbook's heading for each measure, in the order [`TYPOLOGY_HEADER`] names them.
///
/// `Pct. Minority` is the department's abbreviation and full stop, matched exactly, so the year
/// they tidy it this reader says so rather than dropping a column.
const MEASURES: &[&str] = &[
    "Enrollment",
    "Median Income",
    "Student Poverty",
    "Pct. Minority",
];

/// How many districts the department assigns a code. A floor, not an equality: districts close.
const AT_LEAST: usize = 600;

/// Build the typology extract from the `Typology By District` sheet.
///
/// # Errors
///
/// Returns the column whose heading has moved, a code outside `0..=8`, or a row count that has
/// fallen below the floor this module holds the workbook to — any of which means the file is not
/// the one this was written against, and every one of which would otherwise produce a plausible
/// fixture rather than a failure.
pub fn build_typology(rows: &[Vec<String>]) -> Result<Vec<Vec<String>>, String> {
    const FILE: &str = "the 2013 school district typology";
    let head = rows.first().cloned().unwrap_or_default();
    let at = |name: &str| column(&head, name, FILE);
    let (irn, district, county, code) = (
        at("IRN")?,
        at("District Name")?,
        at("County")?,
        at("2013 Typology")?,
    );
    let measures: Vec<usize> = MEASURES
        .iter()
        .map(|name| at(name))
        .collect::<Result<_, _>>()?;

    let mut out = Vec::new();
    for row in rows.iter().skip(1) {
        let get = |i: usize| row.get(i).map_or("", String::as_str).trim();
        if get(irn).is_empty() {
            continue;
        }
        let assigned = get(code);
        // Ordered 1..=8 by urbanicity, with 0 for the districts the department removes. A code
        // outside that is not a district this repository has a name for, and reading one as a
        // band would put it at an end of the ramp it does not belong at.
        if !matches!(
            assigned,
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8"
        ) {
            return Err(format!(
                "{FILE} gives district {} the typology code {assigned:?}, which is not 0-8",
                get(irn)
            ));
        }
        let mut record = vec![
            get(irn).to_string(),
            clean_name(get(district)),
            clean_name(get(county)),
            assigned.to_string(),
        ];
        for index in &measures {
            record.push(measure(get(*index)));
        }
        out.push(record);
    }

    if out.len() < AT_LEAST {
        return Err(format!(
            "{FILE} yielded {} districts, fewer than the {AT_LEAST} it should carry",
            out.len()
        ));
    }
    Ok(out)
}

/// One classifying measure, or an empty cell where the workbook suppresses it.
///
/// The workbook writes `N/A` for the census measures of its very small districts, and an empty
/// field is how every other fixture here says "not published". Writing `N/A` through would make
/// a reader parse the department's prose; writing `0` would assert a measurement.
///
/// The numbers are carried at the width the workbook states them — enrollment to fifteen decimal
/// places, because it is a weighted average rather than a headcount — for the reason
/// [`super::designated`] gives about its shares: rounding here would break the arithmetic that
/// makes the classification checkable against its own inputs.
fn measure(raw: &str) -> String {
    if raw.is_empty() || raw.eq_ignore_ascii_case("N/A") {
        return String::new();
    }
    raw.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(rows: &[&[&str]]) -> Vec<Vec<String>> {
        rows.iter()
            .map(|row| row.iter().map(|cell| (*cell).to_string()).collect())
            .collect()
    }

    fn head() -> Vec<&'static str> {
        vec![
            "IRN",
            "District Name",
            "County",
            "2013 Typology",
            "Enrollment",
            "Median Income",
            "Student Poverty",
            "Pct. Minority",
        ]
    }

    fn body(count: usize, code: &str) -> Vec<Vec<String>> {
        let mut rows = vec![head().iter().map(|h| (*h).to_string()).collect::<Vec<_>>()];
        for i in 0..count {
            rows.push(
                [
                    format!("{:06}", i + 1),
                    "Somewhere Local".to_string(),
                    "Adams".to_string(),
                    code.to_string(),
                    "842.09621085324795".to_string(),
                    "22901".to_string(),
                    "0.73242766382938296".to_string(),
                    "3.083414648400563E-2".to_string(),
                ]
                .to_vec(),
            );
        }
        rows
    }

    #[test]
    fn a_heading_that_moved_is_an_error_rather_than_a_misread() {
        let rows = sheet(&[&["IRN", "District Name", "County", "Typology"]]);
        let error = build_typology(&rows).unwrap_err();
        assert!(
            error.contains("2013 Typology"),
            "the error names the column that moved: {error}"
        );
    }

    #[test]
    fn a_code_outside_the_scheme_is_refused() {
        let mut rows = body(AT_LEAST, "1");
        rows[1][3] = "9".to_string();
        let error = build_typology(&rows).unwrap_err();
        assert!(error.contains("not 0-8"), "{error}");
        assert!(
            error.contains("000001"),
            "and it names the district: {error}"
        );
    }

    #[test]
    fn a_suppressed_measure_is_empty_rather_than_zero() {
        let mut rows = body(AT_LEAST, "0");
        for cell in rows[1].iter_mut().take(8).skip(4) {
            *cell = "N/A".to_string();
        }
        let built = build_typology(&rows).unwrap();
        assert_eq!(
            built[0][4..8],
            ["", "", "", ""],
            "an unpublished census measure is an absence, not a zero"
        );
        // And the code survives: a district removed from the analysis is still assigned code 0.
        assert_eq!(built[0][3], "0");
    }

    #[test]
    fn a_workbook_that_lost_its_districts_is_refused() {
        let error = build_typology(&body(12, "1")).unwrap_err();
        assert!(
            error.contains("fewer than"),
            "a short file is a changed file: {error}"
        );
    }

    #[test]
    fn the_measures_are_carried_at_the_width_the_department_states_them() {
        let built = build_typology(&body(AT_LEAST, "1")).unwrap();
        assert_eq!(
            built[0][6], "0.73242766382938296",
            "rounding here breaks the check that the classification follows from its inputs"
        );
    }
}
