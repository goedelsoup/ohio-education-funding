//! Which Ohio House and Senate districts a school district lies in, from Census PL 94-171.
//!
//! There is no published crosswalk, so this builds one: block-level population from the
//! redistricting file, assigned to school districts by the geographic header record, aggregated
//! to legislative districts. The two readers below are the PL 94-171 format itself — a geographic
//! header keyed by logical record number, and a table keyed the same way.

use std::collections::{BTreeMap, BTreeSet};

use super::delimited::{column, delimited_fields};
use super::format::format_value;

/// Columns of the legislative-district crosswalk.
pub const CROSSWALK_HEADER: &[&str] = &[
    "chamber",
    "irn",
    "district",
    "population",
    "population_under_18",
    "share",
];

/// `LOGRECNO` to block code, for the block records of a PL 94-171 geoheader.
///
/// The geoheader carries every summary level in one file — state, county, tract, block — and
/// only summary level `750` is a block. `LOGRECNO` is the join key the table files use; the
/// geoheader is the only place it can be turned back into a geography.
#[must_use]
pub fn pl_blocks(geo: &str) -> BTreeMap<String, String> {
    const SUMMARY_LEVEL: usize = 2;
    const LOGRECNO: usize = 7;
    const GEOCODE: usize = 9;
    const BLOCK: &str = "750";

    geo.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            (f.get(SUMMARY_LEVEL) == Some(&BLOCK))
                .then(|| Some((f.get(LOGRECNO)?.to_string(), f.get(GEOCODE)?.to_string())))
                .flatten()
        })
        .collect()
}

/// `LOGRECNO` to the first count in a PL 94-171 table file.
///
/// The first data column of file 1 is `P0010001`, the total population; of file 2, `P0030001`,
/// the population 18 and over. Nothing here needs any of the other 140-odd columns, and reading
/// only the first keeps a 150 MB file from being parsed into anything larger than a count.
#[must_use]
pub fn pl_counts(table: &str) -> BTreeMap<String, i64> {
    const LOGRECNO: usize = 4;
    const FIRST_COUNT: usize = 5;

    table
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            Some((
                f.get(LOGRECNO)?.to_string(),
                f.get(FIRST_COUNT)?.parse::<i64>().ok()?,
            ))
        })
        .collect()
}

/// Everything the crosswalk is assembled from.
#[derive(Debug, Clone, Copy)]
pub struct Crosswalk<'a> {
    /// Block code to its 2020 total and under-18 population.
    pub population: &'a BTreeMap<String, (i64, i64)>,
    /// The `BlockAssign` unified school district file: block to five-digit district code.
    pub school_districts: &'a str,
    /// The 2024 lower-chamber block equivalency file.
    pub house: &'a str,
    /// And the upper chamber's.
    pub senate: &'a str,
    /// The CCD LEA directory, which carries the district code to IRN join.
    pub directory: &'a str,
    /// The funding panel's IRNs. A district outside it is not carried — see
    /// [`build_legislative_crosswalk`].
    pub panel: &'a BTreeSet<String>,
}

/// Apportion each school district's pupils across the legislative seats that contain them.
///
/// # Why this has to be built from blocks
///
/// Ohio's funding system contains no legislative district, so the mapping does not exist to be
/// downloaded. The census block is the only geography both a school district and a House seat are
/// built out of, and 339 of the 609 districts straddle two or more seats — so a seat cannot be an
/// attribution the way a county can, and the crosswalk carries a share rather than an assignment.
///
/// # The weight is children, not people
///
/// A seat's share of a district is its share of the district's **under-18 population**, not of its
/// total. A district's pupils are what is being apportioned, and Ohio's under-18 share varies
/// enough between blocks that the choice moves the answer. Shares sum to one per district and
/// chamber, which is what lets
/// [`project::legislative_district`](../../../project/src/legislative_district.rs) apportion a
/// statewide total across seats without losing or inventing a dollar.
///
/// # What is left out
///
/// A block with no population contributes nothing and is not carried; 53 such rows would
/// otherwise appear with a share of zero. Districts outside the funding panel are dropped
/// entirely — the two Lake Erie island districts, Middle Bass and North Bass, are in the CCD
/// directory and in the census, and are not in the model this crosswalk exists to apportion.
///
/// # Errors
///
/// Returns the missing column's name if the CCD directory's layout has moved.
pub fn build_legislative_crosswalk(sources: &Crosswalk<'_>) -> Result<Vec<Vec<String>>, String> {
    let mut rows = sources.directory.lines();
    let head = delimited_fields(rows.next().unwrap_or_default(), ',');
    let (key, st_leaid) = (
        column(&head, "LEAID", "the CCD LEA directory")?,
        column(&head, "ST_LEAID", "the CCD LEA directory")?,
    );
    // The census keys school districts by a five-digit local code; the CCD's LEAID is the state
    // FIPS followed by that code, and its ST_LEAID is `OH-` followed by the IRN.
    let mut irn_of: BTreeMap<String, String> = BTreeMap::new();
    for line in rows {
        let f = delimited_fields(line, ',');
        let (Some(leaid), Some(irn)) = (f.get(key), f.get(st_leaid)) else {
            continue;
        };
        if let Some(local) = leaid.trim().strip_prefix("39") {
            irn_of.insert(
                local.to_string(),
                irn.trim()
                    .strip_prefix("OH-")
                    .unwrap_or(irn.trim())
                    .to_string(),
            );
        }
    }

    // `BLOCKID|DISTRICT`, against the equivalency files' `GEOID,SLDLST`.
    let assignment = |text: &str, delimiter: char| -> BTreeMap<String, String> {
        text.lines()
            .skip(1)
            .filter_map(|line| {
                let (block, district) = line.trim_end().split_once(delimiter)?;
                Some((block.to_string(), district.to_string()))
            })
            .collect()
    };
    let in_district = assignment(sources.school_districts, '|');
    let chambers = [
        ("house", assignment(sources.house, ',')),
        ("senate", assignment(sources.senate, ',')),
    ];

    let mut totals: BTreeMap<(&str, String, String), (i64, i64)> = BTreeMap::new();
    for (block, (people, children)) in sources.population {
        if *people == 0 {
            continue;
        }
        let Some(irn) = in_district.get(block).and_then(|code| irn_of.get(code)) else {
            continue;
        };
        if !sources.panel.contains(irn) {
            continue;
        }
        for (chamber, blocks) in &chambers {
            let Some(seat) = blocks.get(block) else {
                continue;
            };
            let entry = totals
                .entry((chamber, irn.clone(), seat.clone()))
                .or_insert((0, 0));
            entry.0 += people;
            entry.1 += children;
        }
    }

    let mut pupils: BTreeMap<(&str, &String), i64> = BTreeMap::new();
    for ((chamber, irn, _), (_, children)) in &totals {
        *pupils.entry((chamber, irn)).or_insert(0) += children;
    }

    // `BTreeMap` orders by key, and the key is (chamber, IRN, seat) — but "house" sorts before
    // "senate" only by luck of the alphabet, so the chamber order is made explicit.
    let mut out: Vec<(usize, Vec<String>)> = totals
        .iter()
        .map(|((chamber, irn, seat), (people, children))| {
            let of_district = pupils.get(&(chamber, irn)).copied().unwrap_or(0);
            let share = if of_district > 0 {
                #[allow(clippy::cast_precision_loss)]
                let value = *children as f64 / of_district as f64;
                value
            } else {
                0.0
            };
            (
                usize::from(*chamber == "senate"),
                vec![
                    (*chamber).to_string(),
                    irn.clone(),
                    seat.clone(),
                    people.to_string(),
                    children.to_string(),
                    format_value(Some(share), 8),
                ],
            )
        })
        .collect();
    out.sort_by_key(|a| a.0);
    Ok(out.into_iter().map(|(_, row)| row).collect())
}
