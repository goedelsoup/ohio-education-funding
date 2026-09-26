//! `[L1]` split into the guarantee it contains and the funding it would contain without one.
//!
//! # Why `[L1]` needs splitting at all
//!
//! `[K] Formula Transition Supplement` tops a district up to `[L1]`, a FY2021 funding base, from a
//! total that already contains the guarantee:
//! `max([L1] − (foundation funding + [I] + transportation), 0)`. So `[K]` **backstops** `[I]`, and
//! retiring the guarantee has three coherent readings rather than two — Section 265.225 stands,
//! Section 265.225 is repealed alongside, or Section 265.225 is recomputed against a base that
//! never contained a guarantee either. See [`crate::policy::Backstop`].
//!
//! The third needs a quantity `[L1]` does not publish. This module supplies it from the
//! department's own two files, and the arithmetic that licenses reading one as a term of the other
//! is asserted in this module's tests rather than described.
//!
//! # The two files, and the identity between them
//!
//! [`fy2019`] is the FY2019 final payment report — the last year of the formula H.B. 110 replaced,
//! which paid `TRANSITIONAL GUARANTEE` as one of thirteen components of a district's total.
//! [`fy2021_base`] is the department's decomposition of `[L1]` into seven terms. The first of those
//! terms is the FY2021 final formula payment with the executive budget reductions restored, and it
//! **equals the FY2019 total, district for district**, because FY2020 and FY2021 foundation funding
//! for traditional districts were held flat to FY2019.
//!
//! That is what makes the FY2019 guarantee a term *inside* `[L1]` rather than a sibling of it, and
//! the department states the same thing in prose on the funding-bases workbook's `Introduction`
//! sheet: the guarantee *"default\[s\] to the funding base used for line A Base Cost"*. It is in
//! there and it is not itemised. See
//! [`dew-foundation-funding-bases`](../../../.yidam/catalog/dew-foundation-funding-bases.md).
//!
//! # Three districts where the attribution is not a plain lookup
//!
//! **Newbury Local dissolved into West Geauga Local.** The FY2019 report carries Newbury and the
//! base sheet does not, and West Geauga's first term exceeds its own FY2019 total by exactly
//! Newbury's — $1,048,132.43. So Newbury's guarantee of $469,077.69 is inside West Geauga's
//! `[L1]`, and [`guarantee_in_fy21_base`] puts it there. See [`DISSOLVED`].
//!
//! **Two districts' first terms are restated, and the department's own footnote says why.** Deer
//! Park Community's is $13,789.98 above its FY2019 total (a Taxation Department revision to TY16
//! certified values, Section 18 of Am. Sub. S.B. 51 of the 132nd General Assembly) and Hamilton
//! City's is $754,132.81 below (Section 265.227 moved its career-technical funding to Butler Tech
//! on joining that joint vocational district). Hamilton City drew no guarantee in FY2019, so only
//! Deer Park's attribution is approximate — by at most $13,789.98 against a guarantee of
//! $47,928.29, in a quantity this module totals at $256.5m. See [`RESTATED`].

use std::collections::BTreeMap;

use edfund_core::Dollars;

/// The committed extract of the FY2019 final traditional district payment report.
const FY19_FIXTURE: &str = include_str!("../fixtures/fy19-payment-report.csv");

const FY19_HEADER: &str = "irn,district,county,opportunity_grant,targeted_assistance,k3_literacy,\
     economic_disadvantaged,limited_english_proficiency,gifted,transportation,\
     special_education_additional,capacity_aid,graduation_bonus,third_grade_reading_bonus,\
     transitional_guarantee,career_technical,total_foundation_funding,funding_before_guarantee,\
     funding_within_guarantee,guarantee_base,cap_ratio";

/// The committed extract of the department's decomposition of `[L1]`.
const FY21_FIXTURE: &str = include_str!("../fixtures/fy21-funding-base.csv");

const FY21_HEADER: &str = "irn,district,county,foundation_funding_pre_reduction,\
     net_open_enrollment,net_excess_cost,scholarship_deduction,community_stem_deduction,\
     student_wellness_success,enrollment_growth_supplement,funding_base";

/// The district the FY2019 report carries that the FY2021 base sheet does not, and its successor.
///
/// Newbury Local SD dissolved into West Geauga Local SD. The department did not drop its funding:
/// the successor's `[L1]` first term carries it, so the guarantee inside that first term is the
/// sum of both districts'.
pub const DISSOLVED: &[(&str, &str)] = &[("047217", "047225")];

/// The districts whose `[L1]` first term is not their FY2019 total, and are not meant to be.
///
/// Deer Park Community SD (`043851`) and Hamilton City SD (`044107`), the two exceptions the
/// department's own *Line by Line* explanation names. Every other district agrees to the cent.
pub const RESTATED: &[&str] = &["043851", "044107"];

/// One district's FY2019 final payment, on the department's own lines.
///
/// The thirteen fields between [`Self::opportunity_grant`] and [`Self::career_technical`] are the
/// **capped components**, and they sum to [`Self::total_foundation_funding`]. That total is what
/// became the first term of `[L1]`.
#[derive(Debug, Clone)]
pub struct Fy2019 {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name, as the payment report spells it.
    pub name: String,
    /// County the department attributes the district to.
    pub county: String,
    /// The FY2019 formula's general slice, capped.
    pub opportunity_grant: Dollars,
    /// The capacity-weighted tier, capped.
    pub targeted_assistance: Dollars,
    /// A line the Fair School Funding Plan does not have. `formula-component/fsfp-*` has no
    /// successor to it; it was eliminated rather than renamed.
    pub k3_literacy: Dollars,
    /// The FY2019 name for what is now DPIA.
    pub economic_disadvantaged: Dollars,
    /// English learners, under the FY2019 spelling.
    pub limited_english_proficiency: Dollars,
    /// Gifted education funding, capped.
    pub gifted: Dollars,
    /// Capped transportation. Inside this total, and therefore inside `[L1]` — which is why `[L1]`
    /// exceeds `[H2]`, the base the guarantee itself compares against, by roughly this much.
    pub transportation: Dollars,
    /// Special education above the base, capped.
    pub special_education_additional: Dollars,
    /// Capacity aid, capped. Zero for most districts.
    pub capacity_aid: Dollars,
    /// The graduation bonus as paid. The report publishes this label twice; this is the paid one.
    pub graduation_bonus: Dollars,
    /// The third-grade reading bonus as paid.
    pub third_grade_reading_bonus: Dollars,
    /// **The quantity this module exists for.** The FY2019 formula's own hold-harmless, paid to 335
    /// of the 612 districts and totalling $256,985,672.30.
    pub transitional_guarantee: Dollars,
    /// Career-technical funding. Outside the FY2019 guarantee's reach — see
    /// [`Self::funding_within_guarantee`].
    pub career_technical: Dollars,
    /// The thirteen components' sum, and the first term of `[L1]`.
    pub total_foundation_funding: Dollars,
    /// The total before the guarantee is applied.
    pub funding_before_guarantee: Dollars,
    /// The same, less career-technical funding — exactly, on all 612 rows. The FY2019 guarantee did
    /// not protect career-technical funding, which is a fact about the instrument and not about the
    /// file.
    pub funding_within_guarantee: Dollars,
    /// What the guarantee held the district at.
    pub guarantee_base: Dollars,
    /// The cap, as a multiplier. Below one for 164 districts, and none of those is guaranteed.
    pub cap_ratio: f64,
}

/// One district's `[L1]`, on the seven terms the department defines it by.
#[derive(Debug, Clone)]
pub struct Fy2021Base {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name. Taken from the FY2019 report for the one row the base sheet leaves as `#N/A`.
    pub name: String,
    /// County the department attributes the district to.
    pub county: String,
    /// The FY2021 final formula payment with the executive budget reductions restored — and the
    /// FY2019 total, for every district but the two in [`RESTATED`].
    pub foundation_funding_pre_reduction: Dollars,
    /// Net open enrolment for K-12 students.
    pub net_open_enrollment: Dollars,
    /// FY2022 net excess cost. Zero on every row: the workbook says the figure was not yet
    /// available when it was last revised in June 2022.
    pub net_excess_cost: Dollars,
    /// The FY2021 total scholarship transfer, as a negative.
    pub scholarship_deduction: Dollars,
    /// The FY2021 community and STEM school deduction, as a negative.
    pub community_stem_deduction: Dollars,
    /// FY2021 student wellness and success funding.
    pub student_wellness_success: Dollars,
    /// The FY2021 enrolment growth supplement.
    pub enrollment_growth_supplement: Dollars,
    /// `[L1]` — the seven terms' sum, to the cent on all 611 rows.
    pub funding_base: Dollars,
}

/// The FY2019 final payment report, in IRN order.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against, by way of
/// [`edfund_core::csv::rows`].
#[must_use]
pub fn fy2019() -> Vec<Fy2019> {
    edfund_core::csv::rows(FY19_FIXTURE, FY19_HEADER)
        .map(|row| Fy2019 {
            irn: row.str(0).to_string(),
            name: row.str(1).to_string(),
            county: row.str(2).to_string(),
            opportunity_grant: row.num(3).unwrap_or(0.0),
            targeted_assistance: row.num(4).unwrap_or(0.0),
            k3_literacy: row.num(5).unwrap_or(0.0),
            economic_disadvantaged: row.num(6).unwrap_or(0.0),
            limited_english_proficiency: row.num(7).unwrap_or(0.0),
            gifted: row.num(8).unwrap_or(0.0),
            transportation: row.num(9).unwrap_or(0.0),
            special_education_additional: row.num(10).unwrap_or(0.0),
            capacity_aid: row.num(11).unwrap_or(0.0),
            graduation_bonus: row.num(12).unwrap_or(0.0),
            third_grade_reading_bonus: row.num(13).unwrap_or(0.0),
            transitional_guarantee: row.num(14).unwrap_or(0.0),
            career_technical: row.num(15).unwrap_or(0.0),
            total_foundation_funding: row.num(16).unwrap_or(0.0),
            funding_before_guarantee: row.num(17).unwrap_or(0.0),
            funding_within_guarantee: row.num(18).unwrap_or(0.0),
            guarantee_base: row.num(19).unwrap_or(0.0),
            cap_ratio: row.num(20).unwrap_or(0.0),
        })
        .collect()
}

impl Fy2019 {
    /// The thirteen capped components, in the order the report prints them.
    #[must_use]
    pub fn components(&self) -> [Dollars; 13] {
        [
            self.opportunity_grant,
            self.targeted_assistance,
            self.k3_literacy,
            self.economic_disadvantaged,
            self.limited_english_proficiency,
            self.gifted,
            self.transportation,
            self.special_education_additional,
            self.capacity_aid,
            self.graduation_bonus,
            self.third_grade_reading_bonus,
            self.transitional_guarantee,
            self.career_technical,
        ]
    }
}

/// The department's decomposition of `[L1]`, in IRN order.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against, by way of
/// [`edfund_core::csv::rows`].
#[must_use]
pub fn fy2021_base() -> Vec<Fy2021Base> {
    edfund_core::csv::rows(FY21_FIXTURE, FY21_HEADER)
        .map(|row| Fy2021Base {
            irn: row.str(0).to_string(),
            name: row.str(1).to_string(),
            county: row.str(2).to_string(),
            foundation_funding_pre_reduction: row.num(3).unwrap_or(0.0),
            net_open_enrollment: row.num(4).unwrap_or(0.0),
            net_excess_cost: row.num(5).unwrap_or(0.0),
            scholarship_deduction: row.num(6).unwrap_or(0.0),
            community_stem_deduction: row.num(7).unwrap_or(0.0),
            student_wellness_success: row.num(8).unwrap_or(0.0),
            enrollment_growth_supplement: row.num(9).unwrap_or(0.0),
            funding_base: row.num(10).unwrap_or(0.0),
        })
        .collect()
}

impl Fy2021Base {
    /// The seven terms, in the order the department's `Introduction` sheet defines them.
    #[must_use]
    pub fn terms(&self) -> [Dollars; 7] {
        [
            self.foundation_funding_pre_reduction,
            self.net_open_enrollment,
            self.net_excess_cost,
            self.scholarship_deduction,
            self.community_stem_deduction,
            self.student_wellness_success,
            self.enrollment_growth_supplement,
        ]
    }
}

/// How much of each district's `[L1]` is the FY2019 guarantee, by IRN.
///
/// The FY2019 `TRANSITIONAL GUARANTEE`, plus a dissolved predecessor's where one was folded in.
/// Keyed on the FY2021 base sheet's districts, so a district with no `[L1]` has no entry.
#[must_use]
pub fn guarantee_in_fy21_base() -> BTreeMap<String, Dollars> {
    let paid: BTreeMap<String, Dollars> = fy2019()
        .into_iter()
        .map(|row| (row.irn, row.transitional_guarantee))
        .collect();
    let mut out: BTreeMap<String, Dollars> = fy2021_base()
        .into_iter()
        .map(|row| {
            let own = paid.get(&row.irn).copied().unwrap_or(0.0);
            (row.irn, own)
        })
        .collect();
    for (gone, successor) in DISSOLVED {
        let carried = paid.get(*gone).copied().unwrap_or(0.0);
        if let Some(held) = out.get_mut(*successor) {
            *held += carried;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cent, as the tolerance for an identity over published dollars.
    const CENT: Dollars = 0.005;

    #[test]
    fn the_thirteen_components_are_the_fy2019_total() {
        let report = fy2019();
        assert_eq!(report.len(), 612);
        for row in &report {
            let sum: Dollars = row.components().iter().sum();
            assert!(
                (sum - row.total_foundation_funding).abs() < CENT,
                "{} components sum to {sum:.2}, total is {:.2}",
                row.irn,
                row.total_foundation_funding
            );
        }
    }

    /// The FY2019 guarantee did not reach career-technical funding, and this is where that is
    /// asserted rather than described. `FUNDING BEFORE GUARANTEE` less `FUNDING WITHIN GUARANTEE`
    /// is career-technical funding on every row, so the guarantee's comparison was made against a
    /// total from which career-tech had been removed.
    #[test]
    fn the_fy2019_guarantee_did_not_cover_career_technical_funding() {
        for row in fy2019() {
            let outside = row.funding_before_guarantee - row.funding_within_guarantee;
            assert!(
                (outside - row.career_technical).abs() < CENT,
                "{} holds {outside:.2} outside the guarantee, career-tech is {:.2}",
                row.irn,
                row.career_technical
            );
        }
    }

    /// No district is both held up by the guarantee and held down by the cap. They are the two
    /// halves of one provision, and a file in which a district was on both would be a file whose
    /// halves were computed against different populations.
    #[test]
    fn no_fy2019_district_is_both_guaranteed_and_capped() {
        let report = fy2019();
        let guaranteed = report
            .iter()
            .filter(|row| row.transitional_guarantee > 0.0)
            .count();
        let capped = report.iter().filter(|row| row.cap_ratio < 1.0).count();
        assert_eq!(guaranteed, 335);
        assert_eq!(capped, 164);
        for row in &report {
            assert!(
                !(row.transitional_guarantee > 0.0 && row.cap_ratio < 1.0),
                "{} is guaranteed at {:.2} and capped at {}",
                row.irn,
                row.transitional_guarantee,
                row.cap_ratio
            );
        }
    }

    #[test]
    fn the_seven_terms_are_l1() {
        let base = fy2021_base();
        assert_eq!(base.len(), 611);
        for row in &base {
            let sum: Dollars = row.terms().iter().sum();
            assert!(
                (sum - row.funding_base).abs() < CENT,
                "{} terms sum to {sum:.2}, [L1] is {:.2}",
                row.irn,
                row.funding_base
            );
        }
    }

    /// The licence for the split, and the whole of it: `[L1]`'s first term is the FY2019 total on
    /// every district but the two the department's footnote names, once a dissolved predecessor's
    /// funding is folded into its successor.
    ///
    /// The exceptions are asserted as an exact set rather than as a count. A third district
    /// appearing here is the file having stopped meaning what this module reads it to mean, and a
    /// count would absorb one arrival by losing one departure.
    #[test]
    fn l1_is_built_on_the_fy2019_total() {
        let fy19: BTreeMap<String, Dollars> = fy2019()
            .into_iter()
            .map(|row| (row.irn, row.total_foundation_funding))
            .collect();
        let mut expected = fy19.clone();
        for (gone, successor) in DISSOLVED {
            let carried = fy19[*gone];
            *expected
                .get_mut(*successor)
                .expect("successor is a district") += carried;
            expected.remove(*gone);
        }

        let mut differ: Vec<String> = Vec::new();
        for row in fy2021_base() {
            let want = expected
                .get(&row.irn)
                .copied()
                .expect("every base district has an FY2019 row");
            if (row.foundation_funding_pre_reduction - want).abs() >= CENT {
                differ.push(row.irn);
            }
        }
        assert_eq!(
            differ, RESTATED,
            "the restated districts are not the two named"
        );
    }

    /// The dissolution is a fold and not a drop, to the cent.
    #[test]
    fn a_dissolved_districts_funding_is_inside_its_successors_base() {
        let fy19: BTreeMap<String, Fy2019> = fy2019()
            .into_iter()
            .map(|row| (row.irn.clone(), row))
            .collect();
        let base: BTreeMap<String, Fy2021Base> = fy2021_base()
            .into_iter()
            .map(|row| (row.irn.clone(), row))
            .collect();
        for (gone, successor) in DISSOLVED {
            let carried = fy19[*gone].total_foundation_funding;
            let over = base[*successor].foundation_funding_pre_reduction
                - fy19[*successor].total_foundation_funding;
            assert!(
                (over - carried).abs() < CENT,
                "{successor} carries {over:.2} over its own FY2019 total; \
                 {gone} was paid {carried:.2}"
            );
        }
        assert!((fy19["047217"].total_foundation_funding - 1_048_132.43).abs() < CENT);
        assert!((fy19["047217"].transitional_guarantee - 469_077.69).abs() < CENT);
    }

    /// The quantity the third reading of Section 265.225 is priced from.
    #[test]
    fn the_guarantee_inside_l1_totals_what_fy2019_paid() {
        let inside = guarantee_in_fy21_base();
        assert_eq!(inside.len(), 611);

        let paid: Dollars = fy2019().iter().map(|row| row.transitional_guarantee).sum();
        let attributed: Dollars = inside.values().sum();
        assert!(
            (attributed - paid).abs() < CENT,
            "attributed {attributed:.2} of {paid:.2} paid"
        );
        assert!((paid - 256_985_672.30).abs() < CENT);

        // Newbury's is inside West Geauga's rather than lost with the district.
        assert!((inside["047225"] - 2_769_788.67).abs() < CENT);
    }

    /// Every district the FY2027 model carries has a guarantee figure, so the third reading is
    /// priced on the panel's own population rather than on part of it.
    #[test]
    fn the_panel_is_covered() {
        let inside = guarantee_in_fy21_base();
        let panel = crate::panel::panel();
        assert_eq!(panel.len(), 609);
        for record in &panel {
            assert!(
                inside.contains_key(&record.irn),
                "{} has no FY2019 guarantee figure",
                record.irn
            );
        }
        let held: Dollars = panel
            .iter()
            .map(|record| inside[&record.irn])
            .sum::<Dollars>();
        assert!(
            (held - 256_496_921.97 - 469_077.69).abs() < CENT,
            "the panel holds {held:.2}"
        );
    }
}
