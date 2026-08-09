//! Ohio House districts, and the apportionment that makes them comparable.
//!
//! # There is no published crosswalk, because there is no clean one to publish
//!
//! Counties work as a grouping because the department attributes each district to exactly one of
//! them. House districts do not: **339 of Ohio's 609 school districts have population in two or
//! more of the 99 House districts**, and one has population in eleven. Assigning each district to
//! a single House district — the county page's simplification — would state something false about
//! more than half the state.
//!
//! So this is a derivation rather than a lookup, and every step of it is somebody else's published
//! file:
//!
//! 1. The Census Bureau's 2020 block assignment file gives, for each of Ohio's **276,428** census
//!    blocks, the unified school district containing it.
//! 2. The Bureau's **2024** state legislative block equivalency file gives, for the same blocks,
//!    the House district. The 2024 vintage matters: 66.3% of Ohio's blocks changed House district
//!    between the 2020-cycle map and the one now in use, so the older file — which ships in the
//!    same block assignment archive as step 1 and is the obvious thing to reach for — would be
//!    wrong for two-thirds of the state.
//! 3. The 2020 PL 94-171 redistricting file gives each block's population and its population aged
//!    18 and over, and the difference between them is the weight used here.
//! 4. The NCES Common Core of Data directory carries `ST_LEAID`, which for Ohio is the IRN behind
//!    an `OH-` prefix, and joins the Census school district code to the funding model.
//!
//! All 609 districts in the funding panel come through that chain with no losses.
//!
//! # The weight is children, not people
//!
//! A school district's figures are apportioned across the House districts it overlaps in
//! proportion to its **under-18 population**, not its total population. Total population would
//! weight a House district full of retirees the same as one full of families, and the quantity
//! being divided is school funding.
//!
//! Ohio is 22.0% under 18, and that share varies enough between blocks for the choice to matter.
//! It is still a proxy: under-18 population is not enrolled ADM, it counts children in community
//! schools, private schools and none, and it is a 2020 count being applied to a FY2027 model.
//!
//! # What that means for the numbers this produces
//!
//! **Every apportioned figure here is an estimate that no one publishes, including the
//! department.** The department computes funding per district and stops; nothing in Ohio's system
//! has a House district as a unit of account. A page built on this must say so, because a reader
//! who takes "House District 24 receives $312m" for a published fact has been misled by precision
//! that is not there.
//!
//! What the estimate is good for is comparison and scale — which members' districts contain
//! districts on the guarantee, how much of the state's aid flows through the schools a member
//! represents, whether a proposed change lands in their district. What it is not good for is any
//! statement about a single district's entitlement, which is exact and lives on that district's
//! own page.
//!
//! The apportionment is **exact in aggregate by construction**: each district's shares sum to one,
//! so summing every House district's apportioned amount returns the statewide total to the cent.
//! That is asserted in the tests, and it is the only accuracy claim made for these figures.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};

use crate::panel::DistrictRecord;

/// The crosswalk, one row per (district, House district) pair with population in it.
const CROSSWALK: &str = include_str!("../fixtures/house-district-crosswalk.csv");

/// The header this loader was written against, so a regenerated crosswalk fails loudly.
const EXPECTED_HEADER: &str = "irn,house_district,population,population_under_18,share";

/// Ohio's House of Representatives has ninety-nine seats, and every one of them contains school
/// districts. Pinned because a crosswalk that produced a hundredth would mean a bad join.
pub const HOUSE_DISTRICTS: usize = 99;

/// One district's presence in one House district.
#[derive(Debug, Clone, PartialEq)]
pub struct Overlap {
    /// The district, by IRN.
    pub irn: String,
    /// The House district, as a zero-padded two- or three-character number: `001` through `099`.
    pub house_district: String,
    /// 2020 census population of the blocks in both.
    pub population: f64,
    /// Of that, the population under 18 — the weight the share is computed from.
    pub population_under_18: f64,
    /// This House district's share of the district's under-18 population. Sums to one per district.
    pub share: f64,
}

/// Every (district, House district) overlap with population in it.
///
/// # Panics
///
/// If the crosswalk's header is not the one this loader expects. Reading shifted columns would
/// apportion by the wrong number and produce a page of plausible wrong totals.
#[must_use]
pub fn overlaps() -> Vec<Overlap> {
    let mut lines = CROSSWALK.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the House district crosswalk header changed; update project::house_district"
    );

    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            Some(Overlap {
                irn: f.first()?.to_string(),
                house_district: f.get(1)?.to_string(),
                population: f.get(2)?.parse().ok()?,
                population_under_18: f.get(3)?.parse().ok()?,
                share: f.get(4)?.parse().ok()?,
            })
        })
        .collect()
}

/// One House district, with the school districts in it and their apportioned figures.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HouseDistrict {
    /// `001` through `099`.
    pub number: String,
    /// The districts overlapping it, largest share of *this* House district's pupils first.
    pub members: Vec<Membership>,
    /// Apportioned enrolled ADM. An estimate; see the module note.
    pub adm: Adm,
    /// Apportioned state aid as districts receive it, guarantee included.
    pub realized_aid: Dollars,
    /// Apportioned state share of base cost, and the categorical half beside it.
    pub base_cost_state_share: Dollars,
    /// Apportioned categorical funding — the other half of formula aid.
    pub categorical_funding: Dollars,
    /// Apportioned guarantee — what the formula does *not* justify, in this member's schools.
    pub guarantee: Dollars,
    /// Districts overlapping this House district that are on the guarantee.
    pub districts_on_guarantee: usize,
    /// And those at the minimum state share, where local capacity has stopped setting their aid.
    pub districts_at_minimum_state_share: usize,
    /// Whole districts, i.e. those with no population in any other House district.
    pub districts_wholly_inside: usize,
}

/// One district's membership of one House district, from the House district's point of view.
#[derive(Debug, Clone, PartialEq)]
pub struct Membership {
    /// The school district, by IRN.
    pub irn: String,
    /// Its name, carried so a page need not join back to the panel.
    pub name: String,
    /// The share of the *district* that lies in this House district.
    pub share: f64,
    /// The share of this *House district's* apportioned pupils that this district provides.
    pub share_of_house_district: f64,
    /// Apportioned pupils and aid for this pair.
    pub adm: Adm,
    /// Apportioned state aid as the district receives it.
    pub realized_aid: Dollars,
    /// Whether the district lies entirely within this House district.
    pub wholly_inside: bool,
}

/// Apportion the panel across the 99 House districts.
///
/// Each district's figures are multiplied by its share of that House district and summed. Because
/// the shares sum to one per district, the result sums to the statewide total exactly — which is
/// the one accuracy guarantee an apportionment like this can offer and is asserted in the tests.
#[must_use]
pub fn house_districts(panel: &[DistrictRecord]) -> Vec<HouseDistrict> {
    let by_irn: BTreeMap<&str, &DistrictRecord> =
        panel.iter().map(|r| (r.irn.as_str(), r)).collect();

    // How many House districts each district reaches, so "wholly inside" is a fact about the
    // district rather than about the row being looked at.
    let all = overlaps();
    let mut reach: BTreeMap<&str, usize> = BTreeMap::new();
    for o in &all {
        *reach.entry(o.irn.as_str()).or_default() += 1;
    }

    let mut out: BTreeMap<String, HouseDistrict> = BTreeMap::new();
    for o in &all {
        let Some(record) = by_irn.get(o.irn.as_str()) else {
            continue;
        };
        let entry = out
            .entry(o.house_district.clone())
            .or_insert_with(|| HouseDistrict {
                number: o.house_district.clone(),
                ..HouseDistrict::default()
            });
        let wholly_inside = reach.get(o.irn.as_str()).copied().unwrap_or(0) == 1;

        entry.adm += record.base_cost_adm() * o.share;
        entry.realized_aid += record.realized_aid() * o.share;
        entry.base_cost_state_share += record.base_cost_state_share * o.share;
        entry.categorical_funding += record.categorical_funding() * o.share;
        entry.guarantee += record.guarantee * o.share;
        if record.on_guarantee() {
            entry.districts_on_guarantee += 1;
        }
        if record.at_minimum_state_share() {
            entry.districts_at_minimum_state_share += 1;
        }
        if wholly_inside {
            entry.districts_wholly_inside += 1;
        }
        entry.members.push(Membership {
            irn: record.irn.clone(),
            name: record.name.clone(),
            share: o.share,
            // Filled in once the House district's total is known.
            share_of_house_district: 0.0,
            adm: record.base_cost_adm() * o.share,
            realized_aid: record.realized_aid() * o.share,
            wholly_inside,
        });
    }

    let mut districts: Vec<HouseDistrict> = out.into_values().collect();
    for hd in &mut districts {
        let total = hd.adm;
        for m in &mut hd.members {
            m.share_of_house_district = if total > 0.0 { m.adm / total } else { 0.0 };
        }
        hd.members.sort_by(|a, b| {
            b.adm
                .partial_cmp(&a.adm)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    districts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel;

    #[test]
    fn every_house_district_contains_school_districts() {
        let districts = house_districts(&panel::panel());
        assert_eq!(districts.len(), HOUSE_DISTRICTS);
        assert!(districts.iter().all(|hd| !hd.members.is_empty()));
    }

    #[test]
    fn every_district_in_the_panel_is_placed() {
        // A district missing from the crosswalk would silently vanish from every House district
        // total, and the totals would still look plausible because they would still be large.
        let panel = panel::panel();
        let all = overlaps();
        let placed: std::collections::BTreeSet<&str> = all.iter().map(|o| o.irn.as_str()).collect();
        let missing: Vec<&str> = panel
            .iter()
            .map(|r| r.irn.as_str())
            .filter(|irn| !placed.contains(irn))
            .collect();
        assert!(
            missing.is_empty(),
            "districts with no House district: {missing:?}"
        );
    }

    #[test]
    fn each_districts_shares_sum_to_one() {
        let all = overlaps();
        let mut totals: BTreeMap<&str, f64> = BTreeMap::new();
        for o in &all {
            *totals.entry(o.irn.as_str()).or_default() += o.share;
        }
        for (irn, total) in totals {
            assert!(
                (total - 1.0).abs() < 1e-6,
                "{irn}: shares sum to {total}, not one"
            );
        }
    }

    /// The only accuracy claim an apportionment like this can make, and it must hold exactly.
    ///
    /// Every district's shares sum to one, so summing the 99 House districts returns the statewide
    /// figure. If it did not, some district's money would have been created or destroyed by the
    /// act of dividing it, and a reader adding the House districts up would find a discrepancy
    /// with no explanation on the page.
    #[test]
    fn apportioned_totals_reconcile_to_the_statewide_figures() {
        let panel = panel::panel();
        let districts = house_districts(&panel);

        for (label, apportioned, expected) in [
            (
                "enrolled ADM",
                districts.iter().map(|h| h.adm).sum::<f64>(),
                panel.iter().map(DistrictRecord::base_cost_adm).sum::<f64>(),
            ),
            (
                "realized aid",
                districts.iter().map(|h| h.realized_aid).sum::<f64>(),
                panel.iter().map(DistrictRecord::realized_aid).sum::<f64>(),
            ),
            (
                "guarantee",
                districts.iter().map(|h| h.guarantee).sum::<f64>(),
                panel.iter().map(|r| r.guarantee).sum::<f64>(),
            ),
            (
                "categorical funding",
                districts.iter().map(|h| h.categorical_funding).sum::<f64>(),
                panel
                    .iter()
                    .map(DistrictRecord::categorical_funding)
                    .sum::<f64>(),
            ),
        ] {
            let scale = expected.abs().max(1.0);
            assert!(
                (apportioned - expected).abs() / scale < 1e-9,
                "{label}: apportioned {apportioned:.2} against statewide {expected:.2}"
            );
        }
    }

    #[test]
    fn most_districts_span_more_than_one_house_district() {
        // The fact that rules out the county page's design. If this ever stopped being true the
        // simpler approach would be available, and the page's framing would need revisiting.
        let all = overlaps();
        let mut reach: BTreeMap<&str, usize> = BTreeMap::new();
        for o in &all {
            *reach.entry(o.irn.as_str()).or_default() += 1;
        }
        let split = reach.values().filter(|n| **n > 1).count();
        assert!(
            split > reach.len() / 2,
            "{split} of {} districts span more than one House district",
            reach.len()
        );
        assert!(
            reach.values().copied().max().unwrap_or(0) >= 10,
            "expected at least one district spread across ten or more"
        );
    }

    #[test]
    fn a_member_share_is_of_the_house_district_not_of_the_school_district() {
        // Two shares travel with every membership and they answer opposite questions: how much of
        // the school district is here, and how much of here is that school district. Confusing
        // them would put "100%" beside a small district that happens to lie wholly inside a large
        // House district.
        let districts = house_districts(&panel::panel());
        for hd in &districts {
            let total: f64 = hd.members.iter().map(|m| m.share_of_house_district).sum();
            assert!(
                (total - 1.0).abs() < 1e-6,
                "House district {}: member shares sum to {total}",
                hd.number
            );
        }
        let wholly = districts
            .iter()
            .flat_map(|hd| &hd.members)
            .find(|m| m.wholly_inside)
            .expect("some district lies wholly inside one House district");
        assert!((wholly.share - 1.0).abs() < 1e-6);
    }
}
