# corpus

This corpus covers how Ohio funds its public schools — the state foundation formula and its
successive regimes, the constrained local property tax base beside it, the litigation that
tested the combination against the Ohio Constitution, and the programs that route money
around the formula. The level of analysis is the education agency in a fiscal period: a
single district, community school, JVSD, or STEM school in a single year.

The corpus holds schema, mechanism, legal history, metric definitions, and a curated set of
exemplar agencies. It does not hold the bulk per-agency-year facts — those live as committed
data files under [`crates/`](../../crates/) and are cited from here, because ~610 agencies
across the years with usable data would drown the graph. A node names a series and points at
it; it does not restate it.

Each file is one node. Nodes are small and focused — one concept, one relation, one artifact,
or one open question. Every node must have at least one outgoing link.

**The shapes are checked.** `.yidam/schemas/` holds JSON Schema for a node and for an ontology
class, generated from the definitions the web build validates against, and
[`.vscode/settings.json`](../../.vscode/settings.json) points the editor's YAML language server at
them — so a malformed node is underlined as it is typed. The same definitions stop the build. Four
authoring defects reached this corpus before any of that existed, and not one of them was visible
by reading the file: two were invalid YAML, four had a plain scalar containing `: ` that YAML reads
as a nested mapping, and one wrote its whole `links:` block as a paragraph, which is valid YAML and
made fifteen edges invisible to every consumer.

**`edges:` in an ontology class is illustrative unless it says otherwise.** Each class declares
`edge_policy: characteristic` or `exhaustive`. Characteristic — which all 13 currently are — means
the declared edges name what the class is *defined by*, and a node may use a relationship outside
that list. The corpus has always worked this way (90 relationships in use against 65 declared,
the difference being mostly single-use precise verbs); the field says so, so that a validator can
tell a deliberate coinage from a typo.

See [corpus conventions](../.vendor/prelude/guidelines/directories.md) for authoring rules,
the authored/generated node distinction, and `[verified]`/`[inference]`/`[open]` claim tags.

## A node writes prose into four fields, and each is for a different reader

A `description:` was measured carrying four documents at once — 53,196 words across 101 nodes, a
quarter of the paragraphs about this repository rather than about Ohio, and 130 paragraphs opening
in shouted capitals to win a skimming reader's attention against the wall of body copy around them.
[`the-four-genres-of-a-description`](../decisions/the-four-genres-of-a-description.yml) separates
them.

| Field | Holds | Where it renders |
|---|---|---|
| `summary` | the lead — what the thing *is* | under the `h1`, and in every `<meta>` and OG card |
| `description` | the subject: mechanism, statute, history | the body |
| `findings` | what this repository computed, and how to read it | its own section |
| `revisions` | what the node used to say and no longer does | a collapsed disclosure |

**`summary:` is required, at most 50 words, and may not contain a markdown link.** It is the one
string five call sites substitute for the whole node, and none of those five can render a link.
Both limits are in the JSON Schema, so an editor says so while it is being typed.

**`revisions:` is a list, and each entry has four fields.** `was` — the claim as it stood. `now` —
what replaced it. `found_by` — the test, source or record that settled it, which is the field that
earns the structure, because it turns "this was wrong once" into a check somebody can re-run.
`reach` — what else the mistake touched, or the explicit statement that nothing downstream moved.

**The corpus is never rewritten to have always been right**, the same rule
[decision records](../decisions/README.md) follow and for the same reason: the wrong turn is the
most useful thing on the page, and editing it out leaves a document that teaches nothing. A
withdrawal moves out of the body so a reader does not meet it as current, and it does not leave
the file.

**Four checks run at build time and report rather than stop.** A `summary` that is its own
description cut; a `description` over 400 words, or carrying a shouted lead, or referring to the
corpus rather than to Ohio. `pnpm corpus:report` lists them.

**The count is zero, and it is asserted as a zero rather than as a ceiling.** While the genre
migration was staged the pin in `web/tests/unit/schema.spec.ts` was an upper bound that only moved
down — 118 nodes, then 105, 100, 96, 92, 72, 38, 20. Every node has been through it now, so a
ratchet is the wrong shape: there is nothing left to shrink, and a bound of 20 would quietly permit
twenty new defects. The failure the pin exists to catch is a finding folded back into a
`description`, and that is a change from zero.

## `[unentered]` — a fourth mark, and why it is not a fourth confidence level

The prelude's three tags grade *how well a claim is supported*. This domain needs one more mark,
and it sits on a different axis: whether there is a claim at all.

- `[open]` — a live question. Why the fortieth-ranked district is the local capacity benchmark
  and not some other rank is not stated in the section, and reading more of the section will not
  say. Somebody has to find out.
- `[unentered]` — a knowable value nobody has typed in. A district's `established` date, a bill's
  `vetoes` list, an agency's typology code. No investigation is required; a source is open and
  the field is empty.

The [audit of every open claim](../decisions/the-open-item-audit.yml) found that four of the
largest `[open]` clusters were the second kind and concluded that "a future pass should probably
distinguish 'unknown' from 'unentered' in the notation rather than marking both `[open]`." It
matters because the two imply different work and different reading. A corpus reporting a hundred
and fifty open questions sounds like one with deep uncertainty about its domain; reporting live
questions and empty fields apart says where the thinking is needed and where the typing is.

`[unentered]` is a **narrowing** of `[open]`, not a replacement for it, so nothing outside this
repository has to learn it: a consumer that knows only the prelude's three tags can treat an
`[unentered]` field as unfilled and lose nothing.

The line was drawn conservatively, and one cluster the audit put on the empty-field side stayed
`[open]`: `vetoes` reads "line-item vetoes exercised; education funding effects not yet assessed",
and assessing an effect is analysis rather than transcription. A distinction that inflates itself
is not worth having. Two fields split across both columns for the same reason —
[Eastland-Fairfield](education-agency/eastland-fairfield-ctc.yml)'s typology is genuinely open,
because JVSDs are outside the department's typology altogether and how they should be grouped for
peer comparison is unresolved, while its neighbours are simply missing a code that exists.

The counts below are generated. A hand count is what the audit did, and it answered the question
once for a corpus that has been changing ever since.

## Claim inventory

<!-- REGEN: yidam claim-audit
Regenerated by: `edfund-connect index`
Fields: count per claim tag across every node; then the unresolved marks by the field they
        sit in, `[open]` and `[unentered]` in separate columns.
-->
| Tag | Count | What it records |
|---|--:|---|
| `[verified]` | 901 | supported by a committed primary source |
| `[inference]` | 241 | drawn from verified facts, not witnessed |
| `[open]` | 200 | a live question — unknown, contested, or being worked |
| `[unentered]` | 38 | a knowable value nobody has typed in yet |

238 unresolved marks in total, 200 of them live questions and 38 of them empty fields. Before the two were distinguished the corpus reported the sum as its count of what it does not know, which overstated it by 15%.

| Field | `[open]` | `[unentered]` |
|---|--:|--:|
| `description` | 54 | 3 |
| `findings` | 51 | 1 |
| `vetoes` | 11 | 1 |
| `series` | 12 | 0 |
| `established` | 0 | 7 |
| `typology` | 1 | 5 |
| `statutory_basis` | 6 | 0 |
| `roles` | 4 | 1 |
| `mechanism` | 5 | 0 |
| `amount` | 5 | 0 |
| `series_path` | 0 | 4 |
| `eligibility` | 4 | 0 |
| `accountability_effect` | 4 | 0 |
| `sponsors` | 1 | 2 |
| `revisions` | 3 | 0 |
| `restriction` | 3 | 0 |
| `performance` | 0 | 3 |
| `grades` | 0 | 3 |
| `definition` | 3 | 0 |
| `contested` | 3 | 0 |
| `caveats` | 3 | 0 |
| `unpriced` | 2 | 0 |
| `subject` | 2 | 0 |
| `legal_basis` | 2 | 0 |
| `holding` | 2 | 0 |
| `general_assembly` | 0 | 2 |
| `exit` | 1 | 1 |
| `effective` | 0 | 2 |
| `confidence` | 2 | 0 |
| `calculator` | 2 | 0 |
| `boundary_note` | 2 | 0 |
| `appropriation_line` | 1 | 1 |
| `adoption_evidence` | 2 | 0 |
| `trigger` | 1 | 0 |
| `status` | 1 | 0 |
| `sensitivity` | 1 | 0 |
| `remedy` | 1 | 0 |
| `measure` | 0 | 1 |
| `kind` | 1 | 0 |
| `irn` | 0 | 1 |
| `fiscal_effect` | 1 | 0 |
| `district_funding_effect` | 1 | 0 |
| `context` | 1 | 0 |
| `appropriating_bill` | 1 | 0 |

**37 recorded withdrawals across 21 nodes.** A claim the corpus published and no longer stands behind is kept in a `revisions:` block rather than edited out, with the test or source that settled it — see [`the-four-genres-of-a-description`](../decisions/the-four-genres-of-a-description.yml). Counted here for the same reason the tags above are: how often this corpus has corrected itself is a fact about it, and one nobody would think to update by hand.
<!-- /REGEN -->

## Node index

<!-- REGEN: yidam corpus-index
Regenerated by: `yidam corpus-index`
Fields per node: filename, title, kind (concept/relation/artifact/question/hypothesis),
                 outgoing link count, incoming link count, line count, last commit date.
Sorted by: kind, then alphabetically.
-->
| Node | Class | Label | Out | In |
|---|---|---|--:|--:|
| [`essa`](accountability-regime/essa.yml) | accountability-regime | Every Student Succeeds Act | 6 | 6 |
| [`ohio-report-card`](accountability-regime/ohio-report-card.yml) | accountability-regime | Ohio School Report Card | 7 | 2 |
| [`department-of-education-and-workforce`](actor/department-of-education-and-workforce.yml) | actor | Department of Education and Workforce | 3 | 6 |
| [`ohio-department-of-education`](actor/ohio-department-of-education.yml) | actor | Ohio Department of Education (superseded) | 2 | 2 |
| [`ohio-general-assembly`](actor/ohio-general-assembly.yml) | actor | Ohio General Assembly | 6 | 5 |
| [`supreme-court-of-ohio`](actor/supreme-court-of-ohio.yml) | actor | Supreme Court of Ohio | 3 | 5 |
| [`adequacy`](doctrine/adequacy.yml) | doctrine | Adequacy | 8 | 25 |
| [`equity`](doctrine/equity.yml) | doctrine | Equity | 9 | 31 |
| [`thorough-and-efficient`](doctrine/thorough-and-efficient.yml) | doctrine | Thorough and Efficient | 5 | 14 |
| [`fund-the-plan-and-retire-the-guarantee`](draft-legislation/fund-the-plan-and-retire-the-guarantee.yml) | draft-legislation | Fund the Plan and Retire the Guarantee | 8 | 0 |
| [`hb-643-136-introduced`](draft-legislation/hb-643-136-introduced.yml) | draft-legislation | H.B. 643 (136th G.A., as introduced) | 3 | 0 |
| [`hb-96-with-refreshed-inputs`](draft-legislation/hb-96-with-refreshed-inputs.yml) | draft-legislation | "Counterfactual: H.B. 96 with FY2024 Cost Inputs" | 6 | 1 |
| [`cleveland-municipal`](education-agency/cleveland-municipal.yml) | education-agency | Cleveland Municipal School District | 3 | 7 |
| [`eastland-fairfield-ctc`](education-agency/eastland-fairfield-ctc.yml) | education-agency | Eastland-Fairfield Career and Technical Schools | 3 | 2 |
| [`electronic-classroom-of-tomorrow`](education-agency/electronic-classroom-of-tomorrow.yml) | education-agency | Electronic Classroom of Tomorrow (closed) | 2 | 1 |
| [`northern-local-perry`](education-agency/northern-local-perry.yml) | education-agency | Northern Local School District (Perry County) | 6 | 11 |
| [`perrysburg-exempted-village`](education-agency/perrysburg-exempted-village.yml) | education-agency | Perrysburg Exempted Village School District | 4 | 1 |
| [`toledo-city`](education-agency/toledo-city.yml) | education-agency | Toledo City School District | 3 | 2 |
| [`upper-arlington-city`](education-agency/upper-arlington-city.yml) | education-agency | Upper Arlington City School District | 3 | 6 |
| [`fy2012-13`](fiscal-period/fy2012-13.yml) | fiscal-period | FY2012-2013 Biennium | 3 | 2 |
| [`fy2014-15`](fiscal-period/fy2014-15.yml) | fiscal-period | FY2014-2015 Biennium | 3 | 1 |
| [`fy2016-17`](fiscal-period/fy2016-17.yml) | fiscal-period | FY2016-2017 Biennium | 3 | 1 |
| [`fy2018-19`](fiscal-period/fy2018-19.yml) | fiscal-period | FY2018-2019 Biennium | 3 | 1 |
| [`fy2020-21`](fiscal-period/fy2020-21.yml) | fiscal-period | FY2020-2021 Biennium | 3 | 3 |
| [`fy2022`](fiscal-period/fy2022.yml) | fiscal-period | Fiscal Year 2022 | 3 | 3 |
| [`fy2022-23`](fiscal-period/fy2022-23.yml) | fiscal-period | FY2022-23 Biennium | 5 | 2 |
| [`fy2024-25`](fiscal-period/fy2024-25.yml) | fiscal-period | FY2024-25 Biennium | 5 | 3 |
| [`fy2026`](fiscal-period/fy2026.yml) | fiscal-period | Fiscal Year 2026 | 3 | 8 |
| [`fy2026-27`](fiscal-period/fy2026-27.yml) | fiscal-period | FY2026-27 Biennium | 4 | 6 |
| [`fy2027`](fiscal-period/fy2027.yml) | fiscal-period | Fiscal Year 2027 | 6 | 3 |
| [`charge-off-local-share`](formula-component/charge-off-local-share.yml) | formula-component | Charge-Off Local Share | 6 | 8 |
| [`fsfp-base-cost-calculation`](formula-component/fsfp-base-cost-calculation.yml) | formula-component | FSFP Base Cost Calculation | 7 | 9 |
| [`fsfp-career-technical-weights`](formula-component/fsfp-career-technical-weights.yml) | formula-component | FSFP Career-Technical Weights | 4 | 3 |
| [`fsfp-disadvantaged-pupil-impact-aid`](formula-component/fsfp-disadvantaged-pupil-impact-aid.yml) | formula-component | FSFP Disadvantaged Pupil Impact Aid | 2 | 2 |
| [`fsfp-english-learner-weights`](formula-component/fsfp-english-learner-weights.yml) | formula-component | FSFP English Learner Weights | 4 | 3 |
| [`fsfp-enrolment-supplements`](formula-component/fsfp-enrolment-supplements.yml) | formula-component | FSFP Base and Enrollment Growth Supplements | 3 | 1 |
| [`fsfp-formula-transition-supplement`](formula-component/fsfp-formula-transition-supplement.yml) | formula-component | FSFP Formula Transition Supplement | 4 | 4 |
| [`fsfp-gifted-units`](formula-component/fsfp-gifted-units.yml) | formula-component | FSFP Gifted Identification and Units | 3 | 1 |
| [`fsfp-local-capacity-measure`](formula-component/fsfp-local-capacity-measure.yml) | formula-component | FSFP Local Capacity Measure | 5 | 8 |
| [`fsfp-performance-supplement`](formula-component/fsfp-performance-supplement.yml) | formula-component | FSFP Performance Supplement | 7 | 4 |
| [`fsfp-preschool-special-education`](formula-component/fsfp-preschool-special-education.yml) | formula-component | FSFP Preschool Special Education | 4 | 2 |
| [`fsfp-special-education-weights`](formula-component/fsfp-special-education-weights.yml) | formula-component | FSFP Special Education Weights | 5 | 9 |
| [`fsfp-targeted-assistance`](formula-component/fsfp-targeted-assistance.yml) | formula-component | FSFP Targeted Assistance | 4 | 4 |
| [`fsfp-transportation`](formula-component/fsfp-transportation.yml) | formula-component | FSFP Transportation | 6 | 3 |
| [`guarantee-open-enrolment-clawback`](formula-component/guarantee-open-enrolment-clawback.yml) | formula-component | Guarantee Open Enrolment Clawback | 3 | 2 |
| [`temporary-transitional-aid-guarantee`](formula-component/temporary-transitional-aid-guarantee.yml) | formula-component | Temporary Transitional Aid Guarantee | 10 | 16 |
| [`bridge-formula`](funding-regime/bridge-formula.yml) | funding-regime | Bridge Formula | 12 | 16 |
| [`equal-yield-formula`](funding-regime/equal-yield-formula.yml) | funding-regime | Equal Yield Formula | 3 | 2 |
| [`evidence-based-model`](funding-regime/evidence-based-model.yml) | funding-regime | Evidence-Based Model | 3 | 6 |
| [`fair-school-funding-plan`](funding-regime/fair-school-funding-plan.yml) | funding-regime | Fair School Funding Plan | 21 | 27 |
| [`foundation-base-cost-formula`](funding-regime/foundation-base-cost-formula.yml) | funding-regime | Foundation Base Cost Formula | 5 | 7 |
| [`academic-distress-commission`](intervention/academic-distress-commission.yml) | intervention | Academic Distress Commission | 6 | 4 |
| [`lea-level-action`](intervention/lea-level-action.yml) | intervention | Additional Optional Action (LEA level) | 4 | 2 |
| [`more-rigorous-interventions`](intervention/more-rigorous-interventions.yml) | intervention | More Rigorous Interventions (CSI) | 5 | 2 |
| [`hb-1-2009`](legislation/hb-1-2009.yml) | legislation | Am. Sub. H.B. 1 (2009) — FY2010-11 Budget; Evidence-Based Model | 2 | 3 |
| [`hb-110-2021`](legislation/hb-110-2021.yml) | legislation | Am. Sub. H.B. 110 (2021) — FY2022-23 Budget; Fair School Funding Plan | 5 | 9 |
| [`hb-153-2011`](legislation/hb-153-2011.yml) | legislation | Am. Sub. H.B. 153 (2011) — FY2012-13 Budget; Bridge Formula | 6 | 3 |
| [`hb-166-2019`](legislation/hb-166-2019.yml) | legislation | Am. Sub. H.B. 166 (2019) — FY2020-21 Budget; the Year There Was No Formula | 4 | 3 |
| [`hb-33-2023`](legislation/hb-33-2023.yml) | legislation | Am. Sub. H.B. 33 (2023) — FY2024-25 Budget | 8 | 12 |
| [`hb-49-2017`](legislation/hb-49-2017.yml) | legislation | Am. Sub. H.B. 49 (2017) — FY2018-19 Budget; the Guarantee Made Conditional | 4 | 2 |
| [`hb-583-2022`](legislation/hb-583-2022.yml) | legislation | Sub. H.B. 583 (2022) — corrective and technical changes to the Fair School Funding Plan | 5 | 1 |
| [`hb-59-2013`](legislation/hb-59-2013.yml) | legislation | Am. Sub. H.B. 59 (2013) — FY2014-15 Budget; a Formula Over the Bridge | 3 | 2 |
| [`hb-64-2015`](legislation/hb-64-2015.yml) | legislation | Am. Sub. H.B. 64 (2015) — FY2016-17 Budget; Capacity Aid | 5 | 2 |
| [`hb-66-2005`](legislation/hb-66-2005.yml) | legislation | Am. Sub. H.B. 66 (2005) — FY2006-07 Budget; Tangible Personal Property Tax Phase-Out | 2 | 4 |
| [`hb-920-1976`](legislation/hb-920-1976.yml) | legislation | Am. Sub. H.B. 920 (1976) — Tax Reduction Factors | 4 | 12 |
| [`hb-94-2001`](legislation/hb-94-2001.yml) | legislation | Am. Sub. H.B. 94 (2001) — FY2002-03 Budget; the post-DeRolph II formula | 9 | 2 |
| [`hb-96-2025`](legislation/hb-96-2025.yml) | legislation | Am. Sub. H.B. 96 (2025) — FY2026-27 Budget | 6 | 10 |
| [`ohio-constitution-article-vi-section-2`](legislation/ohio-constitution-article-vi-section-2.yml) | legislation | Ohio Constitution, Article VI, Section 2 (1851) | 2 | 5 |
| [`cincinnati-v-walter-1979`](litigation/cincinnati-v-walter-1979.yml) | litigation | Cincinnati City School District Board of Education v. Walter (1979) | 5 | 4 |
| [`derolph-i-1997`](litigation/derolph-i-1997.yml) | litigation | DeRolph v. State (DeRolph I, 1997) | 6 | 14 |
| [`derolph-ii-2000`](litigation/derolph-ii-2000.yml) | litigation | DeRolph v. State (DeRolph II, 2000) | 13 | 4 |
| [`derolph-iii-2001`](litigation/derolph-iii-2001.yml) | litigation | DeRolph v. State (DeRolph III, 2001) | 12 | 3 |
| [`derolph-iv-2002`](litigation/derolph-iv-2002.yml) | litigation | DeRolph v. State (DeRolph IV, 2002) | 10 | 5 |
| [`vouchers-hurt-ohio-2025`](litigation/vouchers-hurt-ohio-2025.yml) | litigation | EdChoice Constitutional Challenge (Franklin County, 2025) | 4 | 3 |
| [`assessed-valuation-per-pupil`](metric/assessed-valuation-per-pupil.yml) | metric | Assessed Valuation Per Pupil | 7 | 4 |
| [`effective-operating-millage`](metric/effective-operating-millage.yml) | metric | Effective Operating Millage | 7 | 1 |
| [`enrolled-adm`](metric/enrolled-adm.yml) | metric | Enrolled ADM | 6 | 8 |
| [`expenditure-per-equivalent-pupil`](metric/expenditure-per-equivalent-pupil.yml) | metric | Expenditure Per Equivalent Pupil | 8 | 6 |
| [`general-fund-cash-balance`](metric/general-fund-cash-balance.yml) | metric | General Fund Cash Balance | 2 | 3 |
| [`per-pupil-operating-expenditure`](metric/per-pupil-operating-expenditure.yml) | metric | Per-Pupil Operating Expenditure | 12 | 4 |
| [`performance-index`](metric/performance-index.yml) | metric | Performance Index | 5 | 12 |
| [`progress-value-added`](metric/progress-value-added.yml) | metric | Progress (Value-Added) | 5 | 5 |
| [`state-share-percentage`](metric/state-share-percentage.yml) | metric | State Share Percentage | 5 | 3 |
| [`education-savings-account-act`](model-policy/education-savings-account-act.yml) | model-policy | Education Savings Account Act | 4 | 0 |
| [`parental-choice-scholarship-act`](model-policy/parental-choice-scholarship-act.yml) | model-policy | Parental Choice Scholarship Program Act (Universal Eligibility) | 4 | 1 |
| [`appropriation-proration-factor`](parameter/appropriation-proration-factor.yml) | parameter | Appropriation Proration Factor | 2 | 2 |
| [`base-cost-per-pupil`](parameter/base-cost-per-pupil.yml) | parameter | Base Cost Per Pupil | 9 | 18 |
| [`fsfp-phase-in-percentage`](parameter/fsfp-phase-in-percentage.yml) | parameter | FSFP Phase-In Percentage | 4 | 13 |
| [`guarantee-funding-base`](parameter/guarantee-funding-base.yml) | parameter | Guarantee Funding Base | 5 | 3 |
| [`local-share-charge-off-millage`](parameter/local-share-charge-off-millage.yml) | parameter | Local Share Charge-Off Millage | 8 | 4 |
| [`twenty-mill-floor`](parameter/twenty-mill-floor.yml) | parameter | Twenty-Mill Floor | 4 | 8 |
| [`autism-scholarship`](program/autism-scholarship.yml) | program | Autism Scholarship | 4 | 0 |
| [`classroom-facilities-assistance`](program/classroom-facilities-assistance.yml) | program | Classroom Facilities Assistance Program | 3 | 2 |
| [`cleveland-scholarship`](program/cleveland-scholarship.yml) | program | Cleveland Scholarship and Tutoring Program | 4 | 4 |
| [`edchoice-expansion`](program/edchoice-expansion.yml) | program | EdChoice Expansion Scholarship | 4 | 9 |
| [`edchoice-scholarship`](program/edchoice-scholarship.yml) | program | Traditional EdChoice Scholarship | 4 | 2 |
| [`jon-peterson-special-needs`](program/jon-peterson-special-needs.yml) | program | Jon Peterson Special Needs Scholarship | 3 | 1 |
| [`casino-tax-distribution`](revenue-stream/casino-tax-distribution.yml) | revenue-stream | Casino Tax — County Student Fund | 6 | 1 |
| [`esser`](revenue-stream/esser.yml) | revenue-stream | ESSER — Elementary and Secondary School Emergency Relief | 4 | 2 |
| [`idea-part-b`](revenue-stream/idea-part-b.yml) | revenue-stream | IDEA Part B | 5 | 3 |
| [`local-property-tax`](revenue-stream/local-property-tax.yml) | revenue-stream | Local Property Tax | 6 | 8 |
| [`lottery-profits`](revenue-stream/lottery-profits.yml) | revenue-stream | Lottery Profits Education Fund | 4 | 1 |
| [`state-foundation-aid`](revenue-stream/state-foundation-aid.yml) | revenue-stream | State Foundation Aid | 5 | 16 |
| [`title-i`](revenue-stream/title-i.yml) | revenue-stream | Title I, Part A | 4 | 5 |
| [`tpp-replacement-payments`](revenue-stream/tpp-replacement-payments.yml) | revenue-stream | Tangible Personal Property Tax Replacement Payments | 3 | 4 |
| [`fsfp-input-year-refresh`](scenario/fsfp-input-year-refresh.yml) | scenario | FSFP Cost Input Refresh vs. Freeze | 9 | 6 |
| [`guarantee-phase-out`](scenario/guarantee-phase-out.yml) | scenario | Phasing Out the Temporary Transitional Aid Guarantee | 11 | 3 |
| [`anton-grdina`](school/anton-grdina.yml) | school | Anton Grdina | 6 | 1 |
| [`barrington-road-elementary`](school/barrington-road-elementary.yml) | school | Barrington Road Elementary School | 4 | 2 |
| [`sheridan-high-school`](school/sheridan-high-school.yml) | school | Sheridan High School | 6 | 1 |

110 nodes across 18 classes. **4 have nothing pointing at them**, which the corpus rules treat as a gap rather than a fact about the node.
<!-- /REGEN -->

## Semantic index status

<!-- REGEN: yidam index-status
Regenerated by: `yidam index-status`
Fields: total nodes indexed, embedding model, index freshness (last indexed commit vs HEAD),
        stale node count.
-->
No semantic index is built. The corpus is 110 nodes and fits in context; an index is added when direct retrieval stops working, which has not happened.
<!-- /REGEN -->
