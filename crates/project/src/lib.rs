//! Forward projection of funding inputs, and policy simulation over them.
//!
//! The other half of the goal this repository was built for: not only what Ohio's school
//! funding formula did, but what it would do under a different set of decisions.
//!
//! ```no_run
//! use project::{panel::panel, policy::{GuaranteeRule, Policy}, report::simulate};
//!
//! let districts = panel();
//! let effect = simulate(&districts, &Policy {
//!     guarantee: GuaranteeRule::PhasedOut { remaining: 0.5 },
//!     ..Policy::current_law()
//! });
//! println!("{:+.0} dollars, {} districts down", effect.cost(), effect.losers());
//! ```
//!
//! # Two acts, kept apart
//!
//! The [`project` skill](../../../.yidam/skills/project.md) sets the rule this crate is organised
//! around: **simulation and projection are not the same epistemic act.** Re-running a formula
//! with a changed parameter is deterministic. Projecting enrollment five years out is a
//! forecast. A result that adds them together reports forecast error as though it were policy
//! effect, and the sum looks more precise than either part.
//!
//! So [`report::Run`] holds a [`report::PolicyEffect`] with no interval and an optional
//! [`report::EnrollmentEffect`] with one, and there is no field containing their sum.
//!
//! # What this cannot do, and why that is worth saying
//!
//! **Assessed valuation cannot be projected from the committed data.** There is exactly one
//! observation per district, FY2023. One point supports no trend, and Ohio valuation does not
//! move smoothly anyway — it steps on a county reappraisal cycle, six years for a full
//! reappraisal with an update at three, so a district's valuation is flat for years and then
//! jumps. A smooth annual rate applied to it is wrong in every individual year and only
//! approximately right across a whole cycle.
//!
//! [`series::Method::Assumed`] therefore exists: a caller may supply a rate, and it is recorded
//! as the caller's assumption rather than as something fitted. Getting past that needs the
//! `tax-abstract` connector — which is **wired**, and does not carry it. Table SD-1 publishes total
//! taxable value and not the H.B. 920-adjusted recognized valuation, so this gap survived the
//! connector that was supposed to close it.
//! Since local capacity is 60% valuation, every projection of the *local* side of Ohio school
//! funding is currently an assumption wearing a number.
//!
//! Enrollment is better but not good: three observations per district, and the last of them,
//! FY2026, is partly a departmental estimate because the calculator is published before that
//! year closes.

#![forbid(unsafe_code)]

pub mod crosswalk;
pub mod finances;
pub mod house_district;
pub mod outcomes;
pub mod panel;
pub mod policy;
pub mod report;
pub mod series;

pub use crosswalk::{coverage, Coverage};
pub use outcomes::{joined, report_cards, Joined, ReportCard};
pub use panel::{panel, DistrictRecord};
pub use policy::{GuaranteeRule, Outcome, Policy};
pub use report::{forecast, run, simulate, EnrollmentEffect, PolicyEffect, Run, Totals};
pub use series::{Basis, Method, Observation, Prior, Projection, DEFAULT_DAMPING};
