//! The NCES Common Core of Data LEA directory, longitudinal.
//!
//! Thirty years of one directory in two formats — fourteen fixed-width years to 2007-08 and
//! delimited years after — reduced to the NCES-to-IRN map that every federal join in this corpus
//! needs. The fixed-width layouts are pinned as data because one published layout document states
//! a column position wrongly, twice, and this repository takes the position from the file.

use std::collections::BTreeMap;

use super::delimited::{column, delimited_fields};
use super::format::clean_name;
use super::text::fixed;

/// Columns of the directory panel.
pub const CCD_DIRECTORY_HEADER: &[&str] = &[
    "school_year",
    "leaid",
    "irn",
    "name",
    "agency_type",
    "status",
];

/// One published year of the LEA universe directory.
#[derive(Debug, Clone, Copy)]
pub struct DirectoryYear<'a> {
    /// The school year it describes, as the calendar year it begins in: 2008 for 2008-09.
    pub opens: u16,
    /// The delimited file's text.
    pub text: &'a str,
}

/// Column names that mean the same thing in different years, most recent naming first.
///
/// Resolved by trying each in turn rather than by a per-year table. The year suffix is the reason:
/// 2008-09 calls it `STID08` and 2009-10 calls it `STID09`, so a table would carry one row per
/// year to say one thing. A list of aliases says the same thing once and fails loudly if a year
/// uses none of them.
const CCD_ALIASES: [(&str, &[&str]); 5] = [
    ("the agency identifier", &["LEAID"]),
    (
        "the state's own identifier",
        &["ST_LEAID", "STID", "STID09", "STID08", "STID07"],
    ),
    (
        "the agency name",
        &["LEA_NAME", "NAME", "NAME09", "NAME08", "NAME07"],
    ),
    (
        "the agency type",
        &["LEA_TYPE", "TYPE", "TYPE09", "TYPE08", "TYPE07"],
    ),
    (
        "the operational status",
        &["SY_STATUS", "BOUND", "BOUND09", "BOUND08", "BOUND07"],
    ),
];

/// Ohio's FIPS state code, which is how a row is selected.
const OHIO_FIPS: &str = "39";

/// One fixed-width year of the directory, by the byte positions its columns occupy.
///
/// Positions are one-based and inclusive, as the layout documents write them, and are converted
/// once at the point of use rather than stored two ways.
#[derive(Debug, Clone, Copy)]
struct FixedWidth {
    /// The school year, as the calendar year it opens in.
    opens: u16,
    /// The record length, every byte of it, excluding the terminator.
    record: usize,
    /// Where the agency name ends. It starts at 22 in every year.
    name_ends: usize,
    /// The single byte holding the agency type.
    agency_type: usize,
    /// The single byte holding the boundary-change status.
    status: usize,
}

/// Every year of the directory that is fixed-width, with where its columns are.
///
/// # Why a per-year table and not a per-era one
///
/// Because the record length moves in seven of the thirteen years and there is no era to speak
/// of. What the table also shows, which the issue that asked for this reader did not expect, is
/// that **almost nothing this reader wants moves with it**. The agency identifier is bytes 1-7 in
/// every year, the state identifier 8-21 in every year, and the name starts at 22 in every year.
/// Two columns move at all: the name's end and the agency type move exactly once, together, in
/// 1998-99; the status moves four times. A reader written per era would still be wrong, and a
/// reader written per year is thirteen lines.
///
/// # The record length is the check, not decoration
///
/// Each length is the `LRECL` its own layout document states, and [`build_ccd_directory`] holds
/// every row to it. That is what makes a wrong table loud: a layout that does not describe the
/// file fails on the first record rather than yielding plausible garbage from the wrong offsets.
///
/// # Where the table disagrees with the publisher
///
/// **2003-04.** Its layout document gives the status column as position 281 and, elsewhere in the
/// same document, as 384. 281 is the metropolitan-statistical-area indicator's position — stated
/// four lines above, for the same year — and 284 is the one byte the document leaves unaccounted
/// for between the locale code at 283 and the low grade span at 285. The neighbouring years put
/// the column at 284. So does this table, and it is not a guess: at 284 the codes are the
/// boundary-change vocabulary and every agency the file flags closed is absent from the following
/// year, which is the property [`build_ccd_directory`] checks across all thirty years. At 281 the
/// column reads `1` or `2` for reasons of geography and the check fails immediately.
const CCD_FIXED_WIDTH: [FixedWidth; 13] = [
    // 1994-95 through 1997-98: the wide record, the 30-character name.
    fixed_width(1994, 1030, 51, 121, 162),
    fixed_width(1995, 1030, 51, 121, 162),
    fixed_width(1996, 1030, 51, 121, 162),
    fixed_width(1997, 1030, 51, 121, 162),
    // 1998-99: the name doubles to 60 characters and everything after it shifts.
    fixed_width(1998, 722, 81, 234, 280),
    fixed_width(1999, 722, 81, 234, 280),
    fixed_width(2000, 723, 81, 234, 281),
    fixed_width(2001, 725, 81, 234, 281),
    fixed_width(2002, 729, 81, 234, 284),
    // 2003-04: 284 against a document that says 281. See the note above.
    fixed_width(2003, 729, 81, 234, 284),
    // 2004-05: the dropout and completer counts leave and the record almost halves.
    fixed_width(2004, 519, 81, 234, 284),
    fixed_width(2005, 519, 81, 234, 284),
    // 2006-07: coordinates and a congressional district arrive ahead of the status column.
    fixed_width(2006, 530, 81, 234, 309),
];

const fn fixed_width(
    opens: u16,
    record: usize,
    name_ends: usize,
    agency_type: usize,
    status: usize,
) -> FixedWidth {
    FixedWidth {
        opens,
        record,
        name_ends,
        agency_type,
        status,
    }
}

/// The layout for a year, if that year is one of the fixed-width ones.
fn fixed_width_of(opens: u16) -> Option<FixedWidth> {
    CCD_FIXED_WIDTH.into_iter().find(|w| w.opens == opens)
}

/// The Ohio rows of one fixed-width year: agency, state identifier, name, type, status.
///
/// Sliced by character rather than by byte. The archives are read Latin-1 byte-for-byte, so a
/// record holding one accented character is one byte longer in memory than it is on disk, and
/// byte offsets would walk off the columns from that record onward.
fn ccd_fixed_rows(layout: FixedWidth, text: &str, label: &str) -> Result<Vec<[String; 5]>, String> {
    let mut out = Vec::new();
    for (at, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        if chars.len() != layout.record {
            return Err(format!(
                "{label} record {} is {} bytes and its layout says {}; the layout does not \
                 describe this file",
                at + 1,
                chars.len(),
                layout.record
            ));
        }
        if fixed(&chars, 0, 2) != OHIO_FIPS {
            continue;
        }
        out.push([
            fixed(&chars, 0, 7),
            fixed(&chars, 7, 21),
            fixed(&chars, 21, layout.name_ends),
            fixed(&chars, layout.agency_type - 1, layout.agency_type),
            fixed(&chars, layout.status - 1, layout.status),
        ]);
    }
    Ok(out)
}

/// Every Ohio agency in every published year of the directory this repository holds.
///
/// # Why more than one year of a directory is worth holding
///
/// One year answers "what is this agency's Ohio number". Thirty answer "when did this agency
/// exist", which is a different question and the one a panel spanning years actually asks.
/// [`crate::fixtures::F33_OHIO_PANEL_FIXTURE`] resolved every identifier through the 2022-23 file
/// alone, so an agency that closed before 2023 had no Ohio number at all — 124 of them in FY2012 —
/// and the module reading it described that count as the consolidation history. It is not. Of the
/// 689 agencies that leave this window, **616 are community schools**, 66 are service agencies,
/// and five are regular districts.
///
/// # Two eras of file, one set of rows
///
/// 2007-08 onward is delimited with a header and is read by the column names in `CCD_ALIASES`.
/// 1994-95 through 2006-07 is fixed-width with no header at all and is read by byte position
/// through a per-year byte-position table. The extension does not decide which: `ag031b.txt` is fixed-width
/// and `ag071b.txt` is delimited, and the year does.
///
/// # What the status column can and cannot say
///
/// The CCD vocabulary has eight operational-status codes and exactly one of them marks a
/// consolidation: code 5, *"significant change in geographic boundaries or instructional
/// responsibility"*. **Ohio has never used it** — zero occurrences in every Ohio agency-year this
/// reader covers, which is now every one the directory has published since 1994-95. What Ohio
/// files instead, for all 689 departures without exception, is code 2: *"closed with no effect on
/// another agency's boundaries"*.
///
/// That is not silence. It is the negation of the thing, filed about three districts whose
/// territory demonstrably went to a neighbour — and, thirty-nine more times in the 1990s, about
/// the county boards of education that became today's educational service centers. So the code is
/// carried verbatim and nothing here reads a reason out of it. See `dispersion::lea_directory`.
///
/// # Ohio is selected on `FIPST` and never on `LSTATE`
///
/// They disagree. LEAID 3901497, Urban Pathways of Youngstown, is filed under `FIPST=39` with
/// `LSTATE=PA` in 2012-13 and 2013-14 — a mailing address, not a jurisdiction. Earlier years have
/// the mirror-image defect, filing an Arizona agency as `LSTATE=OH`. The FIPS code is part of the
/// agency identifier's first two digits and cannot drift from it, and in the fixed-width years it
/// is the only way in: those files have no state column of their own.
///
/// # Errors
///
/// Returns a description if a year uses none of the names a field is known by, if a fixed-width
/// year holds a record of the wrong length, if a year's Ohio row count is outside the band the
/// survey has ever produced, or if an agency leaves the register without having been filed closed
/// — which is the check that settles the one year whose layout document is wrong.
pub fn build_ccd_directory(years: &[DirectoryYear<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for year in years {
        let label = format!(
            "the CCD LEA directory for {}-{:02}",
            year.opens,
            (year.opens + 1) % 100
        );
        let rows = match fixed_width_of(year.opens) {
            Some(layout) => ccd_fixed_rows(layout, year.text, &label)?,
            None => ccd_delimited_rows(year.text, &label)?,
        };

        let kept = rows.len();
        for row in rows {
            // The prefix appears in 2016-17 and the digits either side of it are the same six.
            // Stripped here so the panel joins to itself across the change, and so it joins to
            // every other fixture in this repository, which write the bare IRN.
            let irn = row[1].trim_start_matches("OH-");
            if irn.len() != 6 || !irn.chars().all(|c| c.is_ascii_digit()) {
                return Err(format!(
                    "{label} gives agency {} the state identifier {irn:?}, which is not a \
                     six-digit IRN",
                    row[0]
                ));
            }
            out.push(vec![
                year.opens.to_string(),
                row[0].clone(),
                irn.to_string(),
                clean_name(&row[2]),
                row[3].clone(),
                row[4].clone(),
            ]);
        }

        // Ohio has run between seven hundred and twelve hundred agencies in every year the
        // directory has published, the low end being 1996-97 before community schools existed and
        // the high end 2005-06 at the top of the charter opening wave. A count outside that band
        // is a state filter that matched the wrong column, which is the failure `LSTATE` would
        // have produced silently.
        if !(700..=1400).contains(&kept) {
            return Err(format!(
                "{label} yielded {kept} Ohio agencies, which is outside anything the directory \
                 has published"
            ));
        }
    }
    check_ccd_closures(&out)?;
    Ok(out)
}

/// The Ohio rows of one delimited year, by the names its header gives the columns.
fn ccd_delimited_rows(text: &str, label: &str) -> Result<Vec<[String; 5]>, String> {
    let mut lines = text.lines();
    let header_line = lines.next().unwrap_or_default();
    let delimiter = if header_line.contains('\t') {
        '\t'
    } else {
        ','
    };
    let head = delimited_fields(header_line, delimiter);

    let mut at = [0usize; CCD_ALIASES.len()];
    for (i, (what, names)) in CCD_ALIASES.iter().enumerate() {
        at[i] = names
            .iter()
            .find_map(|name| column(&head, name, label).ok())
            .ok_or_else(|| {
                format!("{label} names {what} none of {names:?}; its layout has moved")
            })?;
    }
    let fips = column(&head, "FIPST", label)?;

    let mut out = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f = delimited_fields(line, delimiter);
        let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
        if field(fips).trim_start_matches('0') != OHIO_FIPS {
            continue;
        }
        out.push([
            field(at[0]).to_string(),
            field(at[1]).to_string(),
            field(at[2]).to_string(),
            field(at[3]).to_string(),
            field(at[4]).to_string(),
        ]);
    }
    Ok(out)
}

/// Every agency that stops appearing was flagged closed in the last year that names it.
///
/// # Why this is the check that matters
///
/// The status column is one byte, its position moves five times, and reading it one byte off
/// yields a column of plausible small integers rather than an error. Nothing about a single
/// year's rows can catch that. What can is the column's relationship to the *next* year's
/// membership: code 2 means *"education agency has closed"*, so an agency carrying it should not
/// be in the following year's file, and one that vanishes without it would be an agency the
/// register lost track of rather than closed.
///
/// Across the twenty-nine transitions from 1994-95 to 2023-24 the containment holds exactly —
/// every departure is flagged, without a single exception in thirty years. The converse very
/// nearly holds too: two agencies are flagged closed and come back, both community schools, and
/// both are then filed under code 8, *"closed on previous year's file but has reopened"*. So the
/// vocabulary accounts for its own exceptions and this reader checks the direction that has none.
///
/// This is also what settles 2003-04's column against its own layout document. At the documented
/// position the check fails on that year's transition; at 284 it passes.
fn check_ccd_closures(rows: &[Vec<String>]) -> Result<(), String> {
    const CLOSED: &str = "2";
    let mut by_year: BTreeMap<u16, BTreeMap<&str, &str>> = BTreeMap::new();
    for row in rows {
        let Ok(opens) = row[0].parse::<u16>() else {
            continue;
        };
        by_year
            .entry(opens)
            .or_default()
            .insert(row[1].as_str(), row[5].as_str());
    }
    let years: Vec<u16> = by_year.keys().copied().collect();
    for pair in years.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        // Only consecutive years can say anything about each other. A gap in what is held is not
        // a claim about the register.
        if after != before + 1 {
            continue;
        }
        let gone: Vec<&str> = by_year[&before]
            .keys()
            .filter(|leaid| !by_year[&after].contains_key(*leaid))
            .filter(|leaid| by_year[&before][*leaid] != CLOSED)
            .copied()
            .collect();
        if !gone.is_empty() {
            return Err(format!(
                "{} agencies are absent from {after}-{:02} without {before}-{:02} filing them \
                 closed, the first being {}; the status column is being read at the wrong \
                 position in one of those two years",
                gone.len(),
                (after + 1) % 100,
                (before + 1) % 100,
                gone[0]
            ));
        }
    }
    Ok(())
}
