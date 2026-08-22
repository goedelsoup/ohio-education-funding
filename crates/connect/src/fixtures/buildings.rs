//! The 2024-25 report card at building grain.
//!
//! The same publisher's file as [`super::report_card`], one level down. Building grain is what
//! makes a within-district question askable at all.

use std::collections::BTreeMap;

use super::delimited::column;

/// Columns of the building fixture.
///
/// **There is no overall rating here, and its absence is the point.** R.C. 3302.10 triggers an
/// academic distress commission on three consecutive years of an *overall* grade or star rating,
/// and ESSA rank-orders the same overall rating to identify CSI schools. Neither published
/// building file carries it: `Achievement_Building` gives the **achievement component** star
/// rating, and `Building_Details` gives no rating at all. What is held is therefore the input to
/// the accountability trigger rather than the trigger itself, and a node must not present the
/// achievement star as though it were the overall one.
pub const BUILDING_HEADER: &[&str] = &[
    "building_irn",
    "building_name",
    "district_irn",
    "district_name",
    "county",
    "enrollment",
    "chronic_absenteeism",
    "achievement_star_rating",
    "performance_index_2425",
    "performance_index_2324",
    "performance_index_2223",
];

/// The building-level report card extract, from the achievement and details workbooks.
///
/// # Why the building grain is held at all
///
/// Every other fixture here is agency-level, because the funding formula pays agencies. The
/// accountability system does not: ESSA identifies **schools**, and R.C. 3302.12 attaches its
/// intervention to a **building**. Rolling either up to the district would record the consequence
/// and discard the unit that triggers it.
///
/// # Three years of Performance Index, on purpose
///
/// The achievement workbook publishes 2024-25 beside 2023-24 and 2022-23 in the same row. The
/// distress trigger and the CSI escalation both read three consecutive years, so a single-year
/// extract could not be checked against either. Carried as three columns rather than three rows
/// because that is how the source publishes them and reshaping would invent a panel the file does
/// not contain.
///
/// # Chronic absenteeism is a CSI criterion, not colour
///
/// For schools with fewer than three rated report card components, Ohio identifies the lowest 5%
/// **by chronic absenteeism**. It comes from the details workbook's `All Students` rows, which are
/// long-form — one row per building per student group — so everything but `All Students` is
/// dropped here rather than aggregated.
///
/// # Errors
///
/// Returns the missing column's name if either workbook's layout has moved.
pub fn build_buildings(
    achievement: &[Vec<String>],
    details: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    const ACH: &str = "the building achievement workbook";
    const DET: &str = "the building details workbook";

    let head = |rows: &[Vec<String>]| rows.first().cloned().unwrap_or_default();
    let ach_head = head(achievement);
    let det_head = head(details);
    let at = |h: &[String], name: &str, file: &str| column(h, name, file);

    let (b_irn, b_name, d_irn, d_name, county) = (
        at(&ach_head, "Building IRN", ACH)?,
        at(&ach_head, "Building Name", ACH)?,
        at(&ach_head, "District IRN", ACH)?,
        at(&ach_head, "District Name", ACH)?,
        at(&ach_head, "County", ACH)?,
    );
    let star = at(&ach_head, "Achievement Component Star Rating", ACH)?;
    let pi = [
        at(&ach_head, "Performance Index Score 2024-2025", ACH)?,
        at(&ach_head, "Performance Index Score 2023-2024", ACH)?,
        at(&ach_head, "Performance Index Score 2022-2023", ACH)?,
    ];

    let (d_b_irn, d_group, d_enrol, d_absent) = (
        at(&det_head, "Building IRN", DET)?,
        at(&det_head, "Student Group", DET)?,
        at(&det_head, "Enrollment", DET)?,
        at(&det_head, "Chronic Absenteeism Rate", DET)?,
    );

    let mut detail: BTreeMap<String, (String, String)> = BTreeMap::new();
    for row in details.iter().skip(1) {
        let get = |i: usize| row.get(i).map(|v| v.trim()).unwrap_or_default();
        if get(d_group) != "All Students" {
            continue;
        }
        detail.insert(
            get(d_b_irn).to_string(),
            (get(d_enrol).to_string(), get(d_absent).to_string()),
        );
    }

    let mut out = Vec::new();
    for row in achievement.iter().skip(1) {
        let get = |i: usize| row.get(i).map(|v| v.trim()).unwrap_or_default();
        let irn = get(b_irn).to_string();
        if irn.is_empty() {
            continue;
        }
        let (enrolment, absent) = detail.get(&irn).cloned().unwrap_or_default();
        // "4  Stars" as published, with the doubled space. Kept as the number alone so a consumer
        // does not have to know that, and blank where the building is unrated.
        let stars = get(star)
            .split_whitespace()
            .next()
            .filter(|v| v.parse::<f64>().is_ok())
            .unwrap_or_default()
            .to_string();
        out.push(vec![
            irn,
            get(b_name).replace(',', ";"),
            get(d_irn).to_string(),
            get(d_name).replace(',', ";"),
            get(county).replace(',', ";"),
            enrolment,
            absent,
            stars,
            get(pi[0]).to_string(),
            get(pi[1]).to_string(),
            get(pi[2]).to_string(),
        ]);
    }
    out.sort();
    Ok(out)
}
