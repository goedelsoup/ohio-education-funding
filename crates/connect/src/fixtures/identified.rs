//! Schools identified for support under ESSA, 2026, and the subgroups that identified them.
//!
//! A school appears once per subgroup that triggered its identification, so the count of rows is
//! not the count of schools, and the subgroup list is pinned rather than derived.

use super::delimited::column;

/// Columns of the identified-schools extract.
pub const IDENTIFIED_HEADER: &[&str] = &[
    "status",
    "building_irn",
    "building_name",
    "lea_irn",
    "lea_name",
    "school_type",
    "subgroups",
    "year_identified",
    "open_closed",
    "cycle",
    "school_year",
];

/// The subgroup columns TSI and ATSI mark. CSI has none — it is a whole-school identification.
const IDENTIFIED_SUBGROUPS: &[&str] = &[
    "Asian or Pacific Islander",
    "Black, Non-Hispanic",
    "Economic Disadvantage",
    "English Learner",
    "Hispanic",
    "Multiracial",
    "Students with Disabilities",
    "White, Non-Hispanic",
];

/// Who is actually in the accountability system, from the three published lists.
///
/// # Why one fixture for three files
///
/// CSI, TSI and ATSI are three tiers of one system and a reader almost always wants them
/// together — a building's status is the union of what it appears on, and the interesting rows
/// are those on more than one list. Keeping them apart would mean three joins to ask one
/// question.
///
/// The files do not share a layout. CSI carries `Year Identified`, `Previous Cycle Status` and an
/// open/closed flag and no subgroups, because it is a whole-school identification. TSI and ATSI
/// carry eight subgroup columns and no year, because they identify a school *through* a subgroup.
/// The union is taken with the missing fields blank rather than by inventing a common shape.
///
/// # `subgroups` is joined with a semicolon on purpose
///
/// [`super::write::write_csv`] refuses a field containing a comma, and "Black, Non-Hispanic" contains one. The
/// separator is a semicolon and the subgroup names keep their own commas replaced, so a consumer
/// splits on `;` and gets the department's labels back unaltered apart from that substitution.
///
/// # Errors
///
/// Returns the missing column's name if a list's layout has moved.
pub fn build_identified(lists: &[(&str, &[Vec<String>])]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for (status, rows) in lists {
        let label = format!("the {} identified-schools list", status.to_uppercase());
        let head = rows.first().cloned().unwrap_or_default();
        let at = |name: &str| column(&head, name, &label);
        let (lea_irn, lea_name, b_irn, b_name, kind) = (
            at("LEA IRN")?,
            at("LEA Name")?,
            at("Building IRN")?,
            at("Building Name")?,
            at("School Type")?,
        );
        // Present on CSI only; TSI and ATSI identify through a subgroup and carry neither.
        let year = at("Year Identified").ok();
        let open = at("Open/Closed Status").ok();
        let cycle = at("Identification Year Cycle").ok();
        let school_year = at("School Year").ok();
        let subgroups: Vec<(usize, &str)> = IDENTIFIED_SUBGROUPS
            .iter()
            .filter_map(|name| at(name).ok().map(|i| (i, *name)))
            .collect();

        for row in rows.iter().skip(1) {
            let get = |i: usize| row.get(i).map(|v| v.trim()).unwrap_or_default();
            // `Year Identified` is not always a year. It carries an escalation history —
            // "ATSI 2022 , CSI 2025" — for a building that moved between tiers, which is the
            // three-year ATSI-to-CSI path written into a cell. The comma is substituted for the
            // same reason the names are: this writer refuses one, and it caught this on the first
            // build.
            let opt =
                |i: Option<usize>| i.map(get).unwrap_or_default().replace(',', ";").to_string();
            if get(b_irn).is_empty() {
                continue;
            }
            let marked: Vec<String> = subgroups
                .iter()
                .filter(|(i, _)| !get(*i).is_empty())
                .map(|(_, name)| name.replace(',', ""))
                .collect();
            out.push(vec![
                (*status).to_string(),
                get(b_irn).to_string(),
                get(b_name).replace(',', ";"),
                get(lea_irn).to_string(),
                get(lea_name).replace(',', ";"),
                get(kind).replace(',', ";"),
                marked.join(";"),
                opt(year),
                opt(open),
                opt(cycle),
                opt(school_year),
            ]);
        }
    }
    out.sort();
    Ok(out)
}
