//! The Classroom Facilities Assistance Program's share rule: a district's rank, its percentile,
//! and the share of the basic project cost it pays.
//!
//! # The rule is three sections long
//!
//! R.C. 3318.011 has the department rank the city, local and exempted village districts on their
//! three-year average adjusted valuation per pupil, lowest first, and **"divide such ranking into
//! percentiles with the first percentile containing the one per cent of school districts having
//! the lowest"** values and the one-hundredth the one per cent having the highest. R.C. 3318.01(K)
//! makes the district's required percentage "one per cent of the basic project costs times the
//! percentile in which the school district ranks", for the fiscal year before the controlling
//! board approves the project. R.C. 3318.032(D) caps the district's portion at ninety-five per
//! cent.
//!
//! # The rounding is settled by the text
//!
//! Neither section says "round". With 609 districts a percentile holds 6.09 of them, so some rule
//! has to put a rank into a whole percentile, and three readings of `100 × rank / n` are on
//! offer. Only one satisfies the section's own two anchors:
//!
//! - **Floor** puts ranks 1 to 6 in a *zeroth* percentile, which division (D) does not have.
//! - **Nearest** puts ranks 1 to 3 there too.
//! - **Ceiling** puts rank 1 in the first percentile and rank 609 in the one-hundredth, and no
//!   percentile holds more than ⌈n/100⌉ districts. It is the reading on which each percentile
//!   *contains* one per cent of the districts counting up from the bottom.
//!
//! [`percentile`] is the ceiling and [`Rounding`] keeps the other two so the claim can be tested
//! rather than asserted. See `tests/the_rank_that_rounds_up.rs`.
//!
//! # And the Commission's worked example is approximate
//!
//! The rolling programme-status edition explains the rule by example: "the school at ranking 307
//! would have a state/local split of 50/50. This is done by taking the ranking divided by the
//! total number of schools which gives you the local share." 307 of 609 is 50.4%. On the
//! ceiling the district pays 51% and on the floor 50%, so the example names the rule's shape and
//! not its rounding. [`worked_example`] reads it from the committed edition.

use edfund_core::csv;

/// The equity lists, FY2024 and FY2025, one row per district per list year.
const FIXTURE: &str = include_str!("../fixtures/ofcc-equity-ranks.csv");

/// The Commission's rolling *Classroom Facilities Assistance Program Status* edition, as text.
const STATUS: &str = include_str!("../fixtures/ofcc-cfap-status.txt");

/// LSC's greenbook for the Facilities Construction Commission under H.B. 96, as text.
const GREENBOOK: &str = include_str!("../fixtures/fcc-greenbook.txt");

const HEADER: &str = "list_year,rank,district,county,adjusted_valuation_per_pupil";

/// R.C. 3318.032(D): a district's portion never exceeds ninety-five per cent of the basic project
/// cost.
pub const CAP_PERCENT: u32 = 95;

/// How a rank is put into a whole percentile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    /// `⌈100 × rank / n⌉`, the reading R.C. 3318.011(D) requires.
    Ceiling,
    /// `⌊100 × rank / n⌋`.
    Floor,
    /// `100 × rank / n` to the nearest whole number, halves up.
    Nearest,
}

/// The percentile a rank falls in under `rounding`, which may be zero for the two readings the
/// statute does not support.
///
/// # Panics
///
/// If `rank` is zero or exceeds `districts`.
#[must_use]
pub fn percentile_by(rank: u32, districts: u32, rounding: Rounding) -> u32 {
    assert!(
        (1..=districts).contains(&rank),
        "rank {rank} is outside 1..={districts}"
    );
    let scaled = 100 * rank;
    match rounding {
        Rounding::Ceiling => scaled.div_ceil(districts),
        Rounding::Floor => scaled / districts,
        Rounding::Nearest => (2 * scaled + districts) / (2 * districts),
    }
}

/// The R.C. 3318.011(D) percentile of `rank` among `districts`.
///
/// # Panics
///
/// As [`percentile_by`].
#[must_use]
pub fn percentile(rank: u32, districts: u32) -> u32 {
    percentile_by(rank, districts, Rounding::Ceiling)
}

/// The district's required percentage of the basic project cost, R.C. 3318.01(K), under the cap
/// of R.C. 3318.032(D).
///
/// This is the whole of the rule for a first project approved on or after 30 September 2025. A
/// segment after the first takes the first segment's share (3318.032(B)), and a district assisted
/// within the past twenty years pays the lesser of this and the greater of this and its previous
/// share (3318.032(E)); neither is modelled here. Before 30 September 2025 the district paid the
/// greater of this and a net-indebtedness amount, which no held source states per district.
///
/// # Panics
///
/// As [`percentile_by`].
#[must_use]
pub fn required_percent(rank: u32, districts: u32) -> u32 {
    percentile(rank, districts).min(CAP_PERCENT)
}

/// One district's place on one list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranked {
    /// The list's fiscal year. It governs the approvals of the fiscal year after it.
    pub list_year: u16,
    /// 1 is the lowest three-year average adjusted valuation per pupil.
    pub rank: u32,
    /// The Commission's name for the district.
    pub district: String,
    /// The county the Commission lists it under.
    pub county: String,
    /// The three-year average adjusted valuation per pupil, in whole dollars.
    pub adjusted_valuation_per_pupil: u64,
}

/// Every district on the `list_year` list, in rank order. Empty for a year not held.
///
/// # Panics
///
/// If the committed fixture is malformed, which is a build-time fact.
#[must_use]
pub fn list(list_year: u16) -> Vec<Ranked> {
    csv::rows(FIXTURE, HEADER)
        .filter(|row| row.str(0) == list_year.to_string())
        .map(|row| Ranked {
            list_year,
            rank: row.str(1).parse().expect("a rank"),
            district: row.str(2).to_string(),
            county: row.str(3).to_string(),
            adjusted_valuation_per_pupil: row.str(4).parse().expect("whole dollars"),
        })
        .collect()
}

/// The number of districts on the `list_year` list.
#[must_use]
pub fn districts(list_year: u16) -> u32 {
    u32::try_from(list(list_year).len()).expect("a list fits in u32")
}

/// One district's rank on the `list_year` list, keyed on name and county.
#[must_use]
pub fn rank_of(list_year: u16, district: &str, county: &str) -> Option<u32> {
    list(list_year)
        .into_iter()
        .find(|row| row.district == district && row.county == county)
        .map(|row| row.rank)
}

/// How many districts on the `list_year` list pay the capped ninety-five per cent.
#[must_use]
pub fn at_cap(list_year: u16) -> usize {
    let n = districts(list_year);
    list(list_year)
        .iter()
        .filter(|row| percentile(row.rank, n) >= CAP_PERCENT)
        .count()
}

/// How many districts on the `list_year` list the cap actually lowers — those whose percentile is
/// above ninety-five.
#[must_use]
pub fn lowered_by_cap(list_year: u16) -> usize {
    let n = districts(list_year);
    list(list_year)
        .iter()
        .filter(|row| percentile(row.rank, n) > CAP_PERCENT)
        .count()
}

/// The Commission's worked example in the rolling status edition, as `(rank, local share)`:
/// "the school at ranking 307 would have a state/local split of 50/50".
///
/// # Panics
///
/// If the committed edition no longer carries the sentence.
#[must_use]
pub fn worked_example() -> (u32, u32) {
    let flat = STATUS.split_whitespace().collect::<Vec<_>>().join(" ");
    let (_, after) = flat
        .split_once("the school at ranking ")
        .expect("the status edition states its worked example");
    let (rank, rest) = after.split_once(' ').expect("a rank");
    let (_, split) = rest
        .split_once("state/local split of ")
        .expect("the example's split");
    let local = split
        .split('/')
        .nth(1)
        .and_then(|cell| cell.split(|c: char| !c.is_ascii_digit()).next())
        .expect("a local half");
    (
        rank.parse().expect("a whole rank"),
        local.parse().expect("a whole share"),
    )
}

/// The district total in the status edition's legend: every district the programme counts.
///
/// # Panics
///
/// If the committed edition no longer carries a `TOTAL` line.
#[must_use]
pub fn status_total() -> u32 {
    STATUS
        .lines()
        .find_map(|line| line.trim().strip_prefix("TOTAL"))
        .and_then(|rest| rest.trim().parse().ok())
        .expect("the status edition's legend totals the districts")
}

/// A district's FY2026 rank as the status edition prints it beside its programme status.
///
/// The FY2026 list itself is not archived; this is the only held statement of the ranking that
/// governs FY2027 approvals. The edition abbreviates some names (`Groveport Madison`), so the
/// match is on the name as the edition prints it.
#[must_use]
pub fn fy2026_rank(district: &str, county: &str) -> Option<u32> {
    STATUS.lines().find_map(|line| {
        let cells: Vec<&str> = line
            .split("  ")
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect();
        match cells.as_slice() {
            [_, name, at, rank, _] if *name == district && *at == county => rank.parse().ok(),
            _ => None,
        }
    })
}

/// The FY2026 appropriation to Project Support, ALI 230647, as the greenbook's line table
/// prints it: the fifth of six fiscal-year cells after the line's own heading.
///
/// # Panics
///
/// If the committed greenbook no longer carries the line.
#[must_use]
pub fn project_support_fy2026() -> u64 {
    let mut lines = GREENBOOK.lines();
    lines
        .find(|line| line.contains("ALI 230647, Project Support"))
        .expect("the greenbook prints the Project Support line");
    let amounts = lines.next().expect("the line's amounts");
    let cells: Vec<&str> = amounts.split_whitespace().collect();
    assert_eq!(cells.len(), 6, "FY2022 to FY2027: {amounts}");
    cells[4]
        .trim_start_matches('$')
        .replace(',', "")
        .parse()
        .expect("whole dollars")
}

/// The greenbook, with its line breaks and runs of spaces collapsed, for phrase searches that
/// would otherwise straddle a line.
#[must_use]
pub fn greenbook_flat() -> String {
    GREENBOOK.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ceiling_reaches_both_ends_and_the_others_do_not() {
        assert_eq!(percentile(1, 609), 1);
        assert_eq!(percentile(609, 609), 100);
        assert_eq!(percentile_by(1, 609, Rounding::Floor), 0);
        assert_eq!(percentile_by(3, 609, Rounding::Nearest), 0);
        assert_eq!(percentile_by(4, 609, Rounding::Nearest), 1);
    }

    #[test]
    fn the_cap_binds_above_ninety_five() {
        assert_eq!(required_percent(609, 609), 95);
        assert_eq!(required_percent(578, 609), 95);
        assert_eq!(required_percent(577, 609), 95);
        assert_eq!(required_percent(572, 609), 94);
    }

    #[test]
    fn both_lists_are_whole() {
        for year in [2024, 2025] {
            let rows = list(year);
            assert_eq!(rows.len(), 609);
            assert!(rows.iter().zip(1..).all(|(row, rank)| row.rank == rank));
        }
        assert!(list(2023).is_empty());
    }
}
