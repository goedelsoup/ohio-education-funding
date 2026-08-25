//! One district, and the twenty-odd blocks it carries.
//!
//! [`District`] is the feed's unit of publication — the base cost build-up, the property tax
//! years, the categoricals, the transportation and supplement blocks, and the position of each
//! against the nation.

use crate::*;

/// The base cost build-up for one district, all twenty-two elements of R.C. 3317.011.
///
/// # Why the feed carries the parts and not just the total
///
/// `base_cost_per_pupil` answers "how much"; this answers "why". Base cost is assembled from
/// statutory staffing ratios applied to a district's own enrollment, priced at statewide average
/// salaries — so the number a district argues about is the sum of twenty-two decisions, and the
/// interface showed only the sum.
///
/// # And why it carries the department's figure beside its own
///
/// This is computed by `foundation`, not read. That is a claim, so the published aggregate travels
/// with it and [`BaseCostBuildUp::residual`] is the difference — a dollar or so on figures in the
/// millions, from twenty-two elements each rounded where the department rounds. Publishing the
/// residual is the difference between reproducing a number and asserting that you have.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BaseCostBuildUp {
    /// A1 — classroom teachers, at the statutory ratio for each grade band.
    pub classroom_teachers: Dollars,
    /// A2 — special teachers, at one per 150 pupils.
    pub special_teachers: Dollars,
    /// A3 — substitutes.
    pub substitutes: Dollars,
    /// A4 — professional development.
    pub professional_development: Dollars,
    /// A — teacher base cost, R.C. 3317.011(D).
    pub teachers: Dollars,
    /// B1 — guidance counselors.
    pub counselors: Dollars,
    /// B2 — librarians and media staff.
    pub librarians: Dollars,
    /// B3 — student wellness and success staff.
    pub wellness: Dollars,
    /// B4 — academic co-curricular activities.
    pub academic_cocurricular: Dollars,
    /// B5 — building safety and security.
    pub safety: Dollars,
    /// B6 — supplies and academic content.
    pub supplies: Dollars,
    /// B7 — student technology.
    pub technology: Dollars,
    /// B — student support base cost, R.C. 3317.011(E).
    pub student_support: Dollars,
    /// C1 — superintendent. The one price in the formula that varies with district size.
    pub superintendent: Dollars,
    /// C2 — treasurer.
    pub treasurer: Dollars,
    /// C3 — other district administrators, priced at 82.8% of the superintendent band.
    pub other_administrators: Dollars,
    /// C4 — fiscal support.
    pub fiscal_support: Dollars,
    /// C5 — EMIS support.
    pub emis: Dollars,
    /// C6 — district leadership support.
    pub leadership_support: Dollars,
    /// C7 — information technology centre support.
    pub itc: Dollars,
    /// C — district leadership and accountability, R.C. 3317.011(F).
    pub district_leadership: Dollars,
    /// D1 — building leadership, priced at 79.38% of the superintendent band.
    pub building_leadership_staff: Dollars,
    /// D2 — building leadership support.
    pub building_support: Dollars,
    /// D3 — building operation.
    pub building_operation: Dollars,
    /// D — building leadership and operation, R.C. 3317.011(G).
    pub building_leadership: Dollars,
    /// E — athletic co-curricular activities, R.C. 3317.011(H).
    pub athletic_cocurricular: Dollars,
    /// Funded classroom teaching positions, as the department rounds them.
    pub funded_classroom_teachers: f64,
    /// Funded special teaching positions.
    pub funded_special_teachers: f64,
    /// A + B + C + D + E, as computed here.
    pub computed_aggregate: Dollars,
    /// What the department published for the same district.
    pub published_aggregate: Dollars,
    /// `computed_aggregate - published_aggregate`. Accumulated rounding across the elements.
    pub residual: Dollars,
}

/// One tax year of a district's property tax base and the tax charged on it.
///
/// From Table SD-1, the Department of Taxation's own per-district table — a different department
/// from the one that publishes the funding model, which is what makes it worth carrying: the
/// state's two halves describe the same district and are not obliged to agree. Where they overlap
/// they do, to 0.01 mills across all 606 districts that appear in both.
///
/// Two years are carried rather than one because the mechanism this data exists to show only
/// exists as a change. H.B. 920's reduction factors roll an effective rate back as valuation
/// rises, and cannot roll it below twenty mills — so what a reappraisal does to a district's
/// revenue depends entirely on which side of that floor it sits, and a single year cannot show it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PropertyTaxYear {
    /// Tax year. Offset from the fiscal year, and collected across the following calendar year.
    pub tax_year: u16,
    /// Class I: residential and agricultural, which carry their own reduction factor.
    pub class1_value: Dollars,
    /// Class II: everything else real — commercial, industrial, mineral, railroad.
    pub class2_value: Dollars,
    /// Public utility tangible property, which is neither class and is not reduced.
    pub public_utility_value: Dollars,
    /// Class I + Class II + public utility.
    pub total_value: Dollars,
    /// Agricultural value, inside Class I.
    pub agricultural_value: Dollars,
    /// Residential value, inside Class I. Seven-tenths of the state's base.
    pub residential_value: Dollars,
    /// Commercial value, inside Class II.
    pub commercial_value: Dollars,
    /// Industrial value, inside Class II.
    pub industrial_value: Dollars,
    /// Mineral value, inside Class II.
    pub mineral_value: Dollars,
    /// Railroad value, inside Class II.
    pub railroad_value: Dollars,
    /// Effective Class I operating millage, after reduction factors.
    pub class1_rate: f64,
    /// Effective Class II operating millage.
    pub class2_rate: f64,
    /// Class I tax charged for current expenses.
    pub class1_taxes_charged: Dollars,
    /// Class II tax charged for current expenses.
    pub class2_taxes_charged: Dollars,
    /// Real property tax charged, both classes, excluding joint vocational operating levies.
    pub real_property_taxes_charged: Dollars,
    /// Public utility tax charged.
    pub public_utility_taxes_charged: Dollars,
    /// Total value over [`PropertyTaxYear::adm`].
    pub value_per_pupil: Dollars,
    /// The pupil count Table SD-1 divides by, which is **not** the funding formula's.
    ///
    /// Carried explicitly because the two departments publish the same numerator over different
    /// denominators and the difference is large. Multiply the District Profile Report's
    /// `assessed_valuation_per_pupil` by its enrolled ADM and you recover this table's
    /// `total_value` to 1.000 for all 606 districts carrying both — the taxable valuations are
    /// identical to the dollar. The pupil counts are not: Columbus is 43,019 to the Department of
    /// Education and 71,947 here, Youngstown 4,322 against 9,655.
    ///
    /// Taxation counts children residing in the district; Education's enrolled ADM counts the
    /// ones the district teaches. The gap is charter, voucher and open-enrolment-out students, so
    /// it is widest in exactly the districts where valuation per pupil does the most work in the
    /// aid formula. A page that prints one figure against the other's median is comparing two
    /// quantities that share a name and nothing else.
    pub adm: f64,
}

/// Where a district's operating money went in FY2025, per pupil, by function.
///
/// The report card's spending file, and therefore **not** the audited actuals in
/// [`District::finances`]: a different source, a different basis, and a per-pupil figure rather
/// than a total. The two answer "what did it spend it on" and "what changed hands", and this feed
/// keeps them apart because a reader who added them would be double-counting.
///
/// `classroom_instruction` and `nonclassroom` are the department's own two roll-ups and partition
/// operating spending exactly; the named functions below sit inside one or the other.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpendingByFunction {
    /// Unweighted ADM, FY2025 — the headcount denominator, not the need-weighted one.
    pub adm: f64,
    /// Total operating expenditure per pupil, FY2025.
    pub operating_per_pupil: Dollars,
    /// Classroom instruction, the department's roll-up.
    pub classroom_instruction: Dollars,
    /// Everything else, the department's other roll-up.
    pub nonclassroom: Dollars,
    /// Instruction.
    pub instruction: Dollars,
    /// Pupil support.
    pub pupil_support: Dollars,
    /// Instructional staff support.
    pub instructional_staff_support: Dollars,
    /// General administration.
    pub general_admin: Dollars,
    /// School administration.
    pub school_admin: Dollars,
    /// Operations and maintenance.
    pub operations_maintenance: Dollars,
    /// Pupil transportation.
    pub pupil_transportation: Dollars,
    /// Other support services.
    pub other_support: Dollars,
    /// Food service.
    pub food_service: Dollars,
}

/// H.B. 920 applied to one district, using the [`millage`] crate rather than restating it.
///
/// # Why this is computed and not quoted
///
/// Every other property-tax figure in the feed is a published number copied across. These are
/// the [`millage`] calculator run against two tax years of Table SD-1, which lets the page say
/// three things no published column states: how much of the voted rate the reduction factors
/// have removed, what the factors alone predict for the current year, and how far the observed
/// rate departs from that prediction.
///
/// # The residual is the interesting field
///
/// Reduction factors apply to *existing* levies on *existing* property. New construction and
/// newly voted millage are exempt from them by statute. So the gap between the predicted rate
/// and the observed one is not error — it is precisely the millage the factors do not reach,
/// and its sign says which way. Positive means new levies or new construction outran the
/// reduction; negative means levies expired faster than the factors alone would explain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MillageAnalysis {
    /// The tax year the observed and predicted rates describe.
    pub tax_year: u16,
    /// Effective Class I rate the prior tax year, the base the prediction runs from.
    pub prior_rate: f64,
    /// Effective Class I rate this tax year, as Table SD-1 publishes it.
    pub observed_rate: f64,
    /// What reduction factors alone predict: the prior rate scaled by the change in Class I
    /// value, held at the statutory floor. [`millage::effective_millage`].
    pub predicted_rate: f64,
    /// `observed_rate - predicted_rate`, in mills. What the factors cannot account for.
    pub residual: f64,
    /// Whether the floor is what stopped the reduction, per [`millage::FloorStatus`].
    pub at_floor: bool,
    /// Fraction of the voted rate H.B. 920 has removed, cumulatively, since each levy passed.
    /// `None` where the profile CSV carries no voted millage.
    pub cumulative_reduction: Option<f64>,
    /// What one mill raises per pupil against this district's real property base.
    /// [`millage::yield_of`] at one mill, over ADM. The local half of the formula in one number.
    pub yield_per_mill_per_pupil: Dollars,
}

/// What the mechanism the Fair School Funding Plan replaced would charge this district today.
///
/// [`regime_diff::at_fy2027`], which holds the plan's own computed base cost fixed and swaps only
/// the local share: instead of the local capacity measure, the charge-off's flat statutory
/// millage against assessed valuation. It is a counterfactual at current inputs, **not** a
/// reconstruction of any year the charge-off governed — those need the era's formula amount,
/// cost-of-doing-business factor and DPIA, none of which this corpus holds.
///
/// # Why this belongs beside the property tax
///
/// The charge-off *was* a millage calculation: a rate the legislature set, multiplied by a
/// district's valuation, subtracted from its cost. Its documented failure is that the rate was
/// uniform while H.B. 920 made effective rates anything but, so a district whose own rate had
/// fallen below the charge-off rate was charged for revenue it could not collect. The corpus has
/// asserted that since it was written. With Table SD-1 it is countable, and it is not a fringe
/// case: half the state is below the terminal rate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegimeCounterfactual {
    /// The statutory rate the counterfactual runs at — [`regime_diff::TERMINAL_MILLS`].
    pub charge_off_mills: f64,
    /// Deemed local share per pupil under the charge-off: the rate against valuation.
    pub charge_off_local_share: Option<Dollars>,
    /// Local capacity per pupil as the plan measures it, recovered by subtraction.
    ///
    /// `None` where the minimum state share binds and all that is knowable is that capacity
    /// exceeds a threshold. Not zero: a censored quantity is not a small one, and substituting
    /// zero would invert the comparison for the districts where it is most interesting.
    pub local_capacity: Option<Dollars>,
    /// Base cost aid per pupil under the charge-off, floored at zero — it had no minimum share.
    pub aid_charge_off: Option<Dollars>,
    /// Base cost aid per pupil as the plan computes it.
    pub aid_fsfp: Option<Dollars>,
    /// Plan minus charge-off, per pupil. Positive means the district gained by the change.
    pub difference: Option<Dollars>,
    /// What the one aligned component fails to explain. Zero is the expected answer.
    pub residual: Option<Dollars>,
    /// Whether the charge-off would have run past the whole base cost it was subtracted from.
    ///
    /// The charge-off had no minimum state share — that is the plan's invention — so these
    /// districts would receive nothing at all. Ohio's answer was a supplement rather than a
    /// floor, and neither this crate nor `regime-diff` models it.
    pub exceeds_base_cost: bool,
    /// Effective Class I mills short of the charge-off rate, where the district is short.
    ///
    /// The phantom revenue mechanism, per district. `None` where the district's own effective
    /// rate is at or above the rate it would be charged at.
    pub mills_short_of_charge_off: Option<f64>,
    /// The share of taxable value the charge-off reaches, after the reappraisal phase-in.
    ///
    /// One where the district's county has finished phasing in a revaluation, below one where it
    /// has not. This is what makes the counterfactual run on **recognized valuation** rather than
    /// on total taxable value, which is what the corpus wrongly used until it read the mechanism's
    /// actual definition. See `regime_diff::recognized_valuation`.
    pub recognized_share: f64,
    /// The tax year the district's county last reappraised or updated.
    pub reappraisal_year: u16,
    /// How much less the charge-off is on recognized valuation than on total taxable value.
    ///
    /// Zero for a district past its phase-in. The point of publishing it is that its size is
    /// decided by the county's place on the Department of Taxation's calendar and by nothing
    /// about the district itself.
    pub overstated_by: Option<Dollars>,
}

/// Special education's six categories for one district: pupils and the aid they generate.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SpecialEducation {
    /// ADM in each category, 1 through 6.
    pub adm: [f64; 6],
    /// The aid each produces, after the district's state share.
    pub aid: [Dollars; 6],
}

/// Disadvantaged Pupil Impact Aid, for one district: a blend of two counts and a squared index.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dpia {
    /// FY2025 economically disadvantaged ADM, weighted 65%.
    pub economically_disadvantaged_adm: f64,
    /// FY2026 directly certified ADM, weighted 35%. Consistently the smaller of the two.
    ///
    /// A year behind the FY2027 weights it is multiplied by. The act and the LSC greenbook both
    /// put this term on the year being funded; the department's workbook heads the column `d1b
    /// FY26 Directly Certified ADM` and sources it from the `FY26 Nov #2` collection, because a
    /// simulation of FY2027 cannot use a count FY2027 has not produced. See #174 and
    /// `project::panel::Dpia::directly_certified_adm`.
    pub directly_certified_adm: f64,
    /// The blend of the two.
    pub weighted_adm: f64,
    /// That blend as a share of enrolled ADM.
    pub percentage: f64,
    /// The share indexed against the statewide 0.5334, **squared**.
    pub index: f64,
}

/// Targeted assistance, for one district: two tiers that measure different things and add.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TargetedAssistance {
    /// Assessed valuation and federal adjusted gross income, the 60/40 halves of weighted wealth.
    pub property_valuation: Dollars,
    /// Federal adjusted gross income, the 40% half.
    pub federal_gross_income: Dollars,
    /// The blend of the two.
    pub weighted_wealth: Dollars,
    /// The median district's total wealth over this district's.
    pub capacity_index: f64,
    /// 0.8% of the shortfall below that median, phased by district size.
    pub capacity_amount: Dollars,
    /// Weighted wealth per **resident** pupil — enrolled, less open-enrolling in, plus out.
    pub wealth_per_pupil: Dollars,
    /// The median per pupil over this district's, so poorer scores higher.
    pub wealth_index: f64,
    /// A rate against wealth per pupil, paid on **enrolled** pupils. Zero below an index of 0.8.
    pub wealth_amount: Dollars,
    /// The count the wealth tier measures against, which is not the one it pays on.
    pub resident_adm: f64,
    /// Whether the district qualifies for the supplemental tier, which pays nothing.
    pub supplement_eligible: bool,
}

/// Career-technical education's five categories, plus associated services.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CareerTechnical {
    /// FTE in each category, 1 through 5.
    pub fte: [f64; 5],
    /// The aid each produces, after the district's state share.
    pub aid: [Dollars; 5],
    /// A sixth weight against the sum of all five FTE.
    pub associated_services: Dollars,
}

/// English learners' three categories, whose weights descend rather than ascend.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnglishLearners {
    /// ADM in each category, 1 through 3.
    pub adm: [f64; 3],
    /// The aid each produces, after the district's state share.
    pub aid: [Dollars; 3],
}

/// Gifted: two per-pupil amounts and three kinds of unit, with floors and a cap.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Gifted {
    /// $24 per K-6 pupil, after the state share.
    pub identification: Dollars,
    /// $2.50 per enrolled pupil, after the state share.
    pub referral: Dollars,
    /// Identified gifted FTE, which drive the specialist units.
    pub fte_k8: f64,
    /// Identified gifted FTE in grades 9-12.
    pub fte_9_12: f64,
    /// Units then dollars, for each of the three unit kinds.
    pub coordinator_units: f64,
    /// What the coordinator units pay.
    pub coordinator_aid: Dollars,
    /// Intervention specialist units for K-8, floored at 0.3.
    pub specialist_k8_units: f64,
    /// What those units pay.
    pub specialist_k8_aid: Dollars,
    /// Intervention specialist units for 9-12, floored at 0.3 and priced lower.
    pub specialist_9_12_units: f64,
    /// What those units pay.
    pub specialist_9_12_aid: Dollars,
    /// Whether every unit this district draws is a floor rather than an earned entitlement.
    pub entirely_on_the_floor: bool,
}

/// The six categorical programs, per district.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Categoricals {
    /// Equalisation for low-valuation districts. Zero for 135 of 609.
    pub targeted_assistance: Dollars,
    /// Six weighted categories of disability.
    pub special_education: Dollars,
    /// Disadvantaged Pupil Impact Aid, driven by the economically disadvantaged count.
    pub dpia: Dollars,
    /// Three weights by time in the country.
    pub english_learners: Dollars,
    /// Identification and service.
    pub gifted: Dollars,
    /// Career-technical education weights.
    pub career_technical: Dollars,
}

/// One school district's presence in one Ohio House district.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseDistrictMember {
    /// The school district, by IRN.
    pub irn: String,
    /// Its name, carried so a page need not join back to the district array.
    pub name: String,
    /// How much of the *school district* lies in this House district.
    pub share: f64,
    /// How much of this *House district's* apportioned pupils this school district provides.
    pub share_of_house_district: f64,
    /// Apportioned pupils and aid for this pair. An estimate; see [`HouseDistrict`].
    pub adm: f64,
    /// Apportioned state aid as the district receives it.
    pub realized_aid: Dollars,
    /// Whether the school district lies entirely inside this House district.
    pub wholly_inside: bool,
}

/// One of Ohio's 99 House districts, with the school funding apportioned to it.
///
/// # These figures are estimates, and nothing in Ohio's system publishes them
///
/// The department computes funding per school district and stops. No House district is a unit of
/// account anywhere in the funding system, and 339 of 609 school districts straddle two or more of
/// them — so a House district total has to be *derived*, by splitting each school district's
/// figures across the House districts it overlaps in proportion to under-18 population from the
/// 2020 census.
///
/// The one guarantee is that the split is exact in aggregate: each school district's shares sum to
/// one, so the apportionment loses no dollar. That is a property of the arithmetic and not of the
/// serialized figures — each seat's total is rounded to the cent before it is written, and summing
/// 99 of them accumulates. On the committed feed the House seats total $7,281,227,593.71 and the
/// Senate seats $7,281,227,592.75 against a statewide `realized_aid_total` of $7,281,227,591.65:
/// the two chambers disagree by $0.96, the House sitting $2.06 above the statewide figure and the
/// Senate $1.10. Do not write
/// a consumer that reconciles them to the cent. Everything else about a House district figure is
/// an estimate, and any page showing one has to say so.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseDistrict {
    /// `001` through `099`.
    pub number: String,
    /// The school districts in it, largest contributor first.
    pub members: Vec<HouseDistrictMember>,
    /// Apportioned enrolled ADM.
    pub adm: f64,
    /// Apportioned state aid as districts receive it, guarantee included.
    pub realized_aid: Dollars,
    /// Apportioned state share of base cost.
    pub base_cost_state_share: Dollars,
    /// Apportioned categorical funding — the other half of formula aid.
    pub categorical_funding: Dollars,
    /// Apportioned guarantee: what the formula does not justify, in this member's schools.
    pub guarantee: Dollars,
    /// Districts overlapping this House district that are on the guarantee.
    pub districts_on_guarantee: usize,
    /// And those at the minimum state share.
    pub districts_at_minimum_state_share: usize,
    /// Of the districts here, those that lie entirely inside this House district.
    pub districts_wholly_inside: usize,
}

/// Which House districts a school district lies in, and how much of it is in each.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseDistrictShare {
    /// `001` through `099`.
    pub number: String,
    /// How much of the school district lies in that House district.
    pub share: f64,
}

/// The payments outside foundation funding, for one district.
///
/// `[H] Foundation Funding` is base cost plus the six categoricals, and the guarantee holds a
/// district at it. These sit in `[R] Total State Support` instead, so nothing cushions a fall in
/// either: a district that drops a star, or slips below 3% growth, loses the money outright.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Supplements {
    /// The overall star rating and the progress component rating. `None` where unrated.
    pub stars: Option<f64>,
    /// The progress component rating, which the payment uses when it is the higher of the two.
    pub progress: Option<f64>,
    /// Whether any of the three routes qualified the district, and what it was paid.
    pub performance_eligible: bool,
    /// $13 a pupil times the greater of the two ratings.
    pub performance: Dollars,
    /// $40 a pupil, every district, no test.
    pub base_funding: Dollars,
    /// The three-year enrolment change the 3% growth test is applied to.
    pub enrollment_change: f64,
    /// Whether the three-year change cleared 3%.
    pub growth_eligible: bool,
    /// $250 on every pupil, for a district that cleared 3%.
    pub growth: Dollars,
    /// What clearing it would have paid a district that did not. `None` where it did.
    pub growth_forgone: Option<Dollars>,
}

/// Transportation, for one district — the largest thing outside foundation funding.
///
/// $726m, plus $183m of special education transportation. Transportation alone is larger than
/// special education, making it the second-largest single program in Ohio's school funding after
/// targeted assistance, and it shares almost nothing with the formula: two competing rate bases
/// with the district paid the greater, a 50% state minimum share against the formula's 10%, two
/// supplements that reward opposite things, its own guarantee on a FY2021 base, and a proration
/// factor on the special education line meaning the appropriation did not cover the entitlement.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transportation {
    /// Riders by the kind of school they attend. Non-public count double, community 1.5 times.
    pub public_riders: f64,
    /// Weighted double.
    pub nonpublic_riders: f64,
    /// Weighted one and a half.
    pub community_riders: f64,
    /// The three, weighted.
    pub weighted_riders: f64,
    /// What each of the two competing bases would pay before the state share.
    pub per_rider_base: Dollars,
    /// The other one.
    pub per_mile_base: Dollars,
    /// Whether the mile base is the one this district is actually paid on.
    pub paid_on_miles: bool,
    /// The state share actually applied, after the 50% floor.
    pub effective_state_share: f64,
    /// The five payments.
    pub school_bus: Dollars,
    /// Mass transit riders at 35% of the rider rate.
    pub mass_transit: Dollars,
    /// Other vehicle types at 50%.
    pub other: Dollars,
    /// Up to 15% more for filling buses.
    pub efficiency: Dollars,
    /// And a payment for not being able to.
    pub density: Dollars,
    /// Riders per bus over a capacity target.
    pub efficiency_index: f64,
    /// Riders per square mile.
    pub district_density: f64,
    /// A second transitional guarantee, on a FY2021 base.
    pub fy21_base: Dollars,
    /// What that base holds the district at.
    pub guarantee: Dollars,
    /// The total, and special education transportation beside it.
    pub total: Dollars,
    /// Prorated at 0.91746.
    pub special_education: Dollars,
    /// What it would have been without the proration.
    pub special_education_unprorated: Dollars,
}

/// Preschool special education, for one district.
///
/// A flat $4,000 a pupil whatever the category — 69% of the program, and not reduced by the state
/// share — plus the six school-age weights at half, all prorated. The proration is the point: the
/// sheet carries a limit beside the factor, and at the stated factor the program runs $908,184
/// over *that cell*. The cell is the FY2025 estimate; the FY2027 appropriation is $153,976,832,
/// which the program is $5,568,648 under. See
/// [`project::panel::supplements::PREK_SPED_APPROPRIATION`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PreschoolSpecialEducation {
    /// ADM in each category, 1 through 6.
    pub adm: [f64; 6],
    /// The aid each produces.
    pub aid: [Dollars; 6],
    /// The six, summed.
    pub total: Dollars,
    /// What the flat $4,000 component alone is worth.
    pub flat_component: Dollars,
    /// What the program would pay without the proration.
    pub unprorated: Dollars,
}

/// The guarantee's machinery, and the second hold-harmless stacked on it.
///
/// The guarantee is not "hold the district at its old amount": it is the FY2021 funding base less
/// an **open-enrolment clawback** less foundation funding. And a *second* hold-harmless sits above
/// it against a larger FY2021 base, one that includes transportation — reaching 17 districts the
/// guarantee does not.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Transition {
    /// The FY2021 amount the guarantee compares foundation funding against.
    pub funding_base: Dollars,
    /// Open enrolment FTE, last year and this.
    pub open_enrollment_prior: f64,
    /// This year.
    pub open_enrollment_current: f64,
    /// How much of a loss is absorbed before the clawback applies, and what it costs beyond it.
    pub open_enrollment_threshold: f64,
    /// What the loss beyond it costs the guarantee.
    pub open_enrollment_adjustment: Dollars,
    /// A FY2021 base that includes transportation, and the supplement it produces.
    pub fy21_funding_base: Dollars,
    /// What that larger base holds the district at.
    pub transition_supplement: Dollars,
}

/// Where a district sits among America's, on federal definitions.
///
/// Ohio describing itself cannot say whether Ohio is unusual, and every other source in this feed
/// is Ohio describing itself. This is the exception: 10,382 comparable school districts in every
/// state, reported on the Census Bureau's own definitions.
///
/// Three caveats travel with it. The year is **FY2022** against the model's FY2027. The
/// denominator is the **federal** fall membership, not Ohio's ADM. And the comparison set excludes
/// charter agencies and non-unified districts, because a community school's finances are not a
/// school district's — leaving them in put Ohio's smallest agencies at an 8% local share.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NationalPosition {
    /// Local revenue as a share of total, and where that sits among all comparable districts.
    pub local_share: f64,
    /// Where that share sits among all comparable districts.
    pub local_share_percentile: f64,
    /// Total revenue per pupil on the federal count, and its percentile.
    pub revenue_per_pupil: f64,
    /// Where that sits.
    pub revenue_per_pupil_percentile: f64,
    /// Current spending per pupil, and its percentile. Zero where unreported.
    pub spending_per_pupil: f64,
    /// And that.
    pub spending_per_pupil_percentile: f64,
}

/// One district, as the web layer needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Where this district sits among America's. `None` for the one K-8 district the comparison
    /// set excludes, which is carried without a position rather than given an invented one.
    pub national: Option<NationalPosition>,
    /// The performance supplement and the two enrolment supplements, outside the formula.
    pub supplements: Supplements,
    /// Transportation, the largest thing outside it.
    pub transportation: Transportation,
    /// Preschool special education, the last line in the same gap.
    pub preschool_special_education: PreschoolSpecialEducation,
    /// The guarantee's machinery, and the transition supplement stacked on it.
    pub transition: Transition,
    /// The Ohio House districts this district lies in, largest share first.
    ///
    /// Usually one — 270 of 609 districts sit inside a single House district — and up to eleven.
    /// Derived from census blocks; see [`HouseDistrict`] for what that does and does not support.
    pub house_districts: Vec<HouseDistrictShare>,
    /// The county the department attributes the district to.
    ///
    /// One county per district, which is the department's own simplification: school district
    /// boundaries cross county lines freely and the calculator picks one anyway. Good enough to
    /// group peers by, and not good enough to sum into a figure called the county's.
    pub county: String,
    /// Base cost enrolled ADM — the greater of the three-year average and the current year.
    pub adm: f64,
    /// Current-year enrolled ADM, FY2026. The denominator the state share is paid on.
    pub current_year_adm: f64,
    /// District base cost per pupil, FY2027.
    pub base_cost_per_pupil: Dollars,
    /// Aggregate base cost, all five sub-components.
    pub aggregate_base_cost: Dollars,
    /// How that aggregate is assembled, recomputed here rather than quoted.
    pub base_cost_build_up: BaseCostBuildUp,
    /// Property tax base and charge, TY2023 and TY2024. Empty where the district is absent.
    pub property_tax: Vec<PropertyTaxYear>,
    /// Operating spending by function, FY2025. `None` for the two districts without a row.
    pub spending_by_function: Option<SpendingByFunction>,
    /// The state's share of base cost alone, before every categorical.
    pub base_cost_state_share: Dollars,
    /// Targeted assistance, special education, DPIA, English learner, gifted, career-technical.
    pub categorical_funding: Dollars,
    /// The part of `categorical_funding` priced in the statewide average base cost per pupil.
    ///
    /// Special education, English learners and career-technical, which are each
    /// `weight x $8,241.61 x count x state share`. A base cost lever moves these too, so the
    /// scenario needs them separated. Emitted rather than re-derived in the browser so that
    /// *which* programs count is decided once, in `project::panel`, and the two implementations
    /// of `apply` cannot disagree about it.
    pub base_cost_denominated_categoricals: Dollars,
    /// Disadvantaged Pupil Impact Aid alone, because the phase-in dials it separately.
    ///
    /// R.C. 3317.022 writes two phase-in terms, one for DPIA and one for everything else. The
    /// browser needs the split to mirror `project::policy::apply`, and the full `dpia` block is
    /// not in the slim panel.
    pub dpia_funding: Dollars,
    /// `[H2] − [H3]` — the general slice of the FY2020 funding base.
    ///
    /// The origin the general phase-in interpolates *from*. Not a multiplier's worth of nothing:
    /// at a 0% phase-in this is what the district receives.
    pub general_funding_base: Dollars,
    /// `[H3]` — the DPIA slice, anchored to the district's FY2019 DPIA payment.
    pub dpia_funding_base: Dollars,
    /// `[H2] − [I1]`, floored at zero: the level the guarantee holds the district at.
    ///
    /// Emitted for **every** district rather than only the guaranteed ones. The browser used to
    /// derive it as `on_guarantee ? realized : 0`, which gave the 315 formula districts no floor
    /// and let a simulated cut push them through one Ohio would have caught them on.
    pub guarantee_floor: Dollars,
    /// Special education's six weighted categories: ADM then aid, Category 1 through 6.
    ///
    /// The weights span a factor of sixteen and the money runs against them — Category 6 is 15%
    /// of pupils and 48% of the program, Category 2 the reverse.
    pub special_education: SpecialEducation,
    /// The other five, each decomposed to the mechanism that produces it.
    ///
    /// Reading these apart is the point. The six programs answer different questions and move for
    /// opposite reasons — targeted assistance rises as a district gets poorer in *property*, DPIA
    /// as its *pupils* get poorer, gifted barely moves at all because it is mostly a staffing
    /// floor. A page that shows six dollar amounts still cannot say which of those a district's
    /// money is.
    pub dpia: Dpia,
    /// The largest, and the only equalisation among the six.
    pub targeted_assistance: TargetedAssistance,
    /// Five weights against a career-technical base cost, plus associated services.
    pub career_technical: CareerTechnical,
    /// Three weights that descend rather than ascend.
    pub english_learners: EnglishLearners,
    /// Two per-pupil amounts and three kinds of unit, with floors no other categorical has.
    pub gifted: Gifted,
    /// `[a] Enrolled ADM` — the pupil count four of the six categoricals are paid on.
    ///
    /// Not [`District::adm`], which averages three years, and not [`District::current_year_adm`],
    /// which is `[b3] FY26 Enrolled ADM`. It equals the latter in 608 of 609 districts and differs
    /// in Akron by fifty pupils. Carried so a per-pupil figure computed here uses the denominator
    /// the department paid on.
    pub categorical_adm: f64,
    /// The same, as its six parts.
    ///
    /// The total was a residual for eight phases — core foundation funding less the state share of
    /// base cost, which is exact and uninterrogable. It is 43% of formula aid, and the six behave
    /// nothing alike: targeted assistance is equalisation and is zero for 135 districts, DPIA
    /// tracks poverty. A page showing the sum cannot say which a district's money is.
    pub categoricals: Categoricals,
    /// State aid per pupil as the formula computes it, before the guarantee.
    pub formula_aid_per_pupil: Dollars,
    /// State aid per pupil as the district receives it.
    pub realized_aid_per_pupil: Dollars,
    /// Temporary transitional aid guarantee, total dollars.
    pub guarantee: Dollars,
    /// Whether the minimum state share is what sets this district's base cost aid.
    pub at_minimum_state_share: bool,
    /// Assessed valuation per pupil, FY2023.
    pub valuation_per_pupil: Option<Dollars>,
    /// Effective Class 1 operating millage, TY2023.
    pub effective_class1_millage: Option<f64>,
    /// Voted current operating millage, TY2023 — the gross rate before reduction factors.
    ///
    /// The rate the district's voters actually approved, which is not the rate anyone pays. It
    /// sat in column 6 of the profile CSV from the first import and was never parsed, which is
    /// why the site could describe H.B. 920 but never say how much of it a district had lost.
    pub voted_operating_millage: Option<f64>,
    /// H.B. 920 run against this district, rather than described. `None` without two tax years.
    pub millage: Option<MillageAnalysis>,
    /// What the mechanism the plan replaced would charge this district. `None` without valuation.
    pub regime: Option<RegimeCounterfactual>,
    /// Total operating expenditure per pupil, FY2024.
    pub operating_expenditure_per_pupil: Option<Dollars>,
    /// Share of students economically disadvantaged, FY2024, as a fraction.
    pub economically_disadvantaged: Option<f64>,
    /// Enrollment change FY2024 to FY2026, as a fraction. FY2026 is partly departmental
    /// estimate, since the calculator is published before that year closes.
    pub enrollment_change: Option<f64>,
    /// Enrolled ADM for FY2024, FY2025, FY2026 — the three years the department's `ADM Data`
    /// sheet carries.
    ///
    /// Shipped as the series rather than only as [`District::enrollment_change`] because the
    /// page projects from it. Three points is not enough to estimate this district's own
    /// variability, which is exactly why the interval comes from the cross-sectional spread
    /// instead; see [`Projection::sigma`].
    pub adm_history: [f64; 3],
    /// Achievement, growth, and need. `None` for the three districts with no report card.
    pub outcome: Option<DistrictOutcome>,
    /// Six closed fiscal years of actuals, oldest first. Empty where no filing was found.
    pub finances: Vec<FinanceYear>,
    /// The casino county student fund, oldest year first. Empty for a district the Department of
    /// Taxation's distributions do not name, which happens where an IRN in the funding calculator
    /// is not the IRN the tax department pays.
    ///
    /// **No per-pupil figure travels with this, and none should be computed from the rest of the
    /// record.** The fund is apportioned on the count R.C. 5753.11 defines — county-resident
    /// students including community, STEM and joint vocational enrolment, with dual-enrolled
    /// pupils counted twice on purpose — which is a fifth pupil denominator and a partition of
    /// nothing. Dividing by `adm` or `categorical_adm` yields a figure that reads as comparable to
    /// the ones beside it and is not.
    pub casino: Vec<CasinoYear>,
    /// How many county funds the district was paid out of in the most recent distribution.
    ///
    /// `None` where the district takes no casino money in the last year of the series. One for
    /// 294 districts and 88 for the statewide e-schools, which are not in this feed — among the
    /// 609 here it is a fact about how far a district's catchment reaches across county lines,
    /// and it is the reason the published sheets are keyed on (county, IRN).
    pub casino_counties: Option<usize>,
}

impl District {
    /// Whether the district is funded by the guarantee rather than the formula.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }

    /// Whether reduction factors have stopped operating on this district, so that valuation
    /// growth reaches its revenue.
    ///
    /// # Why this is `<=` and not `== 20.0`
    ///
    /// This compared the effective rate to a literal `20.0` within half a hundredth of a mill,
    /// which got 21 districts backwards. Six of them — Vinton County at 18.70 mills, Chesapeake
    /// Union and Highland at 19.00, Oak Hill Union and Scioto Valley at 19.60, Northwest at
    /// 19.71 — never voted twenty mills of current operating levy, so there is no reduction for
    /// the factors to make and their voted and effective rates are identical to four decimals.
    /// The other fifteen were reduced to just under twenty. All of them were reported as being
    /// *above* the floor with reduction factors operative, which is the reverse of their
    /// position. [`millage::FloorStatus`] answers it correctly: at or below the floor, the
    /// factors have stopped.
    ///
    /// The floor read from [`millage::floor_for`] rather than written here, so that the two
    /// statutory values — twenty mills, and two for a joint vocational district — stay in the
    /// crate that cites the statute. This feed carries traditional districts only.
    #[must_use]
    pub fn at_millage_floor(&self) -> bool {
        let floor = millage::floor_for(edfund_core::AgencyType::City).unwrap_or(20.0);
        self.millage
            .map(|m| m.observed_rate)
            .or(self.effective_class1_millage)
            .is_some_and(|m| m <= floor + FLOOR_TOLERANCE)
    }

    /// Above the floor, but by less than `NEAR_FLOOR_BAND` — where the binary stops meaning
    /// anything.
    ///
    /// The site calls floor status the highest-leverage single fact about a district's local
    /// revenue, and for most districts it is. For these it is a coin toss decided in the fourth
    /// decimal place, and 75 districts crossed `20.0000` in one direction or the other between
    /// TY2023 and TY2024. Counting them is the honest alternative to widening the tolerance
    /// until they fall on the side that looks tidier.
    #[must_use]
    pub fn near_millage_floor(&self) -> bool {
        let floor = millage::floor_for(edfund_core::AgencyType::City).unwrap_or(20.0);
        !self.at_millage_floor()
            && self
                .millage
                .map(|m| m.observed_rate)
                .or(self.effective_class1_millage)
                .is_some_and(|m| m <= floor + NEAR_FLOOR_BAND)
    }

    /// The FY2020 baseline the guarantee holds this district at, recoverable only when it is
    /// on the guarantee.
    #[must_use]
    pub fn implied_fy2020_baseline_per_pupil(&self) -> Option<Dollars> {
        self.on_guarantee().then_some(self.realized_aid_per_pupil)
    }
}
