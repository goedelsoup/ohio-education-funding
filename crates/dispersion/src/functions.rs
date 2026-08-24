//! FY2025 operating spending broken down by function, per pupil.
//!
//! # What this file adds that the report card cannot
//!
//! [`crate::report_card`] carries one operating expenditure total per district. This carries the
//! same money split into the eleven functions the department reports it under, so it can answer
//! *what a district spends more on* rather than only *how much*. That is what turns a comparison
//! of two districts' totals into a comparison of what they bought: Toledo City spends $6,173 per
//! pupil more than Perrysburg Exempted Village, and the gap opens in plant and building
//! administration rather than in instruction.
//!
//! # Two roll-ups, and they partition the total
//!
//! [`Functions::classroom_instruction`] and [`Functions::nonclassroom`] are the department's own
//! two-way split and sum to [`Functions::operating`] for every district. That is asserted rather
//! than assumed below: a function file whose parts do not sum to its whole invites shares that
//! silently do not add to one.
//!
//! # Every figure is already per pupil
//!
//! The department publishes this file divided through, on the FY2025 headcount in
//! [`Functions::adm`] — not on the weighted count the report card's own per-pupil column uses.
//! A figure from here and one from `exp_per_equivalent_pupil_fy25` are the same numerator over
//! different denominators and differ by about 45% on a high-need district.

use std::sync::OnceLock;

/// The committed function file.
pub const FIXTURE: &str = include_str!("../fixtures/expenditure-functions-fy25.csv");

/// The header this reader was written against.
pub const EXPECTED_HEADER: &str = "irn,district,unweighted_adm_fy25,\
operating_expenditure_per_pupil_fy25,instruction_per_pupil,pupil_support_per_pupil,\
instructional_staff_support_per_pupil,classroom_instruction_per_pupil,\
general_admin_per_pupil,school_admin_per_pupil,operations_maintenance_per_pupil,\
pupil_transportation_per_pupil,other_support_per_pupil,food_service_per_pupil,\
nonclassroom_per_pupil";

/// One district's FY2025 operating spending by function, every figure per pupil.
#[derive(Debug, Clone, PartialEq)]
pub struct Functions {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Enrolled headcount, FY2025 — the denominator every figure below is already divided by.
    pub adm: Option<f64>,
    /// Total operating expenditure per pupil.
    pub operating: Option<f64>,
    /// Instruction.
    pub instruction: Option<f64>,
    /// Pupil support — counselling, health, attendance.
    pub pupil_support: Option<f64>,
    /// Instructional staff support.
    pub instructional_staff_support: Option<f64>,
    /// The department's classroom roll-up. Adds to [`Functions::nonclassroom`] to give
    /// [`Functions::operating`].
    pub classroom_instruction: Option<f64>,
    /// District-level administration.
    pub general_admin: Option<f64>,
    /// Building-level administration.
    pub school_admin: Option<f64>,
    /// Operations and maintenance of plant.
    pub operations_maintenance: Option<f64>,
    /// Pupil transportation.
    pub pupil_transportation: Option<f64>,
    /// Other support services.
    pub other_support: Option<f64>,
    /// Food service.
    pub food_service: Option<f64>,
    /// The department's non-classroom roll-up.
    pub nonclassroom: Option<f64>,
}

impl Functions {
    /// One function as a share of total operating spending.
    ///
    /// `None` where either figure is absent, so a district that reports no total does not read
    /// as spending nothing on the function.
    #[must_use]
    pub fn share(&self, function: Option<f64>) -> Option<f64> {
        match (function, self.operating) {
            (Some(part), Some(total)) if total > 0.0 => Some(part / total),
            _ => None,
        }
    }
}

/// Every district in the function file.
///
/// # Panics
///
/// If the fixture's header is not [`EXPECTED_HEADER`], or a row's width differs from it — both
/// by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn districts() -> Vec<Functions> {
    cached().clone()
}

/// The fixture, parsed once.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and
/// the file is compiled in, and a lookup helper that re-read it per call turned a scan
/// over districts into a quadratic one.
fn cached() -> &'static Vec<Functions> {
    static ROWS: OnceLock<Vec<Functions>> = OnceLock::new();
    ROWS.get_or_init(parse)
}

fn parse() -> Vec<Functions> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            let irn = row.str(0);
            if irn.is_empty() {
                return None;
            }
            Some(Functions {
                irn: irn.to_string(),
                name: row.str(1).to_string(),
                adm: row.num(2),
                operating: row.num(3),
                instruction: row.num(4),
                pupil_support: row.num(5),
                instructional_staff_support: row.num(6),
                classroom_instruction: row.num(7),
                general_admin: row.num(8),
                school_admin: row.num(9),
                operations_maintenance: row.num(10),
                pupil_transportation: row.num(11),
                other_support: row.num(12),
                food_service: row.num(13),
                nonclassroom: row.num(14),
            })
        })
        .collect()
}

/// The district with this IRN, if the file covers it.
#[must_use]
pub fn district(irn: &str) -> Option<Functions> {
    cached().iter().find(|d| d.irn == irn).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_covers_every_rated_district() {
        assert_eq!(districts().len(), 607);
    }

    /// The two roll-ups partition operating spending exactly.
    #[test]
    fn classroom_and_nonclassroom_partition_operating_spending() {
        for d in districts() {
            let (Some(classroom), Some(other), Some(operating)) =
                (d.classroom_instruction, d.nonclassroom, d.operating)
            else {
                continue;
            };
            let sum = classroom + other;
            assert!(
                (sum - operating).abs() < operating * 0.001 + 1.0,
                "{}: {sum:.2} against {operating:.2}",
                d.name
            );
        }
    }

    /// A share is `None` rather than zero where the total is missing. Reading it as zero would
    /// place a district that reports nothing at the bottom of every function ranking.
    #[test]
    fn a_share_without_a_total_is_absent_rather_than_zero() {
        let d = Functions {
            irn: "000000".to_string(),
            name: "nowhere".to_string(),
            adm: None,
            operating: None,
            instruction: Some(1_000.0),
            pupil_support: None,
            instructional_staff_support: None,
            classroom_instruction: None,
            general_admin: None,
            school_admin: None,
            operations_maintenance: None,
            pupil_transportation: None,
            other_support: None,
            food_service: None,
            nonclassroom: None,
        };
        assert_eq!(d.share(d.instruction), None);
    }
}
