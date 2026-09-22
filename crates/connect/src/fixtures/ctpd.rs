//! Who belongs to each career-technical planning district.
//!
//! The membership of Ohio's 92 planning districts, which R.C. 3317.014(E)(1)(a) makes the base
//! of the career awareness and exploration payment: the lead district of each CTPD is paid $3 for
//! every pupil in the summed enrolled ADM of that planning district's members.
//! [`dew-career-technical-planning-districts`](../../../../.yidam/catalog/dew-career-technical-planning-districts.md)
//! gives the 92 **leads**. This gives the members.
//!
//! # Why the API, which is not this crate's habit
//!
//! Every other source here is a file. This one is an endpoint, and it is the only route: the
//! report card's static download catalogue publishes a `Career Technical Planning District`
//! category of **twenty-four** files going back to 2013, and every one of them is ratings or
//! grades. Membership appears in none of them. Two earlier portals that did publish it are gone —
//! `webapp2.ode.state.oh.us/ctpd_region/` no longer resolves in DNS, and the report card's own
//! CTPD pages are an Angular shell that returns one identical 11,166-byte document for every
//! path, `/robots.txt` included.
//!
//! The endpoint is `api/GetByIrn/{irn}/{schYear}` on `reportcarddataapi.education.ohio.gov`. Its
//! key is an Azure Functions key the site ships to every visitor in its `main.<hash>.js`, and it
//! is accepted as a **`?code=` query parameter** as well as a header — which is what lets each
//! roster be a plain URL in the registry, pinned by digest like any other publication, rather
//! than a retrieval that needs a header the `Source` struct has no field for.
//!
//! **The bytes are stable.** Each document carries Cosmos metadata (`_ts`, `_etag`, `_rid`), and
//! `_ts` is the last *write* rather than the read, so two fetches are byte-identical and a
//! changed digest means the department rewrote the record. That is the property that makes
//! pinning 90 API responses meaningful rather than theatre.
//!
//! # The join, and the forty-seven names that cannot carry it
//!
//! `enrollmentDist` names each member and gives **no IRN** — the entries carry `distName` and
//! `enrlCtpd` and nothing else. Resolution is therefore against **the same site's own
//! organisation index**, `live-search.json`, which carries `irn`, `doctype` and `name` for all
//! 4,018 organisations the report card rates. That is the publisher's own table rather than a
//! fuzzy match against this repository's panel, and it places most members exactly.
//!
//! **It cannot place all of them, and the reason is the one this repository already knows.**
//! Ohio has 607 rated districts under fewer distinct names: normalised for the two styles the
//! publisher writes, 607 names collapse to **579 keys**, and **nineteen keys are shared by
//! forty-seven districts** — three `Perry Local`s, three `Buckeye Local`s, three `Madison
//! Local`s. The index disambiguates them with a ` - County` tail that a roster does not have. A
//! name match would place them wrongly and plausibly, which is what
//! `district-names-are-not-unique` records the cost of.
//!
//! So those forty-seven are retrieved individually. **A district's own record carries
//! `ctpdIrn`**, which states its planning district directly, and [`Placement`] is that: one
//! source per ambiguous district, no more, each justified by its own name being ambiguous. No
//! two same-named districts share a planning district — checked, not assumed — so the placement
//! is unique.
//!
//! The result is checkable in a way a name match alone would not be, and
//! [`build_ctpd_membership`] checks it: **every one of the 607 districts must appear in exactly
//! one planning district's roster.** A file that broke that is refused rather than producing a
//! roster with a district dropped out of it or counted twice.
//!
//! # What the roster is for, and the measure it does not carry
//!
//! Of **954** FY2025 members, 607 are districts and **347 are not** — community schools under
//! R.C. Chapter 3314 and STEM schools under Chapter 3326, which is exactly the population the
//! 609-district panel does not hold. That is the quantity `project::drafts` could previously only
//! name: the payment's base spans two populations, and now the size of the second one is
//! measured rather than asserted.
//!
//! **`enrlCtpd` is career-technical enrolment, not enrolled ADM.** The statute pays on enrolled
//! ADM and the API does not publish it, so this fixture carries what the source states and the
//! reader says so. On the measure that *is* published, the non-district members are 4.75% of the
//! total against 36.4% of the member count, and **277 of the 347 carry zero** — a population
//! large in members and small in pupils, which is a different fact from either number alone.

use std::collections::{BTreeMap, BTreeSet};

use super::format::clean_name;
use crate::json::{self, Value};

/// Columns of the planning-district membership extract.
///
/// One row per member per planning district. `district_irn` is the column the fixture exists
/// for: filled for a member that is one of the 607 districts, blank for a community or STEM
/// school, and the blanks are the measurement rather than a gap.
pub const CTPD_MEMBERSHIP_HEADER: &[&str] = &[
    "school_year",
    "ctpd_irn",
    "ctpd",
    "member",
    "district_irn",
    "career_technical_enrolment",
];

/// How many districts Ohio's report card rates, and therefore how many must be placed.
///
/// Not the calculators' 609: the report card rates the 607 that have a rating, and the two the
/// panel carries beyond them are not in this population. An equality rather than a floor,
/// because a district that belonged to no planning district would break the department's own
/// statement that every district belongs to one.
const RATED_DISTRICTS: usize = 607;

/// How many planning districts answer on this endpoint.
///
/// **Ninety, not the catalog's ninety-two.** The two that are absent are the correctional
/// institutions, `200600` and `200602`, which return HTTP 204 and appear in no organisation
/// index — they are rated by nobody, which is the same reason the PDF's map legend files them
/// as their own kind. 90 + 2 is the table's 92, and that is the whole of the discrepancy the
/// catalog entry recorded as unresolved.
const RATED_CTPDS: usize = 90;

/// One planning district's response, and the fiscal year it was asked for.
pub struct CtpdRoster<'a> {
    /// The planning district's number, as the registry key names it.
    pub ctpd_irn: &'a str,
    /// The body of `api/GetByIrn/{ctpd}/{year}`.
    pub body: &'a str,
}

/// One ambiguously-named district's own record, which states its planning district.
///
/// Retrieved only for the forty-seven districts whose normalised name another district shares.
/// Every district carries `ctpdIrn`; these are the only ones whose roster entry cannot say which
/// district it is.
pub struct Placement<'a> {
    /// The district's IRN, as the registry key names it.
    pub district_irn: &'a str,
    /// The body of `api/GetByIrn/{district}/{year}`.
    pub body: &'a str,
}

/// Build the membership extract from the organisation index and every planning district's roster.
///
/// # Errors
///
/// Returns the reason a document cannot be read, a roster whose planning district is not the one
/// requested, a planning district count that is not `RATED_CTPDS`, a district that appears in
/// no roster or in more than one, or a member name that resolves to an organisation the index
/// files as something other than a district or a school. Each of those is a roster that reads
/// perfectly and has quietly lost or duplicated a member.
pub fn build_ctpd_membership(
    school_year: u16,
    index: &str,
    rosters: &[CtpdRoster<'_>],
    placements: &[Placement<'_>],
) -> Result<Vec<Vec<String>>, String> {
    build_ctpd_membership_with(
        school_year,
        index,
        rosters,
        placements,
        RATED_DISTRICTS,
        RATED_CTPDS,
    )
}

/// The body of [`build_ctpd_membership`], with the two population counts as parameters.
///
/// Separated so the tests can exercise the checks against a three-organisation index instead of
/// a 4,018-entry one. The public entry point pins both to the department's own numbers, which is
/// where they belong: a test free to choose them would not be testing them.
fn build_ctpd_membership_with(
    school_year: u16,
    index: &str,
    rosters: &[CtpdRoster<'_>],
    placements: &[Placement<'_>],
    rated_districts: usize,
    rated_ctpds: usize,
) -> Result<Vec<Vec<String>>, String> {
    let index = json::parse(index).map_err(|why| format!("the organisation index: {why}"))?;
    let orgs = index
        .as_array()
        .ok_or("the organisation index is not an array")?;

    // Name -> IRNs for the districts the report card rates. The index writes a district as
    // `Name - County`; a roster writes the bare name, so the county tail is dropped and the
    // comparison is on what remains. A `Vec` rather than one IRN because nineteen of these keys
    // are shared: collapsing them into a map is exactly the silent misplacement this guards.
    let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut district_count = 0usize;
    for org in orgs {
        let (Some(irn), Some(name), Some(kind)) =
            (org.text("irn"), org.text("name"), org.text("doctype"))
        else {
            continue;
        };
        if kind == "district" {
            district_count += 1;
            by_name
                .entry(match_key(strip_county(name)))
                .or_default()
                .push(irn.to_string());
        }
    }
    if district_count != rated_districts {
        return Err(format!(
            "the organisation index carries {district_count} districts and the report card \
             rates {rated_districts}"
        ));
    }

    // Where each ambiguously-named district actually belongs, from its own record.
    let mut placed_by_record: BTreeMap<String, String> = BTreeMap::new();
    for placement in placements {
        let document = json::parse(placement.body)
            .map_err(|why| format!("district {}: {why}", placement.district_irn))?;
        let entry = document
            .as_array()
            .and_then(<[Value]>::first)
            .ok_or_else(|| format!("district {} returned no document", placement.district_irn))?;
        let irn = entry
            .text("irn")
            .ok_or_else(|| format!("district {} has no irn", placement.district_irn))?;
        if irn != placement.district_irn {
            return Err(format!(
                "district {} answered for {irn}; the registry key and the document disagree",
                placement.district_irn
            ));
        }
        let ctpd = entry.text("ctpdIrn").ok_or_else(|| {
            format!("district {irn} publishes no ctpdIrn, so its roster entry cannot be placed")
        })?;
        placed_by_record.insert(irn.to_string(), ctpd.to_string());
    }

    // Every shared name must be covered, or a roster entry for it has no way to say which
    // district it is. Checked here rather than discovered as a wrong IRN later.
    let ambiguous: BTreeSet<&String> = by_name
        .values()
        .filter(|irns| irns.len() > 1)
        .flatten()
        .collect();
    let unplaceable: Vec<&&String> = ambiguous
        .iter()
        .filter(|irn| !placed_by_record.contains_key(**irn))
        .collect();
    if !unplaceable.is_empty() {
        return Err(format!(
            "{} districts share a name with another and have no record of their own to place \
             them, beginning {:?}",
            unplaceable.len(),
            unplaceable.iter().take(5).collect::<Vec<_>>()
        ));
    }

    if rosters.len() != rated_ctpds {
        return Err(format!(
            "{} planning districts were retrieved and {rated_ctpds} answer this endpoint",
            rosters.len()
        ));
    }

    let mut out = Vec::new();
    let mut placed: BTreeMap<String, String> = BTreeMap::new();
    for roster in rosters {
        let document = json::parse(roster.body)
            .map_err(|why| format!("planning district {}: {why}", roster.ctpd_irn))?;
        let entry = document
            .as_array()
            .and_then(<[Value]>::first)
            .ok_or_else(|| format!("planning district {} returned no document", roster.ctpd_irn))?;

        let irn = entry
            .text("irn")
            .ok_or_else(|| format!("planning district {} has no irn", roster.ctpd_irn))?;
        if irn != roster.ctpd_irn {
            return Err(format!(
                "planning district {} answered for {irn}; the registry key and the document \
                 disagree",
                roster.ctpd_irn
            ));
        }
        let name = entry
            .text("name")
            .ok_or_else(|| format!("planning district {irn} has no name"))?;

        let members = entry
            .get("detail")
            .and_then(|d| d.get("enrollmentDist"))
            .and_then(Value::as_array)
            .ok_or_else(|| format!("planning district {irn} publishes no enrollmentDist"))?;

        for member in members {
            let member_name = member
                .text("distName")
                .ok_or_else(|| format!("planning district {irn} has a member with no name"))?;
            let district_irn = resolve(&by_name, &placed_by_record, member_name, irn)?;

            if let Some(district) = &district_irn {
                if let Some(first) = placed.insert(district.clone(), irn.to_string()) {
                    return Err(format!(
                        "district {district} is in the rosters of both {first} and {irn}; a \
                         district belongs to one planning district"
                    ));
                }
            }

            out.push(vec![
                school_year.to_string(),
                irn.to_string(),
                clean_name(name),
                clean_name(member_name),
                district_irn.unwrap_or_default(),
                member
                    .number("enrlCtpd")
                    .map_or_else(String::new, |v| format!("{v:.0}")),
            ]);
        }
    }

    // Every district belongs to a planning district. The department states it; this is where it
    // is checked, and it is the check that would catch a roster silently short of a member.
    let unplaced: BTreeSet<&String> = by_name
        .values()
        .flatten()
        .filter(|irn| !placed.contains_key(*irn))
        .collect();
    if !unplaced.is_empty() {
        return Err(format!(
            "{} districts appear in no planning district's roster, beginning {:?}. The \
             department's own statement is that every district belongs to one",
            unplaced.len(),
            unplaced.iter().take(5).collect::<Vec<_>>()
        ));
    }

    Ok(out)
}

/// Which district a roster entry names, if it names one.
///
/// A name matching exactly one district is that district. A name shared by several is resolved
/// by the planning district each of them states in its own record — and if none of them states
/// *this* one, that is an error rather than a guess, because picking a candidate by position
/// would produce an IRN that is wrong and looks right.
fn resolve(
    by_name: &BTreeMap<String, Vec<String>>,
    placed_by_record: &BTreeMap<String, String>,
    member: &str,
    ctpd_irn: &str,
) -> Result<Option<String>, String> {
    let Some(candidates) = by_name.get(&match_key(member)) else {
        return Ok(None); // a community or STEM school, which is a finding rather than a failure
    };
    match candidates.as_slice() {
        [only] => Ok(Some(only.clone())),
        many => {
            let matching: Vec<&String> = many
                .iter()
                .filter(|irn| placed_by_record.get(*irn).is_some_and(|c| c == ctpd_irn))
                .collect();
            match matching.as_slice() {
                [one] => Ok(Some((*one).clone())),
                [] => Err(format!(
                    "planning district {ctpd_irn} lists `{member}`, a name {} districts share, \
                     and none of them states {ctpd_irn} as its own planning district",
                    many.len()
                )),
                several => Err(format!(
                    "planning district {ctpd_irn} lists `{member}` and {} districts sharing that \
                     name each claim it: {several:?}",
                    several.len()
                )),
            }
        }
    }
}

/// Drop the ` - County` tail the organisation index appends to a district's name.
///
/// Only the last one, and only when there is one: `Manchester Local - Adams County` is a district
/// in Adams County, while a school's index name carries two such tails and is not read here.
fn strip_county(name: &str) -> &str {
    name.rsplit_once(" - ").map_or(name, |(head, _)| head)
}

/// The form two names are compared in.
///
/// Case, punctuation, spacing and the interchangeable `School District` / `Schools` / `District`
/// suffixes are removed, because the two sides of this join are written by the same publisher in
/// two styles — `Cuyahoga Falls City School District` in a roster against `Cuyahoga Falls City
/// School District - Summit County` in the index, and `Hudson City` against `Hudson City School
/// District`. What is *not* removed is anything that distinguishes two districts from each other,
/// which is why `Local`, `City` and `Exempted Village` stay: they are the words that tell
/// `Perry Local` in Stark from `Perry Local` in Allen.
fn match_key(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut cleaned = lower.as_str().to_string();
    for suffix in [
        " school district",
        " schools district",
        " local school district",
        " city school district",
        " district",
        " schools",
    ] {
        if let Some(head) = cleaned.strip_suffix(suffix) {
            cleaned = head.to_string();
        }
    }
    cleaned
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX: &str = r#"[
        {"irn":"043836","doctype":"district","name":"Cuyahoga Falls City School District - Summit County"},
        {"irn":"044909","doctype":"district","name":"Hudson City School District - Summit County"},
        {"irn":"000123","doctype":"school","name":"Schnee Learning Center - Summit County"}
    ]"#;

    fn roster(body: &str) -> String {
        format!(
            r#"[{{"irn":"200097","name":"Six District Voc Ed Compact CTPD","detail":{{"enrollmentDist":[{body}]}}}}]"#
        )
    }

    /// The two publishers' spellings differ and the join still has to be exact.
    #[test]
    fn a_name_resolves_across_the_two_styles_the_publisher_writes() {
        assert_eq!(
            match_key(strip_county(
                "Cuyahoga Falls City School District - Summit County"
            )),
            match_key("Cuyahoga Falls City School District")
        );
        assert_eq!(
            match_key(strip_county("Hudson City School District - Summit County")),
            match_key("Hudson City")
        );
    }

    /// And the words that tell two districts apart are not the ones stripped.
    #[test]
    fn the_distinguishing_words_survive_normalisation() {
        assert_ne!(match_key("Perry Local"), match_key("Perry City"));
        assert_ne!(
            match_key("Jackson Local"),
            match_key("Jackson Exempted Village")
        );
    }

    /// A member that is not a district is recorded without an IRN rather than dropped.
    #[test]
    fn a_community_school_member_keeps_its_row_and_carries_no_district_irn() {
        let body = r#"{"distName":"Cuyahoga Falls City School District","enrlCtpd":567},
                      {"distName":"Hudson City","enrlCtpd":97},
                      {"distName":"Schnee Learning Center","enrlCtpd":0}"#;
        let rosters = [CtpdRoster {
            ctpd_irn: "200097",
            body: &roster(body),
        }];
        // Two districts in the index, both placed, so the completeness check is satisfied.
        let rows = build_for_test(&rosters).expect("builds");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][4], "043836");
        assert_eq!(rows[1][4], "044909");
        assert_eq!(
            rows[2][4], "",
            "a learning centre is not one of the districts"
        );
        assert_eq!(rows[2][3], "Schnee Learning Center");
    }

    /// A district in two rosters is a double-count, and double-counting is the failure this
    /// payment's base is most exposed to.
    #[test]
    fn a_district_in_two_rosters_is_refused() {
        let one = roster(
            r#"{"distName":"Cuyahoga Falls City School District","enrlCtpd":1},{"distName":"Hudson City","enrlCtpd":1}"#,
        );
        let two = r#"[{"irn":"200098","name":"Other CTPD","detail":{"enrollmentDist":[{"distName":"Hudson City","enrlCtpd":2}]}}]"#;
        let rosters = [
            CtpdRoster {
                ctpd_irn: "200097",
                body: &one,
            },
            CtpdRoster {
                ctpd_irn: "200098",
                body: two,
            },
        ];
        let why = build_for_test(&rosters).expect_err("refused");
        assert!(why.contains("belongs to one planning district"), "{why}");
    }

    /// A district in no roster is the silent loss the completeness check exists for.
    #[test]
    fn a_district_in_no_roster_is_refused() {
        let body = r#"{"distName":"Cuyahoga Falls City School District","enrlCtpd":567}"#;
        let rosters = [CtpdRoster {
            ctpd_irn: "200097",
            body: &roster(body),
        }];
        let why = build_for_test(&rosters).expect_err("refused");
        assert!(why.contains("appear in no planning district"), "{why}");
    }

    /// The registry key and the document have to agree about which planning district this is.
    #[test]
    fn a_roster_answering_for_another_ctpd_is_refused() {
        let body = r#"{"distName":"Cuyahoga Falls City School District","enrlCtpd":1},{"distName":"Hudson City","enrlCtpd":1}"#;
        let rosters = [CtpdRoster {
            ctpd_irn: "200099",
            body: &roster(body),
        }];
        let why = build_for_test(&rosters).expect_err("refused");
        assert!(why.contains("disagree"), "{why}");
    }

    /// Run the builder against the two-district test index, with the population checks relaxed to
    /// the fixture's size rather than the department's.
    fn build_for_test(rosters: &[CtpdRoster<'_>]) -> Result<Vec<Vec<String>>, String> {
        build_ctpd_membership_with(2025, INDEX, rosters, &[], 2, rosters.len())
    }

    /// Two districts sharing a name, as nineteen keys in the real index do.
    const SHARED: &str = r#"[
        {"irn":"045781","doctype":"district","name":"Perry Local - Allen County"},
        {"irn":"049924","doctype":"district","name":"Perry Local - Stark County"}
    ]"#;

    fn named(ctpd: &str, member: &str) -> String {
        format!(
            r#"[{{"irn":"{ctpd}","name":"{ctpd} CTPD","detail":{{"enrollmentDist":[{{"distName":"{member}","enrlCtpd":1}}]}}}}]"#
        )
    }

    fn placement(irn: &str, ctpd: &str) -> String {
        format!(r#"[{{"irn":"{irn}","ctpdIrn":"{ctpd}"}}]"#)
    }

    /// A shared name with no record to place it is refused, not guessed.
    ///
    /// This is the case that would otherwise produce an IRN that is wrong and looks right, which
    /// is the whole reason the forty-seven extra sources exist.
    #[test]
    fn a_shared_name_without_a_placement_is_refused() {
        let a = named("200001", "Perry Local");
        let b = named("200002", "Perry Local");
        let rosters = [
            CtpdRoster {
                ctpd_irn: "200001",
                body: &a,
            },
            CtpdRoster {
                ctpd_irn: "200002",
                body: &b,
            },
        ];
        let why =
            build_ctpd_membership_with(2025, SHARED, &rosters, &[], 2, 2).expect_err("refused");
        assert!(why.contains("no record of their own"), "{why}");
    }

    /// With both records present each name resolves to the district that claims that CTPD.
    #[test]
    fn a_shared_name_resolves_by_the_district_that_claims_the_planning_district() {
        let a = named("200001", "Perry Local");
        let b = named("200002", "Perry Local");
        let rosters = [
            CtpdRoster {
                ctpd_irn: "200001",
                body: &a,
            },
            CtpdRoster {
                ctpd_irn: "200002",
                body: &b,
            },
        ];
        let (p1, p2) = (placement("045781", "200002"), placement("049924", "200001"));
        let placements = [
            Placement {
                district_irn: "045781",
                body: &p1,
            },
            Placement {
                district_irn: "049924",
                body: &p2,
            },
        ];
        let rows =
            build_ctpd_membership_with(2025, SHARED, &rosters, &placements, 2, 2).expect("builds");
        // Allen's Perry Local claims 200002, Stark's claims 200001 — deliberately crossed, so a
        // resolver matching by order rather than by claim would fail this.
        assert_eq!(rows[0][1], "200001");
        assert_eq!(rows[0][4], "049924");
        assert_eq!(rows[1][1], "200002");
        assert_eq!(rows[1][4], "045781");
    }

    /// A placement that claims a planning district the roster does not list is an error.
    #[test]
    fn a_shared_name_no_candidate_claims_is_refused() {
        let a = named("200001", "Perry Local");
        let b = named("200002", "Perry Local");
        let rosters = [
            CtpdRoster {
                ctpd_irn: "200001",
                body: &a,
            },
            CtpdRoster {
                ctpd_irn: "200002",
                body: &b,
            },
        ];
        let (p1, p2) = (placement("045781", "200009"), placement("049924", "200001"));
        let placements = [
            Placement {
                district_irn: "045781",
                body: &p1,
            },
            Placement {
                district_irn: "049924",
                body: &p2,
            },
        ];
        let why = build_ctpd_membership_with(2025, SHARED, &rosters, &placements, 2, 2)
            .expect_err("refused");
        assert!(why.contains("none of them states"), "{why}");
    }
}
