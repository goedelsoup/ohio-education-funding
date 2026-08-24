//! The fifty states and the District of Columbia in the Census Bureau's school finance survey.
//!
//! # The only source here that can answer a comparative question
//!
//! Everything else in this corpus is Ohio describing itself, which is enough to say what Ohio
//! does and not enough to say whether it is unusual — and the central holding of *DeRolph* is a
//! claim of the second kind. This is the state-level table, FY2022: Ohio raises **51.8%** of
//! school revenue locally against a national 43.4%, and spends within a tenth of the national
//! average doing it. The split is the story; the level is not.
//!
//! # This is not the district panel and the two do not agree exactly
//!
//! [`crate::ohio_panel`] and [`crate::national_peers`] compute the same local share from the
//! district-level F-33 and get **51.7%**. The difference is the entity set — the state table
//! includes agencies the district panel's comparability filter drops — and it is small and real.
//! `revenue-stream/esser.yml` and `revenue-stream/title-i.yml` cited `crates/dispersion/src/national_peers.rs`
//! for figures that come from *this* table, which is the kind of mis-citation a public reader
//! per source is meant to make hard. Quote whichever is right for the question and say which.
//!
//! # Two traps in this file, both of which produced a wrong answer during extraction
//!
//! **Column 0 is FIPS and the survey's own state ordering is not.** They agree through the early
//! alphabet and diverge after, both two-digit and zero-padded, so filtering the wrong one returns
//! a full and entirely wrong answer. Ohio is FIPS 39 and Census 36; Census 39 is Pennsylvania,
//! and the first run of the extractor reported Pennsylvania's figures under Ohio's name.
//!
//! **Nine states report zero school property tax and levy a great deal of it.** Their districts
//! are dependent agencies of a city or county, so the tax belongs to the parent government and
//! arrives as an appropriation — see [`StateFinance::independent`]. Ranking all fifty-one on
//! property tax share puts Massachusetts and Virginia at the bottom of a measure they are near
//! the top of. [`StateFinance::local_revenue`] survives the difference, which is why Ohio is
//! ranked on it.
//!
//! # Units
//!
//! Revenue and spending are in **thousands of dollars**, as the Bureau publishes them.
//! Enrolment is a headcount. A per-pupil figure from here therefore needs the factor of a
//! thousand — see [`StateFinance::spending_per_pupil`], which applies it.

use std::sync::OnceLock;

/// The committed state table.
pub const FIXTURE: &str = include_str!("../fixtures/census-f33-states.csv");

/// The header this reader was written against.
pub const EXPECTED_HEADER: &str = "fips,state,systems,enrollment,total_revenue,federal_revenue,\
state_revenue,local_revenue,property_tax_revenue,parent_government_revenue,current_spending";

/// One state's school finances, on the Bureau's definitions.
#[derive(Debug, Clone, PartialEq)]
pub struct StateFinance {
    /// FIPS code. Ohio is `39`; see the module note on the other two-digit code in this file.
    pub fips: String,
    /// The state's name.
    pub name: String,
    /// How many school systems the survey covers there.
    pub systems: Option<f64>,
    /// Fall membership.
    pub enrollment: f64,
    /// All revenue, from every source, in thousands of dollars.
    pub total_revenue: f64,
    /// The federal share of it.
    pub federal_revenue: f64,
    /// The state share.
    pub state_revenue: f64,
    /// And the local share, which is the measure *DeRolph* is about.
    pub local_revenue: f64,
    /// School property tax. **Zero** in the dependent-district states, which levy it through a
    /// parent government — see [`StateFinance::independent`].
    pub property_tax: f64,
    /// Revenue arriving as an appropriation from a parent city or county government.
    pub parent_government: f64,
    /// Current spending on elementary and secondary education, in thousands.
    pub current_spending: f64,
}

impl StateFinance {
    /// Local revenue as a share of total.
    #[must_use]
    pub fn local_share(&self) -> f64 {
        self.share(self.local_revenue)
    }

    /// State revenue as a share of total.
    #[must_use]
    pub fn state_share(&self) -> f64 {
        self.share(self.state_revenue)
    }

    /// Federal revenue as a share of total.
    #[must_use]
    pub fn federal_share(&self) -> f64 {
        self.share(self.federal_revenue)
    }

    /// Whether the state's districts are fiscally independent of a parent government.
    ///
    /// A tenth of local revenue is the threshold, and the distribution is bimodal enough that
    /// nothing sits near it: an independent-district state routes essentially none of its school
    /// money through a city or county, and a dependent-district state routes most of it.
    #[must_use]
    pub fn independent(&self) -> bool {
        self.parent_government < self.local_revenue * 0.10
    }

    /// Current spending per pupil, in dollars, with the Bureau's thousands applied.
    #[must_use]
    pub fn spending_per_pupil(&self) -> f64 {
        if self.enrollment > 0.0 {
            self.current_spending * 1_000.0 / self.enrollment
        } else {
            0.0
        }
    }

    fn share(&self, part: f64) -> f64 {
        if self.total_revenue > 0.0 {
            part / self.total_revenue
        } else {
            0.0
        }
    }
}

/// Every state in the table: the fifty and the District of Columbia.
///
/// # Panics
///
/// If the fixture's header is not [`EXPECTED_HEADER`], or a row's width differs from it — both
/// by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn states() -> Vec<StateFinance> {
    cached().clone()
}

/// The fixture, parsed once.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and
/// the file is compiled in, and a lookup helper that re-read it per call turned a scan
/// over districts into a quadratic one.
fn cached() -> &'static Vec<StateFinance> {
    static ROWS: OnceLock<Vec<StateFinance>> = OnceLock::new();
    ROWS.get_or_init(parse)
}

fn parse() -> Vec<StateFinance> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            let fips = row.str(0);
            if fips.is_empty() {
                return None;
            }
            Some(StateFinance {
                fips: fips.to_string(),
                name: row.str(1).to_string(),
                systems: row.num(2),
                // `required` rather than `num`: every row of this table carries every column,
                // which `every_row_carries_every_aggregate` holds, and the arithmetic below
                // has no way to express an absence.
                enrollment: row.required(3),
                total_revenue: row.required(4),
                federal_revenue: row.required(5),
                state_revenue: row.required(6),
                local_revenue: row.required(7),
                property_tax: row.required(8),
                parent_government: row.required(9),
                current_spending: row.required(10),
            })
        })
        .collect()
}

/// Ohio's row.
///
/// # Panics
///
/// If the table has no Ohio, which would mean the extract is not what it claims to be.
#[must_use]
pub fn ohio() -> StateFinance {
    cached()
        .iter()
        .find(|s| s.name == "Ohio")
        .cloned()
        .expect("the state table holds Ohio")
}

/// The national aggregate of one column over every state, as a share of national total revenue.
///
/// Summed rather than averaged: a mean of fifty-one state shares weights Wyoming with
/// California, and the national figure the corpus quotes against is the aggregate.
#[must_use]
pub fn national_share<F>(pick: F) -> f64
where
    F: Fn(&StateFinance) -> f64,
{
    let all = cached();
    let total: f64 = all.iter().map(|s| s.total_revenue).sum();
    if total <= 0.0 {
        return 0.0;
    }
    all.iter().map(pick).sum::<f64>() / total
}

/// Where a state ranks on a measure, counting from the top. Ties take the better rank.
#[must_use]
pub fn rank_of<F>(state: &StateFinance, measure: F) -> usize
where
    F: Fn(&StateFinance) -> f64,
{
    let mine = measure(state);
    cached().iter().filter(|s| measure(s) > mine).count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_fifty_states_and_the_district_of_columbia() {
        assert_eq!(states().len(), 51);
    }

    /// Every aggregate is present on every row, which is what licenses reading them as numbers
    /// rather than as options.
    #[test]
    fn every_row_carries_every_aggregate() {
        for state in states() {
            assert!(state.enrollment > 0.0, "{} has no enrolment", state.name);
            assert!(state.total_revenue > 0.0, "{} has no revenue", state.name);
            assert!(
                state.current_spending > 0.0,
                "{} has no spending",
                state.name
            );
        }
    }

    /// Ohio is FIPS 39. Census 39 is Pennsylvania, whose enrolment is about 1.72m against
    /// Ohio's 1.55m — a quantity nobody would confuse once it is asserted.
    #[test]
    fn ohio_is_fips_thirty_nine_and_not_census_thirty_nine() {
        let oh = ohio();
        assert_eq!(oh.fips, "39");
        assert!((1_500_000.0..1_600_000.0).contains(&oh.enrollment));
        assert_eq!(
            states()
                .iter()
                .find(|s| s.fips == "42")
                .map(|s| s.name.clone()),
            Some("Pennsylvania".to_string())
        );
    }

    /// The three sources partition total revenue, or the columns are not what they are labelled.
    #[test]
    fn the_three_revenue_sources_partition_total_revenue() {
        for state in states() {
            let parts = state.federal_share() + state.state_share() + state.local_share();
            assert!(
                (parts - 1.0).abs() < 0.001,
                "{}: the shares sum to {parts:.4}",
                state.name
            );
        }
    }
}
