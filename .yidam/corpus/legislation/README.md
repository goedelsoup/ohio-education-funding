# legislation

The enacted instruments that built Ohio's school funding system — General Assembly bills,
almost always biennial budget acts, plus the constitutional provision the whole system is
measured against. Ohio does not legislate school funding as standalone policy; the formula
lives inside the operating budget, which is why the nodes here are mostly bill numbers that
also carry Medicaid, corrections, and highway appropriations. Reading a funding change out of
one of these acts means reading a few dozen sections out of several thousand pages.

Constitutional provisions sit in this class rather than in one of their own because they
occupy the same structural position: an authority that establishes, constrains, and gets
litigated against. Article VI, Section 2 is the target of every case in
[`litigation/`](../litigation/), and treating it as a separate class would have duplicated
every edge that already runs to legislation.

See the class definition: [legislation.ont.yml](../legislation.ont.yml).

## Instances

| Node | Year | What it did |
|------|------|-------------|
| [ohio-constitution-article-vi-section-2](ohio-constitution-article-vi-section-2.yml) | 1851 | Imposed the thorough-and-efficient duty |
| [hb-920-1976](hb-920-1976.yml) | 1976 | Froze the dollar yield of voted millage against inflation |
| [hb-94-2001](hb-94-2001.yml) | 2001 | Rebuilt the formula after DeRolph II; parity aid, gap aid, six special education weights |
| [hb-95-2003](hb-95-2003.yml) | 2003 | First budget after DeRolph ended; changed nothing structural |
| [hb-66-2005](hb-66-2005.yml) | 2005 | Phased out the tangible personal property tax |
| [hb-119-2007](hb-119-2007.yml) | 2007 | Abolished the cost of doing business factor and the base cost guarantee |
| [hb-1-2009](hb-1-2009.yml) | 2009 | Enacted the Evidence-Based Model |
| [hb-153-2011](hb-153-2011.yml) | 2011 | Repealed the EBM; installed the Bridge formula |
| [hb-59-2013](hb-59-2013.yml) | 2013 | Built a formula over the Bridge guarantee; anchored it at FY2013 |
| [hb-64-2015](hb-64-2015.yml) | 2015 | Added capacity aid and performance bonuses; re-anchored to FY2015 |
| [hb-49-2017](hb-49-2017.yml) | 2017 | Re-anchored to FY2017 and made the guarantee conditional on enrolment loss |
| [hb-166-2019](hb-166-2019.yml) | 2019 | Ran no formula: froze every district at its FY2019 allocation |
| [hb-110-2021](hb-110-2021.yml) | 2021 | Enacted the Fair School Funding Plan |
| [hb-33-2023](hb-33-2023.yml) | 2023 | Refreshed cost inputs to FY2022; made EdChoice universal |
| [hb-96-2025](hb-96-2025.yml) | 2025 | Phase-in to 100%, cost inputs held at FY2022 |

Read the last three in sequence and the argument of the current era is visible: H.B. 110 built
a formula that prices staffing at a reference year, H.B. 33 proved the reference year can be
refreshed, and H.B. 96 declined to refresh it while completing the phase-in. Freezing was a
choice against an established practice, not an oversight.

Read the four before them in sequence and the practice being established is visible too. Each act
of the Bridge decade re-read the guarantee's anchor and left the mechanism alone — FY2013, then
FY2015, then FY2017 — until H.B. 166 stopped computing altogether and paid every district its
FY2019 amount. The Fair School Funding Plan's `[H2]` guarantee base is the far end of that chain,
which is why the four nodes carry it and [`bridge-formula`](../funding-regime/bridge-formula.yml)
sets it out in one place.

## Known gaps

**The measure is budget lines, not acts.** The Catalog of Budget Line Items records the act that
established each line, and
[`catalog-line-item-basis.tsv`](../../../crates/project/fixtures/catalog-line-item-basis.tsv)
carries 2,416 such citations across 45 acts. **71.5% of them now reach a node**, against 33.7%
before the Bridge decade was written and 65.3% after it. `web/tests/unit/legislationCoverage.spec.ts` holds that share
as a floor and names every remaining act above thirty citations with what it would take — the list
is short enough to read and specific enough to work from.

**The two cheap ones are written and the rest are behind a wall.** H.B. 95 and H.B. 119 had
greenbooks already retrieved and digest-pinned; the nodes were the only thing missing and they took
one phase. Of the seven still above thirty citations, only **H.B. 282 of the 123rd** and **H.B. 650
of the 122nd** are reachable at all — both are enrolled acts `ohio-session-laws` already pins for
their appropriation tables, and writing them means reading the act rather than the table, which is
the judgement step `ohio-bills` deliberately does not automate. Below the 122nd General Assembly
the legislature serves nothing in any form, so H.B. 152 of the 120th, H.B. 191 of the 112th,
H.B. 204 of the 113th, H.B. 238 of the 116th and H.B. 111 of the 118th need a records request or a
library rather than a phase. That is [issue #18](https://github.com/goedelsoup/ohio-education-funding/issues/18).

**And the pre-2005 nodes cannot be sourced the way the later ones are.** `ohio-laws` serves the
Revised Code as it stands today, which is the wrong document for an act from 2001. Every section
H.B. 94 is cited by has since been repealed or reused — R.C. 3317.012 now reads "joint vocational
school district base cost", R.C. 3317.0217 now reads "targeted assistance funding" — so following
a contemporary citation to the current code silently lands on a different programme. Nodes for
that era rest on secondary recitation. [open]

**[H.B. 583](hb-583-2022.yml) has a node**, though what it changed does not: LSC's final analysis
for it is a separate document `lsc-budget` does not retrieve, so the corpus can say the corrections
are operative and not which provisions moved. [open]

**No node here is sourced for its own enactment date.** Every `signed:` and `effective:` in this
class is stated from general knowledge; the greenbooks date the *analysis*, months after the fact,
which is what `published_on` reads and puts in the fixture's `date:` field. The acts themselves
carry their dates and only four of them are retrievable. [open]

## What the vetoes say, which the enacted figures cannot

Every act here whose greenbook exists now records its own vetoes — twelve consecutive General
Assemblies, the 124th to the 135th, plus the 136th from its own file. Read across them they are not
a miscellany. **The General Assembly kept legislating floors, loss protections and oversight into
budget acts, and the Governor kept striking exactly those.**

| Act | What was passed | What survived |
|---|---|---|
| [H.B. 66](hb-66-2005.yml) | A **levy indexed to lost state aid**, to counteract H.B. 920's limit on revenue growth | Vetoed |
| [H.B. 1](hb-1-2009.yml) | **Full TPP reimbursement in perpetuity**, ending the phase-down | Vetoed past FY2013 |
| [H.B. 1](hb-1-2009.yml) | EdChoice maximums | Partially vetoed **downward**, to FY2007 levels |
| [H.B. 59](hb-59-2013.yml) | A requirement that **gifted funding buy gifted staff** | Vetoed |
| [H.B. 64](hb-64-2015.yml) | A **second per-pupil floor** — at least 20% of the formula amount per pupil | Vetoed |
| [H.B. 64](hb-64-2015.yml) | The **TPP supplement's second year**, $78.3m for FY2017 | Vetoed |
| [H.B. 49](hb-49-2017.yml) | A **cap on TPP reimbursement loss** at 3.5% of total resources | Vetoed |
| [H.B. 166](hb-166-2019.yml) | A **per-pupil guarantee from FY2022**, indexed to per-pupil nonpublic spending | Vetoed |
| [H.B. 110](hb-110-2021.yml) | **JCARR review of EMIS** and of community school business rules | Vetoed |
| [H.B. 33](hb-33-2023.yml) | **JCARR review of the community school FTE manual** | Vetoed |

Three shapes recur. **Protection against the tangible personal property phase-out** was passed and
struck three times, in 2009, 2015 and 2017, and never survived once — see
[`tpp-replacement-payments`](../revenue-stream/tpp-replacement-payments.yml). **A floor under
per-pupil funding** was passed and struck twice. And **legislative review of the department's
community school accounting** was passed and struck twice in consecutive budgets, the second time
aimed at the manual that decides what a community school is paid.

Running the other way, four separate new channels to nonpublic schools were passed and vetoed —
H.B. 119's Special Education Scholarship Pilot, H.B. 1's EdChoice eligibility route, H.B. 110's
conditional-approval scholarships and H.B. 96's Nonchartered Educational Savings Accounts.

None of this is in any series this repository holds, and it cannot be. A vetoed provision is never
appropriated, so it appears in no enacted total, no actual, and no line item — the whole apparatus
here reads documents that record what happened. The $78.3 million that did not reach districts in
FY2017 is not a gap in the data; it is a quantity with no place in the data. The only committed
source in which any of it exists is LSC's analysis of the act that contained it.

That is the argument for reading acts and not only the money they moved. It also puts a third thing
beside the two the corpus already models: `legislation` is what became law and
[`draft-legislation`](../draft-legislation/) is what was proposed and did not. A vetoed provision is
neither — passed by both chambers and struck by one person. It is recorded in the `vetoes:` field of
the act that carried it rather than given a class, because it has no life of its own: it exists only
as a thing a particular act tried to do. [open — whether that stays the right shape if the field
keeps growing]

**Two nodes have no greenbook and are exempt by construction.** H.B. 920 of 1976 and the
constitutional provision predate the series entirely. **H.B. 94's greenbook exists and is silent** —
alone in the twelve it carries no vetoed-provisions chapter, so its field records that rather than a
placeholder, which is a different claim and the ratchet in
`web/tests/unit/legislationCoverage.spec.ts` tells them apart.
