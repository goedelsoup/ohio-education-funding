//! Every weight in this corpus, checked against the law that sets it.
//!
//! # Why this is a different check from the one already passing
//!
//! The weights were transcribed from the department's FY2027 calculator and confirmed by
//! reproducing the department's own published per-district amounts. That is a strong check and it
//! verifies the wrong thing: it establishes that the department is **self-consistent**. A
//! spreadsheet can be internally perfect and still not be the statute.
//!
//! `ohio-laws` was `Declared` since genesis on "codes.ohio.gov serves HTML with no bulk export",
//! which is true, and which is a fact about the absence of a convenience. The text is
//! server-rendered. Reading it makes the independent check possible for the first time.
//!
//! **All fourteen multiples match to the last digit** — six special education, three English
//! learner, five career-technical.
//!
//! # And the text answers questions the numbers could not
//!
//! Four items the corpus recorded as `[open]` were open because the calculator publishes
//! quantities and not reasons. The statute publishes reasons: it names the clinical categories
//! behind the special education weights, states what the English learner taper tracks, lists the
//! career-technical programme areas, and contains the squaring the DPIA node had recorded as "not
//! located in statute here".
//!
//! One citation was simply wrong. The DPIA node cited R.C. 3317.029; there is no such section.

const STATUTE: &str = include_str!("../fixtures/revised-code.txt");

/// One record of the committed extract.
struct Section<'a> {
    title: &'a str,
    effective: &'a str,
    legislation: &'a str,
    body: &'a str,
}

fn section(number: &str) -> Section<'_> {
    let marker = format!("\n=== {number}\n");
    let start = STATUTE
        .find(&marker)
        .unwrap_or_else(|| panic!("{number} is not in the committed extract"))
        + marker.len();
    let rest = &STATUTE[start..];
    let end = rest.find("\n=== ").unwrap_or(rest.len());
    let record = &rest[..end];
    let field = |name: &str| {
        record
            .lines()
            .find_map(|l| l.strip_prefix(name))
            .unwrap_or_default()
            .trim()
    };
    let body = record
        .find("\n--\n")
        .map_or("", |i| &record[i + 4..])
        .trim();
    // The extract's field labels are generic — it holds opinions as well as statutes — and are
    // read back into the names this test's subject actually uses.
    Section {
        title: field("title:"),
        effective: field("date:"),
        legislation: field("source:"),
        body,
    }
}

/// Every `multiple of X` the section states, in order.
fn multiples(body: &str) -> Vec<f64> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(i) = rest.find("multiple of ") {
        rest = &rest[i + "multiple of ".len()..];
        let digits: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(v) = digits.trim_end_matches('.').parse::<f64>() {
            out.push(v);
        }
    }
    out
}

/// Every section the corpus cites, and the only ones the extract should hold.
///
/// One list rather than a list and a count. It was both, and adding the three casino sections
/// failed on the count with no indication of which section was unexpected — the count is
/// derivable from the list and a second statement of it is a second thing to forget.
const CITED: &[&str] = &[
    "3317.02",
    "3317.011",
    "3317.013",
    "3317.014",
    "3317.016",
    "3317.017",
    "3317.019",
    "3317.022",
    "3317.051",
    "3317.0212",
    "3317.0213",
    "3317.0217",
    "3317.03",
    "319.301",
    "5705.391",
    // The casino tax and the school share of it. Not part of the formula — which is the finding
    // the `casino-tax-distribution` node rests on — but cited by it, and the extract holds what
    // the corpus cites.
    "3302.01",
    "3302.03",
    "3302.10",
    "3302.12",
    "3770.06",
    "5753.02",
    "5753.03",
    "5753.11",
    // The scholarship chapter, and the pilot project sections that sit outside it. Cited by the
    // five `program` nodes for what each programme is, who qualifies, and what bounds the award —
    // and, jointly with 3317.022 above, for the mechanism: none of these uses the word "deduct".
    "3310.01",
    "3310.03",
    "3310.032",
    "3310.08",
    "3310.41",
    "3310.51",
    "3310.52",
    "3313.975",
    // How a school district stops existing. `dispersion::lea_directory` counts three districts
    // out of the federal agency directory and the directory files all three as closed "with no
    // effect on another agency's boundaries"; 3311.22 is the section that says otherwise, and it
    // is also the section that settles who issues the order — the educational service center
    // governing board, with the State Board of Education reachable only on appeal.
    "3311.06",
    "3311.22",
];

/// The extract is what it claims to be: every cited section, each with a date and an act.
#[test]
fn every_cited_section_is_present_and_dated() {
    let count = STATUTE.lines().filter(|l| l.starts_with("=== ")).count();
    assert_eq!(
        count,
        CITED.len(),
        "the extract holds {count} sections and {} are cited; a section was added to \
         OHIO_LAWS_SECTIONS without being added here, or the reverse",
        CITED.len()
    );
    for number in CITED {
        let s = section(number);
        assert!(!s.title.is_empty(), "{number} has no title");
        assert!(!s.effective.is_empty(), "{number} has no effective date");
        assert!(
            s.body.len() > 400,
            "{number} body is {} chars",
            s.body.len()
        );
    }
    // The current text is the current budget's, which is what makes the FY2027 model checkable
    // against it at all.
    assert!(section("3317.013").legislation.contains("House Bill"));
    assert!(section("3317.014").effective.contains("2025"));
}

/// **The check the corpus could not previously make.** Fourteen weights, against the law.
#[test]
fn all_fourteen_multiples_match_the_revised_code_exactly() {
    assert_eq!(
        multiples(section("3317.013").body),
        project::panel::SPECIAL_EDUCATION_WEIGHTS.to_vec(),
        "R.C. 3317.013 special education"
    );
    assert_eq!(
        multiples(section("3317.016").body),
        project::panel::ENGLISH_LEARNER_WEIGHTS.to_vec(),
        "R.C. 3317.016 English learners"
    );
    assert_eq!(
        multiples(section("3317.014").body),
        vec![0.6230, 0.5905, 0.2154, 0.1830, 0.1570],
        "R.C. 3317.014 career-technical"
    );
}

/// **What Category 1 through Category 6 actually mean**, which the node could not say.
///
/// The special education node recorded that "the clinical definitions that place a child in
/// Category 1 rather than Category 6 are in rule rather than in the calculator, and the corpus has
/// not read them. Until it does, nothing here can say whether the sixteen-fold range tracks a
/// sixteen-fold difference in cost."
///
/// R.C. 3317.013 names them, in the same order as the weights. The range is not arbitrary and it
/// is not obviously proportional to cost either — it runs from a speech and language disability to
/// the multiple-disability and low-incidence categories, which is a severity ordering. Whether it
/// tracks a sixteen-fold difference in *cost* is still not answerable from here; what is now
/// answerable is what is being ordered.
#[test]
fn the_special_education_categories_are_named_in_statute() {
    let body = section("3317.013").body;
    assert!(
        body.contains("speech and language disability"),
        "Category 1"
    );
    assert!(body.contains("specific learning disabled"), "Category 2");
    assert!(
        body.contains("Chapter 3323."),
        "the categories are defined by cross-reference"
    );
    // The ordering is by category letter and the weights ascend with it, so the statute's own
    // sequence is the severity sequence the corpus inferred.
    let weights = multiples(body);
    assert!(
        weights.windows(2).all(|w| w[1] > w[0]),
        "the statute lists them ascending: {weights:?}"
    );
}

/// **What the English learner taper tracks**, which the node left open.
///
/// The node asked whether the taper "is a judgement that acquisition costs most at the start or an
/// incentive to reclassify" and said it was not established. R.C. 3317.016 does not settle the
/// motive, but it settles the mechanism, and the mechanism narrows the question: the first
/// category is defined by **time enrolled in United States schools** — 180 school days or less —
/// and the second runs until the student **achieves a proficient score** on the spring assessment.
///
/// So the taper is keyed to an exit event a district does not control and a clock it does not set.
/// That is evidence against the reclassification-incentive reading, and it is not proof.
#[test]
fn the_english_learner_taper_is_keyed_to_time_and_to_proficiency() {
    let body = section("3317.016").body;
    assert!(body.contains("180 school days or less"));
    assert!(body.contains("proficient score"));
    assert!(
        body.contains("English learner"),
        "the section defines its own population"
    );
    // Descending, unlike special education: the money falls as the pupil progresses.
    let weights = multiples(body);
    assert!(weights.windows(2).all(|w| w[1] < w[0]), "{weights:?}");
}

/// **The career-technical categories, which the node said had not been read out of rule.**
///
/// It recorded: "Unlike special education, where the ordering plainly tracks severity, the CTE
/// ordering's meaning has not been read out of rule." R.C. 3317.014 lists the programme areas
/// behind each multiple, and the ordering is legible once they are named — the two highest
/// multiples are workforce-development programmes grouped by field, and the lower ones are
/// intervention and support programmes rather than full workforce programmes.
#[test]
fn the_career_technical_categories_are_programme_areas_not_severities() {
    let body = section("3317.014").body;
    for area in [
        "agricultural and environmental systems",
        "construction technologies",
        "engineering and science technologies",
        "health science",
        "information technology",
        "hospitality and tourism",
        "law and public safety",
        "career-based intervention",
    ] {
        assert!(body.contains(area), "R.C. 3317.014 should name {area}");
    }
}

/// **The squaring is in statute after all**, and the DPIA node's citation was wrong.
///
/// The node recorded that "the squaring is visible in the workbook's own formula and has not been
/// located in statute here", and cited the programme to R.C. 3317.029. There is no R.C. 3317.029.
/// Disadvantaged pupil impact aid is R.C. 3317.022(A)(4), and the index it multiplies is defined
/// at R.C. 3317.02(I)(1)(a) as **the square of the quotient** of a district's economically
/// disadvantaged percentage over the statewide one.
#[test]
fn the_disadvantaged_index_is_squared_by_statute_and_lives_where_the_node_did_not_say() {
    let definitions = section("3317.02").body;
    assert!(definitions.contains("\"Economically disadvantaged index for a school district\""));
    assert!(
        definitions.contains("the square of the quotient"),
        "R.C. 3317.02(I)(1)(a) states the squaring"
    );

    // The programme itself, and its per-pupil amount, are in the core funding section.
    let core = section("3317.022").body;
    assert!(core.contains("$422"), "the per-pupil amount is statutory");
    assert!(core.contains("economically disadvantaged index"));
    assert!(
        (project::panel::DPIA_PER_PUPIL - 422.0).abs() < f64::EPSILON,
        "the corpus carries the statutory amount"
    );

    // And the statewide denominator is defined as a computation over three populations rather
    // than as a constant, which is why it moves and the node was right to ask.
    assert!(definitions.contains("statewide ADM"));
    assert!(definitions.contains("community schools"));
}

/// **The 65/35 blend is not in the law at all — it is delegated.**
///
/// The DPIA node carries a blend of economically disadvantaged ADM and directly certified ADM at
/// 65/35, transcribed from the calculator. Neither R.C. 3317.022 nor R.C. 3317.03 contains the
/// phrase "directly certified", and neither states a blend. What R.C. 3317.03(B)(21) says is that
/// the count is of students "economically disadvantaged, **as defined by the department**".
///
/// So the weights are a departmental choice operating inside a statutory delegation, and the one
/// constraint the General Assembly imposed on it is a floor on breadth rather than a formula: "A
/// student shall not be categorically excluded from the number reported ... based on anything
/// other than family income."
///
/// That is a materially different status from the multiples above, and worth the corpus recording
/// as such: those are legislated to four decimal places, this is not legislated at all.
#[test]
fn the_dpia_blend_is_departmental_rather_than_statutory() {
    let counts = section("3317.03").body;
    let core = section("3317.022").body;
    assert!(
        !counts.to_lowercase().contains("directly certified"),
        "if statute names direct certification this note is wrong"
    );
    assert!(!core.to_lowercase().contains("directly certified"));
    assert!(counts.contains("as defined by the department"));
    assert!(counts.contains("shall not be categorically excluded"));
    assert!(counts.contains("based on anything other than family income"));
    // The corpus's blend, which has no statutory source.
    assert_eq!(project::panel::DPIA_BLEND, (0.65, 0.35));
}

/// **The twenty-mill floor is not stated in mills, and its last open candidate is confirmed.**
///
/// Two findings, and the second reverses something this repository asserted earlier the same day.
///
/// R.C. 319.301 never says "twenty mills". The floor is **two per cent of the taxable value** of
/// the class, with **two-tenths of one per cent** for joint vocational districts. Those are 20 and
/// 2 mills and the corpus's figures are right; the name is a colloquialism for a percentage, and a
/// reader looking for "twenty mills" in the statute will not find it.
///
/// # The candidate the data could not settle
///
/// The parameter node's last untested candidate was that emergency and other fixed-sum levies sit
/// outside the reduction. `questions_the_corpus_left_open.rs` tested it against Table SD-1, found
/// 155 districts at exactly 20.0000, and concluded the candidate was "disconfirmed as a general
/// rule". **That conclusion was wrong, and the statute is explicit.** R.C. 319.301(A)(1): the
/// reductions "do not apply to ... taxes levied at whatever rate is required to produce a
/// specified amount of tax money, including a tax levied under section 5705.199" — which is the
/// emergency levy section.
///
/// What the data actually showed was narrower than what was concluded from it: the exclusion does
/// not explain the 155 districts sitting exactly on the floor, because those districts have no
/// fixed-sum levies in their Class I current-expense rate. It remains the natural explanation for
/// the 68 just above. An absence of the effect in one group is not an absence of the mechanism,
/// and reading it as one is the mistake being corrected here.
#[test]
fn the_floor_is_a_percentage_and_fixed_sum_levies_are_exempt_from_reduction() {
    let s = section("319.301");
    assert!(s.title.to_lowercase().contains("reduction"));

    // Stated as a percentage of taxable value, never as a millage.
    assert!(s.body.contains("two per cent of the taxable value"));
    assert!(s.body.contains("two-tenths of one per cent"));
    assert!(
        !s.body.contains("twenty mills") && !s.body.contains("twenty-mill"),
        "if the statute starts saying it in mills, this note needs rewriting"
    );

    // The exemption, which is the parameter node's last candidate stated in law.
    assert!(s
        .body
        .contains("The reductions required by division (D) of this section do not apply"));
    assert!(s
        .body
        .contains("required to produce a specified amount of tax money"));
    assert!(
        s.body.contains("5705.199"),
        "the emergency levy section is named in the exemption"
    );

    // And it is current law under a very recent amendment, which is its own caution: Ohio changed
    // this in the 136th General Assembly and a corpus pinned to older reading would not know.
    assert!(s.effective.contains("2026"), "effective {}", s.effective);
    assert!(s.legislation.contains("136"), "{}", s.legislation);
}

/// Local capacity takes the **lesser** of two valuations, which is a rule with a direction.
///
/// R.C. 3317.017(A)(1)(a): the minimum of the three-year average valuation and the most recent tax
/// year's taxable value. Under rising valuations the three-year average is the lower of the two, so
/// the provision has the same shape as recognized valuation had under the charge-off — it lags a
/// reappraisal rather than absorbing it at once — reached by averaging instead of by phasing in.
///
/// Recorded here because the corpus reproduced local capacity from the department's own
/// `Local_Capacity` sheet and so never had to read the rule that produces the input.
#[test]
fn local_capacity_takes_the_lesser_of_two_valuations() {
    let body = section("3317.017").body;
    assert!(body.contains("minimum of the district's three-year average valuation"));
    assert!(body.contains("most recent tax year"));
    assert!(
        body.contains("fiscal years 2026 and 2027"),
        "the section is written biennium by biennium, which is why a parameter series needs one \
         reading per budget"
    );
}

// -------------------------------------------------------------------------------------------
// The audit: what kind of thing each parameter is
// -------------------------------------------------------------------------------------------

/// **Ohio's funding parameters are not one kind of thing, and the corpus presented them as one.**
///
/// An audit pass over every `[open]` claim in the corpus turned up nine on `statutory_basis`
/// fields, all of the form "transcribed from the calculator; the correspondence with statute has
/// not been read." Reading the statute does not just confirm or deny them. It sorts them into four
/// kinds with quite different durability, and the distinction is invisible in the calculator
/// because a spreadsheet renders all four as a number in a cell:
///
/// | kind | where the value lives | what changes it |
/// |---|---|---|
/// | **legislated** | in the section, to four decimals | an act |
/// | **delegated** | statute names the department | a departmental decision |
/// | **measured** | statute specifies a computation over reported data | districts' own behaviour |
/// | **uncodified** | nowhere in the Revised Code | the biennial budget, and it lapses |
///
/// This test pins one example of each so the taxonomy cannot quietly collapse back into a list of
/// numbers.
#[test]
fn the_four_kinds_of_parameter_are_distinguishable_in_the_statute() {
    // LEGISLATED. The preschool flat grant and the halving of its weighted half, both in the
    // section, both to the digit. Four of the nine open items closed this way.
    let preschool = section("3317.0213").body;
    assert!(preschool.contains("$4,000 X the number of students who are preschool children"));
    assert!(
        preschool.contains("X 0.50"),
        "the halving is statutory, not departmental"
    );

    // Targeted assistance's coefficient and its size brackets, likewise.
    let targeted = section("3317.0217").body;
    assert!(targeted.contains("X 0.008"));
    assert!(targeted.contains("greater than or equal to 200"));

    // Gifted: the two per-pupil amounts sit in the omnibus section, the three unit prices in a
    // section it reaches by cross-reference. Both legislated; neither where the node looked.
    assert!(section("3317.022")
        .body
        .contains("$24 X the district's enrolled ADM"));
    let units = section("3317.051").body;
    for price in ["$85,776", "$89,378", "$80,974"] {
        assert!(units.contains(price), "gifted unit price {price}");
    }

    // MEASURED. Transportation's two rates are not chosen at all — they are trimmed means of what
    // districts reported spending last year, with the ten highest and ten lowest dropped. The
    // section contains no dollar figure whatever.
    let transport = section("3317.0212").body;
    assert!(transport.contains("statewide transportation cost per student"));
    assert!(transport.contains(
        "dividing the district's total costs for school bus service in the previous fiscal year"
    ));
    assert!(transport.contains("ten districts with the highest"));
    assert!(
        !transport.contains('$'),
        "if a rate ever appears here, transportation has changed kind"
    );

    // UNCODIFIED. Four supplements the department pays and the Revised Code does not mention.
    for absent in [
        "base funding supplement",
        "enrollment growth supplement",
        "performance supplement",
        "transition supplement",
    ] {
        assert!(
            !section("3317.022").body.to_lowercase().contains(absent),
            "{absent} should not be in the codified formula"
        );
    }
}

/// **$214.8m is paid under provisions that are not in the Revised Code**, and that is a fact about
/// durability rather than about legality.
///
/// The base funding supplement, the enrolment growth supplement, the performance supplement and
/// the formula transition supplement together are larger than the entire English learner and
/// career-technical programmes combined. None of them appears in R.C. 3317.022, the section that
/// assembles core foundation funding, nor anywhere else in the chapter this corpus cites. They are
/// in the budget act's temporary law — H.B. 96 of the 136th General Assembly — which `codes.ohio.gov`
/// does not carry, and which expires with the biennium unless re-enacted.
///
/// Nothing improper follows. Ohio funds a great deal this way. What follows is that a reader who
/// takes these for permanent programme funding is wrong about them in a way no figure on the
/// department's sheet would reveal, and the corpus was presenting them beside base cost as though
/// they were the same kind of commitment.
#[test]
fn the_uncodified_supplements_are_larger_than_two_codified_programmes_together() {
    let panel = project::panel::panel();
    let total = |f: fn(&project::panel::DistrictRecord) -> f64| panel.iter().map(f).sum::<f64>();

    let uncodified = total(|r| r.supplements.base_funding)
        + total(|r| r.supplements.growth)
        + total(|r| r.performance.amount)
        + total(|r| r.transition.transition_supplement);
    let codified = total(|r| r.english_learners.total()) + total(|r| r.career_technical.total());

    assert!(
        (uncodified / 1e6 - 214.8).abs() < 1.0,
        "uncodified supplements ${:.1}m",
        uncodified / 1e6
    );
    assert!(
        uncodified > codified,
        "${:.1}m uncodified against ${:.1}m of English learner and career-technical funding",
        uncodified / 1e6,
        codified / 1e6
    );
}
