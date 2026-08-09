//! The provisions the Revised Code does not contain, read from the act that does.
//!
//! # A correction to this repository's own audit
//!
//! The open-item audit sorted Ohio's funding parameters into four kinds and put DPIA's 65/35
//! blend in the **delegated** slot: "appears in no section … a departmental choice inside a
//! statutory delegation, changeable without a bill."
//!
//! That was wrong, and wrong by the same move it was auditing. It concluded *not in law* from
//! *not in the Revised Code*. **The Revised Code is not all of the law.** H.B. 96 of the 136th
//! General Assembly sets the blend in a table on its own face, and temporary law is law.
//!
//! Three phases running, this repository has read a recorded claim instead of the source behind
//! it. The audit found that pattern and then committed it.
//!
//! # What is actually delegated, which is narrower and more interesting
//!
//! R.C. 3317.03(B)(21) delegates the **definition** of "economically disadvantaged" to the
//! department. The act then sets the **weights** on two counts whose definitions it does not
//! control. Two different levels, and the corpus had collapsed them.
//!
//! # And why "as enacted" is not pedantry
//!
//! The as-passed-by-the-House analysis sits at a sibling URL and gives the base funding supplement
//! as $20 and $30 and the enrolment growth supplement as a *tiered* schedule of $150/$100/$50.
//! None of that became law. Reading the convenient file would have contradicted the department's
//! own published payments and looked like a finding.

const ACT: &str = include_str!("../fixtures/enacted-school-funding.txt");

/// Collapse the analysis's page furniture so a quoted phrase can be matched across a line break.
fn flat() -> String {
    ACT.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// **The enrolment growth supplement is $250 at 3% for FY2027**, exactly as the corpus carries it.
///
/// And FY2026 is a different number at a different threshold — $225 at 5% — which is worth having
/// because it shows the supplement is re-specified each year rather than being a standing rate.
#[test]
fn the_enrolment_growth_supplement_is_two_hundred_and_fifty_at_three_per_cent() {
    let act = flat();
    assert!(act.contains(
        "$250 per student in FY 2027 to each district whose enrolled ADM grew by at least 3% \
         from FY 2023 to FY 2026"
    ));
    assert!(act.contains("$225 per student in FY 2026"));

    assert!((project::panel::ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL - 250.0).abs() < f64::EPSILON);
    assert!((project::panel::ENROLLMENT_GROWTH_THRESHOLD - 0.03).abs() < f64::EPSILON);
}

/// **The performance supplement's $13 and its three routes are the act's**, not the department's.
///
/// The corpus expresses the overall-rating route as `stars > 3.5`, where the act says "four
/// 'stars' or higher". On a half-star scale those are the same set — the act's own worked example
/// carries a district at "two and one-half stars" — and the two forms are pinned together here so
/// that nobody simplifies the strict `>` into a `>=` and quietly admits the 3.5-star districts.
#[test]
fn the_performance_supplement_is_thirteen_dollars_a_star_by_the_act() {
    let act = flat();
    assert!(act.contains("multiplied by $13 multiplied by the greater of the number of “stars”"));
    assert!(act.contains("An overall performance rating of four “stars” or higher"));
    assert!(act.contains("A performance rating of three “stars” or higher for the Progress"));
    // The scale runs in half-stars, which is what makes `> 3.5` and `>= 4` the same rule.
    assert!(act.contains("one-half “stars”") || act.contains("2.5"));

    assert!((project::panel::PERFORMANCE_SUPPLEMENT_PER_POINT - 13.0).abs() < f64::EPSILON);
    // `> 3.5` and `>= 4` agree on every half-star value, and disagree on nothing the scale can
    // produce. Asserted rather than asserted-in-prose.
    let scale = [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0];
    for stars in scale {
        assert_eq!(
            stars > project::panel::PERFORMANCE_STAR_THRESHOLD,
            stars >= 4.0,
            "the two forms disagree at {stars}"
        );
    }
}

/// **The DPIA blend is in the act**, and the audit said it was in no law at all.
///
/// H.B. 96 sets the economically disadvantaged ADM as a weighted pair, by fiscal year:
///
/// ```text
///                                          FY 2026   FY 2027
///   FY 2025 students                          75%       65%
///   Students categorically eligible           25%       35%
/// ```
///
/// So `DPIA_BLEND = (0.65, 0.35)` is statutory for FY2027 — uncodified statutory, the same
/// category as the supplements, and not the departmental choice the audit recorded.
#[test]
fn the_dpia_blend_is_set_by_the_act_and_not_by_the_department() {
    let act = flat();
    assert!(act.contains("Economically disadvantaged student ADM for FY 2026 and FY 2027"));
    // The table wraps its row labels around the figures, so the rows are matched as the extract
    // renders them rather than as a reader sees them on the page.
    assert!(
        act.contains("FY 2025 75% 65% students"),
        "the FY2025-students row"
    );
    assert!(
        act.contains("Students certified as categorically 25% 35% eligible"),
        "the categorically-eligible row"
    );

    assert_eq!(project::panel::DPIA_BLEND, (0.65, 0.35));
}

/// **Edgerton Local, finally explained end to end.**
///
/// The chain took three phases and three sources, and every link is now sourced:
///
///   1. the act blends **FY2025** economically disadvantaged students with currently certified
///      ones — a prior-year count against a current-year enrolment;
///   2. Edgerton's enrolment fell from **504.48 in FY2025 to 342.38 in FY2026**, a third of the
///      district, so its FY2025 count of 497.21 is 98.6% of the enrolment it was measured against
///      and 145% of the one it is applied to;
///   3. the blend therefore produced 352.03 against an enrolment of 342.38;
///   4. the cap at enrolled ADM bound, giving the published 342.38;
///   5. and its disadvantaged percentage reads 100%.
///
/// The corpus recorded step 2 as "the input is on a different basis or it is wrong". It is on a
/// different basis, the act names which, and nothing is wrong with it. **116 districts have an
/// economically disadvantaged count above their current enrolment** and the rest sit between 1.00
/// and 1.05; Edgerton is extreme only because it shrank by a third in one year.
#[test]
fn edgertons_disadvantaged_count_is_a_prior_year_count_against_a_collapsed_enrolment() {
    let panel = project::panel::panel();
    let edgerton = panel
        .iter()
        .find(|r| r.name == "Edgerton Local")
        .expect("in the panel");

    // The enrolment collapse, which is what makes it the only district to hit the cap.
    let [_fy24, fy25, fy26] = edgerton.adm_history;
    assert!((fy25 - 504.4816).abs() < 0.01);
    assert!((fy26 - 342.3818).abs() < 0.01);
    assert!(
        fy26 / fy25 < 0.7,
        "enrolment fell by a third: {:.3}",
        fy26 / fy25
    );

    // Against the year the act measures it in, the count is entirely ordinary.
    let ed = edgerton.dpia.economically_disadvantaged_adm;
    assert!(
        (0.95..1.05).contains(&(ed / fy25)),
        "ED is {:.4} of FY2025 enrolment",
        ed / fy25
    );
    // Against the year it is applied in, it is not.
    assert!(ed / fy26 > 1.4);

    // **The clean demonstration.** Take the three districts whose count most exceeds their
    // current enrolment. Against FY2025 — the year the act measures in — all three sit at 98.6%,
    // indistinguishable from each other. What separates them is entirely how much they shrank.
    //
    //   district                 ED/FY25   current/FY25   ED/current
    //   Edgerton Local            0.9856      0.6787         1.4522
    //   Garfield Heights City     0.9878      0.8634         1.1441
    //   Clearview Local           0.9857      0.9042         1.0901
    //
    // A count that is 98.6% of enrolment in three unrelated districts is not a data error in any
    // of them. The dispersion is in the denominator.
    let mut worst: Vec<&project::panel::DistrictRecord> = panel.iter().collect();
    worst.sort_by(|a, b| {
        let key = |r: &project::panel::DistrictRecord| {
            r.dpia.economically_disadvantaged_adm / r.current_year_adm
        };
        key(b).partial_cmp(&key(a)).expect("no NaN")
    });
    for record in worst.iter().take(3) {
        let against_measured = record.dpia.economically_disadvantaged_adm / record.adm_history[1];
        assert!(
            (0.98..0.99).contains(&against_measured),
            "{} is {against_measured:.4} of its FY2025 enrolment",
            record.name
        );
    }
    // And the two ratios have very different spreads, which is the same fact stated over the
    // whole panel rather than over three districts.
    let spread = |f: fn(&project::panel::DistrictRecord) -> f64| {
        let mut v: Vec<f64> = panel.iter().map(f).filter(|x| x.is_finite()).collect();
        v.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
        v[v.len() * 9 / 10] - v[v.len() / 10]
    };
    let measured = spread(|r| r.dpia.economically_disadvantaged_adm / r.adm_history[1]);
    let applied = spread(|r| r.dpia.economically_disadvantaged_adm / r.current_year_adm);
    assert!(
        applied > measured,
        "the applied ratio should be the noisier one: {applied:.4} against {measured:.4}"
    );
}

/// The extract is the school funding portion and not the whole act.
#[test]
fn the_committed_extract_is_bounded_and_is_about_schools() {
    assert!(ACT.starts_with("DEPARTMENT OF EDUCATION AND WORKFORCE"));
    assert!(ACT.lines().count() > 3_000 && ACT.lines().count() < 6_000);
    // The neighbouring departments are where it stops.
    assert!(!ACT.contains("DEPARTMENT OF MEDICAID"));
    assert!(!ACT.contains("DEPARTMENT OF HIGHER EDUCATION"));
}
