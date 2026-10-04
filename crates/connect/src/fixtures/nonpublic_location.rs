//! The school district each chartered nonpublic school sits in, from its address.
//!
//! R.C. 3317.024(E)(1) pays auxiliary services through the district a chartered nonpublic school
//! is located in, and none of the department's October files says which district that is. The
//! building panel carries an IRN, a name and, before 2017, a county. A join on name was tried and
//! rejected: 711 schools carry 601 names, sixteen of them `St Mary`.
//!
//! Three publications answer it between them, joined on keys rather than names:
//!
//! 1. The department's directory (OEDS) lists every open nonpublic school by building IRN with
//!    a mailing address. It names a parent, which is the diocese or association, never the public
//!    district — the directory has no field for that.
//! 2. The Census Bureau's batch geocoder places each address in a 2020 census block.
//! 3. The 2020 block assignment file, already read for the legislative crosswalk, says which
//!    unified school district each block lay in. The CCD directory turns that district's LEAID
//!    into an IRN.
//!
//! So no boundary file is added: the blocks are the ones [`super::build_legislative_crosswalk`]
//! already apportions.
//!
//! # A snapshot, not a series
//!
//! The directory lists the schools open on the day it was generated, at the address they have on
//! that day. The fixture says so in every row rather than in a note: a building that moved, or
//! closed before the extract, is not in it, and nothing here can be projected backward onto an
//! October the directory did not describe.
//!
//! # The block is 2020's, and so is the district
//!
//! The district is the one the block lay in at the 2020 census. Newbury Local was absorbed into
//! West Geauga on July 1, 2020, after the boundaries the block file records, so a school in
//! Newbury geocodes to Newbury's code. That is written as found, with the last directory year that
//! carried the code beside it, and the transfer is resolved by whoever reads the fixture against
//! the territory-transfer record — a builder that silently rewrote it would hide the one row where
//! the census and the present disagree.

use std::collections::BTreeMap;

use super::delimited::{column, delimited_fields};
use super::format::clean_name;

/// The fixture's columns.
pub const NONPUBLIC_LOCATION_HEADER: &[&str] = &[
    "irn",
    "school",
    "county",
    "geocoder_match",
    "block",
    "district_irn",
    "district",
    "directory_year",
];

/// One open nonpublic school, as the directory lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorySchool {
    /// Six-digit building IRN.
    pub irn: String,
    /// The organisation name, cleaned of commas.
    pub name: String,
    /// The designated county.
    pub county: String,
    /// The mailing address as published: `street, City, Ohio, ZIP`.
    pub address: String,
}

/// Parse the directory extract into one school per IRN.
///
/// The extract opens with a `Generated on …` line ahead of the header, and writes every IRN as an
/// Excel formula (`="057539"`) so a spreadsheet keeps its leading zero. It also lists a school
/// once per school type: 903 rows for 787 buildings when this was written. The duplicates carry
/// the same name and address, so they collapse; if they ever disagree that is an error, because
/// picking one would be choosing an address.
///
/// # Errors
///
/// Returns an error if the header has moved, or two rows for one IRN disagree.
pub fn parse_directory(text: &str) -> Result<Vec<DirectorySchool>, String> {
    const FILE: &str = "the OEDS nonpublic extract";
    let mut lines = text
        .lines()
        .skip_while(|line| line.starts_with("Generated on"));
    let header = delimited_fields(lines.next().ok_or("the extract is empty")?, ',');
    let irn = column(&header, "IRN", FILE)?;
    let name = column(&header, "ORGANIZATION NAME", FILE)?;
    let county = column(&header, "DESIGNATED COUNTY", FILE)?;
    let address = column(&header, "ORG MAILING ADDRESS", FILE)?;
    let status = column(&header, "STATUS", FILE)?;

    let mut schools: BTreeMap<String, DirectorySchool> = BTreeMap::new();
    for line in lines.filter(|line| !line.trim().is_empty()) {
        let fields = delimited_fields(line, ',');
        if fields.len() != header.len() {
            return Err(format!(
                "{FILE}: a row has {} fields against a header of {}: {line}",
                fields.len(),
                header.len()
            ));
        }
        if fields[status] != "Open" {
            return Err(format!(
                "{FILE}: a row has status {:?}; the extract was requested for open schools",
                fields[status]
            ));
        }
        let school = DirectorySchool {
            irn: fields[irn].trim_start_matches('=').to_string(),
            name: clean_name(&fields[name]),
            county: clean_name(&fields[county]),
            address: fields[address].trim().to_string(),
        };
        if school.irn.len() != 6 || !school.irn.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("{FILE}: {:?} is not a building IRN", school.irn));
        }
        match schools.get(&school.irn) {
            Some(seen) if *seen != school => {
                return Err(format!(
                    "{FILE}: IRN {} is listed twice with different details",
                    school.irn
                ));
            }
            Some(_) => {}
            None => {
                schools.insert(school.irn.clone(), school);
            }
        }
    }
    Ok(schools.into_values().collect())
}

/// Split a published address into street, city, state and ZIP.
///
/// From the right, because the street is the only part that can itself hold a comma
/// (`1 Main St, Suite 2`).
fn address_parts(address: &str) -> Option<[String; 4]> {
    let mut parts = address.rsplitn(4, ',').map(str::trim);
    let zip = parts.next()?;
    let state = parts.next()?;
    let city = parts.next()?;
    let street = parts.next()?;
    Some([street, city, state, zip].map(str::to_string))
}

/// The address file the Census batch geocoder takes: `id,street,city,state,zip`, every field
/// quoted.
///
/// # Errors
///
/// Returns an error if the extract cannot be parsed, or an address has fewer than four parts.
pub fn geocoder_addresses(extract: &str) -> Result<String, String> {
    let mut out = String::new();
    for school in parse_directory(extract)? {
        let parts = address_parts(&school.address).ok_or_else(|| {
            format!(
                "{} has an address without city, state and ZIP: {:?}",
                school.irn, school.address
            )
        })?;
        let quoted: Vec<String> = std::iter::once(&school.irn)
            .chain(&parts)
            .map(|field| format!("\"{}\"", field.replace('"', "\"\"")))
            .collect();
        out.push_str(&quoted.join(","));
        out.push('\n');
    }
    Ok(out)
}

/// What the geocoder said about one address.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Geocoded {
    /// `Exact`, `Non_Exact`, `No_Match` or `Tie`.
    outcome: String,
    /// The fifteen-digit 2020 block, when it matched.
    block: Option<String>,
}

/// Parse the batch geocoder's response, keyed by the IDs it was sent.
///
/// The response has no header and does not come back in the order it was sent. A match is twelve
/// fields ending in state, county, tract and block; a miss or a tie is three.
fn parse_geocoded(text: &str) -> Result<BTreeMap<String, Geocoded>, String> {
    let mut out = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let fields = delimited_fields(line, ',');
        let geocoded = match fields.get(2).map(String::as_str) {
            Some("Match") if fields.len() == 12 => Geocoded {
                outcome: fields[3].clone(),
                block: Some(fields[8..12].concat()),
            },
            Some(outcome @ ("No_Match" | "Tie")) => Geocoded {
                outcome: outcome.to_string(),
                block: None,
            },
            _ => return Err(format!("the geocoder answered a line this cannot read: {line}")),
        };
        if let Some(block) = &geocoded.block {
            if block.len() != 15 || !block.starts_with("39") {
                return Err(format!("{block} is not an Ohio 2020 block"));
            }
        }
        if out.insert(fields[0].clone(), geocoded).is_some() {
            return Err(format!("the geocoder answered {} twice", fields[0]));
        }
    }
    Ok(out)
}

/// The inputs to [`build_nonpublic_locations`], named because three of them are text.
#[derive(Debug, Clone, Copy)]
pub struct Locations<'a> {
    /// The OEDS extract, as fetched.
    pub directory: &'a str,
    /// The batch geocoder's response to [`geocoder_addresses`] of that extract.
    pub geocoded: &'a str,
    /// The `_SDUNI.txt` member of the 2020 block assignment file: `BLOCKID|DISTRICT`.
    pub school_districts: &'a str,
    /// The CCD directory fixture's rows: `school_year,leaid,irn,name,agency_type,status`.
    pub ccd: &'a [Vec<String>],
}

/// One row per open nonpublic school, with the district its block lay in.
///
/// A school the geocoder could not place is kept, with its outcome and blank geography, so the
/// unmatched set is in the fixture rather than only in a count. A placed block that the block file
/// does not carry, or a district code the CCD directory never listed, is an error: neither can
/// happen if the three files describe the same 2020 geography, so either means one of them has
/// changed under this builder.
///
/// # Errors
///
/// Returns an error if any input cannot be parsed, the geocoder answered for a school the
/// directory does not list or left one out, or a join misses.
pub fn build_nonpublic_locations(inputs: &Locations<'_>) -> Result<Vec<Vec<String>>, String> {
    let schools = parse_directory(inputs.directory)?;
    let mut geocoded = parse_geocoded(inputs.geocoded)?;

    let districts: BTreeMap<&str, &str> = inputs
        .school_districts
        .lines()
        .skip(1)
        .filter_map(|line| line.split_once('|'))
        .collect();

    // The latest directory year carrying each LEAID, so a district that has since closed is
    // still named, with the year that says it closed.
    let mut leas: BTreeMap<&str, (&str, &str, &str)> = BTreeMap::new();
    for row in inputs.ccd {
        let [year, leaid, irn, name, ..] = &row[..] else {
            return Err(format!("a CCD directory row is short: {row:?}"));
        };
        let entry = leas.entry(leaid).or_insert((year, irn, name));
        if year.as_str() >= entry.0 {
            *entry = (year, irn, name);
        }
    }

    let mut rows = Vec::with_capacity(schools.len());
    for school in schools {
        let found = geocoded.remove(&school.irn).ok_or_else(|| {
            format!(
                "the geocoder did not answer for {}; was it run on this extract?",
                school.irn
            )
        })?;
        let (block, district_irn, district, year) = match &found.block {
            None => Default::default(),
            Some(block) => {
                let code = districts.get(block.as_str()).ok_or_else(|| {
                    format!(
                        "{} geocoded to block {block}, which the block file does not carry",
                        school.irn
                    )
                })?;
                let leaid = format!("39{code}");
                let (year, irn, name) = leas.get(leaid.as_str()).ok_or_else(|| {
                    format!("block {block} lies in district {code}, which no CCD year lists")
                })?;
                (
                    block.clone(),
                    (*irn).to_string(),
                    (*name).to_string(),
                    (*year).to_string(),
                )
            }
        };
        rows.push(vec![
            school.irn,
            school.name,
            school.county,
            found.outcome,
            block,
            district_irn,
            district,
            year,
        ]);
    }
    if let Some(stray) = geocoded.keys().next() {
        return Err(format!(
            "the geocoder answered for {stray}, which the directory does not list"
        ));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXTRACT: &str = "Generated on 10/4/2026 6:46:15 PM\n\
IRN,ORGANIZATION NAME,SCHOOL TYPE,STATUS,DESIGNATED COUNTY,ORG MAILING ADDRESS,PARENT IRN\n\
=\"057539\",\"Mater Dei Academy\",\"Elementary School\",\"Open\",\"Lake\",\"29840 Euclid Ave, Wickliffe, Ohio, 44092\",=\"052522\"\n\
=\"057539\",\"Mater Dei Academy\",\"Middle School\",\"Open\",\"Lake\",\"29840 Euclid Ave, Wickliffe, Ohio, 44092\",=\"052522\"\n\
=\"057448\",\"St Helen\",\"Elementary School\",\"Open\",\"Geauga\",\"12060 Kinsman Rd, Newbury, Ohio, 44065\",=\"052522\"\n\
=\"000001\",\"Edge Academy, The\",\"Elementary School\",\"Open\",\"Lucas\",\"1 Main St, Suite 2, Toledo, Ohio, 43604\",=\"\"\n";

    const GEOCODED: &str = "\
\"057448\",\"12060 Kinsman Rd, Newbury, Ohio, 44065\",\"Match\",\"Exact\",\"12060 KINSMAN RD, NEWBURY, OH, 44065\",\"-81.2,41.4\",\"1\",\"L\",\"39\",\"055\",\"310100\",\"1001\"\n\
\"000001\",\"1 Main St, Suite 2, Toledo, Ohio, 43604\",\"No_Match\"\n\
\"057539\",\"29840 Euclid Ave, Wickliffe, Ohio, 44092\",\"Match\",\"Non_Exact\",\"29840 EUCLID AVE, WICKLIFFE, OH, 44092\",\"-81.4,41.6\",\"2\",\"R\",\"39\",\"085\",\"203500\",\"2002\"\n";

    const SDUNI: &str = "BLOCKID|DISTRICT\n390553101001001|04721\n390852035002002|05025\n";

    fn ccd() -> Vec<Vec<String>> {
        [
            "2019,3904721,047217,Newbury Local,1,1",
            "2020,3904721,047217,Newbury Local,1,2",
            "2022,3905025,048264,Wickliffe City,1,1",
            "2023,3905025,048264,Wickliffe City,1,1",
        ]
        .iter()
        .map(|line| line.split(',').map(str::to_string).collect())
        .collect()
    }

    #[test]
    fn a_school_listed_once_per_type_is_one_school() {
        let schools = parse_directory(EXTRACT).unwrap();
        let irns: Vec<&str> = schools.iter().map(|s| s.irn.as_str()).collect();
        assert_eq!(irns, ["000001", "057448", "057539"]);
        assert_eq!(schools[0].name, "Edge Academy The");
    }

    #[test]
    fn two_rows_for_one_irn_that_disagree_are_an_error() {
        let moved = EXTRACT.replacen("29840 Euclid Ave", "1 Elsewhere Rd", 1);
        let error = parse_directory(&moved).unwrap_err();
        assert!(error.contains("057539"), "{error}");
    }

    #[test]
    fn the_street_keeps_its_own_comma() {
        let addresses = geocoder_addresses(EXTRACT).unwrap();
        assert!(
            addresses.contains(r#""000001","1 Main St, Suite 2","Toledo","Ohio","43604""#),
            "{addresses}"
        );
    }

    #[test]
    fn a_closed_district_is_named_with_the_year_it_was_last_listed() {
        let rows = build_nonpublic_locations(&Locations {
            directory: EXTRACT,
            geocoded: GEOCODED,
            school_districts: SDUNI,
            ccd: &ccd(),
        })
        .unwrap();
        assert_eq!(
            rows[1],
            [
                "057448",
                "St Helen",
                "Geauga",
                "Exact",
                "390553101001001",
                "047217",
                "Newbury Local",
                "2020"
            ]
        );
        assert_eq!(rows[2][5..], ["048264", "Wickliffe City", "2023"]);
    }

    #[test]
    fn an_unmatched_school_is_kept_with_blank_geography() {
        let rows = build_nonpublic_locations(&Locations {
            directory: EXTRACT,
            geocoded: GEOCODED,
            school_districts: SDUNI,
            ccd: &ccd(),
        })
        .unwrap();
        assert_eq!(rows[0][3..], ["No_Match", "", "", "", ""]);
    }

    #[test]
    fn a_school_the_geocoder_skipped_is_an_error() {
        let short: String = GEOCODED.lines().skip(1).map(|l| format!("{l}\n")).collect();
        let error = build_nonpublic_locations(&Locations {
            directory: EXTRACT,
            geocoded: &short,
            school_districts: SDUNI,
            ccd: &ccd(),
        })
        .unwrap_err();
        assert!(error.contains("057448"), "{error}");
    }

    #[test]
    fn a_district_code_no_directory_year_lists_is_an_error() {
        let error = build_nonpublic_locations(&Locations {
            directory: EXTRACT,
            geocoded: GEOCODED,
            school_districts: SDUNI,
            ccd: &ccd()[2..],
        })
        .unwrap_err();
        assert!(error.contains("04721"), "{error}");
    }
}
