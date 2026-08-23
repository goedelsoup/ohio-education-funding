//! Ohio's legislative districts, and the apportionment that makes them comparable.
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

/// The crosswalk, one row per (chamber, district, seat) with population in it.
const CROSSWALK: &str = include_str!("../fixtures/legislative-district-crosswalk.csv");

/// The header this loader was written against, so a regenerated crosswalk fails loudly.
const EXPECTED_HEADER: &str = "chamber,irn,district,population,population_under_18,share";

/// The columns of [`EXPECTED_HEADER`], named where they are read.
mod column {
    pub const CHAMBER: usize = 0;
    pub const IRN: usize = 1;
    pub const DISTRICT: usize = 2;
    pub const POPULATION: usize = 3;
    pub const POPULATION_UNDER_18: usize = 4;
    pub const SHARE: usize = 5;
}

/// Ohio's House has ninety-nine seats and its Senate thirty-three. Pinned because a crosswalk
/// producing a different count would mean a bad join.
pub const HOUSE_DISTRICTS: usize = 99;
/// Thirty-three, each exactly three House districts.
pub const SENATE_DISTRICTS: usize = 33;

/// Which chamber a seat belongs to.
///
/// # Why both, when the Senate could be derived
///
/// Ohio's constitution requires each Senate district to be **exactly three whole House
/// districts**, and the block data confirms it with no exceptions — so the Senate apportionment
/// could be produced by grouping the House one, and would be exact rather than approximate.
///
/// It is read from the Bureau's published Senate file anyway, and the constitutional rule is
/// asserted as a *finding* rather than assumed as a shortcut. The composition is not sequential —
/// Senate 2 is House 44, 75 and 89 — so a derivation would need the mapping regardless, and
/// reading it is how the corpus learns that the rule holds this cycle rather than hoping it does.
///
/// The Senate view is also substantially less of an estimate. Seats three times larger mean
/// **392 of 609 school districts sit wholly inside one**, against 270 for the House.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Chamber {
    /// Ninety-nine seats.
    House,
    /// Thirty-three, each exactly three of the above.
    Senate,
}

impl Chamber {
    /// The key the crosswalk uses.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::House => "house",
            Self::Senate => "senate",
        }
    }

    /// How many seats it has.
    #[must_use]
    pub const fn seats(self) -> usize {
        match self {
            Self::House => HOUSE_DISTRICTS,
            Self::Senate => SENATE_DISTRICTS,
        }
    }

    /// What to call one of its members' districts in prose.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::House => "House district",
            Self::Senate => "Senate district",
        }
    }

    /// Both, for a caller that wants to build every page.
    #[must_use]
    pub const fn all() -> [Self; 2] {
        [Self::House, Self::Senate]
    }
}

/// One school district's presence in one legislative seat.
#[derive(Debug, Clone, PartialEq)]
pub struct Overlap {
    /// Which chamber the seat belongs to.
    pub chamber: Chamber,
    /// The school district, by IRN.
    pub irn: String,
    /// The seat, zero-padded to three characters: `001` through `099` or `033`.
    pub district: String,
    /// 2020 census population of the blocks in both.
    pub population: f64,
    /// Of that, the population under 18 — the weight the share is computed from.
    pub population_under_18: f64,
    /// This House district's share of the district's under-18 population. Sums to one per district.
    pub share: f64,
}

/// Every overlap with population in it, for one chamber.
///
/// # Panics
///
/// If the crosswalk's header is not the one this loader expects. Reading shifted columns would
/// apportion by the wrong number and produce a page of plausible wrong totals.
#[must_use]
pub fn overlaps(chamber: Chamber) -> Vec<Overlap> {
    edfund_core::csv::rows(CROSSWALK, EXPECTED_HEADER)
        .filter_map(|row| {
            if row.str(column::CHAMBER) != chamber.key() {
                return None;
            }
            Some(Overlap {
                chamber,
                irn: row.str(column::IRN).to_string(),
                district: row.str(column::DISTRICT).to_string(),
                population: row.num(column::POPULATION)?,
                population_under_18: row.num(column::POPULATION_UNDER_18)?,
                share: row.num(column::SHARE)?,
            })
        })
        .collect()
}

/// One legislative seat, with the school districts in it and their apportioned figures.
#[derive(Debug, Clone, PartialEq)]
pub struct LegislativeDistrict {
    /// Which chamber.
    pub chamber: Chamber,
    /// The seat number, zero-padded to three characters.
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
    /// Whole districts, i.e. those with no population in any other seat of this chamber.
    pub districts_wholly_inside: usize,
}

/// One school district's membership of one seat, from the seat's point of view.
#[derive(Debug, Clone, PartialEq)]
pub struct Membership {
    /// The school district, by IRN.
    pub irn: String,
    /// Its name, carried so a page need not join back to the panel.
    pub name: String,
    /// The share of the *district* that lies in this House district.
    pub share: f64,
    /// The share of this *seat's* apportioned pupils that this school district provides.
    pub share_of_house_district: f64,
    /// Apportioned pupils and aid for this pair.
    pub adm: Adm,
    /// Apportioned state aid as the district receives it.
    pub realized_aid: Dollars,
    /// Whether the school district lies entirely within this seat.
    pub wholly_inside: bool,
}

/// Apportion the panel across one chamber's seats.
///
/// Each district's figures are multiplied by its share of that House district and summed. Because
/// the shares sum to one per district, the result sums to the statewide total exactly — which is
/// the one accuracy guarantee an apportionment like this can offer and is asserted in the tests.
#[must_use]
pub fn legislative_districts(
    panel: &[DistrictRecord],
    chamber: Chamber,
) -> Vec<LegislativeDistrict> {
    let by_irn: BTreeMap<&str, &DistrictRecord> =
        panel.iter().map(|r| (r.irn.as_str(), r)).collect();

    // How many House districts each district reaches, so "wholly inside" is a fact about the
    // district rather than about the row being looked at.
    let all = overlaps(chamber);
    let mut reach: BTreeMap<&str, usize> = BTreeMap::new();
    for o in &all {
        *reach.entry(o.irn.as_str()).or_default() += 1;
    }

    let mut out: BTreeMap<String, LegislativeDistrict> = BTreeMap::new();
    for o in &all {
        let Some(record) = by_irn.get(o.irn.as_str()) else {
            continue;
        };
        let entry = out
            .entry(o.district.clone())
            .or_insert_with(|| LegislativeDistrict {
                chamber,
                number: o.district.clone(),
                members: Vec::new(),
                adm: 0.0,
                realized_aid: 0.0,
                base_cost_state_share: 0.0,
                categorical_funding: 0.0,
                guarantee: 0.0,
                districts_on_guarantee: 0,
                districts_at_minimum_state_share: 0,
                districts_wholly_inside: 0,
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

    let mut districts: Vec<LegislativeDistrict> = out.into_values().collect();
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
    fn every_seat_in_both_chambers_contains_school_districts() {
        for chamber in Chamber::all() {
            let districts = legislative_districts(&panel::panel(), chamber);
            assert_eq!(districts.len(), chamber.seats(), "{chamber:?}");
            assert!(
                districts.iter().all(|d| !d.members.is_empty()),
                "{chamber:?}"
            );
        }
    }

    /// Ohio's constitution requires each Senate district to be exactly three whole House districts.
    ///
    /// Asserted from the crosswalk rather than assumed, and the composition is *derived* here
    /// rather than looked up: which three House seats make up each Senate seat falls out of the
    /// shares themselves, because a school district's share of a Senate seat must be the sum of
    /// its shares of that seat's three House districts. The mapping is not sequential — Senate 2
    /// is House 44, 75 and 89 — so recovering it is a real reconstruction of the Senate file from
    /// the House one.
    ///
    /// # What this used to assert
    ///
    /// `assert_eq!(house.len(), senate.len() * 3)` — that is, `99 == 33 * 3`, two constants this
    /// module declares itself. It held against a Senate crosswalk replaced wholesale by a
    /// fabrication (#125), because nothing in it read a Senate share. The pupil reconciliation
    /// beside it was real but weak: any partition of the state reconciles, however the seats are
    /// drawn.
    #[test]
    fn every_senate_district_is_exactly_three_whole_house_districts() {
        let panel = panel::panel();
        let house = legislative_districts(&panel, Chamber::House);
        let senate = legislative_districts(&panel, Chamber::Senate);

        // Apportioned pupils must reconcile between the chambers, since both partition the state.
        let h: f64 = house.iter().map(|d| d.adm).sum();
        let s: f64 = senate.iter().map(|d| d.adm).sum();
        assert!(
            (h - s).abs() / h < 1e-9,
            "house apportions {h:.4} pupils and senate {s:.4}"
        );

        // `seat number -> school district IRN -> the share of that district lying in the seat`.
        let shares = |seats: &[LegislativeDistrict]| -> BTreeMap<String, BTreeMap<String, f64>> {
            seats
                .iter()
                .map(|seat| {
                    (
                        seat.number.clone(),
                        seat.members
                            .iter()
                            .map(|m| (m.irn.clone(), m.share))
                            .collect(),
                    )
                })
                .collect()
        };
        let house_shares = shares(&house);
        let senate_shares = shares(&senate);

        // A House seat lies inside a Senate seat only if every school district reaching the House
        // seat reaches the Senate seat by at least as much. On a real map exactly one Senate seat
        // satisfies that for each House seat; on a fabricated one, none does or several do.
        // The crosswalk stores `share` to eight decimal places, so each is rounded by up to 5e-9.
        // Summing three House shares and comparing against a fourth rounded figure admits four of
        // those, and nothing else: 2e-8. The worst actually observed is asserted below, so this
        // cannot quietly become a tolerance chosen to make the test pass.
        const ROUNDING: f64 = 2e-8;
        let mut worst = 0.0_f64;

        let mut composition: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for (number, reach) in &house_shares {
            let containing: Vec<&str> = senate_shares
                .iter()
                .filter(|(_, senate_reach)| {
                    reach.iter().all(|(irn, share)| {
                        senate_reach
                            .get(irn)
                            .is_some_and(|whole| *whole >= share - 1e-9)
                    })
                })
                .map(|(number, _)| number.as_str())
                .collect();
            assert_eq!(
                containing.len(),
                1,
                "House {number} lies inside {} Senate districts, not one: {containing:?}",
                containing.len()
            );
            composition.entry(containing[0]).or_default().push(number);
        }

        assert_eq!(
            composition.len(),
            SENATE_DISTRICTS,
            "the House seats reconstruct {} Senate districts",
            composition.len()
        );

        for (number, members) in &composition {
            assert_eq!(
                members.len(),
                3,
                "Senate {number} is made of {} House districts: {members:?}",
                members.len()
            );

            // Exactly three *whole* House districts: the Senate seat's reach is the sum of theirs,
            // district by district, with nothing left over on either side.
            let seat = &senate_shares[*number];
            let mut summed: BTreeMap<&str, f64> = BTreeMap::new();
            for member in members {
                for (irn, share) in &house_shares[*member] {
                    *summed.entry(irn.as_str()).or_default() += share;
                }
            }
            assert_eq!(
                summed.len(),
                seat.len(),
                "Senate {number} reaches {} districts, its three House seats {}",
                seat.len(),
                summed.len()
            );
            for (irn, share) in seat {
                let from_house = summed.get(irn.as_str()).copied().unwrap_or_default();
                worst = worst.max((from_house - share).abs());
                assert!(
                    (from_house - share).abs() <= ROUNDING,
                    "Senate {number}, district {irn}: the seat claims {share:.9} and its three \
                     House districts supply {from_house:.9}"
                );
            }
        }

        // The measured worst case, so the tolerance above stays honest.
        assert!(
            worst < 1.2e-8,
            "the worst reconstruction gap is {worst:e}, which has grown"
        );

        // And the Senate is the less approximate view, because its seats are larger.
        let whole =
            |v: &[LegislativeDistrict]| v.iter().map(|d| d.districts_wholly_inside).sum::<usize>();
        assert!(
            whole(&senate) > whole(&house),
            "senate holds {} school districts whole against the house's {}",
            whole(&senate),
            whole(&house)
        );
    }

    #[test]
    fn every_house_district_contains_school_districts() {
        let districts = legislative_districts(&panel::panel(), Chamber::House);
        assert_eq!(districts.len(), HOUSE_DISTRICTS);
        assert!(districts.iter().all(|hd| !hd.members.is_empty()));
    }

    #[test]
    fn every_district_in_the_panel_is_placed() {
        // A district missing from the crosswalk would silently vanish from every House district
        // total, and the totals would still look plausible because they would still be large.
        let panel = panel::panel();
        let all = overlaps(Chamber::House);
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
        let all = overlaps(Chamber::House);
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
        let districts = legislative_districts(&panel, Chamber::House);

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
        let all = overlaps(Chamber::House);
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
        let districts = legislative_districts(&panel::panel(), Chamber::House);
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
