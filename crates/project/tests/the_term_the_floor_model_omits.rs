//! The twenty-mill floor, read against the section that states it and the millage it produces.
//!
//! `parameter/twenty-mill-floor` already establishes most of what is here and this file pins it:
//! that R.C. 319.301 never says "twenty mills" but **two per cent of the taxable value**, that
//! fourteen districts voting 23 to 48 mills report effective Class I rates just under twenty, and
//! that a combined Class 1 and Class 2 base does not explain them. Two things are new.
//!
//! # The floor did not move and its base did
//!
//! The committed text is effective **20 March 2026**, amended by H.B. 186 and H.B. 129 of the
//! 136th General Assembly — so `parameter/twenty-mill-floor`'s request that the floor "be
//! confirmed against current R.C. 319.301" is answerable, and the answer is that two per cent
//! still stands. But division (E) now **excludes** from "taxes charged and payable" any tax
//! levied under R.C. 5705.194 or 5705.199 — emergency and substitute levies — approved before
//! 1 January 2026, until the first tax year from 2026 in which R.C. 5715.24 applies in a county
//! holding part of the district. The rate is unchanged and what counts toward it is not.
//!
//! # A named candidate for the district rounding cannot cover
//!
//! The node's account of the fourteen is rounding inside the reduction-factor computation, and it
//! says so with the exception named: *"Bradford Exempted Village, at 19.7247, is the one that a
//! rounding account would not cover."*
//!
//! (E)(2) does not require a district's own current-expense taxes to reach two per cent. It
//! requires them to reach it **"when combined with the pre-1982 joint vocational taxes against
//! that class"** — which (E)(1) defines as the amount by which a joint vocational district's
//! TY1981 current-expense taxes exceeded two-tenths of one per cent of value, floored at zero.
//! That is an additive term, statutorily named, absent from the corpus's model, and of the right
//! size. It is not the combined-*base* hypothesis the node disconfirmed; that one changed the
//! denominator, and this one adds to the numerator.
//!
//! What makes it worth naming rather than guessing is the **bound**: no district that voted
//! twenty sits more than three tenths of a mill below the floor, and every district more than
//! half a mill below is one whose voters never approved twenty. A floor being violated has no
//! bound. An additive term left out has a bound the size of the term.

use dispersion::profile::{self, ProfileDistrict};

/// The floor, as the section states it: a share of taxable value, not a rate.
const FLOOR_AS_A_SHARE: f64 = 0.02;
/// The same, converted. A mill is a thousandth, so two per cent is twenty of them.
const FLOOR_IN_MILLS: f64 = FLOOR_AS_A_SHARE * 1000.0;

/// Districts below the floor that voted enough to reach it — the fourteen.
fn unexplained() -> Vec<ProfileDistrict> {
    profile::districts()
        .into_iter()
        .filter(|d| d.below_twenty_mill_floor() && !d.never_voted_twenty_mills())
        .collect()
}

/// How far below the floor a district sits, in mills.
fn shortfall(d: &ProfileDistrict) -> f64 {
    FLOOR_IN_MILLS - d.effective_class1_millage.expect("an effective rate")
}

/// The section still says two per cent, and it was amended this year.
///
/// `parameter/twenty-mill-floor` states this and nothing asserted it. The point of pinning it is
/// the second assertion: the section contains the word "mill" **nowhere**, so a reader searching
/// the statute for the name this corpus uses finds nothing, and a future amendment that started
/// writing rates would fail here rather than quietly disagree with the node.
#[test]
fn the_section_states_a_share_of_value_and_not_a_number_of_mills() {
    let section = project::statute::section("319.301");
    assert!(
        section.body.contains("two per cent of the taxable value"),
        "the floor is stated as a share of value and the section no longer says so"
    );
    assert!(
        section
            .body
            .contains("two-tenths of one per cent of the taxable value"),
        "the joint vocational floor is stated the same way"
    );
    assert!(
        !section.body.contains("mill"),
        "the section states no rate anywhere, which is what division (F) is about"
    );
    assert!(
        section.body.contains(
            "No reduction shall be made under this section in the rate at which any tax is levied"
        ),
        "division (F) is what makes the published effective rate a quotient rather than a rate"
    );

    // Twenty mills is arithmetic on the section, not a figure read off it.
    assert!((FLOOR_IN_MILLS - 20.0).abs() < f64::EPSILON);
    assert_eq!(section.effective, "March 20, 2026");
}

/// The floor's *base* moved in 2026 even though the floor did not.
///
/// (E) now excludes from "taxes charged and payable" any tax levied under R.C. 5705.194 or
/// 5705.199 — emergency and substitute levies — approved before 1 January 2026, until the first
/// tax year from 2026 in which R.C. 5715.24 applies in a county holding part of the district.
///
/// So a district whose twenty mills included a pre-2026 emergency or substitute levy has that
/// levy taken out of the computation until its county's next reappraisal or update year. The rate
/// is unchanged and what counts toward it is not, which is the kind of amendment a node recording
/// "unmoved since 1976" is most likely to miss.
#[test]
fn what_counts_toward_the_floor_changed_in_2026() {
    let body = project::statute::section("319.301").body;
    assert!(
        body.contains("excludes any tax charged and payable from a tax levied under section 5705.194 or 5705.199"),
        "the 2026 exclusion is what makes the floor's base different from its rate"
    );
    assert!(
        body.contains("approved by electors at an election held before January 1, 2026"),
        "the exclusion is dated, and the date is what bounds it"
    );
    // And the same two sections sit outside H.B. 920's reductions altogether, under (A)(1).
    assert!(
        body.contains("including a tax levied under section 5705.199 or 5748.09"),
        "fixed-sum levies were already outside the reduction; the exclusion above is a second \
         thing and not a restatement"
    );
}

/// The fourteen are all *just* below, and that bound is what admits an omitted term.
///
/// Voted rates run from 23 to 48 mills and every shortfall is under three tenths of a mill, ten
/// of the fourteen under five hundredths. Nothing here is a floor failing to hold — a floor that
/// failed would leave a district wherever its reduction factors put it.
///
/// The counts differ slightly from `parameter/twenty-mill-floor`'s, which reads Table SD-1 at
/// TY2024 and puts eleven within five hundredths and Bradford at 19.7247. This reads the FY2024
/// District Profile Report at TY2023 and gets ten and 19.7221. Two tables, two tax years, the
/// same shape — and the node already insists on naming the year on every count for exactly this
/// reason.
#[test]
fn no_district_that_voted_twenty_sits_more_than_a_third_of_a_mill_below_the_floor() {
    let below = unexplained();
    assert_eq!(
        below.len(),
        14,
        "districts below the floor that voted enough to reach it"
    );

    let mut shortfalls: Vec<f64> = below.iter().map(shortfall).collect();
    shortfalls.sort_by(f64::total_cmp);
    assert!(
        shortfalls[0] > 0.005,
        "the smallest shortfall is {:.4}, which is inside publication rounding and would make \
         this a different finding",
        shortfalls[0]
    );
    assert!(
        *shortfalls.last().expect("fourteen") < 0.30,
        "the largest shortfall is {:.4} mills; the bound is what says an omitted term rather \
         than a violated floor",
        shortfalls.last().expect("fourteen")
    );
    assert_eq!(
        shortfalls.iter().filter(|s| **s < 0.05).count(),
        10,
        "shortfalls under five hundredths of a mill"
    );

    // Every one of them voted comfortably past the floor, so none is the six that could not reach it.
    for district in &below {
        let voted = district.current_operating_millage.expect("a voted rate");
        assert!(
            voted > 23.0,
            "{} voted {voted:.4}, which is nearer the floor than this finding assumes",
            district.name
        );
    }
}

/// And every district that sits *well* below the floor is one the floor cannot lift.
///
/// Three districts in the profile report carry an effective Class I rate under 19.5 mills, and
/// all three are among those whose voters never approved twenty — Vinton County at 18.70,
/// Chesapeake Union at 19.00, Highland at 19.00. A floor cannot raise a rate above the voted one,
/// so these are the floor working rather than failing.
///
/// The two populations do not overlap, which is what lets the fourteen be attributed to a term
/// rather than to noise: below half a mill, only districts that never voted twenty appear.
#[test]
fn the_districts_far_below_the_floor_are_the_ones_it_cannot_lift() {
    let far: Vec<ProfileDistrict> = profile::districts()
        .into_iter()
        .filter(|d| d.effective_class1_millage.is_some_and(|m| m < 19.5))
        .collect();
    assert_eq!(far.len(), 3);
    for district in &far {
        assert!(
            district.never_voted_twenty_mills(),
            "{} is more than half a mill below the floor and did vote twenty, which would put a \
             district outside both explanations this file offers",
            district.name
        );
    }

    // The floor holds 170 districts exactly on it, which is the population the fourteen are the
    // edge of rather than an exception to.
    let at = profile::districts()
        .iter()
        .filter(|d| d.at_twenty_mill_floor())
        .count();
    assert_eq!(at, 170);
}

/// The omitted term is the right size to be the pre-1982 joint vocational one.
///
/// (E)(1) makes that term the excess of a joint vocational district's TY1981 current-expense
/// taxes over two-tenths of one per cent of value. Read backwards from the shortfalls, the
/// fourteen imply 1981 joint vocational rates between **2.006 and 2.278 mills** — a hair above
/// the two-mill figure the same section names, which is where a joint vocational district's
/// current-expense rate sits.
///
/// This is arithmetic on a hypothesis and not a measurement of one. Confirming it needs TY1981
/// joint vocational current-expense rates by district, which no source this corpus holds carries;
/// the membership lists alone are unpopulated, as `education-agency/eastland-fairfield-ctc`
/// records. What the arithmetic establishes is that the term is not the wrong size, which is the
/// cheapest way a named cause can fail and the one this passes.
///
/// It also reaches the district the rounding account does not. Bradford Exempted Village is a
/// quarter of a mill below the floor — two orders of magnitude past the 0.0004 the same report's
/// publication rounding produces elsewhere — and implies a 1981 rate of 2.278, which is an
/// ordinary joint vocational current-expense rate and not an anomaly at all.
#[test]
fn the_shortfalls_imply_a_nineteen_eighty_one_rate_just_above_two_mills() {
    /// (E)(1)(b), in mills: the floor a joint vocational district's own taxes are measured from.
    const JOINT_VOCATIONAL_FLOOR_IN_MILLS: f64 = 0.002 * 1000.0;

    let implied: Vec<f64> = unexplained()
        .iter()
        .map(|d| JOINT_VOCATIONAL_FLOOR_IN_MILLS + shortfall(d))
        .collect();
    let low = implied.iter().cloned().fold(f64::MAX, f64::min);
    let high = implied.iter().cloned().fold(f64::MIN, f64::max);

    assert!(
        (2.00..2.01).contains(&low),
        "the smallest implied 1981 rate is {low:.4}, which is no longer just above two mills"
    );
    assert!(
        (2.2..2.3).contains(&high),
        "the largest implied 1981 rate is {high:.4}; past about two and a half the term stops \
         being a plausible joint vocational current-expense rate and this reading needs redoing"
    );
}
