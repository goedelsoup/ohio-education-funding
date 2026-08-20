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
| [hb-66-2005](hb-66-2005.yml) | 2005 | Phased out the tangible personal property tax |
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
carries 2,416 such citations across 45 acts. **65.3% of them now reach a node**, against 33.7%
before the Bridge decade was written. `web/tests/unit/legislationCoverage.spec.ts` holds that share
as a floor and names every remaining act above thirty citations with what it would take — the list
is short enough to read and specific enough to work from.

**Two of the remaining nine are cheap and the rest are behind a wall.** H.B. 119 of the 127th
(FY2008-09) and H.B. 95 of the 125th (FY2004-05) have greenbooks already retrieved and digest-
pinned; only the nodes are missing. Below the 122nd General Assembly the legislature serves nothing
in any form — its own version index stops there — so H.B. 152 of the 120th, H.B. 191 of the 112th,
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

**What no act node here records is its vetoes.** Ohio budget acts are line-item vetoed and the
vetoes have repeatedly touched education funding; every node in this class carries the same `[open]`
in its `vetoes:` field. It matters most on [H.B. 166](hb-166-2019.yml), whose greenbook has a
`Vetoed provisions` chapter: a veto inside a freeze changes what is frozen. [open]
