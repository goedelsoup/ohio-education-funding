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
`edge_policy: characteristic` or `exhaustive`. Characteristic — which all eighteen currently are —
means the declared edges name what the class is *defined by*, and a node may use a relationship
outside that list. The corpus has always worked this way, the difference being mostly single-use
precise verbs; the field says so, so that a validator can tell a deliberate coinage from a typo.
The evidence for it is measured under [relationship vocabulary](#relationship-vocabulary) rather
than stated here, because the version stated here was wrong for a long time, in seventeen places
at once.

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

## `[unentered]` — the fourth mark, and where it went

The prelude's three tags grade *how well a claim is supported*. This domain needed one more mark,
and it sat on a different axis: whether there is a claim at all.

- `[open]` — a live question. Why the fortieth-ranked district is the local capacity benchmark and
  not some other rank is not stated in the section, and reading more of the section will not say.
  Somebody has to find out.
- **A field nobody has filled in** — a knowable value, a source open, and nothing typed. No
  investigation required.

For a long time the second was written `[unentered]`, inline, in the same brackets as the other
three. **This section used to argue that it was not a fourth confidence level while the parser went
on rendering it as a fourth badge**, and that contradiction is the whole story: sitting in the
sequence *is* the claim that it belongs to the sequence, so inline it read as "worse than open"
whatever it wore.

**It is structure now.** A node carries `unfilled:` entries — a `field` naming the missing fact and
a `why` saying where the value lives — and they render as a block in the position the content would
occupy rather than as a mark in somebody's sentence. Thirty-three property-level marks moved; four
that named a gap in this corpus rather than an empty field on a node were reclassified `[open]`,
which is what they always were; and four more on draft legislation were dropped outright, because a
bill nobody introduced has no General Assembly and no sponsors and never will.

The count below is therefore zero and should stay there. `web/tests/unit/prose.spec.ts` fails if a
node writes the old mark, and the badge assertion beside it fails if `unentered` is added back to
the rendering vocabulary.

`field` names the missing *fact*, not necessarily a property key. Only a third of these were whole
fields; the rest sat inside a property carrying real content — an agency's `typology` holds "Urban,
very high poverty" and lacks only the department's own code. Naming the property there would claim
the whole field was empty, which is worse than the badge it replaced.

The counts below are generated. A hand count is what the audit did, and it answered the question
once for a corpus that has been changing ever since.

## `figures:` — the numbers a node quotes, bound to the crate that computes them

`[verified — crates/regime-diff]` is hand-typed text, and nothing related it to
`crates/regime-diff`. A correction's blast radius was therefore whichever files the author
happened to open. That is not a hypothetical failure: the `recognized-valuation` correction
reached **three of its six carriers**, and two `formula-component` nodes went on publishing a
reversed sign on the headline distributional claim, live, under `[verified]`, while all 849 crate
tests stayed green.

A node now binds the number itself. `crates/figures` computes each figure from the public API of
the crate that owns it and writes [`crates/figures.json`](../../crates/figures.json);
`web/tests/unit/corpusFigures.spec.ts` asserts a three-way agreement between the manifest, the
entry, and the sentence a reader sees.

```yaml
figures:
  - key: regime-diff/charge-off-zeroes-base-cost-aid
    value: 65
    field: sensitivity
    as_written: "65 of 606 districts"
```

**`as_written` is a phrase and not a numeral, and that is the leg worth having.** The reversal
above was not a wrong number — it was the *same* two numerals attached to the opposite regimes. A
numeral match passes that. `316 districts would have done better under the charge-off` stops being
a substring the moment the sentence is turned around.

**`revisions:` cannot be bound, on purpose.** A revision records what a node used to say, and this
corpus is never rewritten to have always been right; a check that made a withdrawal track a moving
figure would demand editing the record of a correction. The body restates the live figure and the
body is what binds.

**Roughly a fifth of the crate-attributed numeric claims are covered.** A figure is exportable when
the crate computes it through public API; the rest live inside a test file on a fixture parser that
test declares privately. The spec pins the bound count as a floor at its value rather than below
it, so the coverage can only rise — see #131 for why a floor with slack in it is not a ratchet.

## Claim inventory

<!-- REGEN: edfund-connect claim-audit
Regenerated by: `edfund-connect index`
Fields: count per claim tag across every node; then the unresolved marks by the field they
        sit in, `[open]` and `[unentered]` in separate columns.
-->
| Tag | Count | What it records |
|---|--:|---|
| `[verified]` | 999 | supported by a committed primary source |
| `[inference]` | 254 | drawn from verified facts, not witnessed |
| `[open]` | 208 | a live question — unknown, contested, or being worked |
| `[unentered]` | 0 | a knowable value nobody has typed in yet |

208 unresolved marks, and every one of them is a live question. The fourth mark is gone from the prose: a field nobody has filled in is carried as `unfilled:` structure on the node it belongs to, which is what `[unentered]` used to say inline on an axis it did not belong to.

| Field | `[open]` | `[unentered]` |
|---|--:|--:|
| `findings` | 60 | 0 |
| `description` | 54 | 0 |
| `series` | 12 | 0 |
| `revisions` | 8 | 0 |
| `statutory_basis` | 6 | 0 |
| `accountability_effect` | 6 | 0 |
| `mechanism` | 5 | 0 |
| `amount` | 5 | 0 |
| `roles` | 4 | 0 |
| `eligibility` | 4 | 0 |
| `vetoes` | 3 | 0 |
| `restriction` | 3 | 0 |
| `definition` | 3 | 0 |
| `contested` | 3 | 0 |
| `caveats` | 3 | 0 |
| `unpriced` | 2 | 0 |
| `subject` | 2 | 0 |
| `legal_basis` | 2 | 0 |
| `holding` | 2 | 0 |
| `confidence` | 2 | 0 |
| `calculator` | 2 | 0 |
| `boundary_note` | 2 | 0 |
| `adoption_evidence` | 2 | 0 |
| `typology` | 1 | 0 |
| `trigger` | 1 | 0 |
| `status` | 1 | 0 |
| `sponsors` | 1 | 0 |
| `sensitivity` | 1 | 0 |
| `remedy` | 1 | 0 |
| `kind` | 1 | 0 |
| `fiscal_effect` | 1 | 0 |
| `exit` | 1 | 0 |
| `district_funding_effect` | 1 | 0 |
| `context` | 1 | 0 |
| `appropriation_line` | 1 | 0 |
| `appropriating_bill` | 1 | 0 |

**56 recorded withdrawals across 32 nodes.** A claim the corpus published and no longer stands behind is kept in a `revisions:` block rather than edited out, with the test or source that settled it — see [`the-four-genres-of-a-description`](../decisions/the-four-genres-of-a-description.yml). Counted here for the same reason the tags above are: how often this corpus has corrected itself is a fact about it, and one nobody would think to update by hand.
<!-- /REGEN -->

## Relationship vocabulary

<!-- REGEN: edfund-connect edge-vocabulary
Regenerated by: `edfund-connect index`
Fields: edges between nodes; distinct relationships in use; relationships declared across every
        ontology class; edges whose relationship its own class does not declare; relationships
        used exactly once.
An edge here is a `links:` entry pointing at another node — `instance-of` and `sourced-from`
leave the corpus and are not counted.
-->
| Measure | Count |
|---|--:|
| edges between nodes | 503 |
| distinct relationships in use | 158 |
| relationships declared across every class | 70 |
| edges whose relationship its class does not declare | 258 |
| relationships used exactly once | 80 |

**51% of edges use a relationship the class does not declare**, and 80 of the 158 relationships in use are used a single time. That is the case for `edge_policy: characteristic`: closing the vocabulary would reject those edges or require 80 declarations that each describe one link.
<!-- /REGEN -->

This was a hand-written sentence in fifteen ontology files, in the paragraph above, and in
`web/src/lib/schema/corpus.ts` — seventeen copies of *"90 relationships in use against 65
declared"*, *"46 of the 90 are undeclared"*, *"a third of the graph's edges"*. Every figure in it
was stale, the corpus having grown past all three, and none of the seventeen copies had any way of
knowing — while the generated node index below them stayed current on every run. The argument was
right and its evidence had rotted, which is the failure a generated block exists to prevent.

## Node index

<!-- REGEN: edfund-connect corpus-index
Regenerated by: `edfund-connect index`
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
| [`northern-local-perry`](education-agency/northern-local-perry.yml) | education-agency | Northern Local School District (Perry County) | 6 | 13 |
| [`perrysburg-exempted-village`](education-agency/perrysburg-exempted-village.yml) | education-agency | Perrysburg Exempted Village School District | 4 | 1 |
| [`toledo-city`](education-agency/toledo-city.yml) | education-agency | Toledo City School District | 3 | 2 |
| [`upper-arlington-city`](education-agency/upper-arlington-city.yml) | education-agency | Upper Arlington City School District | 3 | 8 |
| [`fy2002-03`](fiscal-period/fy2002-03.yml) | fiscal-period | FY2002-2003 Biennium | 4 | 2 |
| [`fy2004-05`](fiscal-period/fy2004-05.yml) | fiscal-period | FY2004-2005 Biennium | 5 | 2 |
| [`fy2006-07`](fiscal-period/fy2006-07.yml) | fiscal-period | FY2006-2007 Biennium | 5 | 2 |
| [`fy2008-09`](fiscal-period/fy2008-09.yml) | fiscal-period | FY2008-2009 Biennium | 4 | 2 |
| [`fy2010-11`](fiscal-period/fy2010-11.yml) | fiscal-period | FY2010-2011 Biennium | 6 | 4 |
| [`fy2012-13`](fiscal-period/fy2012-13.yml) | fiscal-period | FY2012-2013 Biennium | 4 | 3 |
| [`fy2014-15`](fiscal-period/fy2014-15.yml) | fiscal-period | FY2014-2015 Biennium | 4 | 2 |
| [`fy2016-17`](fiscal-period/fy2016-17.yml) | fiscal-period | FY2016-2017 Biennium | 4 | 2 |
| [`fy2018-19`](fiscal-period/fy2018-19.yml) | fiscal-period | FY2018-2019 Biennium | 4 | 2 |
| [`fy2020-21`](fiscal-period/fy2020-21.yml) | fiscal-period | FY2020-2021 Biennium | 4 | 4 |
| [`fy2022`](fiscal-period/fy2022.yml) | fiscal-period | Fiscal Year 2022 | 3 | 3 |
| [`fy2022-23`](fiscal-period/fy2022-23.yml) | fiscal-period | FY2022-23 Biennium | 6 | 4 |
| [`fy2024-25`](fiscal-period/fy2024-25.yml) | fiscal-period | FY2024-25 Biennium | 5 | 5 |
| [`fy2026`](fiscal-period/fy2026.yml) | fiscal-period | Fiscal Year 2026 | 4 | 8 |
| [`fy2026-27`](fiscal-period/fy2026-27.yml) | fiscal-period | FY2026-27 Biennium | 5 | 6 |
| [`fy2027`](fiscal-period/fy2027.yml) | fiscal-period | Fiscal Year 2027 | 6 | 3 |
| [`charge-off-local-share`](formula-component/charge-off-local-share.yml) | formula-component | Charge-Off Local Share | 6 | 8 |
| [`fsfp-base-cost-calculation`](formula-component/fsfp-base-cost-calculation.yml) | formula-component | FSFP Base Cost Calculation | 7 | 9 |
| [`fsfp-career-technical-weights`](formula-component/fsfp-career-technical-weights.yml) | formula-component | FSFP Career-Technical Weights | 4 | 3 |
| [`fsfp-disadvantaged-pupil-impact-aid`](formula-component/fsfp-disadvantaged-pupil-impact-aid.yml) | formula-component | FSFP Disadvantaged Pupil Impact Aid | 2 | 5 |
| [`fsfp-english-learner-weights`](formula-component/fsfp-english-learner-weights.yml) | formula-component | FSFP English Learner Weights | 4 | 3 |
| [`fsfp-enrolment-supplements`](formula-component/fsfp-enrolment-supplements.yml) | formula-component | FSFP Base and Enrollment Growth Supplements | 5 | 3 |
| [`fsfp-formula-transition-supplement`](formula-component/fsfp-formula-transition-supplement.yml) | formula-component | FSFP Formula Transition Supplement | 8 | 4 |
| [`fsfp-gifted-units`](formula-component/fsfp-gifted-units.yml) | formula-component | FSFP Gifted Identification and Units | 3 | 2 |
| [`fsfp-local-capacity-measure`](formula-component/fsfp-local-capacity-measure.yml) | formula-component | FSFP Local Capacity Measure | 5 | 8 |
| [`fsfp-performance-supplement`](formula-component/fsfp-performance-supplement.yml) | formula-component | FSFP Performance Supplement | 9 | 6 |
| [`fsfp-preschool-special-education`](formula-component/fsfp-preschool-special-education.yml) | formula-component | FSFP Preschool Special Education | 4 | 2 |
| [`fsfp-special-education-weights`](formula-component/fsfp-special-education-weights.yml) | formula-component | FSFP Special Education Weights | 5 | 9 |
| [`fsfp-targeted-assistance`](formula-component/fsfp-targeted-assistance.yml) | formula-component | FSFP Targeted Assistance | 4 | 4 |
| [`fsfp-transportation`](formula-component/fsfp-transportation.yml) | formula-component | FSFP Transportation | 6 | 4 |
| [`guarantee-open-enrolment-clawback`](formula-component/guarantee-open-enrolment-clawback.yml) | formula-component | Guarantee Open Enrolment Clawback | 3 | 2 |
| [`temporary-transitional-aid-guarantee`](formula-component/temporary-transitional-aid-guarantee.yml) | formula-component | Temporary Transitional Aid Guarantee | 10 | 16 |
| [`bridge-formula`](funding-regime/bridge-formula.yml) | funding-regime | Bridge Formula | 12 | 20 |
| [`equal-yield-formula`](funding-regime/equal-yield-formula.yml) | funding-regime | Equal Yield Formula | 3 | 2 |
| [`evidence-based-model`](funding-regime/evidence-based-model.yml) | funding-regime | Evidence-Based Model | 5 | 8 |
| [`fair-school-funding-plan`](funding-regime/fair-school-funding-plan.yml) | funding-regime | Fair School Funding Plan | 21 | 27 |
| [`foundation-base-cost-formula`](funding-regime/foundation-base-cost-formula.yml) | funding-regime | Foundation Base Cost Formula | 8 | 13 |
| [`academic-distress-commission`](intervention/academic-distress-commission.yml) | intervention | Academic Distress Commission | 6 | 4 |
| [`lea-level-action`](intervention/lea-level-action.yml) | intervention | Additional Optional Action (LEA level) | 4 | 2 |
| [`more-rigorous-interventions`](intervention/more-rigorous-interventions.yml) | intervention | More Rigorous Interventions (CSI) | 5 | 2 |
| [`hb-1-2009`](legislation/hb-1-2009.yml) | legislation | Am. Sub. H.B. 1 (2009) — FY2010-11 Budget; Evidence-Based Model | 4 | 7 |
| [`hb-110-2021`](legislation/hb-110-2021.yml) | legislation | Am. Sub. H.B. 110 (2021) — FY2022-23 Budget; Fair School Funding Plan | 7 | 9 |
| [`hb-119-2007`](legislation/hb-119-2007.yml) | legislation | Am. Sub. H.B. 119 (2007) — FY2008-09 Budget; Two Protections Removed | 7 | 3 |
| [`hb-153-2011`](legislation/hb-153-2011.yml) | legislation | Am. Sub. H.B. 153 (2011) — FY2012-13 Budget; Bridge Formula | 7 | 5 |
| [`hb-166-2019`](legislation/hb-166-2019.yml) | legislation | Am. Sub. H.B. 166 (2019) — FY2020-21 Budget; the Year There Was No Formula | 5 | 4 |
| [`hb-33-2023`](legislation/hb-33-2023.yml) | legislation | Am. Sub. H.B. 33 (2023) — FY2024-25 Budget | 9 | 13 |
| [`hb-49-2017`](legislation/hb-49-2017.yml) | legislation | Am. Sub. H.B. 49 (2017) — FY2018-19 Budget; the Guarantee Made Conditional | 5 | 4 |
| [`hb-583-2022`](legislation/hb-583-2022.yml) | legislation | Sub. H.B. 583 (2022) — corrective and technical changes to the Fair School Funding Plan | 10 | 3 |
| [`hb-59-2013`](legislation/hb-59-2013.yml) | legislation | Am. Sub. H.B. 59 (2013) — FY2014-15 Budget; a Formula Over the Bridge | 5 | 3 |
| [`hb-64-2015`](legislation/hb-64-2015.yml) | legislation | Am. Sub. H.B. 64 (2015) — FY2016-17 Budget; Capacity Aid | 7 | 4 |
| [`hb-66-2005`](legislation/hb-66-2005.yml) | legislation | Am. Sub. H.B. 66 (2005) — FY2006-07 Budget; Tangible Personal Property Tax Phase-Out | 4 | 7 |
| [`hb-920-1976`](legislation/hb-920-1976.yml) | legislation | Am. Sub. H.B. 920 (1976) — Tax Reduction Factors | 5 | 13 |
| [`hb-94-2001`](legislation/hb-94-2001.yml) | legislation | Am. Sub. H.B. 94 (2001) — FY2002-03 Budget; the post-DeRolph II formula | 10 | 4 |
| [`hb-95-2003`](legislation/hb-95-2003.yml) | legislation | Am. Sub. H.B. 95 (2003) — FY2004-05 Budget; the First After DeRolph | 4 | 2 |
| [`hb-96-2025`](legislation/hb-96-2025.yml) | legislation | Am. Sub. H.B. 96 (2025) — FY2026-27 Budget | 8 | 11 |
| [`ohio-constitution-article-vi-section-2`](legislation/ohio-constitution-article-vi-section-2.yml) | legislation | Ohio Constitution, Article VI, Section 2 (1851) | 2 | 5 |
| [`cincinnati-v-walter-1979`](litigation/cincinnati-v-walter-1979.yml) | litigation | Cincinnati City School District Board of Education v. Walter (1979) | 5 | 4 |
| [`derolph-i-1997`](litigation/derolph-i-1997.yml) | litigation | DeRolph v. State (DeRolph I, 1997) | 6 | 14 |
| [`derolph-ii-2000`](litigation/derolph-ii-2000.yml) | litigation | DeRolph v. State (DeRolph II, 2000) | 13 | 4 |
| [`derolph-iii-2001`](litigation/derolph-iii-2001.yml) | litigation | DeRolph v. State (DeRolph III, 2001) | 12 | 3 |
| [`derolph-iv-2002`](litigation/derolph-iv-2002.yml) | litigation | DeRolph v. State (DeRolph IV, 2002) | 10 | 6 |
| [`vouchers-hurt-ohio-2025`](litigation/vouchers-hurt-ohio-2025.yml) | litigation | EdChoice Constitutional Challenge (Franklin County, 2025) | 4 | 4 |
| [`assessed-valuation-per-pupil`](metric/assessed-valuation-per-pupil.yml) | metric | Assessed Valuation Per Pupil | 7 | 4 |
| [`effective-operating-millage`](metric/effective-operating-millage.yml) | metric | Effective Operating Millage | 7 | 1 |
| [`enrolled-adm`](metric/enrolled-adm.yml) | metric | Enrolled ADM | 8 | 8 |
| [`expenditure-per-equivalent-pupil`](metric/expenditure-per-equivalent-pupil.yml) | metric | Expenditure Per Equivalent Pupil | 8 | 6 |
| [`general-fund-cash-balance`](metric/general-fund-cash-balance.yml) | metric | General Fund Cash Balance | 4 | 3 |
| [`per-pupil-operating-expenditure`](metric/per-pupil-operating-expenditure.yml) | metric | Per-Pupil Operating Expenditure | 12 | 4 |
| [`performance-index`](metric/performance-index.yml) | metric | Performance Index | 5 | 12 |
| [`progress-value-added`](metric/progress-value-added.yml) | metric | Progress (Value-Added) | 5 | 5 |
| [`state-share-percentage`](metric/state-share-percentage.yml) | metric | State Share Percentage | 5 | 3 |
| [`education-savings-account-act`](model-policy/education-savings-account-act.yml) | model-policy | Education Savings Account Act | 4 | 1 |
| [`parental-choice-scholarship-act`](model-policy/parental-choice-scholarship-act.yml) | model-policy | Parental Choice Scholarship Program Act (Universal Eligibility) | 5 | 1 |
| [`appropriation-proration-factor`](parameter/appropriation-proration-factor.yml) | parameter | Appropriation Proration Factor | 2 | 2 |
| [`base-cost-per-pupil`](parameter/base-cost-per-pupil.yml) | parameter | Base Cost Per Pupil | 9 | 18 |
| [`fsfp-phase-in-percentage`](parameter/fsfp-phase-in-percentage.yml) | parameter | FSFP Phase-In Percentage | 4 | 13 |
| [`guarantee-funding-base`](parameter/guarantee-funding-base.yml) | parameter | Guarantee Funding Base | 6 | 5 |
| [`local-share-charge-off-millage`](parameter/local-share-charge-off-millage.yml) | parameter | Local Share Charge-Off Millage | 8 | 4 |
| [`twenty-mill-floor`](parameter/twenty-mill-floor.yml) | parameter | Twenty-Mill Floor | 4 | 8 |
| [`autism-scholarship`](program/autism-scholarship.yml) | program | Autism Scholarship | 4 | 1 |
| [`classroom-facilities-assistance`](program/classroom-facilities-assistance.yml) | program | Classroom Facilities Assistance Program | 3 | 2 |
| [`cleveland-scholarship`](program/cleveland-scholarship.yml) | program | Cleveland Scholarship and Tutoring Program | 4 | 4 |
| [`edchoice-expansion`](program/edchoice-expansion.yml) | program | EdChoice Expansion Scholarship | 5 | 10 |
| [`edchoice-scholarship`](program/edchoice-scholarship.yml) | program | Traditional EdChoice Scholarship | 4 | 3 |
| [`jon-peterson-special-needs`](program/jon-peterson-special-needs.yml) | program | Jon Peterson Special Needs Scholarship | 4 | 1 |
| [`casino-tax-distribution`](revenue-stream/casino-tax-distribution.yml) | revenue-stream | Casino Tax — County Student Fund | 6 | 1 |
| [`esser`](revenue-stream/esser.yml) | revenue-stream | ESSER — Elementary and Secondary School Emergency Relief | 4 | 2 |
| [`idea-part-b`](revenue-stream/idea-part-b.yml) | revenue-stream | IDEA Part B | 5 | 3 |
| [`local-property-tax`](revenue-stream/local-property-tax.yml) | revenue-stream | Local Property Tax | 6 | 8 |
| [`lottery-profits`](revenue-stream/lottery-profits.yml) | revenue-stream | Lottery Profits Education Fund | 4 | 1 |
| [`state-foundation-aid`](revenue-stream/state-foundation-aid.yml) | revenue-stream | State Foundation Aid | 5 | 21 |
| [`title-i`](revenue-stream/title-i.yml) | revenue-stream | Title I, Part A | 4 | 5 |
| [`tpp-replacement-payments`](revenue-stream/tpp-replacement-payments.yml) | revenue-stream | Tangible Personal Property Tax Replacement Payments | 6 | 7 |
| [`fsfp-input-year-refresh`](scenario/fsfp-input-year-refresh.yml) | scenario | FSFP Cost Input Refresh vs. Freeze | 9 | 7 |
| [`guarantee-phase-out`](scenario/guarantee-phase-out.yml) | scenario | Phasing Out the Temporary Transitional Aid Guarantee | 11 | 3 |
| [`anton-grdina`](school/anton-grdina.yml) | school | Anton Grdina | 6 | 1 |
| [`barrington-road-elementary`](school/barrington-road-elementary.yml) | school | Barrington Road Elementary School | 4 | 2 |
| [`sheridan-high-school`](school/sheridan-high-school.yml) | school | Sheridan High School | 6 | 1 |

117 nodes across 18 classes, and **2 with nothing pointing at them** — counting a citation in somebody's prose as pointing. Whether that is a gap depends on the class: `web/tests/unit/reachability.spec.ts` holds every node to having an inbound *edge* and exempts `draft-legislation`, where a node with nothing pointing at it is the design.
<!-- /REGEN -->

## Semantic index status

<!-- REGEN: edfund-connect index-status
Regenerated by: `edfund-connect index`
Fields: total nodes indexed, embedding model, index freshness (last indexed commit vs HEAD),
        stale node count.
-->
No semantic index is built. The corpus is 117 nodes and fits in context; an index is added when direct retrieval stops working, which has not happened.
<!-- /REGEN -->
