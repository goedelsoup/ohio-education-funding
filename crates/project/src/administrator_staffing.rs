//! What districts employ against what R.C. 3317.011 funds, for the one staff category the
//! corpus can see.
//!
//! # The question, and how much of it a published source can answer
//!
//! [`crate::staffing_minimums`] priced the section's staffing floors and closed on the one thing
//! it could not establish: whether they are set in the right places. Issue #408 pointed at the
//! District Profile Report, which the corpus already held and which names *personnel* as one of
//! its five families, and asked the deciding question first — which staff categories does it
//! actually count?
//!
//! **One.** The report's personnel block is a teachers' average salary, three experience shares,
//! an administrators' average salary, a pupil-administrator ratio and `FTE Number of
//! Administrators`. No teacher FTE, no counselor, no wellness, fiscal, EMIS or clerical count.
//! Six of the section's seven floors meet nothing in it. The department's definition of the one
//! count is twelve EMIS position codes — superintendent and deputy, treasurer, principal and
//! assistant principal, director, supervisor, coordinator, education administration specialist,
//! licensed administrative assistant, community school administrator, other officials — which is
//! the union of the section's four administrator elements: the superintendent at (F)(1), the
//! treasurer at (F)(2), other district administrators at (F)(3) and building leaders at (G)(1).
//! (F)(6)'s leadership support is priced on a salary band that tops out at $65,000 and is not in
//! it. So the comparison this module makes is the weaker one the issue named third — an aggregate
//! against an aggregate — and it is made for administrators alone.
//!
//! # What the comparison is not
//!
//! **Funded is not employed, and nothing here is an audit.** R.C. 3317.011 does not require a
//! district to staff what the build-up itemises, and a district above or below its funded count
//! may be there for reasons that have nothing to do with adequacy. The defensible finding is a
//! **gradient** — whether the gap between funded and employed behaves differently below the
//! administrator floor's threshold than above it — and not a level. Every level below is
//! reported because the gradient cannot be read without it.
//!
//! **The vintages are aligned here, unlike most joins in this crate.** The employed count is
//! FY2024. The funded count is computed at the report's own FY2024 enrolled ADM, using ratios
//! the section has stated since it was enacted, so both sides are the same year. The FY2027
//! model's base cost ADM is carried beside it for a robustness check and nothing else.
//!
//! # What it finds
//!
//! Ohio's 606 traditional districts employ **12,938** FTE administrators and the section funds
//! **6,580** positions for them — the build-up counts about half of what is on payroll, and 47
//! districts employ fewer than it funds. That is the level, and it is the least interesting
//! number here.
//!
//! The gradient runs the other way from the one a cliff story would predict. Ordered on
//! enrolment, the median district's employed-to-funded multiple is **1.23 in the smallest
//! sextile** and about 1.8 from the fourth sextile up; below the 1,500 ADM threshold at which the
//! two-administrator floor stops binding it is 1.52, above it 1.83. The floor does not put small
//! districts above their funded count. It brings the funded count closest to payroll exactly
//! where it binds — and that is the same finding as the wedge in [`crate::staffing_minimums`],
//! seen from the payroll side: employed administrators per thousand pupils fall monotonically
//! from 12.3 in the smallest sextile to 7.2 in the largest, so real staffing has the fixed
//! component the section models.
//!
//! How large a fixed component? A straight line through the 319 districts under the threshold
//! has an intercept of **3.85 administrators and a slope of 6.2 per thousand pupils**. The
//! section's fixed component is **four** — one superintendent, one treasurer, two other
//! administrators — and its slope is 3.6 per thousand. The number the floors set is close to
//! what the smallest districts carry; the ratio beside it is not.
//!
//! But the *shape* the floor gives the funded count has no counterpart on payroll. The section
//! funds exactly two other administrators at every enrolment under 1,500. What districts employ
//! beyond a superintendent, a treasurer and their funded building leaders rises from about three
//! at 250-500 ADM to about six at 1,250-1,500 — a slope, not a plateau. Whether the floor is "in
//! the right place" therefore has two answers: its height matches the smallest districts and its
//! reach does not match any feature of the data.

use std::collections::BTreeMap;

use dispersion::profile::ProfileDistrict;
use edfund_core::{round_dp, Adm};
use foundation::minimums::Minimum;
use foundation::ratios;

use crate::panel::DistrictRecord;

/// The fiscal year of the employed count, and of the enrolment the funded count is taken at.
pub const EMPLOYED_VINTAGE: &str = "FY2024";

/// The enrolment below which the other-administrator floor binds: 750 pupils per position, two
/// positions.
#[must_use]
pub fn threshold() -> Adm {
    Minimum::OtherAdministrators
        .threshold()
        .expect("the administrator floor has an ADM threshold")
}

/// The four administrator elements of R.C. 3317.011, as funded position counts.
///
/// The report's count sums the same four kinds of position and no others, which is what makes
/// the comparison an aggregate against the *same* aggregate rather than against a wider one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FundedAdministrators {
    /// (F)(1): one, at any enrolment.
    pub superintendent: f64,
    /// (F)(2): one, at any enrolment.
    pub treasurer: f64,
    /// (F)(3): the greater of ADM / 750 and two, rounded as the department rounds it.
    pub other: f64,
    /// (G)(1): ADM / 450, with no floor under it.
    pub building_leaders: f64,
}

impl FundedAdministrators {
    /// What the section funds at an enrolment.
    #[must_use]
    pub fn at(adm: Adm) -> Self {
        Self {
            superintendent: 1.0,
            treasurer: 1.0,
            other: round_dp(
                (adm / ratios::OTHER_ADMINISTRATOR).max(ratios::OTHER_ADMINISTRATOR_MINIMUM),
                2,
            ),
            building_leaders: round_dp(adm / ratios::BUILDING_LEADER, 2),
        }
    }

    /// Every funded administrator.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.superintendent + self.treasurer + self.other + self.building_leaders
    }

    /// The part of the count that does not move with enrolment below the threshold: one
    /// superintendent, one treasurer, two other administrators.
    pub const FIXED: f64 = 1.0 + 1.0 + ratios::OTHER_ADMINISTRATOR_MINIMUM;

    /// How many administrators one more pupil is funded for, above the threshold.
    #[must_use]
    pub fn slope_per_pupil() -> f64 {
        1.0 / ratios::OTHER_ADMINISTRATOR + 1.0 / ratios::BUILDING_LEADER
    }
}

/// One district's employed administrators beside its funded ones.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// The district, keyed the only way district figures may be keyed.
    pub irn: String,
    /// Its name as the report publishes it.
    pub name: String,
    /// Enrolled ADM, FY2024 — the report's own denominator.
    pub adm: Adm,
    /// FTE administrators, FY2024.
    pub employed: f64,
    /// What the section funds at [`District::adm`].
    pub funded: FundedAdministrators,
    /// What the section funds at the FY2027 model's base cost enrolled ADM, for the robustness
    /// check. `None` for a district the model does not carry.
    pub funded_fy27: Option<FundedAdministrators>,
}

impl District {
    /// Employed over funded.
    #[must_use]
    pub fn multiple(&self) -> f64 {
        self.employed / self.funded.total()
    }

    /// Employed administrators per thousand pupils.
    #[must_use]
    pub fn per_thousand(&self) -> f64 {
        self.employed / self.adm * 1_000.0
    }

    /// What the district employs beyond its funded superintendent, treasurer and building
    /// leaders — the count the two-administrator floor is set against.
    #[must_use]
    pub fn beyond_the_fixed_elements(&self) -> f64 {
        self.employed
            - self.funded.superintendent
            - self.funded.treasurer
            - self.funded.building_leaders
    }

    /// Whether the district is under the administrator floor's threshold.
    #[must_use]
    pub fn floored(&self) -> bool {
        self.adm < threshold()
    }
}

/// Every district the report counts administrators for, joined to the model where it can be.
///
/// The report's 606 districts against the model's 609: the three the report omits are Ohio's
/// smallest, and they are absent here rather than carried with nothing to compare.
#[must_use]
pub fn districts(profile: &[ProfileDistrict], panel: &[DistrictRecord]) -> Vec<District> {
    let by_irn: BTreeMap<&str, &DistrictRecord> =
        panel.iter().map(|d| (d.irn.as_str(), d)).collect();
    profile
        .iter()
        .filter_map(|d| {
            let adm = d.enrolled_adm?;
            let employed = d.fte_administrators?;
            if adm <= 0.0 {
                return None;
            }
            Some(District {
                irn: d.irn.clone(),
                name: d.name.clone(),
                adm,
                employed,
                funded: FundedAdministrators::at(adm),
                funded_fy27: by_irn
                    .get(d.irn.as_str())
                    .map(|record| FundedAdministrators::at(record.base_cost_adm())),
            })
        })
        .collect()
}

/// The median of one group of districts, on every measure the module reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    /// Districts in the group.
    pub n: usize,
    /// Median enrolled ADM.
    pub adm: Adm,
    /// Median employed administrators.
    pub employed: f64,
    /// Median funded administrators.
    pub funded: f64,
    /// Median employed-to-funded multiple.
    pub multiple: f64,
    /// Median employed administrators per thousand pupils.
    pub per_thousand: f64,
    /// Median employed beyond the superintendent, treasurer and building leaders.
    pub beyond_the_fixed_elements: f64,
}

/// The crates' median: the upper middle, never the mean of two.
fn upper_middle(mut values: Vec<f64>) -> f64 {
    assert!(!values.is_empty(), "no districts to take a median of");
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Summarise one group of districts.
///
/// # Panics
///
/// If the group is empty.
#[must_use]
pub fn band(rows: &[&District]) -> Band {
    let median = |f: fn(&District) -> f64| upper_middle(rows.iter().map(|d| f(d)).collect());
    Band {
        n: rows.len(),
        adm: median(|d| d.adm),
        employed: median(|d| d.employed),
        funded: median(|d| d.funded.total()),
        multiple: median(District::multiple),
        per_thousand: median(District::per_thousand),
        beyond_the_fixed_elements: median(District::beyond_the_fixed_elements),
    }
}

/// Sextiles ordered on FY2024 enrolled ADM.
///
/// Ordered on the report's own count rather than on the model's base cost ADM, so the two sides
/// of every band are the same year. [`crate::staffing_minimums`]'s sextiles are on the model's
/// count and a handful of districts sit in adjacent bands between the two orderings.
#[must_use]
pub fn sextiles(rows: &[District]) -> Vec<Band> {
    let mut ordered: Vec<&District> = rows.iter().collect();
    ordered.sort_by(|a, b| a.adm.total_cmp(&b.adm));
    let sixth = ordered.len() / 6;
    (0..6)
        .map(|s| {
            let end = if s == 5 {
                ordered.len()
            } else {
                (s + 1) * sixth
            };
            band(&ordered[s * sixth..end])
        })
        .collect()
}

/// Districts whose FY2024 enrolment lies in `[lo, hi)`.
#[must_use]
pub fn within(rows: &[District], lo: Adm, hi: Adm) -> Vec<&District> {
    rows.iter().filter(|d| d.adm >= lo && d.adm < hi).collect()
}

/// A straight line through employed administrators on enrolment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    /// Districts fitted.
    pub n: usize,
    /// Administrators at zero pupils — the fixed component the line implies.
    pub intercept: f64,
    /// Administrators per pupil.
    pub slope: f64,
}

/// Fit employed administrators on FY2024 enrolled ADM by ordinary least squares.
///
/// [`dispersion::least_squares`] centres its design, so its first coefficient is the outcome
/// mean and not the value at zero; the intercept is recovered from it here, which is the one
/// place in this module a reader might expect a library number and get a different one.
///
/// # Panics
///
/// If fewer than three districts are given, or if they all have the same enrolment.
#[must_use]
pub fn fit(rows: &[&District]) -> Fit {
    let adm: Vec<f64> = rows.iter().map(|d| d.adm).collect();
    let employed: Vec<f64> = rows.iter().map(|d| d.employed).collect();
    let regression = dispersion::least_squares(std::slice::from_ref(&adm), &employed)
        .expect("a fittable set of districts");
    let slope = regression.coefficients[1];
    let mean_adm = adm.iter().sum::<f64>() / adm.len() as f64;
    Fit {
        n: rows.len(),
        intercept: regression.coefficients[0] - slope * mean_adm,
        slope,
    }
}
