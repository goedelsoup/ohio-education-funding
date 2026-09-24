//! The EdChoice designated list: which buildings' students may claim a traditional scholarship.
//!
//! Eligibility, not participation. A designated building does not mean a scholarship was used
//! there — it means a student enrolled in or assigned to it may apply — so this answers a
//! different question from the unreached per-district participation route it stands in for, and
//! answers it per district and per building IRN.
//!
//! # Why the criteria columns are carried rather than collapsed
//!
//! The workbook publishes the designation *and* every input to it: the two statutory options,
//! three years of bottom-20% Performance Index flags, three years of Title I formula shares, and
//! the average of those three. Keeping them makes the designation checkable against its own
//! stated rule, and that check is not redundant — it fails on four rows, all of them buildings
//! the department marks Closed or Inactive, which is a rule the workbook applies and does not
//! state.

use super::delimited::column;
use super::format::{clean_name, format_value};

/// Columns of the designated-list extract.
///
/// The year suffixes are school years written the way the workbook's own headings write them —
/// `Bottom 20% PI Ranking 2022-2023` becomes `bottom_20_pi_2223` — so a column here can be
/// traced back to a column there without a lookup table.
pub const DESIGNATED_HEADER: &[&str] = &[
    "county",
    "district_irn",
    "district_name",
    "building_irn",
    "building_name",
    "grade_levels",
    "open_closed",
    "designated",
    "option_a_academic_distress",
    "option_b_pi_and_title1",
    "academic_distress_school",
    "bottom_20_pi_two_of_three",
    "bottom_20_pi_2223",
    "bottom_20_pi_2324",
    "bottom_20_pi_2425",
    "title1_at_least_20_three_years",
    "title1_share_2324",
    "title1_share_2425",
    "title1_share_2526",
    "title1_average",
];

/// The workbook's heading for each flag column, in the order [`DESIGNATED_HEADER`] names them.
///
/// `Title 1Formula Average` is the department's spacing and not a transcription slip. It is
/// matched exactly, so the year they fix it this reader says so rather than quietly dropping the
/// column that decides five hundred designations.
const FLAGS: &[&str] = &[
    "DESIGNATED for 2026-2027",
    "Meets Criteria ADC Designation (Option A)",
    "Meets Criteria for Bottom 20% PI Ranking and Title 1Formula Average (Option B)",
    "Academic Distress School",
    "Bottom 20% PI Ranking Across Two of Prior Three Years",
    "Bottom 20% PI Ranking 2022-2023",
    "Bottom 20% PI Ranking 2023-2024",
    "Bottom 20% PI Ranking 2024-2025",
    "Title 1 Percent 20% or Above Across 3 Years",
];

/// The workbook's heading for each share column, in order.
const SHARES: &[&str] = &[
    "Title 1 Percent Formula 2023-2024",
    "Title 1 Percent Formula 2024-2025",
    "Title 1 Percent Formula 2025-2026",
    "Title 1 Average",
];

/// A `Yes`/`No` cell, lowered.
///
/// Every flag column in this edition is one of the two with no blanks anywhere, so anything else
/// is a layout change or a new value the department has started using, and either is worth an
/// error rather than a row that reads `no` because it did not read `Yes`.
fn flag(cell: &str, heading: &str, irn: &str) -> Result<String, String> {
    match cell.trim() {
        "Yes" => Ok("yes".to_string()),
        "No" => Ok("no".to_string()),
        other => Err(format!(
            "building {irn} reads {other:?} under {heading:?}, which is neither Yes nor No"
        )),
    }
}

/// One row per building on the designated list, whether or not it is designated.
///
/// # Why the whole list and not the designated buildings
///
/// 513 of 2,877 buildings are designated. Keeping the other 2,364 is what makes the file a
/// denominator: "how many buildings in this district are designated" needs both halves, and a
/// question about a district with none of them cannot be answered from a file that holds only
/// the ones that qualified.
///
/// # Errors
///
/// If a column the reader needs is not in the header under the name it was written against, or
/// if a flag cell is neither `Yes` nor `No`.
pub fn build_designated(rows: &[Vec<String>]) -> Result<Vec<Vec<String>>, String> {
    const FILE: &str = "the EdChoice designated list";
    let head = rows.first().cloned().unwrap_or_default();
    let at = |name: &str| column(&head, name, FILE);
    let (county, district_irn, district, building_irn, building, grades, open) = (
        at("County")?,
        at("District IRN")?,
        at("District Name")?,
        at("Building IRN")?,
        at("Building Name")?,
        at("Grade Levels Served")?,
        at("Open/Closed Status")?,
    );
    let flags: Vec<usize> = FLAGS
        .iter()
        .map(|name| at(name))
        .collect::<Result<_, _>>()?;
    let shares: Vec<usize> = SHARES
        .iter()
        .map(|name| at(name))
        .collect::<Result<_, _>>()?;

    let mut out = Vec::new();
    for row in rows.iter().skip(1) {
        let get = |i: usize| row.get(i).map_or("", String::as_str).trim();
        if get(building_irn).is_empty() {
            continue;
        }
        // `Grade Levels Served` is where the commas are — 602 of 2,877 buildings read "7-12,PS"
        // or "K-6,P" — and `write_csv` refuses a field containing one. A semicolon keeps the
        // list splittable; `clean_name` would turn it into a space and lose the boundary.
        let mut record = vec![
            clean_name(get(county)),
            get(district_irn).to_string(),
            clean_name(get(district)),
            get(building_irn).to_string(),
            clean_name(get(building)),
            get(grades).replace(", ", ";").replace(',', ";"),
            get(open).to_string(),
        ];
        for (i, heading) in flags.iter().zip(FLAGS) {
            record.push(flag(get(*i), heading, get(building_irn))?);
        }
        // Fifteen places, because these are shares of a formula count and the workbook carries
        // them at full float width. Rounding to the two or four places the money fixtures use
        // would break the identity that the average is the mean of the three years beside it,
        // which is the only check this file's arithmetic can be held to.
        for i in &shares {
            record.push(format_value(get(*i).parse().ok(), 15));
        }
        out.push(record);
    }
    out.sort();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> Vec<String> {
        let mut head = vec![
            "County",
            "District IRN",
            "District Name",
            "Building IRN",
            "Building Name",
            "Grade Levels Served",
            "Open/Closed Status",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        head.extend(FLAGS.iter().map(|c| (*c).to_string()));
        head.extend(SHARES.iter().map(|c| (*c).to_string()));
        head
    }

    fn building(irn: &str, grades: &str, open: &str, designated: &str) -> Vec<String> {
        let mut row = vec![
            "Summit".to_string(),
            "043489".into(),
            "Akron City".into(),
            irn.into(),
            "Barber Community Learning Center".into(),
            grades.into(),
            open.into(),
        ];
        row.push(designated.to_string());
        row.extend(std::iter::repeat_n("No".to_string(), FLAGS.len() - 1));
        row.extend(["0.25", "0.35", "0.3", "0.3"].into_iter().map(String::from));
        row
    }

    fn sheet() -> Vec<Vec<String>> {
        vec![
            header(),
            building("001537", "K-5,P", "Open", "Yes"),
            building("002634", "9-12", "Open", "No"),
        ]
    }

    #[test]
    fn a_comma_in_the_grade_list_becomes_a_semicolon() {
        let rows = build_designated(&sheet()).expect("the sheet reads");
        assert_eq!(rows[0][5], "K-5;P");
        assert!(rows
            .iter()
            .all(|r| r.iter().all(|cell| !cell.contains(','))));
    }

    #[test]
    fn every_building_is_carried_not_only_the_designated_ones() {
        let rows = build_designated(&sheet()).expect("the sheet reads");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][7], "yes");
        assert_eq!(rows[1][7], "no");
    }

    #[test]
    fn every_row_is_the_header_width() {
        let rows = build_designated(&sheet()).expect("the sheet reads");
        assert!(rows.iter().all(|r| r.len() == DESIGNATED_HEADER.len()));
    }

    #[test]
    fn a_renamed_column_is_an_error() {
        let mut rows = sheet();
        rows[0][9] = "Meets Criteria for Bottom 20% PI Ranking and Title 1 Formula Average \
                      (Option B)"
            .to_string();
        let error = build_designated(&rows).expect_err("the heading was respaced");
        assert!(error.contains("layout has moved"), "{error}");
    }

    #[test]
    fn a_flag_that_is_not_yes_or_no_is_an_error() {
        let mut rows = sheet();
        rows[1][7] = "Pending".to_string();
        let error = build_designated(&rows).expect_err("an unknown flag value");
        assert!(error.contains("001537"), "{error}");
        assert!(error.contains("neither Yes nor No"), "{error}");
    }

    /// The shares are what the average identity is checked against downstream, so they keep
    /// their published width rather than the money fixtures' two places.
    #[test]
    fn a_share_keeps_enough_places_to_average() {
        let mut rows = sheet();
        rows[1][16] = "0.27990135635018498".to_string();
        let built = build_designated(&rows).expect("the sheet reads");
        assert_eq!(built[0][16], "0.279901356350185");
    }
}
