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
//! the bottom-20% Performance Index flags, three years of Title I formula shares, and the
//! average of those three. Keeping them makes the designation checkable against its own stated
//! rule, and that check is not redundant — it fails on rows the department marks Closed or
//! Inactive, which is a rule the workbook applies and does not state.
//!
//! # Why an edition is a parameter
//!
//! The department publishes one edition at a time and deletes the last, so the three this
//! repository holds have three different layouts and the differences are not cosmetic. Each
//! edition names the school year in its own `DESIGNATED for` heading, carries its own Title I
//! vintages, and — in 2024-2025 — its own *number* of Performance Index years. Matching the
//! headings exactly, per edition, is what makes a layout change an error rather than a column
//! read from the wrong year.

use super::delimited::column;
use super::format::{clean_name, format_value};

/// One published edition of the designated list, and the workbook headings it writes.
///
/// The year suffixes in [`Self::header`] are school years written the way the workbook's own
/// headings write them — `Bottom 20% PI Ranking 2022-2023` becomes `bottom_20_pi_2223` — so a
/// column in a fixture can be traced back to a column in the workbook without a lookup table.
pub struct Edition {
    /// The school year the list governs, as its own headings write it.
    pub school_year: &'static str,
    /// The fixture header this edition builds to.
    pub header: &'static [&'static str],
    /// The workbook's heading for each flag column, in the order [`Self::header`] names them.
    flags: &'static [&'static str],
    /// The workbook's heading for each share column, in order.
    shares: &'static [&'static str],
    /// The flag headings where an empty cell is admissible, and means "not ranked that year".
    ///
    /// Only the per-year Performance Index columns. A building the department did not rank in a
    /// given year — an academy that had not opened, an early learning centre, an online
    /// programme — is left blank there rather than written `No`, and the difference is real: it
    /// was not outside the bottom fifth, it was outside the ranking. 67 cells across the two
    /// archived editions and none in the current one, which writes `No` everywhere.
    unranked: &'static [&'static str],
}

/// 2026-2027, the edition the department currently serves.
///
/// `Title 1Formula Average` is the department's spacing and not a transcription slip — and it is
/// this edition's alone: both archived editions spell it `Title 1 Formula Average`, so the
/// missing space arrived with the 2026-2027 workbook. It is matched exactly, so the year they
/// fix it this reader says so rather than quietly dropping the column that decides five hundred
/// designations.
pub const DESIGNATED_2627: Edition = Edition {
    school_year: "2026-2027",
    header: &[
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
    ],
    flags: &[
        "DESIGNATED for 2026-2027",
        "Meets Criteria ADC Designation (Option A)",
        "Meets Criteria for Bottom 20% PI Ranking and Title 1Formula Average (Option B)",
        "Academic Distress School",
        "Bottom 20% PI Ranking Across Two of Prior Three Years",
        "Bottom 20% PI Ranking 2022-2023",
        "Bottom 20% PI Ranking 2023-2024",
        "Bottom 20% PI Ranking 2024-2025",
        "Title 1 Percent 20% or Above Across 3 Years",
    ],
    shares: &[
        "Title 1 Percent Formula 2023-2024",
        "Title 1 Percent Formula 2024-2025",
        "Title 1 Percent Formula 2025-2026",
        "Title 1 Average",
    ],
    unranked: &[
        "Bottom 20% PI Ranking 2022-2023",
        "Bottom 20% PI Ranking 2023-2024",
        "Bottom 20% PI Ranking 2024-2025",
    ],
};

/// 2025-2026, recovered from the Internet Archive.
///
/// Same shape as [`DESIGNATED_2627`] one year back: three Performance Index years, three Title I
/// vintages, and the same `Across Two of Prior Three Years` window.
pub const DESIGNATED_2526: Edition = Edition {
    school_year: "2025-2026",
    header: &[
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
        "bottom_20_pi_2122",
        "bottom_20_pi_2223",
        "bottom_20_pi_2324",
        "title1_at_least_20_three_years",
        "title1_share_2223",
        "title1_share_2324",
        "title1_share_2425",
        "title1_average",
    ],
    flags: &[
        "DESIGNATED for 2025-2026",
        "Meets Criteria ADC Designation (Option A)",
        "Meets Criteria for Bottom 20% PI Ranking and Title 1 Formula Average (Option B)",
        "Academic Distress School",
        "Bottom 20% PI Ranking Across Two of Prior Three Years",
        "Bottom 20% PI Ranking 2021-2022",
        "Bottom 20% PI Ranking 2022-2023",
        "Bottom 20% PI Ranking 2023-2024",
        "Title 1 Percent 20% or Above Across 3 Years",
    ],
    shares: &[
        "Title 1 Percent Formula 2022-2023",
        "Title 1 Percent Formula 2023-2024",
        "Title 1 Percent Formula 2024-2025",
        "Title 1 Average",
    ],
    unranked: &[
        "Bottom 20% PI Ranking 2021-2022",
        "Bottom 20% PI Ranking 2022-2023",
        "Bottom 20% PI Ranking 2023-2024",
    ],
};

/// 2024-2025, recovered from the Internet Archive, and the edition whose window is two years.
///
/// The statute asks for the bottom twenty per cent "for at least two of the three most recent
/// consecutive rankings". This edition ranks **two** years, not three, and its own heading says
/// so: `Bottom 20% PI Ranking Across 2022 and 2023 School Years` rather than
/// `Across Two of Prior Three Years`. So the column is named `bottom_20_pi_two_of_two` here —
/// two of two is a materially easier test than two of three, and a fixture column that called
/// it `two_of_three` would state something the workbook does not.
pub const DESIGNATED_2425: Edition = Edition {
    school_year: "2024-2025",
    header: &[
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
        "bottom_20_pi_two_of_two",
        "bottom_20_pi_2122",
        "bottom_20_pi_2223",
        "title1_at_least_20_three_years",
        "title1_share_2122",
        "title1_share_2223",
        "title1_share_2324",
        "title1_average",
    ],
    flags: &[
        "DESIGNATED for 2024-2025",
        "Meets Criteria ADC Designation (Option A)",
        "Meets Criteria for Bottom 20% PI Ranking and Title 1 Formula Average (Option B)",
        "Academic Distress School",
        "Bottom 20% PI Ranking Across 2022 and 2023 School Years",
        "Bottom 20% PI Ranking 2021-2022",
        "Bottom 20% PI Ranking 2022-2023",
        "Title 1 Percent 20% or Above Across 3 Years",
    ],
    shares: &[
        "Title 1 Percent Formula 2021-2022",
        "Title 1 Percent Formula 2022-2023",
        "Title 1 Percent Formula 2023-2024",
        "Title 1 Average",
    ],
    unranked: &[
        "Bottom 20% PI Ranking 2021-2022",
        "Bottom 20% PI Ranking 2022-2023",
    ],
};

/// A `Yes`/`No` cell, lowered, and an empty one where [`Edition::unranked`] admits it.
///
/// Anything else is a layout change or a new value the department has started using, and either
/// is worth an error rather than a row that reads `no` because it did not read `Yes`. A blank
/// stays blank rather than becoming `no`, because in a per-year ranking column those are
/// different facts and collapsing them would make a building that was never ranked look like one
/// the ranking cleared.
fn flag(cell: &str, heading: &str, irn: &str, unranked: bool) -> Result<String, String> {
    match cell.trim() {
        "Yes" => Ok("yes".to_string()),
        "No" => Ok("no".to_string()),
        "" if unranked => Ok(String::new()),
        other => Err(format!(
            "building {irn} reads {other:?} under {heading:?}, which is neither Yes nor No"
        )),
    }
}

/// One row per building on the designated list, whether or not it is designated.
///
/// # Why the whole list and not the designated buildings
///
/// 513 of 2,877 buildings are designated for 2026-2027. Keeping the other 2,364 is what makes
/// the file a denominator: "how many buildings in this district are designated" needs both
/// halves, and a question about a district with none of them cannot be answered from a file that
/// holds only the ones that qualified. It is also what makes an *edition pair* readable: a
/// building that stops being designated and a building that stops being listed are different
/// events, and only a file carrying both can tell them apart.
///
/// # Errors
///
/// If a column the reader needs is not in the header under the name it was written against, or
/// if a flag cell is neither `Yes` nor `No`.
pub fn build_designated(
    edition: &Edition,
    rows: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    let file = format!("the EdChoice designated list for {}", edition.school_year);
    let head = rows.first().cloned().unwrap_or_default();
    let at = |name: &str| column(&head, name, &file);
    let (county, district_irn, district, building_irn, building, grades, open) = (
        at("County")?,
        at("District IRN")?,
        at("District Name")?,
        at("Building IRN")?,
        at("Building Name")?,
        at("Grade Levels Served")?,
        at("Open/Closed Status")?,
    );
    let flags: Vec<usize> = edition
        .flags
        .iter()
        .map(|name| at(name))
        .collect::<Result<_, _>>()?;
    let shares: Vec<usize> = edition
        .shares
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
        for (i, heading) in flags.iter().zip(edition.flags) {
            let unranked = edition.unranked.contains(heading);
            record.push(flag(get(*i), heading, get(building_irn), unranked)?);
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

    fn header(edition: &Edition) -> Vec<String> {
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
        head.extend(edition.flags.iter().map(|c| (*c).to_string()));
        head.extend(edition.shares.iter().map(|c| (*c).to_string()));
        head
    }

    fn building(
        edition: &Edition,
        irn: &str,
        grades: &str,
        open: &str,
        designated: &str,
    ) -> Vec<String> {
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
        row.extend(std::iter::repeat_n(
            "No".to_string(),
            edition.flags.len() - 1,
        ));
        row.extend(["0.25", "0.35", "0.3", "0.3"].into_iter().map(String::from));
        row
    }

    fn sheet(edition: &Edition) -> Vec<Vec<String>> {
        vec![
            header(edition),
            building(edition, "001537", "K-5,P", "Open", "Yes"),
            building(edition, "002634", "9-12", "Open", "No"),
        ]
    }

    #[test]
    fn a_comma_in_the_grade_list_becomes_a_semicolon() {
        let rows =
            build_designated(&DESIGNATED_2627, &sheet(&DESIGNATED_2627)).expect("the sheet reads");
        assert_eq!(rows[0][5], "K-5;P");
        assert!(rows
            .iter()
            .all(|r| r.iter().all(|cell| !cell.contains(','))));
    }

    #[test]
    fn every_building_is_carried_not_only_the_designated_ones() {
        let rows =
            build_designated(&DESIGNATED_2627, &sheet(&DESIGNATED_2627)).expect("the sheet reads");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][7], "yes");
        assert_eq!(rows[1][7], "no");
    }

    /// Every edition, because their widths differ and a header that has drifted out of step with
    /// its flag list would otherwise only show up in a rebuild.
    #[test]
    fn every_row_is_the_header_width() {
        for edition in [&DESIGNATED_2627, &DESIGNATED_2526, &DESIGNATED_2425] {
            let rows = build_designated(edition, &sheet(edition)).expect("the sheet reads");
            assert!(
                rows.iter().all(|r| r.len() == edition.header.len()),
                "{} builds rows that are not its header's width",
                edition.school_year
            );
            assert_eq!(
                edition.header.len(),
                7 + edition.flags.len() + edition.shares.len(),
                "{} declares a header its column lists cannot fill",
                edition.school_year
            );
        }
    }

    /// The archived editions are a year apart and not a relabelling of each other: 2024-2025
    /// carries one Performance Index year fewer than the two after it, and its own heading says
    /// so.
    #[test]
    fn the_first_archived_edition_ranks_two_years_and_the_others_three() {
        let pi = |e: &Edition| {
            e.header
                .iter()
                .filter(|c| c.starts_with("bottom_20_pi_2"))
                .count()
        };
        assert_eq!(pi(&DESIGNATED_2425), 2);
        assert_eq!(pi(&DESIGNATED_2526), 3);
        assert_eq!(pi(&DESIGNATED_2627), 3);
        assert!(DESIGNATED_2425.header.contains(&"bottom_20_pi_two_of_two"));
    }

    /// The spacing of the Option B heading is edition-specific, and the reader matches it
    /// exactly rather than normalizing it away. `Title 1Formula Average` is 2026-2027's alone.
    #[test]
    fn the_option_b_heading_is_respaced_in_the_current_edition_only() {
        assert!(DESIGNATED_2627.flags[2].contains("Title 1Formula Average"));
        for edition in [&DESIGNATED_2526, &DESIGNATED_2425] {
            assert!(
                edition.flags[2].contains("Title 1 Formula Average"),
                "{} should carry the spaced heading",
                edition.school_year
            );
        }
    }

    #[test]
    fn a_renamed_column_is_an_error() {
        let mut rows = sheet(&DESIGNATED_2627);
        rows[0][9] = "Meets Criteria for Bottom 20% PI Ranking and Title 1 Formula Average \
                      (Option B)"
            .to_string();
        let error =
            build_designated(&DESIGNATED_2627, &rows).expect_err("the heading was respaced");
        assert!(error.contains("layout has moved"), "{error}");
        assert!(error.contains("2026-2027"), "{error}");
    }

    /// An edition read against another edition's headings fails rather than building a fixture
    /// whose year columns come from the wrong year.
    #[test]
    fn an_edition_will_not_read_its_neighbour() {
        let error = build_designated(&DESIGNATED_2526, &sheet(&DESIGNATED_2627))
            .expect_err("the 2026-2027 sheet is not the 2025-2026 layout");
        assert!(error.contains("DESIGNATED for 2025-2026"), "{error}");
    }

    #[test]
    fn a_flag_that_is_not_yes_or_no_is_an_error() {
        let mut rows = sheet(&DESIGNATED_2627);
        rows[1][7] = "Pending".to_string();
        let error = build_designated(&DESIGNATED_2627, &rows).expect_err("an unknown flag value");
        assert!(error.contains("001537"), "{error}");
        assert!(error.contains("neither Yes nor No"), "{error}");
    }

    /// A blank in a per-year ranking column is the department saying the building was not
    /// ranked that year, not saying it cleared the bottom fifth. It stays blank.
    #[test]
    fn a_blank_year_survives_as_a_blank_and_a_blank_determination_does_not() {
        let year = header(&DESIGNATED_2526)
            .iter()
            .position(|c| c == "Bottom 20% PI Ranking 2021-2022")
            .expect("the edition ranks that year");
        let mut rows = sheet(&DESIGNATED_2526);
        rows[1][year] = String::new();
        let built = build_designated(&DESIGNATED_2526, &rows).expect("the sheet reads");
        assert_eq!(built[0][12], "");

        let mut rows = sheet(&DESIGNATED_2526);
        rows[1][7] = String::new();
        let error = build_designated(&DESIGNATED_2526, &rows)
            .expect_err("a blank determination is a layout change, not an unranked year");
        assert!(error.contains("DESIGNATED for 2025-2026"), "{error}");
    }

    /// Every heading that admits a blank is a heading the edition actually reads, so a typo in
    /// one cannot quietly leave a column strict.
    #[test]
    fn a_blankable_heading_is_one_of_the_editions_own_flags() {
        for (edition, years) in [
            (&DESIGNATED_2627, 3),
            (&DESIGNATED_2526, 3),
            (&DESIGNATED_2425, 2),
        ] {
            assert_eq!(edition.unranked.len(), years, "{}", edition.school_year);
            for heading in edition.unranked {
                assert!(
                    edition.flags.contains(heading),
                    "{}: {heading:?} is not a flag column",
                    edition.school_year
                );
            }
        }
    }

    /// The shares are what the average identity is checked against downstream, so they keep
    /// their published width rather than the money fixtures' two places.
    #[test]
    fn a_share_keeps_enough_places_to_average() {
        let mut rows = sheet(&DESIGNATED_2627);
        rows[1][16] = "0.27990135635018498".to_string();
        let built = build_designated(&DESIGNATED_2627, &rows).expect("the sheet reads");
        assert_eq!(built[0][16], "0.279901356350185");
    }
}
