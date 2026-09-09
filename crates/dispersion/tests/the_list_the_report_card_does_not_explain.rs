//! What federal identification selects, read against the rating it is said to select on.
//!
//! `accountability-regime/essa` states the coupling as its thesis: Ohio "identifies schools by
//! rank-ordering Ohio's own report card rather than any federal measure", so "a change to the
//! state rating propagates into federal consequences without a federal act." Both halves of that
//! join have been committed since the building extract was wired — 408 identifications in
//! [`dispersion::identified`], 3,318 buildings in [`dispersion::building`] — and nothing joined
//! them.
//!
//! # What the join can and cannot decide
//!
//! Neither published building file carries the **overall** rating, which is the measure the plan
//! actually identifies on. What is published is the achievement component: a star rating and the
//! Performance Index behind it. So nothing here can reproduce the department's cut, and nothing
//! here is trying to. The question this file can answer is the weaker and more useful one — how
//! much of the list the published rating *accounts for* — and the answer is: much less than a
//! reader would assume from the node's own summary.
//!
//! The second limit is [`dispersion::identified`]'s: CSI selects among Title I served schools and
//! no fixture here carries a service flag. That cuts one way only. It explains why a low-rated
//! building might be **absent**; it does not explain why a well-rated one is **present**, and
//! eleven are.

use std::collections::{BTreeMap, BTreeSet};

use dispersion::building::{self, Building};
use dispersion::identified::{self, Identified};

/// The identification lists, joined to the building each one names.
fn joined() -> Vec<(Identified, Building)> {
    let by_irn: BTreeMap<String, Building> = building::buildings()
        .into_iter()
        .map(|b| (b.irn.clone(), b))
        .collect();
    identified::identifications()
        .into_iter()
        .filter_map(|i| by_irn.get(&i.building_irn).cloned().map(|b| (i, b)))
        .collect()
}

/// Rank on the 2024-25 Performance Index, worst first, one-based. Unrated buildings have none.
fn ranks() -> BTreeMap<String, usize> {
    building::by_index()
        .into_iter()
        .enumerate()
        .map(|(i, b)| (b.irn, i + 1))
        .collect()
}

/// The whole-school tier, and the route each of its schools took into it.
fn csi() -> Vec<(Identified, Building)> {
    joined()
        .into_iter()
        .filter(|(i, _)| i.status == "csi")
        .collect()
}

/// Every identification names a building the report card rates.
///
/// The join is total, which is worth pinning because it is what makes every count below a count
/// over the same population rather than over whatever survived a merge. It is also not
/// guaranteed: the two files come from different department releases and the identification list
/// carries three closed or inactive buildings, which a report card could reasonably omit and
/// does not.
#[test]
fn every_identification_names_a_building_the_report_card_carries() {
    let all = identified::identifications();
    assert_eq!(all.len(), 408);
    assert_eq!(
        joined().len(),
        all.len(),
        "an identification named no building"
    );

    let closed = all
        .iter()
        .filter(|i| i.open_closed != "Open" && !i.open_closed.is_empty())
        .count();
    assert_eq!(closed, 4, "closed or inactive buildings still on the list");
}

/// The list is three identification cycles deep, so it is not a bottom-five-per-cent snapshot.
///
/// This matters because the node states the rule — "the lowest-performing 5% of Title I served
/// schools" — and a reader who takes the list as that cut reads 231 schools as the current bottom
/// of a population. The arithmetic refuses *that* reading without needing a Title I count: five
/// per cent of **every building in the state** is 165, and there are 231 CSI schools, so the 231
/// cannot be a five per cent cut of anything.
///
/// It does not by itself establish what the excess is. Ohio's plan names three other routes into
/// CSI — a graduation rate at or below 67%, chronic absenteeism for schools with fewer than three
/// rated components, and failure to exit ATSI — and any of them could carry a cycle past the
/// threshold. What establishes the answer is the `year_identified` column, which says directly
/// that three cycles are on the list at once, and says how the 231 accumulated.
#[test]
fn the_whole_school_tier_is_larger_than_five_per_cent_of_every_building_in_the_state() {
    let csi = csi();
    assert_eq!(csi.len(), 231);

    let ceiling = (building::buildings().len() as f64 * 0.05).floor() as usize;
    assert_eq!(ceiling, 165);
    assert!(
        csi.len() > ceiling,
        "{} CSI schools against a five per cent ceiling of {ceiling} over every building in the \
         state; if this ever falls below, the list has stopped being cumulative",
        csi.len()
    );

    let mut cohorts: BTreeMap<String, usize> = BTreeMap::new();
    for (id, _) in &csi {
        let route = if id.escalated() {
            "escalated".to_string()
        } else {
            id.year_identified.clone()
        };
        *cohorts.entry(route).or_default() += 1;
    }
    assert_eq!(cohorts["2018"], 100);
    assert_eq!(cohorts["2022"], 27);
    assert_eq!(cohorts["2025"], 62);
    assert_eq!(cohorts["escalated"], 42);
    assert_eq!(cohorts.len(), 4, "a fifth route appeared: {cohorts:?}");
}

/// A hundred schools have carried the identification for seven years against a three-year exit.
///
/// `dispersion::identified` asserts only that the 2018 cohort is non-empty. It is 43% of the
/// whole-school tier, it has survived the 2022 and 2025 cycles, and every one of its hundred
/// schools is still open.
///
/// What distinguishes it is not the index — its median sits within three points of the cohorts
/// identified since — but attendance. The median 2018-cohort school has **four in five students
/// chronically absent**, against one in five statewide, and 81 of the hundred are above half.
/// That is the population Ohio's exit criteria have not moved, and it is not a population that
/// looks like a school failing to teach.
#[test]
fn the_cohort_that_has_not_exited_since_twenty_eighteen_is_an_attendance_population() {
    let stale: Vec<(Identified, Building)> = csi()
        .into_iter()
        .filter(|(i, _)| i.year_identified == "2018")
        .collect();
    assert_eq!(stale.len(), 100);
    assert!(
        stale.iter().all(|(i, _)| i.open_closed == "Open"),
        "the 2018 cohort is no longer entirely open, which would be an exit route this reads as \
         stalling"
    );

    let absent = |b: &Building| b.chronic_absenteeism.expect("published for all but five");
    let above_half = stale.iter().filter(|(_, b)| absent(b) > 50.0).count();
    assert_eq!(above_half, 81);

    let mut cohort: Vec<f64> = stale.iter().map(|(_, b)| absent(b)).collect();
    cohort.sort_by(f64::total_cmp);
    let mut state: Vec<f64> = building::buildings()
        .iter()
        .filter_map(|b| b.chronic_absenteeism)
        .collect();
    state.sort_by(f64::total_cmp);
    let median = |v: &[f64]| v[v.len() / 2];
    assert!(
        (median(&cohort) - 80.7).abs() < 0.05,
        "the 2018 cohort's median chronic absenteeism is {:.1}",
        median(&cohort)
    );
    assert!(
        (median(&state) - 20.8).abs() < 0.05,
        "the statewide building median is {:.1}",
        median(&state)
    );
    assert!(
        median(&cohort) > 3.0 * median(&state),
        "the cohort is no longer an outlier on attendance and this reading needs redoing"
    );
}

/// The subgroup route puts a different kind of school into the same tier.
///
/// `dispersion::identified` records that 42 CSI schools arrived by failing to exit ATSI, and
/// `school/sheridan-high-school` records that one of them is a four-star building. Across all 42
/// the pattern holds: **every CSI school rated three stars or better arrived by escalation**, and
/// not one of the 189 direct identifications is above two.
///
/// On the index the two routes barely overlap. Four of the 41 rated escalations sit in the bottom
/// five per cent of buildings, against 75 of 185 direct ones, and the highest-ranked escalation
/// sits above the state median. A tier holding both is holding two different findings under one
/// name, and any consequence attached to CSI membership falls on both.
#[test]
fn every_well_rated_school_in_the_tier_arrived_through_a_subgroup() {
    let csi = csi();
    let (escalated, direct): (Vec<_>, Vec<_>) = csi.into_iter().partition(|(i, _)| i.escalated());
    assert_eq!(escalated.len(), 42);
    assert_eq!(direct.len(), 189);

    let stars = |rows: &[(Identified, Building)], at_least: f64| {
        rows.iter()
            .filter(|(_, b)| b.achievement_stars.is_some_and(|s| s >= at_least))
            .count()
    };
    assert_eq!(stars(&direct, 3.0), 0);
    assert_eq!(stars(&escalated, 3.0), 11);

    let ranks = ranks();
    let cut = (building::by_index().len() as f64 * 0.05) as usize;
    let inside = |rows: &[(Identified, Building)]| {
        rows.iter()
            .filter(|(_, b)| ranks.get(&b.irn).is_some_and(|r| *r <= cut))
            .count()
    };
    assert_eq!(inside(&direct), 75);
    assert_eq!(inside(&escalated), 4);

    let highest = escalated
        .iter()
        .filter_map(|(_, b)| ranks.get(&b.irn))
        .max()
        .expect("41 of the 42 are rated");
    assert!(
        *highest > building::by_index().len() / 2,
        "the highest-ranked escalation is {highest} of {}, no longer above the median",
        building::by_index().len()
    );
}

/// Between the worst unlisted building and the best listed one lies most of the state.
///
/// The lowest Performance Index on none of the three lists is **30.3** and the highest on the CSI
/// list is **89.9**. Between them sit 1,958 of the 3,151 rated buildings — 62 per cent of the
/// state — and inside that band the published achievement rating does not say who is identified.
///
/// The sharpest form of it is exact agreement: **113 published index values carry both a CSI
/// school and a building on no list at all**, and six of those pairs share an operating district,
/// so the same superintendent runs both. Cleveland Municipal's Anton Grdina and Alfred Benesch
/// both score 36.8; Benesch is CSI and Grdina is on nothing.
///
/// None of this is evidence that the department erred. The rating that identifies is the overall
/// one and this is one component of it; the eligible population is Title I served schools and this
/// is every building. It is evidence about what the **published** rating can support, which is
/// what a corpus quoting the published rating needs to know.
#[test]
fn the_published_index_does_not_separate_the_identified_from_the_rest() {
    let listed: BTreeSet<String> = identified::identifications()
        .into_iter()
        .map(|i| i.building_irn)
        .collect();
    let in_csi: BTreeSet<String> = csi().into_iter().map(|(_, b)| b.irn).collect();
    let rated = building::by_index();

    let index = |b: &Building| b.performance_index.expect("by_index drops the unrated");
    let worst_unlisted = rated
        .iter()
        .filter(|b| !listed.contains(&b.irn))
        .map(index)
        .fold(f64::MAX, f64::min);
    let best_csi = rated
        .iter()
        .filter(|b| in_csi.contains(&b.irn))
        .map(index)
        .fold(f64::MIN, f64::max);
    assert!((worst_unlisted - 30.3).abs() < 0.05, "{worst_unlisted:.1}");
    assert!((best_csi - 89.9).abs() < 0.05, "{best_csi:.1}");

    let overlap = rated
        .iter()
        .filter(|b| (worst_unlisted..=best_csi).contains(&index(b)))
        .count();
    assert_eq!(overlap, 1958);
    assert!(
        overlap * 2 > rated.len(),
        "{overlap} of {} buildings are in the band where membership is unpredicted",
        rated.len()
    );

    // Exact agreement on the published figure, which no threshold can separate.
    let mut at_value: BTreeMap<String, Vec<&Building>> = BTreeMap::new();
    for b in &rated {
        at_value
            .entry(format!("{:.1}", index(b)))
            .or_default()
            .push(b);
    }
    let contested: Vec<&Vec<&Building>> = at_value
        .values()
        .filter(|group| {
            group.iter().any(|b| in_csi.contains(&b.irn))
                && group.iter().any(|b| !listed.contains(&b.irn))
        })
        .collect();
    assert_eq!(contested.len(), 113);

    let same_operator = contested
        .iter()
        .flat_map(|group| {
            let mut by_operator: BTreeMap<&str, Vec<&&Building>> = BTreeMap::new();
            for b in group.iter() {
                by_operator
                    .entry(b.district_irn.as_str())
                    .or_default()
                    .push(b);
            }
            by_operator.into_values().collect::<Vec<_>>()
        })
        .filter(|group| {
            group.iter().any(|b| in_csi.contains(&b.irn))
                && group.iter().any(|b| !listed.contains(&b.irn))
        })
        .count();
    assert_eq!(same_operator, 6);
}

/// Anton Grdina is in the bottom one per cent, which narrows why it is on no list.
///
/// `school/anton-grdina` records three readings of its absence — not Title I served, served but
/// outside the bottom five per cent of that population, or identified in an earlier cycle and
/// exited — and says nothing held distinguishes them. Two of the three are now much harder to
/// hold.
///
/// The building ranks **23rd of 3,151** on the published index. Eighteen of the twenty-two
/// buildings below it are identified. And Alfred Benesch, operated by the same district, scores
/// the same 36.8 and is CSI on the 2025 cycle by the direct route. "Outside the bottom five per
/// cent" would have to be true of a building in the bottom one per cent while false of its
/// neighbour at the same published score.
///
/// What survives is Title I service, and the earlier-cycle exit — which the list makes look rare,
/// since it carries a hundred schools that have not managed it since 2018.
#[test]
fn the_corpus_exemplar_is_in_the_bottom_one_per_cent_of_the_state() {
    let ranks = ranks();
    let rated = building::by_index();
    let grdina = ranks["000828"];
    assert_eq!(grdina, 23);
    assert!(
        (grdina as f64) < 0.01 * rated.len() as f64,
        "{grdina} of {} is no longer the bottom one per cent",
        rated.len()
    );

    let listed: BTreeSet<String> = identified::identifications()
        .into_iter()
        .map(|i| i.building_irn)
        .collect();
    let below = &rated[..grdina - 1];
    assert_eq!(below.len(), 22);
    assert_eq!(below.iter().filter(|b| listed.contains(&b.irn)).count(), 18);

    let benesch = building::by_irn("005637").expect("Alfred Benesch is on the report card");
    let grdina_row = building::by_irn("000828").expect("Anton Grdina is on the report card");
    assert_eq!(benesch.district_irn, grdina_row.district_irn);
    assert_eq!(benesch.performance_index, grdina_row.performance_index);
    assert_eq!(benesch.achievement_stars, grdina_row.achievement_stars);
    assert!(listed.contains(&benesch.irn));
    assert!(!listed.contains(&grdina_row.irn));
}

/// The state's own worst *achievement* rating and the federal list agree on two fifths of it.
///
/// Ohio rates **347 buildings at one star** on achievement — one of the six components
/// R.C. 3302.03(D)(3) names, and the only one either published building file carries. The CSI
/// list holds 140 of them. A hundred and sixty-four are on no federal list at all — nearly half
/// the state's worst-rated buildings, outside the regime the node describes as identifying on
/// the state's rating.
///
/// The relation runs the other way too and more weakly than the summary implies. The 231 CSI
/// schools split 140 at one star, **86 above it**, and five the department did not rate at all —
/// so a majority of the tier is not among the buildings Ohio rates worst.
#[test]
fn the_states_worst_achievement_rating_and_the_federal_list_agree_on_two_fifths() {
    let listed: BTreeSet<String> = identified::identifications()
        .into_iter()
        .map(|i| i.building_irn)
        .collect();
    let in_csi: BTreeSet<String> = csi().into_iter().map(|(_, b)| b.irn).collect();

    let one_star: Vec<Building> = building::buildings()
        .into_iter()
        .filter(|b| b.achievement_stars == Some(1.0))
        .collect();
    assert_eq!(one_star.len(), 347);
    assert_eq!(
        one_star.iter().filter(|b| in_csi.contains(&b.irn)).count(),
        140
    );
    assert_eq!(
        one_star.iter().filter(|b| !listed.contains(&b.irn)).count(),
        164
    );

    let tier = csi();
    let at_star = |at: f64| {
        tier.iter()
            .filter(|(_, b)| b.achievement_stars == Some(at))
            .count()
    };
    let above_one = tier
        .iter()
        .filter(|(_, b)| b.achievement_stars.is_some_and(|s| s > 1.0))
        .count();
    let unrated = tier
        .iter()
        .filter(|(_, b)| b.achievement_stars.is_none())
        .count();
    assert_eq!(at_star(1.0), 140);
    assert_eq!(above_one, 86);
    assert_eq!(unrated, 5);
    assert_eq!(at_star(1.0) + above_one + unrated, tier.len());
}

/// What the sixteen-rung intervention ladder can reach, and where those schools already sit.
///
/// `intervention/more-rigorous-interventions` triggers on a CSI school that has not exited within
/// three years, and recorded the size of that population as "knowable and not known". It is 127 —
/// the 2018 and 2022 cohorts, holding 50,086 pupils. The 2025 cohorts are excluded whichever route
/// they took, the escalated 42 included: their ATSI clock started in 2022 but the plan's three
/// years run from the CSI identification, and theirs is 2025.
///
/// The node also describes three of the sixteen rungs — charter conversion, merger with a charter,
/// closure — as transfers into the community-school sector. **Eighty-two of the 127 are already
/// their own local education agency**: every one of the 64 dropout recovery schools and 18 of the
/// 20 community schools, holding 30,085 of the 50,086 pupils. Only 43 are traditional district
/// buildings, and they are the only ones a conversion or merger rung has anywhere to move.
#[test]
fn most_of_what_the_ladder_reaches_is_already_its_own_agency() {
    let reachable: Vec<(Identified, Building)> = csi()
        .into_iter()
        .filter(|(i, _)| matches!(i.year_identified.as_str(), "2018" | "2022"))
        .collect();
    assert_eq!(reachable.len(), 127);

    let pupils: f64 = reachable.iter().filter_map(|(_, b)| b.enrollment).sum();
    assert_eq!(pupils, 50_086.0);

    let own_agency: Vec<&(Identified, Building)> = reachable
        .iter()
        .filter(|(i, _)| i.building_irn == i.lea_irn)
        .collect();
    assert_eq!(own_agency.len(), 82);
    let held: f64 = own_agency.iter().filter_map(|(_, b)| b.enrollment).sum();
    assert_eq!(held, 30_085.0);

    let by_type = |kind: &str| {
        reachable
            .iter()
            .filter(|(i, _)| i.school_type == kind)
            .collect::<Vec<_>>()
    };
    let recovery = by_type("Dropout Recovery School");
    assert_eq!(recovery.len(), 64);
    assert!(
        recovery.iter().all(|(i, _)| i.building_irn == i.lea_irn),
        "a dropout recovery school in this population is operated by a district, which would put \
         it inside the sector the conversion rungs move a building out of"
    );
    assert_eq!(by_type("Traditional School").len(), 43);
    assert_eq!(reachable.len() - own_agency.len(), 45);
}
