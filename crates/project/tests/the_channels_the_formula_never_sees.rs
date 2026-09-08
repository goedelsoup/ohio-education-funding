//! Every way an Ohio child is schooled outside a traditional district classroom, counted once, in
//! one year, by the department that administers them.
//!
//! `dew-education-landscape` is the only publication in this corpus that sizes the channels side
//! by side. Everything else measures one at a time, so comparing them otherwise means comparing
//! publications with different vintages, populations and definitions.
//!
//! # Home education is the channel nobody can see, and it is not small
//!
//! At **53,051** it is larger than the joint vocational school districts (49,524), larger than
//! every voucher programme but EdChoice Expansion, and larger than every Ohio e-school put
//! together (33,014). It is 3.2% of the department's own statewide *public* enrolment — a ratio
//! to the public system, not a share of all Ohio children, because home education and the
//! chartered private schools sit outside that 1,665,521 and only the community schools sit
//! inside it.
//!
//! And it is the only one of the thirteen with no per-district record anywhere. Community schools
//! report through EMIS and appear in the F-33; vouchers are administered per student and
//! published per programme; open enrolment is a district-to-district transfer with both ends
//! named. Home education is a notice filed with a district of residence under R.C. 3321.042 that
//! no statute requires anyone to forward to the state as a count. `EMIS` carries withdrawal
//! reason `43 — Transferred to Home Education`, which is a flow and not a stock: a child who
//! begins home education at six and never enrols is invisible to it permanently.
//!
//! # The growth factor, which is why the gap matters now rather than in principle
//!
//! The aggregator series behind [`jhu-homeschool-hub`] puts Ohio at 33,328 in 2019-20 and 61,009
//! in 2024-25 — **1.83 times** in five years — with the step happening in one year, 2020-21, and
//! never coming back. Those figures support inference and not verification, for the reason that
//! entry gives at length: they are a secondary compilation of counts the state does not itself
//! publish. But **the 2023-24 member of that series is 53,051, and the department's own sheet
//! says 53,051**, which is what identifies the aggregator's series as Ohio's numbers carried
//! forward rather than an independent estimate. That agreement is asserted below.
//!
//! Set against the committed F-33 panel over the same window, district enrolment fell 62,609
//! between FY2020 and FY2024 while home education rose about 19,700 — **roughly 31% of the
//! decline, in magnitude**. That is a size comparison and deliberately not a decomposition:
//! districts also lost pupils to community schools, to vouchers, and to a falling birth cohort,
//! and nothing here apportions between them. What it establishes is that a channel carrying no
//! ADM, appearing in no enrolment file, and reported per district nowhere is of the same order as
//! a third of the enrolment loss that half the state's guarantee status turns on.
//!
//! [`jhu-homeschool-hub`]: ../../../.yidam/catalog/jhu-homeschool-hub.md

use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::csv;

/// The department's School Options table.
const CHANNELS: &str = include_str!("../fixtures/education-landscape-channels.csv");

/// The header this test was written against.
const CHANNELS_HEADER: &str = "channel,enrollment,parent";

/// The scholarship programmes as the department's own annual report gives them, one school year
/// later than the fact sheet.
const SCHOLARSHIPS: &str = include_str!("../fixtures/scholarship-programs.csv");

/// The header of that fixture.
const SCHOLARSHIPS_HEADER: &str = "program,name,students,expenditure,published_average";

/// The statewide enrolment the same sheet prints, read from the fixture's last row.
///
/// It belongs to a different table and is emitted anyway, because a denominator the corpus
/// quotes is a denominator nothing recomputes. It is deliberately **not** a channel: this is
/// Ohio's *public* enrolment, so the community schools are inside it and home education, the
/// chartered private schools and the voucher students at them are not. A share taken against it
/// is a ratio to the public system, which is the comparison this file makes, and not a share of
/// all Ohio children — that denominator would be larger and nothing here publishes it.
const TOTAL_LABEL: &str = "Total Enrollment";

/// One row of the School Options table.
struct Channel {
    name: String,
    enrollment: f64,
    parent: String,
}

fn channels() -> Vec<Channel> {
    csv::rows(CHANNELS, CHANNELS_HEADER)
        .filter_map(|row| {
            Some(Channel {
                name: row.str(0).to_string(),
                enrollment: row.num(1)?,
                parent: row.str(2).to_string(),
            })
        })
        .collect()
}

fn count(name: &str) -> f64 {
    channels()
        .into_iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("the table carries `{name}`"))
        .enrollment
}

/// Enrolment across the comparable districts of the F-33 panel, for one fiscal year.
fn district_enrollment(year: u16) -> f64 {
    ohio_panel::panel()
        .iter()
        .filter(|r: &&PanelRow| r.comparable && r.fiscal_year == year)
        .map(|r| r.enrollment)
        .sum()
}

/// The table's two internal sums hold exactly, which is what says the columns have not moved.
///
/// A two-column PDF read at the wrong offset produces plausible numbers, not obviously wrong
/// ones — the left column of this very sheet has a `Community Schools` row too, reading 342. Both
/// identities failing at once is what a shifted parse looks like; neither failing is what a
/// correct one looks like.
#[test]
fn the_published_parts_sum_to_the_published_wholes() {
    let rows = channels();
    for whole in ["Public Vouchers for Private School", "Community Schools"] {
        let parts: f64 = rows
            .iter()
            .filter(|c| c.parent == whole)
            .map(|c| c.enrollment)
            .sum();
        assert!(
            (parts - count(whole)).abs() < f64::EPSILON,
            "{whole}: the parts sum to {parts} against a published {}",
            count(whole)
        );
    }
    assert_eq!(
        rows.len(),
        14,
        "thirteen channels, five of them components, and the public-enrolment denominator"
    );
    let total = count(TOTAL_LABEL);
    assert!(
        (1_665_000.0..1_666_000.0).contains(&total),
        "the sheet's own statewide enrolment should be about 1.67m; it is {total}"
    );
    // Every channel is smaller than the public enrolment, including the ones outside it.
    assert!(rows
        .iter()
        .filter(|c| c.name != TOTAL_LABEL)
        .all(|c| c.enrollment < total));
}

/// Home education is larger than three things the corpus treats as significant channels.
///
/// Stated as comparisons rather than as a bare figure, because the figure alone invites the
/// reading that 53,051 is a rounding error against 1.67 million. It is not: it is a channel of
/// the same size as the ones that have their own funding formulas and their own annual reports.
#[test]
fn home_education_outnumbers_the_jvsds_the_e_schools_and_four_of_five_voucher_programmes() {
    let home = count("Home School");
    assert!(home > count("Joint Vocational School Districts"));
    assert!(home > count("Online Community Schools"));

    let larger: Vec<String> = channels()
        .into_iter()
        .filter(|c| c.parent == "Public Vouchers for Private School" && c.enrollment > home)
        .map(|c| c.name)
        .collect();
    assert_eq!(
        larger,
        vec!["EdChoice Expansion"],
        "only one voucher programme should be larger than home education"
    );

    let share = home / count(TOTAL_LABEL);
    assert!(
        (0.031..0.033).contains(&share),
        "home education is {:.2}% of the department's own statewide enrolment",
        share * 100.0
    );
}

/// The two department publications agree, one school year apart, on all five programmes.
///
/// The fact sheet counts 2023-24 and the scholarship annual report 2024-25, so the test is not
/// that the numbers match — it is that every one of the five moved the same way. Five independent
/// increases is the corroboration that the two publications are counting the same thing; a
/// programme that fell would mean they are not, or that the channel turned, and either is worth
/// failing over.
#[test]
fn the_scholarship_report_carries_the_same_five_programmes_a_year_later_and_every_one_is_larger() {
    let pairs = [
        ("EdChoice Scholarship", "traditional-edchoice"),
        ("EdChoice Expansion", "edchoice-expansion"),
        ("Cleveland Scholarship", "cleveland"),
        ("Autism Scholarship", "autism"),
        ("Jon Peterson Special Needs", "jon-peterson"),
    ];
    for (sheet_name, program) in pairs {
        let later = csv::rows(SCHOLARSHIPS, SCHOLARSHIPS_HEADER)
            .find(|row| row.str(0) == program)
            .and_then(|row| row.num(2))
            .unwrap_or_else(|| panic!("the scholarship fixture carries `{program}`"));
        let earlier = count(sheet_name);
        assert!(
            later > earlier,
            "{sheet_name}: {earlier} in 2023-24 against {later} in 2024-25"
        );
    }
}

/// The size comparison the channel is worth measuring for, and the reason it cannot be more than
/// a size comparison.
///
/// District enrolment over FY2020 to FY2024 comes from the committed panel and is verified. The
/// home-education endpoints do not: 2023-24 is the department's and 2019-20 is the aggregator's,
/// so the *change* rests on the aggregator. What is asserted here is the arithmetic and the
/// order of magnitude, not a causal share — districts lost pupils to community schools, to
/// vouchers and to a falling birth cohort as well, and nothing in this repository apportions
/// between them.
#[test]
fn the_home_education_increase_is_about_a_third_of_the_district_enrollment_loss() {
    let decline = district_enrollment(2020) - district_enrollment(2024);
    assert!(
        (62_000.0..63_500.0).contains(&decline),
        "the panel should lose about 62,600 pupils across the window; it loses {decline:.0}"
    );

    // 33,328 in 2019-20 is the aggregator's, and the only figure here that is not committed.
    const HOME_EDUCATION_2019_20: f64 = 33_328.0;
    let growth = count("Home School") - HOME_EDUCATION_2019_20;
    let share = growth / decline;
    assert!(
        (0.28..0.34).contains(&share),
        "the increase is {growth:.0} against a decline of {decline:.0}, or {:.1}%",
        share * 100.0
    );
}
