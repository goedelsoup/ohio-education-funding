//! Home education by district: the only per-district breakdown of it known to exist.
//!
//! Ohio publishes **one** home-education number in machine-reachable form — `Home School —
//! 53,051`, statewide, in the department's 2023-2024 fact sheet, which this repository already
//! holds as [`super::landscape`]. The department collects the counts per district of residence
//! under R.C. 3321.042 and releases them on request; it publishes them nowhere. The Johns
//! Hopkins Homeschool Hub workbook is an aggregator that asked, and its Ohio district sheet is
//! the released file carried forward.
//!
//! This module commits it. The catalog entry
//! [`jhu-homeschool-hub`](../../../../.yidam/catalog/jhu-homeschool-hub.md) recorded four
//! defects and concluded the sheet was not fixture material; three of the four turned out to be
//! answerable from the file and the fixtures this repository already holds, and what remains is
//! carried in the data rather than argued around.
//!
//! # The sheet is not keyed, and this module keys it
//!
//! The 611 Ohio rows carry a district name and a count. No IRN, no county — the schema's county
//! column is the same column C, which other states fill with counties because the codebook
//! prefers them. 583 distinct names across 611 rows: 20 names repeat, `Perry Local` three times,
//! `Buckeye Local` three times, and a name alone cannot say which is Lake and which is Stark.
//!
//! **The rows are in IRN order**, which is what closes it. Of the 611, 560 have a name unique in
//! the department's own district list and 558 of those are strictly increasing in IRN — one
//! monotone run covering rows 1 through 606, with the last four rows appended out of order. A
//! repeated name between two anchors is bounded by them, and in every case but four the bound
//! admits exactly one candidate; the four in the appended tail fall out by elimination once the
//! ordered region has taken its own. **611 rows resolve to 611 distinct IRNs and nothing is left
//! over**, and [`build_home_education_districts`] fails rather than emits if that ever stops
//! being true.
//!
//! The join is against [`super::typology`], which is the department's own list of districts with
//! their counties and already disambiguates the repeats the way this needs — `Perry Local
//! (Allen)`, `Buckeye Local (Jefferson)`. Source to source, not fixture to fixture.
//!
//! # The year label is wrong, and the fixture says only what the file says
//!
//! Every Ohio row is labeled `SY21-22`. The 559 numeric rows sum to **51,349**, and the same
//! workbook's statewide sheet gives 2021-2022 as **47,491**. A district file cannot exceed its
//! own state total by 3,858, so the label is refuted by the workbook against itself, and
//! [`build_home_education_districts`] asserts precisely that contradiction — it is the one claim
//! here that rests on nothing but the two sheets.
//!
//! Which year it *is* does not belong in committed data. The catalog entry carries the argument
//! (2023-2024, on a size-matched reconstruction of the censored cells that lands within 49 of
//! the published 53,051, where every other year misses by 3% to 13%); this fixture carries
//! `published_year` verbatim as the workbook wrote it. A reader gets the publisher's label and
//! can find out from the catalog why not to trust it. Writing an inferred year into a column
//! would launder the inference into data, which is the failure mode the whole
//! `[verified]`/`[inference]` split exists to prevent.
//!
//! # Three censoring markers, one of them bounded
//!
//! 27 cells read `<10`, 20 read `.`, and **5 read `NC`** — the third marker is not in the
//! catalog's count of two, and the entry's "564 numeric rows" is 611 less 47 rather than a
//! count: there are 559. The codebook defines none of the three.
//!
//! `.` is the only one nine other states also use, which reads as the compiler's own blank;
//! `<10` and `NC` appear on Ohio rows alone and read as the department's. So they are carried
//! as the floor/ceiling pair [`super::nonpublic`] uses for the same problem: `<10` bounds a cell
//! at `[0, 9]`, and `.` and `NC` bound it at `[0, ]` with the ceiling left empty, because
//! nothing in the file bounds them above and Southwest Local — 4,150 pupils — is one of them.
//! The published cell text is carried beside the pair so the distinction survives.
//!
//! # Twenty districts report an explicit zero
//!
//! Not censored, not blank: `0`. Among them **Cincinnati Public Schools**, 34,860 pupils, and
//! Washington Local at 6,771. Whether those are true zeros or a second unmarked blank is `[open]`
//! and the fixture does not decide it — a zero floor and a zero ceiling is what the file says.
//! It matters to the arithmetic: read as real they scale to 53,851, read as missing to 56,254,
//! and only the first matches any published year.

use std::collections::{BTreeMap, BTreeSet};

use super::delimited::column;
use super::format::clean_name;

/// Columns of the per-district extract.
///
/// `published` is the cell as the workbook wrote it — a count, or one of the three censoring
/// markers. The floor and ceiling are the same number on a reported row, which is what makes
/// summing the column meaningful without a reader having to know which rows were masked.
pub const HOME_EDUCATION_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "published_year",
    "published",
    "students_floor",
    "students_ceiling",
];

/// Columns of the statewide series.
pub const HOME_EDUCATION_STATEWIDE_HEADER: &[&str] = &["school_year", "students"];

/// The sheet the district rows are on. 12,280 rows over 39 states; Ohio's are 611 of them.
pub const HOME_EDUCATION_SHEET: &str = "BY COUNTY, DISTRICT, TOWN, LEA";

/// The sheet Ohio's statewide annual series is on. One sheet per state, named for the state.
pub const HOME_EDUCATION_STATEWIDE_SHEET: &str = "OH";

/// The two-letter code the district sheet files Ohio's rows under.
const OHIO: &str = "OH";

/// How many Ohio districts the sheet should carry. A floor, not an equality: districts close.
const AT_LEAST: usize = 600;

/// Workbook spellings that are a district's name under another form.
///
/// Six, and each is the aggregator writing the district's common name where the department
/// writes its legal one — `Cincinnati Public Schools` for `Cincinnati City`, and four more of
/// the same kind. The sixth is punctuation: the department's `Adams County/Ohio Valley Local`
/// has a solidus the workbook drops.
///
/// Kept as an explicit table rather than absorbed into [`normalize`] because each one is a
/// judgement about a particular district, and a normalizer loose enough to make them match
/// without being told would also collapse districts that are genuinely different.
const ALIASES: &[(&str, &str)] = &[
    (
        "adams county ohio valley local",
        "adams county/ohio valley local",
    ),
    ("cincinnati public", "cincinnati city"),
    ("cleveland municipal", "cleveland municipal city"),
    ("grandview heights", "grandview heights city"),
    ("new lexington", "new lexington city"),
    ("sylvania", "sylvania city"),
];

/// Reduce a district name to the form the two publishers agree on.
///
/// Strips a parenthesised disambiguator — the typology fixture carries `Perry Local (Allen)`
/// where the workbook carries `Perry Local`, and the county is exactly what is being recovered
/// here, so it cannot be an input. Then the governance suffixes, which the two spell four
/// different ways, and the hyphens and full stops they disagree about.
fn normalize(raw: &str) -> String {
    let mut bare = String::with_capacity(raw.len());
    let mut depth = 0usize;
    for ch in raw.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => bare.push(ch),
            _ => {}
        }
    }
    let mut name = bare.to_lowercase().trim().to_string();
    // Longest first: `schools district` must not be read as `district` with `schools` left on.
    for suffix in [
        " school district",
        " schools district",
        " schools",
        " district",
        " sd",
    ] {
        while let Some(shorter) = name.strip_suffix(suffix) {
            name = shorter.trim_end().to_string();
        }
    }
    name.replace('-', " ")
        .replace('.', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The normalized name to look a workbook spelling up under.
fn key_for(raw: &str) -> String {
    let name = normalize(raw);
    ALIASES
        .iter()
        .find(|(from, _)| *from == name)
        .map_or(name, |(_, to)| (*to).to_string())
}

/// Build the statewide annual series from the `OH` sheet.
///
/// Drops the leading years the workbook leaves blank rather than writing them as zero: 1999-2000
/// through 2004-05 are cells nobody filled, and a zero there would be a count of no home
/// educators in Ohio.
///
/// # Errors
///
/// Returns the heading that has moved, or a cell that is not a whole number of students.
pub fn build_home_education_statewide(rows: &[Vec<String>]) -> Result<Vec<Vec<String>>, String> {
    const FILE: &str = "the Homeschool Hub's OH sheet";
    let head = rows.first().cloned().unwrap_or_default();
    let (year, count) = (
        column(&head, "School Year", FILE)?,
        column(&head, "Series 1", FILE)?,
    );

    let mut out = Vec::new();
    for row in rows.iter().skip(1) {
        let get = |i: usize| row.get(i).map_or("", String::as_str).trim();
        if get(year).is_empty() || get(count).is_empty() {
            continue;
        }
        out.push(vec![get(year).to_string(), whole(get(count), FILE)?]);
    }
    if out.is_empty() {
        return Err(format!("{FILE} carries no years at all"));
    }
    Ok(out)
}

/// Build the per-district extract from the district sheet, keyed by IRN.
///
/// `typology` is the output of [`super::typology::build_typology`] — the department's own
/// district list, which supplies both the IRNs this sheet lacks and the counties it leaves
/// empty. `statewide` is the output of [`build_home_education_statewide`], read only to check
/// the year label against the workbook's own state total.
///
/// # Errors
///
/// Returns an error if a heading has moved, if the sheet carries fewer than 600 Ohio
/// rows or labels them with more than one school year, if any district name fails to resolve to
/// a distinct IRN, or if the district rows no longer contradict the year they are labeled with.
/// Each of those would otherwise produce a plausible fixture rather than a failure, and the last
/// is the one finding here that the file alone establishes.
pub fn build_home_education_districts(
    sheet: &[Vec<String>],
    typology: &[Vec<String>],
    statewide: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    const FILE: &str = "the Homeschool Hub's district sheet";
    let head = sheet.first().cloned().unwrap_or_default();
    let (state, year, name, count) = (
        column(&head, "State", FILE)?,
        column(&head, "Year", FILE)?,
        column(&head, "County / District / Town / LEA", FILE)?,
        column(&head, "HS Enrollment", FILE)?,
    );

    let mut ohio = Vec::new();
    for row in sheet.iter().skip(1) {
        let get = |i: usize| row.get(i).map_or("", String::as_str).trim();
        if get(state) != OHIO {
            continue;
        }
        ohio.push((
            clean_name(get(name)),
            get(count).to_string(),
            get(year).to_string(),
        ));
    }
    if ohio.len() < AT_LEAST {
        return Err(format!(
            "{FILE} carries {} Ohio rows, fewer than the {AT_LEAST} it should",
            ohio.len()
        ));
    }
    let labels: BTreeSet<&str> = ohio.iter().map(|(_, _, y)| y.as_str()).collect();
    let [label] = labels.iter().copied().collect::<Vec<_>>()[..] else {
        return Err(format!(
            "{FILE} labels Ohio's rows with {} school years, not one: {labels:?}",
            labels.len()
        ));
    };

    let districts = district_list(typology)?;
    let irns = resolve(
        &ohio.iter().map(|(n, _, _)| n.clone()).collect::<Vec<_>>(),
        &districts,
    )?;

    let mut out = Vec::with_capacity(ohio.len());
    let mut floor_total = 0u64;
    for (irn, (name, cell, _)) in irns.iter().zip(&ohio) {
        let (district, county) = districts
            .names
            .get(irn)
            .ok_or_else(|| format!("{FILE} resolved {name} to {irn}, which is not a district"))?;
        let (floor, ceiling) = match cell.as_str() {
            // The department's own marker for a cell it masked. Nine is the largest a cell
            // under ten can be, which is the same bound `nonpublic` reads off the same rule.
            "<10" => (0u64, "9".to_string()),
            // Unbounded above by anything in the file. `.` is nine other states' blank and `NC`
            // is Ohio's alone, and neither the codebook nor the sheet says what either means.
            "." | "NC" => (0, String::new()),
            reported => {
                let students = whole(reported, FILE)?;
                let value: u64 = students.parse().map_err(|_| {
                    format!("{FILE} gives {name} the count {reported:?}, which is not a number")
                })?;
                (value, students)
            }
        };
        floor_total += floor;
        out.push(vec![
            irn.clone(),
            district.clone(),
            county.clone(),
            label.to_string(),
            published(cell, FILE)?,
            floor.to_string(),
            ceiling,
        ]);
    }

    refute_the_label(label, floor_total, statewide, FILE)?;
    out.sort_by(|a, b| a[0].cmp(&b[0]));
    Ok(out)
}

/// The workbook's own state total for the year its district rows claim to be, against the sum
/// of those rows.
///
/// A floor against a total: every censored cell is at least zero, so the numeric rows alone are
/// the least the district sheet can come to. If that floor is above the state total for the year
/// on the label, the two sheets of one workbook contradict each other and the label is the one
/// that gives — which is the whole reason these rows can be committed at all.
///
/// This is an assertion that can fail. It reads two sheets that are only pinned together by
/// being in the same file, and the label is the aggregator's rather than the department's.
fn refute_the_label(
    label: &str,
    floor_total: u64,
    statewide: &[Vec<String>],
    file: &str,
) -> Result<(), String> {
    let Some(labeled) = school_year(label) else {
        return Err(format!(
            "{file} labels Ohio's rows {label:?}, which is not a school year"
        ));
    };
    let published: u64 = statewide
        .iter()
        .find(|row| row.first().is_some_and(|y| *y == labeled))
        .and_then(|row| row.get(1))
        .ok_or_else(|| format!("{file} is labeled {label} and the OH sheet has no {labeled}"))?
        .parse()
        .map_err(|_| format!("the OH sheet's {labeled} total is not a number"))?;
    if floor_total <= published {
        return Err(format!(
            "{file} sums to at least {floor_total} against the same workbook's {labeled} total of \
             {published}, so the rows no longer contradict their own label — the reason they are \
             committed without one has gone, and the year needs establishing again"
        ));
    }
    Ok(())
}

/// Expand the workbook's `SY21-22` into the `2021-2022` its state sheets are keyed by.
///
/// Two-digit years, so a century has to be assumed; the series starts in 1999-2000 and this
/// reads anything under 90 as 2000s, which is wrong in 2090 and right until then.
fn school_year(label: &str) -> Option<String> {
    let (first, second) = label.strip_prefix("SY")?.split_once('-')?;
    let century = |two: &str| -> Option<u32> {
        let n: u32 = two.parse().ok()?;
        Some(if n < 90 { 2000 + n } else { 1900 + n })
    };
    Some(format!("{}-{}", century(first)?, century(second)?))
}

/// The department's district list, keyed by IRN and indexed by normalized name.
struct Districts {
    /// IRN to the department's spelling of the name and the county.
    names: BTreeMap<String, (String, String)>,
    /// Normalized name to every IRN carrying it, in IRN order.
    by_name: BTreeMap<String, Vec<String>>,
}

/// Index the typology extract by name and by IRN.
fn district_list(typology: &[Vec<String>]) -> Result<Districts, String> {
    let mut names = BTreeMap::new();
    let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in typology {
        let [irn, district, county, ..] = &row[..] else {
            return Err("the typology extract has fewer than three columns".to_string());
        };
        by_name
            .entry(normalize(district))
            .or_default()
            .push(irn.clone());
        names.insert(irn.clone(), (district.clone(), county.clone()));
    }
    if names.len() < AT_LEAST {
        return Err(format!(
            "the typology extract carries {} districts, too few to resolve a name against",
            names.len()
        ));
    }
    for irns in by_name.values_mut() {
        irns.sort();
    }
    Ok(Districts { names, by_name })
}

/// Resolve each row of the district sheet to an IRN, on the sheet's own ordering.
///
/// Three passes, in decreasing strength. A name unique in the department's list resolves
/// outright. A repeated name inside the sheet's monotone region is bounded by the nearest
/// unique names either side of it, which is only sound because the rows are IRN-ordered — so the
/// region is measured rather than assumed, as the longest strictly increasing run of those
/// unique names, and rows outside it get no positional bound at all. Whatever is left resolves
/// by elimination against the IRNs already taken, to a fixed point.
///
/// # Errors
///
/// Returns the rows that did not resolve, or the IRN that two rows both claim. Either means the
/// two publishers' lists have moved apart and the join is no longer safe.
fn resolve(names: &[String], districts: &Districts) -> Result<Vec<String>, String> {
    let empty: Vec<String> = Vec::new();
    let candidates: Vec<&Vec<String>> = names
        .iter()
        .map(|name| districts.by_name.get(&key_for(name)).unwrap_or(&empty))
        .collect();

    let mut resolved: Vec<Option<String>> = vec![None; names.len()];
    let anchors: Vec<(usize, &String)> = candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| c.len() == 1)
        .map(|(i, c)| (i, &c[0]))
        .collect();
    for (i, irn) in &anchors {
        resolved[*i] = Some((*irn).clone());
    }

    // The longest run of consecutive anchors that strictly increases in IRN. The sheet is one
    // such run of 558 with four rows appended after it, and taking the longest rather than the
    // first is what keeps the appended tail from bounding anything.
    let (mut best, mut start) = ((0usize, 0usize), 0usize);
    for k in 0..anchors.len() {
        if k > 0 && anchors[k].1 <= anchors[k - 1].1 {
            start = k;
        }
        if k + 1 - start > best.1 - best.0 {
            best = (start, k + 1);
        }
    }
    let ordered = &anchors[best.0..best.1];

    if let (Some(first), Some(last)) = (ordered.first(), ordered.last()) {
        for i in first.0..=last.0 {
            if resolved[i].is_some() || candidates[i].is_empty() {
                continue;
            }
            let low = ordered
                .iter()
                .filter(|(j, _)| *j < i)
                .map(|(_, v)| *v)
                .next_back();
            let high = ordered.iter().find(|(j, _)| *j > i).map(|(_, v)| v);
            let mut within = candidates[i]
                .iter()
                .filter(|c| low.is_none_or(|l| *c > l) && high.is_none_or(|h| *c < h));
            if let (Some(only), None) = (within.next(), within.next()) {
                resolved[i] = Some(only.clone());
            }
        }
    }

    loop {
        let taken: BTreeSet<&String> = resolved.iter().flatten().collect();
        let mut settled = None;
        for (i, options) in candidates.iter().enumerate() {
            if resolved[i].is_some() {
                continue;
            }
            let mut free = options.iter().filter(|c| !taken.contains(c));
            if let (Some(only), None) = (free.next(), free.next()) {
                settled = Some((i, only.clone()));
                break;
            }
        }
        match settled {
            Some((i, irn)) => resolved[i] = Some(irn),
            None => break,
        }
    }

    let unresolved: Vec<&str> = resolved
        .iter()
        .enumerate()
        .filter(|(_, r)| r.is_none())
        .map(|(i, _)| names[i].as_str())
        .collect();
    if !unresolved.is_empty() {
        return Err(format!(
            "{} rows of the Homeschool Hub sheet resolve to no single district IRN: \
             {unresolved:?}",
            unresolved.len()
        ));
    }
    let out: Vec<String> = resolved.into_iter().flatten().collect();
    let distinct: BTreeSet<&String> = out.iter().collect();
    if distinct.len() != out.len() {
        return Err(format!(
            "the Homeschool Hub sheet's {} rows resolve to only {} distinct IRNs",
            out.len(),
            distinct.len()
        ));
    }
    Ok(out)
}

/// A cell the workbook writes as a float and this fixture writes as a count of students.
fn whole(raw: &str, file: &str) -> Result<String, String> {
    let value: f64 = raw
        .parse()
        .map_err(|_| format!("{file} has the count {raw:?}, which is not a number"))?;
    if value < 0.0 || value.fract() != 0.0 {
        return Err(format!(
            "{file} has the count {raw:?}, which is not a whole number of students"
        ));
    }
    Ok(format!("{value:.0}"))
}

/// The published cell, with the workbook's trailing `.0` taken off a count and a marker left
/// exactly as written.
fn published(cell: &str, file: &str) -> Result<String, String> {
    match cell {
        "<10" | "." | "NC" => Ok(cell.to_string()),
        reported => whole(reported, file),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One Ohio row, and the department district it should resolve to.
    struct Row {
        irn: u32,
        /// How the department's list spells the name — with a parenthesised county where the
        /// name repeats, which is the case this join exists to handle.
        listed: &'static str,
        /// How the Hub's sheet spells it.
        workbook: &'static str,
        /// The published cell.
        cell: &'static str,
    }

    /// Rows enough to clear [`AT_LEAST`], each with a name unique in the list and an IRN above
    /// anything a test writes by hand, so a test only has to state the case it is about.
    fn filler(count: usize) -> Vec<Row> {
        (0..count)
            .map(|i| Row {
                irn: 100_000 + u32::try_from(i).unwrap_or_default() * 100,
                listed: "Filler Local",
                workbook: "Filler Local",
                cell: "1",
            })
            .collect()
    }

    /// The three sheets [`build_home_education_districts`] takes: hub rows, districts, statewide.
    type Corpus = (Vec<Vec<String>>, Vec<Vec<String>>, Vec<Vec<String>>);

    /// The three inputs [`build_home_education_districts`] takes, from one list of rows.
    ///
    /// The sheet is in the order given, so a test can put a row out of IRN order and see what
    /// that does; the district list is always sorted, because the department's is.
    fn corpus(rows: &[Row], label: &str, statewide: &[(&str, &str)]) -> Corpus {
        let mut sheet = vec![vec![
            "State".to_string(),
            "Year".to_string(),
            "County / District / Town / LEA".to_string(),
            "HS Enrollment".to_string(),
        ]];
        // A row of another state's, to prove the filter is on the column and not the sheet.
        sheet.push(vec![
            "AR".to_string(),
            label.to_string(),
            "Pulaski County".to_string(),
            "9999".to_string(),
        ]);
        let mut listed: Vec<(u32, &str)> = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            let name = if row.listed == "Filler Local" {
                Box::leak(format!("Filler {i:04} Local").into_boxed_str())
            } else {
                row.listed
            };
            let written = if row.workbook == "Filler Local" {
                Box::leak(format!("Filler {i:04} Local").into_boxed_str())
            } else {
                row.workbook
            };
            sheet.push(vec![
                OHIO.to_string(),
                label.to_string(),
                written.to_string(),
                row.cell.to_string(),
            ]);
            listed.push((row.irn, name));
        }
        listed.sort_unstable();
        let typology = listed
            .iter()
            .map(|(irn, name)| {
                vec![
                    format!("{irn:06}"),
                    (*name).to_string(),
                    "Somewhere".to_string(),
                    "5".to_string(),
                ]
            })
            .collect();
        let series = statewide
            .iter()
            .map(|(year, total)| vec![(*year).to_string(), (*total).to_string()])
            .collect();
        (sheet, typology, series)
    }

    /// A label the rows contradict, which is the condition the builder insists on.
    const REFUTED: &[(&str, &str)] = &[("2021-2022", "10")];

    fn built(rows: &[Row]) -> Vec<Vec<String>> {
        let (sheet, typology, statewide) = corpus(rows, "SY21-22", REFUTED);
        build_home_education_districts(&sheet, &typology, &statewide).expect("the corpus builds")
    }

    #[test]
    fn a_repeated_name_resolves_to_the_district_its_neighbours_bound() {
        let mut rows = filler(AT_LEAST);
        // Two districts called Perry Local, and a sheet row between two fillers that can only be
        // the first of them. The department's list disambiguates by county; the sheet does not.
        rows.insert(
            10,
            Row {
                irn: 100_950,
                listed: "Perry Local (Allen)",
                workbook: "Perry Local",
                cell: "15",
            },
        );
        rows.push(Row {
            irn: 900_000,
            listed: "Perry Local (Stark)",
            workbook: "Perry Local",
            cell: "87",
        });
        let out = built(&rows);
        let perry: Vec<&Vec<String>> = out.iter().filter(|r| r[1].starts_with("Perry")).collect();
        assert_eq!(perry.len(), 2, "both Perry Locals are in the panel");
        assert_eq!(
            (perry[0][0].as_str(), perry[0][4].as_str()),
            ("100950", "15"),
            "the row between the fillers took the IRN they bound it to"
        );
        assert_eq!(
            (perry[1][0].as_str(), perry[1][4].as_str()),
            ("900000", "87")
        );
    }

    #[test]
    fn a_row_appended_after_the_ordered_region_does_not_borrow_its_bounds() {
        // The sheet's real shape: one long IRN-ordered run, then four rows tacked on at the end
        // in no order. A repeated name in that tail sits between two anchors that would bound it
        // to an IRN the ordered region has already taken, and positional reasoning there is
        // simply wrong — the appended row is the *other* one, and only elimination says so.
        let mut rows = filler(AT_LEAST);
        rows.insert(
            10,
            Row {
                irn: 100_950,
                listed: "Buckeye Local (Ashtabula)",
                workbook: "Buckeye Local",
                cell: "44",
            },
        );
        // The tail, in the shape the sheet has it: out of the main run's order, and with two
        // anchors of its own that bound the repeated name between them — to 100950, which the
        // ordered region has already taken. Reasoning positionally inside the tail gives the
        // wrong district here, which is why the ordered region is the longest run and not
        // every run.
        rows.push(Row {
            irn: 100_930,
            listed: "Tail Local",
            workbook: "Tail Local",
            cell: "7",
        });
        rows.push(Row {
            irn: 900_000,
            listed: "Buckeye Local (Medina)",
            workbook: "Buckeye Local",
            cell: "91",
        });
        rows.push(Row {
            irn: 101_020,
            listed: "Tail Two Local",
            workbook: "Tail Two Local",
            cell: "3",
        });
        let out = built(&rows);
        let buckeye: Vec<&Vec<String>> =
            out.iter().filter(|r| r[1].starts_with("Buckeye")).collect();
        assert_eq!(
            (buckeye[0][0].as_str(), buckeye[0][4].as_str()),
            ("100950", "44")
        );
        assert_eq!(
            (buckeye[1][0].as_str(), buckeye[1][4].as_str()),
            ("900000", "91"),
            "the appended row is the district the ordered region did not take"
        );
    }

    #[test]
    fn a_name_that_resolves_to_no_district_is_an_error() {
        let mut rows = filler(AT_LEAST);
        rows[3].workbook = "A District The Department Has Never Heard Of";
        let (sheet, typology, statewide) = corpus(&rows, "SY21-22", REFUTED);
        let error = build_home_education_districts(&sheet, &typology, &statewide)
            .expect_err("an unresolvable name is fatal");
        assert!(error.contains("no single district IRN"), "{error}");
    }

    #[test]
    fn the_three_markers_bound_a_cell_differently() {
        let mut rows = filler(AT_LEAST);
        rows[0].cell = "<10";
        rows[1].cell = ".";
        rows[2].cell = "NC";
        rows[3].cell = "0";
        let out = built(&rows);
        let bounds = |published: &str| -> (String, String) {
            let row = out
                .iter()
                .find(|r| r[4] == published)
                .unwrap_or_else(|| panic!("no row published as {published}"));
            (row[5].clone(), row[6].clone())
        };
        // Under ten is the only marker the file bounds above.
        assert_eq!(bounds("<10"), ("0".to_string(), "9".to_string()));
        // Nine other states use `.`, and `NC` is Ohio's alone. Neither is documented, and
        // nothing in the file caps either — Southwest Local is 4,150 pupils and reads `NC`.
        assert_eq!(bounds("."), ("0".to_string(), String::new()));
        assert_eq!(bounds("NC"), ("0".to_string(), String::new()));
        // An explicit zero is not a censored cell, and the pair says so: bounded both ways.
        assert_eq!(bounds("0"), ("0".to_string(), "0".to_string()));
    }

    #[test]
    fn a_label_the_rows_do_not_contradict_is_an_error() {
        // The whole licence for committing rows with no year of their own is that the label is
        // refuted by the workbook against itself. If a revision ever makes the two sheets agree,
        // that licence has gone, and this must fail rather than quietly ship a dated panel.
        let rows = filler(AT_LEAST);
        let (sheet, typology, statewide) = corpus(&rows, "SY21-22", &[("2021-2022", "999999")]);
        let error = build_home_education_districts(&sheet, &typology, &statewide)
            .expect_err("an uncontradicted label is fatal");
        assert!(
            error.contains("no longer contradict their own label"),
            "{error}"
        );
    }

    #[test]
    fn two_school_years_in_one_state_block_is_an_error() {
        let rows = filler(AT_LEAST);
        let (mut sheet, typology, statewide) = corpus(&rows, "SY21-22", REFUTED);
        sheet[5][1] = "SY22-23".to_string();
        let error = build_home_education_districts(&sheet, &typology, &statewide)
            .expect_err("two labels on one block is fatal");
        assert!(error.contains("school years, not one"), "{error}");
    }

    #[test]
    fn only_ohio_rows_are_read() {
        let out = built(&filler(AT_LEAST));
        assert_eq!(out.len(), AT_LEAST, "the Arkansas row is not in the panel");
    }

    #[test]
    fn the_statewide_series_drops_the_years_nobody_filled() {
        let rows = vec![
            vec!["School Year".to_string(), "Series 1".to_string()],
            vec!["1999-2000".to_string(), String::new()],
            vec!["2005-2006".to_string(), "25937.0".to_string()],
            vec!["2023-2024".to_string(), "53051.0".to_string()],
        ];
        let out = build_home_education_statewide(&rows).expect("the series builds");
        assert_eq!(
            out,
            vec![
                vec!["2005-2006".to_string(), "25937".to_string()],
                vec!["2023-2024".to_string(), "53051".to_string()],
            ],
            "a blank year is dropped rather than written as no home educators at all"
        );
    }

    #[test]
    fn the_publishers_four_spellings_of_a_suffix_are_one_name() {
        for spelling in [
            "Eastern Local School District",
            "Eastern Local Schools",
            "Eastern Local SD",
            "Eastern  Local",
            "Eastern Local (Brown)",
        ] {
            assert_eq!(normalize(spelling), "eastern local", "{spelling}");
        }
        // Six spellings are a district's common name rather than its legal one, and no
        // normalizer safe enough to keep the other 605 apart will join them.
        assert_eq!(key_for("Cincinnati Public Schools"), "cincinnati city");
        assert_eq!(
            key_for("Adams County Ohio Valley Local"),
            "adams county/ohio valley local"
        );
    }

    #[test]
    fn a_two_digit_label_expands_to_the_year_the_state_sheets_are_keyed_by() {
        assert_eq!(school_year("SY21-22").as_deref(), Some("2021-2022"));
        assert_eq!(school_year("SY99-00").as_deref(), Some("1999-2000"));
        assert_eq!(school_year("2021-22"), None);
    }
}
